//! The issue-#468 text → Links Notation formalization recipe.
//!
//! State machine: discover missing terms when necessary → bind the requested
//! source → `write_file(formalize)` → `run_command(verify)` → final. The
//! canonical task may still use its historical source as a last-resort fixture;
//! every other task formalizes its own bytes rather than substituting that tale.

use serde_json::json;

use super::formalize::{
    CANONICAL_FISHERMAN_SYNOPSIS, FISHERMAN_DOC_ID, FormalizedKnowledgeBase, PRIMITIVE_KINDS,
    coverage_line, formalize_text_to_links,
};
use super::lexicon::Lexicon;
use super::planner::{
    AgenticPlan, Capability, Progress, fetch_arguments, plan_one, tool_for, trace_route,
    write_arguments,
};
use crate::protocol::ChatMessage;

/// The Russian web-search query the planner issues when a search tool exists.
pub const SEARCH_QUERY: &str = "Пушкин Сказка о рыбаке и рыбке полный текст";

/// The source URL the planner fetches when a fetch tool exists.
pub const CANONICAL_SOURCE_URL: &str =
    "https://ru.wikisource.org/wiki/Сказка_о_рыбаке_и_рыбке_(Пушкин)";

/// The path the planner writes the knowledge base to.
pub const KB_PATH: &str = "knowledge-base.lino";

/// Whether `prompt` asks to formalize a text into a knowledge base.
pub(super) fn is_formalization_task(prompt: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(prompt);
    crate::seed::lexicon().mentions_role(crate::seed::ROLE_AGENT_ACTION_FORMALIZE_VERB, &normalized)
}

/// The source text a formalization task carries inline, when it quotes one.
///
/// `formal-ai agent --task "Formalize «The cat sat on the mat» …"` names its
/// own document, so the recipe must formalize that text instead of silently
/// substituting the canonical tale (issue #956). A quoted *title* of the
/// canonical tale — the canonical task's own phrasing — still selects the full
/// tale rather than its five-word name.
fn inline_formalization_source(task: &str) -> Option<String> {
    let segment = crate::normal_markov::quoted_segment_spans(task)
        .into_iter()
        .find(|segment| !segment.text.trim().is_empty())?;
    let source = segment.text.trim();
    // Identity is not topic overlap. Only this legacy recipe's supported work
    // may select its cached resource; other catalogue entries stay literal.
    // Once the first source is bound, a later quoted label cannot replace it.
    let is_supported_reference = Lexicon::standard()
        .work_for_title(source)
        .is_some_and(|work| work.doc_id == FISHERMAN_DOC_ID);
    (!is_supported_reference).then(|| source.to_owned())
}

/// Bind an unquoted request to the document it actually asks us to formalize.
///
/// A recognised catalogue work remains a reference that may be fetched. Every
/// other request is itself the only authoritative source available: when it
/// uses the conventional `instruction: document` shape, the right-hand side is
/// the document; otherwise the full request is preserved.
fn requested_formalization_source(task: &str) -> Option<String> {
    if Lexicon::standard()
        .best_work_for(task)
        .is_some_and(|work| work.doc_id == FISHERMAN_DOC_ID)
    {
        return None;
    }
    let source = task
        .split_once(':')
        .map_or(task, |(_, document)| document)
        .trim();
    (!source.is_empty()).then(|| source.to_owned())
}

/// The first unresolved surface is the next discovery query. Which sources
/// answer it remains the source registry's decision; the recipe contributes no
/// domain noun or pinned URL.
fn discovery_query(source: &str) -> Option<String> {
    let language = crate::language::detect(source).slug();
    crate::concept_lookup::unknown_surfaces(source, language)
        .into_iter()
        .next()
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
    let requested_source = inline_source
        .clone()
        .or_else(|| requested_formalization_source(task));
    trace_route(
        "formalization_source",
        requested_source.as_deref().unwrap_or(FISHERMAN_DOC_ID),
    );

    if inline_source.is_none() {
        // An unfamiliar document searches for what it does not understand;
        // the recognised catalogue work alone retains its source-text fetch as
        // a compatibility fallback.
        if let Some(tool) = tool_for(tool_names, Capability::Search)
            && !progress.done(Capability::Search)
        {
            let query = requested_source
                .as_deref()
                .and_then(discovery_query)
                .unwrap_or_else(|| SEARCH_QUERY.to_owned());
            return plan_one(tool, json!({ "query": query }).to_string());
        }
        if requested_source.is_none()
            && let Some(tool) = tool_for(tool_names, Capability::Fetch)
            && !progress.done(Capability::Fetch)
        {
            return plan_one(tool, fetch_arguments(CANONICAL_SOURCE_URL));
        }
    }

    // The source text for the knowledge base: the text quoted in the task if
    // there is one, else the latest non-errored fetch result, else the
    // canonical synopsis (the determinism fallback).
    let source = requested_source
        .as_deref()
        .or(progress.fetched_text.as_deref())
        .unwrap_or(CANONICAL_FISHERMAN_SYNOPSIS);
    let formalized = formalize_text_to_links(source, "");

    // Step 3: write the formalized knowledge base.
    if let Some(tool) = write_tool
        && !progress.done(Capability::Write)
    {
        return plan_one(tool, write_arguments(KB_PATH, &formalized.links_notation));
    }
    // Step 4: verify by reading the file back.
    if let Some(tool) = run_tool
        && !progress.done(Capability::Run)
    {
        let arguments = json!({ "command": format!("cat {KB_PATH}") });
        return plan_one(tool, arguments.to_string());
    }

    // Step 5: nothing left to do — answer with the knowledge base inline.
    AgenticPlan::Final(final_answer(
        &formalized,
        crate::language::detect(task).slug(),
    ))
}

/// The self-contained final answer: a natural-language summary, the coverage
/// line, and the Links Notation knowledge base inline.
#[allow(
    clippy::literal_string_with_formatting_args,
    reason = "bind slots in a localized seed response"
)]
fn final_answer(formalized: &FormalizedKnowledgeBase, language: &str) -> String {
    let summary = &formalized.summary;
    let subject = if summary.doc_id == FISHERMAN_DOC_ID {
        "«Сказка о рыбаке и рыбке»".to_owned()
    } else {
        format!("the source text ({})", summary.doc_id)
    };
    crate::seed::localized_response("agentic_formalization_report", language)
        .unwrap_or_default()
        .replace("{subject}", &subject)
        .replace("{records}", &summary.total_records().to_string())
        .replace("{covered_count}", &summary.covered.len().to_string())
        .replace("{primitive_count}", &PRIMITIVE_KINDS.len().to_string())
        .replace("{coverage}", &coverage_line(summary))
        .replace("{path}", KB_PATH)
        .replace(
            "{kb}",
            &crate::issue_report::fenced_block(
                crate::issue_report::LINO_FENCE_LANGUAGE,
                &formalized.links_notation,
            ),
        )
}
