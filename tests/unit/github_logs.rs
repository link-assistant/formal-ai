use std::path::PathBuf;

use formal_ai::{
    collect_github_logs_with_runner, github_log_capture_plan, GithubLogCollectorConfig,
};

#[test]
fn github_log_plan_captures_issue_pr_run_and_all_comment_types() {
    let config = GithubLogCollectorConfig {
        repo: String::from("link-assistant/hive-mind"),
        output_dir: PathBuf::from("docs/case-studies/issue-115/raw-data/hive-mind"),
        issues: vec![1814],
        pulls: vec![1816],
        runs: vec![26_058_054_431],
        recent_issues: 10,
        recent_pulls: 10,
        recent_runs: 5,
        branch: Some(String::from("issue-1814-0f855d3671ac")),
    };

    let plan = github_log_capture_plan(&config).expect("valid plan");
    let commands: Vec<String> = plan
        .iter()
        .map(formal_ai::GithubLogCapture::command_line)
        .collect();

    assert!(commands.iter().any(|command| command.contains(
        "gh repo view link-assistant/hive-mind --json name,description,owner,url,defaultBranchRef,createdAt,pushedAt,isPrivate,licenseInfo"
    )));
    assert!(commands.iter().any(|command| command.contains(
        "gh issue view 1814 --repo link-assistant/hive-mind --json number,title,body,author,state,labels,comments,createdAt,updatedAt,url"
    )));
    assert!(commands.iter().any(|command| command
        .contains("gh api repos/link-assistant/hive-mind/issues/1814/comments --paginate")));
    assert!(commands.iter().any(|command| command.contains(
        "gh pr view 1816 --repo link-assistant/hive-mind --json number,title,body,author,state,isDraft,headRefName,baseRefName,commits,comments,reviews,reviewDecision,mergeStateStatus,createdAt,updatedAt,url"
    )));
    assert!(commands.iter().any(|command| command
        .contains("gh api repos/link-assistant/hive-mind/pulls/1816/comments --paginate")));
    assert!(commands.iter().any(|command| command
        .contains("gh api repos/link-assistant/hive-mind/issues/1816/comments --paginate")));
    assert!(commands.iter().any(|command| command
        .contains("gh api repos/link-assistant/hive-mind/pulls/1816/reviews --paginate")));
    assert!(commands
        .iter()
        .any(|command| command.contains("gh pr diff 1816 --repo link-assistant/hive-mind")));
    assert!(commands.iter().any(|command| command.contains(
        "gh run list --repo link-assistant/hive-mind --limit 5 --branch issue-1814-0f855d3671ac --json databaseId,workflowName,status,conclusion,createdAt,updatedAt,headSha,headBranch,event,url"
    )));
    assert!(commands.iter().any(|command| {
        command.contains("gh run view 26058054431 --repo link-assistant/hive-mind --log")
    }));
}

#[test]
fn github_log_collector_writes_manifest_and_captured_files() {
    let root = std::env::temp_dir().join(format!(
        "formal-ai-github-logs-{}-{}",
        std::process::id(),
        line!()
    ));
    let _ = std::fs::remove_dir_all(&root);

    let config = GithubLogCollectorConfig {
        repo: String::from("link-assistant/hive-mind"),
        output_dir: root.clone(),
        issues: vec![1814],
        pulls: Vec::new(),
        runs: Vec::new(),
        recent_issues: 0,
        recent_pulls: 0,
        recent_runs: 0,
        branch: None,
    };

    let mut seen_commands = Vec::new();
    let summary = collect_github_logs_with_runner(&config, |args| {
        seen_commands.push(args.join(" "));
        Ok(format!("captured: {}\n", args.join(" ")).into_bytes())
    })
    .expect("collector should write fake captures");

    assert_eq!(summary.captured.len(), 3);
    assert!(root.join("repo.json").is_file());
    assert!(root.join("issue-1814.json").is_file());
    assert!(root.join("issue-1814-comments.json").is_file());
    assert!(root.join("manifest.json").is_file());

    let manifest = std::fs::read_to_string(root.join("manifest.json")).expect("manifest");
    assert!(manifest.contains("\"repo\": \"link-assistant/hive-mind\""));
    assert!(manifest.contains("issue-1814-comments.json"));
    assert!(manifest.contains("repos/link-assistant/hive-mind/issues/1814/comments"));
    assert!(seen_commands
        .iter()
        .any(|command| command.starts_with("gh issue view 1814")));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn github_log_plan_rejects_repo_names_without_owner() {
    let config = GithubLogCollectorConfig {
        repo: String::from("hive-mind"),
        output_dir: PathBuf::from("out"),
        issues: Vec::new(),
        pulls: Vec::new(),
        runs: Vec::new(),
        recent_issues: 0,
        recent_pulls: 0,
        recent_runs: 0,
        branch: None,
    };

    let error = github_log_capture_plan(&config).expect_err("repo owner should be required");
    assert!(error.contains("OWNER/REPO"));
}
