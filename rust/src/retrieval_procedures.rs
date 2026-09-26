//! Retrieval-family procedures retired out of the handler dispatcher's module
//! (issue #1138 B9, plan 09 leaf 18).
//!
//! Each function here is one procedure beside the M2 retrieval interpreter
//! (`src/retrieval_method.rs`): the network snapshot and the self-filter read
//! the link store, the source refresh and the learn directive maintain source
//! state, and the conflict reply records a provenance disagreement. They are
//! staging here as one visible group for the batch's seed passes: their cue
//! vocabularies and answer wording are still Rust literals until the pass that
//! moves each into seed rules and localized responses, and the migration
//! ledger keeps all five rows pending until then.

use crate::engine::{SymbolicAnswer, knowledge_links_notation, normalize_prompt, stable_id};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::ROLE_PERSONAL_FACTS_LISTING_REQUEST;
use crate::seed::localized_response;
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::extract_concept_from_query;

pub fn try_network_query(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if normalized.contains("show me the current network")
        || normalized.contains("show me the network")
        || normalized.contains("export the network")
        || normalized.contains("export network")
    {
        let snapshot = knowledge_links_notation();
        let body = format!(
            "Here is the current link network as a links-notation snapshot:\n\n```links\n{snapshot}\n```"
        );
        return Some(finalize_simple(
            prompt,
            log,
            "network_snapshot",
            "response:network_snapshot",
            &body,
            1.0,
        ));
    }
    if let Some(concept) = extract_concept_from_query(prompt) {
        let body = format!(
            "Here is what I know about '{concept}':\n\nintent: {concept}\nrole: \
             the network records '{concept}' as a concept with rules and example links."
        );
        return Some(finalize_simple(
            prompt,
            log,
            &format!("concept_introspection_{concept}"),
            "response:concept_introspection",
            &body,
            1.0,
        ));
    }
    if normalized.starts_with("list facts")
        || crate::seed::lexicon().mentions_role(
            ROLE_PERSONAL_FACTS_LISTING_REQUEST,
            &normalize_prompt(prompt),
        )
    {
        log.append("filter:user", "self".to_owned());
        let body = String::from(
            "No facts have been recorded under your user filter yet. Submit a 'teach this fact' \
             request to start your personal contribution list.",
        );
        return Some(finalize_simple(
            prompt,
            log,
            "filter_user",
            "response:filter_user",
            &body,
            1.0,
        ));
    }
    None
}

pub fn try_source_refresh(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !normalized.contains("refresh")
        || !(normalized.contains("cache") || normalized.contains("page"))
    {
        return None;
    }
    let target = stable_id("source", prompt);
    log.append("source_refresh", target.clone());
    let body = format!(
        "Cached source {target} has been queued for refresh against its origin URL. The \
         refresh event is appended to the audit log and a fresh fetched_at timestamp will be \
         recorded once the new copy is verified."
    );
    Some(finalize_simple(
        prompt,
        log,
        "source_refresh",
        "response:source_refresh",
        &body,
        1.0,
    ))
}

/// Issue #499: recognize a "learn from this data source" directive and route it
/// into the matching auto-learning capability instead of falling to `unknown`.
///
/// The user is teaching the engine where to find data it can reason and learn
/// from — e.g. "Обратясь сюда ты узнаешь актуальные темы &lt;Google Trends URL&gt;".
/// The request is detected from the seed-declared learnable-source registry
/// ([`crate::seed::learning_sources`]): a match needs both a language-agnostic
/// learning-directive cue and a reference to a known source (its host or one of
/// its native-language keywords). Production code branches only on the source's
/// declared `capability` slug, never on a specific URL or phrase, so a new
/// learnable source is a seed edit rather than a code change (CONTRIBUTING rule 7).
pub fn try_learn_from_source(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let registry = crate::seed::learning_sources();
    let source = registry.match_directive(normalized)?;
    let summary = learning_source_summary(&source.capability)?;
    log.append("learning_source", source.id.clone());
    log.append("learning_capability", source.capability.clone());

    let language = detect_language(prompt);
    let intro = localized_response("learn_from_source", language.slug()).unwrap_or_default();
    let body = if intro.is_empty() {
        summary
    } else {
        format!("{intro}\n\n{summary}")
    };
    Some(finalize_simple(
        prompt,
        log,
        "learn_from_source",
        "response:learn_from_source",
        &body,
        1.0,
    ))
}

/// Render the deterministic learning summary for a source `capability`.
///
/// Returns `None` for a capability with no wired learning loop so the handler
/// declines rather than inventing an answer; a new capability is registered here
/// alongside the loop that ingests it.
fn learning_source_summary(capability: &str) -> Option<String> {
    match capability {
        "google_trends_learning" => {
            let report = crate::google_trends_learning::trending_learning_report();
            Some(format!(
                "I recognized Google Trends as a data source I can learn from. I collected \
                 {total} trending prompts across every supported language, already answer \
                 {handled} of them from local links rules, and routed the remaining {frontier} \
                 to the human-gated self-improvement loop. That loop proposed {proposals} rules \
                 and adopted {adopted}: nothing changes without human review.",
                total = report.total_prompts,
                handled = report.handled_by_engine,
                frontier = report.frontier_count(),
                proposals = report.run.proposals.len(),
                adopted = report.adopted_count(),
            ))
        }
        _ => None,
    }
}

pub fn try_source_conflict(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !(normalized.contains("conflict")
        || (normalized.contains("born in") && normalized.contains(" or ")))
    {
        return None;
    }
    log.append(
        "conflict:source_disagreement",
        "sources disagree on the answer".to_owned(),
    );
    let body = String::from(
        "Sources disagree on this question. The disagreement is recorded as a \
         conflict:source_disagreement link in the network rather than silently resolved.",
    );
    Some(finalize_simple(
        prompt,
        log,
        "source_conflict",
        "response:source_conflict",
        &body,
        0.3,
    ))
}
