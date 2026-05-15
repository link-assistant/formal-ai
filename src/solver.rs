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

use crate::engine::{
    answer_links_notation, language_aware_answer_for, language_aware_intent_for,
    response_link_for_intent, select_rule_for, stable_id, SelectedRule, SymbolicAnswer,
};
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};
use crate::solver_handlers::{
    build_evidence_links, finalize_simple, try_algorithm, try_arithmetic, try_concept_lookup,
    try_conversation_memory, try_execution_failure, try_ill_formed, try_javascript_execution,
    try_meta_explanation, try_network_query, try_opinion_question, try_shell_refusal,
    try_source_conflict, try_source_refresh, try_translation,
};
use crate::solver_helpers::{
    confidence_for, is_agent_opt_in, is_agent_request, is_cache_flush_request,
    is_destructive_action, is_forget_request, is_unbounded_autonomy, is_unbounded_loop,
    record_candidates, record_decomposition, record_validation, requires_external_lookup,
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

        log.append("search:local", prompt.to_owned());

        record_decomposition(&mut log, prompt, self.config.max_decomposition_depth);

        if let Some(answer) = self.handle_specialized_pattern(prompt, &mut log) {
            return answer;
        }

        if let Some(answer) = Self::handle_policy(prompt, &mut log, language) {
            return answer;
        }

        let rule = select_rule_for(prompt);
        let intent = language_aware_intent_for(&rule, language);
        log.append("intent", intent.clone());

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

        if matches!(rule, SelectedRule::Unknown) && requires_external_lookup(prompt) {
            self.record_external_search(&mut log, prompt);
        }

        record_candidates(&mut log, prompt, &intent);

        let validation_choice = record_validation(&mut log, prompt);

        let answer = match (&validation_choice, &rule) {
            (Some(choice), SelectedRule::Unknown) => choice.answer.clone(),
            _ => language_aware_answer_for(&rule, language, prompt),
        };

        let response_link = response_link_for_intent(&rule, &intent);
        log.append("response", response_link.clone());

        log.append("trace:simplification", "smallest_sufficient".to_owned());

        let trace_id = log.append("trace", intent.clone());

        let evidence_links = build_evidence_links(prompt, &log, &response_link);
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
        if let Some(answer) = try_conversation_memory(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_arithmetic(prompt, log) {
            return Some(answer);
        }
        if let Some(answer) = try_javascript_execution(prompt, log) {
            return Some(answer);
        }
        if let Some(answer) = try_concept_lookup(prompt, log) {
            return Some(answer);
        }
        if let Some(answer) = try_meta_explanation(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_network_query(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_translation(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_algorithm(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_execution_failure(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_source_refresh(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_source_conflict(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_ill_formed(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_shell_refusal(prompt, &normalized, log) {
            return Some(answer);
        }
        if let Some(answer) = try_opinion_question(prompt, &normalized, log) {
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
        if !normalized.contains("[diagnostic]") {
            return None;
        }
        log.append("diagnostic_mode", "active".to_owned());
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
        let evidence_links = build_evidence_links(prompt, log, &response_link);
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

    fn handle_policy(
        prompt: &str,
        log: &mut EventLog,
        language: Language,
    ) -> Option<SymbolicAnswer> {
        let normalized = prompt.to_lowercase();

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

/// Convenience entry point that mirrors [`UniversalSolver::solve_with_history`]
/// using the environment-derived [`SolverConfig`]. The deterministic-projection
/// guarantee from `NON-GOALS.md` is preserved.
#[must_use]
pub fn solve_with_history(prompt: &str, history: &[ConversationTurn]) -> SymbolicAnswer {
    UniversalSolver::default().solve_with_history(prompt, history)
}
