//! Approximate resolution of proper names against everything the system
//! already remembers.
//!
//! Issue #699 batch 2 migrates the `who_is` method. Its recognition was already
//! seed-role driven, but its *body* carried a fixed enumeration: eight people
//! with three hand-written misspellings each. That table could only correct
//! typos an author had anticipated, and adding a person meant editing Rust.
//!
//! What stays in Rust here is one language-neutral primitive: nearest-surface
//! search under a length-scaled edit-distance budget. What moved to data is
//! every name. Candidate surfaces are drawn from three memories that already
//! existed — the canonical name registry (`data/seed/entity-names.lino`),
//! concept terms/aliases including their localized variants, and fact subject
//! and value labels — so any name the store learns becomes correctable without
//! a code change, and no misspelling is ever stored.

use std::sync::OnceLock;

use crate::concepts::extract_concept_query;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::fuzzy::typo_distance;
use crate::language::detect as detect_language;
use crate::seed::{self, response_for, Slot, WordForm};
use crate::solver_handlers::finalize_simple;

/// Placeholders the seed response records carry.
const TERM_PLACEHOLDER: &str = "{term}";
const CORRECTED_PLACEHOLDER: &str = "{corrected}";

/// Every correctly spelled name surface the system remembers.
///
/// The order is stable: canonical registry first, then concept terms and
/// aliases, then fact labels. Deduplicated case-insensitively, keeping the
/// first spelling seen so suggestions render with their canonical casing.
#[must_use]
pub fn known_entity_names() -> &'static [String] {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(collect_known_entity_names).as_slice()
}

fn collect_known_entity_names() -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut push = |candidate: &str| {
        let trimmed = candidate.trim();
        if trimmed.is_empty() || !looks_like_a_name(trimmed) {
            return;
        }
        if !names
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(trimmed))
        {
            names.push(trimmed.to_owned());
        }
    };

    for entity in seed::entity_names() {
        for surface in &entity.surfaces {
            push(surface);
        }
    }
    for concept in seed::concepts() {
        push(&concept.term);
        for alias in &concept.aliases {
            push(alias);
        }
        for localized in &concept.localized {
            push(&localized.term);
            for alias in &localized.aliases {
                push(alias);
            }
        }
    }
    for fact in seed::facts() {
        push(&fact.subject_label);
        push(&fact.value_label);
        for localized in &fact.localized {
            push(&localized.subject_label);
            push(&localized.value_label);
        }
    }
    names
}

/// A stored label is usable as a name candidate when it is short enough to be
/// a name rather than a sentence. This is a shape test, not a language test:
/// no word list is consulted.
fn looks_like_a_name(candidate: &str) -> bool {
    let words = candidate.split_whitespace().count();
    (1..=4).contains(&words) && candidate.chars().count() <= 48
}

/// Suggest the remembered name closest to `term`, or `None` when `term` is
/// already spelled exactly like one or when nothing is close enough.
///
/// The budget scales with length (one edit per eight characters, at least one)
/// so that long names tolerate the extra slips long names attract, while short
/// ones do not collapse into each other. Among candidates within budget the
/// smallest distance wins; ties keep the earlier-remembered spelling, which
/// makes the result independent of iteration timing.
#[must_use]
pub fn suggest_known_name(term: &str) -> Option<String> {
    let probe = normalize_name(term);
    if probe.is_empty() {
        return None;
    }
    let mut best: Option<(usize, &str)> = None;
    for candidate in known_entity_names() {
        let normalized = normalize_name(candidate);
        if normalized == probe {
            // Already spelled correctly: there is nothing to correct.
            return None;
        }
        let budget = edit_budget(&normalized);
        let distance = typo_distance(&probe, &normalized);
        if distance > budget {
            continue;
        }
        if best.is_none_or(|(best_distance, _)| distance < best_distance) {
            best = Some((distance, candidate.as_str()));
        }
    }
    best.map(|(_, candidate)| candidate.to_owned())
}

/// Compare names on their letters alone: casing and punctuation such as the
/// dots in initialisms are spelling noise, not evidence of a different person.
fn normalize_name(term: &str) -> String {
    let mut normalized = String::new();
    let mut pending_space = false;
    for character in term.to_lowercase().chars() {
        if character.is_alphanumeric() {
            if pending_space && !normalized.is_empty() {
                normalized.push(' ');
            }
            pending_space = false;
            normalized.push(character);
        } else {
            pending_space = true;
        }
    }
    normalized
}

fn edit_budget(candidate: &str) -> usize {
    (candidate.chars().count() / 8).max(1)
}

/// The literal lead-in of every prefix-slot form of a role, in lexicon
/// declaration order.
fn prefix_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .map(WordForm::before_slot)
        .collect()
}

/// The literal tail of every suffix-slot form of a role, for languages whose
/// question marker trails the topic (Hindi `… कौन है`, Chinese `…是谁`).
fn suffix_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Suffix)
        .map(WordForm::after_slot)
        .collect()
}

/// Answer a "who is X" prompt the concept lookup could not claim.
///
/// The entity is not in the knowledge base, so acknowledge the question form,
/// report the miss, and offer the nearest remembered spelling when the term
/// looks like a typo of one.
pub fn resolve_who_is(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let is_who_question = prefix_literals(seed::ROLE_WHO_QUESTION_LEAD)
        .iter()
        .any(|&lead| normalized.starts_with(lead))
        || suffix_literals(seed::ROLE_WHO_QUESTION_TAIL)
            .iter()
            .any(|&tail| normalized.ends_with(tail));
    if !is_who_question {
        return None;
    }
    let query = extract_concept_query(prompt)?;
    let term = &query.term;
    log.append("concept_lookup:miss", term.clone());

    let language = query
        .response_language
        .clone()
        .unwrap_or_else(|| detect_language(prompt).slug().to_owned());
    let suggestion = suggest_known_name(term);
    let intent = if suggestion.is_some() {
        "who_is_unknown_entity_suggestion"
    } else {
        "who_is_unknown_entity"
    };
    let mut body = response_for(intent, &language)
        .or_else(|| response_for(intent, "en"))?
        .replace(TERM_PLACEHOLDER, term);
    if let Some(corrected) = suggestion {
        log.append("entity_resolution:suggestion", corrected.clone());
        body = body.replace(CORRECTED_PLACEHOLDER, &corrected);
    }

    Some(finalize_simple(
        prompt,
        log,
        "who_is_question",
        "response:who_is_question",
        &body,
        0.5,
    ))
}
