// STATUS: DIAMANT VGT SUPREME
#![forbid(unsafe_code)]

mod binary;
mod engine;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use binary::BinaryFinding;
use engine::{Action, Mode, Severity};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;
const MAX_BINARY_BYTES: usize = 32 * 1024 * 1024;
const MAX_BINARY_BASE64_BYTES: usize = MAX_BINARY_BYTES.div_ceil(3) * 4;
const MAX_POSITIONS_PER_GROUP: usize = 8;
const RULE_SET_VERSION: &str = "nullmark-unicode-1.0.0";

#[derive(Serialize, Clone)]
struct RiskCounts {
    info: usize,
    low: usize,
    medium: usize,
    high: usize,
}

#[derive(Serialize, Clone)]
struct FindingGroup {
    codepoint: String,
    name: &'static str,
    category: &'static str,
    severity: &'static str,
    action: &'static str,
    count: usize,
    first_positions: Vec<usize>,
}

#[derive(Serialize, Clone)]
struct AnalysisReport {
    rule_set_version: &'static str,
    bytes: usize,
    characters: usize,
    finding_count: usize,
    sha256: String,
    risk_counts: RiskCounts,
    findings: Vec<FindingGroup>,
}

#[derive(Serialize)]
struct SanitizeReport {
    output: String,
    mode: &'static str,
    removed_count: usize,
    normalized_count: usize,
    verification_passed: bool,
    canonical_projection_unchanged: bool,
    verification_scope: &'static str,
    probabilistic_watermark_status: &'static str,
    change_count: usize,
    changes_truncated: bool,
    changes: Vec<TextChange>,
    before: AnalysisReport,
    after: AnalysisReport,
}

#[derive(Serialize)]
struct TextChange {
    source_index: usize,
    output_index: usize,
    before: String,
    after: String,
    kind: &'static str,
}

impl From<engine::EngineChange> for TextChange {
    fn from(change: engine::EngineChange) -> Self {
        Self {
            source_index: change.source_index,
            output_index: change.output_index,
            before: change.before,
            after: change.after,
            kind: change.kind,
        }
    }
}

#[derive(Serialize, Clone)]
struct BinaryAnalysisReport {
    format: &'static str,
    mime: &'static str,
    bytes: usize,
    sha256: String,
    metadata_count: usize,
    c2pa_detected: bool,
    findings: Vec<BinaryFinding>,
}

#[derive(Serialize)]
struct BinarySanitizeReport {
    output_base64: String,
    removed_items: usize,
    verification_passed: bool,
    before: BinaryAnalysisReport,
    after: BinaryAnalysisReport,
}

#[derive(Default)]
struct GroupBuilder {
    name: Option<&'static str>,
    category: Option<&'static str>,
    severity: Option<Severity>,
    action: Option<Action>,
    count: usize,
    positions: Vec<usize>,
}

fn severity_label(value: Severity) -> &'static str {
    match value {
        Severity::Info => "info",
        Severity::Low => "low",
        Severity::Medium => "medium",
        Severity::High => "high",
    }
}

fn action_label(value: Action) -> &'static str {
    match value {
        Action::RemoveSafe => "remove_safe",
        Action::RemoveStrict => "remove_strict",
        Action::RemoveMaximum => "remove_maximum",
        Action::NormalizeStrict => "normalize_strict",
        Action::NormalizeMaximum => "normalize_maximum",
        Action::ReportOnly => "report_only",
    }
}

fn sha256_hex(value: &str) -> String {
    sha256_hex_bytes(value.as_bytes())
}

fn sha256_hex_bytes(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn validate_input(text: &str) -> Result<(), String> {
    if text.len() > MAX_INPUT_BYTES {
        return Err("Input exceeds the 8 MiB beta safety limit.".into());
    }
    Ok(())
}

fn decode_binary(payload_base64: &str) -> Result<Vec<u8>, String> {
    if payload_base64.len() > MAX_BINARY_BASE64_BYTES {
        return Err("Encoded file exceeds the 32 MiB safety limit.".into());
    }
    let bytes = BASE64
        .decode(payload_base64)
        .map_err(|_| "Binary payload encoding rejected.".to_string())?;
    if bytes.is_empty() || bytes.len() > MAX_BINARY_BYTES {
        return Err("File size boundary violation.".into());
    }
    Ok(bytes)
}

fn build_binary_analysis(bytes: &[u8]) -> Result<BinaryAnalysisReport, String> {
    let parsed = binary::analyze(bytes).map_err(|detail| {
        eprintln!("[BINARY_VALIDATION] {detail}");
        "Binary file validation failed.".to_string()
    })?;
    Ok(BinaryAnalysisReport {
        format: parsed.format.label(),
        mime: parsed.format.mime(),
        bytes: bytes.len(),
        sha256: sha256_hex_bytes(bytes),
        metadata_count: parsed.metadata_count(),
        c2pa_detected: parsed.c2pa_detected,
        findings: parsed.findings,
    })
}

fn build_analysis(text: &str) -> AnalysisReport {
    // Stream over the input and aggregate by code point. This keeps memory bounded
    // even for adversarial files containing millions of invisible characters.
    let mut groups: BTreeMap<u32, GroupBuilder> = BTreeMap::new();
    let mut risk_counts = RiskCounts {
        info: 0,
        low: 0,
        medium: 0,
        high: 0,
    };
    let mut finding_count = 0usize;

    for (char_index, c) in text.chars().enumerate() {
        let Some(rule) = engine::classify(c) else {
            continue;
        };
        finding_count += 1;

        match rule.severity {
            Severity::Info => risk_counts.info += 1,
            Severity::Low => risk_counts.low += 1,
            Severity::Medium => risk_counts.medium += 1,
            Severity::High => risk_counts.high += 1,
        }

        let group = groups.entry(c as u32).or_default();
        group.name = Some(rule.name);
        group.category = Some(rule.category);
        group.severity = Some(rule.severity);
        group.action = Some(rule.action);
        group.count += 1;
        if group.positions.len() < MAX_POSITIONS_PER_GROUP {
            group.positions.push(char_index);
        }
    }

    let findings = groups
        .into_iter()
        .map(|(codepoint, group)| FindingGroup {
            codepoint: format!("U+{codepoint:04X}"),
            name: group.name.unwrap_or("UNKNOWN"),
            category: group.category.unwrap_or("Unknown"),
            severity: severity_label(group.severity.unwrap_or(Severity::Info)),
            action: action_label(group.action.unwrap_or(Action::ReportOnly)),
            count: group.count,
            first_positions: group.positions,
        })
        .collect();

    AnalysisReport {
        rule_set_version: RULE_SET_VERSION,
        bytes: text.len(),
        characters: text.chars().count(),
        finding_count,
        sha256: sha256_hex(text),
        risk_counts,
        findings,
    }
}

#[tauri::command]
fn analyze_text(text: String) -> Result<AnalysisReport, String> {
    validate_input(&text)?;
    Ok(build_analysis(&text))
}

#[tauri::command]
fn sanitize_text(text: String, mode: String) -> Result<SanitizeReport, String> {
    validate_input(&text)?;

    let selected_mode = match mode.as_str() {
        "safe" => Mode::Safe,
        "strict" => Mode::Strict,
        "maximum" => Mode::Maximum,
        _ => return Err("Unsupported sanitization mode.".into()),
    };

    let before = build_analysis(&text);
    let projection_before = engine::canonical_projection(&text);
    let sanitized = engine::sanitize(&text, selected_mode);
    let projection_after = engine::canonical_projection(&sanitized.output);
    let after = build_analysis(&sanitized.output);
    let verification_passed = engine::is_clean_for_mode(&sanitized.output, selected_mode);

    Ok(SanitizeReport {
        output: sanitized.output,
        mode: match selected_mode {
            Mode::Safe => "safe",
            Mode::Strict => "strict",
            Mode::Maximum => "maximum",
        },
        removed_count: sanitized.removed_count,
        normalized_count: sanitized.normalized_count,
        verification_passed,
        canonical_projection_unchanged: projection_before == projection_after,
        verification_scope: match selected_mode {
            Mode::Safe => "deterministic-safe-unicode",
            Mode::Strict => "deterministic-strict-unicode",
            Mode::Maximum => "deterministic-unicode-and-token-surface",
        },
        probabilistic_watermark_status: "not-verifiable-without-vendor-detector",
        change_count: sanitized.change_count,
        changes_truncated: sanitized.changes.len() < sanitized.change_count,
        changes: sanitized
            .changes
            .into_iter()
            .map(TextChange::from)
            .collect(),
        before,
        after,
    })
}

#[tauri::command]
fn analyze_binary(payload_base64: String) -> Result<BinaryAnalysisReport, String> {
    let bytes = decode_binary(&payload_base64)?;
    build_binary_analysis(&bytes)
}

#[tauri::command]
fn sanitize_binary(payload_base64: String) -> Result<BinarySanitizeReport, String> {
    let bytes = decode_binary(&payload_base64)?;
    let before = build_binary_analysis(&bytes)?;
    let cleaned = binary::sanitize(&bytes).map_err(|detail| {
        eprintln!("[BINARY_SANITIZE] {detail}");
        "Binary sanitation failed.".to_string()
    })?;
    if cleaned.bytes.is_empty() || cleaned.bytes.len() > MAX_BINARY_BYTES {
        return Err("Sanitized file exceeds the 32 MiB output safety limit.".into());
    }
    let after = build_binary_analysis(&cleaned.bytes)?;
    let verification_passed = after.metadata_count == 0 && !after.c2pa_detected;
    Ok(BinarySanitizeReport {
        output_base64: BASE64.encode(&cleaned.bytes),
        removed_items: cleaned.removed_items,
        verification_passed,
        before,
        after,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            analyze_text,
            sanitize_text,
            analyze_binary,
            sanitize_binary
        ])
        .run(tauri::generate_context!())
        .expect("error while running NullMark");
}
