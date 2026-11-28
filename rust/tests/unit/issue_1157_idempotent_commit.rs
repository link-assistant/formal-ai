//! Issue #1157 (sub-issue of #1162): the delivery commit stages only the
//! verified files by literal pathspec, and a repeated or resumed run on a
//! finished checkout is a success that creates no commit and no pull request
//! edit, never a failure.

use crate::offline_delivery::{FORMAL_AI_PROMPT, Workspace, assert_delivered, available, offline};

#[cfg(unix)]
#[test]
fn the_commit_names_its_files_and_a_repeated_run_is_a_clean_no_op() {
    if !available(&["git", "node", "jq", "python3", "rustc", "timeout"]) {
        return;
    }
    let workspace = Workspace::new("Rust", "issue-1157");
    let prompt = offline(FORMAL_AI_PROMPT);
    let first = workspace.session(&prompt, 40);
    // Name a failed first delivery here, with its final answer and commands:
    // main runs 45490077033 and 45549993600 only reported the unchanged
    // "[WIP]" title below, while the test passes in isolation (issue #1637).
    assert!(
        first.answer().ends_with("the pull request is ready."),
        "the first run delivers: {}\ncommands: {:#?}",
        first.answer(),
        first.commands().collect::<Vec<_>>()
    );
    let delivered = assert_delivered(&workspace, "Rust", "main.rs");
    assert_eq!(
        delivered.commits, 2,
        "one delivery commit on the initial commit"
    );
    let commits: Vec<&str> = first
        .commands()
        .filter(|command| command.contains("git commit"))
        .collect();
    assert_eq!(commits.len(), 1, "{commits:?}");
    for command in &commits {
        for broad in [
            "git add -A",
            "git add .",
            "git add --all",
            "git commit -a",
            "git commit --all",
        ] {
            assert!(!command.contains(broad), "`{broad}` in {command}");
        }
        for file in [
            "main.rs",
            "tests/verify-output.sh",
            ".github/workflows/test-hello-world.yml",
        ] {
            assert!(
                command.contains(file),
                "{file} is a literal pathspec of {command}"
            );
        }
    }

    let second = workspace.session(&prompt, 40);
    assert!(
        second.answer().ends_with("the pull request is ready."),
        "a repeated run reports success: {}",
        second.answer()
    );
    let again = assert_delivered(&workspace, "Rust", "main.rs");
    assert_eq!(again.commits, 2, "a repeated run makes no new commit");
    let added = &again.gh_calls[delivered.gh_calls.len()..];
    assert!(
        !added
            .iter()
            .any(|call| call == "pr edit" || call == "pr ready"),
        "{added:?}"
    );
}
