// STATUS: DIAMANT VGT SUPREME

use super::model::{BinaryFinding, BinaryFormat, BinaryResult, CleanedBinary, ParsedBinary};
use crate::engine::{self, Action, Mode};
use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::collections::HashSet;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

const MAX_ENTRIES: usize = 4096;
const MAX_ENTRY_BYTES: u64 = 32 * 1024 * 1024;
const MAX_EXPANDED_BYTES: u128 = 128 * 1024 * 1024;
const DOCX_CORE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"/>"#;
const DOCX_CUSTOM: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties"/>"#;
const DOCX_APP: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"/>"#;
const ODT_META: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"><office:meta/></office:document-meta>"#;
const ODT_RDF: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/>"#;
const C2PA_ENTRY: &str = "META-INF/content_credential.c2pa";

fn zip_error(_: zip::result::ZipError) -> &'static str {
    "Document ZIP validation failed."
}
fn io_error(_: std::io::Error) -> &'static str {
    "Document container I/O failed."
}

fn open(bytes: &[u8]) -> BinaryResult<ZipArchive<Cursor<&[u8]>>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(zip_error)?;
    if archive.offset() != 0 || archive.is_empty() || archive.len() > MAX_ENTRIES {
        return Err("Document ZIP entry boundary rejected.");
    }
    if !matches!(archive.decompressed_size(), Some(size) if size <= MAX_EXPANDED_BYTES) {
        return Err("Document decompression boundary rejected.");
    }
    if archive.has_overlapping_files().map_err(zip_error)? {
        return Err("Overlapping ZIP entries rejected.");
    }
    Ok(archive)
}

fn validate_entries(archive: &mut ZipArchive<Cursor<&[u8]>>) -> BinaryResult<()> {
    let mut names = HashSet::with_capacity(archive.len());
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(zip_error)?;
        let name = file.name();
        if file.enclosed_name().is_none() || name.contains('\\') || !names.insert(name.to_owned()) {
            return Err("Unsafe or duplicate document entry name rejected.");
        }
        if file.encrypted() || file.is_symlink() || file.size() > MAX_ENTRY_BYTES {
            return Err("Encrypted, linked or oversized document entry rejected.");
        }
        if !matches!(
            file.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err("Unsupported document compression method.");
        }
        if name == C2PA_ENTRY && file.compression() != CompressionMethod::Stored {
            return Err("Compressed C2PA document manifest rejected.");
        }
    }
    Ok(())
}

fn read_entry(archive: &mut ZipArchive<Cursor<&[u8]>>, index: usize) -> BinaryResult<Vec<u8>> {
    let file = archive.by_index(index).map_err(zip_error)?;
    let mut data = Vec::with_capacity(usize::try_from(file.size()).unwrap_or(0));
    file.take(MAX_ENTRY_BYTES + 1)
        .read_to_end(&mut data)
        .map_err(io_error)?;
    if data.len() as u64 > MAX_ENTRY_BYTES {
        return Err("Expanded document entry limit exceeded.");
    }
    Ok(data)
}

pub fn detect(bytes: &[u8]) -> BinaryResult<BinaryFormat> {
    let mut archive = open(bytes)?;
    validate_entries(&mut archive)?;
    let mut content_types = false;
    let mut docx_document = false;
    let mut xlsx_workbook = false;
    let mut pptx_presentation = false;
    let mut odt_mimetype = false;
    for index in 0..archive.len() {
        let name = archive
            .name_for_index(index)
            .ok_or("Document entry name unavailable.")?;
        match name {
            "[Content_Types].xml" => content_types = true,
            "word/document.xml" => docx_document = true,
            "xl/workbook.xml" => xlsx_workbook = true,
            "ppt/presentation.xml" => pptx_presentation = true,
            "mimetype" => {
                let valid_layout = {
                    let file = archive.by_index(index).map_err(zip_error)?;
                    index == 0 && file.compression() == CompressionMethod::Stored
                };
                odt_mimetype = valid_layout
                    && read_entry(&mut archive, index)?
                        == b"application/vnd.oasis.opendocument.text";
            }
            _ => {}
        }
    }
    let ooxml = [docx_document, xlsx_workbook, pptx_presentation]
        .into_iter()
        .filter(|present| *present)
        .count();
    if odt_mimetype && !content_types && ooxml == 0 {
        return Ok(BinaryFormat::Odt);
    }
    if content_types && !odt_mimetype && ooxml == 1 {
        return Ok(if docx_document {
            BinaryFormat::Docx
        } else if xlsx_workbook {
            BinaryFormat::Xlsx
        } else {
            BinaryFormat::Pptx
        });
    }
    Err("ZIP container is not an unambiguous supported Office document.")
}

fn canonical_metadata(format: BinaryFormat, name: &str) -> Option<&'static [u8]> {
    match (format, name) {
        (BinaryFormat::Docx | BinaryFormat::Xlsx | BinaryFormat::Pptx, "docProps/core.xml") => {
            Some(DOCX_CORE)
        }
        (BinaryFormat::Docx | BinaryFormat::Xlsx | BinaryFormat::Pptx, "docProps/custom.xml") => {
            Some(DOCX_CUSTOM)
        }
        (BinaryFormat::Docx | BinaryFormat::Xlsx | BinaryFormat::Pptx, "docProps/app.xml") => {
            Some(DOCX_APP)
        }
        (BinaryFormat::Odt, "meta.xml") => Some(ODT_META),
        (BinaryFormat::Odt, "manifest.rdf") => Some(ODT_RDF),
        _ => None,
    }
}

fn should_scrub_private_attributes(format: BinaryFormat, name: &str) -> bool {
    match format {
        BinaryFormat::Docx => name.starts_with("word/"),
        BinaryFormat::Xlsx => name.starts_with("xl/comments") || name.starts_with("xl/persons"),
        BinaryFormat::Pptx => {
            name.starts_with("ppt/comments")
                || name.starts_with("ppt/commentAuthors")
                || name.starts_with("ppt/authors")
        }
        _ => false,
    }
}

fn is_private_attribute(key: &[u8]) -> bool {
    let local = key.rsplit(|byte| *byte == b':').next().unwrap_or(key);
    matches!(
        local,
        b"author" | b"initials" | b"date" | b"creator" | b"lastModifiedBy"
    )
}

fn scrub_xml_attributes(input: &[u8]) -> BinaryResult<Vec<u8>> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().check_end_names = true;
    let mut writer = Writer::new(Vec::with_capacity(input.len()));
    loop {
        match reader
            .read_event()
            .map_err(|_| "Malformed document XML rejected.")?
        {
            Event::Start(event) => {
                let attributes = event
                    .attributes()
                    .with_checks(true)
                    .map(|attribute| {
                        attribute.map(|value| {
                            (value.key.as_ref().to_vec(), value.value.as_ref().to_vec())
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| "Malformed XML attribute rejected.")?;
                let mut cleaned = event.into_owned();
                cleaned.clear_attributes();
                for (key, value) in &attributes {
                    if !is_private_attribute(key) {
                        cleaned.push_attribute((key.as_slice(), value.as_slice()));
                    }
                }
                writer
                    .write_event(Event::Start(cleaned))
                    .map_err(io_error)?;
            }
            Event::Empty(event) => {
                let attributes = event
                    .attributes()
                    .with_checks(true)
                    .map(|attribute| {
                        attribute.map(|value| {
                            (value.key.as_ref().to_vec(), value.value.as_ref().to_vec())
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| "Malformed XML attribute rejected.")?;
                let mut cleaned = event.into_owned();
                cleaned.clear_attributes();
                for (key, value) in &attributes {
                    if !is_private_attribute(key) {
                        cleaned.push_attribute((key.as_slice(), value.as_slice()));
                    }
                }
                writer
                    .write_event(Event::Empty(cleaned))
                    .map_err(io_error)?;
            }
            Event::Eof => break,
            event => writer.write_event(event.into_owned()).map_err(io_error)?,
        }
    }
    Ok(writer.into_inner())
}

fn privacy_attribute_count(data: &[u8]) -> BinaryResult<usize> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = true;
    let mut count = 0usize;
    loop {
        match reader
            .read_event()
            .map_err(|_| "Malformed document XML rejected.")?
        {
            Event::Start(event) | Event::Empty(event) => {
                for attribute in event.attributes().with_checks(true) {
                    let attribute = attribute.map_err(|_| "Malformed XML attribute rejected.")?;
                    if is_private_attribute(attribute.key.as_ref()) {
                        count += 1;
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(count)
}

fn unicode_payload_count(data: &[u8]) -> usize {
    std::str::from_utf8(data).map_or(0, |text| {
        text.chars()
            .filter(|character| {
                engine::classify(*character).is_some_and(|rule| rule.action == Action::RemoveSafe)
            })
            .count()
    })
}

pub fn analyze(bytes: &[u8]) -> BinaryResult<ParsedBinary> {
    let format = detect(bytes)?;
    let mut archive = open(bytes)?;
    validate_entries(&mut archive)?;
    let mut properties = 0usize;
    let mut author_traces = 0usize;
    let mut unicode_payloads = 0usize;
    let mut c2pa_manifests = 0usize;
    for index in 0..archive.len() {
        let name = archive
            .name_for_index(index)
            .ok_or("Document entry name unavailable.")?
            .to_owned();
        if name.ends_with('/') {
            continue;
        }
        if name == C2PA_ENTRY {
            c2pa_manifests += 1;
            continue;
        }
        let data = read_entry(&mut archive, index)?;
        if canonical_metadata(format, &name).is_some_and(|canonical| data != canonical) {
            properties += 1;
        }
        if name.ends_with(".xml") {
            scrub_xml_attributes(&data)?;
            if should_scrub_private_attributes(format, &name) {
                author_traces += privacy_attribute_count(&data)?;
            }
            unicode_payloads += unicode_payload_count(&data);
        }
    }
    let mut findings = Vec::new();
    if properties > 0 {
        findings.push(BinaryFinding {
            kind: "document-properties",
            count: properties,
            description: "Document package properties",
        });
    }
    if author_traces > 0 {
        findings.push(BinaryFinding {
            kind: "author-traces",
            count: author_traces,
            description: "Author/date revision attributes",
        });
    }
    if unicode_payloads > 0 {
        findings.push(BinaryFinding {
            kind: "unicode",
            count: unicode_payloads,
            description: "Invisible Unicode payloads in document XML",
        });
    }
    if c2pa_manifests > 0 {
        findings.push(BinaryFinding {
            kind: "c2pa",
            count: c2pa_manifests,
            description: "ZIP-based C2PA provenance manifest",
        });
    }
    Ok(ParsedBinary {
        format,
        findings,
        c2pa_detected: c2pa_manifests > 0,
    })
}

fn clean_entry(format: BinaryFormat, name: &str, data: &[u8]) -> BinaryResult<Vec<u8>> {
    if let Some(canonical) = canonical_metadata(format, name) {
        return Ok(canonical.to_vec());
    }
    if !name.ends_with(".xml") {
        return Ok(data.to_vec());
    }
    let validated = scrub_xml_attributes(data)?;
    let scrubbed = if should_scrub_private_attributes(format, name) {
        validated
    } else {
        data.to_vec()
    };
    let text = std::str::from_utf8(&scrubbed).map_err(|_| "Document XML must be UTF-8.")?;
    Ok(engine::sanitize(text, Mode::Safe).output.into_bytes())
}

pub fn sanitize(bytes: &[u8]) -> BinaryResult<CleanedBinary> {
    let format = detect(bytes)?;
    let before = analyze(bytes)?;
    let mut archive = open(bytes)?;
    validate_entries(&mut archive)?;
    let cursor = Cursor::new(Vec::with_capacity(bytes.len()));
    let mut writer = ZipWriter::new(cursor);

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(zip_error)?;
        let name = file.name().to_owned();
        let is_dir = file.is_dir();
        let compression = file.compression();
        let permissions = file
            .unix_mode()
            .unwrap_or(if is_dir { 0o700 } else { 0o600 })
            & 0o777;
        let mut data = Vec::with_capacity(usize::try_from(file.size()).unwrap_or(0));
        file.by_ref()
            .take(MAX_ENTRY_BYTES + 1)
            .read_to_end(&mut data)
            .map_err(io_error)?;
        if data.len() as u64 > MAX_ENTRY_BYTES {
            return Err("Expanded document entry limit exceeded.");
        }
        drop(file);

        if name == C2PA_ENTRY {
            continue;
        }

        let options = SimpleFileOptions::default()
            .compression_method(compression)
            .last_modified_time(DateTime::default())
            .unix_permissions(permissions);
        if is_dir {
            writer.add_directory(name, options).map_err(zip_error)?;
        } else {
            let cleaned = clean_entry(format, &name, &data)?;
            writer.start_file(name, options).map_err(zip_error)?;
            writer.write_all(&cleaned).map_err(io_error)?;
        }
    }

    let output = writer.finish().map_err(zip_error)?.into_inner();
    Ok(CleanedBinary {
        bytes: output,
        removed_items: before.metadata_count(),
    })
}
