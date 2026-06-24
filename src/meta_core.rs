//! Issue #559: the general recursive meta core, wired as one cohesive pass.
//!
//! Every request flows through the same pipeline before registry-backed method
//! dispatch executes the selected method:
//!
//! 1. the explicit, link-serializable problem frame (R330) — the meaning record
//!    made first-class, enumerating every detected need (R7);
//! 2. the recursive, bounded downward decomposition into work units (R332) —
//!    decompose until each leaf is directly solvable (R19);
//! 3. the need-satisfaction ledger (R333) — every detected need carries an
//!    explicit status, a blocked need recorded rather than dropped (R8);
//! 4. the method registry (R331) — the catalogue of methods each atomic leaf can
//!    route to, derived from the live dispatch constants and used by the solver's
//!    live method dispatch;
//! 5. the bidirectional recursive reasoning (R337, R338) — a human-readable
//!    thought at every recursive step. The downward pass (observe →
//!    decompose/atomic → method) explains *how the request is taken apart*; the
//!    upward construction pass (leaf method → compose children → root) explains
//!    *how the answer is put back together*. Which directions are emitted is
//!    governed by [`RecursionMode`](crate::meta_construction::RecursionMode)
//!    (default `Down`, behavior-preserving), so the box is inspectable in both
//!    directions, not just the predicate;
//! 6. the solution evidence (R334) — the end-to-end join, per need `frame →
//!    work-unit leaf → status → method`, so "address every detected need" is one
//!    auditable record;
//! 7. the method-selection trace (R339) — for every atomic leaf, the method the
//!    single data-driven registry authority resolves (alias-aware), recorded so
//!    the selection step of the algorithm is inspectable. Governed by
//!    [`SelectionMode`](crate::selection::SelectionMode) (default `Off`, which
//!    records nothing); a leaf with no serving method is recorded `unresolved`
//!    rather than dropped;
//! 8. the skill-accumulation ledger (R342) — distilled from the solution evidence,
//!    every satisfied need becomes a proposed reusable skill and every blocked need
//!    a curriculum item, so the loop accumulates what it can do and a list of what
//!    it cannot yet do. Governed by [`SkillMode`](crate::skill_ledger::SkillMode)
//!    (default `Off`, which records nothing); it is proposal-only — no skill is ever
//!    auto-promoted to stable without tests and a benchmark delta (C3).
//!
//! The recording stages are append-only: each stage appends Links Notation
//! artifacts to the event log. The same registry they record is also the live
//! method-selection authority used by `meta_method_dispatch`.

use crate::event_log::EventLog;
use crate::intent_formalization::IntentFormalization;
use crate::meta_construction::RecursionMode;
use crate::selection::SelectionMode;
use crate::skill_ledger::SkillMode;

/// Record the full meta core for one formalized prompt as trace events.
///
/// `max_depth` bounds the recursive decomposition so the downward pass always
/// terminates. `recursion_mode` selects which recursive directions are reasoned
/// about: the default ([`RecursionMode::Down`]) emits the downward decomposition
/// reasoning only, reproducing the pre-knob trace exactly (R13); `Up`/`Both`
/// additionally emit the upward construction pass. `selection_mode` selects
/// whether the per-leaf registry method-selection trace is recorded: the
/// default ([`SelectionMode::Off`]) records nothing. `skill_mode` selects whether
/// the skill-accumulation ledger is recorded: the default ([`SkillMode::Off`])
/// records nothing. The structural work-unit decomposition events
/// (`work_unit:enter` / `work_unit:exit`) are always emitted regardless of any
/// mode. This is the single seam the solver loop calls; keeping the stages together
/// here keeps the loop body small and the pipeline cohesive.
pub fn record_meta_core(
    log: &mut EventLog,
    formalization: &IntentFormalization,
    max_depth: u8,
    recursion_mode: RecursionMode,
    selection_mode: SelectionMode,
    skill_mode: SkillMode,
) {
    let problem_frame = crate::meta_frame::record_problem_frame(log, formalization);
    let work_unit_root = crate::meta_frame::record_work_units(log, formalization, max_depth);
    let need_ledger = crate::meta_frame::record_need_ledger(log, &problem_frame, &work_unit_root);
    let method_registry = crate::method_registry::record_method_registry(log);
    if recursion_mode.emits_downward() {
        let _reasoning = crate::meta_reasoning::record_work_unit_reasoning(
            log,
            &work_unit_root,
            &method_registry,
        );
    }
    let _construction = crate::meta_construction::record_upward_construction(
        log,
        &work_unit_root,
        &method_registry,
        recursion_mode,
    );
    let solution_evidence = crate::solution_evidence::record_solution_evidence(
        log,
        &problem_frame,
        &need_ledger,
        &method_registry,
    );
    let _selection =
        crate::selection::record_selection(log, &work_unit_root, &method_registry, selection_mode);
    let _skills = crate::skill_ledger::record_skill_ledger(log, &solution_evidence, skill_mode);
}

/// Apply the meta-core mode environment overrides in place.
///
/// `FORMAL_AI_RECURSION_MODE` selects which recursive directions are traced,
/// `FORMAL_AI_SELECTION_MODE` selects whether the method-selection trace is
/// recorded, and `FORMAL_AI_SKILL_MODE` selects whether the skill-accumulation
/// ledger is recorded; an unset or unrecognized value leaves the corresponding mode
/// at its (behavior-preserving) default. Kept here so the meta-core knobs are parsed
/// in one place rather than inline in [`crate::solver::SolverConfig::from_env`].
pub fn apply_env_modes(
    recursion_mode: &mut RecursionMode,
    selection_mode: &mut SelectionMode,
    skill_mode: &mut SkillMode,
) {
    if let Ok(value) = std::env::var("FORMAL_AI_RECURSION_MODE") {
        if let Some(mode) = RecursionMode::from_slug(&value) {
            *recursion_mode = mode;
        }
    }
    if let Ok(value) = std::env::var("FORMAL_AI_SELECTION_MODE") {
        if let Some(mode) = SelectionMode::from_slug(&value) {
            *selection_mode = mode;
        }
    }
    if let Ok(value) = std::env::var("FORMAL_AI_SKILL_MODE") {
        if let Some(mode) = SkillMode::from_slug(&value) {
            *skill_mode = mode;
        }
    }
}
