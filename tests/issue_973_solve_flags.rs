//! Issue #973: every `solve` invocation this repository publishes must carry
//! `--attach-logs --verbose`.
//!
//! The 2026-08-04 run on PR #927 failed after 22 seconds and left exactly one
//! artefact — a comment whose whole reason was `AGENT execution failed with
//! Agent reported error: [object Object]`, followed by "Logs were not attached
//! because `--attach-logs` was not enabled." The container is gone, so the real
//! cause is unrecoverable. Both flags exist to make that impossible again:
//! `--attach-logs` publishes the session log to the pull request, and
//! `--verbose` is what makes the Agent adapter dump the raw JSON of error and
//! fatal-startup records (link-assistant/hive-mind#2143).
//!
//! These tests read the repository's own text, so dropping either flag from a
//! documented or scripted invocation turns red instead of silently producing
//! another unlearnable failure.

use std::fs;
use std::path::{Path, PathBuf};

/// Directories that hold recorded history rather than instructions we follow.
/// A past run is evidence and must stay byte-for-byte as it happened, even when
/// it shows the invocation this issue forbids going forward.
const HISTORY_PREFIXES: &[&str] = &[
    "docs/case-studies/",
    "dev/log/",
    "experiments/",
    "coverage/",
];

/// Where a `solve` invocation may be published from: guides and runnable code.
const SCANNED_ROOTS: &[&str] = &[
    "CONTRIBUTING.md",
    "README.md",
    "ARCHITECTURE.md",
    "REQUIREMENTS.md",
    "ROADMAP.md",
    "GOALS.md",
    "docs",
    "examples",
    "scripts",
    ".github",
    "src",
];

const SCANNED_EXTENSIONS: &[&str] = &["md", "sh", "yml", "yaml", "rs", "toml", "json", "mjs", "ts"];

const REQUIRED_FLAGS: [&str; 2] = ["--attach-logs", "--verbose"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    let path = root().join(path);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn is_history(relative: &str) -> bool {
    HISTORY_PREFIXES
        .iter()
        .any(|prefix| relative.starts_with(prefix))
}

fn collect(path: &Path, relative: String, files: &mut Vec<(String, String)>) {
    if is_history(&relative) {
        return;
    }
    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .unwrap_or_else(|error| panic!("read dir {}: {error}", path.display()))
            .filter_map(Result::ok)
            .collect();
        entries.sort_by_key(std::fs::DirEntry::path);
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') && name != ".github" {
                continue;
            }
            let child = if relative.is_empty() {
                name
            } else {
                format!("{relative}/{name}")
            };
            collect(&entry.path(), child, files);
        }
        return;
    }
    let scanned = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| SCANNED_EXTENSIONS.contains(&extension));
    if !scanned {
        return;
    }
    if let Ok(text) = fs::read_to_string(path) {
        files.push((relative, text));
    }
}

fn scanned_files() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for entry in SCANNED_ROOTS {
        let path = root().join(entry);
        if path.exists() {
            collect(&path, (*entry).to_owned(), &mut files);
        }
    }
    files
}

/// Join shell/markdown line continuations so a wrapped command is judged as the
/// single command it is, not as a first line that "lost" its flags.
fn join_continuations(text: &str) -> Vec<(usize, String)> {
    let mut joined: Vec<(usize, String)> = Vec::new();
    let mut pending: Option<(usize, String)> = None;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_end();
        let (body, continues) = trimmed
            .strip_suffix('\\')
            .map_or((trimmed, false), |head| (head.trim_end(), true));
        let entry = if let Some((start, mut buffer)) = pending.take() {
            buffer.push(' ');
            buffer.push_str(body.trim_start());
            (start, buffer)
        } else {
            (index + 1, body.to_owned())
        };
        if continues {
            pending = Some(entry);
        } else {
            joined.push(entry);
        }
    }
    if let Some(entry) = pending {
        joined.push(entry);
    }
    joined
}

/// A `solve` command names an issue or pull request URL, or the placeholder a
/// guide substitutes for one. Prose such as "we do not solve a task by hand"
/// never matches, because the token after `solve` is an ordinary word.
fn targets_an_issue(rest: &str) -> bool {
    let target = rest.split_whitespace().next().unwrap_or_default();
    let target = target.trim_start_matches(['"', '\'', '`']);
    target.starts_with("https://github.com/")
        || target.starts_with("ISSUE_URL")
        || target.starts_with("$2")
        || target.starts_with("${")
        || target.starts_with("$ISSUE")
        || target.starts_with("<issue")
}

/// Every published `solve` invocation, as `(file, line, command)`.
fn solve_invocations() -> Vec<(String, usize, String)> {
    let mut found = Vec::new();
    for (file, text) in scanned_files() {
        for (line, command) in join_continuations(&text) {
            let bytes = command.as_bytes();
            let mut search = 0;
            while let Some(offset) = command[search..].find("solve ") {
                let start = search + offset;
                search = start + "solve ".len();
                let preceded_by_word = start > 0
                    && (bytes[start - 1].is_ascii_alphanumeric()
                        || bytes[start - 1] == b'_'
                        || bytes[start - 1] == b'-');
                if preceded_by_word {
                    continue;
                }
                let rest = &command[search..];
                if !targets_an_issue(rest) {
                    continue;
                }
                let invocation = command[start..]
                    .split('`')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_owned();
                found.push((file.clone(), line, invocation));
            }
        }
    }
    found
}

#[test]
fn the_live_self_coding_entry_point_attaches_logs_and_runs_verbose() {
    let script = read("examples/self-coding/run.sh");
    let live = script
        .lines()
        .find(|line| line.contains("exec solve"))
        .expect("examples/self-coding/run.sh must keep its --live solve entry point");
    for flag in REQUIRED_FLAGS {
        assert!(
            live.contains(flag),
            "the --live entry point must pass {flag} so a failed run leaves evidence behind: {live}"
        );
    }
}

#[test]
fn every_published_solve_invocation_carries_both_evidence_flags() {
    let invocations = solve_invocations();
    assert!(
        !invocations.is_empty(),
        "the repository must keep publishing at least one runnable solve invocation"
    );
    for (file, line, invocation) in &invocations {
        for flag in REQUIRED_FLAGS {
            assert!(
                invocation.contains(flag),
                "{file}:{line} publishes a solve invocation without {flag}; \
                 a run started without both flags can fail with no recoverable \
                 evidence (issue #973): {invocation}"
            );
        }
    }
}

#[test]
fn contributing_explains_why_both_flags_are_load_bearing() {
    let contributing = read("CONTRIBUTING.md");
    for needle in [
        "--attach-logs --verbose",
        "[object Object]",
        "raw JSON",
        "https://github.com/link-assistant/hive-mind/pull/2143",
        "docs/case-studies/issue-973/README.md",
    ] {
        assert!(
            contributing.contains(needle),
            "CONTRIBUTING.md must document the solve session policy; missing {needle:?}"
        );
    }
}

#[test]
fn the_case_study_records_the_unrecoverable_failure_and_the_fix() {
    let case_study = read("docs/case-studies/issue-973/README.md");
    for needle in [
        "https://github.com/link-assistant/formal-ai/pull/927#issuecomment-5174474849",
        "2026-08-04T04:05:17Z",
        "[object Object]",
        "https://github.com/link-assistant/hive-mind/issues/2141",
        "https://github.com/link-assistant/agent/issues/289",
        "https://github.com/link-assistant/agent/issues/290",
        "raw-data/pr-927-failure-comment.json",
        "examples/self-coding/run.sh",
        "tests/issue_973_solve_flags.rs",
    ] {
        assert!(
            case_study.contains(needle),
            "issue 973 case study is missing {needle:?}"
        );
    }

    let evidence = read("docs/case-studies/issue-973/raw-data/pr-927-failure-comment.json");
    for needle in [
        "[object Object]",
        "Logs were not attached because `--attach-logs` was not enabled.",
        "2026-08-04T04:05:17Z",
    ] {
        assert!(
            evidence.contains(needle),
            "the captured failure comment is missing {needle:?}"
        );
    }
}
