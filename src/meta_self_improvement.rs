//! Issue #559 (R340): the meta algorithm reasoning about improving *itself*.
//!
//! The headline self-improvement requirement of #559 is meta-circular: the
//! algorithm must be able to take *itself* — the recursive-core recipe, i.e. the
//! algorithm encoded as Links Notation (`data/meta/recursive-core-recipe.lino`) —
//! together with what it is required to do (here: the stages the live pipeline
//! `meta_core::record_meta_core` actually runs) as input, both meta-language
//! encoded, and produce the *updated* algorithm as output, again link-encoded.
//!
//! This module realises that loop in its safest, fully-gated form. It compares the
//! algorithm-as-data (the `meta_function` citations in the recipe) against the
//! algorithm-as-code (the `record_*` stages the pipeline invokes) and, when they
//! have drifted, emits a *proposed* recipe update as Links Notation — the "updated
//! meta algorithm". It never writes the recipe back: adoption stays a human review
//! step, exactly like [`crate::self_improvement`] stops at proposing seed rules
//! (R12, C3). The default [`SelfImprovementMode::Off`] proposes nothing, so the
//! loop is dormant unless explicitly engaged.
//!
//! The drift it detects is real and useful: a maintainer who adds a stage to
//! `record_meta_core` but forgets to describe it in the recipe creates exactly the
//! gap this loop surfaces, keeping the self-description honest in the code→data
//! direction (the recipe→code direction is already pinned by
//! `tests/unit/specification/recursive_core_recipe.rs`).

use std::collections::BTreeSet;

use crate::links_format::format_lino_record;

/// The recipe — the meta algorithm encoded as link data — read at compile time so
/// the loop can reason about itself with no runtime filesystem dependency.
const RECIPE_LINO: &str = include_str!("../data/meta/recursive-core-recipe.lino");

/// The live meta-core pipeline source — the ground truth of what the algorithm
/// actually does, also embedded at compile time.
const PIPELINE_SRC: &str = include_str!("meta_core.rs");

/// Whether the meta self-improvement loop may produce a proposal.
///
/// The loop is trace-only and proposal-only regardless of the mode; the mode only
/// gates whether it runs at all, so the default leaves behaviour untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelfImprovementMode {
    /// Default: the loop is dormant and proposes nothing.
    #[default]
    Off,
    /// The loop inspects itself and emits a proposed recipe update (never applied).
    Propose,
}

impl SelfImprovementMode {
    /// Whether this mode lets the loop emit a proposal.
    #[must_use]
    pub const fn proposes(self) -> bool {
        matches!(self, Self::Propose)
    }

    /// The stable slug used in serialization and the env override.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Propose => "propose",
        }
    }

    /// Parse a slug back into a mode (case-insensitive, trimmed).
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.trim().to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "propose" => Some(Self::Propose),
            _ => None,
        }
    }
}

/// One stage the live pipeline runs, as the `(module, function)` pair parsed from
/// a `crate::<module>::record_<name>(` call.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PipelineStage {
    /// The module the stage lives in, e.g. `solution_evidence`.
    pub module: String,
    /// The recorder function the pipeline calls, e.g. `record_solution_evidence`.
    pub function: String,
}

impl PipelineStage {
    /// The source file the recipe should cite for this stage.
    #[must_use]
    pub fn source_file(&self) -> String {
        format!("src/{}.rs", self.module)
    }
}

/// A proposed update to the recipe — the meta algorithm's own description.
///
/// It reconciles the recipe with the pipeline that actually runs, and *is* the
/// "updated meta algorithm" the requirement asks for, in delta form: which
/// `meta_function` citations to add and which stale ones to drop. It is never
/// applied automatically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaRecipeProposal {
    /// Pipeline stages the recipe does not yet cite as a `meta_function`.
    pub undescribed_stages: Vec<PipelineStage>,
    /// `record_*` functions the recipe cites that the pipeline no longer runs.
    pub stale_citations: Vec<String>,
}

impl MetaRecipeProposal {
    /// Whether the algorithm already describes itself completely — no additions and
    /// no stale citations. When true, the recipe and the live pipeline agree.
    #[must_use]
    pub const fn is_self_consistent(&self) -> bool {
        self.undescribed_stages.is_empty() && self.stale_citations.is_empty()
    }

    /// The number of proposed changes (additions plus removals).
    #[must_use]
    pub const fn change_count(&self) -> usize {
        self.undescribed_stages.len() + self.stale_citations.len()
    }

    /// A one-line human-readable summary of the proposal.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.is_self_consistent() {
            return "The recipe already describes every pipeline stage; no update proposed."
                .to_owned();
        }
        let add = self
            .undescribed_stages
            .iter()
            .map(|stage| stage.function.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let drop = self.stale_citations.join(", ");
        match (add.is_empty(), drop.is_empty()) {
            (false, true) => format!("Propose citing undescribed pipeline stage(s): {add}."),
            (true, false) => format!("Propose dropping stale recipe citation(s): {drop}."),
            _ => format!("Propose citing {add}; propose dropping stale citation(s): {drop}."),
        }
    }

    /// Render the proposed updated algorithm as Links Notation.
    #[must_use]
    pub fn to_links_notation(&self, mode: SelfImprovementMode) -> String {
        let pairs: Vec<(&str, String)> = vec![
            ("record_type", "meta_recipe_proposal".to_owned()),
            ("mode", mode.slug().to_owned()),
            ("self_consistent", self.is_self_consistent().to_string()),
            ("change_count", self.change_count().to_string()),
        ];
        let mut out = format_lino_record("meta_recipe_proposal", &pairs);
        for stage in &self.undescribed_stages {
            out.push('\n');
            out.push_str(&format_lino_record(
                &format!("add_{}", stage.function),
                &[
                    ("record_type", "proposed_meta_function".to_owned()),
                    ("function", stage.function.clone()),
                    ("source_file", stage.source_file()),
                ],
            ));
        }
        for stale in &self.stale_citations {
            out.push('\n');
            out.push_str(&format_lino_record(
                &format!("drop_{stale}"),
                &[
                    ("record_type", "stale_meta_function".to_owned()),
                    ("function", stale.clone()),
                ],
            ));
        }
        out
    }
}

/// The meta self-improvement loop: the recipe (itself) versus the live pipeline.
#[derive(Debug, Clone)]
pub struct MetaSelfImprovement {
    /// `record_*` functions the recipe cites as `meta_function` records.
    recipe_record_functions: BTreeSet<String>,
    /// Stages the pipeline actually runs, in source order, de-duplicated.
    pipeline_stages: Vec<PipelineStage>,
}

impl MetaSelfImprovement {
    /// Build the loop from explicit sources (used by tests with synthetic input).
    #[must_use]
    pub fn from_sources(recipe_lino: &str, pipeline_src: &str) -> Self {
        Self {
            recipe_record_functions: recipe_record_functions(recipe_lino),
            pipeline_stages: pipeline_stages(pipeline_src),
        }
    }

    /// Build the loop from the checked-in recipe and pipeline embedded at compile
    /// time — the algorithm reading its own description and implementation.
    #[must_use]
    pub fn from_repo() -> Self {
        Self::from_sources(RECIPE_LINO, PIPELINE_SRC)
    }

    /// The stages the live pipeline runs, in source order.
    #[must_use]
    pub fn pipeline_stages(&self) -> &[PipelineStage] {
        &self.pipeline_stages
    }

    /// Compute the proposed recipe update by comparing the two.
    #[must_use]
    pub fn propose(&self) -> MetaRecipeProposal {
        let undescribed_stages = self
            .pipeline_stages
            .iter()
            .filter(|stage| !self.recipe_record_functions.contains(&stage.function))
            .cloned()
            .collect();
        let pipeline_functions: BTreeSet<&str> = self
            .pipeline_stages
            .iter()
            .map(|stage| stage.function.as_str())
            .collect();
        let stale_citations = self
            .recipe_record_functions
            .iter()
            .filter(|function| !pipeline_functions.contains(function.as_str()))
            .cloned()
            .collect();
        MetaRecipeProposal {
            undescribed_stages,
            stale_citations,
        }
    }
}

/// Run the meta self-improvement loop, gated by `mode`.
///
/// Returns `None` when `mode` is [`SelfImprovementMode::Off`] (the default), so the
/// loop is dormant unless explicitly engaged. When it runs it only computes a
/// proposal; it never writes the recipe back.
#[must_use]
pub fn propose_recipe_update(
    recipe_lino: &str,
    pipeline_src: &str,
    mode: SelfImprovementMode,
) -> Option<MetaRecipeProposal> {
    mode.proposes()
        .then(|| MetaSelfImprovement::from_sources(recipe_lino, pipeline_src).propose())
}

/// Collect the `record_*` function names the recipe cites in `meta_function`
/// records. Only the `record_*` convention is compared, since those are the
/// pipeline stages; helper functions like `decompose_once` are not stages.
fn recipe_record_functions(recipe_lino: &str) -> BTreeSet<String> {
    let mut functions = BTreeSet::new();
    let mut in_meta_function = false;
    for line in recipe_lino.lines() {
        let trimmed = line.trim();
        if !line.starts_with(char::is_whitespace) {
            // A new top-level record begins; reset the record-type context.
            in_meta_function = false;
            continue;
        }
        if let Some(value) = field_value(trimmed, "record_type") {
            in_meta_function = value == "meta_function";
            continue;
        }
        if in_meta_function {
            if let Some(name) = field_value(trimmed, "function") {
                if name.starts_with("record_") {
                    functions.insert(name);
                }
            }
        }
    }
    functions
}

/// Parse `crate::<module>::record_<name>(` calls from the pipeline source, in
/// source order, de-duplicated (a stage called once is one stage).
fn pipeline_stages(pipeline_src: &str) -> Vec<PipelineStage> {
    let mut stages = Vec::new();
    let mut seen = BTreeSet::new();
    for fragment in pipeline_src.split("crate::").skip(1) {
        let Some(call) = fragment.split('(').next() else {
            continue;
        };
        let parts: Vec<&str> = call.split("::").collect();
        if parts.len() != 2 {
            continue;
        }
        let module = parts[0].trim();
        let function = parts[1].trim();
        if !function.starts_with("record_")
            || !is_ident(module)
            || !is_ident(function)
            || !seen.insert(function.to_owned())
        {
            continue;
        }
        stages.push(PipelineStage {
            module: module.to_owned(),
            function: function.to_owned(),
        });
    }
    stages
}

/// Whether `value` is a plain Rust identifier (letters, digits, underscores).
fn is_ident(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Read `key "value"` from a recipe line, returning the unquoted value.
fn field_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let raw = rest.trim();
    let unquoted = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw);
    Some(unquoted.to_owned())
}
