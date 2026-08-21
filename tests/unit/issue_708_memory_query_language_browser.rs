use std::path::PathBuf;
use std::process::Command;

#[test]
fn browser_worker_executes_the_exact_query_language_parity_contract() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let node = Command::new("node")
        .arg("--version")
        .output()
        .expect("issue #708 browser parity requires Node.js");
    assert!(node.status.success(), "Node.js must be executable");

    let output = Command::new("node")
        .current_dir(&root)
        .arg("experiments/issue_708_agent_cli/test_browser_query_language.mjs")
        .output()
        .expect("run issue #708 browser query-language contract");
    assert!(
        output.status.success(),
        "browser query-language parity failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("issue-708 browser query-language parity: ok")
    );
}
