//! Append-only event log for the universal solver.
//!
//! Every step the solver takes is recorded as a content-addressed event
//! before the user-facing answer is built. The answer is then a projection
//! of the log — see `VISION.md` and `GOALS.md` for the rationale.
//!
//! The log is intentionally small: it lives in-process, holds plain Rust
//! records, and uses the same FNV-1a 64-bit hash that `engine::stable_id`
//! uses so identifiers stay stable across surfaces.

use crate::engine::{stable_id, ThinkingStep};
use crate::link_store::{LinkStore, LinkStoreError};
use crate::memory::MemoryEvent;

/// A single event in the append-only log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: String,
    pub kind: &'static str,
    pub payload: String,
}

/// In-process append-only event log.
#[derive(Debug, Default, Clone)]
pub struct EventLog {
    events: Vec<Event>,
}

impl EventLog {
    #[must_use]
    pub const fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Append a new event with content-addressed id.
    ///
    /// The id is derived from the kind, the payload, and the current log
    /// length so that repeated events produce stable, distinct ids.
    pub fn append(&mut self, kind: &'static str, payload: impl Into<String>) -> String {
        let payload = payload.into();
        let seed = format!("{kind}:{}:{payload}", self.events.len());
        let id = stable_id(kind, &seed);
        self.events.push(Event {
            id: id.clone(),
            kind,
            payload,
        });
        id
    }

    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Returns the first event of the given kind, if any.
    #[must_use]
    pub fn first_of(&self, kind: &str) -> Option<&Event> {
        self.events.iter().find(|event| event.kind == kind)
    }

    /// Returns the most recent event of the given kind, if any.
    #[must_use]
    pub fn last_of(&self, kind: &str) -> Option<&Event> {
        self.events.iter().rev().find(|event| event.kind == kind)
    }

    /// Project the log to a list of `<kind>:<id>` links for the user-facing
    /// evidence array. Each link points back to a distinct event.
    #[must_use]
    pub fn evidence_links(&self) -> Vec<String> {
        self.events
            .iter()
            .map(|event| format!("{}:{}", event.kind, event.id))
            .collect()
    }

    /// Build a Links Notation `steps` block listing every event in order.
    /// Used by trace serialization in [`crate::solver`].
    #[must_use]
    pub fn steps_block(&self) -> String {
        use std::fmt::Write as _;
        let mut buffer = String::from("steps:");
        for (index, event) in self.events.iter().enumerate() {
            let _ = write!(
                buffer,
                "\n  step_{index} {} {}",
                event.kind,
                sanitize_payload(&event.payload)
            );
        }
        buffer
    }

    /// Project raw solver events into user-readable, ordered thinking steps.
    ///
    /// This is the *curated* projection introduced for issue #488. Instead of a
    /// noisy 1:1 dump of every internal event, it keeps only the meaningful
    /// reasoning milestones, cleans each `detail` down to concrete content,
    /// folds the calculator's reduction trace into one composite `compute` step
    /// with detailed children, de-duplicates consecutive repeats, and labels the
    /// universal-algorithm phases `high` so the minimum granularity surfaces only
    /// the high-level direction.
    ///
    /// The raw, lossless trace stays available to maintainers through
    /// [`EventLog::steps_block`] and [`EventLog::evidence_links`].
    #[must_use]
    pub fn thinking_steps(&self) -> Vec<ThinkingStep> {
        self.thinking_steps_for_answer("")
    }

    /// Like [`EventLog::thinking_steps`], but substitutes the actual composed
    /// answer for the opaque `response:<route>` payload of the closing
    /// `deformalize` step so the final reasoning step is concrete (issue #488).
    #[must_use]
    pub fn thinking_steps_for_answer(&self, final_answer: &str) -> Vec<ThinkingStep> {
        let answer = collapse_thinking_whitespace(final_answer);

        // Phase 1 — curate raw events into a clean plan, folding the
        // calculator's reduction trace into one composite parent + children.
        let mut plan: Vec<PlannedThinkingStep> = Vec::new();
        let mut calculation = CalculationCluster::default();
        for event in &self.events {
            if event.kind == "calculation" || event.kind.starts_with("calculation:") {
                calculation.absorb(event.kind, &event.payload);
                continue;
            }
            if calculation.has_data() {
                calculation.drain_into(&mut plan);
            }
            if let Some(step) = curate_thinking_event(event.kind, &event.payload, &answer) {
                plan.push(step);
            }
        }
        if calculation.has_data() {
            calculation.drain_into(&mut plan);
        }

        // Phase 2 — de-duplicate consecutive repeats, assign order and parent
        // ids (so composite steps render as recursively nested thinking).
        let mut steps: Vec<ThinkingStep> = Vec::new();
        let mut order: u32 = 0;
        let mut previous_key: Option<String> = None;
        let mut current_parent: Option<String> = None;
        for planned in plan {
            let key = format!("{}\u{1f}{}", planned.step, planned.detail);
            let is_child = planned.role == StepRole::Child;
            if !is_child && previous_key.as_deref() == Some(key.as_str()) {
                continue;
            }
            previous_key = Some(key);
            let mut step = ThinkingStep::new(
                order,
                planned.step,
                planned.detail,
                planned.level,
                planned.source,
            );
            match planned.role {
                StepRole::Parent => current_parent = Some(step.id.clone()),
                StepRole::Child => {
                    if let Some(parent) = current_parent.clone() {
                        step = step.with_parent(parent);
                    }
                }
                StepRole::Normal => current_parent = None,
            }
            steps.push(step);
            order += 1;
        }
        steps
    }

    /// Replay every in-process event into a durable link store projection.
    ///
    /// This is additive: the original event log remains in-process, while
    /// the target store receives memory records that can be exported as
    /// `.lino` or reduced to doublets by the active backend.
    pub fn append_to_link_store<S: LinkStore>(
        &self,
        store: &mut S,
    ) -> Result<usize, LinkStoreError> {
        for event in &self.events {
            store.append_memory_event(MemoryEvent {
                id: event.id.clone(),
                kind: Some(event.kind.to_owned()),
                content: Some(event.payload.clone()),
                evidence: vec![format!("{}:{}", event.kind, event.id)],
                ..MemoryEvent::default()
            })?;
        }
        Ok(self.events.len())
    }
}

/// Role of a planned step in a recursively-composite (fractal) thinking tree.
#[derive(Clone, Copy, PartialEq, Eq)]
enum StepRole {
    /// A leaf step at the top level.
    Normal,
    /// A composite step that owns the following [`StepRole::Child`] steps.
    Parent,
    /// A sub-step nested under the most recent [`StepRole::Parent`].
    Child,
}

/// A curated thinking step before ordering, de-duplication and id assignment.
struct PlannedThinkingStep {
    step: &'static str,
    detail: String,
    level: &'static str,
    source: &'static str,
    role: StepRole,
}

impl PlannedThinkingStep {
    fn normal(
        step: &'static str,
        detail: impl Into<String>,
        level: &'static str,
        source: &'static str,
    ) -> Self {
        Self {
            step,
            detail: detail.into(),
            level,
            source,
            role: StepRole::Normal,
        }
    }

    fn parent(
        step: &'static str,
        detail: impl Into<String>,
        level: &'static str,
        source: &'static str,
    ) -> Self {
        Self {
            role: StepRole::Parent,
            ..Self::normal(step, detail, level, source)
        }
    }

    fn child(
        step: &'static str,
        detail: impl Into<String>,
        level: &'static str,
        source: &'static str,
    ) -> Self {
        Self {
            role: StepRole::Child,
            ..Self::normal(step, detail, level, source)
        }
    }
}

/// Accumulates the calculator's `calculation:*` trace so it can be rendered as a
/// single composite `compute` step (parent) with detailed children (issue #488,
/// requirement R11 — recursively composite / fractal steps).
#[derive(Default)]
struct CalculationCluster {
    request: Option<String>,
    engine: Option<String>,
    expression: Option<String>,
    steps: Option<String>,
    result: Option<String>,
}

impl CalculationCluster {
    fn absorb(&mut self, kind: &str, payload: &str) {
        let value = payload.trim().to_owned();
        match kind {
            "calculation" => self.result = Some(value),
            "calculation:request" => self.request = Some(value),
            "calculation:engine" => self.engine = Some(value),
            "calculation:lino" => self.expression = Some(value),
            "calculation:steps" => self.steps = Some(value),
            // `calculation:rate_basis` and any other detail kinds stay in the raw
            // evidence trace but are not surfaced in the curated view.
            _ => {}
        }
    }

    const fn has_data(&self) -> bool {
        self.result.is_some()
            || self.request.is_some()
            || self.engine.is_some()
            || self.expression.is_some()
            || self.steps.is_some()
    }

    fn drain_into(&mut self, plan: &mut Vec<PlannedThinkingStep>) {
        let parent_detail = self
            .result
            .clone()
            .or_else(|| self.request.clone())
            .unwrap_or_default();
        plan.push(PlannedThinkingStep::parent(
            "compute",
            parent_detail,
            "high",
            "calculation",
        ));
        if let Some(engine) = self.engine.take() {
            plan.push(PlannedThinkingStep::child(
                "compute_engine",
                engine,
                "detailed",
                "calculation:engine",
            ));
        }
        if let Some(expression) = self.expression.take() {
            plan.push(PlannedThinkingStep::child(
                "compute_expression",
                expression,
                "detailed",
                "calculation:lino",
            ));
        }
        if let Some(steps) = self.steps.take() {
            plan.push(PlannedThinkingStep::child(
                "compute_steps",
                steps,
                "detailed",
                "calculation:steps",
            ));
        }
        *self = Self::default();
    }
}

/// Collapse any run of whitespace (including newlines) into single spaces so a
/// multi-line answer renders as one concrete `deformalize` detail.
fn collapse_thinking_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Curate a single raw solver event into a clean, concrete thinking step, or
/// `None` when the event is internal bookkeeping that should not surface to the
/// user.
///
/// This is an **allowlist**: only events that carry meaningful reasoning are
/// kept, and each is reduced to concrete content (a search term, a route, a
/// looked-up entity, the composed answer). Unknown kinds are dropped so the
/// default view stays concrete — the lossless trace remains in the evidence
/// links for maintainers.
fn curate_thinking_event(
    kind: &'static str,
    payload: &str,
    final_answer: &str,
) -> Option<PlannedThinkingStep> {
    let detail = payload.trim();
    let keep = |step: &'static str, detail: &str, level: &'static str| {
        Some(PlannedThinkingStep::normal(
            step,
            detail.to_owned(),
            level,
            kind,
        ))
    };
    match kind {
        // ---- Universal-algorithm phases (high level / minimum granularity) ----
        "impulse" => keep("impulse", detail, "high"),
        "language" | "language_from" => keep("detect_language", detail, "high"),
        "language_to" => keep("resolve_response_language", detail, "high"),
        "intent_formalization:route" => keep("formalize", detail, "high"),
        "intent" | "specialized_handler" | "legacy_intent" => {
            keep("dispatch_handler", detail, "high")
        }
        "program_plan" | "program_parameters" => keep("program_plan", detail, "high"),
        "response" => {
            let value = if final_answer.is_empty() {
                detail
            } else {
                final_answer
            };
            keep("deformalize", value, "high")
        }

        // ---- Concrete detailed reasoning ----
        "concept_lookup:request" | "procedural_how_to:request" => {
            keep("scan_memory", detail, "detailed")
        }
        "concept_lookup:hit" | "fact_query:relation" | "fact_query:subject" | "fact_lookup:hit" => {
            keep("lookup_fact", detail, "detailed")
        }
        "web_search:request" | "http_fetch:request" | "url_navigate:request" => {
            keep("http_chat", detail, "detailed")
        }
        "tool_call" | "tool_result" => keep("invoke_tool", detail, "detailed"),
        "coreference" | "program_coreference" => keep("coreference_binding", "", "detailed"),
        "modifier_detection" | "program_modifiers" => keep("modifier_detection", "", "detailed"),
        "rule_construction" => keep("rule_construction", "", "detailed"),
        // The validation payload is an internal acceptance reason
        // (`accepted_without_extra_constraints`); surface a clean verification
        // step without the meta-language token.
        "validation" => keep("rule_verification", "", "detailed"),
        _ if kind.starts_with("tool_") => keep("invoke_tool", detail, "detailed"),
        _ if kind.starts_with("agent_mode") || kind == "action_log" => {
            keep("agent_plan", detail, "detailed")
        }

        // ---- Everything else: internal bookkeeping, dropped from the view ----
        _ => None,
    }
}

/// Build the evidence links array for a symbolic answer.
///
/// Translates each log event into a typed link, appending `response_link` at
/// the end when it is not already present.
#[must_use]
pub fn build_evidence_links(prompt: &str, log: &EventLog, response_link: &str) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    links.push(format!("prompt:{}", stable_id("prompt", prompt)));
    for event in log.events() {
        let evidence = match event.kind {
            "trace:execution_failure" => format!("trace:execution_failure:{}", event.id),
            "language" => format!("language:{}", event.payload),
            "language_from" => format!("language_from:{}", event.payload),
            "language_to" => format!("language_to:{}", event.payload),
            "definition_merge:language" => format!("definition_merge:language:{}", event.payload),
            "meaning" => format!("meaning:{}", event.payload),
            "translation_gap" => format!("translation_gap:{}", event.payload),
            "wikidata" => format!("wikidata:{}", event.payload),
            "formalization" => format!("formalization:{}", event.id),
            "formalization:subject_q" => {
                format!("formalization:subject_q:{}", event.payload)
            }
            "formalization:predicate_p" => {
                format!("formalization:predicate_p:{}", event.payload)
            }
            "formalization:object_q" => {
                format!("formalization:object_q:{}", event.payload)
            }
            "formalization:item_q" => format!("formalization:item_q:{}", event.payload),
            "formalization:property_p" => {
                format!("formalization:property_p:{}", event.payload)
            }
            "formalization:fallback" => {
                format!("formalization:fallback:{}", event.payload)
            }
            "formalization:raw" => format!("formalization:raw:{}", event.payload),
            "formalization_unresolved" => {
                format!("formalization_unresolved:{}", event.payload)
            }
            "intent_formalization" => format!("intent_formalization:{}", event.id),
            "intent_formalization_cache" => {
                format!(
                    "intent_formalization_cache:{}",
                    event.payload.replace(' ', ":")
                )
            }
            "intent_formalization:kind" => {
                format!("intent_formalization:kind:{}", event.payload)
            }
            "intent_formalization:route" => {
                format!("intent_formalization:route:{}", event.payload)
            }
            "intent_formalization:relevant" => {
                format!("intent_formalization:relevant:{}", event.payload)
            }
            // Structured fact_query trace events (Issue #127): preserve the
            // payload verbatim so memory consumers can render the parsed
            // relation, subject, and cache decision (rather than a hash id).
            "fact_query:request" => format!("fact_query:request:{}", event.id),
            "fact_query:relation" => format!("fact_query:relation:{}", event.payload),
            "fact_query:subject" => format!("fact_query:subject:{}", event.payload),
            "fact_query:cache:hit" => format!("fact_query:cache:hit:{}", event.payload),
            "fact_query:cache:miss" => String::from("fact_query:cache:miss"),
            "fact_query:cache:bypass" => String::from("fact_query:cache:bypass"),
            "fact_query:force_fresh" => String::from("fact_query:force_fresh"),
            "fact_query:subject_qid" => format!("fact_query:subject_qid:{}", event.payload),
            "fact_query:value_qid" => format!("fact_query:value_qid:{}", event.payload),
            // Structured web_search trace events (Issue #133): record the
            // request, the providers considered (DuckDuckGo first, plus
            // CORS-readable knowledge bases), per-provider ranks, and the
            // combined-ranking strategy so memory consumers can reconstruct
            // the multi-engine reasoning offline.
            "web_search:request" => format!("web_search:request:{}", event.payload),
            "web_search:query_kind" => format!("web_search:query_kind:{}", event.payload),
            "web_search:provider" => format!("web_search:provider:{}", event.payload),
            "web_search:language" => format!("web_search:language:{}", event.payload),
            "web_search:combined" => format!("web_search:combined:{}", event.payload),
            "web_search:rank" => format!("web_search:rank:{}", event.payload),
            "web_search:fused" => format!("web_search:fused:{}", event.payload),
            "web_search:disabled" => format!("web_search:disabled:{}", event.payload),
            "http_fetch:request" => format!("http_fetch:request:{}", event.payload),
            "docs_method:request" => format!("docs_method:request:{}", event.id),
            "docs_method:project" => format!("docs_method:project:{}", event.payload),
            "docs_method:method" => format!("docs_method:method:{}", event.payload),
            "docs_method:source_kind" => {
                format!("docs_method:source_kind:{}", event.payload)
            }
            "docs_method:source" => format!("source:{}", event.payload),
            "project:promoted" => format!("project:promoted:{}", event.payload),
            "project_lookup:promotion" => {
                format!("project_lookup:promotion:{}", event.payload)
            }
            "project_lookup:repository:github" => {
                format!("project_lookup:repository:github:{}", event.payload)
            }
            "project_lookup:repository:gitlab" => {
                format!("project_lookup:repository:gitlab:{}", event.payload)
            }
            "project_lookup:repository:bitbucket" => {
                format!("project_lookup:repository:bitbucket:{}", event.payload)
            }
            "url_navigate:request" => format!("url_navigate:request:{}", event.payload),
            "url_preview:iframe" => format!("url_preview:iframe:{}", event.payload),
            "tool_call" => format!("tool_call:{}", event.payload),
            "tool_parameter" => format!("tool_parameter:{}", event.payload.replace(' ', ":")),
            "tool_result" => format!("tool_result:{}", event.payload.replace(' ', ":")),
            "tool_permission" => {
                format!("tool_permission:{}", event.payload.replace(' ', ":"))
            }
            "text_operation" => format!("text_operation:{}", event.payload),
            "text_rule" => format!("text_rule:{}", event.payload),
            "text_rule_chain" => format!("text_rule_chain:{}", event.payload),
            "text_result" => format!("text_result:{}", event.id),
            "text_substitution_rules" => format!("text_substitution_rules:{}", event.id),
            "text_substitution_trace" => format!("text_substitution_trace:{}", event.id),
            "text_substitution_graph" => format!("text_substitution_graph:{}", event.id),
            "procedural_how_to:request" => {
                format!("procedural_how_to:request:{}", event.payload)
            }
            "procedural_how_to:action" => {
                format!("procedural_how_to:action:{}", event.payload)
            }
            "procedural_how_to:object" => {
                format!("procedural_how_to:object:{}", event.payload)
            }
            "procedural_how_to:stage" => {
                format!("procedural_how_to:stage:{}", event.payload)
            }
            "procedural_how_to:wikihow_candidate" => {
                format!("procedural_how_to:wikihow_candidate:{}", event.payload)
            }
            "procedural_how_to:source_gate" => {
                format!("procedural_how_to:source_gate:{}", event.payload)
            }
            "spelling_correction" => {
                format!("spelling_correction:{}", event.payload.replace(' ', ""))
            }
            "concept_lookup:request" => format!("concept_lookup:request:{}", event.payload),
            "concept_lookup:context" => format!("concept_lookup:context:{}", event.payload),
            "concept_lookup:response-language" => {
                format!("concept_lookup:response-language:{}", event.payload)
            }
            "concept_lookup:hit" => format!("concept_lookup:hit:{}", event.payload),
            "concept_lookup:miss" => format!("concept_lookup:miss:{}", event.payload),
            "concept_lookup:context-match" => {
                format!("concept_lookup:context-match:{}", event.payload)
            }
            "concept_lookup:context-mismatch" => {
                format!("concept_lookup:context-mismatch:{}", event.payload)
            }
            "followup:subject" => format!("followup:subject:{}", event.payload),
            "mechanism_query:request" => {
                format!("mechanism_query:request:{}", event.payload)
            }
            "mechanism_query:stage" => format!("mechanism_query:stage:{}", event.payload),
            "mechanism_query:source_gate" => {
                format!("mechanism_query:source_gate:{}", event.payload)
            }
            "search:local" => format!("search:local:{}", event.id),
            "search:external" if event.payload == "skipped:offline" => {
                String::from("policy:offline")
            }
            "search:external" => format!("search:external:{}", event.id),
            "source:http" => format!("source:http:{}", event.payload.replace(' ', ":")),
            "source_refresh" => format!("source_refresh:{}", event.payload),
            "skill_compile:package" => format!("skill_compile:package:{}", event.payload),
            "compiled_skill:package" => format!("compiled_skill:package:{}", event.id),
            "compiled_skill:replay" => format!("compiled_skill:replay:{}", event.payload),
            "conflict:source_disagreement" => {
                format!("conflict:source_disagreement:{}", event.id)
            }
            "cache_hit" => format!("cache_hit:{}", event.payload),
            "network_fetch" => format!("network_fetch:{}", event.id),
            "calculation:engine" => format!("calculation:engine:{}", event.payload),
            "calculation:lino" => format!("calculation:lino:{}", event.payload),
            "intent" => format!("intent:{}", event.payload),
            "program_parameter:language" => {
                format!("program_parameter:language:{}", event.payload)
            }
            "program_parameter:task" => format!("program_parameter:task:{}", event.payload),
            "program_parameters" => {
                format!(
                    "program_parameters:{}",
                    event.payload.replace([' ', ','], ":")
                )
            }
            "legacy_intent" => format!("legacy_intent:{}", event.payload),
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
            "policy:offline" => String::from("policy:offline"),
            "policy:inappropriate_content" => String::from("policy:inappropriate_content"),
            "policy:agent_mode_required_for_tools" => {
                format!("policy:agent_mode_required_for_tools:{}", event.payload)
            }
            "policy:package_permission_required" => {
                format!("policy:package_permission_required:{}", event.payload)
            }
            "policy:temperature_selection" => {
                format!("policy:temperature_selection:{}", event.id)
            }
            "policy:guessed_under_ambiguity" => String::from("policy:guessed_under_ambiguity"),
            "policy:clarify_under_ambiguity" => String::from("policy:clarify_under_ambiguity"),
            "probability:evidence" => format!("probability:evidence:{}", event.id),
            "probability:model" => format!("probability:model:{}", event.payload),
            "probability:ranking" => format!("probability:ranking:{}", event.id),
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

fn sanitize_payload(value: &str) -> String {
    value
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

#[path = "source_tests/event_log/tests.rs"]
mod tests;
