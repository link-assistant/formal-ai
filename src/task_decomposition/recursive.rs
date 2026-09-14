//! The repository's splitter, wired into the recursive controller's split hook.
//!
//! [`crate::recursive_execution`] asks a failed childless task to split itself.
//! This adapter answers that question with [`super::decompose_task`], so
//! failure-driven execution and reviewable decomposition are the same algorithm
//! seen from two sides rather than two separate implementations.
//!
//! One level per split, deliberately. Handing the controller a whole subtree
//! would commit to a plan before a single child has been attempted, which is the
//! plan-driven shape this hook exists to replace. Splitting one level at a time
//! means every deeper split is justified by a failure that actually happened,
//! and the controller's own depth bound remains the single place that stops the
//! recursion.

use std::collections::BTreeSet;

use crate::recursive_execution::{RecursiveTask, TaskAttempt, TaskExecutor};

use super::{SubTask, TaskStrategyLedger, decompose_task_with_ledger};

/// One split the controller requested and this adapter answered.
///
/// Recorded so a run can be explained after the fact: which failure forced a
/// split, how deep it was, and what the splitter proposed. An empty `children`
/// records that the splitter declared the task irreducible, which is the
/// evidence that extending the tool was the honest next move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedSplit {
    /// Identity of the task that failed.
    pub task_id: String,
    /// The failed task's goal, as handed to the splitter.
    pub goal: String,
    /// Evidence from the failing attempt that triggered the split.
    pub failure_evidence: String,
    /// How many splits already happened above this task.
    pub split_depth: u8,
    /// Goals of the sub-tasks the splitter produced, in composition order.
    pub children: Vec<String>,
}

impl RecordedSplit {
    /// Did the splitter shrink the task, rather than declare it irreducible?
    #[must_use]
    pub const fn is_productive(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Adapter that gives any [`TaskExecutor`] the repository's real splitter.
///
/// `attempt`, `retry_after_children`, and `extend_for` are delegated untouched:
/// this type adds the ability to shrink a failure, and changes nothing about
/// how a task is run or how the tool is extended.
#[derive(Debug, Clone)]
pub struct SplittingExecutor<E> {
    inner: E,
    ledger: TaskStrategyLedger,
    splits: Vec<RecordedSplit>,
    /// Atomicity decisions from the exact one-level trees this run inspected.
    /// `RecursiveTask` carries only goals and children, so the adapter retains
    /// this part of each emitted leaf's contract until that leaf is attempted.
    atomic_tasks: BTreeSet<(String, String)>,
}

impl<E> SplittingExecutor<E> {
    /// Wrap `inner` with the shipped, review-gated strategy ledger.
    #[must_use]
    pub fn new(inner: E) -> Self {
        Self::with_ledger(inner, TaskStrategyLedger::shipped())
    }

    /// Wrap `inner` with an explicit strategy ledger.
    ///
    /// An empty ledger restricts splitting to what the seed lexicon can observe
    /// directly, which is what a run that means to *learn* a missing strategy
    /// has to start from.
    #[must_use]
    pub const fn with_ledger(inner: E, ledger: TaskStrategyLedger) -> Self {
        Self {
            inner,
            ledger,
            splits: Vec::new(),
            atomic_tasks: BTreeSet::new(),
        }
    }

    /// Every split this executor answered, in request order.
    #[must_use]
    pub fn splits(&self) -> &[RecordedSplit] {
        &self.splits
    }

    /// Splits that actually shrank a task.
    #[must_use]
    pub fn productive_splits(&self) -> Vec<&RecordedSplit> {
        self.splits
            .iter()
            .filter(|split| split.is_productive())
            .collect()
    }

    /// Borrow the wrapped executor.
    pub const fn inner(&self) -> &E {
        &self.inner
    }

    /// Recover the wrapped executor.
    #[must_use]
    pub fn into_inner(self) -> E {
        self.inner
    }
}

impl<E: TaskExecutor> TaskExecutor for SplittingExecutor<E> {
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt {
        self.inner.attempt(task)
    }

    fn extend_for(&mut self, task: &RecursiveTask, failure: &TaskAttempt) -> bool {
        self.inner.extend_for(task, failure)
    }

    fn retry_after_children(&mut self, task: &RecursiveTask) -> TaskAttempt {
        self.inner.retry_after_children(task)
    }

    fn split(
        &mut self,
        task: &RecursiveTask,
        failure: &TaskAttempt,
        split_depth: u8,
    ) -> Vec<RecursiveTask> {
        if self
            .atomic_tasks
            .iter()
            .any(|(id, goal)| id == &task.id && goal == &task.goal)
        {
            self.splits.push(RecordedSplit {
                task_id: task.id.clone(),
                goal: task.goal.clone(),
                failure_evidence: failure.evidence.clone(),
                split_depth,
                children: Vec::new(),
            });
            return Vec::new();
        }
        let decomposition = decompose_task_with_ledger(&task.goal, 1, &self.ledger);
        self.atomic_tasks.extend(
            decomposition
                .root
                .children
                .iter()
                .filter(|child| child.atomic)
                .map(|child| (child.id.clone(), child.text.clone())),
        );
        let children: Vec<RecursiveTask> = decomposition
            .root
            .children
            .iter()
            .map(SubTask::to_recursive_task)
            .collect();
        self.splits.push(RecordedSplit {
            task_id: task.id.clone(),
            goal: task.goal.clone(),
            failure_evidence: failure.evidence.clone(),
            split_depth,
            children: children.iter().map(|child| child.goal.clone()).collect(),
        });
        children
    }
}
