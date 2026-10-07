//! Ordered executable method catalogue for the universal solver.
//!
//! Extracted from `solver.rs` to keep that module under the repository line
//! limit. The method catalogue is the executable backing for the meta method
//! registry: the registry chooses method names, then this module supplies the
//! Rust function for names implemented as regular solver handlers.

use crate::definition_merge::merge_definitions;
use crate::engine::SymbolicAnswer;
use crate::entity_resolution::resolve_who_is;
use crate::event_log::EventLog;
use crate::legality_warning::handle_legality_warning;
use crate::number_constraints::solve_number_constraints;
use crate::proof_engine::ProofRenderConfig;
use crate::solver::{ConversationTurn, SolverConfig};
use crate::solver_handler_how::{
    try_how_it_works, try_how_to_procedure, try_procedural_how_to_followup,
};
use crate::solver_handler_how_synthesis::try_how_to_procedure_with_offline;
use crate::solver_handler_units::try_incompatible_units;
use crate::solver_handlers::{
    SelfAwarenessRuntime, handle_advice_request, handle_brainstorm_request, handle_code_debugging,
    handle_code_explanation, handle_code_refactoring, handle_code_review,
    handle_creative_writing_request, handle_formalization_request, handle_format_conversion,
    handle_planning_request, handle_product_search, handle_regex_synthesis,
    handle_shell_command_compose, handle_sql_synthesis, handle_statistics,
    handle_summarization_request, handle_test_generation, handle_text_rewrite,
    handle_unit_conversion, handle_word_problem, try_algorithm, try_arithmetic,
    try_brainstorming_request, try_calendar_create_event, try_calendar_reasoning,
    try_compound_interest, try_concept_lookup, try_conversation_memory,
    try_conversation_topic_request, try_coreference_request, try_document_originality_check,
    try_document_request, try_execution_failure, try_fact_checking, try_fact_lookup,
    try_http_fetch, try_http_fetch_with_offline, try_installation_conversion,
    try_javascript_execution, try_learn_from_source, try_meta_explanation,
    try_meta_explanation_with_runtime, try_network_query, try_numeric_list,
    try_numeric_list_with_history, try_program_synthesis, try_program_synthesis_with_online,
    try_proof_request, try_proof_request_with_config, try_research_comparison_table,
    try_research_result_followup, try_response_language_followup, try_roleplay_request,
    try_shell_command_transform, try_shell_command_transform_with_history,
    try_software_project_followup, try_software_project_request, try_source_conflict,
    try_source_refresh, try_summarization_request, try_task_decomposition_with_depth,
    try_text_manipulation, try_text_manipulation_with_history, try_translation, try_url_navigate,
    try_verifiable_task, try_web_search, try_web_search_with_offline, try_world_state,
    try_write_script,
};
use crate::triz_solver::handle_triz;

/// Uniform signature every specialized handler conforms to. Handlers that
/// don't need `normalized` go through tiny adapter wrappers below so the
/// dispatch registry stays homogeneous and the registry executor can call every
/// regular table entry through one function shape.
pub type NativeHandler = fn(&str, &str, &mut EventLog) -> Option<SymbolicAnswer>;

/// An executable method: a native function, or a rule set interpreted from
/// `data/seed/handler-rules.lino` (issue #1085 D1.3).
#[derive(Clone, Copy, Debug)]
pub enum SpecializedHandler {
    Native(NativeHandler),
    Rule(&'static str),
}

impl SpecializedHandler {
    /// Run the handler the way the dispatch loop runs every table entry.
    #[must_use]
    pub fn call(
        self,
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        match self {
            Self::Native(handler) => handler(prompt, normalized, log),
            Self::Rule(name) => crate::rule_interpreter::run_handler(name, prompt, normalized, log),
        }
    }
}

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

const fn response_language_followup_noop(
    _prompt: &str,
    _normalized: &str,
    _log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    None
}

/// The issue #1177 handlers whose subject is a pasted code artifact. In such a
/// request the code is what the instruction acts on (review, explain, refactor,
/// debug), never a task to synthesize, and its paths and operators are code,
/// never a workspace object.
const CODE_ARTIFACT_HANDLERS: &[NativeHandler] = &[
    handle_code_debugging,
    handle_code_explanation,
    handle_code_review,
    handle_code_refactoring,
];

/// True when one of `handlers` claims `prompt`, probed on a scratch log so
/// the caller's trace records only the handler that actually answers.
fn any_handler_claims(handlers: &[NativeHandler], prompt: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(prompt);
    handlers
        .iter()
        .any(|handler| handler(prompt, &normalized, &mut EventLog::new()).is_some())
}

/// True when a code-artifact handler (debugging, explanation, review,
/// refactoring) claims the request. Route promotion asks it before reading a
/// pasted `def` as a program-synthesis task (issue #1177).
#[must_use]
pub fn code_artifact_task_claims(prompt: &str) -> bool {
    any_handler_claims(CODE_ARTIFACT_HANDLERS, prompt)
}

/// True when the shell-command composer claims the request. The capability
/// table's honest gap declines these so the rank walk reaches the composer
/// instead of reading the composed command's search root as a workspace read
/// (issue #1177).
#[must_use]
pub fn shell_compose_claims(prompt: &str) -> bool {
    any_handler_claims(&[handle_shell_command_compose], prompt)
}

#[derive(Clone, Copy)]
pub struct ContextualRuntime {
    proof_render_config: ProofRenderConfig,
    self_awareness_runtime: SelfAwarenessRuntime,
    /// Full solver config, so the response-language follow-up can replay the
    /// previous request through a fresh [`crate::solver::UniversalSolver`] with
    /// the requested language forced (issue #556).
    solver_config: SolverConfig,
}

impl ContextualRuntime {
    #[must_use]
    pub const fn new(
        proof_render_config: ProofRenderConfig,
        self_awareness_runtime: SelfAwarenessRuntime,
        solver_config: SolverConfig,
    ) -> Self {
        Self {
            proof_render_config,
            self_awareness_runtime,
            solver_config,
        }
    }
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
    "http_fetch",
    "proof_request",
    "meta_explanation",
    "numeric_list",
    "program_synthesis",
    "shell_command_transform",
    "text_manipulation",
    "task_decomposition",
    "response_language_followup",
    "fact_checking",
    "world_state",
    "web_search",
    // Issue #991: the procedural handler must execute the bounded multi-source
    // synthesis, which needs the runtime's offline flag to decide whether the
    // capture client may go to the network. It stays a *contextual* override of
    // the same `procedural_how_to` name the regular table registers, so the
    // discovery-plan handler remains the fallback when the evidence is thin.
    "procedural_how_to",
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
    runtime: ContextualRuntime,
    log: &mut EventLog,
) -> ContextualOutcome {
    let answer = match name {
        "http_fetch" => try_http_fetch_with_offline(
            prompt,
            normalized,
            log,
            runtime.solver_config.offline || !crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH"),
        ),
        "web_search" => try_web_search_with_offline(
            prompt,
            normalized,
            log,
            runtime.solver_config.offline || !crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH"),
        ),
        "proof_request" => {
            try_proof_request_with_config(prompt, normalized, log, runtime.proof_render_config)
        }
        "meta_explanation" => try_meta_explanation_with_runtime(
            prompt,
            normalized,
            log,
            runtime.self_awareness_runtime,
        ),
        "numeric_list" => try_numeric_list_with_history(prompt, normalized, log, history),
        "program_synthesis" => try_program_synthesis_with_online(
            prompt,
            normalized,
            log,
            !runtime.solver_config.offline,
        ),
        "shell_command_transform" => {
            try_shell_command_transform_with_history(prompt, normalized, log, history)
        }
        "text_manipulation" => try_text_manipulation_with_history(prompt, normalized, log, history),
        // Issue #847: the recursion has to stop at the configured
        // `max_decomposition_depth` and say so, which means the handler needs
        // the live solver config rather than the shipped default.
        "task_decomposition" => try_task_decomposition_with_depth(
            prompt,
            normalized,
            log,
            runtime.solver_config.max_decomposition_depth,
        ),
        "response_language_followup" => {
            try_response_language_followup(prompt, normalized, log, history, runtime.solver_config)
        }
        "fact_checking" => {
            try_fact_checking(prompt, normalized, log, history, runtime.solver_config)
        }
        "world_state" => try_world_state(prompt, normalized, log, history, runtime.solver_config),
        "procedural_how_to" => try_how_to_procedure_with_offline(
            prompt,
            normalized,
            log,
            runtime.solver_config.offline || !crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH"),
        ),
        _ => return ContextualOutcome::NotHandled,
    };
    answer.map_or(ContextualOutcome::Skip, ContextualOutcome::Answer)
}

/// The specialized-handler function registry: handler name → executable Rust
/// function.
///
/// This is the *code* half of the dispatch table — the function pointers, which
/// cannot live in seed data. The *order* in which handlers are tried (the
/// behaviour) lives in `data/seed/handler-precedence.lino` and is applied by
/// [`specialized_handlers`]; issue #663 retired the precedence remnant that used
/// to be baked into this constant's declaration order into that seed file.
///
/// The declaration order here is kept aligned with the shipped seed purely for
/// reviewability — it is not the dispatch authority. [`specialized_handlers`]
/// asserts the two are an exact permutation, so this registry and the seed can
/// never silently drift.
const HANDLER_FUNCTIONS: &[(&str, NativeHandler)] = &[
    ("legality_warning", handle_legality_warning),
    ("http_fetch", try_http_fetch),
    ("url_navigate", try_url_navigate),
    ("document_originality_check", try_document_originality_check),
    ("formalization_request", handle_formalization_request),
    ("product_search", handle_product_search),
    ("web_search", try_web_search),
    // Issue #499: a "learn from this data source" directive (a user pointing the
    // engine at Google Trends or another declared source it can learn from) must
    // route into the matching auto-learning capability instead of the unknown
    // opener. It sits after the URL/search handlers — which decline a directive
    // that carries no fetch/open/search cue — and above the general lookups. The
    // match is self-gated on the seed-declared learnable-source registry, so an
    // unrelated prompt is untouched.
    ("learn_from_source", try_learn_from_source),
    ("research_comparison_table", try_research_comparison_table),
    ("research_result_followup", try_research_result_followup),
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
    // Issue #1174: a summarization request that carries its own text (paste or
    // quote) is answered from that text. It must run before the seeded-topic
    // summarizer below, which declines any prompt without a payload; a
    // no-payload "summarize the weather" still falls through to it.
    ("summarization_text", handle_summarization_request),
    ("summarization", try_summarization_request),
    ("verifiable_task", try_verifiable_task),
    ("text_manipulation", try_text_manipulation),
    ("brainstorm_composition", handle_brainstorm_request),
    ("brainstorming", try_brainstorming_request),
    ("conversation_topic", try_conversation_topic_request),
    ("advice_request", handle_advice_request),
    ("fact_lookup", try_fact_lookup),
    ("coreference", try_coreference_request),
    ("roleplay", try_roleplay_request),
    ("creative_writing", handle_creative_writing_request),
    ("translation", try_translation),
    // Issue #1174: register rewriting, grammar correction, commit-message and
    // email composition. The cues (rewrite/correct/commit/email) are disjoint
    // from translation's, so it sits after it and before the language
    // follow-up noop.
    ("text_rewrite", handle_text_rewrite),
    (
        "response_language_followup",
        response_language_followup_noop,
    ),
    ("planning_request", handle_planning_request),
    ("calendar_reasoning", try_calendar_reasoning),
    ("calendar_create_event", try_calendar_create_event),
    ("compound_interest", try_compound_interest),
    // Issue #1176: price-times-count word problems ("5 boxes pay 3 each,
    // total?") are prose, not bare number lists, so they claim the prompt
    // before `numeric_list` can read "total" as a sum cue.
    ("word_problem", handle_word_problem),
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
    // Issue #1177: the code-task family, part one. Each handler is structural
    // (nothing is ever executed) and claims only prompts carrying its own cue
    // vocabulary plus a code artifact, so they sit between the shell-command
    // rewrite and the number-constraint reasoner without disturbing either.
    ("code_debugging", handle_code_debugging),
    ("regex_synthesis", handle_regex_synthesis),
    ("sql_synthesis", handle_sql_synthesis),
    ("shell_command_compose", handle_shell_command_compose),
    ("number_constraint_reasoning", solve_number_constraints),
    // Issue #1177: the code-task family, part two — explanation, review, test
    // generation, refactoring, and JSON/YAML format conversion.
    ("code_explanation", handle_code_explanation),
    ("code_review", handle_code_review),
    ("test_generation", handle_test_generation),
    ("code_refactoring", handle_code_refactoring),
    ("format_conversion", handle_format_conversion),
    // Issue #531: a concrete "find the pattern in 1 2 1 2" / "what comes next"
    // request over an explicit sequence or grid runs the link-native
    // pattern-inference substrate. It is data-gated (needs a run of atoms) so a
    // bare "what is a pattern?" still falls through to the concept lookup, and
    // it sits before `arithmetic` so a numeric sequence is analysed structurally
    // rather than mistaken for a calculation.
    ("arithmetic", handle_arithmetic),
    // Issue #1176: statistics over an explicit numeric dataset. It sits after
    // `arithmetic` and `numeric_list` so a bare "range of 1..5" style prompt is
    // computed by them first; the statistics handler requires the dataset
    // vocabulary plus at least two numbers.
    ("statistics", handle_statistics),
    ("javascript_execution", handle_javascript_execution),
    ("definition_merge", merge_definitions),
    ("triz_resolution", handle_triz),
    ("concept_lookup", handle_concept_lookup),
    ("who_is", resolve_who_is),
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
    // Proof requests must beat `opinion_question` so prompts like
    // "Do you think you can prove …" land on the formalization pipeline
    // explanation instead of the no-opinion policy.
    ("proof_request", try_proof_request),
    // Issue #1176: exact unit conversion ("3 km in m", "how many feet in 3
    // miles") with the arithmetic shown. It sits directly before
    // `incompatible_units`, which declines the pairs that have no conversion
    // path; a compatible pair is answered here and never reaches it.
    ("unit_conversion", handle_unit_conversion),
    ("incompatible_units", try_incompatible_units),
];

/// The ordered specialized-handler table, with precedence read from
/// `data/seed/handler-precedence.lino` (issue #663).
///
/// The seed supplies the *order* (behaviour); [`HANDLER_FUNCTIONS`] supplies the
/// executable *functions* (code). This joins them and validates that the seed is
/// an exact permutation of the registered handlers — every handler present once
/// and only once — panicking otherwise so a seed edit can never silently drop or
/// duplicate a handler.
#[must_use]
pub fn specialized_handlers() -> Vec<(&'static str, SpecializedHandler)> {
    let precedence = crate::seed::handler_precedence();
    // Plan 09 leaf 13 (issue #1138): rows the seed marks `browser_only` name
    // handlers only the browser worker runs. The native surface partitions by
    // phase instead of pretending those rows do not exist.
    let browser_only = crate::seed::browser_only_handlers();
    let native: Vec<&String> = precedence
        .iter()
        .filter(|name| !browser_only.iter().any(|slug| slug == name.as_str()))
        .collect();
    let rule_names = crate::rule_interpreter::handler_names();
    assert_eq!(
        native.len(),
        HANDLER_FUNCTIONS.len() + rule_names.len(),
        "handler-precedence.lino lists {} native handlers ({} total minus {} browser-only) \
         but {} native functions and {} rule sets are registered; the seed must be an \
         exact permutation of both",
        native.len(),
        precedence.len(),
        browser_only.len(),
        HANDLER_FUNCTIONS.len(),
        rule_names.len(),
    );
    let mut seen = std::collections::BTreeSet::new();
    let ordered: Vec<(&'static str, SpecializedHandler)> = native
        .iter()
        .map(|name| {
            assert!(
                seen.insert((*name).clone()),
                "handler-precedence.lino lists handler `{name}` more than once"
            );
            resolve_handler(name, &rule_names).unwrap_or_else(|| {
                panic!(
                    "handler-precedence.lino names handler `{name}`, which is neither registered \
                     in HANDLER_FUNCTIONS nor declared in data/seed/handler-rules.lino"
                )
            })
        })
        .collect();
    debug_assert_eq!(
        seen.len(),
        HANDLER_FUNCTIONS.len() + rule_names.len(),
        "every registered handler must appear in handler-precedence.lino exactly once"
    );
    ordered
}

fn resolve_handler(
    name: &str,
    rule_names: &[&'static str],
) -> Option<(&'static str, SpecializedHandler)> {
    if let Some((registered, handler)) = HANDLER_FUNCTIONS
        .iter()
        .find(|(candidate, _)| *candidate == name)
    {
        assert!(
            !rule_names.contains(registered),
            "handler `{name}` is both a native function and a seed rule set"
        );
        return Some((*registered, SpecializedHandler::Native(*handler)));
    }
    rule_names
        .iter()
        .find(|candidate| **candidate == name)
        .map(|rule| (*rule, SpecializedHandler::Rule(rule)))
}

/// Return the executable handler for a registry method name implemented by the
/// regular solver-handler table or by a seed rule set.
#[must_use]
pub fn handler_for_method(name: &str) -> Option<SpecializedHandler> {
    resolve_handler(name, &crate::rule_interpreter::handler_names()).map(|(_, handler)| handler)
}
