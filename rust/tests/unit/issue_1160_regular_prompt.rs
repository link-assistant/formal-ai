//! Issue #1160 (sub-issue of #1162): Hive Mind's regular prompt ("Issue to
//! solve: … Your prepared branch: … Your prepared Pull Request: …") and its
//! auto-restart prompt route to the same delivery as the formal-ai prompt.
//! Fixtures: `rust/tests/fixtures/hive-mind/`.

use crate::offline_delivery::{
    BRANCH, FORMAL_AI_PROMPT, ISSUE, PR, REGULAR_PROMPT, RESTART_PROMPT, Workspace,
    assert_delivered, available, offline,
};

#[test]
fn the_recorded_prompts_name_one_issue_branch_and_pull_request() {
    for prompt in [FORMAL_AI_PROMPT, REGULAR_PROMPT, RESTART_PROMPT] {
        let prompt = offline(prompt);
        assert!(
            prompt.contains(ISSUE) && prompt.contains(BRANCH) && prompt.contains(PR),
            "{prompt}"
        );
        assert!(
            !prompt.contains("konard/"),
            "every work item URL is offline: {prompt}"
        );
    }
    assert!(RESTART_PROMPT.contains("```\n?? Main.jar\n```"));
}

#[cfg(unix)]
#[test]
fn the_regular_prompt_delivers_a_ready_pull_request() {
    if !available(&["git", "node", "jq", "python3", "rustc", "timeout"]) {
        return;
    }
    let workspace = Workspace::new("Rust", "issue-1160-regular");
    let session = workspace.session(&offline(REGULAR_PROMPT), 40);
    assert!(
        session.answer().ends_with("the pull request is ready."),
        "{}",
        session.answer()
    );
    let outcome = assert_delivered(&workspace, "Rust", "main.rs");
    assert_eq!(outcome.commits, 2);
    for call in ["pr view", "pr checks", "pr edit", "pr ready"] {
        assert!(
            outcome.gh_calls.iter().any(|made| made == call),
            "{call}: {:?}",
            outcome.gh_calls
        );
    }
}

#[cfg(unix)]
#[test]
fn the_restart_prompt_ignores_a_stray_kotlin_jar_commits_pushes_and_keeps_the_pull_request_ready() {
    if !available(&["git", "node", "jq", "python3", "kotlinc", "java", "timeout"]) {
        return;
    }
    let workspace = Workspace::new("Kotlin", "issue-1160-restart");
    let _ = workspace.session(&offline(FORMAL_AI_PROMPT), 40).answer();
    let delivered = assert_delivered(&workspace, "Kotlin", "Main.kt");
    std::fs::write(
        workspace.dir.join("Main.jar"),
        b"PK\x03\x04 stray build output",
    )
    .unwrap();
    let session = workspace.session(&offline(RESTART_PROMPT), 40);
    assert!(
        session.answer().ends_with("the pull request is ready."),
        "{}",
        session.answer()
    );
    let outcome = assert_delivered(&workspace, "Kotlin", "Main.kt");
    assert!(
        !outcome.tracked.iter().any(|file| file == "Main.jar"),
        "{:?}",
        outcome.tracked
    );
    assert!(
        outcome.pushed.iter().any(|file| file == ".gitignore"),
        "{:?}",
        outcome.pushed
    );
    let ignore = std::fs::read_to_string(workspace.dir.join(".gitignore")).unwrap();
    assert!(
        ignore
            .lines()
            .any(|line| line == "Main.jar" || line == "/Main.jar"),
        "{ignore}"
    );
    assert!(
        outcome.commits > delivered.commits,
        "the ignore rule is committed"
    );
    assert_eq!(outcome.pr["isDraft"], false);
}
