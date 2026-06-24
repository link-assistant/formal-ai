//! Issue #559: the general recursive meta core, wired as one cohesive pass.
//!
//! Every request flows through the same pipeline before the existing specialized
//! dispatch decides routing and the answer:
//!
//! 1. the explicit, link-serializable problem frame (R330) — the meaning record
//!    made first-class, enumerating every detected need (R7);
//! 2. the recursive, bounded downward decomposition into work units (R332) —
//!    decompose until each leaf is directly solvable (R19);
//! 3. the need-satisfaction ledger (R333) — every detected need carries an
//!    explicit status, a blocked need recorded rather than dropped (R8);
//! 4. the method registry (R331) — the catalogue of handlers each atomic leaf can
//!    route to, derived from the live dispatch constants, so the meta algorithm
//!    can later reason about its own methods;
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
//!    auditable record.
//!
//! The whole pass is **trace-only**: each stage appends Links Notation artifacts
//! to the append-only event log, and none of them changes routing or the produced
//! answer (R13). Selection still flows through the existing dispatch downstream;
//! moving routing onto these artifacts is a later, behavior-changing phase.

use crate::event_log::EventLog;
use crate::intent_formalization::IntentFormalization;
use crate::meta_construction::RecursionMode;

/// Record the full meta core for one formalized prompt as trace events.
///
/// `max_depth` bounds the recursive decomposition so the downward pass always
/// terminates. `mode` selects which recursive directions are reasoned about: the
/// default ([`RecursionMode::Down`]) emits the downward decomposition reasoning
/// only, reproducing the pre-knob trace exactly (R13); `Up`/`Both` additionally
/// emit the upward construction pass. The structural work-unit decomposition
/// events (`work_unit:enter` / `work_unit:exit`) are always emitted regardless of
/// `mode`. This is the single seam the solver loop calls; keeping the stages
/// together here keeps the loop body small and the pipeline cohesive.
pub fn record_meta_core(
    log: &mut EventLog,
    formalization: &IntentFormalization,
    max_depth: u8,
    mode: RecursionMode,
) {
    let problem_frame = crate::meta_frame::record_problem_frame(log, formalization);
    let work_unit_root = crate::meta_frame::record_work_units(log, formalization, max_depth);
    let need_ledger = crate::meta_frame::record_need_ledger(log, &problem_frame, &work_unit_root);
    let method_registry = crate::method_registry::record_method_registry(log);
    if mode.emits_downward() {
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
        mode,
    );
    let _solution_evidence = crate::solution_evidence::record_solution_evidence(
        log,
        &problem_frame,
        &need_ledger,
        &method_registry,
    );
}
