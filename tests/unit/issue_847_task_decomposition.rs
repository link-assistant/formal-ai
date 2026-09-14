//! Issue #847: task decomposition must itself be a working task.
//!
//! Before this change both halves of decomposition were refused: "split this
//! task into subtasks" and "is this task atomic?" fell through to the
//! unknown-prompt fallback (`intent == "unknown"`), reproduced by
//! `experiments/issue_847_coding_ladder/run_coding_ladder.sh` nodes
//! `meta.L2.decompose` and `meta.L3.is_atomic`.
//!
//! These tests pin the *observable* contract only — recognition, the shape of
//! the answer, determinism, and the `sub_impulse:` trace. The algorithm's
//! internals are pinned by `tests/unit/specification/task_decomposition.rs`.

use std::fs;

use formal_ai::SymbolicAnswer;
use formal_ai::solver::solve;

/// The two prompts named in the issue body, verbatim except for shortening the
/// embedded paths so the assertion stays readable.
const SPLIT_PROMPT: &str = "Split this coding task into at least two smaller independent subtasks and list them as a numbered list, nothing else: 'Add a paths-ignore filter for experiments to release.yml and make docs-changed respect excluded_folders.'";
const ATOMIC_PROMPT: &str = "Answer with one word, yes or no: is the following coding task already atomic, meaning it cannot be usefully split further? 'In the file scripts/detect-code-changes.rs, add dev/log/ to the excluded_folders array.'";

fn answered(prompt: &str) -> SymbolicAnswer {
    let answer = solve(prompt);
    assert_ne!(
        answer.intent, "unknown",
        "prompt fell through to the unknown fallback: {prompt}\nanswer: {}",
        answer.answer
    );
    answer
}

#[test]
fn splitting_a_coding_task_into_subtasks_is_answered_not_refused() {
    let answer = answered(SPLIT_PROMPT);
    assert!(
        answer.answer.contains("1."),
        "expected a numbered sub-task list, got: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("2."),
        "expected at least two sub-tasks, got: {}",
        answer.answer
    );
}

#[test]
fn asking_whether_a_task_is_atomic_is_answered_standalone() {
    let answer = answered(ATOMIC_PROMPT);
    let lowered = answer.answer.to_lowercase();
    assert!(
        lowered.starts_with("yes"),
        "a single-file single-edit task is atomic, got: {}",
        answer.answer
    );
}

#[test]
fn a_composite_task_is_reported_as_not_atomic() {
    let answer = answered(
        "Answer with one word, yes or no: is the following coding task already atomic? 'Solve issue 843 and open a pull request.'",
    );
    let lowered = answer.answer.to_lowercase();
    assert!(
        lowered.starts_with("no"),
        "a two-verb task is not atomic, got: {}",
        answer.answer
    );
}

#[test]
fn decomposition_is_recognised_in_every_supported_language() {
    // English, Russian (Cyrillic), Hindi (Devanagari), Chinese (Han). Each
    // prompt asks for the same thing in its own language, so recognition can
    // only come from the seed lexicon, never from an English phrase list.
    let prompts = [
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

    for (language, prompt) in prompts {
        let answer = answered(prompt);
        assert!(
            answer.answer.contains("1.") && answer.answer.contains("2."),
            "{language}: expected a numbered sub-task list, got: {}",
            answer.answer
        );
    }
}

#[test]
fn atomicity_is_answerable_in_every_supported_language() {
    let prompts = [
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

    for (language, prompt) in prompts {
        let answer = answered(prompt);
        assert!(
            !answer.answer.trim().is_empty(),
            "{language}: atomicity answer must not be empty"
        );
    }
}

#[test]
fn every_sub_task_is_an_inspectable_sub_impulse_event() {
    let answer = answered(SPLIT_PROMPT);
    let sub_impulses: Vec<&String> = answer
        .evidence_links
        .iter()
        .filter(|link| link.starts_with("sub_impulse"))
        .collect();
    assert!(
        sub_impulses.len() >= 2,
        "each sub-task must surface as a sub_impulse event; got {sub_impulses:?} in {:?}",
        answer.evidence_links
    );
}

#[test]
fn decomposition_is_deterministic() {
    let first = solve(SPLIT_PROMPT);
    let second = solve(SPLIT_PROMPT);
    assert_eq!(first.answer, second.answer);
    assert_eq!(first.links_notation, second.links_notation);
}

#[test]
fn same_task_agent_cli_authorship_is_preserved() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };
    let session = "ses_05171239affeLG6WcrsR27Rb8U";
    let generated = read(
        "docs/case-studies/issue-847/self-hosting-authorship/task-decomposition-invariant.lino",
    );
    let canonical = read("data/meta/task-decomposition-invariant.lino");
    // The generated file records what this Agent CLI session actually wrote, so
    // it stays byte-for-byte as authored: rewriting it to match later work would
    // destroy the authorship claim this test exists to make. What must remain
    // true is that the shipped contract still carries every clause that session
    // produced. A clause added afterwards — the `binary` rule from #1028 — is
    // additional work, not a contradiction of this one.
    for clause in generated
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        assert!(
            canonical.lines().any(|shipped| shipped.trim() == clause),
            "the shipped contract dropped a clause session {session} authored: {clause}"
        );
    }

    let agent_log = read("docs/case-studies/issue-847/self-hosting-authorship/agent-cli.log");
    assert!(agent_log.contains(session));
    let formal_ai_log = read("docs/case-studies/issue-847/self-hosting-authorship/formal-ai.log");
    for transition in [
        "planned ToolCalls",
        "tool=write",
        "tool: \"bash\"",
        "planned Final",
        "task-decomposition-invariant.lino",
    ] {
        assert!(
            formal_ai_log.contains(transition),
            "server trace is missing {transition}"
        );
    }

    let decomposition =
        read("docs/case-studies/issue-847/self-hosting-authorship/decomposition.lino");
    assert_eq!(decomposition.matches("issue_847_smallest_leaf_").count(), 4);
    assert_eq!(
        decomposition
            .matches("authorship formal_ai_agent_cli")
            .count(),
        1
    );
    assert!(decomposition.contains(&format!("session {session}")));
    assert!(decomposition.contains("formal_ai_authored_percent 25"));
}
