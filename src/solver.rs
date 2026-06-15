//! Universal problem-solving algorithm.
//!
//! Every prompt the assistant ever receives walks the same 11-step loop
//! described in `VISION.md` and `REQUIREMENTS.md`:
//!
//! 1. **Impulse** — append the raw user message to the event log.
//! 2. **Formalization** — derive an intent (the smallest formal requirement).
//! 3. **Context** — detect the surface language and mode flags.
//! 4. **History lookup** — search local Links Notation knowledge first.
//! 5. **Decomposition** — split composite prompts into sub-impulses.
//! 6. **TDD-style validation** — when the requirement implies a constraint,
//!    generate at least one executable check and record the validation event.
//! 7. **Solution synthesis** — gather candidate answers.
//! 8. **Combination** — pick the smallest sufficient candidate.
//! 9. **Verification** — when execution is implied, surface execution events.
//! 10. **Simplification** — collapse meaning-preserving redundancies.
//! 11. **Documentation** — emit the user-facing reply with a `trace:` pointer.
//!
//! The solver is deterministic for a given [`SolverConfig`] and impulse: the
//! same input always produces the same event log and the same answer. Any
//! "random guessing" is seeded from the content-addressed impulse id so the
//! deterministic-projection invariant from `NON-GOALS.md` is preserved.

use std::fmt::Write as _;

use crate::coding::guidance as coding_guidance;
use crate::engine::{
    answer_links_notation, language_aware_answer_for, language_aware_intent_for,
    response_link_for_intent, stable_id, SelectedRule, SymbolicAnswer,
};
use crate::event_log::{build_evidence_links, EventLog};
use crate::intent_formalization::{
    ordered_handler_names, record_intent_formalization, recover_write_program_rule,
    rewrite_bare_program_coreference_rule, select_rule_for_intent, IntentFormalization,
    IntentFormalizationCache, IntentFormalizationCacheEntry,
};
use crate::language::{detect as detect_language, Language};
use crate::probability::{ProbabilityDecisionPolicy, ProbabilityStore};
use crate::proof_engine::ProofRenderConfig;
use crate::rule_synthesis::try_construct_unknown_rule;
use crate::seed;
use crate::solver_diagnostics::append_diagnostic_trace;
use crate::solver_dispatch::{try_contextual_override, ContextualOutcome, SPECIALIZED_HANDLERS};
use crate::solver_formalization::{record_formalization, record_formalization_selection};
use crate::solver_handler_oracle::try_unsupported_write_program;
use crate::solver_handlers::{
    finalize_simple, try_agent_workspace_task, try_behavior_rules_with_runtime,
    try_definition_merge_by_default, try_feature_capability, try_natural_language_tool_request,
    try_playwright_script, try_project_lookup, CapabilityRuntime, SelfAwarenessRuntime,
};
use crate::solver_helpers::{
    confidence_for, env_bool, env_bounded_f32, env_definition_fusion_by_default, env_truthy,
    is_agent_opt_in, is_agent_request, is_cache_flush_request, is_destructive_action,
    is_forget_request, is_inappropriate_content, is_unbounded_autonomy, is_unbounded_loop,
    record_candidates, record_decomposition, record_validation, requires_external_lookup,
};
use crate::solver_synthesis::try_synthesize_from_sub_results;
use crate::solver_unknown_reasoning::{answer_unknown_prompt, UnknownReasoningConfig};
use crate::translation::{
    formalize_prompt_candidates, select_formalization_candidate_with_policy, FormalizationDecision,
    FormalizationSelectionConfig,
};

/// Runtime surface where the solver is embedded.
///
/// Self-awareness answers use this to avoid claiming browser-only, CLI-only, or
/// server-only affordances in the wrong environment.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionSurface {
    #[default]
    RustLibrary,
    Cli,
    HttpServer,
    Browser,
    Telegram,
    DockerMicroservice,
}

impl ExecutionSurface {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::RustLibrary => "rust_library",
            Self::Cli => "cli",
            Self::HttpServer => "http_server",
            Self::Browser => "browser",
            Self::Telegram => "telegram",
            Self::DockerMicroservice => "docker_microservice",
        }
    }

    fn from_env_value(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "rust" | "rust_library" | "library" | "lib" => Some(Self::RustLibrary),
            "cli" | "terminal" | "shell" => Some(Self::Cli),
            "http" | "http_server" | "server" | "api" => Some(Self::HttpServer),
            "browser" | "web" | "wasm" | "demo" => Some(Self::Browser),
            "telegram" | "telegram_bot" | "bot" => Some(Self::Telegram),
            "docker" | "docker_microservice" | "container" => Some(Self::DockerMicroservice),
            _ => None,
        }
    }
}

/// How the composite-program [`blueprint`](crate::coding::blueprint) synthesizer
/// turns its annotated recipe template into the program shown to the user.
///
/// Issue #340 asked the engine to "try all directions" of program synthesis and
/// let the user switch between them. A blueprint recipe is stored as an annotated
/// template whose optional sub-tasks (error handling, comments, …) are wrapped in
/// `region:<capability>` markers; every emitted program is a *projection* of that
/// template (never the raw, marker-bearing string — markers are always stripped).
/// This knob selects which projection to emit:
///
/// - [`Composed`](Self::Composed) (default, the most promising direction): the
///   program is assembled from exactly the capabilities the request decomposed
///   into — optional regions whose capability the prompt did not ask for are
///   dropped, and when comments were not requested the documentation is stripped
///   too. The same recipe therefore yields genuinely different programs for
///   different requests, which is the honest, anti-memoization demonstration that
///   the code is composed from the decomposition (`NON-GOALS.md`).
/// - [`Documented`](Self::Documented): always emit the fully documented program
///   with every optional region present, regardless of which sub-tasks the
///   request named. Useful as a stable reference and for users who want the
///   maximal annotated program every time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintComposition {
    /// Project the program from the detected capabilities (default).
    #[default]
    Composed,
    /// Always emit the fully documented program with every region present.
    Documented,
}

impl BlueprintComposition {
    /// Stable slug used in the event log and the demo preference value.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Composed => "composed",
            Self::Documented => "documented",
        }
    }

    /// Parse a configuration value (env var or demo preference). Accepts the
    /// canonical slugs plus a few intuitive aliases; returns `None` for anything
    /// unrecognized so callers keep the default.
    #[must_use]
    pub fn from_value(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "composed" | "compose" | "projection" | "project" | "decomposed" => {
                Some(Self::Composed)
            }
            "documented" | "document" | "full" | "verbatim" | "curated" => Some(Self::Documented),
            _ => None,
        }
    }
}

/// Runtime configuration for the universal solver.
///
/// These knobs control the universal loop's tradeoffs and let the same engine
/// be tuned per surface (CLI, HTTP, Telegram) or per user. The default
/// configuration matches the bounded-chat, offline-friendly stance from
/// `GOALS.md` so the engine is safe to embed without further setup.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolverConfig {
    /// `0.0` = always ask a clarifying question, `1.0` = always guess.
    ///
    /// When this is high the engine commits to its best interpretation of the
    /// prompt, shows that interpretation, translates the claim into the
    /// chosen formal system, and executes the proof. When it is low the
    /// engine stays literal and avoids speculative reductions.
    pub guess_probability: f32,
    /// `0.0` = stay action-only, `1.0` = always invite the user to refine the
    /// proof inputs before final execution.
    ///
    /// Independent of `guess_probability`. When this is high the proof engine
    /// appends a "Clarifying questions" section listing every input the user
    /// still has to confirm (axiom set, definitions, proof technique) so the
    /// final research execution is unambiguous.
    pub follow_up_probability: f32,
    /// `0.0` = ignore surrounding context, `1.0` = use all available context.
    pub context_sensitivity: f32,
    /// `0.0` = accept any phrasing, `1.0` = demand fully formal phrasing.
    pub questioning_rigor: f32,
    /// `0.0` = deterministic projection, `1.0` = allow maximum variation.
    pub temperature: f32,
    /// Hard upper bound on recursive sub-impulse expansion.
    pub max_decomposition_depth: u8,
    /// Whether agent mode is opted in. Off by default.
    pub agent_mode: bool,
    /// Whether diagnostic links are echoed inside the user-facing reply.
    pub diagnostic_mode: bool,
    /// When true, the solver must not perform any external lookup.
    pub offline: bool,
    /// Time-to-live for cached external sources, in seconds.
    pub cache_ttl_seconds: u64,
    /// When true, plain definition prompts such as "What is IIR?" use
    /// cross-language definition fusion before falling back to concept lookup.
    pub definition_fusion_by_default: bool,
    /// When true, repository/project questions prefer known projects from
    /// Link Assistant, Link Foundation, and `LinksPlatform` before showing the
    /// generic multi-host repository lookup path.
    pub associative_project_promotion: bool,
    /// Embedding surface used for environment-aware self-description.
    pub execution_surface: ExecutionSurface,
    /// How composite-program blueprints (issue #340) project their annotated
    /// recipe template into the program shown to the user.
    pub blueprint_composition: BlueprintComposition,
    /// Interpretable decision-policy knobs (`CU`/`TU`/`TC`/`SS`) from
    /// arXiv:2605.00940 that govern how symbolic probability evidence ranks
    /// candidates. The default is the paper's recommended baseline, which keeps
    /// the additive exact-evidence behaviour the solver shipped before the
    /// policy existed, so every existing surface is unaffected unless it opts in.
    pub probability_policy: ProbabilityDecisionPolicy,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            guess_probability: 0.8,
            follow_up_probability: 0.75,
            context_sensitivity: 0.6,
            questioning_rigor: 0.4,
            temperature: 0.7,
            max_decomposition_depth: 4,
            agent_mode: false,
            diagnostic_mode: false,
            offline: false,
            cache_ttl_seconds: 60 * 60 * 24 * 60,
            definition_fusion_by_default: false,
            associative_project_promotion: true,
            execution_surface: ExecutionSurface::default(),
            blueprint_composition: BlueprintComposition::default(),
            probability_policy: ProbabilityDecisionPolicy::default(),
        }
    }
}

impl SolverConfig {
    /// Build a [`SolverConfig`] using the documented environment overrides.
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if env_truthy("FORMAL_AI_OFFLINE") {
            config.offline = true;
        }
        if env_truthy("FORMAL_AI_AGENT_MODE") {
            config.agent_mode = true;
        }
        if env_truthy("FORMAL_AI_DIAGNOSTIC_MODE") {
            config.diagnostic_mode = true;
        }
        if let Some(value) = env_definition_fusion_by_default() {
            config.definition_fusion_by_default = value;
        }
        if let Some(value) = env_bool("FORMAL_AI_ASSOCIATIVE_PROJECT_PROMOTION")
            .or_else(|| env_bool("FORMAL_AI_PROJECT_PROMOTION"))
        {
            config.associative_project_promotion = value;
        }
        if let Ok(value) = std::env::var("FORMAL_AI_EXECUTION_SURFACE")
            .or_else(|_| std::env::var("FORMAL_AI_SURFACE"))
        {
            if let Some(surface) = ExecutionSurface::from_env_value(&value) {
                config.execution_surface = surface;
            }
        }
        if let Some(value) = env_bounded_f32("FORMAL_AI_TEMPERATURE", 0.0, 1.0) {
            config.temperature = value;
        }
        if let Some(value) = env_bounded_f32("FORMAL_AI_GUESS_PROBABILITY", 0.0, 1.0) {
            config.guess_probability = value;
        }
        if let Some(value) = env_bounded_f32("FORMAL_AI_FOLLOW_UP_PROBABILITY", 0.0, 1.0) {
            config.follow_up_probability = value;
        }
        if let Ok(value) = std::env::var("FORMAL_AI_CACHE_TTL_SECONDS") {
            if let Ok(parsed) = value.parse::<u64>() {
                config.cache_ttl_seconds = parsed;
            }
        }
        if let Ok(value) = std::env::var("FORMAL_AI_BLUEPRINT_COMPOSITION")
            .or_else(|_| std::env::var("FORMAL_AI_PROGRAM_COMPOSITION"))
        {
            if let Some(mode) = BlueprintComposition::from_value(&value) {
                config.blueprint_composition = mode;
            }
        }
        config
    }
}

/// Speaker role for [`ConversationTurn`]. The solver only inspects user
/// turns when recalling prior context; assistant turns are kept in the log
/// so the trace stays balanced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationRole {
    User,
    Assistant,
}

impl ConversationRole {
    /// Lowercase slug used in `prior_turn:<role>` event kinds.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

/// A single message in a multi-turn conversation.
///
/// The solver records every turn as a `prior_turn:<role>` event before
/// processing the current impulse so memory recall is grounded in the
/// append-only log, not in implicit state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationTurn {
    pub role: ConversationRole,
    pub content: String,
}

impl ConversationTurn {
    /// Construct a user turn.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ConversationRole::User,
            content: content.into(),
        }
    }

    /// Construct an assistant turn.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ConversationRole::Assistant,
            content: content.into(),
        }
    }
}

/// The universal solver itself. See module docs for the 11-step loop.
#[derive(Debug, Clone, Copy)]
pub struct UniversalSolver {
    pub config: SolverConfig,
}

impl Default for UniversalSolver {
    fn default() -> Self {
        Self {
            config: SolverConfig::from_env(),
        }
    }
}

impl UniversalSolver {
    /// Construct a solver with an explicit configuration.
    #[must_use]
    pub const fn new(config: SolverConfig) -> Self {
        Self { config }
    }

    /// Run the universal loop against a single user impulse and return the
    /// projected [`SymbolicAnswer`]. Every step is recorded in the in-process
    /// append-only log so the user-facing answer is, by construction, a
    /// projection of an inspectable trace.
    #[must_use]
    pub fn solve(&self, prompt: &str) -> SymbolicAnswer {
        self.solve_with_history(prompt, &[])
    }

    /// Run the universal loop with conversational context. Each prior turn is
    /// appended to the event log as `prior_turn:user` or `prior_turn:assistant`
    /// before the current impulse, so memory-recall handlers can search the
    /// log instead of holding implicit state.
    #[must_use]
    pub fn solve_with_history(&self, prompt: &str, history: &[ConversationTurn]) -> SymbolicAnswer {
        self.solve_with_history_and_probability_store(prompt, history, &ProbabilityStore::new())
    }

    #[must_use]
    pub fn solve_with_probability_store(
        &self,
        prompt: &str,
        probability_store: &ProbabilityStore,
    ) -> SymbolicAnswer {
        self.solve_with_history_and_probability_store(prompt, &[], probability_store)
    }

    #[must_use]
    pub fn solve_with_history_and_probability_store(
        &self,
        prompt: &str,
        history: &[ConversationTurn],
        probability_store: &ProbabilityStore,
    ) -> SymbolicAnswer {
        let mut intent_cache = IntentFormalizationCache::new();
        self.solve_with_history_probability_store_and_intent_cache(
            prompt,
            history,
            probability_store,
            &mut intent_cache,
        )
    }

    pub(crate) fn solve_with_history_probability_store_and_intent_cache(
        &self,
        prompt: &str,
        history: &[ConversationTurn],
        probability_store: &ProbabilityStore,
        intent_cache: &mut IntentFormalizationCache,
    ) -> SymbolicAnswer {
        let mut log = EventLog::new();

        for turn in history {
            let kind: &'static str = match turn.role {
                ConversationRole::User => "prior_turn:user",
                ConversationRole::Assistant => "prior_turn:assistant",
            };
            log.append(kind, turn.content.clone());
        }

        log.append("impulse", prompt.to_owned());

        let language = detect_language(prompt);
        log.append("language", language.slug().to_owned());
        probability_store.replay_into_event_log(&mut log, self.config.offline);

        let intent_entry = if let Some(formalization) = intent_cache.get(prompt).cloned() {
            IntentFormalizationCacheEntry {
                formalization,
                cache_hit: true,
            }
        } else {
            let formalization_candidates = formalize_prompt_candidates(prompt, language.slug());
            let formalization_selection = select_formalization_candidate_with_policy(
                &formalization_candidates,
                FormalizationSelectionConfig {
                    temperature: self.config.temperature,
                    guess_probability: self.config.guess_probability,
                    questioning_rigor: self.config.questioning_rigor,
                },
                prompt,
                probability_store,
                self.config.offline,
                self.config.probability_policy,
            );
            record_formalization_selection(&mut log, &formalization_selection);
            if let FormalizationDecision::Clarify { question, .. } =
                &formalization_selection.decision
            {
                return finalize_simple(
                    prompt,
                    &mut log,
                    "clarify_interpretation",
                    "response:clarify_interpretation",
                    question,
                    0.5,
                );
            }
            if let Some(candidate) = formalization_selection.selected_candidate() {
                record_formalization(&mut log, candidate);
            }
            intent_cache.formalize_or_insert(
                prompt,
                language.slug(),
                formalization_selection.selected_candidate(),
            )
        };
        record_intent_formalization(&mut log, &intent_entry);
        let intent_formalization = intent_entry.formalization;

        log.append("search:local", prompt.to_owned());

        let sub_impulses =
            record_decomposition(&mut log, prompt, self.config.max_decomposition_depth);
        let sub_results =
            self.solve_sub_impulses(&mut log, &sub_impulses, probability_store, intent_cache);

        let selected_rule = select_rule_for_intent(&intent_formalization);
        let rule = try_construct_unknown_rule(selected_rule, prompt, history, &mut log);
        let rule =
            if let Some(rewrite) = rewrite_bare_program_coreference_rule(&rule, prompt, history) {
                log.append("write_program_coreference_rewrite", rewrite.trace);
                rewrite.rule
            } else {
                rule
            };

        // Issue #324: a follow-up modification ("make the program accept a path
        // argument") routes to write_program but names no concrete task or
        // language — they came from the previous turn. Recover the missing
        // parameters from the conversation so the request completes instead of
        // surfacing the "language `missing` and task `missing`" error.
        let rule = if matches!(rule, SelectedRule::UnsupportedWriteProgram { .. }) {
            let recovery = recover_write_program_rule(rule, prompt, history);
            if let Some(trace) = recovery.trace {
                log.append("write_program_context_recovery", trace);
            }
            if let Some(plan) = recovery.plan {
                log.append("write_program_plan", plan);
            }
            recovery.rule
        } else {
            rule
        };

        // Issue #340: a `write_program` request can name a supported language but
        // a composite task the verified catalog has no single template for
        // (HTTP GET -> parse JSON -> compute mean/median -> output). Rather than
        // dead-ending on `write_program_unsupported`, decompose the request into
        // capabilities and, when they match a curated blueprint recipe, return a
        // real, idiomatic program with an honest "not run" execution report. The
        // verified catalog stays untouched, so its "compiled and ran" guarantee
        // is preserved.
        // Issue #340 + #412: rescue an `UnsupportedWriteProgram` request via the
        // composite blueprint, then the cached coding oracle (uncatalogued
        // languages), so "write a hello world program in Kotlin" returns code.
        if let SelectedRule::UnsupportedWriteProgram { task, language } = &rule {
            if let Some(answer) = try_unsupported_write_program(
                prompt,
                task.as_deref(),
                language.as_deref(),
                self.config.blueprint_composition,
                &mut log,
            ) {
                return answer;
            }
        }

        if let Some(answer) = try_synthesize_from_sub_results(
            prompt,
            &mut log,
            &sub_results,
            probability_store,
            self.config,
        ) {
            return answer;
        }

        // Issue #312: a concrete write_program request (recognized task and
        // language with a matching template) must take precedence over the
        // specialized handlers. Otherwise concept_lookup answers the language
        // name ("Rust") as an encyclopedia definition instead of returning the
        // requested program. Policy guards still run for these prompts below.
        let is_concrete_write_program = matches!(rule, SelectedRule::WriteProgram(_));
        if !is_concrete_write_program {
            if let Some(answer) =
                self.handle_specialized_pattern(prompt, &intent_formalization, history, &mut log)
            {
                return answer;
            }
        }

        if let Some(answer) = Self::handle_policy(prompt, &mut log, language) {
            return answer;
        }

        if matches!(rule, SelectedRule::Unknown) {
            let intent = language_aware_intent_for(&rule, language);
            record_candidates(&mut log, prompt, &intent);
            if let Some(choice) = record_validation(&mut log, prompt) {
                let response_link = response_link_for_intent(&rule, &intent);
                return finalize_simple(
                    prompt,
                    &mut log,
                    &intent,
                    &response_link,
                    &choice.answer,
                    1.0,
                );
            }
            if requires_external_lookup(prompt) {
                self.record_external_search(&mut log, prompt);
            }
            return answer_unknown_prompt(
                prompt,
                language,
                &mut log,
                UnknownReasoningConfig {
                    questioning_rigor: self.config.questioning_rigor,
                    offline: self.config.offline,
                },
            );
        }

        let intent = language_aware_intent_for(&rule, language);
        log.append("intent", intent.clone());

        if let SelectedRule::WriteProgram(spec) = &rule {
            log.append(
                "execution_status",
                spec.language.execution.status.label().to_owned(),
            );
            log.append(
                "execution_environment",
                spec.language.execution.environment.to_owned(),
            );
            log.append("program_parameter:language", spec.language.slug.to_owned());
            log.append("program_parameter:task", spec.task.slug.to_owned());
            log.append("program_parameters", spec.parameter_summary());
            log.append("legacy_intent", spec.legacy_intent());
        }

        record_candidates(&mut log, prompt, &intent);

        let validation_choice = record_validation(&mut log, prompt);
        if validation_choice.is_none() && log.first_of("validation").is_none() {
            log.append(
                "validation",
                "accepted_without_extra_constraints".to_owned(),
            );
        }
        let prior = coding_guidance::history_has_prior_code(history);
        let base_answer = match (&validation_choice, &rule) {
            (Some(choice), SelectedRule::Unknown) => choice.answer.clone(),
            _ => language_aware_answer_for(&rule, language, prompt, prior),
        };

        let response_link = response_link_for_intent(&rule, &intent);
        log.append("response", response_link.clone());

        log.append("trace:simplification", "smallest_sufficient".to_owned());
        let trace_id = log.append("trace", intent.clone());

        let evidence_links = build_evidence_links(prompt, &log, &response_link);
        let links_notation = answer_links_notation(prompt, &intent, &base_answer, &log, &trace_id);
        let thinking_steps = log.thinking_steps();
        let answer =
            append_diagnostic_trace(self.config.diagnostic_mode, base_answer, &links_notation);

        SymbolicAnswer {
            intent,
            answer,
            confidence: confidence_for(&rule, validation_choice.as_ref()),
            evidence_links,
            thinking_steps,
            links_notation,
        }
    }

    fn handle_specialized_pattern(
        &self,
        prompt: &str,
        intent_formalization: &IntentFormalization,
        history: &[ConversationTurn],
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        let normalized = prompt.to_lowercase();

        if let Some(answer) = self.try_diagnostic(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) =
            try_natural_language_tool_request(prompt, &normalized, log, self.config.agent_mode)
        {
            log.append("specialized_handler", "nl_tool".to_owned());
            return Some(answer);
        }
        let capability_runtime = CapabilityRuntime::new(
            self.config.offline,
            self.config.agent_mode,
            self.config.diagnostic_mode,
            self.config.definition_fusion_by_default,
        );
        let self_awareness_runtime = SelfAwarenessRuntime::new(
            self.config.execution_surface,
            self.config.offline,
            self.config.agent_mode,
            self.config.diagnostic_mode,
            self.config.definition_fusion_by_default,
            self.config.blueprint_composition,
        );
        if let Some(answer) =
            try_behavior_rules_with_runtime(prompt, &normalized, log, self_awareness_runtime)
        {
            log.append("specialized_handler", "behavior_rules".to_owned());
            return Some(answer);
        }
        if let Some(answer) = try_feature_capability(prompt, &normalized, log, capability_runtime) {
            log.append("specialized_handler", "feature_capability".to_owned());
            return Some(answer);
        }
        if let Some(answer) =
            try_playwright_script(prompt, &normalized, log, self.config.guess_probability)
        {
            log.append("specialized_handler", "playwright_script".to_owned());
            return Some(answer);
        }
        let proof_render_config = ProofRenderConfig {
            guess_probability: self.config.guess_probability,
            follow_up_probability: self.config.follow_up_probability,
        };
        let handler_names = ordered_handler_names(
            intent_formalization,
            SPECIALIZED_HANDLERS.iter().map(|(name, _)| *name),
        );
        for name in handler_names {
            let Some((_, handler)) = SPECIALIZED_HANDLERS
                .iter()
                .find(|(candidate, _)| *candidate == name)
            else {
                continue;
            };
            if self.config.definition_fusion_by_default && name == "concept_lookup" {
                if let Some(answer) = try_definition_merge_by_default(prompt, log) {
                    log.append(
                        "specialized_handler",
                        "definition_merge_by_default".to_owned(),
                    );
                    return Some(answer);
                }
            }
            // A few handlers need more than the uniform signature: the proof
            // handler depends on the solver configuration sliders, the
            // meta-explanation handler on the self-awareness runtime, and the
            // numeric-list handler on the conversation history (issue #412, so a
            // bare follow-up "Отсортируй 4, 3, 1, 17, 8, 9, 15" inherits the
            // language and code request established by an earlier coding turn).
            // Their entries stay in `SPECIALIZED_HANDLERS` to keep the precedence
            // order documented in one place; `try_contextual_override` routes the
            // few contextual names through their richer variants.
            match try_contextual_override(
                name,
                prompt,
                &normalized,
                history,
                proof_render_config,
                self_awareness_runtime,
                log,
            ) {
                ContextualOutcome::Answer(answer) => return Some(answer),
                ContextualOutcome::Skip => continue,
                ContextualOutcome::NotHandled => {}
            }
            if let Some(answer) = handler(prompt, &normalized, log) {
                log.append("specialized_handler", name.to_owned());
                return Some(answer);
            }
            if name == "concept_lookup" {
                if let Some(answer) = try_project_lookup(
                    prompt,
                    &normalized,
                    log,
                    self.config.associative_project_promotion,
                    intent_formalization.route.as_deref() == Some("identity"),
                ) {
                    log.append("specialized_handler", "project_lookup".to_owned());
                    return Some(answer);
                }
            }
        }
        None
    }

    fn try_diagnostic(
        &self,
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        if !normalized.contains("[diagnostic]") {
            return None;
        }
        log.append("diagnostic_mode", "active".to_owned());
        let stripped = prompt.replace("[diagnostic]", "").trim().to_owned();
        let inner_solver = Self::new(self.config);
        let inner = inner_solver.solve(&stripped);
        let mut decorated = inner.answer.clone();
        decorated.push_str("\n\n[diagnostic]\n");
        decorated.push_str(inner.links_notation.trim_end());
        decorated.push('\n');
        for link in &inner.evidence_links {
            let _ = writeln!(decorated, "evidence: {link}");
        }
        let _ = writeln!(decorated, "trace: {}", inner.intent);
        log.append("intent", inner.intent.clone());
        let response_link = format!("response:diagnostic:{}", inner.intent);
        log.append("response", response_link.clone());
        let trace_id = log.append("trace", inner.intent.clone());
        let evidence_links = build_evidence_links(prompt, log, &response_link);
        let links_notation =
            answer_links_notation(prompt, &inner.intent, &decorated, log, &trace_id);
        let thinking_steps = log.thinking_steps();
        Some(SymbolicAnswer {
            intent: inner.intent,
            answer: decorated,
            confidence: inner.confidence,
            evidence_links,
            thinking_steps,
            links_notation,
        })
    }

    fn handle_policy(
        prompt: &str,
        log: &mut EventLog,
        language: Language,
    ) -> Option<SymbolicAnswer> {
        let normalized = prompt.to_lowercase();

        if is_inappropriate_content(&normalized) {
            log.append("policy:inappropriate_content", prompt.to_owned());
            let lang_slug = language.slug();
            let fallback = "That message contains inappropriate content. Please keep the conversation respectful.";
            let body = seed::response_for("inappropriate_content", lang_slug)
                .unwrap_or_else(|| String::from(fallback));
            return Some(Self::finalize_policy(
                prompt,
                log,
                "inappropriate_content",
                language,
                &body,
            ));
        }

        if is_unbounded_autonomy(&normalized) && !is_agent_opt_in(&normalized) {
            log.append("policy:chat_bounded_autonomy", prompt.to_owned());
            return Some(Self::finalize_policy(
                prompt,
                log,
                "bounded_autonomy",
                language,
                concat!(
                    "I can only run a bounded chat reply per message. To take repeated, ",
                    "open-ended actions I need an explicit opt-in to agent mode, and agent ",
                    "mode runs in an isolated sandbox so the host stays safe."
                ),
            ));
        }

        if is_forget_request(&normalized) {
            log.append("policy:add_only_history", prompt.to_owned());
            return Some(Self::finalize_policy(
                prompt,
                log,
                "add_only_history",
                language,
                concat!(
                    "The link network is append-only. To retract a fact, send the explicit ",
                    "retraction protocol; it will append a superseding event without erasing ",
                    "history."
                ),
            ));
        }

        if is_cache_flush_request(&normalized) {
            log.append(
                "policy:cache_flush_requires_confirmation",
                prompt.to_owned(),
            );
            return Some(Self::finalize_policy(
                prompt,
                log,
                "cache_flush_requires_confirmation",
                language,
                "Flushing the source cache is an auditable action. Confirm explicitly.",
            ));
        }

        if is_agent_request(&normalized) && is_destructive_action(&normalized) {
            log.append("agent_mode:opted_in", prompt.to_owned());
            log.append(
                "policy:destructive_action_requires_confirmation",
                prompt.to_owned(),
            );
            return Some(Self::finalize_policy(
                prompt,
                log,
                "destructive_action_requires_confirmation",
                language,
                concat!(
                    "Destructive agent actions require an explicit human confirmation. ",
                    "The action will run inside an isolated sandbox once confirmed."
                ),
            ));
        }

        if is_agent_request(&normalized) && is_unbounded_loop(&normalized) {
            log.append("agent_mode:opted_in", prompt.to_owned());
            log.append("policy:agent_time_budget", prompt.to_owned());
            return Some(Self::finalize_policy(
                prompt,
                log,
                "agent_time_budget",
                language,
                concat!(
                    "Agent execution is bounded by a documented time budget; unbounded ",
                    "loops are refused. Re-send a bounded version inside an isolated sandbox."
                ),
            ));
        }

        if is_agent_request(&normalized) {
            if let Some(answer) = try_agent_workspace_task(prompt, &normalized, log) {
                return Some(answer);
            }
            log.append("agent_mode:opted_in", prompt.to_owned());
            log.append("agent_mode:active", prompt.to_owned());
            log.append("action_log", prompt.to_owned());
            return Some(Self::finalize_policy(
                prompt,
                log,
                "agent_action",
                language,
                concat!(
                    "Agent mode is opted in for this message. The action will run inside ",
                    "an isolated sandbox (docker, webvm or sandbox-equivalent) and every ",
                    "step will be appended to the action log."
                ),
            ));
        }

        None
    }

    fn finalize_policy(
        prompt: &str,
        log: &mut EventLog,
        intent_slug: &str,
        _language: Language,
        body: &str,
    ) -> SymbolicAnswer {
        let intent = format!("policy_{intent_slug}");
        let response_link = format!("response:policy:{intent_slug}");
        finalize_simple(prompt, log, &intent, &response_link, body, 0.5)
    }

    fn record_external_search(&self, log: &mut EventLog, prompt: &str) {
        if self.config.offline {
            log.append("search:external", "skipped:offline".to_owned());
            return;
        }
        log.append("search:external", prompt.to_owned());
        let source_id = stable_id("source", prompt);
        let fetched_at = "1970-01-01T00:00:00Z";
        let sha256 = stable_id("sha256", prompt);
        log.append(
            "source:http",
            format!("https://example.org/{source_id} fetched_at={fetched_at} sha256={sha256}"),
        );
        log.append("cache_hit", source_id);
    }
}

/// Convenience entry point that mirrors [`UniversalSolver::solve`] using the
/// environment-derived [`SolverConfig`]. The deterministic-projection
/// guarantee from `NON-GOALS.md` is preserved.
#[must_use]
pub fn solve(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::default().solve(prompt)
}

/// Convenience entry point that mirrors [`UniversalSolver::solve_with_history`].
#[must_use]
pub fn solve_with_history(prompt: &str, history: &[ConversationTurn]) -> SymbolicAnswer {
    UniversalSolver::default().solve_with_history(prompt, history)
}
