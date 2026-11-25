//! Issue #1158 (sub-issue of #1162): a Hive Mind delivery reads the pull
//! request feedback, writes a title and a description that name the verified
//! artifact, the verification commands and the closing reference, and only
//! then marks the pull request ready.
//!
//! Root cause pinned here: Hive Mind's formal-ai prompt says "Update the pull
//! request at <url>." and that wording did not bind the delivery to the pull
//! request, so the 2026-09-27 runs left a `[WIP]` draft that was never read,
//! retitled, described or marked ready. The prompt below is that run's prompt,
//! verbatim (`rust/tests/fixtures/hive-mind/formal-ai-prompt.txt`).

use crate::offline_delivery::{
    FORMAL_AI_PROMPT, PR, Workspace, assert_delivered, available, offline,
};

#[cfg(unix)]
#[test]
fn the_hive_mind_prompt_reads_feedback_then_titles_describes_and_readies_the_pull_request() {
    if !available(&["git", "node", "jq", "python3", "rustc", "timeout"]) {
        return;
    }
    // Issue #1639: the planner re-solves the recipe on every turn, and with read
    // access on by default (c0e480a36, #1582) each solve consults the per-job
    // shared source cache, which the rediscovery tests running beside this one
    // write. In main run 45636781871 the recipe no longer matched the source it
    // had written and the delivery stopped at the scope refusal. Replay it with
    // read access denied and an exclusive, empty cache.
    if crate::offline_replay::in_network_denied_child(
        "issue_1158_pull_request_steps::the_hive_mind_prompt_reads_feedback_then_titles_describes_and_readies_the_pull_request",
    ) {
        return;
    }
    let workspace = Workspace::new("Rust", "issue-1158");
    let session = workspace.session(&offline(FORMAL_AI_PROMPT), 40);
    assert!(
        session.answer().ends_with("the pull request is ready."),
        "{}",
        session.answer()
    );
    let outcome = assert_delivered(&workspace, "Rust", "main.rs");
    let calls = &outcome.gh_calls;
    let position = |call: &str| calls.iter().position(|made| made == call);
    let feedback = session
        .commands()
        .position(|command| command.contains("--json comments,reviews"))
        .expect("the pull request feedback is read");
    let edit = session
        .commands()
        .position(|command| command.contains("gh pr edit") && command.contains("--title"))
        .expect("the pull request is retitled and described");
    assert!(
        feedback < edit,
        "feedback is read before the description is written"
    );
    let (Some(edited), Some(ready)) = (
        position("pr edit"),
        calls.iter().rposition(|made| made == "pr ready"),
    ) else {
        panic!("edit and ready ran: {calls:?}");
    };
    assert!(
        position("pr checks").is_some_and(|checks| checks < edited),
        "{calls:?}"
    );
    assert!(
        edited < ready,
        "the description is written before readiness: {calls:?}"
    );
    assert!(
        session
            .commands()
            .all(|command| !command.contains("gh pr create")),
        "the prepared pull request {PR} is updated, never replaced"
    );
}
