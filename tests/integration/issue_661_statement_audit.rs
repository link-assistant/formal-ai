use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_directory() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-statement-audit-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(directory.join("src")).expect("create fixture directory");
    directory
}

#[test]
fn statement_audit_cli_scans_a_repository_and_replays_evidence() {
    let directory = fixture_directory();
    let report = directory.join("statement-audit.lino");
    let evidence = directory.join("evidence.json");
    fs::write(
        directory.join("README.md"),
        "The implementation lives in src/runtime.rs.\nThe obsolete helper lives in src/missing.rs.\nThe protocol is externally standardized.\n",
    )
    .expect("write prose fixture");
    fs::write(
        directory.join("src/runtime.rs"),
        "// Always emit an audit artifact.\npub fn run() {}\n",
    )
    .expect("write code fixture");
    fs::write(
        &evidence,
        r#"{"captures":[{
          "statement":"The protocol is externally standardized.",
          "source_label":"primary specification",
          "source_url":"https://example.test/specification",
          "tier":"original_first_party",
          "stance":"supports",
          "strength":1.0,
          "captured_at":"2026-07-19T00:00:00Z",
          "sha256":"sha256:specification"
        }]}"#,
    )
    .expect("write evidence fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["statement-audit", "--root"])
        .arg(&directory)
        .arg("--evidence")
        .arg(&evidence)
        .arg("--output")
        .arg(&report)
        .output()
        .expect("run statement audit command");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let links = fs::read_to_string(&report).expect("read generated audit");
    assert!(links.contains("summary"), "{links}");
    assert!(links.contains("relative_weight"), "{links}");
    assert!(links.contains("source_location"), "{links}");
    assert!(links.contains("source_url"), "{links}");
    assert!(links.contains("audit_finding"), "{links}");
    let summary: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("machine-readable summary");
    assert_eq!(
        summary["statement_audit"]["output"],
        report.display().to_string()
    );
    assert!(summary["statement_audit"]["statements"].as_u64().unwrap() >= 3);

    fs::remove_dir_all(directory).expect("remove fixture directory");
}
