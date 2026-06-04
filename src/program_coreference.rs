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

#[cfg(test)]
mod tests {
    use super::looks_like_bare_program_artifact_follow_up;
    use crate::engine::normalize_prompt;

    #[test]
    fn additive_sort_follow_ups_are_recognized_in_every_language() {
        for prompt in [
            "sort the results in reverse order",
            "сделай сортировку результатов в обратном порядке",
            "परिणामों को उल्टे क्रम में क्रमबद्ध करो",
            "对结果倒序排序",
        ] {
            assert!(
                looks_like_bare_program_artifact_follow_up(&normalize_prompt(prompt)),
                "additive follow-up must be recognized: {prompt:?}"
            );
        }
    }

    #[test]
    fn cancel_sort_follow_ups_are_recognized_in_every_language() {
        // Issue #386: the subtractive follow-up must clear the same coreference
        // gate as the additive one it undoes, in every supported language.
        for prompt in [
            "cancel the sorting",
            "undo the sort",
            "Отмени сортировку",
            "убери сортировку",
            "सॉर्ट हटाओ",
            "取消排序",
        ] {
            assert!(
                looks_like_bare_program_artifact_follow_up(&normalize_prompt(prompt)),
                "cancel follow-up must be recognized: {prompt:?}"
            );
        }
    }

    #[test]
    fn unrelated_prompts_do_not_trip_the_gate() {
        for prompt in [
            "what is the capital of France",
            "напиши стихотворение про осень",
        ] {
            assert!(
                !looks_like_bare_program_artifact_follow_up(&normalize_prompt(prompt)),
                "unrelated prompt must not look like a program follow-up: {prompt:?}"
            );
        }
    }
}
