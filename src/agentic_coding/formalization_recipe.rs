//! The issue-#468 text → Links Notation formalization recipe.
//!
//! State machine: `web_search` → `web_fetch` → `write_file(formalize)` →
//! `run_command(verify)` → final. The canonical task formalizes «Сказка о рыбаке
//! и рыбке»; a task that quotes its own source text (issue #956) skips the
//! search/fetch steps and formalizes the quoted text instead.

use serde_json::json;

use super::formalize::{
    coverage_line, formalize_text_to_links, FormalizedKnowledgeBase, CANONICAL_FISHERMAN_SYNOPSIS,
    FISHERMAN_DOC_ID,
};
use super::planner::{
    fetch_arguments, plan_one, tool_for, trace_route, write_arguments, AgenticPlan, Capability,
    Progress,
};
use crate::protocol::ChatMessage;

/// The Russian web-search query the planner issues when a search tool exists.
pub const SEARCH_QUERY: &str = "Пушкин Сказка о рыбаке и рыбке полный текст";

/// The source URL the planner fetches when a fetch tool exists.
pub const CANONICAL_SOURCE_URL: &str =
    "https://ru.wikisource.org/wiki/Сказка_о_рыбаке_и_рыбке_(Пушкин)";

/// The path the planner writes the knowledge base to.
pub const KB_PATH: &str = "knowledge-base.lino";

/// Keywords that mark a user turn as an issue-#468 formalization task.
const FORMALIZATION_KEYWORDS: [&str; 4] =
    ["formaliz", "формализ", "knowledge base", "links notation"];

/// Keywords that name the canonical tale itself, quoted or not.
const CANONICAL_TALE_KEYWORDS: [&str; 3] = ["рыбак", "fisherman", "сказк"];

/// Whether `prompt` asks to formalize a text into a knowledge base.
pub(super) fn is_formalization_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    FORMALIZATION_KEYWORDS
        .iter()
        .chain(CANONICAL_TALE_KEYWORDS.iter())
        .any(|keyword| lower.contains(keyword))
}

/// Quote pairs a formalization task may wrap its inline source text in.
const INLINE_SOURCE_DELIMITERS: [(char, char); 4] =
    [('«', '»'), ('“', '”'), ('「', '」'), ('《', '》')];

/// The source text a formalization task carries inline, when it quotes one.
///
/// `formal-ai agent --task "Formalize «The cat sat on the mat» …"` names its
/// own document, so the recipe must formalize that text instead of silently
/// substituting the canonical tale (issue #956). A quoted *title* of the
/// canonical tale — the canonical task's own phrasing — still selects the full
/// tale rather than its five-word name.
fn inline_formalization_source(task: &str) -> Option<String> {
    for (open, close) in INLINE_SOURCE_DELIMITERS {
        let Some(start) = task.find(open) else {
            continue;
        };
        let rest = &task[start + open.len_utf8()..];
        let Some(end) = rest.find(close) else {
            continue;
        };
        let source = rest[..end].trim();
        if source.is_empty() || is_canonical_tale_reference(source) {
            continue;
        }
        return Some(source.to_owned());
    }
    None
}

/// Whether the quoted text merely names the canonical tale.
fn is_canonical_tale_reference(text: &str) -> bool {
    let lower = text.to_lowercase();
    CANONICAL_TALE_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
}

/// Plan the next formalization step from the conversation and advertised tools.
// State machine: web_search → web_fetch → write_file(formalize) → run_command(verify) → final.
pub(super) fn plan_formalization_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);

    // A task that quotes its own source text carries the document inline, so
    // searching or fetching the canonical tale would formalize the wrong text.
    // Before this route existed the custom `--task` was silently discarded in
    // favour of the seeded fairy tale (issue #956).
    let inline_source = inline_formalization_source(task);
    trace_route(
        "formalization_source",
        inline_source.as_deref().unwrap_or(FISHERMAN_DOC_ID),
    );

    if inline_source.is_none() {
        // Step 1: search for the source text.
        if let Some(tool) = tool_for(tool_names, Capability::Search) {
            if !progress.done(Capability::Search) {
                return plan_one(tool, json!({ "query": SEARCH_QUERY }).to_string());
            }
        }
        // Step 2: fetch the source text.
        if let Some(tool) = tool_for(tool_names, Capability::Fetch) {
            if !progress.done(Capability::Fetch) {
                return plan_one(tool, fetch_arguments(CANONICAL_SOURCE_URL));
            }
        }
    }

    // The source text for the knowledge base: the text quoted in the task if
    // there is one, else the latest non-errored fetch result, else the
    // canonical synopsis (the determinism fallback).
    let source = inline_source
        .as_deref()
        .or(progress.fetched_text.as_deref())
        .unwrap_or(CANONICAL_FISHERMAN_SYNOPSIS);
    let formalized = formalize_text_to_links(source, "");

    // Step 3: write the formalized knowledge base.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(KB_PATH, &formalized.links_notation));
        }
    }
    // Step 4: verify by reading the file back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {KB_PATH}") });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 5: nothing left to do — answer with the knowledge base inline.
    AgenticPlan::Final(final_answer(&formalized))
}

/// The self-contained final answer: a natural-language summary, the coverage
/// line, and the Links Notation knowledge base inline.
fn final_answer(formalized: &FormalizedKnowledgeBase) -> String {
    let summary = &formalized.summary;
    let subject = if summary.doc_id == FISHERMAN_DOC_ID {
        "«Сказка о рыбаке и рыбке»".to_owned()
    } else {
        format!("the source text ({})", summary.doc_id)
    };
    format!(
        "Formalized {subject} into a Links Notation knowledge base: {records} records realising \
         all nine protocol primitives ({coverage}).\n\nKnowledge base ({KB_PATH}):\n\n{kb}",
        records = summary.total_records(),
        coverage = coverage_line(summary),
        kb = crate::issue_report::fenced_block(
            crate::issue_report::LINO_FENCE_LANGUAGE,
            &formalized.links_notation,
        ),
    )
}
