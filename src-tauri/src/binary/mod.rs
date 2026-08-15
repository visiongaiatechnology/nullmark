// STATUS: DIAMANT VGT SUPREME

mod document;
mod jpeg;
mod model;
mod pdf;
mod png;
mod svg;
mod webp;

pub use model::{BinaryFinding, BinaryFormat, BinaryResult, CleanedBinary, ParsedBinary};

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

pub fn detect_format(bytes: &[u8]) -> BinaryResult<BinaryFormat> {
    if bytes.starts_with(PNG_SIGNATURE) {
        return Ok(BinaryFormat::Png);
    }
    if bytes.starts_with(&[0xFF, 0xD8]) {
        return Ok(BinaryFormat::Jpeg);
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(BinaryFormat::WebP);
    }
    if bytes.starts_with(b"PK\x03\x04") {
        return document::detect(bytes);
    }
    if bytes.starts_with(b"%PDF-") {
        return pdf::detect(bytes);
    }
    if svg::is_candidate(bytes) {
        return svg::detect(bytes);
    }
    Err("Unsupported or mismatched binary file signature.")
}

pub fn analyze(bytes: &[u8]) -> BinaryResult<ParsedBinary> {
    match detect_format(bytes)? {
        BinaryFormat::Png => png::analyze(bytes),
        BinaryFormat::Jpeg => jpeg::analyze(bytes),
        BinaryFormat::WebP => webp::analyze(bytes),
        BinaryFormat::Docx | BinaryFormat::Xlsx | BinaryFormat::Pptx | BinaryFormat::Odt => {
            document::analyze(bytes)
        }
        BinaryFormat::Svg => svg::analyze(bytes),
        BinaryFormat::Pdf => pdf::analyze(bytes),
    }
}

pub fn sanitize(bytes: &[u8]) -> BinaryResult<CleanedBinary> {
    match detect_format(bytes)? {
        BinaryFormat::Png => png::sanitize(bytes),
        BinaryFormat::Jpeg => jpeg::sanitize(bytes),
        BinaryFormat::WebP => webp::sanitize(bytes),
        BinaryFormat::Docx | BinaryFormat::Xlsx | BinaryFormat::Pptx | BinaryFormat::Odt => {
            document::sanitize(bytes)
        }
        BinaryFormat::Svg => svg::sanitize(bytes),
        BinaryFormat::Pdf => pdf::sanitize(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crc32fast::Hasher;
    use lopdf::{dictionary, Document, Object, Stream};
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    fn zip_container(entries: &[(&str, &[u8], CompressionMethod)]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data, compression) in entries {
            let options = SimpleFileOptions::default()
                .compression_method(*compression)
                .unix_permissions(0o600);
            writer
                .start_file(*name, options)
                .expect("synthetic ZIP entry must start");
            writer
                .write_all(data)
                .expect("synthetic ZIP entry must write");
        }
        writer
            .finish()
            .expect("synthetic ZIP must finish")
            .into_inner()
    }

    fn png_chunk(kind: &[u8; 4], data: &[u8], output: &mut Vec<u8>) {
        output.extend_from_slice(&(data.len() as u32).to_be_bytes());
        output.extend_from_slice(kind);
        output.extend_from_slice(data);
        let mut hasher = Hasher::new();
        hasher.update(kind);
        hasher.update(data);
        output.extend_from_slice(&hasher.finalize().to_be_bytes());
    }

    fn marked_png() -> Vec<u8> {
        let mut output = PNG_SIGNATURE.to_vec();
        let mut header = [0u8; 13];
        header[3] = 1;
        header[7] = 1;
        header[8] = 8;
        header[9] = 6;
        png_chunk(b"IHDR", &header, &mut output);
        png_chunk(b"tEXt", b"Author\0NullMark", &mut output);
        png_chunk(b"caBX", b"c2pa", &mut output);
        png_chunk(b"IDAT", b"synthetic", &mut output);
        png_chunk(b"IEND", b"", &mut output);
        output
    }

    fn marked_jpeg() -> Vec<u8> {
        let mut output = vec![0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x08];
        output.extend_from_slice(b"Exif\0\0");
        output.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x08, 1, 1, 0, 0, 0, 0]);
        output.extend_from_slice(&[0x11, 0x22, 0xFF, 0xD9]);
        output
    }

    fn append_jpeg_segment(output: &mut Vec<u8>, marker: u8, payload: &[u8]) {
        output.extend_from_slice(&[0xFF, marker]);
        output.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        output.extend_from_slice(payload);
    }

    fn webp_chunk(kind: &[u8; 4], data: &[u8], output: &mut Vec<u8>) {
        output.extend_from_slice(kind);
        output.extend_from_slice(&(data.len() as u32).to_le_bytes());
        output.extend_from_slice(data);
        if data.len() & 1 == 1 {
            output.push(0);
        }
    }

    fn marked_webp() -> Vec<u8> {
        let mut output = b"RIFF\0\0\0\0WEBP".to_vec();
        webp_chunk(
            b"VP8X",
            &[0b0000_1100, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &mut output,
        );
        webp_chunk(b"VP8 ", b"synthetic", &mut output);
        webp_chunk(b"EXIF", b"private", &mut output);
        webp_chunk(b"XMP ", b"private", &mut output);
        webp_chunk(b"C2PA", b"provenance", &mut output);
        let size = (output.len() - 8) as u32;
        output[4..8].copy_from_slice(&size.to_le_bytes());
        output
    }

    fn marked_pdf() -> Vec<u8> {
        let mut document = Document::with_version("1.7");
        let pages = document.add_object(dictionary! {
            "Type" => "Pages",
            "Kids" => Vec::<Object>::new(),
            "Count" => 0,
        });
        let metadata = document.add_object(Stream::new(
            dictionary! {
                "Type" => "Metadata",
                "Subtype" => "XML",
            },
            b"<x:xmpmeta>private-xmp</x:xmpmeta>".to_vec(),
        ));
        let c2pa_stream = document.add_object(Stream::new(
            dictionary! { "Subtype" => "application/c2pa" },
            b"private-c2pa-manifest".to_vec(),
        ));
        let c2pa_filespec = document.add_object(dictionary! {
            "Type" => "Filespec",
            "AFRelationship" => "C2PA_Manifest",
            "EF" => dictionary! { "F" => c2pa_stream },
        });
        let catalog = document.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages,
            "Metadata" => metadata,
            "AF" => vec![Object::Reference(c2pa_filespec)],
        });
        let info = document.add_object(dictionary! {
            "Author" => Object::string_literal("Alice"),
            "Creator" => Object::string_literal("NullMark test"),
        });
        document.trailer.set("Root", catalog);
        document.trailer.set("Info", info);
        let mut output = Vec::new();
        document
            .save_to(&mut output)
            .expect("synthetic PDF must serialize");
        output
    }

    fn deep_privacy_pdf() -> Vec<u8> {
        let mut document = Document::with_version("1.7");
        let pages = document.add_object(dictionary! {
            "Type" => "Pages",
            "Kids" => Vec::<Object>::new(),
            "Count" => 0,
        });
        document.add_object(Stream::new(
            dictionary! { "Type" => "Metadata", "Subtype" => "XML" },
            b"<x:xmpmeta>page-private-xmp</x:xmpmeta>".to_vec(),
        ));
        document.add_object(dictionary! {
            "Type" => "Annot",
            "Subtype" => "Text",
            "T" => Object::string_literal("Alice"),
            "M" => Object::string_literal("D:20260815"),
            "CreationDate" => Object::string_literal("D:20260814"),
            "NM" => Object::string_literal("private-annotation-id"),
            "Contents" => Object::string_literal("Keep this visible note"),
        });
        let javascript = document.add_object(dictionary! {
            "S" => "JavaScript",
            "JS" => Object::string_literal("app.alert('tracking')"),
        });
        let catalog = document.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages,
            "OpenAction" => javascript,
        });
        document.trailer.set("Root", catalog);
        let mut output = Vec::new();
        document
            .save_to(&mut output)
            .expect("deep PDF must serialize");
        output
    }

    #[test]
    fn png_cleaning_removes_metadata_and_c2pa_then_verifies() {
        let input = marked_png();
        let before = analyze(&input).expect("synthetic PNG must parse");
        assert_eq!(before.metadata_count(), 2);
        assert!(before.c2pa_detected);
        let cleaned = sanitize(&input).expect("synthetic PNG must sanitize");
        let after = analyze(&cleaned.bytes).expect("sanitized PNG must reparse");
        assert_eq!(cleaned.removed_items, 2);
        assert_eq!(after.metadata_count(), 0);
        assert!(!after.c2pa_detected);
    }

    #[test]
    fn png_crc_corruption_is_rejected() {
        let mut input = marked_png();
        input[29] ^= 1;
        assert!(analyze(&input).is_err());
    }

    #[test]
    fn jpeg_cleaning_preserves_scan_and_removes_exif() {
        let input = marked_jpeg();
        assert_eq!(
            analyze(&input).expect("JPEG must parse").metadata_count(),
            1
        );
        let cleaned = sanitize(&input).expect("JPEG must sanitize");
        assert_eq!(
            analyze(&cleaned.bytes)
                .expect("JPEG must reparse")
                .metadata_count(),
            0
        );
        assert!(cleaned.bytes.ends_with(&[0x11, 0x22, 0xFF, 0xD9]));
    }

    #[test]
    fn jpeg_removes_post_scan_metadata_and_all_c2pa_fragments() {
        let mut input = vec![0xFF, 0xD8];
        append_jpeg_segment(&mut input, 0xEB, b"c2pa first fragment");
        append_jpeg_segment(&mut input, 0xEB, b"continuation");
        append_jpeg_segment(&mut input, 0xDA, &[1, 1, 0, 0, 0, 0]);
        input.extend_from_slice(&[0x11, 0x22]);
        append_jpeg_segment(&mut input, 0xFE, b"post-scan comment");
        input.extend_from_slice(&[0xFF, 0xD9]);
        let before = analyze(&input).expect("multi-segment JPEG must parse");
        assert_eq!(before.metadata_count(), 3);
        assert!(before.c2pa_detected);
        let cleaned = sanitize(&input).expect("multi-segment JPEG must sanitize");
        let after = analyze(&cleaned.bytes).expect("cleaned JPEG must reparse");
        assert_eq!(after.metadata_count(), 0);
        assert!(!after.c2pa_detected);
    }

    #[test]
    fn webp_cleaning_clears_metadata_flags_and_rewrites_riff_size() {
        let input = marked_webp();
        assert_eq!(
            analyze(&input).expect("WebP must parse").metadata_count(),
            3
        );
        let cleaned = sanitize(&input).expect("WebP must sanitize");
        assert_eq!(
            analyze(&cleaned.bytes)
                .expect("WebP must reparse")
                .metadata_count(),
            0
        );
        assert_eq!(cleaned.bytes[20] & 0b0000_1100, 0);
        assert!(
            !analyze(&cleaned.bytes)
                .expect("WebP must verify")
                .c2pa_detected
        );
        assert_eq!(
            u32::from_le_bytes(cleaned.bytes[4..8].try_into().expect("RIFF size")) as usize + 8,
            cleaned.bytes.len()
        );
    }

    #[test]
    fn docx_cleaning_removes_properties_revision_authorship_and_unicode() {
        let content_types = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#;
        let document = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><w:document xmlns:w=\"urn:w\"><w:body><w:ins w:author=\"Alice\" w:date=\"2026-01-01\"><w:t>A\u{200B}B</w:t></w:ins></w:body></w:document>";
        let core = br#"<?xml version="1.0"?><cp:coreProperties xmlns:cp="urn:cp"><dc:creator xmlns:dc="urn:dc">Alice</dc:creator></cp:coreProperties>"#;
        let input = zip_container(&[
            (
                "[Content_Types].xml",
                content_types,
                CompressionMethod::Deflated,
            ),
            (
                "word/document.xml",
                document.as_bytes(),
                CompressionMethod::Deflated,
            ),
            ("docProps/core.xml", core, CompressionMethod::Deflated),
            (
                "META-INF/content_credential.c2pa",
                b"provenance",
                CompressionMethod::Stored,
            ),
        ]);

        let before = analyze(&input).expect("DOCX must parse");
        assert_eq!(before.format, BinaryFormat::Docx);
        assert_eq!(before.metadata_count(), 5);
        assert!(before.c2pa_detected);
        let cleaned = sanitize(&input).expect("DOCX must sanitize");
        let after = analyze(&cleaned.bytes).expect("cleaned DOCX must reopen");
        assert_eq!(after.metadata_count(), 0);
        assert_eq!(cleaned.removed_items, 5);
        assert!(!after.c2pa_detected);
    }

    #[test]
    fn odt_cleaning_preserves_container_contract_and_removes_private_metadata() {
        let mimetype = b"application/vnd.oasis.opendocument.text";
        let content = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><office:document-content xmlns:office=\"urn:office\"><office:body>A\u{2060}B</office:body></office:document-content>";
        let meta = br#"<?xml version="1.0"?><office:document-meta xmlns:office="urn:office"><office:meta>private</office:meta></office:document-meta>"#;
        let input = zip_container(&[
            ("mimetype", mimetype, CompressionMethod::Stored),
            (
                "content.xml",
                content.as_bytes(),
                CompressionMethod::Deflated,
            ),
            ("meta.xml", meta, CompressionMethod::Deflated),
            (
                "META-INF/content_credential.c2pa",
                b"provenance",
                CompressionMethod::Stored,
            ),
        ]);

        let before = analyze(&input).expect("ODT must parse");
        assert_eq!(before.format, BinaryFormat::Odt);
        assert_eq!(before.metadata_count(), 3);
        assert!(before.c2pa_detected);
        let cleaned = sanitize(&input).expect("ODT must sanitize");
        let after = analyze(&cleaned.bytes).expect("cleaned ODT must reopen");
        assert_eq!(after.metadata_count(), 0);
        assert_eq!(detect_format(&cleaned.bytes), Ok(BinaryFormat::Odt));
        assert!(!after.c2pa_detected);
    }

    #[test]
    fn xlsx_cleaning_scrubs_properties_comments_shared_strings_and_c2pa() {
        let input = zip_container(&[
            (
                "[Content_Types].xml",
                b"<Types/>",
                CompressionMethod::Deflated,
            ),
            (
                "xl/workbook.xml",
                b"<workbook/>",
                CompressionMethod::Deflated,
            ),
            (
                "xl/sharedStrings.xml",
                "<sst><si><t>A\u{200B}B</t></si></sst>".as_bytes(),
                CompressionMethod::Deflated,
            ),
            (
                "xl/comments1.xml",
                b"<comments author=\"Alice\" date=\"2026-01-01\"/>",
                CompressionMethod::Deflated,
            ),
            (
                "docProps/core.xml",
                b"<private>Alice</private>",
                CompressionMethod::Deflated,
            ),
            (
                "META-INF/content_credential.c2pa",
                b"provenance",
                CompressionMethod::Stored,
            ),
        ]);
        let before = analyze(&input).expect("XLSX must parse");
        assert_eq!(before.format, BinaryFormat::Xlsx);
        assert_eq!(before.metadata_count(), 5);
        let cleaned = sanitize(&input).expect("XLSX must sanitize");
        let after = analyze(&cleaned.bytes).expect("cleaned XLSX must reopen");
        assert_eq!(after.metadata_count(), 0);
        assert!(!after.c2pa_detected);
    }

    #[test]
    fn pptx_cleaning_scrubs_properties_authors_slide_unicode_and_c2pa() {
        let input = zip_container(&[
            (
                "[Content_Types].xml",
                b"<Types/>",
                CompressionMethod::Deflated,
            ),
            (
                "ppt/presentation.xml",
                b"<presentation/>",
                CompressionMethod::Deflated,
            ),
            (
                "ppt/slides/slide1.xml",
                "<slide><text>A\u{2060}B</text></slide>".as_bytes(),
                CompressionMethod::Deflated,
            ),
            (
                "ppt/commentAuthors.xml",
                b"<authors><author author=\"Alice\" initials=\"A\"/></authors>",
                CompressionMethod::Deflated,
            ),
            (
                "docProps/app.xml",
                b"<private>Office</private>",
                CompressionMethod::Deflated,
            ),
            (
                "META-INF/content_credential.c2pa",
                b"provenance",
                CompressionMethod::Stored,
            ),
        ]);
        let before = analyze(&input).expect("PPTX must parse");
        assert_eq!(before.format, BinaryFormat::Pptx);
        assert_eq!(before.metadata_count(), 5);
        let cleaned = sanitize(&input).expect("PPTX must sanitize");
        let after = analyze(&cleaned.bytes).expect("cleaned PPTX must reopen");
        assert_eq!(after.metadata_count(), 0);
        assert!(!after.c2pa_detected);
    }

    #[test]
    fn svg_cleaning_removes_metadata_active_content_external_links_and_unicode() {
        let input = "<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\"><metadata>private</metadata><!--trace--><script>alert(1)</script><a href=\"https://tracker.invalid\" onclick=\"run()\"><text>A\u{200B}B</text></a><path d=\"M0 0h1v1z\"/></svg>".as_bytes();
        let before = analyze(input).expect("SVG must parse");
        assert_eq!(before.format, BinaryFormat::Svg);
        assert_eq!(before.metadata_count(), 6);
        let cleaned = sanitize(input).expect("SVG must sanitize");
        let after = analyze(&cleaned.bytes).expect("cleaned SVG must reopen");
        assert_eq!(after.metadata_count(), 0);
        assert!(!String::from_utf8_lossy(&cleaned.bytes).contains("tracker.invalid"));
        assert!(String::from_utf8_lossy(&cleaned.bytes).contains("<path"));
    }

    #[test]
    fn svg_rejects_document_type_declarations() {
        assert!(analyze(br#"<!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg"/>"#).is_err());
    }

    #[test]
    fn document_container_rejects_path_traversal_names() {
        let input = zip_container(&[
            (
                "[Content_Types].xml",
                b"<Types/>",
                CompressionMethod::Stored,
            ),
            (
                "word/document.xml",
                b"<document/>",
                CompressionMethod::Stored,
            ),
            ("../outside.xml", b"<unsafe/>", CompressionMethod::Stored),
        ]);
        assert!(analyze(&input).is_err());
    }

    #[test]
    fn odt_rejects_noncanonical_mimetype_layout() {
        let input = zip_container(&[
            ("content.xml", b"<content/>", CompressionMethod::Stored),
            (
                "mimetype",
                b"application/vnd.oasis.opendocument.text",
                CompressionMethod::Deflated,
            ),
        ]);
        assert!(analyze(&input).is_err());
    }

    #[test]
    fn pdf_cleaning_removes_info_and_xmp_then_reopens() {
        let input = marked_pdf();
        let before = analyze(&input).expect("PDF must parse");
        assert_eq!(before.format, BinaryFormat::Pdf);
        assert_eq!(before.metadata_count(), 3);
        assert!(before.c2pa_detected);
        let cleaned = sanitize(&input).expect("PDF must sanitize");
        let after = analyze(&cleaned.bytes).expect("cleaned PDF must reopen");
        assert_eq!(after.metadata_count(), 0);
        assert_eq!(cleaned.removed_items, 3);
        assert!(!after.c2pa_detected);
        assert!(!cleaned.bytes.windows(5).any(|window| window == b"Alice"));
        assert!(!cleaned
            .bytes
            .windows(b"private-xmp".len())
            .any(|window| window == b"private-xmp"));
        assert!(!cleaned
            .bytes
            .windows(b"private-c2pa-manifest".len())
            .any(|window| window == b"private-c2pa-manifest"));
    }

    #[test]
    fn pdf_deep_cleaning_removes_page_xmp_annotation_identity_and_javascript() {
        let input = deep_privacy_pdf();
        let before = analyze(&input).expect("deep PDF must parse");
        assert!(before
            .findings
            .iter()
            .any(|finding| finding.kind == "pdf-metadata"));
        assert!(before
            .findings
            .iter()
            .any(|finding| finding.kind == "pdf-annotation-privacy"));
        assert!(before
            .findings
            .iter()
            .any(|finding| finding.kind == "pdf-active-content"));
        let cleaned = sanitize(&input).expect("deep PDF must sanitize");
        let after = analyze(&cleaned.bytes).expect("deep cleaned PDF must reopen");
        assert_eq!(after.metadata_count(), 0);
        assert!(!cleaned.bytes.windows(5).any(|window| window == b"Alice"));
        assert!(!cleaned.bytes.windows(8).any(|window| window == b"tracking"));
        assert!(cleaned
            .bytes
            .windows(b"Keep this visible note".len())
            .any(|window| window == b"Keep this visible note"));
    }
}
