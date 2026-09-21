//! Incremental decomposition: a task is split because it failed, not because a
//! plan said so.
//!
//! The review on pull request #995 asked for tasks to be attempted whole, split
//! only when they fail, and split again until the pieces are solvable -- with
//! the tool extended only for what stays irreducible. These tests pin that
//! protocol on the repository's own splitter rather than on a fixture one.

use formal_ai::recursive_execution::{
    RecursiveExecution, RecursiveTask, TaskAttempt, TaskExecutor, solve_recursively,
    solve_recursively_within,
};
use formal_ai::task_decomposition::SplittingExecutor;

/// A corpus task the shipped splitter is known to split into two checkable
/// children, used so these tests exercise the real decomposition.
const COMPOUND_TASK: &str = "Add a paths-ignore filter for experiments to release.yml \
     and make docs-changed respect excluded_folders.";

/// An unsplittable task: the splitter returns nothing for it.
const ATOMIC_TASK: &str = "Add dev/log/ to the excluded_folders array.";

/// A tool that can only solve tasks shorter than a limit.
///
/// This is the honest shape of "the task was too big": nothing about the task
/// is unsupported, there is simply more of it than one attempt can carry. The
/// parent passes on its retry because by then its children have done the work.
#[derive(Debug)]
struct SizeLimitedExecutor {
    limit: usize,
    solved: Vec<String>,
    attempted: Vec<String>,
    extensions: Vec<String>,
    extendable: bool,
}

impl SizeLimitedExecutor {
    const fn new(limit: usize) -> Self {
        Self {
            limit,
            solved: Vec::new(),
            attempted: Vec::new(),
            extensions: Vec::new(),
            extendable: false,
        }
    }

    const fn extendable(mut self) -> Self {
        self.extendable = true;
        self
    }

    fn covered_by_children(&self, task: &RecursiveTask) -> bool {
        !self.solved.is_empty()
            && self
                .solved
                .iter()
                .any(|done| done != &task.goal && task.goal.contains(first_words(done)))
    }
}

/// Enough of a child's text to recognise it inside its parent's text.
fn first_words(text: &str) -> &str {
    let cut = text
        .char_indices()
        .filter(|(_, character)| *character == ' ')
        .nth(3)
        .map_or(text.len(), |(index, _)| index);
    &text[..cut]
}

impl TaskExecutor for SizeLimitedExecutor {
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt {
        self.attempted.push(task.goal.clone());
        let solvable = task.goal.len() <= self.limit
            || self.covered_by_children(task)
            || self.extensions.contains(&task.goal);
        if solvable {
            self.solved.push(task.goal.clone());
            TaskAttempt::passed(format!("solved within limit {}", self.limit))
        } else {
            TaskAttempt::failed(format!(
                "task of {} characters exceeds the {} the tool can carry",
                task.goal.len(),
                self.limit
            ))
        }
    }

    fn extend_for(&mut self, task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        if !self.extendable {
            return false;
        }
        self.extensions.push(task.goal.clone());
        true
    }
}

#[test]
fn a_whole_task_is_attempted_first_and_split_only_because_it_failed() {
    let root = RecursiveTask::leaf("root", COMPOUND_TASK);
    let mut executor = SplittingExecutor::new(SizeLimitedExecutor::new(70));

    let run = solve_recursively(&root, &mut executor);

    assert_eq!(run.status, RecursiveExecution::Passed);
    assert!(
        run.split_applied,
        "the root's children must come from its own failure, not from a plan"
    );
    assert_eq!(run.children.len(), 2, "got {:?}", run.children);
    assert_eq!(run.split_depth_reached(), 1);
    assert_eq!(
        run.attempts.len(),
        2,
        "the whole task is attempted before the split and retried after it"
    );
    assert!(!run.attempts[0].passed, "the split needs a real failure");

    let splits = executor.splits();
    assert_eq!(splits.len(), 1, "got {splits:?}");
    assert_eq!(splits[0].goal, COMPOUND_TASK);
    assert_eq!(splits[0].split_depth, 0);
    assert!(
        splits[0].failure_evidence.contains("exceeds"),
        "the recorded split must carry the failure that justified it: {}",
        splits[0].failure_evidence
    );
    assert!(splits[0].is_productive());
    assert_eq!(executor.productive_splits().len(), 1);

    let attempted = &executor.inner().attempted;
    assert_eq!(
        attempted.first().map(String::as_str),
        Some(COMPOUND_TASK),
        "the whole task must be tried before any piece of it"
    );
}

#[test]
fn an_unsplittable_failure_is_reported_irreducible_instead_of_split_forever() {
    let root = RecursiveTask::leaf("root", ATOMIC_TASK);
    let mut executor = SplittingExecutor::new(SizeLimitedExecutor::new(1));

    let run = solve_recursively(&root, &mut executor);

    assert_eq!(run.status, RecursiveExecution::Blocked);
    assert!(!run.split_applied);
    assert_eq!(run.split_depth_reached(), 0);
    let blocked = run.blocked_leaves();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].task.goal, ATOMIC_TASK);

    let splits = executor.splits();
    assert_eq!(splits.len(), 1, "the split must still be recorded");
    assert!(
        !splits[0].is_productive(),
        "an irreducible task is recorded as such, and that is what justifies \
         extending the tool"
    );
    assert!(executor.productive_splits().is_empty());
}

#[test]
fn an_irreducible_failure_is_the_only_thing_that_extends_the_tool() {
    let root = RecursiveTask::leaf("root", COMPOUND_TASK);
    let mut executor = SplittingExecutor::new(SizeLimitedExecutor::new(1).extendable());

    let run = solve_recursively(&root, &mut executor);

    assert_eq!(run.status, RecursiveExecution::Passed);
    assert!(run.split_applied);
    assert!(
        !run.extension_applied,
        "a task that could be split must be split, not patched around"
    );
    assert!(
        run.children.iter().all(|child| child.extension_applied),
        "only the pieces that stayed irreducible may extend the tool: {:?}",
        run.children
            .iter()
            .map(|child| (child.task.goal.clone(), child.extension_applied))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        executor.inner().extensions.len(),
        run.children.len(),
        "one extension per irreducible piece"
    );
}

#[test]
fn a_bound_of_zero_reproduces_the_plan_driven_protocol_exactly() {
    let root = RecursiveTask::leaf("root", COMPOUND_TASK);
    let mut executor = SplittingExecutor::new(SizeLimitedExecutor::new(70));

    let run = solve_recursively_within(&root, &mut executor, 0);

    assert_eq!(run.status, RecursiveExecution::Blocked);
    assert!(run.children.is_empty());
    assert_eq!(run.split_depth_reached(), 0);
    assert!(
        executor.splits().is_empty(),
        "below the bound the splitter is never even asked"
    );
}

/// A splitter that answers a failure with the failing task itself.
struct EchoSplitter;

impl TaskExecutor for EchoSplitter {
    fn attempt(&mut self, _task: &RecursiveTask) -> TaskAttempt {
        TaskAttempt::failed("nothing here can pass")
    }

    fn extend_for(&mut self, _task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        false
    }

    fn split(
        &mut self,
        task: &RecursiveTask,
        _failure: &TaskAttempt,
        _split_depth: u8,
    ) -> Vec<RecursiveTask> {
        vec![RecursiveTask::leaf("echo", task.goal.clone())]
    }
}

#[test]
fn a_split_that_does_not_shrink_the_task_is_refused() {
    let root = RecursiveTask::leaf("root", COMPOUND_TASK);
    let mut executor = EchoSplitter;

    let run = solve_recursively(&root, &mut executor);

    assert_eq!(run.status, RecursiveExecution::Blocked);
    assert!(
        run.children.is_empty(),
        "a child identical to its parent would be attempted under the very \
         conditions that just failed"
    );
    assert_eq!(run.attempts.len(), 1, "and it must not be retried for free");
}
