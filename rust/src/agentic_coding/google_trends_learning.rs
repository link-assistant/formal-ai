//! Agentic recipe for routing a captured feed through human-gated learning.

use crate::google_trends_catalog::feed_for_task;
use crate::google_trends_learning::trending_learning_report_from_feed;
use crate::seed::{agentic_message, parser::parse_lino};

/// The workspace path the planner writes.
pub const GOOGLE_TRENDS_LEARNING_PATH: &str = "google-trends-learning.lino";

/// A differently worded task for the Google Trends learning-frontier recipe.
pub const GOOGLE_TRENDS_LEARNING_TASK: &str = "Collect the Google Trends learning frontier — the trending searches Formal AI cannot \
     yet resolve — route them through the human-gated self-improvement loop, and record the \
     learning report in Links Notation.";

/// The learning-capability slug this recipe ingests. A learn-from-source
/// directive naming a seed source with this capability drives this exact recipe.
const LEARNING_CAPABILITY: &str = "google_trends_learning";

const GOOGLE_TRENDS_KEYWORDS: [&str; 2] = ["google trends", "trending search"];

/// Whether `prompt` asks for the Google Trends learning-frontier recipe.
///
/// Two disjoint phrasings route here, and both stay clear of the sibling catalog
/// recipe (which keys on prompt/answer/catalog/test):
///
/// 1. **Operator framing** — the explicit learning-loop request: the *frontier*,
///    the *self-improvement loop*, or the "cannot … resolve" pairing.
/// 2. **User teaching directive** (issue #499) — a natural-language "learn from
///    this source" directive that names the Google Trends source, in *any*
///    supported language. This is detected from the same seed-declared registry
///    the chat handler uses ([`crate::seed::learning_sources`]), so the very
///    directive a user types — e.g. "Обратясь сюда ты узнаешь актуальные темы
///    &lt;Google Trends URL&gt;" — drives this artifact-writing recipe through the
///    Agent CLI, not just a chat acknowledgement. Routing keys on the source's
///    declared `capability` slug, never a literal URL or phrase, so a new
///    learnable source is a seed edit rather than a code change (CONTRIBUTING
///    rule 7).
#[must_use]
pub fn is_google_trends_learning_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    if is_learn_from_source_directive(&lower) {
        return true;
    }
    let cannot_resolve = lower.contains("resolve")
        && (lower.contains("cannot") || lower.contains("can't") || lower.contains("not yet"));
    GOOGLE_TRENDS_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
        && (lower.contains("learning frontier")
            || lower.contains("self-improvement loop")
            || cannot_resolve)
}

/// Whether `lowercased` is a user directive teaching the engine to learn from the
/// Google Trends source, resolved entirely from the seed `learning_sources`
/// registry so it stays language-agnostic and free of hardcoded phrases.
fn is_learn_from_source_directive(lowercased: &str) -> bool {
    crate::seed::learning_sources()
        .match_directive(lowercased)
        .is_some_and(|source| source.capability == LEARNING_CAPABILITY)
}

/// Render a report for the requested publisher feed.
pub(super) fn document_for_task(task: &str) -> Option<String> {
    let feed = feed_for_task(task)?;
    let report = trending_learning_report_from_feed(&feed);
    Some(format!("{}\n", report.links_notation()))
}

/// Render the default regional report, or an empty document if the feed is unavailable.
#[must_use]
pub fn render_document() -> String {
    document_for_task("").unwrap_or_default()
}

/// The self-contained final answer for the agentic loop.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let parsed = parse_lino(document);
    let Some(report) = parsed.children.first() else {
        return agentic_message("google_trends_source_unavailable", &[]);
    };
    agentic_message(
        "google_trends_learning_final_answer",
        &[
            ("total", report.find_child_value("total_prompts")),
            ("handled", report.find_child_value("handled_by_engine")),
            ("frontier", report.find_child_value("learning_frontier")),
            (
                "proposals",
                report.find_child_value("learning_run_proposals"),
            ),
            ("adopted", report.find_child_value("learning_run_adopted")),
            ("source_url", report.find_child_value("source_url")),
            ("path", GOOGLE_TRENDS_LEARNING_PATH),
            ("document", document.trim_end()),
        ],
    )
}

/// Shell command used by the agentic recipe to verify the written report exists.
#[must_use]
pub fn verification_command() -> String {
    let program = format!(
        "p='{GOOGLE_TRENDS_LEARNING_PATH}';s=open(p).read().splitlines();print(len(s));print('\\n'.join(s[:14]))"
    );
    format!(
        "python3 -c {}",
        super::general_planner::shell_quote(&program)
    )
}
