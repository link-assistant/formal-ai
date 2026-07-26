//! Failure-driven recursive task execution.
//!
//! The meta core already describes a bounded task tree. This module supplies the
//! missing execution back-edge: try the parent, descend only after failure,
//! extend the tool at an irreducible unsupported leaf, then climb back up and
//! retry the parent after every child passes.

/// One executable task in a caller-provided decomposition tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursiveTask {
    /// Stable task identity used by evidence and tests.
    pub id: String,
    /// Natural-language task handed to the executor.
    pub goal: String,
    /// Smaller tasks, in composition order.
    pub children: Vec<Self>,
}

impl RecursiveTask {
    /// Construct an elementary task.
    #[must_use]
    pub fn leaf(id: impl Into<String>, goal: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            goal: goal.into(),
            children: Vec::new(),
        }
    }

    /// Construct a task with an explicit, reviewable decomposition.
    #[must_use]
    pub fn branch(id: impl Into<String>, goal: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            id: id.into(),
            goal: goal.into(),
            children,
        }
    }
}

/// Result of one test-backed attempt by the task executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskAttempt {
    /// Whether the task's acceptance test passed.
    pub passed: bool,
    /// Captured test/session evidence or the concrete failure.
    pub evidence: String,
}

impl TaskAttempt {
    /// Record a passing attempt.
    #[must_use]
    pub fn passed(evidence: impl Into<String>) -> Self {
        Self {
            passed: true,
            evidence: evidence.into(),
        }
    }

    /// Record a failing attempt.
    #[must_use]
    pub fn failed(evidence: impl Into<String>) -> Self {
        Self {
            passed: false,
            evidence: evidence.into(),
        }
    }
}

/// Tool boundary used by the recursive controller.
///
/// `attempt` must run the task and its acceptance test. `extend_for` is called
/// only for a failed elementary task; returning `true` promises that the tool
/// was extended and the same leaf should be retried once.
pub trait TaskExecutor {
    /// Attempt one task and capture its evidence.
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt;

    /// Extend the general tool for a failed elementary task.
    fn extend_for(&mut self, task: &RecursiveTask, failure: &TaskAttempt) -> bool;
}

/// Terminal state of one executed task node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecursiveExecution {
    /// Its test passed, directly or after recursion.
    Passed,
    /// It remained unsolved after the bounded recovery protocol.
    Blocked,
}

/// Complete evidence tree for a failure-driven recursive run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursiveRun {
    /// Task represented by this node.
    pub task: RecursiveTask,
    /// Attempts made at this level, in time order.
    pub attempts: Vec<TaskAttempt>,
    /// Executed child runs, in composition order.
    pub children: Vec<Self>,
    /// Whether an elementary failure caused a tool extension.
    pub extension_applied: bool,
    /// Terminal state.
    pub status: RecursiveExecution,
}

impl RecursiveRun {
    /// Whether this node passed its acceptance test.
    #[must_use]
    pub const fn is_passed(&self) -> bool {
        matches!(self.status, RecursiveExecution::Passed)
    }

    /// Count recursively executed leaves.
    #[must_use]
    pub fn executed_leaf_count(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().map(Self::executed_leaf_count).sum()
        }
    }
}

/// Execute the shrink-on-failure protocol over `root`.
///
/// The tree is caller-provided so decomposition stays explicit and auditable.
/// Every node is attempted before descent. A failed branch executes its smaller
/// children and is retried only if all of them pass. A failed leaf gets one
/// extension opportunity and one retry, which keeps execution bounded.
#[must_use]
pub fn solve_recursively<E: TaskExecutor>(root: &RecursiveTask, executor: &mut E) -> RecursiveRun {
    let first = executor.attempt(root);
    if first.passed {
        return RecursiveRun {
            task: root.clone(),
            attempts: vec![first],
            children: Vec::new(),
            extension_applied: false,
            status: RecursiveExecution::Passed,
        };
    }

    if root.children.is_empty() {
        let extension_applied = executor.extend_for(root, &first);
        let mut attempts = vec![first];
        if extension_applied {
            attempts.push(executor.attempt(root));
        }
        let status = if attempts.last().is_some_and(|attempt| attempt.passed) {
            RecursiveExecution::Passed
        } else {
            RecursiveExecution::Blocked
        };
        return RecursiveRun {
            task: root.clone(),
            attempts,
            children: Vec::new(),
            extension_applied,
            status,
        };
    }

    let children = root
        .children
        .iter()
        .map(|child| solve_recursively(child, executor))
        .collect::<Vec<_>>();
    let mut attempts = vec![first];
    if children.iter().all(RecursiveRun::is_passed) {
        attempts.push(executor.attempt(root));
    }
    let status = if attempts.last().is_some_and(|attempt| attempt.passed) {
        RecursiveExecution::Passed
    } else {
        RecursiveExecution::Blocked
    };
    RecursiveRun {
        task: root.clone(),
        attempts,
        children,
        extension_applied: false,
        status,
    }
}
