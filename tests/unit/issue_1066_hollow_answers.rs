//! An answer that announces something must show it (issue #1066).
//!
//! The ladder scored a node green when its proof file existed, opened with the
//! pinned marker, and was not empty. Thirty-two of the sixty-three files that
//! passed that check said nothing: the check is mechanical and the answers were
//! hollow. These tests pin the two ways a decomposition answer could come out
//! hollow, each with wording the ladder never uses, because the guard is on the
//! shape of the answer and not on any prompt.

use formal_ai::engine::{FormalAiEngine, SymbolicAnswer};
use formal_ai::meta_frame::AtomicityReason;
use formal_ai::task_decomposition::decompose_task;

/// The lines of an answer that are numbered sub-task entries.
fn numbered_lines(answer: &str) -> Vec<&str> {
    answer
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once(['.', ')'])
                .is_some_and(|(head, _)| !head.is_empty() && head.chars().all(|c| c.is_numeric()))
        })
        .collect()
}

#[test]
fn an_answer_that_announces_sub_tasks_never_lists_none() {
    // The recursion can reach a leaf it did not resolve: a need it cannot split
    // into two independently checkable halves and for which it knows no
    // observable completion criterion. That leaf is a root with no children
    // that is nonetheless not reported atomic, so the reply said "these are the
    // sub-tasks" and then listed nothing at all -- a heading with no list.
    // Either the list is made, or the answer says instead why there is none.
    // The colon is the tell: it is the punctuation that promises the list, so an
    // answer that ends on one has announced something it never showed.
    for prompt in [
        "Work out whether migrating the billing database divides into smaller pieces.",
        "Split the following into sub-tasks: soothe the reviewer.",
    ] {
        let answer = FormalAiEngine.answer(prompt).answer;
        let answer = answer.trim();
        if numbered_lines(answer).is_empty() {
            assert!(
                !answer.ends_with([':', '\u{ff1a}']),
                "announced sub-tasks and listed none for {prompt:?}: {answer:?}"
            );
            assert!(
                answer.split_whitespace().count() > 4,
                "listed no sub-tasks and did not say why for {prompt:?}: {answer:?}"
            );
        }
    }
}

#[test]
fn a_root_the_recursion_never_split_is_reported_as_such() {
    // Both questions are two views of one recursion, so when the recursion
    // resolved nothing both views owe the reader the same explanation -- and
    // neither may borrow the other's verdict: the task is not atomic in the
    // sense that makes it directly checkable, and it is not split either.
    let decomposition = decompose_task("soothe the reviewer", 4);
    assert_eq!(
        decomposition.unenumerable_reason(),
        Some(AtomicityReason::SingleNeed),
        "the irreducible single need was not reported as one"
    );
}

#[test]
fn a_bound_that_stopped_the_split_before_it_started_is_reported() {
    // A depth bound of zero cuts the recursion before the first split, which
    // leaves the same shape -- a childless root that is not atomic -- for an
    // entirely different reason. A reader who lowered the bound has to be told
    // that the bound is what they are looking at, not the task's nature.
    let decomposition = decompose_task("rewrite the deployment script", 0);
    assert_eq!(
        decomposition.unenumerable_reason(),
        Some(AtomicityReason::DepthBound),
        "the depth bound was not reported as the reason nothing was enumerated"
    );
}

#[test]
fn a_split_that_did_happen_is_still_enumerated() {
    // The guard must cost nothing to the tasks that do split: reporting "there
    // is nothing to enumerate" for a task with sub-tasks would be the same
    // hollowness pointing the other way.
    let decomposition = decompose_task("rewrite the deployment script", 4);
    assert_eq!(decomposition.unenumerable_reason(), None);
    assert!(
        !decomposition.rows().is_empty(),
        "a splittable task enumerated nothing"
    );
}

#[test]
fn a_listed_sub_task_keeps_the_text_that_says_what_to_do() {
    // A sub-task is composed by putting the task inside a statement about it.
    // When the task was recovered from a question it carried the question mark
    // along, the statement became a question, and the answer's own question
    // policy deleted it as unearned -- leaving `1.  [criterion]`: a numbered
    // list whose every entry had lost its text. The criterion names the check;
    // the text is what a reader would have to do.
    for prompt in [
        "Is polishing the onboarding copy an atomic task?",
        "Is the checkout rewrite a task you can split into steps?",
        "Является ли рефакторинг платёжного модуля атомарной задачей?",
    ] {
        let answer = FormalAiEngine.answer(prompt).answer;
        let lines = numbered_lines(&answer);
        assert!(
            !lines.is_empty(),
            "no sub-tasks were listed for {prompt:?}: {answer:?}"
        );
        for line in lines {
            let text = line
                .split_once(['.', ')'])
                .map(|(_, rest)| rest)
                .unwrap_or_default();
            let prose = text
                .rsplit_once('[')
                .map_or(text, |(before, _)| before)
                .trim();
            assert!(
                prose.chars().any(char::is_alphabetic),
                "a listed sub-task lost its text for {prompt:?}: {answer:?}"
            );
        }
    }
}

#[test]
fn an_answer_is_never_a_heading_with_no_list() {
    // The handler that composes a reply knows why it has nothing to enumerate
    // and says so; this is the backstop underneath it, for the callers that
    // deliver an answer somewhere a reader will later find it. A reply that
    // stops on the colon introducing its list is a heading with nothing under
    // it, and it passes every mechanical check a harness makes -- a file that
    // says nothing is still a non-empty file.
    for text in [
        "This task divides into the following sub-tasks:",
        "Задача делится на следующие подзадачи:",
        // Chinese and Japanese introduce a list with the full-width colon, so a
        // guard that only read ASCII would hold for four supported languages
        // and not the fifth.
        "该任务可分解为以下子任务：",
    ] {
        let answer = announcing(text);
        assert!(
            answer.announces_a_list_it_does_not_make(),
            "an answer that promised a list and made none was not recognised: {text:?}"
        );
    }
    for text in [
        "This task divides into two sub-tasks:\n1. Write the migration.\n2. Verify it.",
        "It is an irreducible single need.",
    ] {
        let answer = announcing(text);
        assert!(
            !answer.announces_a_list_it_does_not_make(),
            "an answer that kept its promise was refused: {text:?}"
        );
    }
}

/// An answer carrying exactly `text`, for testing the shape of what it says.
fn announcing(text: &str) -> SymbolicAnswer {
    SymbolicAnswer {
        intent: "task_decomposition".to_owned(),
        answer: text.to_owned(),
        confidence: 1.0,
        evidence_links: Vec::new(),
        thinking_steps: Vec::new(),
        links_notation: String::new(),
        execution_recipe: None,
    }
}
