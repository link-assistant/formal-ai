//! Formalizing gathered instructions into a machine-checkable instruction set.
//!
//! Issue #1073, requirement 2: "relevant instructions, guides, and best
//! practices from the internet must be collected for the *class* of task … and
//! formalized: turned into explicit, checkable instruction sets rather than
//! paraphrased prose."
//!
//! A gathered excerpt is a list of `action → check` pairs as some source states
//! them. [`formalize`] merges excerpts from several sources into one ordered
//! [`InstructionSet`]: identical actions collapse into a single step that
//! remembers every source that stated it and every check any source attached,
//! so cross-referencing is a property of the compiled set rather than an
//! informal reading. A step nobody attached a check to is
//! [`StepStatus::Unverifiable`] — the audit refuses it, because an instruction
//! that cannot be checked has not been formalized, only quoted.

use std::collections::BTreeMap;

/// One step as a single source states it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStep {
    /// The action slug the source prescribes.
    pub action: String,
    /// The machine-checkable condition that shows the action took effect. Empty
    /// when the source only describes the action in prose.
    pub check: String,
}

impl SourceStep {
    /// Build a step as stated by one source.
    #[must_use]
    pub fn new(action: impl Into<String>, check: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            check: check.into(),
        }
    }
}

/// The ordered steps one source states for a task class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceExcerpt {
    /// The source these steps were gathered from.
    pub source_id: String,
    /// The steps in the order the source states them.
    pub steps: Vec<SourceStep>,
}

impl SourceExcerpt {
    /// Build an excerpt.
    #[must_use]
    pub fn new(source_id: impl Into<String>, steps: Vec<SourceStep>) -> Self {
        Self {
            source_id: source_id.into(),
            steps,
        }
    }
}

/// How well a compiled step is backed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    /// Stated by enough distinct sources and carrying a check.
    Corroborated,
    /// Carries a check but only one source states it.
    SingleSourced,
    /// No source attached a machine-checkable condition.
    Unverifiable,
}

impl StepStatus {
    /// Stable slug for the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Corroborated => "corroborated",
            Self::SingleSourced => "single_sourced",
            Self::Unverifiable => "unverifiable",
        }
    }
}

/// One compiled instruction: an action plus the checks that decide whether it
/// took effect, with the sources that stated it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionStep {
    /// Position in the compiled set, starting at 1.
    pub order: usize,
    /// The action slug.
    pub action: String,
    /// Every distinct check any source attached, in first-seen order.
    pub checks: Vec<String>,
    /// The sources that stated this action, in first-seen order.
    pub sources: Vec<String>,
    /// How well the step is backed.
    pub status: StepStatus,
}

impl InstructionStep {
    /// Whether any source attached a machine-checkable condition.
    #[must_use]
    pub const fn is_checkable(&self) -> bool {
        !self.checks.is_empty()
    }
}

/// A compiled, ordered, checkable instruction set for one task class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionSet {
    /// The class of task these instructions cover, not the single request.
    pub task_class: String,
    /// The compiled steps in execution order.
    pub steps: Vec<InstructionStep>,
    /// Every source that contributed, in first-seen order.
    pub sources: Vec<String>,
}

impl InstructionSet {
    /// How many distinct sources the set draws on.
    #[must_use]
    pub const fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Steps that no source made checkable — the ones that fail the audit.
    #[must_use]
    pub fn unverifiable_steps(&self) -> Vec<&InstructionStep> {
        self.steps
            .iter()
            .filter(|step| !step.is_checkable())
            .collect()
    }

    /// Steps whose checks were never discharged by an observation.
    ///
    /// This is what makes the set executable rather than decorative: given the
    /// checks the episode actually observed, the set names the instructions that
    /// remain outstanding.
    #[must_use]
    pub fn unmet_steps(&self, discharged: &[String]) -> Vec<&InstructionStep> {
        self.steps
            .iter()
            .filter(|step| {
                !step
                    .checks
                    .iter()
                    .any(|check| discharged.iter().any(|done| done == check))
            })
            .collect()
    }
}

/// Merge gathered excerpts into one ordered, checkable instruction set.
///
/// Steps are keyed by action slug. Ordering is deterministic: by the earliest
/// position any source gave the action, then by the order the action was first
/// seen, so the same excerpts always compile to the same set regardless of how
/// the sources were interleaved.
#[must_use]
pub fn formalize(
    task_class: &str,
    excerpts: &[SourceExcerpt],
    minimum_sources_per_step: usize,
) -> InstructionSet {
    let mut merged: BTreeMap<String, MergedStep> = BTreeMap::new();
    let mut sources: Vec<String> = Vec::new();
    let mut seen = 0usize;
    for excerpt in excerpts {
        if !sources.contains(&excerpt.source_id) {
            sources.push(excerpt.source_id.clone());
        }
        for (position, step) in excerpt.steps.iter().enumerate() {
            let action = step.action.trim().to_owned();
            if action.is_empty() {
                continue;
            }
            let entry = merged.entry(action.clone()).or_insert_with(|| {
                seen += 1;
                MergedStep {
                    action,
                    first_seen: seen,
                    earliest_position: position,
                    checks: Vec::new(),
                    sources: Vec::new(),
                }
            });
            entry.earliest_position = entry.earliest_position.min(position);
            let check = step.check.trim();
            if !check.is_empty() && !entry.checks.iter().any(|known| known == check) {
                entry.checks.push(check.to_owned());
            }
            if !entry.sources.contains(&excerpt.source_id) {
                entry.sources.push(excerpt.source_id.clone());
            }
        }
    }
    let mut ordered = merged.into_values().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.earliest_position
            .cmp(&right.earliest_position)
            .then(left.first_seen.cmp(&right.first_seen))
    });
    let steps = ordered
        .into_iter()
        .enumerate()
        .map(|(index, step)| {
            let status = if step.checks.is_empty() {
                StepStatus::Unverifiable
            } else if step.sources.len() >= minimum_sources_per_step {
                StepStatus::Corroborated
            } else {
                StepStatus::SingleSourced
            };
            InstructionStep {
                order: index + 1,
                action: step.action,
                checks: step.checks,
                sources: step.sources,
                status,
            }
        })
        .collect();
    InstructionSet {
        task_class: task_class.to_owned(),
        steps,
        sources,
    }
}

struct MergedStep {
    action: String,
    first_seen: usize,
    earliest_position: usize,
    checks: Vec<String>,
    sources: Vec<String>,
}
