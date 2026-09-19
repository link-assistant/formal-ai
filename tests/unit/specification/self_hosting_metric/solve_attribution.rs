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
        produces: Vec::new(),
        into: Vec::new(),
        seed: None,
        contains: Vec::new(),
        port: 8899,
        message: None,
        server_executable: None,
        agent_executable: None,
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

/// Plan 03 L13, second half: the live authoring loop (`src/authoring_loop.rs`,
/// reached through the reduced `scripts/author-change-with-formal-ai.sh`
/// wrapper) lands the same attributable commit payload — and a hosted model is
/// refused before anything runs.
#[test]
#[cfg(unix)]
fn the_live_authoring_loop_lands_a_commit_the_canonical_parser_accepts() {
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;

    use formal_ai::cli_solve::SolveArgs;

    fn executable(path: &std::path::Path, source: &str) {
        fs::write(path, source).expect("fixture script");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("fixture mode");
    }

    let root = fixture_root();
    let base = write_fixture(&root);
    let _ = base;
    for dir in ["bin", "scripts", "seed", "published"] {
        fs::create_dir_all(root.join(dir)).expect("fixture directories");
    }
    fs::write(root.join("seed/leaf.txt"), "original\n").expect("seed artifact");
    fs::write(root.join("seed/support.txt"), "support\n").expect("seed support");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::copy(
        repository.join("scripts/classify-agent-cli-stderr.sh"),
        root.join("scripts/classify-agent-cli-stderr.sh"),
    )
    .expect("copy the stderr classifier");
    executable(
        &root.join("bin/formal-ai-server"),
        "#!/bin/sh\nexec sleep 30\n",
    );
    executable(
        &root.join("bin/agent"),
        r#"#!/bin/sh
printf 'modified\n' > leaf.txt
echo "progress chatter from the CLI"
printf '{"session_id":"ses_authoringfixture"}\n'
"#,
    );

    // A hosted model is refused before the loop touches anything.
    let refused = run_solve(&SolveArgs {
        issue: None,
        task: Some(String::from("author an isolated fixture")),
        repository: root.display().to_string(),
        base_commit: None,
        model: String::from("claude-opus-4.8"),
        evidence: root.join("evidence"),
        pull_request: Some(String::from(
            "https://github.com/link-assistant/formal-ai/pull/1138",
        )),
        commit: true,
        produces: vec![String::from("leaf.txt")],
        into: Vec::new(),
        seed: None,
        contains: Vec::new(),
        port: 8899,
        message: Some(String::from("fixture")),
        server_executable: None,
        agent_executable: None,
    });
    assert!(
        refused.is_err(),
        "a hosted model must never be an authoring path"
    );

    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral port");
    let port = listener.local_addr().expect("bound address").port();
    let outcome = run_solve(&SolveArgs {
        issue: None,
        task: Some(String::from("author an isolated fixture")),
        repository: root.display().to_string(),
        base_commit: None,
        model: String::from("formal-ai"),
        evidence: PathBuf::from("evidence"),
        pull_request: Some(String::from(
            "https://github.com/link-assistant/formal-ai/pull/1138",
        )),
        commit: true,
        produces: vec![String::from("leaf.txt"), String::from("support.txt")],
        into: vec![
            String::from("published/leaf.txt"),
            String::from("published/support.txt"),
        ],
        seed: Some(String::from("seed")),
        contains: Vec::new(),
        port,
        message: Some(String::from("fixture: authoring loop landing")),
        server_executable: Some(root.join("bin/formal-ai-server")),
        agent_executable: Some(root.join("bin/agent")),
    })
    .expect("the live loop should land an attributable commit");
    drop(listener);
    assert!(outcome.committed, "the loop must commit under --commit");

    let commit = git(&root, &["rev-parse", "HEAD"]);
    let message = git(&root, &["log", "-1", "--format=%B"]);
    let session = trailer(&message, "Formal-AI-Session");
    let model = trailer(&message, "Formal-AI-Model");
    let evidence_path = trailer(&message, "Formal-AI-Evidence");
    assert_eq!(session, "ses_authoringfixture");
    assert_eq!(evidence_path, "evidence");
    let session_metadata = git(
        &root,
        &["show", &format!("{commit}:evidence/session-id.txt")],
    );
    let evidence = vec![(String::from("session-id.txt"), session_metadata)];
    assert_eq!(
        metric_script::model_attribution(&root, &commit, &[session], &evidence)
            .expect("the canonical parser must accept the authoring-loop payload"),
        Some(model)
    );

    let _ = fs::remove_dir_all(root);
}
