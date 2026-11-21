//! Universal problem-solving algorithm.
//! Every prompt walks the same 11-step loop from `VISION.md` and `REQUIREMENTS.md`:
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
    language_aware_intent_for, normalize_prompt, response_link_for_intent, stable_id,
};
use crate::event_log::{EventLog, build_evidence_links};
use crate::intent_formalization::{
    IntentFormalization, IntentFormalizationCache, IntentFormalizationCacheEntry,
    ordered_handler_names, record_intent_formalization, recover_write_program_rule,
    rewrite_bare_program_coreference_rule, select_rule_for_intent,
};
use crate::language::{Language, detect as detect_language};
use crate::probability::ProbabilityStore;
use crate::proof_engine::ProofRenderConfig;
use crate::rule_synthesis::try_construct_unknown_rule;
use crate::seed;
use crate::solver_diagnostics::append_diagnostic_trace;
use crate::solver_dispatch::{ContextualOutcome, SPECIALIZED_HANDLERS, try_contextual_override};
use crate::solver_formalization::{record_formalization, record_formalization_selection};
use crate::solver_handlers::{
    CapabilityRuntime, SelfAwarenessRuntime, finalize_simple, try_agent_workspace_task,
    try_behavior_rules_with_runtime, try_definition_merge_by_default, try_feature_capability,
    try_natural_language_tool_request, try_playwright_script, try_program_blueprint,
    try_project_lookup,
};
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
    select_formalization_candidate_with_probability_store,
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

fn env_definition_fusion_by_default() -> Option<bool> {
    env_bool_with_extra_truthy(
        "FORMAL_AI_DEFINITION_FUSION",
        &["auto", "merge", "fusion", "default"],
        &["explicit", "manual", "none"],
    )
}

fn env_bool(name: &str) -> Option<bool> {
    env_bool_with_extra_truthy(name, &[], &[])
}

fn env_bool_with_extra_truthy(name: &str, truthy: &[&str], falsy: &[&str]) -> Option<bool> {
    let raw = std::env::var(name).ok()?;
    let value = raw.trim().to_ascii_lowercase();
    if value.is_empty() {
        return None;
    }
    match value.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        other if truthy.contains(&other) => Some(true),
        other if falsy.contains(&other) => Some(false),
        _ => None,
    }
}

fn env_bounded_f32(name: &str, min: f32, max: f32) -> Option<f32> {
    let parsed = std::env::var(name).ok()?.trim().parse::<f32>().ok()?;
    if parsed.is_finite() {
        Some(parsed.clamp(min, max))
    } else {
        None
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

#[path = "solver/execution.rs"]
mod execution;
/// Mirrors [`UniversalSolver::solve`] with the environment-derived [`SolverConfig`]
/// while preserving the deterministic-projection guarantee from `NON-GOALS.md`.
#[must_use]
pub fn solve(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::default().solve(prompt)
}

/// Convenience entry point that mirrors [`UniversalSolver::solve_with_history`].
#[must_use]
pub fn solve_with_history(prompt: &str, history: &[ConversationTurn]) -> SymbolicAnswer {
    UniversalSolver::default().solve_with_history(prompt, history)
}
