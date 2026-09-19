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
use crate::concept_lookup::RegistrySourceLookup;
use crate::formalization::concept_links::{ConceptGraph, formalize_deeply};
use crate::how_to_guide::ServicePreferences;
use crate::protocol::ChatMessage;
use crate::service_accessibility::ServiceAccessibilityCache;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};
use crate::source_walk::LookupBounds;

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

/// The process source cache the offline agent replay consults, and the env var
/// that moves it — the same convention `try_how_to_procedure_with_offline`
/// established for offline capture replay.
const SOURCE_CACHE_ENV: &str = "FORMAL_AI_SOURCE_CACHE_DIR";

/// The deep pass the recipe adds on top of the nine-primitive formalizer
/// (issue #1138, plan 04 L11/L16): every need the document raised is put to
/// plan 01's registry lookup over the process source cache, offline. The agent
/// loop has no transport of its own — an offline replay of committed captures
/// grounds what it can, and what no capture answers is reported
/// `unsatisfiable` with its exact span rather than dropped.
fn deep_grounding(source: &str, doc_id: &str, language: &str) -> Option<ConceptGraph> {
    let cache_dir = std::env::var(SOURCE_CACHE_ENV).unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(&cache_dir, CurlSourceTransport).with_online(false);
    let mut availability =
        ServiceAccessibilityCache::new(std::env::temp_dir().join("formal-ai-agent-formalization"));
    let preferences = ServicePreferences::default();
    let bounds = LookupBounds::default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let mut lookup = RegistrySourceLookup::new(
        &client,
        &preferences,
        &mut availability,
        bounds.clone(),
        language,
        now,
    );
    let graph = formalize_deeply(source, doc_id, &mut lookup, &bounds, 1);
    (!graph.needs.is_empty()).then_some(graph)
}

/// The nine-primitive knowledge base, plus — when the document raised needs —
/// the concept graph the registry grounded for it.
///
/// The canonical tale raises no needs, so its knowledge base is byte-identical
/// with the shallow pass and the regression corpus stays intact. An
/// unfamiliar document gets its needs satisfied from the captured sources the
/// registry declares; a need nobody answers is reported, never silently kept
/// as a covered primitive.
fn formalize_with_grounding(source: &str) -> (FormalizedKnowledgeBase, Option<ConceptGraph>) {
    let base = formalize_text_to_links(source, "");
    if base.summary.needs_raised == 0 {
        return (base, None);
    }
    let language = crate::language::detect(source).slug();
    let grounding = deep_grounding(source, &base.summary.doc_id, &language);
    (base, grounding)
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
        //
        // Step 1: search for what the task itself does not understand — the
        // query is the source's first unresolved surface, never a pinned one.
        if let Some(tool) = tool_for(tool_names, Capability::Search)
            && !progress.done(Capability::Search)
        {
            let query = requested_source
                .as_deref()
                .and_then(discovery_query)
                .unwrap_or_else(|| SEARCH_QUERY.to_owned());
            return plan_one(tool, json!({ "query": query }).to_string());
        }
        // Step 2: fetch the source the task names. Only the recognised
        // catalogue work — no requested source of its own — still fetches the
        // canonical tale URL, as its regression fixture.
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
    let (formalized, grounding) = formalize_with_grounding(source);
    // The written knowledge base carries the deep grounding too: the graph's
    // concept records with their provenance, and every need row — satisfied or
    // reported unsatisfiable (issue #1138, plan 04 L11/L16).
    let mut knowledge_base = formalized.links_notation.clone();
    if let Some(graph) = &grounding {
        knowledge_base.push('\n');
        knowledge_base.push_str(&graph.to_links_notation());
        knowledge_base.push('\n');
    }

    // Step 3: write the formalized knowledge base.
    if let Some(tool) = write_tool
        && !progress.done(Capability::Write)
    {
        return plan_one(tool, write_arguments(KB_PATH, &knowledge_base));
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
        &knowledge_base,
        grounding.as_ref(),
        crate::language::detect(task).slug(),
    ))
}

/// The self-contained final answer: a natural-language summary, the coverage
/// line, the need report when the document raised needs, and the Links
/// Notation knowledge base inline.
#[allow(
    clippy::literal_string_with_formatting_args,
    reason = "bind slots in a localized seed response"
)]
fn final_answer(
    formalized: &FormalizedKnowledgeBase,
    knowledge_base: &str,
    grounding: Option<&ConceptGraph>,
    language: &str,
) -> String {
    let summary = &formalized.summary;
    let subject = if summary.doc_id == FISHERMAN_DOC_ID {
        "«Сказка о рыбаке и рыбке»".to_owned()
    } else {
        format!("the source text ({})", summary.doc_id)
    };
    // The coverage-honesty rule of issue #1138 plan 04: a document with an
    // unresolved need is never reported as covered, so the answer names how
    // many needs were raised and how many the sources grounded before the
    // knowledge base it shows.
    let need_report = grounding.map_or(String::new(), |graph| {
        let (grounded, total) = graph.grounded_ratio();
        let total = total.to_string();
        let grounded = grounded.to_string();
        crate::seed::render_response(
            "agentic_formalization_need_report",
            language,
            &[("total", total.as_str()), ("grounded", grounded.as_str())],
        )
        .map_or(String::new(), |line| format!("{line}\n\n"))
    });
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
            &format!(
                "{need_report}{}",
                crate::issue_report::fenced_block(
                    crate::issue_report::LINO_FENCE_LANGUAGE,
                    knowledge_base,
                )
            ),
        )
}
