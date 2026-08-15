// STATUS: DIAMANT VGT SUPREME

use super::model::{BinaryFinding, BinaryFormat, BinaryResult, CleanedBinary, ParsedBinary};
use crate::engine::{self, Action, Mode};
use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

const MAX_SVG_BYTES: usize = 32 * 1024 * 1024;

#[derive(Default)]
struct SvgInventory {
    metadata: usize,
    comments: usize,
    active_content: usize,
    external_references: usize,
    unicode_payloads: usize,
    c2pa: usize,
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn is_removed_element(name: &[u8]) -> bool {
    matches!(local_name(name), b"metadata" | b"script")
}

fn is_external_reference(value: &[u8]) -> bool {
    let Ok(value) = std::str::from_utf8(value) else {
        return true;
    };
    let normalized = value.trim().to_ascii_lowercase();
    normalized.starts_with("http:")
        || normalized.starts_with("https:")
        || normalized.starts_with("//")
        || normalized.starts_with("data:")
        || normalized.starts_with("javascript:")
        || normalized.starts_with("file:")
}

fn inspect_attributes(event: &BytesStart<'_>, inventory: &mut SvgInventory) -> BinaryResult<()> {
    for attribute in event.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| "Malformed SVG attribute rejected.")?;
        let key = attribute.key.as_ref();
        let local = local_name(key);
        if local.len() > 2 && local[..2].eq_ignore_ascii_case(b"on") {
            inventory.active_content += 1;
        } else if matches!(local, b"href") && is_external_reference(attribute.value.as_ref()) {
            inventory.external_references += 1;
        }
        inventory.unicode_payloads += unicode_count(attribute.value.as_ref());
    }
    Ok(())
}

fn unicode_count(value: &[u8]) -> usize {
    std::str::from_utf8(value).map_or(0, |text| {
        text.chars()
            .filter(|character| {
                engine::classify(*character).is_some_and(|rule| rule.action == Action::RemoveSafe)
            })
            .count()
    })
}

fn inventory(bytes: &[u8]) -> BinaryResult<SvgInventory> {
    if bytes.is_empty() || bytes.len() > MAX_SVG_BYTES {
        return Err("SVG size boundary rejected.");
    }
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().check_end_names = true;
    let mut result = SvgInventory::default();
    let mut root_seen = false;
    loop {
        match reader
            .read_event()
            .map_err(|_| "Malformed SVG XML rejected.")?
        {
            Event::Start(event) | Event::Empty(event) => {
                if !root_seen {
                    if local_name(event.name().as_ref()) != b"svg" {
                        return Err("XML root is not SVG.");
                    }
                    root_seen = true;
                }
                let name = event.name();
                let local = local_name(name.as_ref());
                if local == b"metadata" {
                    result.metadata += 1;
                } else if local == b"script" {
                    result.active_content += 1;
                }
                if local == b"manifest" && name.as_ref().starts_with(b"c2pa:") {
                    result.c2pa += 1;
                }
                inspect_attributes(&event, &mut result)?;
            }
            Event::Text(event) => result.unicode_payloads += unicode_count(event.as_ref()),
            Event::CData(event) => result.unicode_payloads += unicode_count(event.as_ref()),
            Event::Comment(_) => result.comments += 1,
            Event::DocType(_) => return Err("SVG document type declarations are rejected."),
            Event::Eof => break,
            _ => {}
        }
    }
    if !root_seen {
        return Err("SVG root element missing.");
    }
    Ok(result)
}

pub fn is_candidate(bytes: &[u8]) -> bool {
    let value = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    value
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
        == Some(b'<')
}

pub fn detect(bytes: &[u8]) -> BinaryResult<BinaryFormat> {
    inventory(bytes)?;
    Ok(BinaryFormat::Svg)
}

pub fn analyze(bytes: &[u8]) -> BinaryResult<ParsedBinary> {
    let inventory = inventory(bytes)?;
    let mut findings = Vec::new();
    let mut push = |kind, count, description| {
        if count > 0 {
            findings.push(BinaryFinding {
                kind,
                count,
                description,
            });
        }
    };
    push("svg-metadata", inventory.metadata, "SVG metadata elements");
    push("svg-comments", inventory.comments, "SVG comments");
    push(
        "active-content",
        inventory.active_content,
        "SVG scripts and event handlers",
    );
    push(
        "external-references",
        inventory.external_references,
        "External SVG references",
    );
    push(
        "unicode",
        inventory.unicode_payloads,
        "Invisible Unicode payloads in SVG",
    );
    push("c2pa", inventory.c2pa, "SVG C2PA provenance manifest");
    Ok(ParsedBinary {
        format: BinaryFormat::Svg,
        findings,
        c2pa_detected: inventory.c2pa > 0,
    })
}

fn clean_element(event: BytesStart<'_>) -> BinaryResult<BytesStart<'static>> {
    let attributes = event
        .attributes()
        .with_checks(true)
        .map(|attribute| {
            attribute.map(|value| (value.key.as_ref().to_vec(), value.value.as_ref().to_vec()))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Malformed SVG attribute rejected.")?;
    let mut cleaned = event.into_owned();
    cleaned.clear_attributes();
    for (key, value) in attributes {
        let local = local_name(&key);
        if local.len() > 2 && local[..2].eq_ignore_ascii_case(b"on") {
            continue;
        }
        if local == b"href" && is_external_reference(&value) {
            continue;
        }
        let text = std::str::from_utf8(&value).map_err(|_| "SVG attributes must be UTF-8.")?;
        let sanitized = engine::sanitize(text, Mode::Safe).output;
        cleaned.push_attribute((key.as_slice(), sanitized.as_bytes()));
    }
    Ok(cleaned)
}

pub fn sanitize(bytes: &[u8]) -> BinaryResult<CleanedBinary> {
    let before = analyze(bytes)?;
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().check_end_names = true;
    let mut writer = Writer::new(Vec::with_capacity(bytes.len()));
    let mut skip_depth = 0usize;
    loop {
        let event = reader
            .read_event()
            .map_err(|_| "Malformed SVG XML rejected.")?;
        if skip_depth > 0 {
            match event {
                Event::Start(_) => skip_depth += 1,
                Event::End(_) => skip_depth -= 1,
                Event::DocType(_) => return Err("SVG document type declarations are rejected."),
                Event::Eof => return Err("Truncated private SVG subtree rejected."),
                _ => {}
            }
            continue;
        }
        match event {
            Event::Start(event) if is_removed_element(event.name().as_ref()) => skip_depth = 1,
            Event::Empty(event) if is_removed_element(event.name().as_ref()) => {}
            Event::Start(event) => writer
                .write_event(Event::Start(clean_element(event)?))
                .map_err(|_| "SVG output write failed.")?,
            Event::Empty(event) => writer
                .write_event(Event::Empty(clean_element(event)?))
                .map_err(|_| "SVG output write failed.")?,
            Event::Text(event) => {
                let raw = event.decode().map_err(|_| "SVG text must be UTF-8.")?;
                let sanitized = engine::sanitize(&raw, Mode::Safe).output;
                writer
                    .write_event(Event::Text(BytesText::from_escaped(sanitized)))
                    .map_err(|_| "SVG output write failed.")?;
            }
            Event::CData(event) => {
                let raw = event.decode().map_err(|_| "SVG CDATA must be UTF-8.")?;
                let sanitized = engine::sanitize(&raw, Mode::Safe).output;
                writer
                    .write_event(Event::CData(quick_xml::events::BytesCData::new(sanitized)))
                    .map_err(|_| "SVG output write failed.")?;
            }
            Event::Comment(_) => {}
            Event::DocType(_) => return Err("SVG document type declarations are rejected."),
            Event::Eof => break,
            event => writer
                .write_event(event.into_owned())
                .map_err(|_| "SVG output write failed.")?,
        }
    }
    Ok(CleanedBinary {
        bytes: writer.into_inner(),
        removed_items: before.metadata_count(),
    })
}
