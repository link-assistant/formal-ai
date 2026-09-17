//! Issue #1138 B7, plan 07 leaf 13: `open_draft_pull_request`, `DraftPullRequest`
//! and `--open-draft-pr`.
//!
//! The promotion protocol stops one step short of the review it exists to
//! produce: it prints the `git`/`gh` commands and a human runs them. The draft
//! pull request is the last step of the loop, and it is the step that must never
//! target the default branch, never merge, and never be marked ready.
//!
//! Four cases, each a refusal or a recorded fact — never a network action taken
//! by default (#656 stays green: without the flag the protocol behaves exactly
//! as before).
//!
//! Written before the leaf that makes them pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::promotion::{DraftPullRequest, PromotionRun, open_draft_pull_request};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn promoted_run() -> PromotionRun {
    formal_ai::promotion::demonstration_promotion_run()
}

fn empty_run() -> PromotionRun {
    PromotionRun::evaluate(Vec::new())
}

#[test]
fn open_draft_pull_request_never_targets_the_default_branch() {
    let draft: DraftPullRequest =
        open_draft_pull_request(&promoted_run()).expect("a promoted run opens a draft");
    assert!(
        draft.draft,
        "the pull request is opened as a draft and is never marked ready by the protocol"
    );
    assert_ne!(
        draft.head, draft.base,
        "a pull request from a branch to itself is not a review"
    );
    for forbidden in ["main", "master"] {
        assert_ne!(
            draft.head, forbidden,
            "the head branch must be the promotion's own review branch, never `{forbidden}`"
        );
    }
    assert!(
        draft.head.contains("promotion") || draft.head.contains("improve"),
        "the head branch names the promotion run that produced it, got {:?}",
        draft.head
    );
    assert!(
        !draft.merge,
        "the protocol never merges; a human reviews the draft"
    );
}

#[test]
fn open_draft_pull_request_refuses_when_nothing_was_promoted() {
    let refusal = open_draft_pull_request(&empty_run());
    assert!(
        refusal.is_err(),
        "a run that promoted nothing has nothing to review; opening an empty draft \
         would be a network action with no content behind it"
    );
    let reason = refusal.unwrap_err();
    assert!(
        !reason.trim().is_empty(),
        "the refusal names why it refused"
    );
}

#[test]
fn the_published_draft_is_recorded_as_an_append_only_event() {
    let draft = open_draft_pull_request(&promoted_run()).expect("a promoted run opens a draft");
    let events = draft.memory_events();
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_deref() == Some("promotion_published")),
        "publishing the draft appends a `promotion_published` event, so the act is part \
         of the add-only history rather than an untracked side effect"
    );
    assert!(
        events.iter().all(|event| !event.id.trim().is_empty()),
        "every appended event carries its stable id"
    );
}

#[test]
fn without_the_flag_the_protocol_behaves_exactly_as_before() {
    // #656 regression: `--open-draft-pr` is opt-in. Without it, `run_improve`
    // still only prints the branch plan for a human to run.
    let source = fs::read_to_string(repo_root().join("src/cli_improve.rs"))
        .expect("cli_improve.rs readable");
    assert!(
        source.contains("open_draft_pr"),
        "plan 07 leaf 13 adds `--open-draft-pr` to ImproveArgs"
    );
    let block = source
        .split("pub struct ImproveArgs {")
        .nth(1)
        .and_then(|tail| tail.split("\n}").next())
        .expect("ImproveArgs declares its fields");
    assert!(
        block.contains("pub open_draft_pr: bool"),
        "the flag is a bool that defaults to false, so the network action is opt-in"
    );

    let plan = promoted_run().branch_plan();
    assert!(
        !plan.commands.is_empty(),
        "without the flag the protocol still prints the commands a human would run"
    );
    assert!(
        plan.commands.iter().any(|command| command.contains("gh ")),
        "the printed plan still names the `gh` invocation, unchanged"
    );
}
