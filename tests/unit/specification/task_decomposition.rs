//! Task-decomposition specifications (issue #847).
//!
//! Decomposition is itself a task the engine must be able to do: splitting a
//! task into sub-tasks and judging whether a task is atomic are two views of
//! one recursion. Each test below pins one line of the issue's acceptance
//! checklist, and the multilingual tests pin it in all four supported
//! languages so recognition can only come from the seed lexicon (issue #386).

use formal_ai::recursive_execution::{RecursiveTask, TaskAttempt, TaskExecutor, solve_recursively};
use formal_ai::task_decomposition::{
    CONTRACT_LINO, Decomposition, TaskLearningApproval, TaskLearningGate, TaskStrategyLedger,
    TaskStrategyProposal, decompose_task, decompose_task_with_ledger, is_checkable,
    split_once_checkable, task_decomposition_contract,
};
use formal_ai::{ExecutionSurface, SolverConfig, UniversalSolver};

/// A task with two independent halves, in each supported language. Every
/// variant names two observable edits joined by that language's "and".
const COMPOSITE_TASKS: [(&str, &str); 4] = [
    (
        "en",
        "Add the flag to release.yml and update the changelog.",
    ),
    ("ru", "Добавь флаг в release.yml и обнови changelog."),
    ("hi", "release.yml में फ़्लैग जोड़ें और changelog अपडेट करें।"),
    ("zh", "在 release.yml 中添加标志并更新 changelog。"),
];

/// "Split this task into subtasks", asked in each supported language.
const SPLIT_PROMPTS: [(&str, &str); 4] = [
    (
        "en",
        "Split this task into subtasks: 'Add the flag to release.yml and update the changelog.'",
    ),
    (
        "ru",
        "Разбей задачу на подзадачи: «Добавь флаг в release.yml и обнови changelog.»",
    ),
    (
        "hi",
        "इस कार्य को उपकार्यों में विभाजित करें: 'release.yml में फ़्लैग जोड़ें और changelog अपडेट करें।'",
    ),
    (
        "zh",
        "把这个任务拆分成子任务：“在 release.yml 中添加标志并更新 changelog。”",
    ),
];

/// "Is this task atomic?", asked in each supported language about a task that
/// is a single observable edit.
const ATOMICITY_PROMPTS: [(&str, &str); 4] = [
    (
        "en",
        "Is this task atomic? 'Add dev/log/ to the excluded_folders array.'",
    ),
    (
        "ru",
        "Эта задача атомарная? «Добавь dev/log/ в массив excluded_folders.»",
    ),
    (
        "hi",
        "क्या यह कार्य अविभाज्य है? 'excluded_folders सूची में dev/log/ जोड़ें।'",
    ),
    (
        "zh",
        "这个任务是原子的吗？“在 excluded_folders 数组中添加 dev/log/。”",
    ),
];

fn decomposition_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

#[test]
fn agent_authored_contract_is_exact_and_controls_the_shipped_ledger() {
    let evidence = include_str!(
        "../../../docs/case-studies/issue-847/self-hosting-authorship/task-decomposition-invariant.lino"
    );
    // The evidence file is the record of what the agent authored in #847, so it
    // is never rewritten to match later work — doing that would falsify the
    // thing it exists to prove. What must hold is that every clause the agent
    // wrote is still shipped verbatim; a clause added afterwards (the `binary`
    // rule from #1028) is additional, and is asserted on its own below.
    for clause in evidence
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        assert!(
            CONTRACT_LINO
                .lines()
                .any(|shipped| shipped.trim() == clause),
            "the shipped contract dropped a clause the agent authored in #847: {clause}"
        );
    }

    let contract = task_decomposition_contract().expect("the embedded contract must be complete");
    assert!(!contract.atomic.is_empty());
    assert!(!contract.execution.is_empty());
    assert!(!contract.learning.is_empty());
    // Issue #1028: decomposition is a *binary* split, and the contract is what
    // makes that a shipped rule rather than a convention the runner happens to
    // follow. A parse that lost the field would return `None` above, so this
    // asserts the clause is actually populated.
    assert!(
        !contract.binary.is_empty(),
        "the binary-split rule must be part of the shipped contract"
    );
    assert_eq!(
        TaskStrategyLedger::shipped().approved_strategy_ids(),
        ["task_strategy_verified_change"]
    );
}

/// Acceptance: recognised in every supported language, never the unknown
/// fallback.
#[test]
fn splitting_is_recognised_in_every_supported_language() {
    let solver = decomposition_solver();
    for (language, prompt) in SPLIT_PROMPTS {
        let response = solver.solve(prompt);
        assert_eq!(
            response.intent, "task_decomposition",
            "{language}: expected the decomposition intent, got {} for {prompt}",
            response.intent
        );
    }
}

/// Acceptance: "Is this task atomic?" is answerable standalone, in every
/// supported language.
#[test]
fn atomicity_is_recognised_in_every_supported_language() {
    let solver = decomposition_solver();
    for (language, prompt) in ATOMICITY_PROMPTS {
        let response = solver.solve(prompt);
        assert_eq!(
            response.intent, "task_atomicity",
            "{language}: expected the atomicity intent for {prompt}"
        );
        assert!(
            !response.answer.trim().is_empty(),
            "{language}: the atomicity answer must not be empty"
        );
    }
}

/// Acceptance: the result is a list of children, each with an observable
/// completion criterion.
#[test]
fn every_child_has_an_observable_completion_criterion() {
    for (language, task) in COMPOSITE_TASKS {
        let decomposition = decompose_task(task, 4);
        assert!(
            !decomposition.is_atomic(),
            "{language}: a two-edit task must split"
        );
        let leaves = decomposition.leaves();
        assert!(
            leaves.len() >= 2,
            "{language}: expected at least two children, got {}",
            leaves.len()
        );
        for leaf in leaves {
            assert!(
                is_checkable(&leaf.text),
                "{language}: child without an observable completion criterion: {}",
                leaf.text
            );
        }
    }
}

/// The issue names "Understand the codebase" as exactly the child a
/// decomposition must never emit: it has no observable completion criterion.
#[test]
fn an_unobservable_fragment_never_becomes_a_child() {
    assert!(!is_checkable("Understand the codebase"));
    assert!(!is_checkable("Изучи кодовую базу"));

    let decomposition = decompose_task("Understand the codebase and fix the bug.", 4);
    for leaf in decomposition.leaves() {
        assert!(
            !leaf.text.eq_ignore_ascii_case("understand the codebase"),
            "an unobservable fragment must be merged, not emitted as a child"
        );
    }
}

/// Acceptance: the recursion terminates — every leaf is atomic, or the depth
/// bound was reached and is reported rather than hidden.
#[test]
fn recursion_terminates_with_atomic_leaves_or_a_reported_depth_bound() {
    // Two sentences, the first of which splits again — so a depth of 1 really
    // does cut the recursion short rather than merely stopping where it would
    // have stopped anyway.
    let task =
        "Fix the failing test and update the changelog. Open a pull request and announce it.";
    let bounded = decompose_task(task, 1);
    assert!(
        bounded.depth_bound_reached(),
        "a depth of 1 must cut this task short"
    );
    assert!(
        bounded
            .numbered_lines("[cut]")
            .iter()
            .any(|line| line.contains("[cut]")),
        "a depth bound must be reported in the rendered lines"
    );

    let unbounded = decompose_task(task, 6);
    assert!(
        !unbounded.depth_bound_reached(),
        "a generous depth must reach atomic leaves: {:?}",
        unbounded.numbered_lines("[cut]")
    );
    for leaf in unbounded.leaves() {
        assert!(
            leaf.atomic,
            "every leaf of a terminated recursion is atomic: {}",
            leaf.text
        );
    }
}

/// The depth bound is configuration, not a constant: lowering
/// `max_decomposition_depth` must be visible in the answer.
#[test]
fn the_configured_depth_bound_is_reported_in_the_answer() {
    let prompt = "Split this task into subtasks: 'Fix the failing test and update the changelog. Open a pull request and announce it.'";
    let shallow = UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        max_decomposition_depth: 1,
        ..SolverConfig::default()
    })
    .solve(prompt);
    assert!(
        shallow.answer.contains("depth"),
        "a truncated recursion must say so, got: {}",
        shallow.answer
    );

    let deep = decomposition_solver().solve(prompt);
    assert!(
        !deep.answer.contains("[depth bound reached]"),
        "the default depth reaches atomic leaves for this task, got: {}",
        deep.answer
    );
}

/// Acceptance: every sub-task is a first-class inspectable `sub_impulse:`
/// event, reusing the existing structure rather than a parallel one.
#[test]
fn every_sub_task_surfaces_as_a_sub_impulse_event() {
    let response = decomposition_solver().solve(
        "Split this task into subtasks: 'Add the flag to release.yml and update the changelog.'",
    );
    let sub_impulses: Vec<&String> = response
        .evidence_links
        .iter()
        .filter(|link| link.starts_with("sub_impulse"))
        .collect();
    assert!(
        sub_impulses.len() >= 2,
        "each sub-task must be inspectable as a sub_impulse event, got {sub_impulses:?}"
    );
    assert!(
        response.links_notation.contains("sub_task"),
        "the sub-tasks must be present in the links notation record"
    );
}

/// Acceptance: deterministic — the same task and configuration always produce
/// the same decomposition.
#[test]
fn decomposition_is_deterministic_for_a_given_config() {
    let prompt = "Split this task into subtasks: 'Fix the failing test, update the changelog and open a pull request.'";
    let first = decomposition_solver().solve(prompt);
    let second = decomposition_solver().solve(prompt);
    assert_eq!(first.answer, second.answer);
    assert_eq!(first.links_notation, second.links_notation);

    let structural = decompose_task(prompt, 4);
    assert_eq!(
        structural.to_links_notation(),
        decompose_task(prompt, 4).to_links_notation()
    );
}

/// Acceptance: the whole spectrum, from a GitHub issue down to a single atomic
/// edit. The bottom of the ladder must report itself as atomic — that is the
/// recursion's base case.
#[test]
fn the_spectrum_runs_from_issue_to_atomic_edit() {
    let issue = decompose_task("Solve issue 843 and open a pull request.", 4);
    assert!(!issue.is_atomic(), "an issue-level task still splits");

    let edit = decompose_task(
        "In the file scripts/detect-code-changes.rs, add dev/log/ to the excluded_folders array.",
        4,
    );
    assert!(
        edit.is_atomic(),
        "a single-file single-edit task is the bottom of the ladder: {:?}",
        edit.numbered_lines("[cut]")
    );
    assert_eq!(edit.root.reason.slug(), "direct_method");
}

/// A real task taken from this repository's own corpus: the issue that asked
/// for `experiments/` to stop triggering the code-change detector. A human
/// reading the two children agrees they are smaller than the parent and that
/// doing both is doing the parent.
#[test]
fn a_real_corpus_task_splits_into_smaller_jointly_sufficient_children() {
    let decomposition = decompose_task(
        "Add a paths-ignore filter for experiments to release.yml and make docs-changed respect excluded_folders.",
        4,
    );
    let leaves = decomposition.leaves();
    assert_eq!(
        leaves.len(),
        2,
        "got {:?}",
        decomposition.numbered_lines("")
    );
    assert!(leaves[0].text.contains("release.yml"));
    assert!(leaves[1].text.contains("excluded_folders"));
    for leaf in leaves {
        assert!(
            leaf.text.len() < decomposition.task.len(),
            "a child must be smaller than its parent: {}",
            leaf.text
        );
        assert!(is_checkable(&leaf.text));
    }
}

/// A task with nothing to split off is the base case, not an error: the
/// splitter returns no parts rather than inventing a second child.
#[test]
fn an_atomic_task_yields_no_split() {
    assert!(split_once_checkable("Add dev/log/ to the excluded_folders array.").is_empty());
}

/// Regression found while reviewing issue #847 after the arbitrary-procedure
/// framework landed: punctuation is not an atomicity oracle. A one-clause
/// repository issue still contains requirement, regression, implementation,
/// and verification work, so treating its single verb as a directly
/// executable leaf is a false green.
#[test]
fn a_single_clause_issue_is_not_misreported_as_an_atomic_operation() {
    let task = "Implement task decomposition as a first-class working task for \
                https://github.com/link-assistant/formal-ai/issues/847";
    assert!(
        !is_checkable(task),
        "an observable verb without a concrete operation contract is not independently checkable"
    );

    let decomposition = decompose_task(task, 6);
    assert!(
        !decomposition.is_atomic(),
        "issue-sized work needs a learned plan even when its prose has one clause"
    );
    assert!(
        decomposition.leaves().len() >= 3,
        "the issue must reduce through multiple independently checkable leaves: {:?}",
        decomposition.numbered_lines("[cut]")
    );
    assert!(
        decomposition
            .to_links_notation()
            .contains("completion_criterion"),
        "every planned child must expose its verification contract"
    );
    assert!(
        decomposition
            .leaves()
            .iter()
            .all(|leaf| leaf.is_independently_checkable()),
        "an approved strategy must reduce the issue to contracted leaves"
    );
}

/// Reaching the recursion guard creates an unresolved leaf, not an atomic one.
/// `children.is_empty()` and `atomic` are intentionally different facts.
#[test]
fn a_depth_bounded_unsolved_root_is_not_reported_as_atomic() {
    let bounded = decompose_task(
        "Implement task decomposition for https://github.com/link-assistant/formal-ai/issues/847",
        0,
    );
    assert!(bounded.depth_bound_reached());
    assert!(
        !bounded.is_atomic(),
        "the depth guard cannot certify work it did not inspect"
    );
    assert_eq!(bounded.root.reason.slug(), "depth_bound");
}

#[test]
fn the_inspected_tree_round_trips_and_changed_artifacts_are_rejected() {
    let decomposition = decompose_task(
        "Implement task decomposition for \
         https://github.com/link-assistant/formal-ai/issues/847",
        6,
    );
    let artifact = decomposition.to_links_notation();
    let restored = Decomposition::from_links_notation(&artifact)
        .expect("an unchanged decomposition artifact should round-trip");
    assert_eq!(restored, decomposition);

    let tampered = artifact.replacen(
        "requirements_are_independently_checkable",
        "requirements_look_plausible",
        1,
    );
    assert!(
        Decomposition::from_links_notation(&tampered).is_err(),
        "a changed completion contract must invalidate the inspected artifact"
    );
}

#[derive(Default)]
struct RecordingExecutor {
    attempted_ids: Vec<String>,
    root_attempts: usize,
}

impl TaskExecutor for RecordingExecutor {
    fn attempt(&mut self, task: &RecursiveTask) -> TaskAttempt {
        self.attempted_ids.push(task.id.clone());
        if !task.children.is_empty() {
            self.root_attempts += 1;
            if self.root_attempts == 1 {
                return TaskAttempt::failed("whole-task acceptance check failed");
            }
        }
        TaskAttempt::passed(format!("{} acceptance check passed", task.id))
    }

    fn extend_for(&mut self, _task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        false
    }
}

#[test]
fn recursive_execution_reuses_the_exact_inspected_tree() {
    let inspected = decompose_task(
        "Implement task decomposition for \
         https://github.com/link-assistant/formal-ai/issues/847",
        6,
    );
    let restored = Decomposition::from_links_notation(&inspected.to_links_notation())
        .expect("the inspected artifact should be executable");
    let expected_leaf_ids = restored
        .leaves()
        .iter()
        .map(|leaf| leaf.id.clone())
        .collect::<Vec<_>>();
    let executable = restored.to_recursive_task();
    let mut executor = RecordingExecutor::default();

    let run = solve_recursively(&executable, &mut executor);

    assert!(run.is_passed());
    assert_eq!(run.executed_leaf_count(), expected_leaf_ids.len());
    assert_eq!(
        &executor.attempted_ids[1..=expected_leaf_ids.len()],
        expected_leaf_ids,
        "execution must consume the reviewed child identities without re-splitting"
    );
    assert_eq!(
        executor.attempted_ids.last(),
        Some(&restored.root.id),
        "the same parent must be retried after its children pass"
    );
}

#[derive(Default)]
struct BlockingExecutor;

impl TaskExecutor for BlockingExecutor {
    fn attempt(&mut self, _task: &RecursiveTask) -> TaskAttempt {
        TaskAttempt::failed("missing executable operation contract")
    }

    fn extend_for(&mut self, _task: &RecursiveTask, _failure: &TaskAttempt) -> bool {
        false
    }
}

#[test]
fn failed_execution_can_propose_a_strategy_but_only_reviewed_green_learning_activates_it() {
    let task = "Implement task decomposition for \
                https://github.com/link-assistant/formal-ai/issues/847";
    let empty = TaskStrategyLedger::new();
    let unresolved = decompose_task_with_ledger(task, 6, &empty);
    assert!(unresolved.root.children.is_empty());
    assert!(!unresolved.is_atomic());
    let mut executor = BlockingExecutor;
    let failed = solve_recursively(&unresolved.to_recursive_task(), &mut executor);
    let proposal = TaskStrategyProposal::from_failed_run(&unresolved, &failed)
        .expect("a blocked uncontracted task should yield a typed strategy proposal");
    assert!(proposal.links_notation().contains("human_review_required"));

    let mut red = TaskStrategyLedger::new();
    assert!(
        red.promote(
            &proposal,
            TaskLearningGate::failed("task_decomposition_specification", 12, 1),
            TaskLearningApproval::granted("maintainer"),
        )
        .is_err()
    );
    assert!(
        red.promote(
            &proposal,
            TaskLearningGate::passed("task_decomposition_specification", 13),
            TaskLearningApproval::declined("maintainer"),
        )
        .is_err()
    );

    let mut reviewed = TaskStrategyLedger::new();
    reviewed
        .promote(
            &proposal,
            TaskLearningGate::passed("task_decomposition_specification", 13),
            TaskLearningApproval::granted("maintainer"),
        )
        .expect("green evidence plus explicit human approval should promote the strategy");
    let learned = decompose_task_with_ledger(task, 6, &reviewed);
    assert_eq!(learned.leaves().len(), 4);

    let durable = reviewed.links_notation();
    assert!(durable.contains("missing executable operation contract"));
    assert!(durable.contains("task_decomposition_specification"));
    assert!(durable.contains("reviewer \"maintainer\""));
    let restored = TaskStrategyLedger::from_links_notation(&durable)
        .expect("reviewed learning should survive a process restart");
    assert_eq!(
        decompose_task_with_ledger(task, 6, &restored),
        learned,
        "recalled learning must produce the same canonical plan"
    );
    let tampered = durable.replacen(
        "task_strategy_verified_change",
        "task_strategy_unreviewed_change",
        1,
    );
    assert!(
        TaskStrategyLedger::from_links_notation(&tampered).is_err(),
        "the durable learning record must be content-addressed"
    );

    let shipped = TaskStrategyLedger::shipped();
    let recalled = TaskStrategyLedger::from_links_notation(&shipped.links_notation())
        .expect("the repository-reviewed strategy evidence should also round-trip");
    assert_eq!(
        recalled.approved_strategy_ids(),
        shipped.approved_strategy_ids()
    );
}
