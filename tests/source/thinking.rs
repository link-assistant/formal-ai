//! Human-readable "thinking" steps (issue #488), localized (issue #889).
//!
//! This module owns the [`ThinkingStep`] model and the deterministic
//! `(step, detail) -> concrete sentence` naturalizer that every native surface
//! shares: the core `EventLog` projection, the CLI `--thinking` output, the
//! OpenAI/Anthropic API responses, and the Telegram bot. The browser mirrors the
//! same two stages in JavaScript — the curated projection in
//! `src/web/formal_ai_worker.js` and the naturalizer (`naturalizeThinkingStep`)
//! in `src/web/app.js`.
//!
//! The sentences themselves are *data*, not Rust literals: they live in
//! `data/seed/multilingual-responses-thinking*.lino` and are looked up through
//! [`crate::thinking_prose`] in the answer language, so the CLI, the APIs and
//! Telegram narrate a Russian answer in Russian instead of describing it in
//! English (issue #889). The trace keys (`step`, `detail`) stay
//! language-neutral, so nothing downstream has to parse prose.
//!
//! Keeping it in its own module (rather than inside `engine.rs`) keeps each file
//! focused and within the repository's per-file line budget while making
//! "thinking" a first-class concern of the architecture rather than an engine
//! implementation detail.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::engine::stable_id;
use crate::thinking_prose::{
    LANGUAGE_NAME_INTENT_PREFIX, NARRATIVE_INTENT_PREFIX, PLAIN_INTENT_SUFFIX, STEP_INTENT_PREFIX,
    language_label, normalize_language, thinking_prose,
};

/// The language a trace is narrated in when it carries no language step.
pub const DEFAULT_THINKING_LANGUAGE: &str = "en";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingStep {
    pub id: String,
    pub order: u32,
    pub step: String,
    pub detail: String,
    /// Concrete, human-readable description of this step (issue #488).
    ///
    /// This is the "meta-language description" layer: a single sentence that
    /// surfaces the actual content of the step (the prompt, the computed result,
    /// the looked-up entity, the chosen route, the composed answer) rather than
    /// a generic category label. It is written in the answer language once the
    /// trace is complete ([`localize_thinking_steps`], issue #889); the browser
    /// re-derives its own localized sentence from `step` and `detail`.
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
        // The answer language is only known once the whole trace exists (it is
        // itself one of the steps), so a step is born narrating in the fallback
        // language and is re-narrated by `localize_thinking_steps`.
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

    /// This step's sentence in `language`, ignoring any stored summary.
    #[must_use]
    pub fn sentence_in(&self, language: &str) -> String {
        naturalize_thinking_step_in(language, &self.step, &self.detail)
    }
}

/// Rewrite every step's `summary` in `language` (issue #889).
///
/// The `step`/`detail` keys are untouched, so the trace stays language-neutral
/// and this is a pure presentation pass; the ids are derived from those keys
/// too, so localizing never renumbers or re-identifies a trace.
pub fn localize_thinking_steps(steps: &mut [ThinkingStep], language: &str) {
    for step in steps.iter_mut() {
        step.summary = naturalize_thinking_step_in(language, &step.step, &step.detail);
    }
}

/// The language a finished trace should be narrated in.
///
/// Derived from the trace itself — the language the solver resolved to answer
/// in, else the language it detected in the prompt — so every surface renders
/// the same language the answer is written in without extra plumbing.
#[must_use]
pub fn thinking_answer_language(steps: &[ThinkingStep]) -> String {
    for kind in ["resolve_response_language", "detect_language"] {
        if let Some(step) = steps
            .iter()
            .find(|step| strip_agent_substep_prefix(&step.step) == kind)
        {
            let slug = normalize_language(&step.detail);
            if !slug.is_empty() && crate::language::from_slug(&slug).is_some() {
                return slug;
            }
        }
    }
    DEFAULT_THINKING_LANGUAGE.to_owned()
}

/// The word a surface labels a reasoning trace with ("Thinking", «Размышления»),
/// in the language the trace is narrated in.
#[must_use]
pub fn thinking_trace_heading(language: &str) -> String {
    thinking_prose("thinking_trace_heading", language, &[]).unwrap_or_default()
}

/// Map a language slug to its English name for concrete thinking summaries.
#[must_use]
pub fn thinking_language_label(code: &str) -> String {
    thinking_language_label_in(DEFAULT_THINKING_LANGUAGE, code)
}

/// Map a language slug to its name as spoken in `answer_language` (issue #889).
#[must_use]
pub fn thinking_language_label_in(answer_language: &str, code: &str) -> String {
    language_label(answer_language, code)
}

/// Turn a meta-language identifier (`write_program`, `route:greeting`,
/// `concept_lookup:hit`) into a lowercase human phrase (`write program`,
/// `greeting`, `concept lookup hit`).
///
/// The identifiers are language-neutral trace keys, so this stays a mechanical
/// transformation rather than a translation: the localized templates around it
/// carry the words.
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

/// The narrative headline each dispatch route gets, as an intent suffix in
/// `data/seed/multilingual-responses-thinking-narrative.lino`.
///
/// Several routes share one headline (a `concept_lookup` and a `fact_lookup`
/// are the same story to the reader), so the mapping is a table of route keys
/// rather than one intent per route.
const ROUTE_NARRATIVES: &[(&str, &str)] = &[
    ("greeting", "greeting"),
    ("wellbeing", "wellbeing"),
    ("assistant_free_time", "assistant_free_time"),
    ("farewell", "farewell"),
    ("gratitude", "gratitude"),
    ("thanks", "gratitude"),
    ("courtesy_response", "gratitude"),
    ("courtesy", "gratitude"),
    ("identity", "identity"),
    ("assistant_name", "identity"),
    ("recall_name", "identity"),
    ("naming", "identity"),
    ("assistant_naming", "identity"),
    ("calculation", "calculation"),
    ("arithmetic", "calculation"),
    ("fact_lookup", "fact_lookup"),
    ("concept_lookup", "fact_lookup"),
    ("concept_lookup_in_context", "fact_lookup"),
    ("translation", "translation"),
    ("web_search", "web"),
    ("http_fetch", "web"),
    ("url_navigate", "web"),
    ("write_program", "code"),
    ("software_project_plan", "code"),
    ("software_project_implementation", "code"),
    ("algorithm", "code"),
    ("test_status", "test_status"),
    ("self_healing", "self_healing"),
    ("self_heal", "self_healing"),
    ("meta_explanation", "meta_explanation"),
    ("learn_from_source", "learn_from_source"),
    ("clarification", "clarification"),
    ("unknown", "unknown"),
    ("fallback", "unknown"),
];

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
/// deterministic and needs no extra engine state, and it is written in the
/// language the trace answers in (issue #889). Returns `None` when the trace has
/// no recognizable route, leaving the robotic steps to speak for themselves.
#[must_use]
pub fn thinking_narrative(steps: &[ThinkingStep]) -> Option<String> {
    thinking_narrative_in(&thinking_answer_language(steps), steps)
}

/// [`thinking_narrative`] in an explicit language.
#[must_use]
pub fn thinking_narrative_in(language: &str, steps: &[ThinkingStep]) -> Option<String> {
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
    if let Some((_, suffix)) = ROUTE_NARRATIVES
        .iter()
        .find(|(route, _)| *route == route_detail)
    {
        if let Some(text) =
            thinking_prose(&format!("{NARRATIVE_INTENT_PREFIX}{suffix}"), language, &[])
        {
            return Some(text);
        }
    }
    // Any other resolved route still gets a human headline rather than a bare
    // category label: describe it as the task it was read as.
    let task = humanize_meta_identifier(&route_detail);
    thinking_prose(
        &format!("{NARRATIVE_INTENT_PREFIX}generic"),
        language,
        &[("task", &task), ("article", indefinite_article(&task))],
    )
}

/// Render a full reasoning trace as the plain-text form expected by protocol
/// surfaces that expose a single thinking/reasoning string.
///
/// The output leads with a human [`thinking_narrative`] headline (issue #676,
/// R8) when the route is recognizable, followed by the concrete per-step
/// sentences as the recursive "robotic detail" beneath it, all in the language
/// the trace answers in (issue #889).
#[must_use]
pub fn render_thinking_steps(steps: &[ThinkingStep]) -> String {
    render_thinking_steps_in(&thinking_answer_language(steps), steps)
}

/// [`render_thinking_steps`] in an explicit language.
#[must_use]
pub fn render_thinking_steps_in(language: &str, steps: &[ThinkingStep]) -> String {
    let mut lines = Vec::with_capacity(steps.len() + 1);
    if let Some(narrative) = thinking_narrative_in(language, steps) {
        lines.push(narrative);
    }
    for step in steps {
        let sentence = step.sentence_in(language);
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
///
/// Only the English templates interpolate `{article}`; the other languages'
/// records simply do not mention it, so passing it is harmless.
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

/// Step kinds whose sentence interpolates their detail, as
/// `(step kind, placeholder, humanize the detail?)`.
///
/// Each entry uses two seed intents: `thinking_step_<kind>` when the step
/// carries a detail and `thinking_step_<kind>_plain` when it does not. A
/// humanized detail is a meta-language identifier rendered as a phrase
/// (`write_program` -> `write program`); a raw one is user-facing content (a
/// prompt, an expression, a composed answer) that is passed through verbatim.
const DETAIL_STEPS: &[(&str, &str, bool)] = &[
    ("impulse", "prompt", false),
    ("formalize", "task", true),
    ("formalize_resolved", "entity", true),
    ("clarify_formalization", "options", false),
    ("dispatch_handler", "route", true),
    ("route_attempt", "route", true),
    ("match_rule", "rule", true),
    ("compute", "expression", false),
    ("compute_engine", "engine", true),
    ("lookup_fact", "fact", true),
    ("invoke_tool", "tool", true),
    ("rule_verification", "rule", true),
    ("policy_refusal", "policy", true),
    ("program_plan", "plan", true),
    ("scan_memory", "term", false),
    ("user_context", "context", false),
    ("deformalize", "answer", false),
    ("agent_plan", "task", true),
];

/// Step kinds that always interpolate their (possibly empty) detail, so they
/// have no `_plain` variant.
const ALWAYS_DETAIL_STEPS: &[(&str, &str)] = &[
    ("compute_expression", "expression"),
    ("compute_steps", "count"),
];

/// Step kinds whose sentence carries no detail at all.
const PLAIN_STEPS: &[&str] = &[
    "rule_construction",
    "coreference_binding",
    "modifier_detection",
    "http_chat",
    "memory",
    "extract_term",
    "group_by_conversation",
    "fallback",
];

/// Translate a single `(step, detail)` pair into one concrete English sentence.
///
/// Kept as the English-specific entry point that predates issue #889; new
/// callers should pass the answer language to [`naturalize_thinking_step_in`].
#[must_use]
pub fn naturalize_thinking_step(step: &str, detail: &str) -> String {
    naturalize_thinking_step_in(DEFAULT_THINKING_LANGUAGE, step, detail)
}

/// Translate a single `(step, detail)` pair into one concrete sentence in
/// `language`.
///
/// This is the deterministic "meta-language description" stage from issue #488.
/// It is the single source of truth shared by the core projection, the CLI, the
/// OpenAI/Anthropic API surfaces, and (mirrored) the browser worker, so every
/// surface renders the *same* concrete thinking rather than a generic label —
/// and, since issue #889, renders it in the language of the answer.
#[must_use]
pub fn naturalize_thinking_step_in(language: &str, step: &str, detail: &str) -> String {
    let canonical = strip_agent_substep_prefix(step);
    let trimmed = truncate_thinking_detail(detail);
    let has_detail = !trimmed.is_empty();

    // The two language steps name a language rather than quoting their detail,
    // and that name is itself localized (`русский`, not `Russian`).
    if matches!(canonical, "detect_language" | "resolve_response_language") {
        let label = language_label(language, detail);
        if let Some(text) = thinking_prose(
            &format!("{STEP_INTENT_PREFIX}{canonical}"),
            language,
            &[("language", &label)],
        ) {
            return text;
        }
    }

    if let Some((_, placeholder, humanize)) =
        DETAIL_STEPS.iter().find(|(kind, _, _)| *kind == canonical)
    {
        let intent = if has_detail {
            format!("{STEP_INTENT_PREFIX}{canonical}")
        } else {
            format!("{STEP_INTENT_PREFIX}{canonical}{PLAIN_INTENT_SUFFIX}")
        };
        let value: Cow<'_, str> = if *humanize {
            Cow::Owned(humanize_meta_identifier(&trimmed))
        } else {
            Cow::Borrowed(trimmed.as_str())
        };
        if let Some(text) = thinking_prose(
            &intent,
            language,
            &[
                (placeholder, value.as_ref()),
                ("article", indefinite_article(value.as_ref())),
            ],
        ) {
            return text;
        }
    }

    if let Some((_, placeholder)) = ALWAYS_DETAIL_STEPS
        .iter()
        .find(|(kind, _)| *kind == canonical)
    {
        if let Some(text) = thinking_prose(
            &format!("{STEP_INTENT_PREFIX}{canonical}"),
            language,
            &[(placeholder, &trimmed)],
        ) {
            return text;
        }
    }

    if PLAIN_STEPS.contains(&canonical) {
        if let Some(text) =
            thinking_prose(&format!("{STEP_INTENT_PREFIX}{canonical}"), language, &[])
        {
            return text;
        }
    }

    generic_thinking_sentence(language, canonical, &trimmed, has_detail)
}

/// The sentence for a step kind the naturalizer has no dedicated template for:
/// its humanized key, followed by its detail.
fn generic_thinking_sentence(
    language: &str,
    canonical: &str,
    trimmed: &str,
    has_detail: bool,
) -> String {
    let readable = humanize_meta_identifier(canonical);
    let label = if readable.is_empty() {
        thinking_prose(&format!("{STEP_INTENT_PREFIX}unnamed"), language, &[])
            .unwrap_or_else(|| canonical.to_owned())
    } else {
        readable
    };
    let intent = if has_detail {
        format!("{STEP_INTENT_PREFIX}generic")
    } else {
        format!("{STEP_INTENT_PREFIX}generic{PLAIN_INTENT_SUFFIX}")
    };
    thinking_prose(&intent, language, &[("label", &label), ("detail", trimmed)]).unwrap_or_else(
        || {
            // Reached only if the seed lost its generic records; punctuation
            // only, so it stays language-neutral rather than silently English.
            if has_detail {
                format!("{label}: {trimmed}")
            } else {
                label.clone()
            }
        },
    )
}

/// Every seed intent this module renders, for the coverage tests that assert
/// each registered language translates the whole vocabulary (issue #889).
#[must_use]
pub fn thinking_prose_intents() -> Vec<String> {
    let mut intents = Vec::new();
    for (kind, _, _) in DETAIL_STEPS {
        intents.push(format!("{STEP_INTENT_PREFIX}{kind}"));
        intents.push(format!("{STEP_INTENT_PREFIX}{kind}{PLAIN_INTENT_SUFFIX}"));
    }
    for (kind, _) in ALWAYS_DETAIL_STEPS {
        intents.push(format!("{STEP_INTENT_PREFIX}{kind}"));
    }
    for kind in PLAIN_STEPS {
        intents.push(format!("{STEP_INTENT_PREFIX}{kind}"));
    }
    for kind in ["detect_language", "resolve_response_language"] {
        intents.push(format!("{STEP_INTENT_PREFIX}{kind}"));
    }
    intents.push(format!("{STEP_INTENT_PREFIX}unnamed"));
    intents.push(format!("{STEP_INTENT_PREFIX}generic"));
    intents.push(format!("{STEP_INTENT_PREFIX}generic{PLAIN_INTENT_SUFFIX}"));
    let mut narratives: Vec<String> = ROUTE_NARRATIVES
        .iter()
        .map(|(_, suffix)| format!("{NARRATIVE_INTENT_PREFIX}{suffix}"))
        .collect();
    narratives.push(format!("{NARRATIVE_INTENT_PREFIX}generic"));
    narratives.sort();
    narratives.dedup();
    intents.extend(narratives);
    for language in crate::language::registered_languages() {
        intents.push(format!("{LANGUAGE_NAME_INTENT_PREFIX}{}", language.slug()));
    }
    intents.push(format!("{LANGUAGE_NAME_INTENT_PREFIX}unknown"));
    intents
}
