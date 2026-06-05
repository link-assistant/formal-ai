//! Program-artifact coreference markers shared by write-program recovery.
//!
//! Issue #386: this gate used to enumerate ~100 per-language words in two
//! hardcoded Rust arrays. It now references **meanings** instead. A bare
//! follow-up looks like a program-artifact reference when the prompt mentions
//! some meaning that plays the [`ROLE_PROGRAM_ARTIFACT`] role (a result, an
//! output, the program/script/code itself, an ordering) *and* some meaning
//! that plays the [`ROLE_PROGRAM_MODIFICATION`] role (sort/reverse/cancel/…).
//! The surface words for every language live once, in `data/seed/meanings.lino`;
//! this code understands the *concepts*, not the words.

use crate::seed::{lexicon, ROLE_PROGRAM_ARTIFACT, ROLE_PROGRAM_MODIFICATION};

/// True when `normalized` reads like a bare follow-up that modifies an existing
/// program artifact — e.g. "cancel the sorting", "Отмени сортировку",
/// "对结果倒序排序" — in any supported language.
#[must_use]
pub fn looks_like_bare_program_artifact_follow_up(normalized: &str) -> bool {
    let lexicon = lexicon();
    lexicon.mentions_role(ROLE_PROGRAM_ARTIFACT, normalized)
        && lexicon.mentions_role(ROLE_PROGRAM_MODIFICATION, normalized)
}

#[path = "source_tests/program_coreference/tests.rs"]
mod tests;
