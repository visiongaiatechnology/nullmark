// STATUS: DIAMANT VGT SUPREME

use super::model::{BinaryFinding, BinaryFormat, BinaryResult, CleanedBinary, ParsedBinary};

struct Segment {
    marker: u8,
    start: usize,
    end: usize,
    metadata: Option<(&'static str, &'static str)>,
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn classify_segment(marker: u8, payload: &[u8]) -> Option<(&'static str, &'static str)> {
    match marker {
        0xE1 if payload.starts_with(b"Exif\0\0") => Some(("exif", "JPEG EXIF metadata")),
        0xE1 if contains(payload, b"http://ns.adobe.com/xap/1.0/") => {
            Some(("xmp", "JPEG XMP metadata"))
        }
        0xEC => Some(("application", "JPEG application metadata")),
        0xED => Some(("iptc", "JPEG IPTC/Photoshop metadata")),
        0xFE => Some(("comment", "JPEG comment metadata")),
        0xEB if contains(payload, b"c2pa")
            || contains(
                payload,
                &[
                    0x63, 0x32, 0x70, 0x61, 0x00, 0x11, 0x00, 0x10, 0x80, 0x00, 0x00, 0xAA, 0x00,
                    0x38, 0x9B, 0x71,
                ],
            ) =>
        {
            Some(("c2pa", "C2PA provenance manifest"))
        }
        _ => None,
    }
}

fn expand_c2pa_fragments(segments: &mut [Segment]) {
    let seeds: Vec<usize> = segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| {
            segment
                .metadata
                .filter(|(kind, _)| *kind == "c2pa")
                .map(|_| index)
        })
        .collect();
    for seed in seeds {
        let mut left = seed;
        while left > 0
            && segments[left - 1].marker == 0xEB
            && segments[left - 1].end == segments[left].start
        {
            left -= 1;
        }
        let mut right = seed;
        while right + 1 < segments.len()
            && segments[right + 1].marker == 0xEB
            && segments[right].end == segments[right + 1].start
        {
            right += 1;
        }
        for segment in &mut segments[left..=right] {
            segment.metadata = Some(("c2pa", "C2PA provenance manifest"));
        }
    }
}

fn parse(bytes: &[u8]) -> BinaryResult<Vec<Segment>> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return Err("JPEG SOI validation failed.");
    }
    let mut offset = 2usize;
    let mut entropy = false;
    let mut segments = Vec::new();
    let mut finished = false;

    while offset < bytes.len() {
        if entropy {
            if bytes[offset] != 0xFF {
                offset += 1;
                continue;
            }
            let marker_start = offset;
            while offset < bytes.len() && bytes[offset] == 0xFF {
                offset += 1;
            }
            if offset >= bytes.len() {
                return Err("Truncated JPEG entropy marker.");
            }
            let marker = bytes[offset];
            if marker == 0x00 || (0xD0..=0xD7).contains(&marker) {
                offset += 1;
                continue;
            }
            offset = marker_start;
            entropy = false;
            continue;
        }

        let start = offset;
        if bytes[offset] != 0xFF {
            return Err("Malformed JPEG marker boundary.");
        }
        while offset < bytes.len() && bytes[offset] == 0xFF {
            offset += 1;
        }
        if offset >= bytes.len() {
            return Err("Truncated JPEG marker.");
        }
        let marker = bytes[offset];
        if marker == 0x00 || marker == 0xD8 {
            return Err("Invalid JPEG structural marker.");
        }
        if marker == 0xD9 {
            let end = offset + 1;
            if end != bytes.len() {
                return Err("JPEG trailing data rejected.");
            }
            segments.push(Segment {
                marker,
                start,
                end,
                metadata: None,
            });
            finished = true;
            break;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            segments.push(Segment {
                marker,
                start,
                end: offset + 1,
                metadata: None,
            });
            offset += 1;
            continue;
        }
        if bytes.len() - offset < 3 {
            return Err("Truncated JPEG segment length.");
        }
        let length = u16::from_be_bytes([bytes[offset + 1], bytes[offset + 2]]) as usize;
        if length < 2 {
            return Err("Invalid JPEG segment length.");
        }
        let end = (offset + 1)
            .checked_add(length)
            .ok_or("JPEG segment length overflow.")?;
        if end > bytes.len() {
            return Err("Truncated JPEG segment.");
        }
        let metadata = classify_segment(marker, &bytes[offset + 3..end]);
        segments.push(Segment {
            marker,
            start,
            end,
            metadata,
        });
        offset = end;
        entropy = marker == 0xDA;
    }

    if !finished {
        return Err("JPEG EOI marker missing.");
    }
    expand_c2pa_fragments(&mut segments);
    Ok(segments)
}

pub fn analyze(bytes: &[u8]) -> BinaryResult<ParsedBinary> {
    let segments = parse(bytes)?;
    let categories = ["exif", "xmp", "application", "iptc", "comment", "c2pa"];
    let mut findings = Vec::new();
    for category in categories {
        let mut count = 0usize;
        let mut description = "";
        for segment in &segments {
            if let Some((_, label)) = segment.metadata.filter(|(kind, _)| *kind == category) {
                count += 1;
                description = label;
            }
        }
        if count > 0 {
            findings.push(BinaryFinding {
                kind: category,
                count,
                description,
            });
        }
    }
    let c2pa_detected = findings.iter().any(|finding| finding.kind == "c2pa");
    Ok(ParsedBinary {
        format: BinaryFormat::Jpeg,
        findings,
        c2pa_detected,
    })
}

pub fn sanitize(bytes: &[u8]) -> BinaryResult<CleanedBinary> {
    let segments = parse(bytes)?;
    let mut output = Vec::with_capacity(bytes.len());
    let mut cursor = 0usize;
    let mut removed_items = 0usize;
    for segment in segments.iter().filter(|segment| segment.metadata.is_some()) {
        output.extend_from_slice(&bytes[cursor..segment.start]);
        cursor = segment.end;
        removed_items += 1;
    }
    output.extend_from_slice(&bytes[cursor..]);
    Ok(CleanedBinary {
        bytes: output,
        removed_items,
    })
}
