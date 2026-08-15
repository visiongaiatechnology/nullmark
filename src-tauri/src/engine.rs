// STATUS: DIAMANT VGT SUPREME

#[path = "engine/model.rs"]
mod model;
#[path = "engine/rules.rs"]
mod rules;
#[path = "engine/sanitizer.rs"]
mod sanitizer;

pub use model::{Action, EngineChange, Mode, Severity};
pub use rules::classify;
pub use sanitizer::{canonical_projection, is_clean_for_mode, sanitize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unicode_payloads() {
        assert_eq!(
            classify('\u{200B}').map(|rule| rule.action),
            Some(Action::RemoveSafe)
        );
        assert_eq!(
            classify('\u{E0061}').map(|rule| rule.action),
            Some(Action::RemoveSafe)
        );
        assert_eq!(
            classify('\u{E123}').map(|rule| rule.action),
            Some(Action::RemoveMaximum)
        );
    }

    #[test]
    fn mode_policies_are_correct() {
        let input = "A\u{200B}B\u{200D}C\u{00A0}D\u{E123}E";
        assert_eq!(
            sanitize(input, Mode::Safe).output,
            "AB\u{200D}C\u{00A0}D\u{E123}E"
        );
        assert_eq!(sanitize(input, Mode::Strict).output, "ABC D\u{E123}E");
        assert_eq!(sanitize(input, Mode::Maximum).output, "ABC DE");
    }

    #[test]
    fn maximum_canonicalizes_surface() {
        let result = sanitize("\u{201C}Ａ—B\u{2026}\u{201D}", Mode::Maximum);
        assert_eq!(result.output, "\"A-B...\"");
        assert_eq!(result.normalized_count, 5);
        assert!(is_clean_for_mode(&result.output, Mode::Maximum));
    }

    #[test]
    fn modes_are_idempotent_and_projection_stable() {
        let input = "A\u{200B}\u{200D}\u{00A0}\u{2014}Ｂ\u{E123}Z";
        let projection = canonical_projection(input);
        for mode in [Mode::Safe, Mode::Strict, Mode::Maximum] {
            let once = sanitize(input, mode);
            let twice = sanitize(&once.output, mode);
            assert_eq!(once.output, twice.output);
            assert_eq!((twice.removed_count, twice.normalized_count), (0, 0));
            assert_eq!(projection, canonical_projection(&once.output));
        }
    }

    #[test]
    fn strict_preserves_tab_and_newline() {
        assert_eq!(
            sanitize("A\u{0000}B\tC\nD", Mode::Strict).output,
            "AB\tC\nD"
        );
    }

    #[test]
    fn safe_preserves_multilingual_text() {
        let input = "Deutsch, العربية, हिन्दी, 中文, emoji 🛡️";
        assert_eq!(sanitize(input, Mode::Safe).output, input);
    }

    #[test]
    fn change_ledger_maps_exact_source_and_output_positions() {
        let result = sanitize("A\u{200B}\u{2014}B", Mode::Maximum);
        assert_eq!(result.output, "A-B");
        assert_eq!(result.change_count, 2);
        assert_eq!(result.changes.len(), 2);
        assert_eq!(result.changes[0].source_index, 1);
        assert_eq!(result.changes[0].output_index, 1);
        assert_eq!(result.changes[0].before, "\u{200B}");
        assert_eq!(result.changes[0].after, "");
        assert_eq!(result.changes[0].kind, "removed");
        assert_eq!(result.changes[1].source_index, 2);
        assert_eq!(result.changes[1].output_index, 1);
        assert_eq!(result.changes[1].before, "—");
        assert_eq!(result.changes[1].after, "-");
        assert_eq!(result.changes[1].kind, "normalized");
    }
}
