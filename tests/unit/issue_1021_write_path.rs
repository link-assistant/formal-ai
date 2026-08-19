//! Issue #1021: the write path, in both states.
//!
//! The issue asks that the path that publishes a change be tested *refused by
//! default, permitted only under an explicit opt-in, with `issue create`
//! refused in both*. A test that only exercised the permitted state would prove
//! the ladder reaches the rung; a test that only exercised the default would
//! prove it refuses everything. Both states are here because the interesting
//! claim is the difference between them — and because #943 is exactly the bug
//! where one state was never checked.

use std::env;
use std::sync::{Mutex, MutexGuard, OnceLock};

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::contribution_write_path::{
    decide_with, opted_in_with, permits, plan_publication, plan_publication_with, Publication,
    WritePathDecision, WritePathRefusal,
};
use formal_ai::seed::{contribution_artifact_vocabulary, WritePathVocabulary};
use formal_ai::ChatMessage;

/// The opt-in lives in the process environment, so the two states cannot be
/// entered at once. Every test that touches it holds this lock.
fn opt_in_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_path() -> WritePathVocabulary {
    contribution_artifact_vocabulary().write_path
}

/// Run `body` with the opt-in set, then restore whatever was there before, so a
/// failure inside `body` cannot leave the ladder unlocked for the rest of the
/// suite.
fn with_opt_in(body: impl FnOnce()) {
    let vocab = write_path();
    let _guard = opt_in_lock();
    let previous = env::var(&vocab.opt_in_variable).ok();
    env::set_var(&vocab.opt_in_variable, &vocab.opt_in_value);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    match previous {
        Some(value) => env::set_var(&vocab.opt_in_variable, value),
        None => env::remove_var(&vocab.opt_in_variable),
    }
    if let Err(payload) = outcome {
        std::panic::resume_unwind(payload);
    }
}

fn shell_command(prompt: &str) -> Option<String> {
    let plan = plan_chat_step(&[ChatMessage::user(prompt)], &["exec_command"])?;
    let AgenticPlan::ToolCalls(calls) = plan else {
        return None;
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    arguments["command"].as_str().map(str::to_owned)
}

/// The seed has to actually define both rungs; a ladder with an empty rung
/// passes every assertion below without governing anything.
#[test]
fn the_ladder_has_both_rungs_and_an_opt_in_to_climb_the_first() {
    let vocab = write_path();
    assert!(!vocab.opt_in_variable.is_empty());
    assert!(!vocab.opt_in_value.is_empty());
    assert!(vocab.opt_in.iter().any(|action| action == "gh pr create"));
    assert!(vocab
        .refused
        .iter()
        .any(|action| action == "gh issue create"));
    // Merging stays a human decision, so no opt-in reaches it.
    assert!(vocab.refused.iter().any(|action| action == "gh pr merge"));
}

/// Default state: publishing is refused, and the refusal says which rung it is,
/// so a caller can tell "you have not delegated this" from "nobody can".
#[test]
fn publishing_is_refused_until_the_operator_opts_in() {
    let vocab = write_path();
    for command in [
        "gh pr create --fill",
        "gh pr edit 1027 --title x",
        "gh pr ready 1027",
        "git push origin issue-1021",
        "cd /tmp/checkout && gh pr create --draft",
    ] {
        assert_eq!(
            decide_with(command, &vocab, false),
            WritePathDecision::Refused(WritePathRefusal::OptInAbsent),
            "{command}"
        );
    }
}

/// Opted-in state: the same commands go through. This is the half that makes
/// the default meaningful — a ladder that refuses in both states is a wall.
#[test]
fn publishing_is_permitted_once_the_operator_opts_in() {
    let vocab = write_path();
    for command in [
        "gh pr create --fill",
        "gh pr edit 1027 --body-file body.md",
        "git push origin issue-1021",
    ] {
        assert_eq!(
            decide_with(command, &vocab, true),
            WritePathDecision::Permitted,
            "{command}"
        );
    }
}

/// The rung no opt-in reaches. Issue #943 is the record of what the absence of
/// this rule cost: a harness that filed issues nobody asked for.
#[test]
fn filing_an_issue_is_refused_in_both_states() {
    let vocab = write_path();
    for opted_in in [false, true] {
        for command in [
            "gh issue create --title x --body y",
            "gh issue create -R link-assistant/formal-ai -t x -b y",
            "gh pr merge 1027 --squash",
            "gh repo delete link-assistant/formal-ai --yes",
            "cd /tmp && gh issue create --fill",
        ] {
            assert_eq!(
                decide_with(command, &vocab, opted_in),
                WritePathDecision::Refused(WritePathRefusal::NeverDelegated),
                "{command} with opt-in {opted_in}"
            );
        }
    }
}

/// Reading is not publishing. A ladder that also refused `gh pr view` would be
/// answering a question nobody asked, and would take the reporting Formal AI
/// already does down with it.
#[test]
fn reading_is_governed_by_nothing() {
    let vocab = write_path();
    for opted_in in [false, true] {
        for command in [
            "gh pr view 1027 --json body",
            "gh issue view 1021",
            "git status",
            "git log --oneline -5",
            "ls",
            "cargo test --test unit",
        ] {
            assert_eq!(
                decide_with(command, &vocab, opted_in),
                WritePathDecision::Unaffected,
                "{command} with opt-in {opted_in}"
            );
        }
    }
}

/// The opt-in is read from the environment, and only the value the seed names
/// counts: a variable set to anything else is not a delegation.
#[test]
fn only_the_seeded_value_counts_as_an_opt_in() {
    let vocab = write_path();
    with_opt_in(|| assert!(opted_in_with(&write_path())));

    let _guard = opt_in_lock();
    let previous = env::var(&vocab.opt_in_variable).ok();
    for value in ["", "0", "yes", "true"] {
        env::set_var(&vocab.opt_in_variable, value);
        assert!(!opted_in_with(&vocab), "{value:?} is not the opt-in");
    }
    env::remove_var(&vocab.opt_in_variable);
    assert!(!opted_in_with(&vocab));
    if let Some(value) = previous {
        env::set_var(&vocab.opt_in_variable, value);
    }
}

/// The write path Formal AI takes on its own behalf: the publication steps are
/// planned only under the opt-in, and the plan is empty-handed without it.
#[test]
fn publishing_a_contribution_is_planned_only_under_the_opt_in() {
    let vocab = write_path();
    assert!(
        !vocab.publication.is_empty(),
        "a write path with no steps proves nothing"
    );
    let publication = Publication {
        repository: String::from("link-assistant/formal-ai"),
        branch: String::from("issue-1021-bdff51c09742"),
        title: String::from("Compose the artifacts a contribution carries"),
        body_file: String::from("target/pull-request-body.md"),
    };

    assert_eq!(
        plan_publication_with(&publication, &vocab, false),
        Err(WritePathRefusal::OptInAbsent)
    );

    let commands =
        plan_publication_with(&publication, &vocab, true).expect("the opt-in reaches the rung");
    assert_eq!(commands.len(), vocab.publication.len());
    assert!(commands
        .iter()
        .any(|command| command.contains("gh pr create")));
    // Every slot is filled: a leftover placeholder would ship as a literal.
    for command in &commands {
        assert!(!command.contains('{'), "{command}");
        assert!(command.contains(&publication.branch) || command.contains(&publication.repository));
    }

    // A composed title is a sentence, so an unquoted slot would hand `gh` the
    // first word as the title and the rest as positional arguments. Each planned
    // command has to survive a shell with the title still one argument.
    let create = commands
        .iter()
        .find(|command| command.contains("gh pr create"))
        .expect("a step that opens the pull request");
    assert!(
        create.contains(&format!("'{}'", publication.title)),
        "the title must reach gh as one argument: {create}"
    );
    for command in &commands {
        assert_eq!(
            command.matches('\'').count() % 2,
            0,
            "a planned command must not leave a quote open: {command}"
        );
    }

    // And a title carrying the quote character closes and reopens it rather than
    // ending the argument early.
    let awkward = Publication {
        title: String::from("Don't drop the operator's quote"),
        ..publication.clone()
    };
    let awkward_commands =
        plan_publication_with(&awkward, &vocab, true).expect("the opt-in reaches the rung");
    let awkward_create = awkward_commands
        .iter()
        .find(|command| command.contains("gh pr create"))
        .expect("a step that opens the pull request");
    assert!(
        awkward_create.contains(r"'Don'\''t drop the operator'\''s quote'"),
        "an embedded quote must be closed and reopened: {awkward_create}"
    );

    // And through the environment, which is how an operator actually opts in.
    assert_eq!(
        plan_publication(&publication),
        Err(WritePathRefusal::OptInAbsent)
    );
    with_opt_in(|| {
        assert!(plan_publication(&publication).is_ok());
        // The rung no opt-in reaches stays refused while the first one is open.
        assert!(!permits("gh issue create --title x"));
    });
    assert!(!permits("gh issue create --title x"));
}

/// The ladder governs the path Formal AI takes on its own initiative, not a
/// command the operator named. Issue #749 pinned `execute git push` as explicit
/// passthrough, and issue #824 is the record of what over-refusal costs: a
/// ladder that swallowed a delegated command would be that bug again.
#[test]
fn a_command_the_operator_named_is_not_the_ladders_business() {
    assert_eq!(
        shell_command("execute git push").as_deref(),
        Some("git push")
    );
    with_opt_in(|| {
        assert_eq!(
            shell_command("execute git push").as_deref(),
            Some("git push")
        );
    });
}
