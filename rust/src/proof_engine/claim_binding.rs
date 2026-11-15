//! Binding a proof request to the exact proposition of a library theorem
//! (issue #1487).
//!
//! A keyword of `data/seed/proof-library.lino` only proposes a theorem; it is
//! retrieval, not a check. The claim around the keyword decides what the
//! request asks:
//!
//! * [`ClaimBinding::Affirmed`] — no negation cue anywhere around the keyword:
//!   the request asks for the theorem itself;
//! * [`ClaimBinding::Negated`] — exactly one negation cue before the keyword,
//!   none after it, and only function words after it: the request asks for
//!   the theorem's negation, which a theorem marked `negation contradiction`
//!   refutes;
//! * [`ClaimBinding::Unsupported`] — any other placement or count of negation
//!   cues: the scope is not read, so the theorem is neither proven nor
//!   refuted.
//!
//! A negated request never receives the theorem's positive success. The cues
//! and function words are the seeded `statement_negation_cue` and
//! `statement_function_word` roles. The browser twin is
//! `js/agentic/crate/theorem_claim_binding.mjs`.

use crate::seed::{ProofLibraryEntry, proof_library};
use crate::summarization::vocabulary::{function_words, negation_cues, tokenize};

/// The seed file whose theorems a claim is bound to.
pub const PROOF_LIBRARY_PATH: &str = "data/seed/proof-library.lino";

/// What a claim asks of the theorem its keyword proposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimBinding {
    /// The claim is the theorem's proposition.
    Affirmed,
    /// The claim denies the theorem's proposition.
    Negated,
    /// The negation scope is not read; nothing is certified.
    Unsupported,
}

impl ClaimBinding {
    /// Slug for the `proof_binding` trace event.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Affirmed => "affirmed",
            Self::Negated => "negated",
            Self::Unsupported => "unsupported",
        }
    }
}

/// The library theorems in lookup order.
#[must_use]
pub fn library_theorems() -> &'static [ProofLibraryEntry] {
    &proof_library().theorems
}

/// The first theorem (lookup order) one of whose keywords occurs in `claim`,
/// with that keyword: `ProofLibrary::theorem_for` plus the keyword that
/// matched.
#[must_use]
pub fn theorem_match(claim: &str) -> Option<(&'static ProofLibraryEntry, &'static str)> {
    library_theorems().iter().find_map(|theorem| {
        theorem
            .keywords
            .iter()
            .find(|keyword| !keyword.is_empty() && claim.contains(keyword.as_str()))
            .map(|keyword| (theorem, keyword.as_str()))
    })
}

/// The seeded negation cues in `text`: a CJK run is scanned for the longest
/// cue at each position, any other token must equal a cue.
#[must_use]
pub fn negation_cue_count(text: &str) -> usize {
    let cues = negation_cues();
    let mut cjk: Vec<&String> = cues
        .iter()
        .filter(|cue| crate::coding::contains_cjk(cue))
        .collect();
    cjk.sort_by_key(|cue| std::cmp::Reverse(cue.chars().count()));
    let mut count = 0;
    for token in tokenize(text) {
        if !crate::coding::contains_cjk(&token) {
            if cues.iter().any(|cue| *cue == token) {
                count += 1;
            }
            continue;
        }
        let characters: Vec<char> = token.chars().collect();
        let mut index = 0;
        while index < characters.len() {
            let rest: String = characters[index..].iter().collect();
            match cjk.iter().find(|cue| rest.starts_with(cue.as_str())) {
                Some(cue) => {
                    count += 1;
                    index += cue.chars().count();
                }
                None => index += 1,
            }
        }
    }
    count
}

/// Whether every token of `text` is a seeded function word.
#[must_use]
pub fn only_function_words(text: &str) -> bool {
    let words = function_words();
    tokenize(text)
        .iter()
        .all(|token| words.iter().any(|word| word == token))
}

/// What `claim` asks of the theorem `keyword` proposes.
#[must_use]
pub fn bind_theorem_claim(claim: &str, keyword: &str) -> ClaimBinding {
    let Some(at) = claim.find(keyword).filter(|_| !keyword.is_empty()) else {
        return ClaimBinding::Unsupported;
    };
    let after = &claim[at + keyword.len()..];
    let before = negation_cue_count(&claim[..at]);
    let trailing = negation_cue_count(after);
    if before == 0 && trailing == 0 {
        return ClaimBinding::Affirmed;
    }
    if before == 1 && trailing == 0 && only_function_words(after) {
        return ClaimBinding::Negated;
    }
    ClaimBinding::Unsupported
}

/// The binding of `claim` to the theorem it proposes, if any.
#[must_use]
pub fn claim_binding(claim: &str) -> Option<ClaimBinding> {
    theorem_match(claim).map(|(_, keyword)| bind_theorem_claim(claim, keyword))
}
