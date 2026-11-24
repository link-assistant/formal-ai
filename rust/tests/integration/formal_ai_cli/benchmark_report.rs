use std::{fs, process::Command};

use super::tmpdir;

const LEDGER: &str = r#"gsm8k_suite
  record_type "external_benchmark_suite"
  id "gsm8k"
  ratchet_slice "1"
  minimum_pass_count "1"

gsm8k_prior
  record_type "external_benchmark_result"
  suite "gsm8k"
  date "2026-01-01"
  slice "1"
  passed "1"
  failed "0"
  total "1"
"#;
const FRONTIER: &str = "learning_frontier\n";
const DATASET_REVISION: &str = "3101c7d5072418e28b9008a6636bde82a006892c";

// Replays main run 45197910485 with a single synthetic case and local outputs.
#[test]
fn benchmark_report_survives_a_rejected_score_without_publishing_it() {
    for (expected, hash) in [(9, "c082535a036c2189"), (8, "6770c3637f5a2b48")] {
        let root = tmpdir();
        let cache = root.join("target/formal-ai-benchmarks");
        fs::create_dir_all(&cache).unwrap();
        let payload =
            format!("{{\"question\":\"What is 7 + 2?\",\"answer\":\"#### {expected}\"}}\n");
        fs::write(cache.join("gsm8k-test.jsonl"), &payload).unwrap();
        fs::write(
            cache.join("gsm8k-test.jsonl.provenance.lino"),
            format!(
                "external_benchmark_cache\n  suite gsm8k\n  source_ref github:{DATASET_REVISION}\n  \
                 download_url https://raw.githubusercontent.com/openai/grade-school-math/{DATASET_REVISION}/grade_school_math/data/test.jsonl\n  \
                 payload_bytes {}\n  content_id fnv1a64:{hash}\n",
                payload.len(),
            ),
        )
        .unwrap();
        let ledger = root.join("ledger.lino");
        let frontier = root.join("frontier.lino");
        fs::write(&ledger, LEDGER).unwrap();
        fs::write(&frontier, FRONTIER).unwrap();

        let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
            .current_dir(&root)
            .env("PATH", &root)
            .args([
                "benchmark",
                "run",
                "--suite",
                "gsm8k",
                "--slice",
                "1",
                "--date",
                "2026-01-02",
                "--append",
                "--ledger",
                "ledger.lino",
                "--learning-report",
                "retained/proposal.lino",
                "--frontier-record",
                "frontier.lino",
            ])
            .output()
            .expect("run the benchmark CLI");
        let report = fs::read_to_string(root.join("retained/proposal.lino"));
        let ledger_after = fs::read_to_string(ledger).unwrap();
        let frontier_after = fs::read_to_string(frontier).unwrap();
        fs::remove_dir_all(root).unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if expected == 8 {
            assert!(!output.status.success(), "{stdout}");
            assert!(
                stderr.contains("passed=0 is below the recorded minimum_pass_count=1"),
                "{stdout}\n{stderr}"
            );
            assert_eq!(ledger_after, LEDGER);
            assert_eq!(frontier_after, FRONTIER);
            let report = report.expect("ratchet rejection should retain the review-gated report");
            assert!(report.contains("GSM8K/0"), "{report}");
            assert!(report.contains("awaiting_human_review"), "{report}");
        } else {
            assert!(output.status.success(), "{stdout}\n{stderr}");
            assert!(stdout.contains("suite=gsm8k passed=1 failed=0 total=1"));
            assert!(ledger_after.contains("2026-01-02"));
            assert!(frontier_after.contains("2026-01-02"));
            assert_eq!(report.unwrap_err().kind(), std::io::ErrorKind::NotFound);
        }
    }
}
