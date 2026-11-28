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
    SelectedRule, SymbolicAnswer, answer_links_notation, language_aware_answer_for,
    language_aware_intent_for, normalize_prompt, response_link_for_intent,
};
use crate::event_log::{EventLog, build_evidence_links};
use crate::intent_formalization::{
    IntentFormalizationCache, IntentFormalizationCacheEntry, record_intent_formalization,
    recover_write_program_rule, rewrite_bare_program_coreference_rule, select_rule_for_intent,
};
use crate::language::detect as detect_language;
use crate::probability::ProbabilityStore;
use crate::rule_synthesis::{
    try_construct_unknown_rule, try_export_substitution_program, try_recall_approved_rule,
};
use crate::rule_synthesis_portfolio::try_portfolio_rule;
use crate::seed;
pub use crate::solver_config::{BlueprintComposition, ExecutionSurface};
use crate::solver_diagnostics::append_diagnostic_trace;
use crate::solver_formalization::{record_formalization, record_formalization_selection};
use crate::solver_handler_oracle::try_unsupported_write_program;
use crate::solver_handlers::{finalize_simple, try_program_blueprint};
use crate::solver_helpers::{
    catalog_execution_recipe, confidence_for, record_answer_test, record_candidates,
    record_decomposition, record_selection, record_validation, unresolved_surfaces_present,
};
use crate::solver_synthesis::try_synthesize_from_sub_results;
use crate::solver_unknown_reasoning::{UnknownReasoningConfig, answer_unknown_prompt};
use crate::translation::{
    FormalizationDecision, FormalizationSelectionConfig, formalize_prompt_candidates,
    select_formalization_candidate_with_policy,
};

mod configuration;
mod conversation;
pub use configuration::SolverConfig;
pub use conversation::{ConversationRole, ConversationTurn};

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
        self.solve_inner(prompt, history, probability_store, intent_cache, true)
    }

    pub(crate) fn solve_for_execution(&self, prompt: &str) -> SymbolicAnswer {
        self.solve_inner(
            prompt,
            &[],
            &ProbabilityStore::new(),
            &mut IntentFormalizationCache::new(),
            false,
        )
    }

    fn solve_inner(
        &self,
        prompt: &str,
        history: &[ConversationTurn],
        probability_store: &ProbabilityStore,
        intent_cache: &mut IntentFormalizationCache,
        publisher_examples: bool,
    ) -> SymbolicAnswer {
        let documentation_access = publisher_examples
            .then(|| self.config.offline || !crate::solver_config::read_network_allowed());
        // Issue #667 (R383): under a debug session the outermost solve of a
        // connection waits at its first stage before it runs.
        let output_turn = crate::cnl_output::output_turn(
            prompt,
            history,
            self.config.output,
            crate::cnl_output::cnl_policy(),
        );
        let prompt = output_turn.prompt.as_str();
        let _turn = crate::server::begin_turn(prompt);
        let mut log = EventLog::new();
        let mut solve_core = |prompt: &str, log: &mut EventLog| -> SymbolicAnswer {
            // Issue #556: when this solve is a forced-language replay, force
            // detection so every localizable handler renders in the requested
            // language. The guard restores the previous value when this function
            // returns, keeping nested replays balanced.
            //
            // Plan 10 leaf 15 (issue #724): a language the conversation has
            // already established binds the same way — the demonstration or
            // retarget that named it speaks for the turns that follow, until the
            // user names another language. An explicit per-run forcing in the
            // config is the stronger statement and wins.
            let _forced_language_guard = crate::language::set_forced_language(
                self.config
                    .forced_response_language
                    .or_else(|| crate::meta_method_answers::established_response_language(history))
                    .and_then(crate::language::from_slug),
            );

            for turn in history {
                let kind: &'static str = match turn.role {
                    ConversationRole::User => "prior_turn:user",
                    ConversationRole::Assistant => "prior_turn:assistant",
                };
                log.append(kind, turn.content.clone());
            }

            if let Some(text) = &output_turn.rewrite {
                return finalize_simple(
                    prompt,
                    log,
                    "cnl_paraphrase",
                    "response:cnl_paraphrase",
                    &crate::cnl_output::output_source(
                        text,
                        detect_language(text).slug(),
                        crate::cnl_output::cnl_policy(),
                    ),
                    1.0,
                );
            }
            log.append("impulse", prompt.to_owned());

            let language = detect_language(prompt);
            log.append("language", language.slug().to_owned());
            probability_store.replay_into_event_log(log, self.config.offline);

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
                record_formalization_selection(log, &formalization_selection);
                if let FormalizationDecision::Clarify { question, .. } =
                    &formalization_selection.decision
                {
                    return finalize_simple(
                        prompt,
                        log,
                        "clarify_interpretation",
                        "response:clarify_interpretation",
                        question,
                        0.5,
                    );
                }
                if let Some(candidate) = formalization_selection.selected_candidate() {
                    record_formalization(log, candidate);
                }
                intent_cache.formalize_or_insert(
                    prompt,
                    language.slug(),
                    formalization_selection.selected_candidate(),
                )
            };
            record_intent_formalization(log, &intent_entry);
            let intent_formalization = intent_entry.formalization;

            // R1012: the general loop can answer before the handler table.
            if let Some(answer) = crate::meta_reasoner::try_meta_answer(
                prompt,
                language.slug(),
                !history.is_empty(),
                log,
            ) {
                return answer;
            }

            // #661: check for contradictions before contextual language replay.
            // The warning binds both requirements to the append-only retraction protocol.
            if let Some(answer) = crate::requirement_contradiction::detect_and_report(
                prompt,
                language,
                history,
                self.config.temperature,
                log,
            ) {
                return answer;
            }

            // Bind caller-supplied data before decomposition or source lookup.
            if let Some(answer) = crate::solver_handlers::prompt_data::try_prompt_data(prompt, log)
            {
                return answer;
            }

            // Issue #559: record the problem frame, recursive work units, needs,
            // methods, and solution evidence (R330–R334). The trace and executable
            // dispatch share the same registry-backed method vocabulary.
            crate::meta_core::record_meta_core(
                log,
                &intent_formalization,
                self.config.max_decomposition_depth,
                self.config.recursion_mode,
                self.config.selection_mode,
                self.config.skill_mode,
            );

            log.append("search:local", prompt.to_owned());

            // Process operands and quoted passages must stay with their questions.
            if let Some(answer) = crate::coding::program_contract::answer(prompt, log)
                .or_else(|| crate::solver_handlers::try_prompt_text_question(prompt, log))
                .or_else(|| {
                    crate::solver_handlers::handle_summarization_request(prompt, prompt, log)
                })
                .or_else(|| {
                    crate::solver_handlers::try_bound_publisher_recurrence(
                        prompt,
                        intent_formalization.route.as_deref(),
                        self.config,
                        language,
                        log,
                    )
                })
            {
                return answer;
            }

            let sub_impulses =
                record_decomposition(log, prompt, self.config.max_decomposition_depth);
            let sub_results =
                self.solve_sub_impulses(log, &sub_impulses, probability_store, intent_cache);

            if let Some(answer) = try_export_substitution_program(prompt, history, log) {
                return answer;
            }

            let selected_rule = select_rule_for_intent(&intent_formalization);
            // Issue #704: with a portfolio configured, the ledger recall and the
            // vocabulary derivation stop being an ordered fallback chain and become
            // independent drafts that are tested against the same fixture and
            // compared. At the default `draft_count` of 1 this is a no-op and the
            // sequential path below runs exactly as before.
            let drafted_rule =
                try_portfolio_rule(selected_rule, prompt, history, log, self.config.draft_count);
            let recalled_rule = try_recall_approved_rule(drafted_rule, prompt, history, log);
            let rule = try_construct_unknown_rule(recalled_rule, prompt, history, log);
            let rule = if let Some(rewrite) =
                rewrite_bare_program_coreference_rule(&rule, prompt, history)
            {
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
                    log,
                ) {
                    return answer;
                }
            }

            // A coding gap can still resolve from a bound procedure or composition.
            if let SelectedRule::UnsupportedWriteProgram {
                task,
                language: program_language,
            } = &rule
            {
                if let (Some(task), Some(target)) = (task.as_deref(), program_language.as_deref())
                    && let Some(answer) =
                        crate::coding::operand_program::try_write_documented_program(
                            prompt,
                            task,
                            target,
                            crate::coding::operand_program::ProgramReply::Program {
                                language,
                                prior_code: coding_guidance::history_has_prior_code(history),
                            },
                            documentation_access,
                            log,
                        )
                {
                    return answer;
                }
                if let Some(answer) = try_unsupported_write_program(
                    prompt,
                    program_language.as_deref(),
                    self.config.blueprint_composition,
                    log,
                ) {
                    return answer;
                }
                let shape =
                    crate::program_skill_gap::shape(task.as_deref(), program_language.as_deref());
                log.append(
                    shape.event(),
                    crate::program_skill_gap::gap_name(
                        task.as_deref(),
                        program_language.as_deref(),
                    ),
                );
                // The dead end keeps the parameters the request did fill, so a
                // reader sees which target it asked for ("laravel", not "php")
                // and which slot is still missing. The worker's dead end carries
                // the same `program_parameter:*` evidence
                // (`formal_ai_worker_write_program_and_research.js`).
                log.append(
                    "program_parameter:language",
                    program_language
                        .as_deref()
                        .unwrap_or(crate::program_skill_gap::MISSING_PARAMETER)
                        .to_owned(),
                );
                log.append(
                    "program_parameter:task",
                    task.as_deref()
                        .unwrap_or(crate::program_skill_gap::MISSING_PARAMETER)
                        .to_owned(),
                );
            }

            if let Some(answer) = try_synthesize_from_sub_results(prompt, log, &sub_results) {
                return answer;
            }

            // A concrete program request outranks concept lookup (#312).
            // Page queries keep the supplied page as their subject (#1163, #1164).
            let is_concrete_write_program = matches!(rule, SelectedRule::WriteProgram(_))
                && !crate::solver_handlers::page_query_text::page_query_text_claims(prompt);
            if !is_concrete_write_program
                && let Some(answer) = crate::meta_method_dispatch::try_dispatch(
                    self,
                    prompt,
                    &intent_formalization,
                    history,
                    log,
                )
            {
                return answer;
            }

            if let Some(answer) = crate::solver_handlers::policy_gates::try_policy_gates(
                self.config.execution_surface,
                prompt,
                log,
                language,
            ) {
                return answer;
            }

            if let Some(answer) = crate::coding::rosetta_request::try_rosetta_code_request(
                prompt,
                &intent_formalization.normalized_text,
                log,
                self.config.offline,
            ) {
                return answer;
            }

            if matches!(rule, SelectedRule::Unknown) {
                if crate::program_coreference::looks_like_ambiguous_program_modification(
                    &normalize_prompt(prompt),
                ) {
                    // Label the question with the seed's requirement-section
                    // marker. A target-less modification cannot proceed until the
                    // user names the target, so this is a blocking requirement;
                    // unlabelled it falls to the `factual` default and issue
                    // #920's necessity gate drops it, leaving an empty answer.
                    let body = seed::localized_response(
                        "ambiguous_modification_clarification",
                        language.slug(),
                    )
                    .map(|question| {
                        crate::question_necessity::requirement_section_marker(language.slug())
                            .map_or_else(
                                || question.clone(),
                                |marker| [marker, format!("- {question}")].join("\n"),
                            )
                    })
                    .unwrap_or_default();
                    return finalize_simple(
                        prompt,
                        log,
                        "ambiguous_modification_clarification",
                        "response:ambiguous_modification_clarification",
                        &body,
                        0.9,
                    );
                }
                if let Some(choice) = record_validation(log, prompt) {
                    let intent = choice.intent;
                    let response_link = response_link_for_intent(&rule, &intent);
                    return finalize_simple(
                        prompt,
                        log,
                        &intent,
                        &response_link,
                        &choice.answer,
                        1.0,
                    );
                }
                // Issue #513: recognize terminal-command requests (visible fix for
                // #511) before falling through to the unknown answer, so a shell
                // request returns an agent_suggestion intent in both engines.
                if crate::verifiable_task::recognise_verifiable(prompt).is_none()
                    && crate::capability_routing::claim_admitted(
                        "terminal_command",
                        prompt,
                        &prompt.to_lowercase(),
                    )
                    && let Some(answer) =
                        crate::solver_terminal::try_terminal_command(prompt, language, log)
                {
                    return answer;
                }
                // Issue #662: no reusable part or rule matched. Combine reasoning,
                // random search, and evolutionary search within the configured
                // compute budget (GOALS.md Universal Solver Goals) before giving up.
                // On budget exhaustion the `search:` evidence stays on the log and
                // the honest unknown-reasoning reply below takes over.
                if let Some(answer) =
                    crate::solver_search::try_budget_search(prompt, log, self.config)
                {
                    return answer;
                }
                // R1012: discovery runs before external research, as the
                // `metaReasonTurn` step does in solveImpl.
                if let Some(answer) =
                    crate::meta_reasoner::try_meta_discovery(prompt, language.slug(), log)
                {
                    return answer;
                }
                let senses = if unresolved_surfaces_present(
                    &crate::engine::normalize_prompt(prompt),
                    language.slug(),
                ) || !crate::solver_helpers::bare_concept_misses(&log).is_empty()
                {
                    crate::solver_search::record_external_search(
                        &self.config,
                        log,
                        prompt,
                        language,
                    )
                } else {
                    Vec::new()
                };
                return answer_unknown_prompt(
                    prompt,
                    language,
                    log,
                    UnknownReasoningConfig {
                        questioning_rigor: self.config.questioning_rigor,
                        offline: self.config.offline,
                        agent_mode: self.config.agent_mode,
                    },
                    &senses,
                );
            }

            if let SelectedRule::WriteProgram(spec) = &rule
                && let Some(answer) = crate::coding::operand_program::try_write_documented_program(
                    prompt,
                    spec.task.slug,
                    spec.language.slug,
                    crate::coding::operand_program::ProgramReply::Program {
                        language,
                        prior_code: coding_guidance::history_has_prior_code(history),
                    },
                    documentation_access,
                    log,
                )
            {
                return answer;
            }
            // A callable's runtime parameter needs no concrete catalog operand.
            let catalog_program = match &rule {
                SelectedRule::WriteProgram(spec) => {
                    crate::coding::catalog_parameters::bind_catalog_program(*spec, prompt)
                }
                _ => None,
            };
            if crate::solver_helpers::catalog_operand_is_unbound(&rule, catalog_program.is_some())
                && let Some(gap) = crate::program_skill_gap::publisher_recurrence_gap(prompt, log)
            {
                return gap;
            }
            let intent = language_aware_intent_for(&rule, language);
            log.append("intent", intent.clone());

            if let SelectedRule::WriteProgram(spec) = &rule {
                if log.first_of("rule_synthesis_candidate").is_none() {
                    crate::coding::record_algorithm_construction(log);
                }
                if catalog_program.is_some()
                    || crate::coding::catalog_parameters::catalog_parameter_guidance(spec.task.slug)
                        .is_none()
                {
                    let status = crate::coding::ExecutionStatus::NotProbed;
                    log.append("execution_status", status.label().to_owned());
                }
                log.append("program_parameter:language", spec.language.slug.to_owned());
                log.append("program_parameter:task", spec.task.slug.to_owned());
                log.append("program_parameters", spec.parameter_summary());
                log.append("legacy_intent", spec.legacy_intent());
                // R1166-4: report each work obligation this executor cannot discharge.
                if crate::intent_formalization::request_carries_work_obligations(prompt)
                    || crate::intent_formalization::request_demands(
                        prompt,
                        crate::intent_formalization::ObligationKind::CiWorkflow,
                    )
                {
                    crate::intent_formalization::record_obligation_gaps(prompt, log);
                }
            }
            // Issue #1165 R1165-1/R1165-2: a verified cache row for this
            // (language, task) pair replaces the catalog template in the
            // answer and in the execution recipe the work-item executor runs.
            let cached_program = match &rule {
                SelectedRule::WriteProgram(spec) if !spec.task.output.is_empty() => {
                    let cache = crate::discovery_production::ProcedureCache::load();
                    let rendered = crate::code_editing::apply_inline_hello_world_source_replacement(
                        prompt,
                        &spec.template.code,
                        *spec,
                    );
                    match crate::discovery_production::cached_write_program(
                        &cache,
                        spec.language.slug,
                        spec.task.slug,
                        &spec.template.code,
                        &rendered,
                    ) {
                        Some(recipe) => {
                            let content_id = format!("0x{:016x}", recipe.content_id);
                            log.append_fields(
                                "procedure_cache",
                                &[
                                    ("outcome", "hit"),
                                    ("language", &recipe.language),
                                    ("task", &recipe.task),
                                    ("content_id", &content_id),
                                ],
                            );
                            Some(recipe.entry.clone())
                        }
                        None => {
                            if rendered == *spec.template.code
                                && log.first_of("procedure_cache").is_none()
                            {
                                crate::discovery_production::record_program_cache_miss(
                                    log,
                                    spec.language.slug,
                                    spec.task.slug,
                                    crate::discovery_production::NO_DOCUMENTATION_CAPTURE,
                                );
                            }
                            None
                        }
                    }
                }
                _ => None,
            };

            let validation_choice = record_validation(log, prompt);
            let intent = validation_choice
                .as_ref()
                .filter(|_| matches!(&rule, SelectedRule::Unknown))
                .map_or(intent, |choice| choice.intent.clone());
            record_candidates(log, &intent);
            let prior = coding_guidance::history_has_prior_code(history);
            let base_answer = match (&validation_choice, &rule) {
                (Some(choice), SelectedRule::Unknown) => choice.answer.clone(),
                _ => language_aware_answer_for(
                    &rule,
                    language,
                    prompt,
                    prior,
                    catalog_program.as_ref(),
                ),
            };
            let base_answer = match (&rule, &cached_program) {
                (SelectedRule::WriteProgram(spec), Some(entry)) => {
                    base_answer.replace(spec.template.code.as_ref(), entry)
                }
                _ => base_answer,
            };
            record_answer_test(log, &intent, &base_answer);
            let base_answer = crate::solver_helpers::enforce_answer_questions(
                &base_answer,
                log,
                &rule,
                catalog_program.is_some(),
            );

            let response_link = response_link_for_intent(&rule, &intent);
            log.append("response", response_link.clone());

            record_selection(log, &intent);
            let trace_id = log.append("trace", intent.clone());

            let evidence_links = build_evidence_links(prompt, &log, &response_link);
            let links_notation =
                answer_links_notation(prompt, &intent, &base_answer, &log, &trace_id);
            let thinking_steps = log.thinking_steps_for_answer(&base_answer);
            let answer =
                append_diagnostic_trace(self.config.diagnostic_mode, base_answer, &links_notation);

            let execution_recipe =
                catalog_execution_recipe(&rule, prompt, catalog_program, cached_program);

            SymbolicAnswer {
                intent,
                answer,
                confidence: confidence_for(&rule, validation_choice.as_ref()),
                evidence_links,
                thinking_steps,
                links_notation,
                execution_recipe,
            }
        };
        let crate::cnl_output::ControlledSolve {
            prompt,
            mut answer,
            projection,
        } = crate::cnl_output::controlled_solve(
            &output_turn,
            history,
            crate::cnl_output::cnl_policy(),
            &mut log,
            &mut solve_core,
        );
        let prompt = prompt.as_str();
        // R1012: a handler that admits an impasse hands the turn to the
        // general loop (`metaResolveImpasse`, js/worker/formal_ai_worker_solve.js).
        crate::meta_reasoner::resolve_impasse(prompt, &mut answer, &mut log);
        // Issue #1550: every answer goes through the recursive procedure
        // (`crate::meta_executor`, twin of `solveAnswer` in
        // js/agentic/crate/meta_executor.mjs). Its trace joins the log, and the
        // composition of the sub-task answers replaces the answer only when
        // least action selects it over the direct derivation. A controlled
        // answer, a clarification and a paraphrase are drafts like any other.
        self.run_meta_procedure(prompt, history, probability_store, &mut answer, &mut log);
        // A publisher miss can leave only a clarification that the question policy removes.
        if answer.answer.trim().is_empty()
            && let Some(gap) = crate::program_skill_gap::publisher_recurrence_gap(prompt, &mut log)
        {
            answer = gap;
        }
        // A controlled answer or clarification the procedure kept is already
        // its own projection (`projectControlledAnswer` in the worker).
        if projection.as_deref() != Some(answer.answer.as_str()) {
            let output_language = self
                .config
                .forced_response_language
                .unwrap_or_else(|| detect_language(&answer.answer).slug());
            let rendered = crate::cnl_output::render_output(
                &crate::cnl_output::formalize_output(
                    &answer.answer,
                    output_language,
                    crate::cnl_output::cnl_policy(),
                ),
                output_turn.config,
                crate::cnl_output::cnl_policy(),
                &(),
            );
            if output_turn.config != crate::cnl_output::OutputConfig::default()
                || rendered.content != answer.answer
            {
                log.append(
                    "cnl_output",
                    serde_json::to_string(&rendered.report).expect("output report"),
                );
                answer
                    .links_notation
                    .push_str(&crate::cnl_output::output_links_notation(
                        &rendered.document,
                        crate::cnl_output::cnl_policy(),
                    ));
            }
            answer.answer = rendered.content;
        }
        crate::derivation::finalize_answer(&mut answer, &mut log);
        crate::dialog_log::record_native_response_if_enabled(
            "UniversalSolver::solve_with_history_probability_store_and_intent_cache",
            prompt,
            history,
            &self.config,
            || {
                serde_json::json!({
                    "probability-store": probability_store.to_links_notation(),
                    "intent-cache-debug": format!("{intent_cache:?}")
                })
            },
            &answer,
        );
        answer
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
