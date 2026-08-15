// STATUS: DIAMANT VGT SUPREME

use super::model::{BinaryFinding, BinaryFormat, BinaryResult, CleanedBinary, ParsedBinary};
use lopdf::{Document, Object, ObjectId};
use std::collections::HashSet;

const MAX_OBJECTS: usize = 100_000;
const MAX_STREAM_BYTES: usize = 128 * 1024 * 1024;

fn pdf_error<T>(_: T) -> &'static str {
    "PDF structural validation failed."
}

fn load(bytes: &[u8]) -> BinaryResult<Document> {
    if !bytes.starts_with(b"%PDF-") {
        return Err("PDF signature validation failed.");
    }
    let document = Document::load_mem(bytes).map_err(pdf_error)?;
    if document.trailer.has(b"Encrypt") || document.encryption_state.is_some() {
        return Err("Encrypted PDF documents are not accepted.");
    }
    if document.objects.is_empty() || document.objects.len() > MAX_OBJECTS {
        return Err("PDF object boundary rejected.");
    }
    let stream_bytes = document
        .objects
        .values()
        .try_fold(0usize, |total, object| {
            let size = match object {
                Object::Stream(stream) => stream.content.len(),
                _ => 0,
            };
            total.checked_add(size).ok_or("PDF stream size overflow.")
        })?;
    if stream_bytes > MAX_STREAM_BYTES {
        return Err("PDF stream boundary rejected.");
    }
    let root_id = document
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .map_err(pdf_error)?;
    let catalog = document.get_dictionary(root_id).map_err(pdf_error)?;
    if !catalog.has_type(b"Catalog") {
        return Err("PDF catalog validation failed.");
    }
    Ok(document)
}

fn root_id(document: &Document) -> BinaryResult<ObjectId> {
    document
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .map_err(pdf_error)
}

fn metadata_count(document: &Document) -> BinaryResult<usize> {
    root_id(document)?;
    Ok(usize::from(document.trailer.has(b"Info")) + metadata_stream_ids(document).len())
}

fn object_dictionary(object: &Object) -> Option<&lopdf::Dictionary> {
    match object {
        Object::Dictionary(value) => Some(value),
        Object::Stream(value) => Some(&value.dict),
        _ => None,
    }
}

fn object_dictionary_mut(object: &mut Object) -> Option<&mut lopdf::Dictionary> {
    match object {
        Object::Dictionary(value) => Some(value),
        Object::Stream(value) => Some(&mut value.dict),
        _ => None,
    }
}

fn metadata_stream_ids(document: &Document) -> Vec<ObjectId> {
    document
        .objects
        .iter()
        .filter_map(|(id, object)| {
            matches!(object, Object::Stream(stream) if stream.dict.has_type(b"Metadata"))
                .then_some(*id)
        })
        .collect()
}

fn annotation_privacy_count(document: &Document) -> usize {
    document
        .objects
        .values()
        .filter_map(object_dictionary)
        .filter(|dictionary| dictionary.has_type(b"Annot"))
        .map(|dictionary| {
            [b"T".as_slice(), b"M", b"CreationDate", b"NM"]
                .into_iter()
                .filter(|key| dictionary.has(key))
                .count()
        })
        .sum()
}

fn is_active_action(object: &Object) -> bool {
    object_dictionary(object).is_some_and(|dictionary| {
        dictionary
            .get(b"S")
            .and_then(Object::as_name)
            .is_ok_and(|name| matches!(name, b"JavaScript" | b"Launch"))
    })
}

fn active_content_count(document: &Document) -> usize {
    document
        .objects
        .values()
        .filter_map(object_dictionary)
        .map(|dictionary| {
            usize::from(
                dictionary
                    .get(b"S")
                    .and_then(Object::as_name)
                    .is_ok_and(|name| matches!(name, b"JavaScript" | b"Launch")),
            ) + [b"JS".as_slice(), b"OpenAction", b"AA"]
                .into_iter()
                .filter(|key| dictionary.has(key))
                .count()
        })
        .sum()
}

fn is_c2pa_filespec(object: &Object) -> bool {
    object.as_dict().is_ok_and(|dictionary| {
        dictionary
            .get(b"AFRelationship")
            .and_then(Object::as_name)
            .is_ok_and(|name| name == b"C2PA_Manifest")
    })
}

fn c2pa_filespec_ids(document: &Document) -> Vec<ObjectId> {
    document
        .objects
        .iter()
        .filter_map(|(id, object)| is_c2pa_filespec(object).then_some(*id))
        .collect()
}

fn embedded_stream_ids(document: &Document, filespec_ids: &[ObjectId]) -> Vec<ObjectId> {
    let mut ids = Vec::new();
    for filespec_id in filespec_ids {
        let Ok(filespec) = document.get_dictionary(*filespec_id) else {
            continue;
        };
        let Ok(embedded_files) = filespec.get(b"EF") else {
            continue;
        };
        let dictionary = match embedded_files {
            Object::Dictionary(dictionary) => Some(dictionary),
            Object::Reference(id) => document.get_dictionary(*id).ok(),
            _ => None,
        };
        if let Some(dictionary) = dictionary {
            ids.extend(
                dictionary
                    .iter()
                    .filter_map(|(_, object)| object.as_reference().ok()),
            );
        }
    }
    ids.sort_unstable();
    ids.dedup();
    ids
}

pub fn detect(bytes: &[u8]) -> BinaryResult<BinaryFormat> {
    load(bytes)?;
    Ok(BinaryFormat::Pdf)
}

pub fn analyze(bytes: &[u8]) -> BinaryResult<ParsedBinary> {
    let document = load(bytes)?;
    let count = metadata_count(&document)?;
    let annotation_count = annotation_privacy_count(&document);
    let active_count = active_content_count(&document);
    let c2pa_count = c2pa_filespec_ids(&document).len();
    let mut findings = Vec::new();
    if count > 0 {
        findings.push(BinaryFinding {
            kind: "pdf-metadata",
            count,
            description: "PDF Info dictionary or catalog XMP metadata",
        });
    }
    if c2pa_count > 0 {
        findings.push(BinaryFinding {
            kind: "c2pa",
            count: c2pa_count,
            description: "PDF C2PA associated-file manifest",
        });
    }
    if annotation_count > 0 {
        findings.push(BinaryFinding {
            kind: "pdf-annotation-privacy",
            count: annotation_count,
            description: "PDF annotation author and timestamp fields",
        });
    }
    if active_count > 0 {
        findings.push(BinaryFinding {
            kind: "pdf-active-content",
            count: active_count,
            description: "PDF JavaScript, launch and automatic actions",
        });
    }
    Ok(ParsedBinary {
        format: BinaryFormat::Pdf,
        findings,
        c2pa_detected: c2pa_count > 0,
    })
}

fn neutralize_reference(document: &mut Document, object: Option<Object>) {
    if let Some(Object::Reference(id)) = object {
        if let Some(target) = document.objects.get_mut(&id) {
            *target = Object::Null;
        }
    }
}

pub fn sanitize(bytes: &[u8]) -> BinaryResult<CleanedBinary> {
    let mut document = load(bytes)?;
    let c2pa_filespecs = c2pa_filespec_ids(&document);
    let c2pa_streams = embedded_stream_ids(&document, &c2pa_filespecs);
    let before = analyze(bytes)?;
    let removed_items = before.metadata_count();
    let metadata_streams = metadata_stream_ids(&document);
    let active_actions: HashSet<ObjectId> = document
        .objects
        .iter()
        .filter_map(|(id, object)| is_active_action(object).then_some(*id))
        .collect();
    let info = document.trailer.remove(b"Info");
    let root = root_id(&document)?;
    let metadata = document
        .get_dictionary_mut(root)
        .map_err(pdf_error)?
        .remove(b"Metadata");
    neutralize_reference(&mut document, info);
    neutralize_reference(&mut document, metadata);
    for id in c2pa_filespecs
        .into_iter()
        .chain(c2pa_streams)
        .chain(metadata_streams)
        .chain(active_actions.iter().copied())
    {
        if let Some(object) = document.objects.get_mut(&id) {
            *object = Object::Null;
        }
    }
    for (id, object) in &mut document.objects {
        if active_actions.contains(id) {
            continue;
        }
        let Some(dictionary) = object_dictionary_mut(object) else {
            continue;
        };
        if dictionary.has_type(b"Annot") {
            for key in [b"T".as_slice(), b"M", b"CreationDate", b"NM"] {
                dictionary.remove(key);
            }
        }
        for key in [b"JS".as_slice(), b"OpenAction", b"AA"] {
            dictionary.remove(key);
        }
    }
    document.trailer.remove(b"Prev");
    document.trailer.remove(b"XRefStm");
    let mut output = Vec::with_capacity(bytes.len());
    document.save_to(&mut output).map_err(pdf_error)?;
    let verified = analyze(&output)?;
    if verified.metadata_count() != 0 || verified.c2pa_detected {
        return Err("PDF post-sanitization verification failed.");
    }
    Ok(CleanedBinary {
        bytes: output,
        removed_items,
    })
}
