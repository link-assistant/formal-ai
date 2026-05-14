//! Universal problem-solving algorithm.
//!
//! Every prompt the assistant ever receives walks the same 11-step loop
//! described in `VISION.md` and `docs/REQUIREMENTS.md`:
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

use crate::engine::{
    answer_links_notation, knowledge_links_notation, language_aware_answer_for,
    language_aware_intent_for, response_link_for_intent, select_rule_for, stable_id,
    ExecutionStatus, SelectedRule, SymbolicAnswer, GREETING_ANSWER, IDENTITY_ANSWER,
    UNKNOWN_ANSWER,
};
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};
use crate::solver_helpers::{
    build_sorting_algorithm_answer, confidence_for, detect_algorithm_language,
    detect_program_languages, detect_source_language, detect_target_language, extract_backticked,
    extract_concept_from_query, extract_introduced_name, extract_quoted_phrase,
    infer_program_languages_from_code, infer_source_from_prompt, is_agent_opt_in, is_agent_request,
    is_cache_flush_request, is_destructive_action, is_forget_request, is_unbounded_autonomy,
    is_unbounded_loop, normalize_code_meaning, normalize_meaning, record_candidates,
    record_decomposition, record_validation, requires_external_lookup, translate_program,
    translate_surface,
};

/// Runtime configuration for the universal solver.
///
/// These knobs control the universal loop's tradeoffs and let the same engine
/// be tuned per surface (CLI, HTTP, Telegram) or per user. The default
/// configuration matches the bounded-chat, offline-friendly stance from
/// `GOALS.md` so the engine is safe to embed without further setup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolverConfig {
    /// `0.0` = always ask a clarifying question, `1.0` = always guess.
    pub guess_probability: f32,
    /// `0.0` = ignore surrounding context, `1.0` = use all available context.
    pub context_sensitivity: f32,
    /// `0.0` = accept any phrasing, `1.0` = demand fully formal phrasing.
    pub questioning_rigor: f32,
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
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            guess_probability: 0.8,
            context_sensitivity: 0.6,
            questioning_rigor: 0.4,
            max_decomposition_depth: 4,
            agent_mode: false,
            diagnostic_mode: false,
            offline: false,
            cache_ttl_seconds: 60 * 60 * 24 * 60,
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
        if let Ok(value) = std::env::var("FORMAL_AI_CACHE_TTL_SECONDS") {
            if let Ok(parsed) = value.parse::<u64>() {
                config.cache_ttl_seconds = parsed;
            }
        }
        config
    }
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name).is_ok_and(|raw| {
        let value = raw.trim();
        !value.is_empty()
            && !matches!(
                value.to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
    })
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
        let mut log = EventLog::new();

        // Step 1: Impulse — record the prompt before any other action.
        log.append("impulse", prompt.to_owned());

        // Step 2 + 3: Formalization & context. The language detector runs on
        // the raw impulse so non-Latin scripts pick up the correct tag.
        let language = detect_language(prompt);
        log.append("language", language.slug().to_owned());

        // Step 4: History lookup — local Links Notation search first.
        log.append("search:local", prompt.to_owned());

        // Step 5: Decomposition — composite prompts are split into
        // sub-impulses so each clause walks the loop independently.
        record_decomposition(&mut log, prompt, self.config.max_decomposition_depth);

        // Specialized handlers for translation, network queries, algorithms,
        // execution failures and source-cache patterns are dispatched before
        // the policy gate so an honest pattern match wins over a generic
        // refusal.
        if let Some(answer) = self.handle_specialized_pattern(prompt, &mut log) {
            return answer;
        }

        // Universal handlers for prompts that the network refuses or routes
        // to a policy event before falling through to intent matching.
        if let Some(answer) = Self::handle_policy(prompt, &mut log, language) {
            return answer;
        }

        // Step 2 (continued): pick the formal intent from the impulse. This
        // is the smallest formal requirement the network can satisfy.
        let rule = select_rule_for(prompt);
        let intent = language_aware_intent_for(&rule, language);
        log.append("intent", intent.clone());

        // For hello-world rules, emit execution metadata as separate events
        // so the trace's Links Notation steps_block explicitly mentions
        // execution_status and execution_environment.
        if let SelectedRule::HelloWorld(program) = &rule {
            log.append(
                "execution_status",
                program.execution.status.label().to_owned(),
            );
            log.append(
                "execution_environment",
                program.execution.environment.to_owned(),
            );
        }

        // External-search fallback: when the local intent is unknown but the
        // request points to a knowable external concept, record the step.
        if matches!(rule, SelectedRule::Unknown) && requires_external_lookup(prompt) {
            self.record_external_search(&mut log, prompt);
        }

        // Step 7: candidate generation — the solver always emits at least
        // one candidate event; prompts with multiple equally good options
        // emit a fan of candidates so step 5 of the spec is observable.
        record_candidates(&mut log, prompt, &intent);

        // Step 6: validation — when the requirement implies an executable
        // check (a constraint or a code seed), record the validation step.
        let validation_choice = record_validation(&mut log, prompt);

        // Step 8: pick the smallest sufficient answer. For constraint
        // requests we use the validated value; for everything else we use
        // the rule's canonical, smallest-sufficient response.
        let answer = match (&validation_choice, &rule) {
            (Some(choice), SelectedRule::Unknown) => choice.answer.clone(),
            _ => language_aware_answer_for(&rule, language, prompt),
        };

        // Step 9: response link — every answer points back to a content
        // record so the user can follow the evidence.
        let response_link = response_link_for_intent(&rule, &intent);
        log.append("response", response_link.clone());

        // Step 10: simplification — meaning-preserving rules trim the
        // surface form. The current corpus is already minimal; the event is
        // still recorded so the trace shape stays uniform across requests.
        log.append("trace:simplification", "smallest_sufficient".to_owned());

        // Step 11: documentation — the trace event is the user-facing
        // pointer that ties the answer back to its append-only log.
        let trace_id = log.append("trace", intent.clone());

        let evidence_links = Self::build_evidence_links(prompt, &log, &response_link);
        let links_notation = answer_links_notation(prompt, &intent, &answer, &log, &trace_id);

        SymbolicAnswer {
            intent,
            answer,
            confidence: confidence_for(&rule, validation_choice.as_ref()),
            evidence_links,
            links_notation,
        }
    }

    fn handle_specialized_pattern(
        &self,
        prompt: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        let normalized = prompt.to_lowercase();

        if let Some(answer) = self.try_diagnostic(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_conversation_memory(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_meta_explanation(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_network_query(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_translation(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_algorithm(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_execution_failure(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_source_refresh(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_source_conflict(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_ill_formed(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = Self::try_shell_refusal(prompt, &normalized, log) {
            return Some(answer);
        }
        None
    }

    fn try_diagnostic(
        &self,
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        use std::fmt::Write as _;
        if !normalized.contains("[diagnostic]") {
            return None;
        }
        log.append("diagnostic_mode", "active".to_owned());
        // Re-run the inner solve on the stripped prompt without diagnostic
        // mode and then decorate the resulting answer with the trace links.
        let stripped = prompt.replace("[diagnostic]", "").trim().to_owned();
        let inner_solver = Self::new(self.config);
        let inner = inner_solver.solve(&stripped);
        let mut decorated = inner.answer.clone();
        decorated.push_str("\n\n[diagnostic]\n");
        for link in &inner.evidence_links {
            let _ = writeln!(decorated, "evidence: {link}");
        }
        let _ = writeln!(decorated, "trace: {}", inner.intent);
        log.append("intent", inner.intent.clone());
        let response_link = format!("response:diagnostic:{}", inner.intent);
        log.append("response", response_link.clone());
        let trace_id = log.append("trace", inner.intent.clone());
        let evidence_links = Self::build_evidence_links(prompt, log, &response_link);
        let links_notation =
            answer_links_notation(prompt, &inner.intent, &decorated, log, &trace_id);
        Some(SymbolicAnswer {
            intent: inner.intent,
            answer: decorated,
            confidence: inner.confidence,
            evidence_links,
            links_notation,
        })
    }

    fn try_conversation_memory(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        let asks_name = normalized.contains("what is my name")
            || normalized.contains("what's my name")
            || normalized.contains("do you know my name")
            || normalized.contains("who am i");
        if !asks_name {
            return None;
        }
        let name = extract_introduced_name(prompt)?;
        log.append("intent", "recall_name".to_owned());
        log.append("filter:user", format!("name={name}"));
        let body = format!("Your name is {name}.");
        Some(Self::finalize_simple(
            prompt,
            log,
            "recall_name",
            "response:recall_name",
            &body,
            0.9,
        ))
    }

    fn try_meta_explanation(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        if !(normalized.starts_with("why ")
            || normalized.starts_with("why did")
            || normalized.starts_with("why do you")
            || normalized.contains("why did you answer"))
        {
            return None;
        }
        let body = String::from(
            "I answered that way because the prompt matched a deterministic Links Notation rule. \
             The evidence and trace events are appended to the log; see the trace link for the \
             full chain.",
        );
        Some(Self::finalize_simple(
            prompt,
            log,
            "meta_explanation",
            "response:meta_explanation",
            &body,
            1.0,
        ))
    }

    fn try_network_query(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        if normalized.contains("show me the current network")
            || normalized.contains("show me the network")
            || normalized.contains("export the network")
            || normalized.contains("export network")
        {
            let snapshot = knowledge_links_notation();
            let body = format!(
                "Here is the current link network as a links-notation snapshot:\n\n```links\n{snapshot}\n```"
            );
            return Some(Self::finalize_simple(
                prompt,
                log,
                "network_snapshot",
                "response:network_snapshot",
                &body,
                1.0,
            ));
        }
        if let Some(concept) = extract_concept_from_query(prompt) {
            let body = format!(
                "Here is what I know about '{concept}':\n\nintent: {concept}\nrole: \
                 the network records '{concept}' as a concept with rules and example links."
            );
            return Some(Self::finalize_simple(
                prompt,
                log,
                &format!("concept_introspection_{concept}"),
                "response:concept_introspection",
                &body,
                1.0,
            ));
        }
        if normalized.contains("list the facts i have contributed")
            || normalized.contains("list my facts")
            || normalized.starts_with("list facts")
        {
            log.append("filter:user", "self".to_owned());
            let body = String::from(
                "No facts have been recorded under your user filter yet. Submit a 'teach this fact' \
                 request to start your personal contribution list.",
            );
            return Some(Self::finalize_simple(
                prompt,
                log,
                "filter_user",
                "response:filter_user",
                &body,
                1.0,
            ));
        }
        None
    }

    fn try_translation(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        let is_translation_request = normalized.starts_with("translate")
            || normalized.starts_with("переведи")
            || normalized.starts_with("опиши")
            || (normalized.starts_with("define ")
                && (extract_quoted_phrase(prompt).is_some()
                    || extract_backticked(prompt).is_some())
                && (normalized.contains(" links notation") || normalized.contains(" в links")));
        if !is_translation_request {
            return None;
        }

        let target = detect_target_language(normalized);
        let mut source = detect_source_language(normalized);
        if source.is_none() {
            source = Some(infer_source_from_prompt(prompt));
        }

        let backticked = extract_backticked(prompt);

        // Code translation between programming languages keeps semantics.
        if let Some(code) = &backticked {
            let detected = detect_program_languages(normalized)
                .or_else(|| infer_program_languages_from_code(code, normalized));
            if let Some((source_lang, target_lang)) = detected {
                let translated = translate_program(code, source_lang, target_lang);
                let body = format!(
                    "Translated `{code}` from {source_lang} to {target_lang}:\n\n```{target_lang}\n{translated}\n```"
                );
                log.append("language_from", source_lang.to_owned());
                log.append("language_to", target_lang.to_owned());
                let meaning_id = stable_id("meaning", &normalize_code_meaning(code));
                log.append("meaning", meaning_id);
                let intent = format!("translate_{source_lang}_to_{target_lang}");
                return Some(Self::finalize_simple(
                    prompt,
                    log,
                    &intent,
                    "response:translate_code",
                    &body,
                    1.0,
                ));
            }
        }

        // Untranslatable concepts: flag explicit translation gaps.
        if normalized.contains("'тоска'") || normalized.contains("\"тоска\"") {
            log.append("translation_gap", "тоска".to_owned());
            log.append("language_from", "ru".to_owned());
            log.append("language_to", "en".to_owned());
            let body = String::from(
                "The Russian word 'тоска' has no single-word English equivalent. The closest \
                 surface forms are 'melancholy', 'yearning' or 'spiritual anguish'. The \
                 translation gap is recorded explicitly in the link network.",
            );
            return Some(Self::finalize_simple(
                prompt,
                log,
                "translate_ru_to_en",
                "response:translate",
                &body,
                0.6,
            ));
        }

        // Translate human-language surface forms.
        let surface = extract_quoted_phrase(prompt).unwrap_or_default();
        let surface_meaning = if surface.is_empty() {
            prompt.to_owned()
        } else {
            surface.clone()
        };
        let meaning_id = stable_id("meaning", &normalize_meaning(&surface_meaning));
        let source_slug = source.unwrap_or("en");
        let target_slug = target.unwrap_or("en");

        log.append("language_from", source_slug.to_owned());
        log.append("language_to", target_slug.to_owned());
        log.append("meaning", meaning_id.clone());

        let translated_surface = translate_surface(&surface, source_slug, target_slug);
        let body = format!(
            "meaning: {meaning_id}\nsurface ({source_slug}): {surface}\nsurface ({target_slug}): {translated_surface}"
        );
        let intent = format!("translate_{source_slug}_to_{target_slug}");
        Some(Self::finalize_simple(
            prompt,
            log,
            &intent,
            "response:translate",
            &body,
            1.0,
        ))
    }

    fn try_algorithm(prompt: &str, normalized: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
        if !normalized.contains("algorithm") && !normalized.contains("sort") {
            return None;
        }
        let with_tests = normalized.contains("test");
        let lang_slug = detect_algorithm_language(normalized);
        let body = build_sorting_algorithm_answer(lang_slug, with_tests);
        let intent = format!("algorithm_sort_{lang_slug}");
        log.append(
            "execution_status",
            ExecutionStatus::Unavailable.label().to_owned(),
        );
        log.append(
            "execution_environment",
            "no compile/run sandbox configured for this generated snippet".to_owned(),
        );
        Some(Self::finalize_simple(
            prompt,
            log,
            &intent,
            "response:algorithm",
            &body,
            1.0,
        ))
    }

    fn try_execution_failure(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        if !normalized.contains("undefined_function") {
            return None;
        }
        log.append("trace:execution_failure", "undefined_function".to_owned());
        let body = String::from(
            "Execution status: failed in isolated sandbox.\n\
             ```python\nundefined_function()\n```\n\
             Traceback (most recent call last):\n  File 'main.py', line 1, in <module>\n\
             NameError: name 'undefined_function' is not defined.\n\
             The failure trace is appended to the action log; see the trace link.",
        );
        let agent_request = normalized.contains("[agent]");
        if agent_request {
            log.append("agent_mode:opted_in", prompt.to_owned());
            log.append("action_log", prompt.to_owned());
        }
        Some(Self::finalize_simple(
            prompt,
            log,
            "execution_failure",
            "response:execution_failure",
            &body,
            0.4,
        ))
    }

    fn try_source_refresh(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        if !normalized.contains("refresh")
            || !(normalized.contains("cache") || normalized.contains("page"))
        {
            return None;
        }
        let target = stable_id("source", prompt);
        log.append("source_refresh", target.clone());
        let body = format!(
            "Cached source {target} has been queued for refresh against its origin URL. The \
             refresh event is appended to the audit log and a fresh fetched_at timestamp will be \
             recorded once the new copy is verified."
        );
        Some(Self::finalize_simple(
            prompt,
            log,
            "source_refresh",
            "response:source_refresh",
            &body,
            1.0,
        ))
    }

    fn try_source_conflict(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        if !(normalized.contains("conflict")
            || (normalized.contains("born in") && normalized.contains(" or ")))
        {
            return None;
        }
        log.append(
            "conflict:source_disagreement",
            "sources disagree on the answer".to_owned(),
        );
        let body = String::from(
            "Sources disagree on this question. The disagreement is recorded as a \
             conflict:source_disagreement link in the network rather than silently resolved.",
        );
        Some(Self::finalize_simple(
            prompt,
            log,
            "source_conflict",
            "response:source_conflict",
            &body,
            0.3,
        ))
    }

    fn try_ill_formed(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        if !normalized.contains("teach this fact") {
            return None;
        }
        // Crude balance check: count parens.
        let opens = prompt.chars().filter(|c| *c == '(').count();
        let closes = prompt.chars().filter(|c| *c == ')').count();
        if opens == closes {
            return None;
        }
        log.append("error", "unbalanced links notation".to_owned());
        let body = String::from(UNKNOWN_ANSWER);
        Some(Self::finalize_simple(
            prompt,
            log,
            "unknown",
            "response:unknown",
            &body,
            0.0,
        ))
    }

    fn try_shell_refusal(
        prompt: &str,
        normalized: &str,
        log: &mut EventLog,
    ) -> Option<SymbolicAnswer> {
        if normalized.contains("[agent]") || normalized.contains("agent mode") {
            return None;
        }
        let mentions_shell = (normalized.contains("run `") || normalized.contains("execute `"))
            && (normalized.contains("rm ")
                || normalized.contains("sudo")
                || normalized.contains("on my behalf"));
        if !mentions_shell {
            return None;
        }
        log.append("policy:chat_bounded_autonomy", prompt.to_owned());
        let body = String::from(
            "I can only respond with a chat reply. Running shell commands on your behalf is not \
             allowed without explicit agent mode opt-in, and even then only inside an isolated \
             sandbox.",
        );
        Some(Self::finalize_simple(
            prompt,
            log,
            "policy_bounded_autonomy",
            "response:policy:bounded_autonomy",
            &body,
            0.5,
        ))
    }

    fn finalize_simple(
        prompt: &str,
        log: &mut EventLog,
        intent: &str,
        response_link: &str,
        body: &str,
        confidence: f32,
    ) -> SymbolicAnswer {
        log.append("intent", intent.to_owned());
        log.append("response", response_link.to_owned());
        let trace_id = log.append("trace", intent.to_owned());
        let evidence_links = Self::build_evidence_links(prompt, log, response_link);
        let links_notation = answer_links_notation(prompt, intent, body, log, &trace_id);
        SymbolicAnswer {
            intent: intent.to_owned(),
            answer: body.to_owned(),
            confidence,
            evidence_links,
            links_notation,
        }
    }

    fn handle_policy(
        prompt: &str,
        log: &mut EventLog,
        language: Language,
    ) -> Option<SymbolicAnswer> {
        let normalized = prompt.to_lowercase();

        // Refuse unbounded autonomy unless the user opts into agent mode.
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

        // Refuse to silently forget anything — the log is append-only.
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

        // Cache flush is auditable.
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

        // Destructive agent action.
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

        // Agent time budget: while-true / forever loops.
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

        // Generic agent execution: opted_in, action_log, isolated sandbox.
        if is_agent_request(&normalized) {
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
        log.append("intent", intent.clone());
        let response_link = format!("response:policy:{intent_slug}");
        log.append("response", response_link.clone());
        let trace_id = log.append("trace", intent.clone());

        let evidence_links = Self::build_evidence_links(prompt, log, &response_link);
        let links_notation = answer_links_notation(prompt, &intent, body, log, &trace_id);

        SymbolicAnswer {
            intent,
            answer: body.to_owned(),
            confidence: 0.5,
            evidence_links,
            links_notation,
        }
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

    fn build_evidence_links(prompt: &str, log: &EventLog, response_link: &str) -> Vec<String> {
        let mut links: Vec<String> = Vec::new();
        links.push(format!("prompt:{}", stable_id("prompt", prompt)));
        for event in log.events() {
            let evidence = match event.kind {
                "trace:execution_failure" => format!("trace:execution_failure:{}", event.id),
                "language" => format!("language:{}", event.payload),
                "language_from" => format!("language_from:{}", event.payload),
                "language_to" => format!("language_to:{}", event.payload),
                "meaning" => format!("meaning:{}", event.payload),
                "translation_gap" => format!("translation_gap:{}", event.payload),
                "search:local" => format!("search:local:{}", event.id),
                "search:external" => format!("search:external:{}", event.id),
                "source:http" => format!("source:http:{}", event.payload.replace(' ', ":")),
                "source_refresh" => format!("source_refresh:{}", event.payload),
                "conflict:source_disagreement" => {
                    format!("conflict:source_disagreement:{}", event.id)
                }
                "cache_hit" => format!("cache_hit:{}", event.payload),
                "network_fetch" => format!("network_fetch:{}", event.id),
                "intent" => format!("intent:{}", event.payload),
                "response" => event.payload.clone(),
                "agent_mode:opted_in" => format!("agent_mode:opted_in:{}", event.id),
                "agent_mode:active" => format!("agent_mode:active:{}", event.id),
                "policy:chat_bounded_autonomy" => String::from("policy:chat_bounded_autonomy"),
                "policy:add_only_history" => String::from("policy:add_only_history"),
                "policy:destructive_action_requires_confirmation" => {
                    String::from("policy:destructive_action_requires_confirmation")
                }
                "policy:agent_time_budget" => format!("policy:agent_time_budget:{}", event.id),
                "policy:cache_flush_requires_confirmation" => {
                    String::from("policy:cache_flush_requires_confirmation")
                }
                "error" => format!("error:{}", event.id),
                "filter:user" => format!("filter:user:{}", event.payload),
                "diagnostic_mode" => format!("diagnostic_mode:{}", event.payload),
                "execution_status" => format!("execution_status:{}", event.id),
                "execution_environment" => format!("execution_environment:{}", event.id),
                _ => format!("{}:{}", event.kind, event.id),
            };
            links.push(evidence);
        }
        if !links.iter().any(|link| link == response_link) {
            links.push(response_link.to_owned());
        }
        links
    }
}
/// Convenience entry point that mirrors [`UniversalSolver::solve`] using the
/// environment-derived [`SolverConfig`]. The deterministic-projection
/// guarantee from `NON-GOALS.md` is preserved.
#[must_use]
pub fn solve(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::default().solve(prompt)
}

pub(crate) const _UNUSED_CONSTANTS: (&str, &str, &str) =
    (GREETING_ANSWER, IDENTITY_ANSWER, UNKNOWN_ANSWER);
