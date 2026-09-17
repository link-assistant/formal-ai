//! Plan 03 L13: the repository solver's real commit payload satisfies the
//! canonical version-3 attribution parser, not merely a local trailer check.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::cli_solve::{SolveArgs, run_solve};

use super::{git, metric_script};

fn fixture_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("formal-ai-solve-attribution-{stamp}"))
}

fn trailer(message: &str, name: &str) -> String {
    message
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name}: ")))
        .unwrap_or_else(|| panic!("commit message has no {name} trailer:\n{message}"))
        .to_owned()
}

fn write_fixture(root: &Path) -> String {
    fs::create_dir_all(root.join("src")).expect("fixture source directory");
    fs::write(
        root.join("src/web_search_core.rs"),
        "pub const WEB_SEARCH_PROVIDERS: &[&str] = &[\"google\"];\n",
    )
    .expect("fixture source");
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "fixture"]);
    git(root, &["config", "user.email", "fixture@example.invalid"]);
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "fixture base"]);
    git(root, &["rev-parse", "HEAD"])
}

#[test]
fn solve_commit_payload_is_accepted_by_the_canonical_attribution_parser() {
    let root = fixture_root();
    let base = write_fixture(&root);
    let external_evidence = root.with_extension("external-evidence");
    let _ = fs::remove_dir_all(&external_evidence);

    let outcome = run_solve(&SolveArgs {
        issue: None,
        task: Some(String::from(
            "Add \"wikiquote\" to the list of trusted search providers.",
        )),
        repository: root.display().to_string(),
        base_commit: Some(base),
        model: String::from("formal-ai"),
        evidence: external_evidence.clone(),
        pull_request: Some(String::from(
            "https://github.com/link-assistant/formal-ai/pull/1138",
        )),
        commit: true,
    })
    .expect("the isolated solve should produce an attributable commit payload");
    assert!(outcome.committed, "the isolated clone must have committed");

    let patch = root.with_extension("patch");
    fs::write(&patch, &outcome.diff).expect("write produced patch");
    git(&root, &["apply", patch.to_str().expect("UTF-8 patch path")]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-q", "-m", &outcome.commit_message]);
    let commit = git(&root, &["rev-parse", "HEAD"]);

    let session = trailer(&outcome.commit_message, "Formal-AI-Session");
    let model = trailer(&outcome.commit_message, "Formal-AI-Model");
    let evidence_path = trailer(&outcome.commit_message, "Formal-AI-Evidence");
    let trace = git(
        &root,
        &[
            "show",
            &format!("{commit}:{evidence_path}/repository-protocol.lino"),
        ],
    );
    let session_metadata = git(
        &root,
        &["show", &format!("{commit}:{evidence_path}/session-id.txt")],
    );
    let evidence = vec![
        (String::from("repository-protocol.lino"), trace),
        (String::from("session-id.txt"), session_metadata),
    ];
    assert_eq!(
        metric_script::model_attribution(&root, &commit, &[session], &evidence)
            .expect("the canonical parser must accept the solve payload"),
        Some(model)
    );

    let _ = fs::remove_file(patch);
    let _ = fs::remove_dir_all(external_evidence);
    let _ = fs::remove_dir_all(root);
}
