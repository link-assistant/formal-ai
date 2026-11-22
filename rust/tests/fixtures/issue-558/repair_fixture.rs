//! Worked repair records for issue #558 tests.
#![allow(dead_code)]

use formal_ai::event_log::EventLog;
use formal_ai::{
    BenchmarkGateReport, ChangeRequest, HumanApproval, LearningLedger, RebuildPlan, RepairCase,
    RepairStrategy, SourceRoundTrip, UnknownTrace,
};

pub const SELF_HEAL_TASK: &str = "When you cannot answer an input, run your self-healing loop: reason about the \
     failure, map it onto the source that would change with a source-to-links \
     round-trip, learn a benchmark-gated lesson, and record the repair case in \
     Links Notation for human approval.";

pub const CHANGE_TASK: &str = "I want to change Formal AI itself: please add a new capability to the system, and \
     route my request through the same human-gated review loop that produces a \
     requirement, a test, and a patch plan I can review as a pull request.";

pub const LEDGER_TASK: &str = "Promote the approved lesson into your learning ledger and record the approved \
     learning record in Links Notation so a repeated failure is answered from the \
     ledger next time.";

pub const REPAIR_STRATEGY_TASK: &str = "Classify a failure and decide which part to repair: given a failure trace the \
     system could not answer, determine whether the repair is a solver method, a data \
     record, or a test change, then compose the grounded, human-gated repair strategy \
     for it — for every class of failure, not just one.";

pub const REBUILD_TASK: &str = "An improvement to Formal AI was accepted — now rebuild it and reattach the improved \
     WebAssembly worker to the UI: give me the ordered, reversible steps to regenerate \
     the worker, reattach it to the browser UI, hot-swap the local server, and verify \
     the UI uses the accepted version.";

pub fn pinned_round_trip() -> SourceRoundTrip {
    use formal_ai::agentic_coding::self_ast;
    static ROUND_TRIP: std::sync::OnceLock<SourceRoundTrip> = std::sync::OnceLock::new();
    ROUND_TRIP
        .get_or_init(|| {
            SourceRoundTrip::for_module(self_ast::TARGET_MODULE_PATH, self_ast::target_source())
        })
        .clone()
}

pub fn canonical_ledger() -> LearningLedger {
    LearningLedger::from_approved_lessons_seed(include_str!("approved-lessons.lino")).unwrap()
}

/// A deterministic, canonical failure trace for the self-healing recipe.
///
/// It captures an input the system could not answer directly (a reverse-sort
/// program modifier), together with the rule-synthesis candidate and *passed*
/// verification the loop reasons about. Kept in sync with the real events
/// `crate::rule_synthesis` emits.
#[must_use]
pub fn canonical_failure_trace() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "selected_rule",
        "initial unknown reason no_seed_route next try_rule_synthesis",
    );
    log.append(
        "rule_synthesis_candidate",
        "rule_synthesis_candidate\n  id reverse_sort_list_files\n  source constructed_from_operation_vocabulary\n  base_task list_files\n  modifier reverse_sort\n  operation sort\n  operation_modifier descending\n  target program:last.output_order\n  resolved_task list_files_reverse_sort",
    );
    log.append(
        "rule_verification",
        "rule_verification\n  candidate reverse_sort_list_files\n  fixture list_files_output_order\n  input a.txt,b.txt,c.txt\n  expected_order c.txt,b.txt,a.txt\n  lowering_check passed\n  render_check passed\n  status passed",
    );
    UnknownTrace::from_event_log(
        "List the files but sort the results in reverse order",
        "write_program",
        &log,
    )
    .expect("canonical unknown-path trace should accumulate")
}

/// Build the canonical, fully-worked self-healing case.
///
/// The canonical failure is mapped onto the pinned planner target and gated by a
/// passing issue-#362 benchmark, so the outcome is `AwaitingReview` (a lesson
/// ready for human approval). Used by the
/// historical fixture and its replay tests.
#[must_use]
pub fn canonical_case() -> RepairCase {
    RepairCase::from_trace(
        canonical_failure_trace(),
        pinned_round_trip(),
        BenchmarkGateReport::issue_362_from_counts(4, 0),
    )
}

/// A canonical *solver-method* failure trace.
///
/// The meta algorithm had no route to resolve a reverse-sort program modifier and fell
/// back to rule synthesis. Mirrors the self-healing module's failure-trace signals.
#[must_use]
pub fn canonical_solver_method_failure() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "selected_rule",
        "initial unknown reason no_seed_route next try_rule_synthesis",
    );
    log.append(
        "rule_synthesis_candidate",
        "rule_synthesis_candidate id reverse_sort_list_files base_task list_files modifier reverse_sort",
    );
    UnknownTrace::new(
        "List the files but sort the results in reverse order",
        log.events().to_vec(),
    )
}

/// A canonical *data-record* failure trace: the task was understood but a surface the
/// answer needed was missing from the seed data (an issue-#538-style meaning gap).
#[must_use]
pub fn canonical_data_record_failure() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "reasoning:unknown",
        "resolved_meaning tomato but missing surface for the plural form in the lexicon",
    );
    log.append(
        "lexicon_lookup",
        "lexicon gap: no seed data record carries the requested surface",
    );
    UnknownTrace::new(
        "What is the plural of the tomato meaning's second synonym?",
        log.events().to_vec(),
    )
}

/// A canonical *test* failure trace: a previously-passing behaviour regressed and no
/// guard test pinned it.
#[must_use]
pub fn canonical_test_failure() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "reasoning:unknown",
        "answer diverged from a previously passing result and the behaviour is unguarded",
    );
    log.append(
        "regression_check",
        "regression: no test pins this behaviour so the change slipped through",
    );
    UnknownTrace::new(
        "Why did the reverse-sort answer change between releases?",
        log.events().to_vec(),
    )
}

/// The three canonical strategies, one per repair target, in a stable order.
///
/// Deterministic and self-contained — used by the agentic recipe, the example, and the
/// tests to prove the classifier covers every failure class.
#[must_use]
pub fn canonical_strategies() -> Vec<RepairStrategy> {
    vec![
        RepairStrategy::classify(&canonical_solver_method_failure()),
        RepairStrategy::classify(&canonical_data_record_failure()),
        RepairStrategy::classify(&canonical_test_failure()),
    ]
}

/// Build the canonical, fully-worked change request.
///
/// A concrete user request ("let users ask Formal AI to reverse-sort program output")
/// targeting the deterministic planner — the module that would route the new
/// capability. Deterministic and self-contained — used by the agentic recipe, the
/// example, and the tests.
#[must_use]
pub fn canonical_change_request() -> ChangeRequest {
    ChangeRequest::for_module(
        "Please add a new capability to Formal AI: let users ask it to reverse-sort a program's output.",
        "src/agentic_coding/planner.rs",
    )
}

/// The canonical rebuild-and-reattach plan: the plan for the canonical accepted change,
/// used by the agentic recipe, the example, and the tests.
///
/// The canonical change is accepted through the same human gate the change request and
/// learning ledger enforce — a green benchmark gate *and* an explicit approval — so the
/// rebuild plan can only ever follow an approved change.
#[must_use]
pub fn canonical_rebuild_plan() -> RebuildPlan {
    let request = canonical_change_request();
    let green = BenchmarkGateReport::issue_362_from_counts(4, 0);
    let accepted = request
        .review(&green, &HumanApproval::granted("maintainer"))
        .expect("the canonical change is green and approved, so it accepts");
    RebuildPlan::for_accepted_change(&accepted)
}

pub fn task_with_json(cue: &str, value: serde_json::Value) -> String {
    format!("{cue}\n```json\n{value}\n```")
}

pub fn trace_value(trace: UnknownTrace) -> serde_json::Value {
    serde_json::json!({
        "prompt": trace.prompt,
        "events": trace.events.iter().map(|event| serde_json::json!({
            "kind": event.kind, "payload": event.payload,
        })).collect::<Vec<_>>(),
    })
}
