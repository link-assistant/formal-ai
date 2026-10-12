//! Retrieval-family procedures retired out of the handler dispatcher's module
//! (issue #1138 B9, plan 09 leaf 18).
//!
//! The learn directive here is one procedure beside the M2 retrieval
//! interpreter (`src/retrieval_method.rs`); it maintains source state. Its
//! summary wording is still a Rust literal until the pass that moves it into a
//! localized response, and the migration ledger keeps its row pending until
//! then. The network snapshot and self-filter, the source refresh and the
//! conflict reply took that pass (issue #918): they are the `network_query`,
//! `source_refresh` and `source_conflict` rule sets of
//! `data/seed/handler-rules.lino`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::localized_response;
use crate::solver_handlers::finalize_simple;

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
