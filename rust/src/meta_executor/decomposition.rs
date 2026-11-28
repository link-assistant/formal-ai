//! Mirrors `js/agentic/crate/task_decomposition_cues.mjs` using shared seed data.

use super::{Marker, lower_char, marker_matches, rule_values};
use crate::coding::contains_cjk;

pub(super) fn mask_quoted_cues(lowered: &[String], original: &[char], mask: &mut [bool]) {
    let separators = rule_values("quote-cue-separator");
    let openers: Vec<char> = rule_values("quote-pair")
        .iter()
        .filter_map(|pair| pair.chars().next())
        .collect();
    let cues: Vec<Marker> = rule_values("quote-cue-role")
        .iter()
        .flat_map(|role| crate::seed::lexicon().words_for_role(role))
        .map(|word| Marker {
            chars: word.chars().map(lower_char).collect(),
            cjk: contains_cjk(&word),
        })
        .filter(|cue| !cue.chars.is_empty())
        .collect();
    for at in 0..original.len() {
        for cue in &cues {
            if !marker_matches(lowered, original, mask, at, cue) {
                continue;
            }
            let end = at + cue.chars.len();
            let mut next = end;
            while next < original.len()
                && (original[next].is_whitespace()
                    || separators
                        .iter()
                        .any(|separator| separator.chars().eq([original[next]])))
            {
                next += 1;
            }
            if original
                .get(next)
                .is_some_and(|value| openers.contains(value))
                && mask.get(next) == Some(&true)
            {
                mask[at..end].fill(true);
                break;
            }
        }
    }
}
