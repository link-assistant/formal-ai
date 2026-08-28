//! Regression coverage for issue #1064: a workflow that could never pass.
//!
//! `experiments/issue_1028_agent_cli_ladder/run.sh` was committed as mode
//! `100644` while every other script a workflow invokes bare is `100755`. A
//! checkout therefore handed CI a non-executable file, and the workflow's only
//! real step died the same way on every run it ever had:
//!
//! ```text
//! run.sh: Permission denied
//! ##[error]Process completed with exit code 126.
//! ```
//!
//! Nothing in the suite read file modes, so the ladder was never exercised —
//! and because the failure is a shell error rather than a failed assertion, it
//! looked like a broken experiment rather than a broken commit.

use std::fs;
use std::process::Command;

fn workflow_files() -> Vec<(String, String)> {
    let dir = format!("{}/.github/workflows", env!("CARGO_MANIFEST_DIR"));
    fs::read_dir(&dir)
        .expect("workflows directory")
        .map(|entry| entry.expect("workflow entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
        })
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let body = fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
            (name, body)
        })
        .collect()
}

/// The mode git records for a tracked path, or `None` when it tracks none.
///
/// The index is what matters rather than the working tree: it is the bit a
/// fresh checkout restores, and therefore the bit CI actually runs with. A
/// local `chmod` that was never staged would otherwise hide the defect.
fn recorded_mode(path: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .args(["ls-files", "-s", "--", path])
        .output()
        .expect("git must run in the repository");
    let listing = String::from_utf8(output.stdout).expect("git output must be UTF-8");
    listing
        .split_whitespace()
        .next()
        .map(str::to_owned)
        .filter(|mode| !mode.is_empty())
}

#[test]
fn a_script_a_workflow_runs_directly_is_committed_executable() {
    let mut checked = 0_usize;
    for (name, body) in workflow_files() {
        for line in body.lines() {
            let Some(command) = line.trim().strip_prefix("run: ") else {
                continue;
            };
            let command = command.trim();
            // Only a bare invocation depends on the executable bit. `bash x.sh`
            // and `sh x.sh` name their own interpreter and run either way, so
            // they are not evidence of a defect.
            if std::path::Path::new(command)
                .extension()
                .is_none_or(|extension| extension != "sh")
                || command.contains(char::is_whitespace)
            {
                continue;
            }
            let mode = recorded_mode(command)
                .unwrap_or_else(|| panic!("{name} runs `{command}`, which is not a tracked file"));
            checked += 1;
            assert_eq!(
                mode, "100755",
                "{name} runs `{command}` directly, but it is committed as mode {mode}: a \
                 checkout gives CI a non-executable file and the step dies with exit 126"
            );
        }
    }
    assert!(
        checked > 0,
        "the sweep must actually find scripts a workflow runs directly"
    );
}
