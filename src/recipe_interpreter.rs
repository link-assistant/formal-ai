//! Issue #559 (R343): executing the meta algorithm *as data*.
//!
//! The recursive-core recipe (`data/meta/recursive-core-recipe.lino`) is the meta
//! algorithm encoded as Links Notation. The earlier phases used it as a *checked
//! description* — grounded against the source, read by the self-improvement loop —
//! but the algorithm still only ever *ran* as hand-written control flow in
//! `crate::meta_core::record_meta_core`. This module closes that gap: it parses
//! the recipe into an ordered program and **executes it**, driving the live
//! recorder primitives in the order the data prescribes.
//!
//! Each `meta_step` that corresponds to a trace-recorded stage carries a `records`
//! field naming the recorder primitive it drives (e.g. `record_problem_frame`).
//! The interpreter walks the steps in their declared `order`, and for every step
//! with a `records` binding it invokes that primitive against a shared
//! [`EventLog`], threading the intermediate artifacts (problem frame, work-unit
//! tree, need ledger, method registry, solution evidence) through a small
//! execution context exactly as the hand-written pipeline does. Steps without a
//! binding (formalize, resolve-leaves, project-answer) are external to the
//! trace-recorder pass and contribute no event, so the interpreter records them as
//! skipped rather than failing.
//!
//! The point is **parity**: the event log produced by executing the recipe is
//! byte-for-byte identical to the one `crate::meta_core::record_meta_core`
//! produces for the same input and modes. That is the concrete proof that the
//! algorithm-as-data and the algorithm-as-code are the same algorithm — the
//! foundation for eventually driving the pipeline *from* the recipe rather than
//! from a parallel hand-written sequence (issue #558 will take the analogous step
//! for Rust recompilation). The interpreter changes no routing and no answer; it
//! is a second, data-driven way to produce the same trace-only artifacts (R13).

use crate::event_log::EventLog;
use crate::intent_formalization::IntentFormalization;
use crate::links_format::format_lino_record;
use crate::meta_construction::RecursionMode;
use crate::meta_frame::{NeedLedger, ProblemFrame, WorkUnit};
use crate::method_registry::MethodRegistry;
use crate::selection::SelectionMode;
use crate::skill_ledger::SkillMode;
use crate::solution_evidence::SolutionEvidence;

/// The recipe — the meta algorithm encoded as link data — embedded at compile
/// time so the interpreter can execute it with no runtime filesystem dependency.
const RECIPE_LINO: &str = include_str!("../data/meta/recursive-core-recipe.lino");

/// One ordered step of the recipe, parsed from a `meta_step` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeStep {
    /// The 1-based position the step declares in the recipe.
    pub order: u8,
    /// The step's stable id, e.g. `build_problem_frame`.
    pub id: String,
    /// The human-readable title.
    pub title: String,
    /// The source file the step cites.
    pub source_file: String,
    /// The recorder primitive this step drives, if it is a trace-recorded stage.
    /// `None` for the external stages (formalize, resolve-leaves, project-answer)
    /// that contribute no trace event to the meta-core pass.
    pub records: Option<String>,
}

impl RecipeStep {
    /// Whether this step drives a trace-recorded primitive (has a `records` bind).
    #[must_use]
    pub const fn is_executable(&self) -> bool {
        self.records.is_some()
    }
}

/// The recipe parsed into an ordered, executable program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeProgram {
    /// The steps, sorted ascending by their declared `order`.
    pub steps: Vec<RecipeStep>,
}

impl RecipeProgram {
    /// Parse a recipe's `meta_step` records into an ordered program.
    #[must_use]
    pub fn from_lino(recipe_lino: &str) -> Self {
        let mut steps = parse_steps(recipe_lino);
        steps.sort_by_key(|step| step.order);
        Self { steps }
    }

    /// Parse the checked-in recipe embedded at compile time.
    #[must_use]
    pub fn from_repo() -> Self {
        Self::from_lino(RECIPE_LINO)
    }

    /// The number of steps in the program.
    #[must_use]
    pub const fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// The recorder primitives the program drives, in execution order.
    ///
    /// This is the data-declared stage sequence; [`Self::reproduces_pipeline`]
    /// proves it matches the order the live pipeline actually invokes them.
    #[must_use]
    pub fn recorder_sequence(&self) -> Vec<&str> {
        self.steps
            .iter()
            .filter_map(|step| step.records.as_deref())
            .collect()
    }

    /// Execute the program against a fresh event log for one formalized prompt.
    ///
    /// Walks the steps in declared order and invokes each bound recorder primitive,
    /// mirroring `crate::meta_core::record_meta_core` — including its external
    /// gate on the white-box reasoning stage (only run when the recursion mode emits
    /// the downward direction). Returns the [`ExecutionTrace`] (the produced log and
    /// the executed / skipped step ids), or an error if the data is inconsistent
    /// (an out-of-order dependency or an unknown `records` binding).
    ///
    /// # Errors
    ///
    /// Returns `Err` when a step's bound recorder needs an artifact an earlier step
    /// should have produced but did not (a misordered recipe), or names a recorder
    /// the interpreter does not know.
    pub fn execute(
        &self,
        formalization: &IntentFormalization,
        max_depth: u8,
        recursion_mode: RecursionMode,
        selection_mode: SelectionMode,
        skill_mode: SkillMode,
    ) -> Result<ExecutionTrace, String> {
        let mut log = EventLog::new();
        let mut context = ExecutionContext {
            formalization,
            max_depth,
            recursion_mode,
            selection_mode,
            skill_mode,
            problem_frame: None,
            work_unit_root: None,
            need_ledger: None,
            method_registry: None,
            solution_evidence: None,
        };
        let mut executed = Vec::new();
        let mut skipped = Vec::new();
        for step in &self.steps {
            match &step.records {
                Some(recorder) => {
                    let ran = context.run_recorder(&mut log, recorder)?;
                    if ran {
                        executed.push(step.id.clone());
                    } else {
                        // The bound recorder was gated off by the active mode (e.g.
                        // upward construction in `Down`); it emitted no event.
                        skipped.push(step.id.clone());
                    }
                }
                None => skipped.push(step.id.clone()),
            }
        }
        Ok(ExecutionTrace {
            log,
            executed,
            skipped,
        })
    }

    /// Whether executing the recipe reproduces the live pipeline exactly.
    ///
    /// Runs both the data-driven interpreter and the hand-written
    /// `crate::meta_core::record_meta_core` over the same inputs and modes and
    /// compares their event logs event-for-event. True means the algorithm-as-data
    /// and the algorithm-as-code produce the identical trace — the parity that makes
    /// the recipe a faithful executable description of the pipeline.
    #[must_use]
    pub fn reproduces_pipeline(
        &self,
        formalization: &IntentFormalization,
        max_depth: u8,
        recursion_mode: RecursionMode,
        selection_mode: SelectionMode,
        skill_mode: SkillMode,
    ) -> bool {
        let Ok(trace) = self.execute(
            formalization,
            max_depth,
            recursion_mode,
            selection_mode,
            skill_mode,
        ) else {
            return false;
        };
        let mut reference = EventLog::new();
        crate::meta_core::record_meta_core(
            &mut reference,
            formalization,
            max_depth,
            recursion_mode,
            selection_mode,
            skill_mode,
        );
        trace.log.events() == reference.events()
    }

    /// Render the execution plan as Links Notation: the ordered steps, each marked
    /// as the recorder it drives or as an external (non-recording) stage.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = format_lino_record(
            "recipe_program",
            &[
                ("record_type", "recipe_program".to_owned()),
                ("step_count", self.step_count().to_string()),
                ("recorder_count", self.recorder_sequence().len().to_string()),
            ],
        );
        for step in &self.steps {
            out.push('\n');
            out.push_str(&format_lino_record(
                &format!("plan_{}", step.id),
                &[
                    ("record_type", "recipe_plan_step".to_owned()),
                    ("order", step.order.to_string()),
                    ("id", step.id.clone()),
                    (
                        "executes",
                        step.records
                            .clone()
                            .unwrap_or_else(|| "external".to_owned()),
                    ),
                ],
            ));
        }
        out
    }
}

/// The result of executing the recipe program.
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    /// The event log produced by data-driven execution.
    pub log: EventLog,
    /// The ids of steps whose bound recorder actually emitted events, in order.
    pub executed: Vec<String>,
    /// The ids of steps that emitted nothing: external stages and mode-gated ones.
    pub skipped: Vec<String>,
}

/// The artifacts threaded between recorder primitives during execution, mirroring
/// the locals the hand-written pipeline holds across its calls.
struct ExecutionContext<'a> {
    formalization: &'a IntentFormalization,
    max_depth: u8,
    recursion_mode: RecursionMode,
    selection_mode: SelectionMode,
    skill_mode: SkillMode,
    problem_frame: Option<ProblemFrame>,
    work_unit_root: Option<WorkUnit>,
    need_ledger: Option<NeedLedger>,
    method_registry: Option<MethodRegistry>,
    solution_evidence: Option<SolutionEvidence>,
}

impl ExecutionContext<'_> {
    /// Invoke one recorder primitive by its name, threading artifacts through the
    /// context. Returns whether the primitive emitted any event (false when a mode
    /// gated it off), or an error for a missing dependency or unknown primitive.
    fn run_recorder(&mut self, log: &mut EventLog, recorder: &str) -> Result<bool, String> {
        match recorder {
            "record_problem_frame" => {
                self.problem_frame = Some(crate::meta_frame::record_problem_frame(
                    log,
                    self.formalization,
                ));
                Ok(true)
            }
            "record_work_units" => {
                self.work_unit_root = Some(crate::meta_frame::record_work_units(
                    log,
                    self.formalization,
                    self.max_depth,
                ));
                Ok(true)
            }
            "record_need_ledger" => {
                let frame = self.require_problem_frame(recorder)?;
                let root = self.require_work_unit_root(recorder)?;
                self.need_ledger = Some(crate::meta_frame::record_need_ledger(log, frame, root));
                Ok(true)
            }
            "record_method_registry" => {
                self.method_registry = Some(crate::method_registry::record_method_registry(log));
                Ok(true)
            }
            "record_work_unit_reasoning" => {
                // Mirror the pipeline's external gate: only the downward direction
                // emits the white-box reasoning event.
                if !self.recursion_mode.emits_downward() {
                    return Ok(false);
                }
                let root = self.require_work_unit_root(recorder)?;
                let registry = self.require_method_registry(recorder)?;
                crate::meta_reasoning::record_work_unit_reasoning(log, root, registry);
                Ok(true)
            }
            "record_upward_construction" => {
                let root = self.require_work_unit_root(recorder)?;
                let registry = self.require_method_registry(recorder)?;
                let emitted = crate::meta_construction::record_upward_construction(
                    log,
                    root,
                    registry,
                    self.recursion_mode,
                )
                .is_some();
                Ok(emitted)
            }
            "record_solution_evidence" => {
                let frame = self.require_problem_frame(recorder)?;
                let ledger = self.require_need_ledger(recorder)?;
                let registry = self.require_method_registry(recorder)?;
                self.solution_evidence = Some(crate::solution_evidence::record_solution_evidence(
                    log, frame, ledger, registry,
                ));
                Ok(true)
            }
            "record_selection" => {
                let root = self.require_work_unit_root(recorder)?;
                let registry = self.require_method_registry(recorder)?;
                let emitted =
                    crate::selection::record_selection(log, root, registry, self.selection_mode)
                        .is_some();
                Ok(emitted)
            }
            "record_skill_ledger" => {
                let evidence = self.require_solution_evidence(recorder)?;
                let emitted =
                    crate::skill_ledger::record_skill_ledger(log, evidence, self.skill_mode)
                        .is_some();
                Ok(emitted)
            }
            other => Err(format!("recipe binds unknown recorder `{other}`")),
        }
    }

    fn require_problem_frame(&self, recorder: &str) -> Result<&ProblemFrame, String> {
        self.problem_frame
            .as_ref()
            .ok_or_else(|| dependency_error(recorder, "problem frame"))
    }

    fn require_work_unit_root(&self, recorder: &str) -> Result<&WorkUnit, String> {
        self.work_unit_root
            .as_ref()
            .ok_or_else(|| dependency_error(recorder, "work-unit tree"))
    }

    fn require_need_ledger(&self, recorder: &str) -> Result<&NeedLedger, String> {
        self.need_ledger
            .as_ref()
            .ok_or_else(|| dependency_error(recorder, "need ledger"))
    }

    fn require_method_registry(&self, recorder: &str) -> Result<&MethodRegistry, String> {
        self.method_registry
            .as_ref()
            .ok_or_else(|| dependency_error(recorder, "method registry"))
    }

    fn require_solution_evidence(&self, recorder: &str) -> Result<&SolutionEvidence, String> {
        self.solution_evidence
            .as_ref()
            .ok_or_else(|| dependency_error(recorder, "solution evidence"))
    }
}

/// Build a misordered-recipe error message.
fn dependency_error(recorder: &str, dependency: &str) -> String {
    format!("recipe step `{recorder}` runs before its {dependency} dependency")
}

/// Parse every `meta_step` record into a [`RecipeStep`], skipping records that are
/// missing a required field (a malformed recipe is caught by the grounding tests).
fn parse_steps(recipe_lino: &str) -> Vec<RecipeStep> {
    let mut steps = Vec::new();
    let mut block: Vec<&str> = Vec::new();
    for line in recipe_lino.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            if let Some(step) = step_from_block(&block) {
                steps.push(step);
            }
            block.clear();
        }
        block.push(line);
    }
    if let Some(step) = step_from_block(&block) {
        steps.push(step);
    }
    steps
}

/// Build a [`RecipeStep`] from one record's lines, or `None` when the block is not
/// a `meta_step` or is missing a required field.
fn step_from_block(block: &[&str]) -> Option<RecipeStep> {
    let field = |key: &str| {
        block
            .iter()
            .map(|line| line.trim())
            .find_map(|line| field_value(line, key))
    };
    if field("record_type").as_deref() != Some("meta_step") {
        return None;
    }
    Some(RecipeStep {
        order: field("order")?.parse().ok()?,
        id: field("id")?,
        title: field("title")?,
        source_file: field("source_file")?,
        records: field("records"),
    })
}

/// Read `key "value"` (or `key value`) from a recipe line, returning the unquoted
/// value when the key matches exactly.
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
