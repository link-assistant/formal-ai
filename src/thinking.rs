//! Human-readable "thinking" steps (issue #488).
//!
//! This module owns the [`ThinkingStep`] model and the deterministic
//! `(step, detail) -> concrete English sentence` naturalizer that every native
//! surface shares: the core `EventLog` projection, the CLI `--thinking` output,
//! the OpenAI/Anthropic API responses, and the Telegram bot. The browser mirrors
//! the same two stages in JavaScript — the curated projection in
//! `src/web/formal_ai_worker.js` and the naturalizer (`naturalizeThinkingStep`,
//! which additionally localizes into the user's language) in `src/web/app.js`.
//! Keeping it in its own module (rather than inside `engine.rs`) keeps each file
//! focused and within the repository's per-file line budget while making
//! "thinking" a first-class concern of the architecture rather than an engine
//! implementation detail.

use serde::{Deserialize, Serialize};

use crate::engine::stable_id;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingStep {
    pub id: String,
    pub order: u32,
    pub step: String,
    pub detail: String,
    /// Concrete, human-readable description of this step (issue #488).
    ///
    /// This is the "meta-language description" layer: a single English sentence
    /// that surfaces the actual content of the step (the prompt, the computed
    /// result, the looked-up entity, the chosen route, the composed answer)
    /// rather than a generic category label. UI surfaces translate it into the
    /// target user language; non-UI surfaces (CLI, API, Telegram) show it as-is.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    pub level: String,
    pub source_event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

impl ThinkingStep {
    #[must_use]
    pub fn new(
        order: u32,
        step: impl Into<String>,
        detail: impl Into<String>,
        level: impl Into<String>,
        source_event: impl Into<String>,
    ) -> Self {
        let step = step.into();
        let detail = detail.into();
        let level = level.into();
        let source_event = source_event.into();
        let summary = naturalize_thinking_step(&step, &detail);
        let seed = format!("{order}:{step}:{detail}:{level}:{source_event}");
        Self {
            id: stable_id("thinking_step", &seed),
            order,
            step,
            detail,
            summary,
            level,
            source_event,
            parent_id: None,
        }
    }

    /// Attach a parent step id so callers can express recursively composite
    /// (fractal) thinking, where finer-granularity sub-steps roll up into a
    /// single high-level step (issue #488).
    #[must_use]
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }
}

/// Map a language slug to its English name for concrete thinking summaries.
#[must_use]
pub fn thinking_language_label(code: &str) -> String {
    let normalized = code.trim().to_ascii_lowercase();
    let primary = normalized
        .split(['-', '_'])
        .next()
        .unwrap_or(normalized.as_str());
    match primary {
        "en" => "English".to_owned(),
        "ru" => "Russian".to_owned(),
        "hi" => "Hindi".to_owned(),
        "zh" => "Chinese".to_owned(),
        "" | "unknown" => "an unrecognized language".to_owned(),
        other => other.to_owned(),
    }
}

/// Turn a meta-language identifier (`write_program`, `route:greeting`,
/// `concept_lookup:hit`) into a lowercase human phrase (`write program`,
/// `greeting`, `concept lookup hit`).
#[must_use]
pub fn humanize_meta_identifier(value: &str) -> String {
    let mut spaced = String::with_capacity(value.len());
    let mut previous_lower = false;
    for character in value.chars() {
        if character.is_ascii_uppercase() && previous_lower {
            spaced.push(' ');
        }
        if matches!(character, '_' | ':' | '.' | '-' | '/') {
            spaced.push(' ');
        } else {
            spaced.push(character);
        }
        previous_lower = character.is_ascii_lowercase() || character.is_ascii_digit();
    }
    let collapsed = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim().to_ascii_lowercase()
}

/// Build a short, first-person narrative headline that says, in plain language,
/// what the assistant understood and decided (issue #676, R8).
///
/// This is the deliberately *human* layer that sits above the concrete step
/// sentences. The reporter reached Formal AI through an agentic CLI (`OpenCode`)
/// that renders the API `reasoning` field verbatim, and that field read as a
/// robotic list of category steps identical across unrelated prompts. The
/// headline restores a human summary while the detailed steps below stay intact
/// as the recursive "robotic detail" layer (high-level thought → its sub-steps).
///
/// The narrative is derived purely from the steps already present — the chosen
/// route from `dispatch_handler` (falling back to `formalize`) — so it stays
/// deterministic and needs no extra engine state. Returns `None` when the trace
/// has no recognizable route, leaving the robotic steps to speak for themselves.
///
/// The English sentences are `src` template literals, exactly like the
/// [`naturalize_thinking_step`] sentences below; the browser mirrors the same
/// per-intent framing through its localized "plain" thinking variants.
#[must_use]
pub fn thinking_narrative(steps: &[ThinkingStep]) -> Option<String> {
    let route_detail = steps
        .iter()
        .find(|step| strip_agent_substep_prefix(&step.step) == "dispatch_handler")
        .or_else(|| {
            steps
                .iter()
                .find(|step| strip_agent_substep_prefix(&step.step) == "formalize")
        })
        .map(|step| step.detail.trim().to_ascii_lowercase())?;
    if route_detail.is_empty() {
        return None;
    }
    let narrative = match route_detail.as_str() {
        "greeting" => "You said hello, so I greeted you back.",
        "wellbeing" => "You asked how I'm doing, so I told you and offered to help.",
        "assistant_free_time" => {
            "You asked what I get up to, so I answered in a friendly way and offered to help."
        }
        "farewell" => "You said goodbye, so I wished you well in return.",
        "gratitude" | "thanks" | "courtesy_response" | "courtesy" => {
            "You thanked me, so I acknowledged it warmly."
        }
        "identity" | "assistant_name" | "recall_name" | "naming" | "assistant_naming" => {
            "You asked about my name or who I am, so I answered from what I remember of our chat."
        }
        "calculation" | "arithmetic" => {
            "This was a calculation, so I worked it out step by step and checked the result."
        }
        "fact_lookup" | "concept_lookup" | "concept_lookup_in_context" => {
            "You asked for a fact, so I looked it up and reported what I found."
        }
        "translation" => "You asked for a translation, so I converted the text and returned it.",
        "web_search" | "http_fetch" | "url_navigate" => {
            "You pointed me at the web, so I fetched what you needed and summarized it."
        }
        "write_program"
        | "software_project_plan"
        | "software_project_implementation"
        | "algorithm" => "You asked for code, so I planned it and wrote the program.",
        "test_status" => "You asked about the tests, so I checked their status and reported it.",
        "self_healing" | "self_heal" => {
            "You asked me to fix myself, so I diagnosed the failure and repaired it."
        }
        "meta_explanation" => "You asked how I work, so I walked through my reasoning.",
        "learn_from_source" => {
            "You gave me something to learn from, so I read it and updated what I know."
        }
        "clarification" => "The request could mean more than one thing, so I asked you to clarify.",
        "unknown" | "fallback" => {
            "I wasn't sure how to handle this one yet, so I explained what I can do."
        }
        other => {
            // Any other resolved route still gets a human headline rather than a
            // bare category label: describe it as the task it was read as.
            let task = humanize_meta_identifier(other);
            return Some(format!(
                "I read this as {} {task} request, worked out the answer, and replied.",
                indefinite_article(&task)
            ));
        }
    };
    Some(narrative.to_owned())
}

/// Render a full reasoning trace as the plain-text form expected by protocol
/// surfaces that expose a single thinking/reasoning string.
///
/// The output leads with a human [`thinking_narrative`] headline (issue #676,
/// R8) when the route is recognizable, followed by the concrete per-step
/// sentences as the recursive "robotic detail" beneath it.
#[must_use]
pub fn render_thinking_steps(steps: &[ThinkingStep]) -> String {
    let mut lines = Vec::with_capacity(steps.len() + 1);
    if let Some(narrative) = thinking_narrative(steps) {
        lines.push(narrative);
    }
    for step in steps {
        let sentence = if step.summary.is_empty() {
            naturalize_thinking_step(&step.step, &step.detail)
        } else {
            step.summary.clone()
        };
        if step.parent_id.is_some() {
            lines.push(format!("  ↳ {sentence}"));
        } else {
            lines.push(sentence);
        }
    }
    lines.join("\n")
}

/// Pick the English indefinite article (`a`/`an`) for the following phrase based
/// on its first letter, so naturalized steps read grammatically ("an arithmetic
/// task", "a greeting task").
fn indefinite_article(phrase: &str) -> &'static str {
    match phrase.trim_start().chars().next() {
        Some(first) if matches!(first.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

/// Strip an `agent_<n>_` sub-agent prefix from a step kind, mirroring the
/// browser worker's nested-agent naming (`agent_0_impulse` -> `impulse`).
fn strip_agent_substep_prefix(step: &str) -> &str {
    if let Some(rest) = step.strip_prefix("agent_") {
        if let Some(index) = rest.find('_') {
            if index > 0 && rest[..index].bytes().all(|b| b.is_ascii_digit()) {
                return &rest[index + 1..];
            }
        }
    }
    step
}

fn truncate_thinking_detail(value: &str) -> String {
    let trimmed = value.trim();
    // Issue #1963 (problem 2, "Thinking steps are not fully written, some parts
    // are omitted."): the previous 120-char cap clipped the concrete detail of a
    // step mid-sentence (e.g. a pasted prompt or a composed answer), so the
    // visible reasoning read as truncated rather than complete. 600 chars keeps
    // the detail bounded (the panel still scrolls and the fade still applies)
    // while letting realistic single-step content render in full. This mirrors
    // the JS `thinkingDetailText` helper; keep both constants in sync.
    let limit = 600;
    if trimmed.chars().count() <= limit {
        return trimmed.to_owned();
    }
    let truncated: String = trimmed.chars().take(limit - 1).collect();
    format!("{}…", truncated.trim_end())
}

/// Translate a single `(step, detail)` pair into one concrete English sentence.
///
/// This is the deterministic "meta-language description" stage from issue #488.
/// It is the single source of truth shared by the core projection, the CLI, the
/// OpenAI/Anthropic API surfaces, and (mirrored) the browser worker, so every
/// surface renders the *same* concrete thinking rather than a generic label.
#[must_use]
pub fn naturalize_thinking_step(step: &str, detail: &str) -> String {
    let canonical = strip_agent_substep_prefix(step);
    let trimmed = truncate_thinking_detail(detail);
    let has_detail = !trimmed.is_empty();
    match canonical {
        "impulse" => {
            if has_detail {
                format!("Read the request: \"{trimmed}\".")
            } else {
                "Read the incoming request.".to_owned()
            }
        }
        "detect_language" => {
            format!(
                "Detect the request language: {}.",
                thinking_language_label(detail)
            )
        }
        "resolve_response_language" => {
            format!("Plan to answer in {}.", thinking_language_label(detail))
        }
        "formalize" => {
            if has_detail {
                let task = humanize_meta_identifier(&trimmed);
                format!(
                    "Formalize the request as {} {task} task.",
                    indefinite_article(&task)
                )
            } else {
                "Formalize the request into a symbolic tuple.".to_owned()
            }
        }
        "formalize_resolved" => {
            if has_detail {
                format!(
                    "Resolve the request to {}.",
                    humanize_meta_identifier(&trimmed)
                )
            } else {
                "Resolve the request to a concrete entity.".to_owned()
            }
        }
        "clarify_formalization" => {
            if has_detail {
                format!("Ask for clarification between {trimmed}.")
            } else {
                "Ask for clarification because the request was ambiguous.".to_owned()
            }
        }
        "dispatch_handler" => {
            if has_detail {
                format!(
                    "Route to the {} handler.",
                    humanize_meta_identifier(&trimmed)
                )
            } else {
                "Route the request to a handler.".to_owned()
            }
        }
        "route_attempt" => {
            if has_detail {
                format!("Try the {} approach.", humanize_meta_identifier(&trimmed))
            } else {
                "Try the next candidate approach.".to_owned()
            }
        }
        "match_rule" => {
            if has_detail {
                format!("Match the {} rule.", humanize_meta_identifier(&trimmed))
            } else {
                "Match a known rule.".to_owned()
            }
        }
        "compute" => {
            if has_detail {
                format!("Compute {trimmed}.")
            } else {
                "Compute the result.".to_owned()
            }
        }
        "compute_engine" => {
            if has_detail {
                format!("Evaluate with the {}.", humanize_meta_identifier(&trimmed))
            } else {
                "Evaluate with the calculator.".to_owned()
            }
        }
        "compute_expression" => format!("Reduce the expression {trimmed}."),
        "compute_steps" => format!("Apply {trimmed} reduction step(s)."),
        "lookup_fact" => {
            if has_detail {
                format!("Look up {}.", humanize_meta_identifier(&trimmed))
            } else {
                "Look up the relevant fact.".to_owned()
            }
        }
        "invoke_tool" => {
            if has_detail {
                format!("Use the {} capability.", humanize_meta_identifier(&trimmed))
            } else {
                "Use an available capability.".to_owned()
            }
        }
        "rule_verification" => {
            if has_detail {
                format!(
                    "Verify the result against the {} rule.",
                    humanize_meta_identifier(&trimmed)
                )
            } else {
                "Verify the result against the rules.".to_owned()
            }
        }
        "policy_refusal" => {
            if has_detail {
                format!(
                    "Decline the request under the {} policy.",
                    humanize_meta_identifier(&trimmed)
                )
            } else {
                "Decline the request under the safety policy.".to_owned()
            }
        }
        "rule_construction" => "Build a local behavior rule.".to_owned(),
        "coreference_binding" => "Resolve what the follow-up refers to.".to_owned(),
        "modifier_detection" => "Detect modifiers in the request.".to_owned(),
        "program_plan" => {
            if has_detail {
                format!("Plan the program: {}.", humanize_meta_identifier(&trimmed))
            } else {
                "Plan the requested program.".to_owned()
            }
        }
        "scan_memory" => {
            if has_detail {
                format!("Search memory for {trimmed}.")
            } else {
                "Search memory for relevant facts.".to_owned()
            }
        }
        "user_context" => {
            if has_detail {
                format!("Apply available context: {trimmed}.")
            } else {
                "Apply the available context.".to_owned()
            }
        }
        "deformalize" => {
            if has_detail {
                format!("Compose the answer: \"{trimmed}\".")
            } else {
                "Compose the answer in natural language.".to_owned()
            }
        }
        "http_chat" => "Exchange a request with the configured endpoint.".to_owned(),
        "agent_plan" => {
            if has_detail {
                format!("Add an agent task: {}.", humanize_meta_identifier(&trimmed))
            } else {
                "Extend the agent plan.".to_owned()
            }
        }
        "memory" => "Update the local memory bundle.".to_owned(),
        "extract_term" => "Extract the search term.".to_owned(),
        "group_by_conversation" => "Group matching memories by conversation.".to_owned(),
        "fallback" => "Fall back to the general unknown-request strategy.".to_owned(),
        other => {
            let readable = humanize_meta_identifier(other);
            let label = if readable.is_empty() {
                "step".to_owned()
            } else {
                readable
            };
            if has_detail {
                format!("{label}: {trimmed}.")
            } else {
                format!("{label}.")
            }
        }
    }
}
