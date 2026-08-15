// STATUS: DIAMANT VGT SUPREME

use super::model::{BinaryFinding, BinaryFormat, BinaryResult, CleanedBinary, ParsedBinary};
use super::PNG_SIGNATURE;
use crc32fast::Hasher;

struct Chunk {
    kind: [u8; 4],
    start: usize,
    end: usize,
}

fn parse(bytes: &[u8]) -> BinaryResult<Vec<Chunk>> {
    if !bytes.starts_with(PNG_SIGNATURE) {
        return Err("PNG signature validation failed.");
    }
    let mut chunks = Vec::new();
    let mut offset = PNG_SIGNATURE.len();
    let mut seen_header = false;
    let mut seen_data = false;
    let mut seen_end = false;

    while offset < bytes.len() {
        if seen_end || bytes.len() - offset < 12 {
            return Err("Malformed PNG chunk boundary.");
        }
        let length = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| "Invalid PNG length.")?,
        ) as usize;
        let data_start = offset.checked_add(8).ok_or("PNG offset overflow.")?;
        let data_end = data_start
            .checked_add(length)
            .ok_or("PNG chunk length overflow.")?;
        let end = data_end
            .checked_add(4)
            .ok_or("PNG CRC boundary overflow.")?;
        if end > bytes.len() {
            return Err("Truncated PNG chunk.");
        }
        let kind: [u8; 4] = bytes[offset + 4..offset + 8]
            .try_into()
            .map_err(|_| "Invalid PNG chunk type.")?;
        if !kind.iter().all(u8::is_ascii_alphabetic) {
            return Err("Invalid PNG chunk type bytes.");
        }

        let mut hasher = Hasher::new();
        hasher.update(&kind);
        hasher.update(&bytes[data_start..data_end]);
        let stored_crc = u32::from_be_bytes(
            bytes[data_end..end]
                .try_into()
                .map_err(|_| "Invalid PNG CRC.")?,
        );
        if hasher.finalize() != stored_crc {
            return Err("PNG CRC validation failed.");
        }

        if chunks.is_empty() {
            if kind != *b"IHDR" || length != 13 {
                return Err("PNG must begin with a 13-byte IHDR chunk.");
            }
            seen_header = true;
        } else if kind == *b"IHDR" {
            return Err("Duplicate PNG IHDR chunk.");
        }
        if kind == *b"IDAT" {
            seen_data = true;
        }
        if kind == *b"IEND" {
            if length != 0 || end != bytes.len() {
                return Err("PNG IEND or trailing-data validation failed.");
            }
            seen_end = true;
        }
        chunks.push(Chunk {
            kind,
            start: offset,
            end,
        });
        offset = end;
    }

    if !seen_header || !seen_data || !seen_end {
        return Err("PNG structural validation failed.");
    }
    Ok(chunks)
}

fn metadata_kind(kind: &[u8; 4]) -> Option<(&'static str, &'static str)> {
    match kind {
        b"eXIf" => Some(("exif", "EXIF image metadata")),
        b"tEXt" | b"zTXt" | b"iTXt" => Some(("text", "PNG textual metadata")),
        b"tIME" => Some(("timestamp", "PNG modification timestamp")),
        b"caBX" => Some(("c2pa", "C2PA provenance manifest")),
        _ => None,
    }
}

pub fn analyze(bytes: &[u8]) -> BinaryResult<ParsedBinary> {
    let chunks = parse(bytes)?;
    let categories = ["exif", "text", "timestamp", "c2pa"];
    let mut findings = Vec::new();
    for category in categories {
        let matching: Vec<_> = chunks
            .iter()
            .filter_map(|chunk| metadata_kind(&chunk.kind))
            .filter(|(kind, _)| *kind == category)
            .collect();
        if let Some((kind, description)) = matching.first().copied() {
            findings.push(BinaryFinding {
                kind,
                count: matching.len(),
                description,
            });
        }
    }
    let c2pa_detected = findings.iter().any(|finding| finding.kind == "c2pa");
    Ok(ParsedBinary {
        format: BinaryFormat::Png,
        findings,
        c2pa_detected,
    })
}

pub fn sanitize(bytes: &[u8]) -> BinaryResult<CleanedBinary> {
    let chunks = parse(bytes)?;
    let mut output = Vec::with_capacity(bytes.len());
    output.extend_from_slice(PNG_SIGNATURE);
    let mut removed_items = 0usize;
    for chunk in chunks {
        if metadata_kind(&chunk.kind).is_some() {
            removed_items += 1;
        } else {
            output.extend_from_slice(&bytes[chunk.start..chunk.end]);
        }
    }
    Ok(CleanedBinary {
        bytes: output,
        removed_items,
    })
}
