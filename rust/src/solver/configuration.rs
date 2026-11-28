use super::{BlueprintComposition, ExecutionSurface};
use crate::probability::ProbabilityDecisionPolicy;

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
    /// Whether the meta core records the skill-accumulation ledger (R342).
    /// `Accumulate` proposes reusable skills and curriculum items; `Off` records
    /// nothing. Proposals require review, and either mode is trace-only (R13/C3).
    pub skill_mode: crate::skill_ledger::SkillMode,
    /// Whether the dialogue's symbolic world model is maintained (issue #702).
    /// `Track` rebuilds and records current/target contexts; `Off` leaves them
    /// absent. Either mode is trace-only (R13).
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
    /// (issue #556). `None` uses prompt detection; a language-switch follow-up
    /// sets an ISO 639-1 code before replaying the whole solver. The setting also
    /// prevents recursive language-switch replays.
    pub forced_response_language: Option<&'static str>,
    /// Output constraints never restrict the accepted input grammar.
    pub output: crate::cnl_output::OutputConfig,
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
            output: crate::cnl_output::configuration::configured_output(),
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
