use super::*;
use std::fmt::Write as _;

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
            let formalization_selection = select_formalization_candidate_with_probability_store(
                &formalization_candidates,
                FormalizationSelectionConfig {
                    temperature: self.config.temperature,
                    guess_probability: self.config.guess_probability,
                    questioning_rigor: self.config.questioning_rigor,
                },
                prompt,
                probability_store,
                self.config.offline,
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
        if let SelectedRule::UnsupportedWriteProgram { language, .. } = &rule {
            let normalized = normalize_prompt(prompt);
            if let Some(answer) = try_program_blueprint(
                prompt,
                &normalized,
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
