// STATUS: DIAMANT VGT SUPREME

use super::model::{Action, Rule, Severity};

const fn rule(
    name: &'static str,
    category: &'static str,
    severity: Severity,
    action: Action,
) -> Rule {
    Rule {
        name,
        category,
        severity,
        action,
    }
}

pub fn classify(c: char) -> Option<Rule> {
    let matched = match c as u32 {
        0x0000..=0x0008 | 0x000B..=0x000C | 0x000E..=0x001F | 0x007F..=0x009F => rule(
            "CONTROL CHARACTER",
            "Control",
            Severity::High,
            Action::RemoveStrict,
        ),
        0x00AD => rule(
            "SOFT HYPHEN",
            "Invisible format",
            Severity::Medium,
            Action::RemoveSafe,
        ),
        0x034F => rule(
            "COMBINING GRAPHEME JOINER",
            "Invisible format",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0x061C => rule(
            "ARABIC LETTER MARK",
            "Bidirectional control",
            Severity::Medium,
            Action::RemoveStrict,
        ),
        0x070F => rule(
            "SYRIAC ABBREVIATION MARK",
            "Invisible format",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0x0890..=0x0891 | 0x08E2 => rule(
            "ARABIC FORMAT MARK",
            "Invisible format",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0x115F..=0x1160 => rule(
            "HANGUL FILLER",
            "Invisible filler",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0x17B4..=0x17B5 => rule(
            "KHMER INHERENT VOWEL",
            "Invisible format",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0x180B..=0x180D | 0x180F => rule(
            "MONGOLIAN VARIATION SELECTOR",
            "Variation selector",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0x180E => rule(
            "MONGOLIAN VOWEL SEPARATOR",
            "Invisible format",
            Severity::Medium,
            Action::RemoveStrict,
        ),
        0x200B => rule(
            "ZERO WIDTH SPACE",
            "Zero-width",
            Severity::High,
            Action::RemoveSafe,
        ),
        0x200C => rule(
            "ZERO WIDTH NON-JOINER",
            "Zero-width",
            Severity::Medium,
            Action::RemoveStrict,
        ),
        0x200D => rule(
            "ZERO WIDTH JOINER",
            "Zero-width",
            Severity::Medium,
            Action::RemoveStrict,
        ),
        0x200E..=0x200F => rule(
            "DIRECTIONAL MARK",
            "Bidirectional control",
            Severity::Medium,
            Action::RemoveStrict,
        ),
        0x202A..=0x202C | 0x2066..=0x2069 => rule(
            "BIDI EMBEDDING OR ISOLATE",
            "Bidirectional control",
            Severity::High,
            Action::RemoveStrict,
        ),
        0x202D..=0x202E => rule(
            "BIDI OVERRIDE",
            "Bidirectional override",
            Severity::High,
            Action::RemoveStrict,
        ),
        0x2060 => rule(
            "WORD JOINER",
            "Zero-width",
            Severity::Medium,
            Action::RemoveSafe,
        ),
        0x2061..=0x2064 => rule(
            "INVISIBLE OPERATOR",
            "Invisible operator",
            Severity::Medium,
            Action::RemoveStrict,
        ),
        0x206A..=0x206F => rule(
            "DEPRECATED BIDI CONTROL",
            "Bidirectional control",
            Severity::Medium,
            Action::RemoveStrict,
        ),
        0xFEFF => rule(
            "ZERO WIDTH NO-BREAK SPACE / BOM",
            "Zero-width",
            Severity::High,
            Action::RemoveSafe,
        ),
        0xFFF9..=0xFFFB => rule(
            "INTERLINEAR ANNOTATION CONTROL",
            "Annotation control",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0x1BCA0..=0x1BCAF => rule(
            "SHORTHAND FORMAT CONTROL",
            "Invisible format",
            Severity::Medium,
            Action::RemoveStrict,
        ),
        0x1D173..=0x1D17A => rule(
            "MUSICAL SYMBOL CONTROL",
            "Invisible format",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0xE0000..=0xE007F => rule(
            "UNICODE TAG CHARACTER",
            "Tag character",
            Severity::High,
            Action::RemoveSafe,
        ),
        0xFE00..=0xFE0F | 0xE0100..=0xE01EF => rule(
            "VARIATION SELECTOR",
            "Variation selector",
            Severity::Low,
            Action::RemoveStrict,
        ),
        0x00A0 | 0x1680 | 0x2000..=0x200A | 0x202F | 0x205F | 0x3000 => rule(
            "NON-STANDARD SPACE",
            "Suspicious spacing",
            Severity::Info,
            Action::NormalizeStrict,
        ),
        0x2018..=0x201B => rule(
            "TYPOGRAPHIC SINGLE QUOTE",
            "Surface typography",
            Severity::Info,
            Action::NormalizeMaximum,
        ),
        0x201C..=0x201F => rule(
            "TYPOGRAPHIC DOUBLE QUOTE",
            "Surface typography",
            Severity::Info,
            Action::NormalizeMaximum,
        ),
        0x2010..=0x2015 | 0x2212 => rule(
            "DASH VARIANT",
            "Surface typography",
            Severity::Info,
            Action::NormalizeMaximum,
        ),
        0x2026 => rule(
            "HORIZONTAL ELLIPSIS",
            "Surface typography",
            Severity::Info,
            Action::NormalizeMaximum,
        ),
        0xFF01..=0xFF5E => rule(
            "FULLWIDTH ASCII VARIANT",
            "Compatibility character",
            Severity::Low,
            Action::NormalizeMaximum,
        ),
        0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD => rule(
            "PRIVATE USE CHARACTER",
            "Private-use payload",
            Severity::High,
            Action::RemoveMaximum,
        ),
        _ => return None,
    };
    Some(matched)
}

pub fn maximum_replacement(c: char) -> Option<&'static str> {
    match c as u32 {
        0x2018..=0x201B => Some("'"),
        0x201C..=0x201F => Some("\""),
        0x2010..=0x2015 | 0x2212 => Some("-"),
        0x2026 => Some("..."),
        _ => None,
    }
}
