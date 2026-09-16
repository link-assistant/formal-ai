//! TRIZ contradictions as links with a 0-1 selection value (#901; plan 12
//! leaves 15-16).
//!
//! Two candidates that each win on a different cost dimension are a *technical
//! contradiction*, not a tie to be broken by the arbitrary index. The point on
//! the range is counted from the requirement's own clauses — each clause that
//! demands a criterion is one vote for it — and it is stored as integer basis
//! points, so a content id stays exactly comparable and hashable. A requirement
//! that states no trade-off leaves the contradiction named and unresolved. There
//! is no default 50 %.
//!
//! The criterion vocabulary is seed data
//! (`data/seed/meanings-selection-criteria.lino`), so the clause counting reads
//! the same surfaces in every supported language and no spelling appears here.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::engine::stable_id;
use crate::seed::{
    self, ROLE_SELECTION_CRITERION_BREVITY_CUE, ROLE_SELECTION_CRITERION_COMPLETENESS_CUE,
};
use crate::web_engine_core::normalize_prompt;

use super::{
    CandidateScore, ContradictionDerivation, ContradictionLink, TrizResolution,
};

/// The cost dimensions a candidate can win on, in the order a contradiction
/// names them. Declaration order here is the order two criteria are paired in,
/// so the same candidate set always produces the same `criterion_a`.
const DIMENSIONS: [&str; 4] = ["code_size", "steps", "resource_units", "leaf_count"];

/// The criterion meaning each cost dimension serves.
///
/// A dimension is a *measure*; a criterion is what a requirement asks for in its
/// own words. `code_size` serves brevity directly. `steps` is the measure that
/// rises when an answer walks more ground, so the requirement pressure it trades
/// against is completeness — which is why a requirement that says "favour
/// completeness" moves the value toward the candidate that spent fewer steps
/// reaching the same checks only after the clause count says so, never before.
fn criterion_for(dimension: &str) -> &'static str {
    match dimension {
        "code_size" => "answer_brevity",
        "steps" => "answer_completeness",
        "resource_units" => "resource_economy",
        _ => "decomposition_depth",
    }
}

/// The cue role whose clauses count toward `criterion`.
fn cue_role_for(criterion: &str) -> Option<&'static str> {
    match criterion {
        "answer_brevity" => Some(ROLE_SELECTION_CRITERION_BREVITY_CUE),
        "answer_completeness" => Some(ROLE_SELECTION_CRITERION_COMPLETENESS_CUE),
        _ => None,
    }
}

/// One measured dimension of a candidate, so the comparison is data-driven.
fn measure(score: &CandidateScore, dimension: &str) -> u128 {
    score.dimension(dimension, 0)
}

/// Byte ranges of the requirement's clauses, cut on list and sentence
/// punctuation only. A clause is the unit a demand is counted in.
fn clause_spans(requirement: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = 0;
    for (offset, character) in requirement.char_indices() {
        if matches!(
            character,
            ',' | ';' | '.' | '!' | '?' | '，' | '；' | '。' | '！' | '？' | '、' | '।' | '॥'
        ) {
            let end = offset + character.len_utf8();
            if requirement[start..end].trim().len() > 1 {
                spans.push((start, end));
            }
            start = end;
        }
    }
    if requirement[start..].trim().len() > 1 {
        spans.push((start, requirement.len()));
    }
    spans
}

/// Clauses of `requirement` that demand `criterion`, as `start-end` spans.
fn clauses_demanding(requirement: &str, criterion: &str) -> Vec<String> {
    let Some(role) = cue_role_for(criterion) else {
        return Vec::new();
    };
    let lexicon = seed::lexicon();
    clause_spans(requirement)
        .into_iter()
        .filter(|(start, end)| {
            let normalized = normalize_prompt(&requirement[*start..*end]);
            lexicon.mentions_role_raw(role, &normalized)
        })
        .map(|(start, end)| format!("clause:{start}-{end}"))
        .collect()
}

/// The reason recorded when no clause demands either criterion.
///
/// A slug, not a sentence: the surface renders the refusal from seed prose, so
/// the core carries no natural-language literal.
const NO_STATED_TRADE_OFF: &str = "no_clause_demands_either_criterion";

/// Detect the contradictions in a scored candidate set.
pub(super) fn contradictions_in(
    scores: &[CandidateScore],
    requirement: &str,
) -> Vec<ContradictionLink> {
    let satisfying: Vec<&CandidateScore> =
        scores.iter().filter(|score| score.satisfies()).collect();
    let situation = stable_id("situation", requirement);
    let mut links = Vec::new();
    for left_index in 0..satisfying.len() {
        for right_index in (left_index + 1)..satisfying.len() {
            let left = satisfying[left_index];
            let right = satisfying[right_index];
            let left_wins = DIMENSIONS
                .iter()
                .find(|dimension| measure(left, dimension) < measure(right, dimension));
            let right_wins = DIMENSIONS
                .iter()
                .find(|dimension| measure(right, dimension) < measure(left, dimension));
            let (Some(left_dimension), Some(right_dimension)) = (left_wins, right_wins) else {
                continue;
            };
            let criterion_a = criterion_for(left_dimension);
            let criterion_b = criterion_for(right_dimension);
            links.push(link_for(criterion_a, criterion_b, &situation, requirement));
        }
    }
    links
}

/// Build one contradiction link, counting its selection value from the
/// requirement's clauses or reporting honestly that nothing could be counted.
fn link_for(
    criterion_a: &str,
    criterion_b: &str,
    situation: &str,
    requirement: &str,
) -> ContradictionLink {
    let a_evidence = clauses_demanding(requirement, criterion_a);
    let b_evidence = clauses_demanding(requirement, criterion_b);
    let a_clauses = a_evidence.len();
    let b_clauses = b_evidence.len();
    let link_id = stable_id(
        "contradiction",
        &format!("{criterion_a}:{criterion_b}:{situation}"),
    );
    if a_clauses + b_clauses == 0 {
        return ContradictionLink {
            link_id,
            criterion_a: criterion_a.into(),
            criterion_b: criterion_b.into(),
            // Reported, never applied: the derivation says the value was not
            // derivable, and the resolution refuses to invent one.
            selection_basis_points: 0,
            derivation: ContradictionDerivation::Underivable {
                reason: NO_STATED_TRADE_OFF.into(),
            },
            resolution: TrizResolution::Unresolved {
                reason: NO_STATED_TRADE_OFF.into(),
            },
            evidence: Vec::new(),
        };
    }
    let total = a_clauses + b_clauses;
    let basis_points = u16::try_from(b_clauses * 10_000 / total).unwrap_or(10_000);
    let mut evidence = a_evidence;
    evidence.extend(b_evidence);
    ContradictionLink {
        link_id,
        criterion_a: criterion_a.into(),
        criterion_b: criterion_b.into(),
        selection_basis_points: basis_points,
        derivation: ContradictionDerivation::RequirementClauses {
            a_clauses,
            b_clauses,
        },
        resolution: TrizResolution::Range,
        evidence,
    }
}

/// Rank by resolving the detected contradictions instead of by the index
/// tie-break: the candidate that wins the criterion the requirement leans toward
/// comes first, and a contradiction whose value is underivable changes nothing.
pub(super) fn rank(
    scores: &[CandidateScore],
    parameters: &[(String, String)],
) -> Vec<usize> {
    use super::{CandidateRanker, LeastActionRanker};
    let ranked = LeastActionRanker.rank(scores, parameters);
    let requirement = parameters
        .iter()
        .find(|(key, _)| key == "requirement")
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    let links = contradictions_in(scores, requirement);
    let Some(link) = links
        .iter()
        .find(|link| matches!(link.resolution, TrizResolution::Range))
    else {
        return ranked;
    };
    let leaning_dimension = if link.selection_basis_points > 5_000 {
        dimension_of(&link.criterion_b)
    } else {
        dimension_of(&link.criterion_a)
    };
    let mut resolved = ranked;
    resolved.sort_by_key(|index| measure(&scores[*index], leaning_dimension));
    resolved
}

/// The cost dimension a criterion is measured on.
fn dimension_of(criterion: &str) -> &'static str {
    DIMENSIONS
        .iter()
        .copied()
        .find(|dimension| criterion_for(dimension) == criterion)
        .unwrap_or(DIMENSIONS[0])
}
