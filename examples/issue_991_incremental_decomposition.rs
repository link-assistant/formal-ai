//! Watch a task get split because it failed, not because a plan said so.
//!
//! Usage: `cargo run --example issue_991_incremental_decomposition -- "<task>" [limit]`
//!
//! The simulated tool here can only carry a task up to `limit` characters, and
//! counts work its pieces already did. That is the shape of a real agent CLI
//! hitting its own limit: nothing is unsupported, there is simply more of the
//! task than one session can do. `formal-ai agent dispatch --incremental` runs
//! this exact protocol against real CLIs; this example makes the control flow
//! visible without spawning any.

use formal_ai::recursive_execution::{
    solve_recursively, RecursiveRun, RecursiveTask, TaskAttempt, TaskExecutor,
};
use formal_ai::task_decomposition::SplittingExecutor;

struct SizeLimitedTool {
    limit: usize,
    solved: Vec<String>,
}

impl TaskExecutor for SizeLimitedTool {
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt {
        let helped_by_pieces = self
            .solved
            .iter()
            .any(|done| done != &task.goal && task.goal.contains(done.trim_end_matches('.')));
        if task.goal.len() <= self.limit || helped_by_pieces {
            self.solved.push(task.goal.clone());
            TaskAttempt::passed(format!("done ({} characters)", task.goal.len()))
        } else {
            TaskAttempt::failed(format!(
                "{} characters exceeds the {} this tool can carry",
                task.goal.len(),
                self.limit
            ))
        }
    }

    fn extend_for(&mut self, _task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        false
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let task = args.next().unwrap_or_else(|| {
        "Add a paths-ignore filter for experiments to release.yml \
         and make docs-changed respect excluded_folders."
            .to_owned()
    });
    let limit: usize = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(70);

    let root = RecursiveTask::leaf("root", task);
    let mut executor = SplittingExecutor::new(SizeLimitedTool {
        limit,
        solved: Vec::new(),
    });

    let run = solve_recursively(&root, &mut executor);

    print_run(&run, 0);
    println!("\nsolved: {}", run.is_passed());
    println!("split depth reached: {}", run.split_depth_reached());
    for split in executor.splits() {
        println!(
            "\nsplit at depth {} because: {}",
            split.split_depth, split.failure_evidence
        );
        if split.children.is_empty() {
            println!("  (irreducible: the splitter had nothing smaller to offer)");
        }
        for child in &split.children {
            println!("  -> {child}");
        }
    }
    for blocked in run.blocked_leaves() {
        println!("\nblocked: {}", blocked.task.goal);
    }
}

fn print_run(run: &RecursiveRun, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{indent}{} :: {}", marker(run), run.task.goal);
    for attempt in &run.attempts {
        println!(
            "{indent}  attempt passed={} evidence={}",
            attempt.passed, attempt.evidence
        );
    }
    for child in &run.children {
        print_run(child, depth + 1);
    }
}

const fn marker(run: &RecursiveRun) -> &'static str {
    if run.is_passed() {
        "PASS"
    } else {
        "BLOCKED"
    }
}
