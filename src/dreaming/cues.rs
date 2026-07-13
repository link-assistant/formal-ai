//! Requirement-cue phrases grounded as data (issue #540).
//!
//! The cues live in `data/meta/dreaming-cues.lino`. They are loaded from disk
//! when the file is present (honouring `FORMAL_AI_DATA_DIR`) so deployments can
//! extend them without recompiling, fall back to the compiled-in copy
//! otherwise, and the parse is cached process-wide instead of being redone for
//! every scanned event.

use std::sync::OnceLock;

use super::lexicon::load_data_document;

const EMBEDDED_CUES: &str = include_str!("../../data/meta/dreaming-cues.lino");

/// Parse the `cue "..."` lines of a dreaming-cues document.
#[must_use]
pub fn parse_requirement_cues(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("cue \"")?.strip_suffix('"'))
        .map(str::to_lowercase)
        .collect()
}

/// The process-wide requirement cue list, parsed once.
pub fn requirement_cues() -> &'static [String] {
    static CUES: OnceLock<Vec<String>> = OnceLock::new();
    CUES.get_or_init(|| {
        parse_requirement_cues(&load_data_document("dreaming-cues.lino", EMBEDDED_CUES))
    })
}
