use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::file_legality::{
    AssessmentStatus, FileLegalityReport, LegalCategory, SafetyDisposition,
};

#[test]
fn file_legality_cli_accepts_provider_receipts_and_emits_safe_json() {
    let workspace = temp_workspace();
    fs::create_dir_all(&workspace).unwrap();
    let file = workspace.join("candidate.bin");
    let config = workspace.join("evidence.json");
    fs::write(&file, b"synthetic-cli-payload-marker").unwrap();
    fs::write(
        &config,
        serde_json::to_vec_pretty(&serde_json::json!({
            "jurisdictions": ["US"],
            "authorized_hash_matches": [{
                "provider": "approved-provider",
                "provider_reference": "case-cli-1",
                "report_uri": "https://provider.example/report/case-cli-1",
                "confirmed": true
            }]
        }))
        .unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["file-legality", file.to_str().unwrap(), "--config"])
        .arg(&config)
        .output()
        .expect("run file-legality command");

    assert!(output.status.success(), "{output:?}");
    let rendered = String::from_utf8(output.stdout).unwrap();
    let report: FileLegalityReport = serde_json::from_str(&rendered).unwrap();
    assert_eq!(
        report.safety_disposition,
        SafetyDisposition::RefuseAndEscalateToAuthorizedProvider
    );
    assert!(report.file.sha256.is_none());
    assert!(!rendered.contains("synthetic-cli-payload-marker"));
    let forbidden = report
        .assessment("US", LegalCategory::ForbiddenContent)
        .unwrap();
    assert_eq!(forbidden.status, AssessmentStatus::ConfirmedProhibitedMatch);
    assert_eq!(forbidden.evidence_ids, ["case-cli-1"]);
    assert_eq!(
        report.authorized_hash_matches[0].report_uri,
        "https://provider.example/report/case-cli-1"
    );

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn whole_file_legality_task_runs_documented_sidecar_end_to_end() {
    let workspace = temp_workspace();
    fs::create_dir_all(&workspace).unwrap();
    let file = workspace.join("candidate.pdf");
    fs::write(&file, b"%PDF-1.7\nsynthetic example document\n").unwrap();
    let config =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/file-legality/evidence.json");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args(["file-legality", file.to_str().unwrap(), "--config"])
        .arg(config)
        .output()
        .expect("run documented file-legality example");

    assert!(output.status.success(), "{output:?}");
    let report: FileLegalityReport = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report.assessments.len(), 6);
    assert_eq!(
        report
            .assessment("DE", LegalCategory::NationalSecurity)
            .unwrap()
            .status,
        AssessmentStatus::RiskSignal
    );
    assert_eq!(
        report
            .assessment("GB", LegalCategory::ForbiddenContent)
            .unwrap()
            .status,
        AssessmentStatus::NoRiskSignalDetected
    );
    assert_eq!(
        report
            .assessment("GB", LegalCategory::CopyrightAndIp)
            .unwrap()
            .status,
        AssessmentStatus::RiskSignal
    );

    fs::remove_dir_all(workspace).unwrap();
}

fn temp_workspace() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-issue-835-cli-{}-{nonce}",
        std::process::id()
    ))
}
