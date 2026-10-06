//! The reasoner inside the universal solver: it runs ahead of the handler
//! table, and a handler that admits an impasse hands its turn to the loop.
//!
//! Originals: `solve` and `solveImpl` in `js/worker/formal_ai_worker_20.js`, and
//! `metaReasonTurn` / `metaResolveImpasse` in
//! `js/worker/formal_ai_worker_meta_reasoner.js`.

use crate::concept_lookup::lookup_surface;
use crate::engine::{SymbolicAnswer, answer_links_notation};
use crate::event_log::{EventLog, build_evidence_links};
use crate::how_to_guide::ServicePreferences;
use crate::service_accessibility::{ServiceAccessibilityCache, unix_now};
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};
use crate::source_walk::LookupBounds;

use super::answer::{MetaAnswer, meta_answer, meta_reason};
use super::grounding::{Knowledge, Sense};
use super::phrases::{GOAL_STATUS, IMPASSE_HANDOFF, fill};
use super::reasoner::MetaResult;
use super::seed::meta_seed;

/// The response link every meta-reasoned answer records.
const RESPONSE_LINK: &str = "response:meta_reasoner";

/// True when a handler's answer admits it could not do the task.
///
/// Mirrors `metaIsImpasseIntent` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_is_impasse_intent(intent: &str) -> bool {
    meta_seed().is_impasse_intent(intent)
}

/// The turn entry with discovery: words the core could not ground are
/// looked up through the registry concept lookup, reading the committed
/// source cache only (no network), and the core runs again.
///
/// Mirrors `metaReasonTurn` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[must_use]
pub fn meta_reason_turn(prompt: &str, language: &str) -> MetaResult {
    let cache_root = crate::coding::synthesis_runtime::source_cache_root();
    let client = CachedSourceClient::new(&cache_root, CurlSourceTransport).with_online(false);
    let preferences = ServicePreferences::default();
    let mut availability = ServiceAccessibilityCache::load(&cache_root);
    let bounds = LookupBounds::default();
    let now = unix_now();
    let mut lookup = |word: &str, language: &str| -> Vec<Sense> {
        lookup_surface(
            word,
            language,
            &client,
            &preferences,
            &bounds,
            &mut availability,
            now,
        )
        .items
        .into_iter()
        .map(|sense| Sense {
            gloss: sense.gloss,
            source_url: sense.source_url,
        })
        .collect()
    };
    let mut knowledge = Knowledge::new();
    meta_reason(prompt, language, &mut knowledge, Some(&mut lookup))
}

/// Record the answer on the log and project it, the way every solver route
/// finishes.
fn project(prompt: &str, log: &mut EventLog, answer: MetaAnswer) -> SymbolicAnswer {
    log.append("intent", answer.intent.clone());
    log.append("response", RESPONSE_LINK.to_owned());
    let trace_id = log.append("trace", answer.intent.clone());
    let mut evidence_links = build_evidence_links(prompt, log, RESPONSE_LINK);
    evidence_links.extend(answer.evidence);
    let links_notation =
        answer_links_notation(prompt, &answer.intent, &answer.content, log, &trace_id);
    SymbolicAnswer {
        thinking_steps: log.thinking_steps_for_answer(&answer.content),
        intent: answer.intent,
        answer: answer.content,
        confidence: answer.confidence,
        evidence_links,
        links_notation,
        execution_recipe: None,
    }
}

/// The general loop at the start of a turn.
///
/// The request's words are opened and grounded without network; a program
/// verified against the request's examples, or one every word grounds, answers
/// ahead of any handler.
///
/// Mirrors the `metaReason(prompt, language, {})` step at the top of
/// `solveImpl` in `js/worker/formal_ai_worker_20.js`.
pub fn try_meta_answer(prompt: &str, language: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let mut knowledge = Knowledge::new();
    let meta = meta_reason(prompt, language, &mut knowledge, None);
    if meta.status != "solved" || (meta.program.is_none() && meta.subgoals.is_none()) {
        return None;
    }
    let answer = meta_answer(&meta, false)?;
    log.append(
        "meta_reason",
        fill(GOAL_STATUS, &[&meta.goal, &meta.status]),
    );
    log.append("meta_derivation", meta.derivation_lino);
    Some(project(prompt, log, answer))
}

/// Discovery before research: when no handler took the turn and the request
/// asks for an artifact, the loop looks up the words it could not ground and
/// answers if a program now derives, before any external research runs.
///
/// Mirrors the `metaReasonTurn` step ahead of `unknown_intent_research` in
/// `solveImpl` (`js/worker/formal_ai_worker_20.js`).
pub fn try_meta_discovery(
    prompt: &str,
    language: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let first = meta_reason(prompt, language, &mut Knowledge::new(), None);
    if !matches!(
        first.goal.as_str(),
        "synthesize_from_examples" | "synthesize_from_meaning" | "decompose"
    ) {
        return None;
    }
    let meta = meta_reason_turn(prompt, language);
    let answer = meta_answer(&meta, false)?;
    log.append(
        "meta_discover",
        fill(GOAL_STATUS, &[&meta.goal, &meta.status]),
    );
    log.append("meta_derivation", meta.derivation_lino);
    Some(project(prompt, log, answer))
}

/// A handler impasse is a subgoal for the general loop.
///
/// When the loop derives a program (or can name what is still unknown), its
/// answer replaces the admission; otherwise the handler's answer stands,
/// carrying the derivation on the log.
///
/// Mirrors `metaResolveImpasse` in `js/worker/formal_ai_worker_meta_reasoner.js`.
pub fn resolve_impasse(prompt: &str, answer: &mut SymbolicAnswer, log: &mut EventLog) {
    if !meta_is_impasse_intent(&answer.intent) {
        return;
    }
    let language = crate::language::detect(prompt).slug();
    let meta = meta_reason_turn(prompt, language);
    let resolved = meta_answer(&meta, true);
    log.append(
        "meta_impasse",
        fill(IMPASSE_HANDOFF, &[&answer.intent, &meta.goal, &meta.status]),
    );
    log.append("meta_derivation", meta.derivation_lino);
    let Some(resolved) = resolved else {
        return;
    };
    let previous = answer.intent.clone();
    let trace_links: Vec<String> = answer
        .evidence_links
        .iter()
        .filter(|entry| {
            entry
                .split_once(':')
                .is_some_and(|(kind, _)| kind == "trace")
        })
        .cloned()
        .collect();
    let mut projected = project(prompt, log, resolved);
    let mut evidence = core::mem::take(&mut projected.evidence_links);
    evidence.push(["meta:impasse:", &previous].concat());
    evidence.extend(trace_links);
    projected.evidence_links = evidence;
    *answer = projected;
}
