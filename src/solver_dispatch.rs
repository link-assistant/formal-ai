//! Ordered executable method catalogue for the universal solver.
//!
//! Extracted from `solver.rs` to keep that module under the repository line
//! limit. The method catalogue is the executable backing for the meta method
//! registry: the registry chooses method names, then this module supplies the
//! Rust function for names implemented as regular solver handlers.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::proof_engine::ProofRenderConfig;
use crate::solver::ConversationTurn;
use crate::solver_handler_docs::try_docs_method_explanation;
use crate::solver_handler_how::{
    try_how_it_works, try_how_to_procedure, try_procedural_how_to_followup,
};
use crate::solver_handler_units::try_incompatible_units;
use crate::solver_handlers::{
    try_algorithm, try_arithmetic, try_brainstorming_request, try_calendar_create_event,
    try_calendar_reasoning, try_capabilities, try_clarification, try_compound_interest,
    try_concept_lookup, try_conversation_memory, try_conversation_topic_request,
    try_coreference_request, try_definition_merge, try_document_request, try_execution_failure,
    try_fact_lookup, try_http_fetch, try_ill_formed, try_installation_conversion,
    try_javascript_execution, try_meta_explanation, try_meta_explanation_with_runtime,
    try_network_query, try_number_riddle, try_numeric_list, try_numeric_list_with_history,
    try_opinion_question, try_program_synthesis, try_proof_request, try_proof_request_with_config,
    try_punctuation_only_prompt, try_research_comparison_table, try_roleplay_request,
    try_shell_command_transform, try_shell_command_transform_with_history, try_shell_refusal,
    try_software_project_followup, try_software_project_request, try_source_conflict,
    try_source_refresh, try_summarization_request, try_text_manipulation,
    try_text_manipulation_with_history, try_translation, try_url_navigate, try_web_search,
    try_who_is_question, try_write_script, SelfAwarenessRuntime,
};
use crate::solver_handlers_policy::{try_kupi_slona, try_physical_action_question};

/// Uniform signature every specialized handler conforms to. Handlers that
/// don't need `normalized` go through tiny adapter wrappers below so the
/// dispatch registry stays homogeneous and the registry executor can call every
/// regular table entry through one function shape.
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

/// Outcome of routing a handler name through [`try_contextual_override`].
pub enum ContextualOutcome {
    /// `name` is not a contextual handler; fall through to the registry lookup.
    NotHandled,
    /// The contextual handler produced an answer; the loop should return it.
    Answer(SymbolicAnswer),
    /// `name` is contextual but produced nothing; the loop should `continue`
    /// (these handlers never fall back to a plain registry variant).
    Skip,
}

/// A handful of specialized handlers need more than the uniform
/// `(prompt, normalized, log)` signature: they take a runtime/render config or
/// the conversation history. Rather than widen [`SpecializedHandler`] for every
/// handler, the dispatch loop routes those few names through this helper.
///
/// Extracted from `solver.rs` so that module stays under the repository line
/// limit; these branches are now reached through the registry-backed executor.
/// The context-dependent override handlers, in the order `try_contextual_override`
/// evaluates them.
///
/// This is the single source of truth for the contextual surface, kept beside the
/// match below so the two cannot drift: every name here is dispatched in the
/// `match` and every `match` arm is named here (the
/// `tests/unit/specification/method_registry.rs` grounding test pins both
/// directions against this source). The method registry (issue #559, R331) reads
/// this constant so the catalogue-as-data is grounded in the live code.
pub const CONTEXTUAL_HANDLER_NAMES: &[&str] = &[
    "proof_request",
    "meta_explanation",
    "numeric_list",
    "shell_command_transform",
    "text_manipulation",
];

/// Method names that run before the regular handler table.
///
/// These used to be hardwired at the top of `UniversalSolver`'s specialized
/// dispatch loop. Issue #559 makes them first-class registry methods as well, so
/// the solver has one ordered method-selection path instead of a prelude branch
/// plus a separate handler table.
pub const PRELUDE_METHOD_NAMES: &[&str] = &[
    "diagnostic",
    "nl_tool",
    "behavior_rules",
    "feature_capability",
    "playwright_script",
];

pub fn try_contextual_override(
    name: &str,
    prompt: &str,
    normalized: &str,
    history: &[ConversationTurn],
    proof_render_config: ProofRenderConfig,
    self_awareness_runtime: SelfAwarenessRuntime,
    log: &mut EventLog,
) -> ContextualOutcome {
    let answer = match name {
        "proof_request" => {
            try_proof_request_with_config(prompt, normalized, log, proof_render_config)
        }
        "meta_explanation" => {
            try_meta_explanation_with_runtime(prompt, normalized, log, self_awareness_runtime)
        }
        "numeric_list" => try_numeric_list_with_history(prompt, normalized, log, history),
        "shell_command_transform" => {
            try_shell_command_transform_with_history(prompt, normalized, log, history)
        }
        "text_manipulation" => try_text_manipulation_with_history(prompt, normalized, log, history),
        _ => return ContextualOutcome::NotHandled,
    };
    answer.map_or(ContextualOutcome::Skip, ContextualOutcome::Answer)
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
    // Issue #444: a bare follow-up that asks for the concrete steps ("Can you
    // give me specific instructions?") carries no "how to" lead-in of its own.
    // It must rebind to the procedure recovered from the prior turn rather than
    // fall to the unknown opener, so it sits right after the procedural handler
    // and above the general lookups. It only fires when the previous user turn
    // was itself a how-to request, so unrelated prompts are untouched.
    ("procedural_how_to_followup", try_procedural_how_to_followup),
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
    ("calendar_create_event", try_calendar_create_event),
    ("compound_interest", try_compound_interest),
    // Issue #395: a concrete "<operation> these numbers in <language>, give me
    // the code and the result" request must produce generated code plus the
    // deterministically-computed result. The universal numeric-list engine
    // covers sort/reverse_sort/reverse and the sum/product/minimum/maximum
    // reductions. It runs before `arithmetic` (which would otherwise claim the
    // numeric prompt) and before the generic, result-less `algorithm` handler.
    ("numeric_list", try_numeric_list),
    // Issue #552: shell-command rewrites such as "make this an infinite loop"
    // should produce the concrete command text in chat mode, while still not
    // executing the command. This is more specific than generic script writing
    // or terminal-command refusal.
    ("shell_command_transform", try_shell_command_transform),
    ("number_constraint_reasoning", try_number_riddle),
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
    // Issue #423: README install/deploy guide <-> shell/PowerShell conversion
    // is more specific than a generic "write script" request. It extracts an
    // ordered install-command IR, then renders the requested target surfaces.
    ("installation_conversion", try_installation_conversion),
    ("write_script", try_write_script),
    ("program_synthesis", try_program_synthesis),
    // Issue #425: "make me a PDF / document / report with <subject>" is a
    // document-generation task, not a software build. It runs before
    // `software_project` so a document request is not mistaken for code, and it
    // converts the would-be unknown response into the universal-algorithm plan.
    ("document_generation_plan", try_document_request),
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

/// Return the executable handler for a registry method name implemented by the
/// regular solver-handler table.
#[must_use]
pub fn handler_for_method(name: &str) -> Option<SpecializedHandler> {
    SPECIALIZED_HANDLERS
        .iter()
        .find_map(|(candidate, handler)| (*candidate == name).then_some(*handler))
}
