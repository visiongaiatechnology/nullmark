// STATUS: DIAMANT VGT SUPREME

use super::model::{BinaryFinding, BinaryFormat, BinaryResult, CleanedBinary, ParsedBinary};

struct Chunk {
    kind: [u8; 4],
    start: usize,
    data_start: usize,
    data_end: usize,
    end: usize,
}

fn parse(bytes: &[u8]) -> BinaryResult<Vec<Chunk>> {
    if bytes.len() < 20 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return Err("WebP RIFF signature validation failed.");
    }
    let declared = u32::from_le_bytes(
        bytes[4..8]
            .try_into()
            .map_err(|_| "Invalid WebP RIFF size.")?,
    ) as usize;
    if declared.checked_add(8) != Some(bytes.len()) {
        return Err("WebP RIFF size or trailing-data validation failed.");
    }
    let mut chunks = Vec::new();
    let mut offset = 12usize;
    let mut has_image = false;
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err("Truncated WebP chunk header.");
        }
        let kind: [u8; 4] = bytes[offset..offset + 4]
            .try_into()
            .map_err(|_| "Invalid WebP FourCC.")?;
        if !kind
            .iter()
            .all(|byte| byte.is_ascii_graphic() || *byte == b' ')
        {
            return Err("Invalid WebP FourCC bytes.");
        }
        let length = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .map_err(|_| "Invalid WebP chunk size.")?,
        ) as usize;
        let data_start = offset + 8;
        let data_end = data_start
            .checked_add(length)
            .ok_or("WebP chunk length overflow.")?;
        let end = data_end
            .checked_add(length & 1)
            .ok_or("WebP padding overflow.")?;
        if end > bytes.len() {
            return Err("Truncated WebP chunk.");
        }
        if length & 1 == 1 && bytes[data_end] != 0 {
            return Err("Invalid WebP RIFF padding byte.");
        }
        if matches!(&kind, b"VP8 " | b"VP8L" | b"ANMF") {
            has_image = true;
        }
        if kind == *b"VP8X" && length != 10 {
            return Err("Invalid WebP VP8X chunk length.");
        }
        chunks.push(Chunk {
            kind,
            start: offset,
            data_start,
            data_end,
            end,
        });
        offset = end;
    }
    if !has_image {
        return Err("WebP image payload missing.");
    }
    let c2pa_positions = chunks
        .iter()
        .enumerate()
        .filter_map(|(index, chunk)| (chunk.kind == *b"C2PA").then_some(index))
        .collect::<Vec<_>>();
    if c2pa_positions.len() > 1
        || c2pa_positions
            .first()
            .is_some_and(|index| *index + 1 != chunks.len())
    {
        return Err("WebP C2PA chunk placement rejected.");
    }
    Ok(chunks)
}

fn metadata_kind(kind: &[u8; 4]) -> Option<(&'static str, &'static str)> {
    match kind {
        b"EXIF" => Some(("exif", "WebP EXIF metadata")),
        b"XMP " => Some(("xmp", "WebP XMP metadata")),
        b"C2PA" => Some(("c2pa", "WebP C2PA provenance manifest")),
        _ => None,
    }
}

pub fn analyze(bytes: &[u8]) -> BinaryResult<ParsedBinary> {
    let chunks = parse(bytes)?;
    let mut findings = Vec::new();
    for category in ["exif", "xmp", "c2pa"] {
        let count = chunks
            .iter()
            .filter(|chunk| metadata_kind(&chunk.kind).is_some_and(|(kind, _)| kind == category))
            .count();
        if count > 0 {
            let description = match category {
                "exif" => "WebP EXIF metadata",
                "xmp" => "WebP XMP metadata",
                _ => "WebP C2PA provenance manifest",
            };
            findings.push(BinaryFinding {
                kind: category,
                count,
                description,
            });
        }
    }
    Ok(ParsedBinary {
        format: BinaryFormat::WebP,
        findings,
        c2pa_detected: chunks.iter().any(|chunk| chunk.kind == *b"C2PA"),
    })
}

pub fn sanitize(bytes: &[u8]) -> BinaryResult<CleanedBinary> {
    let chunks = parse(bytes)?;
    let mut output = Vec::with_capacity(bytes.len());
    output.extend_from_slice(&bytes[..12]);
    let mut removed_items = 0usize;
    for chunk in chunks {
        if metadata_kind(&chunk.kind).is_some() {
            removed_items += 1;
            continue;
        }
        let output_start = output.len();
        output.extend_from_slice(&bytes[chunk.start..chunk.end]);
        if chunk.kind == *b"VP8X" {
            output[output_start + 8] &= !0b0000_1100;
        }
        debug_assert_eq!(
            chunk.data_end - chunk.data_start,
            u32::from_le_bytes(
                bytes[chunk.start + 4..chunk.start + 8]
                    .try_into()
                    .expect("validated WebP length")
            ) as usize
        );
    }
    let riff_size = u32::try_from(output.len().saturating_sub(8))
        .map_err(|_| "WebP output exceeds RIFF size limit.")?;
    output[4..8].copy_from_slice(&riff_size.to_le_bytes());
    Ok(CleanedBinary {
        bytes: output,
        removed_items,
    })
}
