//! Ordered dispatch table for the universal solver's specialized handlers.
//!
//! Extracted from `solver.rs` to keep that module under the repository line
//! limit. The table is the single source of truth for handler precedence: the
//! first handler that returns `Some` wins, and several tests rely on this
//! resolution order.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver_handler_docs::try_docs_method_explanation;
use crate::solver_handler_how::{try_how_it_works, try_how_to_procedure};
use crate::solver_handler_units::try_incompatible_units;
use crate::solver_handlers::{
    try_algorithm, try_arithmetic, try_brainstorming_request, try_calendar_reasoning,
    try_capabilities, try_clarification, try_compound_interest, try_concept_lookup,
    try_conversation_memory, try_conversation_topic_request, try_coreference_request,
    try_definition_merge, try_execution_failure, try_fact_lookup, try_http_fetch, try_ill_formed,
    try_javascript_execution, try_meta_explanation, try_network_query, try_numeric_list,
    try_opinion_question, try_program_synthesis, try_proof_request, try_punctuation_only_prompt,
    try_research_comparison_table, try_roleplay_request, try_shell_refusal,
    try_software_project_followup, try_software_project_request, try_source_conflict,
    try_source_refresh, try_summarization_request, try_text_manipulation, try_translation,
    try_url_navigate, try_web_search, try_who_is_question, try_write_script,
};
use crate::solver_handlers_policy::{try_kupi_slona, try_physical_action_question};

/// Uniform signature every specialized handler conforms to. Handlers that
/// don't need `normalized` go through tiny adapter wrappers below so the
/// dispatch registry stays homogeneous and the loop in
/// `UniversalSolver::handle_specialized_pattern` remains a single line.
pub type SpecializedHandler = fn(&str, &str, &mut EventLog) -> Option<SymbolicAnswer>;

fn handle_arithmetic(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_arithmetic(prompt, log)
}

fn handle_javascript_execution(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_javascript_execution(prompt, log)
}

fn handle_concept_lookup(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_concept_lookup(prompt, log)
}

/// Ordered dispatch table for the universal solver's specialized handlers.
///
/// Order matters: the first handler that returns `Some` wins, and several
/// downstream tests rely on the resolution order (for example, conversation
/// memory must trigger before the concept lookup when both could match).
/// New handlers should be slotted into the position that preserves intent
/// precedence rather than appended unconditionally.
pub const SPECIALIZED_HANDLERS: &[(&str, SpecializedHandler)] = &[
    ("http_fetch", try_http_fetch),
    ("url_navigate", try_url_navigate),
    ("web_search", try_web_search),
    ("research_comparison_table", try_research_comparison_table),
    ("docs_method_explanation", try_docs_method_explanation),
    ("procedural_how_to", try_how_to_procedure),
    ("conversation_memory", try_conversation_memory),
    // Issue #341: a decomposed agent step like "test it by scraping
    // wikipedia.org and show me the top 10 most frequent words" must stay
    // bound to the active software-project dialogue instead of resolving the
    // `wikipedia` concept or hitting the unknown opener. The handler only
    // fires when the previous assistant turn formalized a
    // `software_project_request`, so it sits above the general lookups.
    ("software_project_followup", try_software_project_followup),
    ("summarization", try_summarization_request),
    ("text_manipulation", try_text_manipulation),
    ("brainstorming", try_brainstorming_request),
    ("conversation_topic", try_conversation_topic_request),
    ("fact_lookup", try_fact_lookup),
    ("coreference", try_coreference_request),
    ("roleplay", try_roleplay_request),
    ("translation", try_translation),
    ("capabilities", try_capabilities),
    ("calendar_reasoning", try_calendar_reasoning),
    ("compound_interest", try_compound_interest),
    // Issue #395: a concrete "<operation> these numbers in <language>, give me
    // the code and the result" request must produce generated code plus the
    // deterministically-computed result. The universal numeric-list engine
    // covers sort/reverse_sort/reverse and the sum/product/minimum/maximum
    // reductions. It runs before `arithmetic` (which would otherwise claim the
    // numeric prompt) and before the generic, result-less `algorithm` handler.
    ("numeric_list", try_numeric_list),
    ("arithmetic", handle_arithmetic),
    ("javascript_execution", handle_javascript_execution),
    ("definition_merge", try_definition_merge),
    ("concept_lookup", handle_concept_lookup),
    ("who_is", try_who_is_question),
    ("how_it_works", try_how_it_works),
    ("meta_explanation", try_meta_explanation),
    ("network_query", try_network_query),
    // `execution_failure` must run before `write_script`/`algorithm` so that
    // explicit failure prompts (e.g. "calls undefined_function()") surface a
    // failure trace instead of being silently transformed into a passing
    // hello-world snippet.
    ("execution_failure", try_execution_failure),
    ("write_script", try_write_script),
    ("program_synthesis", try_program_synthesis),
    ("software_project", try_software_project_request),
    ("algorithm", try_algorithm),
    ("source_refresh", try_source_refresh),
    ("source_conflict", try_source_conflict),
    ("clarification", try_clarification),
    ("punctuation_only_prompt", try_punctuation_only_prompt),
    ("ill_formed", try_ill_formed),
    ("physical_action_question", try_physical_action_question),
    ("kupi_slona", try_kupi_slona),
    ("shell_refusal", try_shell_refusal),
    // Proof requests must beat `opinion_question` so prompts like
    // "Do you think you can prove …" land on the formalization pipeline
    // explanation instead of the no-opinion policy.
    ("proof_request", try_proof_request),
    ("opinion_question", try_opinion_question),
    ("incompatible_units", try_incompatible_units),
];
