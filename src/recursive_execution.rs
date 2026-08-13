//! Failure-driven recursive task execution.
//!
//! The meta core already describes a bounded task tree. This module supplies the
//! missing execution back-edge: try the parent, descend only after failure,
//! extend the tool at an irreducible unsupported leaf, then climb back up and
//! retry the parent after every child passes.
//!
//! A caller-provided tree is not the only source of children. A task that fails
//! with no children left to try is asked to *split itself* through
//! [`TaskExecutor::split`], and the resulting sub-tasks are executed by the same
//! protocol. That is what makes the recursion failure-driven rather than
//! plan-driven: the decomposition a run needs is discovered from the failure it
//! actually hit, and splitting continues until either a level passes or the
//! task is irreducible -- which is exactly when extending the tool is the
//! honest move. [`crate::task_decomposition::SplittingExecutor`] wires the
//! repository's real splitter into this hook.
//!
//! Termination is structural, not hopeful. A split may only run while the
//! node's split depth is below the bound, a child that repeats its parent's
//! identity or goal is discarded, and a node whose split yields nothing usable
//! is treated as irreducible. So every path through the protocol reaches a
//! terminal state in a bounded number of attempts.

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
/// `attempt` must run the task and its acceptance test. `split` is called first
/// for a failed task with no children, and may return smaller tasks derived
/// from the failure; returning an empty list declares the task irreducible.
/// `extend_for` is called only for a failed *irreducible* task; returning
/// `true` promises that the tool was extended and the same leaf should be
/// retried once.
pub trait TaskExecutor {
    /// Attempt one task and capture its evidence.
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt;

    /// Extend the general tool for a failed elementary task.
    fn extend_for(&mut self, task: &RecursiveTask, failure: &TaskAttempt) -> bool;

    /// Split a failed childless task into smaller ones, from its failure.
    ///
    /// `split_depth` is how many splits already happened above this node, so an
    /// implementation can spend a shrinking budget; the controller enforces its
    /// own bound regardless. The default refuses to split, which is the honest
    /// answer for an executor that has no splitter: such a task goes straight
    /// to [`TaskExecutor::extend_for`], exactly as before this hook existed.
    fn split(
        &mut self,
        _task: &RecursiveTask,
        _failure: &TaskAttempt,
        _split_depth: u8,
    ) -> Vec<RecursiveTask> {
        Vec::new()
    }
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
    /// Whether this node's children were produced by splitting its own failure
    /// rather than supplied by the caller's tree.
    pub split_applied: bool,
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

    /// How deep the failure-driven splitting went below this node.
    ///
    /// Zero when nothing was split, so a plan-driven run reports zero however
    /// deep the caller's tree was.
    #[must_use]
    pub fn split_depth_reached(&self) -> u8 {
        let below = self
            .children
            .iter()
            .map(Self::split_depth_reached)
            .max()
            .unwrap_or(0);
        below.saturating_add(u8::from(self.split_applied))
    }

    /// Every node that ended [`RecursiveExecution::Blocked`] with no children,
    /// in execution order.
    ///
    /// These are the irreducible tasks: splitting could not shrink them and the
    /// tool could not be extended for them, so they are precisely the places
    /// where new capability has to be added.
    #[must_use]
    pub fn blocked_leaves(&self) -> Vec<&Self> {
        if self.children.is_empty() {
            return if self.is_passed() {
                Vec::new()
            } else {
                vec![self]
            };
        }
        self.children
            .iter()
            .flat_map(Self::blocked_leaves)
            .collect()
    }
}

/// How many times a failure may be split before the controller stops asking.
///
/// A splitter that keeps producing marginally different children would
/// otherwise recurse without end. Four levels is more than the repository's own
/// decomposition artifacts reach and still terminates in bounded work.
pub const DEFAULT_SPLIT_DEPTH_BOUND: u8 = 4;

/// Execute the shrink-on-failure protocol over `root`.
///
/// Every node is attempted before descent. A failed branch executes its smaller
/// children and is retried only if all of them pass. A failed node with no
/// children is asked to split itself from its own failure; the children it
/// yields are executed by this same protocol and the node is retried when they
/// all pass. Only a node that cannot be split -- an irreducible task -- gets one
/// extension opportunity and one retry, which keeps execution bounded.
///
/// Splitting stops at [`DEFAULT_SPLIT_DEPTH_BOUND`]; use
/// [`solve_recursively_within`] to choose another bound.
#[must_use]
pub fn solve_recursively<E: TaskExecutor>(root: &RecursiveTask, executor: &mut E) -> RecursiveRun {
    solve_recursively_within(root, executor, DEFAULT_SPLIT_DEPTH_BOUND)
}

/// [`solve_recursively`] with an explicit bound on failure-driven splitting.
///
/// A bound of zero reproduces the plan-driven protocol exactly: only the
/// caller's tree is executed and a failed leaf goes straight to extension.
#[must_use]
pub fn solve_recursively_within<E: TaskExecutor>(
    root: &RecursiveTask,
    executor: &mut E,
    split_depth_bound: u8,
) -> RecursiveRun {
    execute(root, executor, 0, split_depth_bound)
}

fn execute<E: TaskExecutor>(
    root: &RecursiveTask,
    executor: &mut E,
    split_depth: u8,
    split_depth_bound: u8,
) -> RecursiveRun {
    let first = executor.attempt(root);
    if first.passed {
        return RecursiveRun {
            task: root.clone(),
            attempts: vec![first],
            children: Vec::new(),
            extension_applied: false,
            split_applied: false,
            status: RecursiveExecution::Passed,
        };
    }

    let (supplied, split_applied) = if root.children.is_empty() {
        let split = if split_depth < split_depth_bound {
            usable_split(root, executor.split(root, &first, split_depth))
        } else {
            Vec::new()
        };
        let applied = !split.is_empty();
        (split, applied)
    } else {
        (root.children.clone(), false)
    };

    if supplied.is_empty() {
        let extension_applied = executor.extend_for(root, &first);
        let mut attempts = vec![first];
        if extension_applied {
            attempts.push(executor.attempt(root));
        }
        let status = terminal_state(&attempts);
        return RecursiveRun {
            task: root.clone(),
            attempts,
            children: Vec::new(),
            extension_applied,
            split_applied: false,
            status,
        };
    }

    let child_depth = split_depth.saturating_add(u8::from(split_applied));
    let children = supplied
        .iter()
        .map(|child| execute(child, executor, child_depth, split_depth_bound))
        .collect::<Vec<_>>();
    let mut attempts = vec![first];
    if children.iter().all(RecursiveRun::is_passed) {
        attempts.push(executor.attempt(root));
    }
    let status = terminal_state(&attempts);
    RecursiveRun {
        task: root.clone(),
        attempts,
        children,
        extension_applied: false,
        split_applied,
        status,
    }
}

fn terminal_state(attempts: &[TaskAttempt]) -> RecursiveExecution {
    if attempts.last().is_some_and(|attempt| attempt.passed) {
        RecursiveExecution::Passed
    } else {
        RecursiveExecution::Blocked
    }
}

/// Drop split children that cannot shrink the problem.
///
/// A child that repeats its parent's identity or its goal would be attempted
/// under the same conditions that just failed, so keeping it would trade a
/// bounded protocol for an unbounded one. Discarding it makes the parent
/// irreducible, which routes it to extension -- the honest outcome for a
/// splitter that had nothing to say.
fn usable_split(parent: &RecursiveTask, children: Vec<RecursiveTask>) -> Vec<RecursiveTask> {
    children
        .into_iter()
        .filter(|child| child.id != parent.id && child.goal.trim() != parent.goal.trim())
        .collect()
}
