//! Issue #1550 — the production entry of the recursive procedure: the
//! in-process executor of `solve_recursively`, the answer host every turn of
//! [`crate::solver::UniversalSolver`] runs through, and the per-thread nesting
//! guard and trace. Mirrors `inProcessExecutor`, `solveTurn` and
//! `solveAnswer` of `js/agentic/crate/meta_executor.mjs`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::intent_formalization::IntentFormalizationCache;
use crate::probability::ProbabilityStore;
use crate::solver::ConversationTurn;
use std::cell::{Cell, RefCell};

use super::{
    Draft, MetaHost, ProcedureRun, Request, RunOptions, TaskNode, TraceEvent, execute_procedure,
};
use crate::recursive_execution::{RecursiveTask, TaskAttempt, TaskExecutor, solve_recursively};

/// Issue #1550 step 1: the in-process executor of
/// [`crate::recursive_execution::solve_recursively`].
///
/// One attempt runs the
/// whole procedure on the task's goal; the procedure splits by the grammar
/// itself, so the executor never splits a failure again and never extends a
/// tool. The external-CLI executor of orchestration stays one more executor
/// of the same protocol.
pub struct InProcessExecutor<'h> {
    host: &'h mut dyn MetaHost,
    language: &'h str,
    options: &'h RunOptions,
    run: Option<ProcedureRun>,
}

impl<'h> InProcessExecutor<'h> {
    /// An executor over `host` for requests in `language`.
    pub fn new(host: &'h mut dyn MetaHost, language: &'h str, options: &'h RunOptions) -> Self {
        Self {
            host,
            language,
            options,
            run: None,
        }
    }

    /// The run of the last attempt.
    #[must_use]
    pub fn into_run(self) -> Option<ProcedureRun> {
        self.run
    }
}

impl TaskExecutor for InProcessExecutor<'_> {
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt {
        let run = execute_procedure(
            Request {
                text: &task.goal,
                language: self.language,
            },
            &mut *self.host,
            self.options,
        );
        let evidence = run
            .events
            .last()
            .map(|event| format!("{} {}", event.kind, event.payload))
            .unwrap_or_default();
        let passed = run.outcome.is_resolved();
        self.run = Some(run);
        if passed {
            TaskAttempt::passed(evidence)
        } else {
            TaskAttempt::failed(evidence)
        }
    }

    fn extend_for(&mut self, _task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        false
    }

    fn split(
        &mut self,
        _task: &RecursiveTask,
        _failure: &TaskAttempt,
        _split_depth: u8,
    ) -> Vec<RecursiveTask> {
        Vec::new()
    }
}

/// The single production entry: `solve_recursively` over the turn as the
/// root task, executed in process. Returns the procedure's run.
///
/// # Panics
///
/// Never in practice: `solve_recursively` attempts the root at least once.
pub fn solve_turn(
    request: Request<'_>,
    host: &mut dyn MetaHost,
    options: &RunOptions,
) -> ProcedureRun {
    let mut executor = InProcessExecutor::new(host, request.language, options);
    let _run = solve_recursively(&RecursiveTask::leaf("t0", request.text), &mut executor);
    executor
        .into_run()
        .expect("solve_recursively attempts the root task")
}

/// The host of [`solve_answer`]: the root's direct derivation is the answer
/// the solver already derived; every deeper task is a nested solve.
struct AnswerHost<'f> {
    root: Option<Draft>,
    solve_sub_task: &'f mut dyn FnMut(&str) -> Option<Draft>,
}

impl MetaHost for AnswerHost<'_> {
    fn can_compose(&self, task: &TaskNode) -> bool {
        // A prose join cannot establish a concrete contract such as source
        // code. Mirror solveAnswer's root-intent check in JavaScript.
        task.depth != 0
            || self
                .root
                .as_ref()
                .is_some_and(|root| super::rule_values("unresolved-intent").contains(&root.intent))
    }

    fn draft(&mut self, text: &str, task: &TaskNode) -> Option<Draft> {
        if task.depth == 0 {
            return self.root.clone();
        }
        nested(|| (self.solve_sub_task)(text))
    }
}

/// Every answer goes through the procedure.
///
/// The turn is the root task, its
/// direct derivation is `root` (what the solver already derived), every
/// sub-task of a split request is solved by `solve_sub_task` (nested, so it
/// starts no procedure of its own). Returns `None` inside a nested solve;
/// otherwise the run, whose selected candidate replaces the root answer
/// when least action selects the composition (see [`adopted_composition`]).
/// Mirrors `solveAnswer` of `js/agentic/crate/meta_executor.mjs`.
pub fn solve_answer(
    request: Request<'_>,
    root: Draft,
    solve_sub_task: &mut dyn FnMut(&str) -> Option<Draft>,
    options: &RunOptions,
) -> Option<ProcedureRun> {
    if is_nested() {
        return None;
    }
    let mut host = AnswerHost {
        root: Some(root),
        solve_sub_task,
    };
    Some(solve_turn(request, &mut host, options))
}

/// The composed draft a run selected over the root's direct derivation.
#[must_use]
pub fn adopted_composition(run: &ProcedureRun) -> Option<&Draft> {
    run.outcome
        .candidate
        .as_ref()
        .filter(|candidate| candidate.method == "composition")
        .map(|candidate| &candidate.draft)
}

thread_local! {
    static NESTING: Cell<u32> = const { Cell::new(0) };
    static LAST_TRACE: RefCell<Vec<TraceEvent>> = const { RefCell::new(Vec::new()) };
}

/// Whether the current thread is inside a run of the procedure (a sub-task
/// solve), so the solver does not start a nested run.
#[must_use]
pub fn is_nested() -> bool {
    NESTING.with(|depth| depth.get() > 0)
}

/// Run `body` as a sub-task solve: [`is_nested`] holds while it runs.
pub fn nested<T>(body: impl FnOnce() -> T) -> T {
    struct Leave;
    impl Drop for Leave {
        fn drop(&mut self) {
            NESTING.with(|depth| depth.set(depth.get().saturating_sub(1)));
        }
    }
    NESTING.with(|depth| depth.set(depth.get() + 1));
    let _leave = Leave;
    body()
}

/// Remember the trace of the last outermost turn on this thread.
pub fn remember_trace(events: &[TraceEvent]) {
    LAST_TRACE.with(|trace| *trace.borrow_mut() = events.to_vec());
}

/// The trace of the last outermost turn on this thread.
#[must_use]
pub fn last_turn_trace() -> Vec<TraceEvent> {
    LAST_TRACE.with(|trace| trace.borrow().clone())
}

// Keep the production adapter with its host rather than the dispatch loop.
impl crate::solver::UniversalSolver {
    pub(crate) fn run_meta_procedure(
        &self,
        prompt: &str,
        history: &[ConversationTurn],
        probability_store: &ProbabilityStore,
        answer: &mut SymbolicAnswer,
        log: &mut EventLog,
    ) {
        let draft_of = |solved: &SymbolicAnswer| crate::meta_executor::Draft {
            intent: solved.intent.clone(),
            content: solved.answer.clone(),
            confidence: solved.confidence,
            evidence: solved.evidence_links.clone(),
        };
        let language = crate::language::detect(prompt).slug();
        let mut solve_sub_task = |text: &str| {
            let solved = self.solve_with_history_probability_store_and_intent_cache(
                text,
                history,
                probability_store,
                &mut IntentFormalizationCache::new(),
            );
            Some(draft_of(&solved))
        };
        let Some(run) = crate::meta_executor::solve_answer(
            crate::meta_executor::Request {
                text: prompt,
                language,
            },
            draft_of(&*answer),
            &mut solve_sub_task,
            &crate::solver_helpers::meta_procedure_options(answer),
        ) else {
            return;
        };
        for event in &run.events {
            log.append(event.kind, event.payload.clone());
        }
        crate::meta_executor::remember_trace(&run.events);
        if let Some(composed) = crate::meta_executor::adopted_composition(&run) {
            answer.intent.clone_from(&composed.intent);
            answer.answer.clone_from(&composed.content);
            answer.confidence = composed.confidence;
            answer.evidence_links.clone_from(&composed.evidence);
        }
    }
}
