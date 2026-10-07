//! Spelling correction by discovery (PR #1188 dogfooding).
//!
//! "Fix the typo 'smal' in README.md." states the misspelled word and not its
//! correction, so the correction is the unique most frequent word one edit
//! (Damerau: insert, delete, substitute, transpose) away from it in a
//! vocabulary counted from the bundled seed files -- the same files both roots
//! embed, read once and cached. No word list is authored for this. Twin of
//! `js/agentic/crate/spelling.mjs`.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Lowercase alphabetic word -> occurrence count across the seed bundle.
fn vocabulary() -> &'static HashMap<String, usize> {
    static VOCABULARY: OnceLock<HashMap<String, usize>> = OnceLock::new();
    VOCABULARY.get_or_init(|| {
        let mut counts = HashMap::new();
        for (_, text) in crate::seed::seed_files() {
            for word in text
                .split(|character: char| !character.is_alphabetic())
                .filter(|word| !word.is_empty())
            {
                *counts.entry(word.to_lowercase()).or_insert(0) += 1;
            }
        }
        counts
    })
}

/// Exactly one Damerau edit between `left` and `right`.
fn one_edit_apart(left: &str, right: &str) -> bool {
    let a: Vec<char> = left.chars().collect();
    let b: Vec<char> = right.chars().collect();
    if a.len().abs_diff(b.len()) > 1 || left == right {
        return false;
    }
    let start = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    if a.len() == b.len() {
        let rest = |from: usize| a.get(from..) == b.get(from..);
        return rest(start + 1)
            || (start + 1 < a.len()
                && a[start] == b[start + 1]
                && a[start + 1] == b[start]
                && rest(start + 2));
    }
    let (longer, shorter) = if a.len() > b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    longer.get(start + 1..) == shorter.get(start..)
}

/// The unique most frequent vocabulary word one edit from `word`, in
/// `word`'s capitalisation, or `None` when there is none or the best is tied.
pub(super) fn corrected_spelling(word: &str) -> Option<String> {
    let lowered = word.to_lowercase();
    let mut best: Option<(&str, usize)> = None;
    let mut tied = false;
    for (candidate, &count) in vocabulary() {
        if !one_edit_apart(&lowered, candidate) {
            continue;
        }
        match best {
            Some((_, best_count)) if count < best_count => {}
            Some((_, best_count)) if count == best_count => tied = true,
            _ => {
                best = Some((candidate.as_str(), count));
                tied = false;
            }
        }
    }
    let (best, _) = best.filter(|_| !tied)?;
    let first = word.chars().next()?;
    if first.to_lowercase().ne(first.to_string().chars()) {
        let mut characters = best.chars();
        let head = characters.next()?;
        Some(head.to_uppercase().chain(characters).collect())
    } else {
        Some(best.to_owned())
    }
}
