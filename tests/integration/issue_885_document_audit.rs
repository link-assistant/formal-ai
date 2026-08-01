use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn statement_audit_cli_persists_resolved_references_and_contextual_probability() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-relative-statement-audit-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create fixture directory");
    fs::write(
        directory.join("README.md"),
        "The protocol is externally standardized.\nIt is independently documented.\n",
    )
    .expect("write prose fixture");
    fs::write(
        directory.join("SECOND.md"),
        "It cannot inherit a subject from another document.\n",
    )
    .expect("write boundary fixture");
    fs::write(
        directory.join("evidence.json"),
        r#"{"captures":[{
          "statement":"The protocol is independently documented.",
          "source_label":"primary specification",
          "source_url":"https://example.test/protocol",
          "tier":"original_first_party",
          "stance":"supports",
          "strength":1.0,
          "captured_at":"2026-08-01T00:00:00Z",
          "sha256":"sha256:issue-885-protocol"
        }]}"#,
    )
    .expect("write evidence fixture");

    let report = directory.join("statement-audit.lino");
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["statement-audit", "--root"])
        .arg(&directory)
        .arg("--evidence")
        .arg(directory.join("evidence.json"))
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
    for expected in [
        "resolved_text \"The protocol is independently documented.\"",
        "contextual_posterior",
        "antecedent_statement_id",
        "resolution_policy \"closest_preceding_subject_same_document\"",
        "source_url \"https://example.test/protocol\"",
    ] {
        assert!(links.contains(expected), "missing {expected:?}:\n{links}");
    }
    let boundary_start = links
        .find("text \"It cannot inherit a subject from another document.\"")
        .expect("boundary statement");
    let boundary = &links[boundary_start..];
    let next_statement = boundary.find("\n    statement_").unwrap_or(boundary.len());
    assert!(
        !boundary[..next_statement].contains("resolved_text"),
        "a reference must never cross a Markdown document boundary"
    );

    fs::remove_dir_all(directory).expect("remove fixture directory");
}
