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

use crate::coding::guidance as coding_guidance;
use crate::engine::{
    ExecutionRecipe, SelectedRule, SymbolicAnswer, answer_links_notation,
    language_aware_answer_for, language_aware_intent_for, normalize_prompt,
    response_link_for_intent,
};
use crate::event_log::{EventLog, build_evidence_links};
use crate::intent_formalization::{
    IntentFormalizationCache, IntentFormalizationCacheEntry, record_intent_formalization,
    recover_write_program_rule, rewrite_bare_program_coreference_rule, select_rule_for_intent,
};
use crate::language::{Language, detect as detect_language};
use crate::probability::{ProbabilityDecisionPolicy, ProbabilityStore};
use crate::rule_synthesis::{
    try_construct_unknown_rule, try_export_substitution_program, try_recall_approved_rule,
};
use crate::rule_synthesis_portfolio::try_portfolio_rule;
use crate::seed;
pub use crate::solver_config::{BlueprintComposition, ExecutionSurface};
use crate::solver_diagnostics::append_diagnostic_trace;
use crate::solver_formalization::{record_formalization, record_formalization_selection};
use crate::solver_handler_oracle::try_unsupported_write_program;
use crate::solver_handlers::{finalize_simple, try_agent_workspace_task, try_program_blueprint};
use crate::solver_helpers::{
    confidence_for, is_agent_opt_in, is_agent_request, is_cache_flush_request,
    is_destructive_action, is_forget_request, is_inappropriate_content, is_unbounded_autonomy,
    is_unbounded_loop, record_candidates, record_decomposition, record_validation,
    requires_external_lookup,
};
use crate::solver_synthesis::try_synthesize_from_sub_results;
use crate::solver_unknown_reasoning::{UnknownReasoningConfig, answer_unknown_prompt};
use crate::translation::{
    FormalizationDecision, FormalizationSelectionConfig, formalize_prompt_candidates,
    select_formalization_candidate_with_policy,
};

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
    /// appends a "Clarifying questions" section. The question-necessity policy
    /// then keeps only the smallest requirement-level input the user still has
    /// to confirm so the final research execution is unambiguous.
    pub follow_up_probability: f32,
    /// `0.0` = ignore surrounding context, `1.0` = use all available context.
    pub context_sensitivity: f32,
    /// `0.0` = accept any phrasing, `1.0` = demand fully formal phrasing.
    pub questioning_rigor: f32,
    /// `0.0` = deterministic projection, `1.0` = allow maximum variation.
    pub temperature: f32,
    /// Hard upper bound on recursive sub-impulse expansion.
    pub max_decomposition_depth: u8,
    /// Which directions of the meta core's recursive reasoning are traced
    /// (issue #559): `Both` (default since issue #1073) traces the downward
    /// decomposition and the upward construction pass; `Down`/`Up` narrow it to
    /// one direction. Trace-only whichever way (R13).
    pub recursion_mode: crate::meta_construction::RecursionMode,
    /// Whether the meta core records the method-selection trace (issue #559,
    /// R339): `Record` (default since issue #1073) names the method the registry
    /// resolves for every atomic leaf, or marks the leaf unresolved; `Off`
    /// records nothing.
    /// The registry is the sole dispatch authority (R344), so there is no
    /// legacy baseline to compare against. Trace-only in either mode (R13).
    pub selection_mode: crate::selection::SelectionMode,
    /// Whether the meta core records the skill-accumulation ledger (issue #559,
    /// R342): `Accumulate` (default since issue #1073) distills each satisfied
    /// need into a proposed reusable skill and each blocked need into a
    /// curriculum item; `Off` records nothing. Proposal-only — no skill is auto-promoted without review — and
    /// trace-only either way (R13/C3).
    pub skill_mode: crate::skill_ledger::SkillMode,
    /// Whether the dialogue's symbolic world model is maintained and traced
    /// (issue #702): `Off` (default) leaves the solver exactly as it was — no
    /// current/target contexts are built and the state-query handler declines;
    /// `Track` rebuilds the model from the conversation, records it as a trace
    /// artifact, and answers "what is left to reach my goal?" from the
    /// current->target difference. Trace-only in either mode (R13).
    pub world_model_mode: crate::world_model_dialog::WorldModelMode,
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
    /// Response language forced onto every localizable handler for one replay
    /// (issue #556). `None` is the normal case: each handler renders in the
    /// language detected from the prompt. When a response-language follow-up
    /// ("I do not understand English, write in Russian") replays the previous
    /// request through the whole solver, it sets this to the requested ISO
    /// 639-1 code so *every* answer family that can localize — concept lookup,
    /// repository/project lookup, … — re-renders in that language rather than
    /// only a single hardcoded handler. It also serves as the recursion guard:
    /// a solve whose config already carries a forced language never fires the
    /// follow-up again.
    pub forced_response_language: Option<&'static str>,
    /// Compute budget for the step-7 random/evolutionary search stage (issue
    /// #662), counted in candidate evaluations. When reuse and rule reasoning
    /// produce no candidate for a recognized search problem, the solver spends
    /// up to this many evaluations combining known parts against the generated
    /// tests before falling back to the honest unknown-reasoning reply. `0`
    /// disables the search entirely; the default is intentionally small. The
    /// stream is seeded from the impulse content hash, so the stage stays
    /// deterministic for a given config per the `VISION.md` contract.
    pub compute_budget: u32,
    /// Maximum number of independent candidate drafts evaluated for one
    /// synthesis leaf (issue #704). `1` preserves the historical single-path
    /// behavior. Values above one enable the deterministic parallel portfolio;
    /// each draft is seeded from the impulse plus its ordered draft index.
    pub draft_count: u8,
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
            recursion_mode: crate::meta_construction::RecursionMode::default(),
            selection_mode: crate::selection::SelectionMode::default(),
            skill_mode: crate::skill_ledger::SkillMode::default(),
            world_model_mode: crate::world_model_dialog::WorldModelMode::default(),
            agent_mode: false,
            diagnostic_mode: false,
            offline: false,
            cache_ttl_seconds: 60 * 60 * 24 * 60,
            definition_fusion_by_default: false,
            associative_project_promotion: true,
            execution_surface: ExecutionSurface::default(),
            blueprint_composition: BlueprintComposition::default(),
            probability_policy: ProbabilityDecisionPolicy::default(),
            forced_response_language: None,
            compute_budget: 512,
            draft_count: 1,
        }
    }
}

impl SolverConfig {
    /// Build a [`SolverConfig`] using the documented environment overrides.
    ///
    /// The parsing body lives in `crate::solver_helpers::config_from_env` to keep
    /// this module under the 1000-line cap enforced by `scripts/check-file-size.rs`.
    #[must_use]
    pub fn from_env() -> Self {
        crate::solver_helpers::config_from_env()
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

        // Issue #556: when this solve is a forced-language replay, force
        // detection so every localizable handler renders in the requested
        // language. The guard restores the previous value when this function
        // returns, keeping nested replays balanced.
        let _forced_language_guard = crate::language::set_forced_language(
            self.config
                .forced_response_language
                .and_then(crate::language::from_slug),
        );

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

        // Issue #661 (R384): before any contextual handler runs (a language
        // directive would otherwise be replayed by the response-language
        // follow-up), check whether this newly formalized requirement
        // contradicts a retained one. A clash — same subject, opposite polarity
        // — is surfaced as a warning naming both statements, their weights, and
        // a resolution that reuses the append-only retraction protocol.
        if let Some(answer) = crate::requirement_contradiction::detect_and_report(
            prompt,
            language,
            history,
            self.config.temperature,
            &mut log,
        ) {
            return answer;
        }

        // Issue #559: record the general recursive meta core — problem frame
        // (R330), recursive work-unit decomposition (R332), need-satisfaction
        // ledger (R333), method registry (R331), and the end-to-end solution
        // evidence (R334) — as one cohesive pass. Method selection below is
        // registry-backed, so the trace and the executable dispatch share the
        // same method vocabulary.
        crate::meta_core::record_meta_core(
            &mut log,
            &intent_formalization,
            self.config.max_decomposition_depth,
            self.config.recursion_mode,
            self.config.selection_mode,
            self.config.skill_mode,
        );

        log.append("search:local", prompt.to_owned());

        let sub_impulses =
            record_decomposition(&mut log, prompt, self.config.max_decomposition_depth);
        let sub_results =
            self.solve_sub_impulses(&mut log, &sub_impulses, probability_store, intent_cache);

        if let Some(answer) = try_export_substitution_program(prompt, history, &mut log) {
            return answer;
        }

        let selected_rule = select_rule_for_intent(&intent_formalization);
        // Issue #704: with a portfolio configured, the ledger recall and the
        // vocabulary derivation stop being an ordered fallback chain and become
        // independent drafts that are tested against the same fixture and
        // compared. At the default `draft_count` of 1 this is a no-op and the
        // sequential path below runs exactly as before.
        let drafted_rule = try_portfolio_rule(
            selected_rule,
            prompt,
            history,
            &mut log,
            self.config.draft_count,
        );
        let recalled_rule = try_recall_approved_rule(drafted_rule, prompt, history, &mut log);
        let rule = try_construct_unknown_rule(recalled_rule, prompt, history, &mut log);
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

        // Issue #458: some composite program prompts also contain strong
        // non-program signals such as "search current prices". Let a recognized
        // blueprint recipe preempt those broader fallback handlers before they
        // can claim the request as generic web search. Concrete catalog programs
        // still win above this path.
        if !matches!(rule, SelectedRule::WriteProgram(_)) {
            let language_hint = match &rule {
                SelectedRule::UnsupportedWriteProgram { language, .. } => language.as_deref(),
                _ => None,
            };
            let normalized_for_blueprint = normalize_prompt(prompt);
            if let Some(answer) = try_program_blueprint(
                prompt,
                &normalized_for_blueprint,
                language_hint,
                self.config.blueprint_composition,
                &mut log,
            ) {
                return answer;
            }
        }

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
            // Issue #699 batch 3: every synthesis route missed. Name the gap in
            // the evidence trail — the same `skill_gap` event the procedure
            // compiler emits — so the miss is actionable instead of being
            // rendered as a recitation of the templates we happen to hold.
            //
            // Issue #906: a request that named no implementation language never
            // reached a route, so it is logged under its own event rather than
            // as a gap in what we can synthesize.
            let shape = crate::program_skill_gap::shape(task.as_deref(), language.as_deref());
            log.append(
                shape.event(),
                crate::program_skill_gap::gap_name(task.as_deref(), language.as_deref()),
            );
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
        if !is_concrete_write_program
            && let Some(answer) = crate::meta_method_dispatch::try_dispatch(
                self,
                prompt,
                &intent_formalization,
                history,
                &mut log,
            )
        {
            return answer;
        }

        if let Some(answer) = self.handle_policy(prompt, &mut log, language) {
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
            // Issue #513: recognize terminal-command requests (visible fix for
            // #511) before falling through to the unknown answer, so a shell
            // request returns an agent_suggestion intent in both engines.
            if let Some(answer) =
                crate::solver_terminal::try_terminal_command(prompt, language, &mut log)
            {
                return answer;
            }
            // Issue #662: no reusable part or rule matched. Combine reasoning,
            // random search, and evolutionary search within the configured
            // compute budget (GOALS.md Universal Solver Goals) before giving up.
            // On budget exhaustion the `search:` evidence stays on the log and
            // the honest unknown-reasoning reply below takes over.
            if let Some(answer) =
                crate::solver_search::try_budget_search(prompt, &mut log, self.config)
            {
                return answer;
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
            if log.first_of("rule_synthesis_candidate").is_none() {
                crate::coding::record_algorithm_construction(&mut log);
            }
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
        let base_answer = crate::question_necessity::enforce_questions(&base_answer, &mut log);

        let response_link = response_link_for_intent(&rule, &intent);
        log.append("response", response_link.clone());

        log.append("trace:simplification", "smallest_sufficient".to_owned());
        let trace_id = log.append("trace", intent.clone());

        let evidence_links = build_evidence_links(prompt, &log, &response_link);
        let links_notation = answer_links_notation(prompt, &intent, &base_answer, &log, &trace_id);
        let thinking_steps = log.thinking_steps_for_answer(&base_answer);
        let answer =
            append_diagnostic_trace(self.config.diagnostic_mode, base_answer, &links_notation);

        let execution_recipe = match &rule {
            SelectedRule::WriteProgram(spec) => Some(Box::new(ExecutionRecipe {
                language: spec.language.code_fence.to_owned(),
                source: crate::code_editing::apply_inline_hello_world_source_replacement(
                    prompt,
                    spec.template.code,
                    *spec,
                ),
                path: spec.language.save_as.to_owned(),
                supporting_files: Vec::new(),
                commands: spec
                    .language
                    .execution
                    .check_command
                    .into_iter()
                    .chain(std::iter::once(spec.language.execution.run_command))
                    .map(str::to_owned)
                    .collect(),
            })),
            _ => None,
        };

        SymbolicAnswer {
            intent,
            answer,
            confidence: confidence_for(&rule, validation_choice.as_ref()),
            evidence_links,
            thinking_steps,
            links_notation,
            execution_recipe,
        }
    }

    fn handle_policy(
        &self,
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
            // The HTTP surface is embedded in an agentic CLI harness. Executing
            // here would mutate the server's private temporary workspace while
            // the caller sees no tool call and cannot audit or approve it. API
            // requests therefore stay declarative; `protocol` routes concrete
            // actions through the tools advertised by the client.
            if self.config.execution_surface != ExecutionSurface::HttpServer
                && let Some(answer) = try_agent_workspace_task(prompt, &normalized, log)
            {
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
        log.append(
            "policy:no_fetch_capability",
            "external search requested but no retrieval was executed".to_owned(),
        );
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
