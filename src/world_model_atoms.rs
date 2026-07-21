//! Utterance → world-state atom extraction (issue #702).
//!
//! Issue #649 gave the project a symbolic world model ([`crate::world_model`]):
//! contexts that are links networks, their difference, action prediction. Issue
//! #702 wires that substrate into the *dialogue*, and the first thing the wiring
//! needs is a deterministic answer to one question: **which links network atom
//! does this sentence assert, and is it about the current state or the target
//! state?**
//!
//! Everything here is symbolic and data-driven:
//!
//! * the recognition vocabulary lives in `data/meta/cue-lexicon.lino` (the
//!   `world_state_*` cue sets), never in Rust string literals, so all four
//!   supported languages are extended by editing reviewable link data;
//! * the atom is a plain doublet — `subject -> state` — so a whole dialogue
//!   turns into links and nothing else (no embeddings, no graph/edge/vertex
//!   vocabulary);
//! * extraction is a pure function of the utterance: same sentence in, same
//!   atom out, on every platform.
//!
//! The split rule is intentionally language-general rather than a per-language
//! branch. A normalized utterance is reduced by (1) dropping the intent markers
//! ("I want", "what is left", "yes", "no"), (2) splitting on a declared state
//! separator (`is`, `是`, `है`) when one sits between two non-empty sides, and
//! (3) otherwise taking the first surviving token as the subject and the rest as
//! the state, after filler words are dropped. Chinese without a separator falls
//! back to a first-character subject split: coarse, but *consistent* between the
//! current and the target utterance, which is what a diff needs.

use crate::coding::contains_cjk;
use crate::cue_lexicon;
use crate::substitution::SubstitutionLink;
use crate::web_engine_core::normalize_prompt;

/// Cue set naming the "I want …" / imperative-request markers that route an
/// utterance into the **target** state.
pub const TARGET_CUES: &str = "world_state_target";
/// Cue set naming the "what is left to do?" state queries.
pub const QUERY_CUES: &str = "world_state_query";
/// Cue set naming the confirmations that accept a proposed target edit.
pub const CONFIRM_CUES: &str = "world_state_confirm";
/// Cue set naming the corrections that reject and replace a target edit.
pub const CORRECT_CUES: &str = "world_state_correct";
/// Cue set naming the causal connectives that make one statement depend on
/// another ("because", "потому что", "क्योंकि", "因为").
pub const BECAUSE_CUES: &str = "world_state_because";
/// Cue set naming the copulas that separate a subject from its state.
pub const SEPARATOR_CUES: &str = "world_state_separator";
/// Cue set naming the filler words dropped before the subject/state split.
pub const FILLER_CUES: &str = "world_state_filler";

/// Every cue set the extractor consults, in the order [`classify`] tests them.
pub const CONSULTED_CUE_SETS: &[&str] = &[
    QUERY_CUES,
    TARGET_CUES,
    CORRECT_CUES,
    CONFIRM_CUES,
    BECAUSE_CUES,
    SEPARATOR_CUES,
    FILLER_CUES,
];

/// What a dialogue utterance does to the world model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtteranceKind {
    /// A plain declarative fact: it lands in the current-state context.
    CurrentState,
    /// A wish or imperative request: it lands in the target-state context.
    TargetState,
    /// "yes, exactly": it accepts the pending target proposal.
    Confirmation,
    /// "no, actually …": it rejects the pending proposal and replaces it.
    Correction,
    /// "what is left to do?": it asks for the current→target difference.
    RemainingQuery,
    /// Nothing the world model can represent as an atom.
    Unrelated,
}

impl UtteranceKind {
    /// The stable slug used in traces and Links Notation.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::CurrentState => "current_state",
            Self::TargetState => "target_state",
            Self::Confirmation => "confirmation",
            Self::Correction => "correction",
            Self::RemainingQuery => "remaining_query",
            Self::Unrelated => "unrelated",
        }
    }
}

/// Classify one utterance against the `world_state_*` cue sets.
///
/// Query, target, and correction markers are explicit, so they are tested first;
/// a confirmation is only a confirmation when it carries no other marker. An
/// utterance with no marker at all is a current-state assertion whenever an atom
/// can be extracted from it, and unrelated otherwise.
#[must_use]
pub fn classify(text: &str) -> UtteranceKind {
    let normalized = normalize_prompt(text);
    if normalized.is_empty() {
        return UtteranceKind::Unrelated;
    }
    if cue_lexicon::matches(QUERY_CUES, &normalized) {
        return UtteranceKind::RemainingQuery;
    }
    if cue_lexicon::matches(TARGET_CUES, &normalized) {
        return UtteranceKind::TargetState;
    }
    if leading_cue(&normalized, CORRECT_CUES) {
        return UtteranceKind::Correction;
    }
    if leading_cue(&normalized, CONFIRM_CUES) {
        return UtteranceKind::Confirmation;
    }
    if state_atom(text).is_some() {
        UtteranceKind::CurrentState
    } else {
        UtteranceKind::Unrelated
    }
}

/// Extract the world-state atom an utterance asserts, as a links-network
/// doublet `subject -> state`.
///
/// Returns `None` when nothing survives marker and filler removal — a bare
/// "yes" asserts no atom.
#[must_use]
pub fn state_atom(text: &str) -> Option<SubstitutionLink> {
    let normalized = normalize_prompt(text);
    let body = strip_markers(&normalized);
    if body.is_empty() {
        return None;
    }
    if let Some((subject, state)) = split_on_separator(&body) {
        let from = canonical(&subject);
        let to = canonical(&state);
        if !from.is_empty() && !to.is_empty() {
            return Some(SubstitutionLink::new(from, to));
        }
    }
    let cleaned = strip_cues(&body, FILLER_CUES);
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    match tokens.len() {
        0 => None,
        1 => cjk_halves(tokens[0]).map(|(from, to)| SubstitutionLink::new(from, to)),
        _ => Some(SubstitutionLink::new(
            tokens[0].to_owned(),
            tokens[1..].join("_"),
        )),
    }
}

/// Split a causal utterance into `(consequent, premise)`.
///
/// The split happens around a declared connective, so the consequent statement
/// can be recorded as *depending on* the premise. Returns `None` when the
/// utterance states no cause.
#[must_use]
pub fn premise_split(text: &str) -> Option<(String, String)> {
    let normalized = normalize_prompt(text);
    let mut best: Option<(usize, usize)> = None;
    for cue in cue_lexicon::cues(BECAUSE_CUES) {
        if let Some(position) = normalized.find(cue.as_str()) {
            let candidate = (position, cue.len());
            if best.is_none_or(|(current, _)| position < current) {
                best = Some(candidate);
            }
        }
    }
    let (position, length) = best?;
    let consequent = normalized[..position].trim().to_owned();
    let premise = normalized[position + length..].trim().to_owned();
    (!consequent.is_empty() && !premise.is_empty()).then_some((consequent, premise))
}

/// Whether the utterance *opens* with a cue of `set`.
///
/// Confirmations and corrections are answers, and an answer leads with its
/// verdict ("no, the door is locked"). Anchoring at the front is the structural
/// glue that keeps the bare token `no` inside "there is no file" from being read
/// as a correction, while the cue strings themselves stay in the lexicon data.
fn leading_cue(normalized: &str, set: &str) -> bool {
    let cues = cue_lexicon::cues(set);
    let first = normalized.split_whitespace().next().unwrap_or_default();
    cues.iter()
        .any(|cue| cue == first || (contains_cjk(cue) && normalized.starts_with(cue.as_str())))
}

/// Drop every intent marker, leaving the assertion body.
fn strip_markers(normalized: &str) -> String {
    let mut body = normalized.to_owned();
    for set in [TARGET_CUES, QUERY_CUES] {
        body = strip_cues(&body, set);
    }
    // A verdict marker only counts at the front (see `leading_cue`), so only the
    // front is removed — "there is no file" keeps its "no".
    for set in [CONFIRM_CUES, CORRECT_CUES] {
        body = strip_leading_cue(&body, set);
    }
    body
}

/// Drop a leading cue of `set` from the body, if it opens with one.
fn strip_leading_cue(body: &str, set: &str) -> String {
    let cues = cue_lexicon::cues(set);
    if let Some(cue) = cues
        .iter()
        .find(|cue| contains_cjk(cue) && body.starts_with(cue.as_str()))
    {
        return body[cue.len()..].trim().to_owned();
    }
    let mut tokens = body.split_whitespace();
    let first = tokens.next().unwrap_or_default();
    if cues.iter().any(|cue| cue == first) {
        return tokens.collect::<Vec<_>>().join(" ");
    }
    body.to_owned()
}

/// Remove every cue of `set` from `text`: multi-word and CJK cues by substring
/// (CJK has no whitespace boundaries), single-word cues as whole tokens so
/// "a" never eats the "a" inside "table".
fn strip_cues(text: &str, set: &str) -> String {
    let cues = cue_lexicon::cues(set);
    let mut body = text.to_owned();
    for cue in cues {
        if contains_cjk(cue) || cue.contains(' ') {
            body = body.replace(cue.as_str(), " ");
        }
    }
    body.split_whitespace()
        .filter(|token| !cues.iter().any(|cue| cue == token))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Split on the first declared state separator that has non-empty text on both
/// sides. CJK separators are matched inside the character stream; the rest are
/// matched as whole tokens.
fn split_on_separator(body: &str) -> Option<(String, String)> {
    let separators = cue_lexicon::cues(SEPARATOR_CUES);
    for cue in separators.iter().filter(|cue| contains_cjk(cue)) {
        if let Some(position) = body.find(cue.as_str()) {
            let subject = body[..position].trim();
            let state = body[position + cue.len()..].trim();
            if !subject.is_empty() && !state.is_empty() {
                return Some((subject.to_owned(), state.to_owned()));
            }
        }
    }
    let tokens: Vec<&str> = body.split_whitespace().collect();
    for (index, token) in tokens.iter().enumerate() {
        let is_separator = separators.iter().any(|cue| cue == token);
        if is_separator && index > 0 && index + 1 < tokens.len() {
            return Some((tokens[..index].join(" "), tokens[index + 1..].join(" ")));
        }
    }
    None
}

/// Reduce one side of a split to a single atom term: fillers dropped, remaining
/// words joined by `_` so the term is one links-network node.
fn canonical(part: &str) -> String {
    strip_cues(part, FILLER_CUES)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
}

/// Coarse fallback for a separator-less CJK utterance: the first character is
/// the subject, the rest is the state. Deterministic, and identical for the
/// current-state and target-state phrasings of the same subject, which is what
/// the difference needs.
fn cjk_halves(token: &str) -> Option<(String, String)> {
    if !contains_cjk(token) {
        return None;
    }
    let characters: Vec<char> = token.chars().collect();
    if characters.len() < 2 {
        return None;
    }
    Some((
        characters[..1].iter().collect(),
        characters[1..].iter().collect(),
    ))
}
