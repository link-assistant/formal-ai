//! Issue #1066: the incremental self-authoring harnesses must survive the
//! locale a release server actually has.
//!
//! `agent dispatch --incremental` writes its report as UTF-8. Ruby's
//! `File.read` decodes with the locale's default external encoding, so under
//! the `POSIX`/`C` locale of a bare container the first non-ASCII byte raises
//! `Encoding::InvalidByteSequenceError`. The harness then aborts and reports a
//! failed self-authoring run for a dispatch that had in fact solved its task —
//! which is exactly what `experiments/issue_924_self_authoring/run.sh` did on
//! the server while its report recorded `"solved": true`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every harness that asserts over an incremental dispatch report.
const HARNESSES: &[&str] = &[
    "experiments/issue_924_self_authoring/run.sh",
    "experiments/issue_933_self_authoring/run.sh",
];

/// A report whose bytes are UTF-8 and whose shape satisfies both harnesses.
/// The em dash is the byte that broke the real run.
const UTF8_REPORT: &str = r#"{
  "mode": "incremental",
  "incremental": {
    "solved": true,
    "split_depth_reached": 1,
    "steps": [
      {"task": "create a file — exactly", "cli": "agent", "passed": false},
      {"task": "create a file — exactly", "cli": "composed-verifier", "passed": true}
    ],
    "splits": [{"children": ["one — first", "two — second"]}]
  }
}
"#;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// The Ruby program a harness embeds in a single-quoted bash string.
fn embedded_ruby_program(script: &str) -> String {
    let body = script
        .split_once("ruby -rjson -e '\n")
        .expect("harness drives its assertions through an embedded ruby program")
        .1;
    body.split_once("\n'")
        .expect("embedded ruby program is closed on its own line")
        .0
        .to_owned()
}

fn ruby_available() -> bool {
    Command::new("ruby")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

#[test]
fn incremental_harnesses_name_the_reports_encoding() {
    for harness in HARNESSES {
        let program = embedded_ruby_program(&read(harness));
        assert!(
            program.contains("File.read(ARGV.fetch(0), encoding: Encoding::UTF_8)"),
            "{harness} must decode the UTF-8 dispatch report as UTF-8, not as the locale default",
        );
        assert!(
            !program.contains("JSON.parse(File.read(ARGV.fetch(0)))"),
            "{harness} still reads the report with the locale's default encoding",
        );
        // A `'UTF-8'` literal would close the single-quoted bash string around
        // the program and leave Ruby with a bare `UTF` constant, so the whole
        // embedded program has to stay free of single quotes.
        assert!(
            !program.contains('\''),
            "{harness} embeds a single quote inside its single-quoted ruby program",
        );
    }
}

#[test]
fn incremental_harness_assertions_parse_a_utf8_report_under_a_posix_locale() {
    if !ruby_available() {
        eprintln!("skipping: ruby is not installed on this host");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "issue-1066-report-{}-{}",
        std::process::id(),
        line!()
    ));
    fs::create_dir_all(&directory).expect("temporary directory");
    let report = directory.join("dispatch-report.json");
    let learning = directory.join("learning.lino");
    fs::write(&report, UTF8_REPORT).expect("write report");
    fs::write(&learning, "client_contract_learning\n").expect("write learning artifact");

    for harness in HARNESSES {
        let program = embedded_ruby_program(&read(harness));
        let output = Command::new("ruby")
            .args(["-rjson", "-e", &program])
            .arg(&report)
            .arg("0")
            .arg(&learning)
            .env("LC_ALL", "POSIX")
            .env_remove("LANG")
            .env_remove("LC_CTYPE")
            .output()
            .expect("run the harness assertions");
        assert!(
            output.status.success(),
            "{harness} assertions failed on a UTF-8 report under LC_ALL=POSIX:\n{}",
            String::from_utf8_lossy(&output.stderr),
        );
    }
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn the_locale_reproduction_experiment_is_committed_and_executable() {
    let script = Path::new("experiments/issue_1066_self_development/reproduce-locale-report-read.sh");
    let path = root().join(script);
    assert!(path.is_file(), "{} is missing", script.display());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = fs::metadata(&path).expect("metadata").permissions().mode();
        assert!(
            mode & 0o111 != 0,
            "{} must be committed executable",
            script.display(),
        );
    }
}
