// STATUS: DIAMANT VGT SUPREME

use super::model::{Action, EngineChange, EngineResult, Mode};
use super::rules::{classify, maximum_replacement};

fn should_remove(mode: Mode, action: Action) -> bool {
    matches!(action, Action::RemoveSafe)
        || matches!(mode, Mode::Strict | Mode::Maximum) && matches!(action, Action::RemoveStrict)
        || matches!(mode, Mode::Maximum) && matches!(action, Action::RemoveMaximum)
}

fn should_normalize(mode: Mode, action: Action) -> bool {
    matches!(mode, Mode::Strict | Mode::Maximum) && matches!(action, Action::NormalizeStrict)
        || matches!(mode, Mode::Maximum) && matches!(action, Action::NormalizeMaximum)
}

fn append_normalized(output: &mut String, c: char, action: Action) {
    match action {
        Action::NormalizeStrict => output.push(' '),
        Action::NormalizeMaximum if (0xFF01..=0xFF5E).contains(&(c as u32)) => {
            let ascii =
                char::from_u32(c as u32 - 0xFEE0).expect("fullwidth ASCII mapping is valid");
            output.push(ascii);
        }
        Action::NormalizeMaximum => output.push_str(maximum_replacement(c).unwrap_or("")),
        _ => output.push(c),
    }
}

const MAX_REPORTED_CHANGES: usize = 4096;

pub fn sanitize(text: &str, mode: Mode) -> EngineResult {
    let mut output = String::with_capacity(text.len());
    let mut removed_count = 0usize;
    let mut normalized_count = 0usize;
    let mut output_index = 0usize;
    let mut changes = Vec::new();
    for (source_index, c) in text.chars().enumerate() {
        let action = classify(c)
            .map(|rule| rule.action)
            .unwrap_or(Action::ReportOnly);
        if should_remove(mode, action) {
            removed_count += 1;
            if changes.len() < MAX_REPORTED_CHANGES {
                changes.push(EngineChange {
                    source_index,
                    output_index,
                    before: c.to_string(),
                    after: String::new(),
                    kind: "removed",
                });
            }
        } else if should_normalize(mode, action) {
            let start = output.len();
            append_normalized(&mut output, c, action);
            let replacement = output[start..].to_owned();
            if changes.len() < MAX_REPORTED_CHANGES {
                changes.push(EngineChange {
                    source_index,
                    output_index,
                    before: c.to_string(),
                    after: replacement.clone(),
                    kind: "normalized",
                });
            }
            output_index += replacement.chars().count();
            normalized_count += 1;
        } else {
            output.push(c);
            output_index += 1;
        }
    }
    EngineResult {
        output,
        removed_count,
        normalized_count,
        change_count: removed_count + normalized_count,
        changes,
    }
}

pub fn canonical_projection(text: &str) -> String {
    sanitize(text, Mode::Maximum).output
}

pub fn is_clean_for_mode(text: &str, mode: Mode) -> bool {
    text.chars().all(|c| {
        let action = classify(c)
            .map(|rule| rule.action)
            .unwrap_or(Action::ReportOnly);
        !should_remove(mode, action) && !should_normalize(mode, action)
    })
}
