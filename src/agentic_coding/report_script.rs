//! The shell script an agentic report hands to the harness (#839).
//!
//! Issue #838 was filed by a single `format!` heredoc: it created two temporary
//! files, exported a context, `printf`-ed one intro line, appended
//! `tail -c 12000` of the export and called `gh issue create`. Nothing in that
//! string could be tested, and every command in it was assumed to exist.
//!
//! This builder replaces the heredoc with a small, inspectable structure: each
//! step names the program it needs, every named program is checked with
//! `command -v` before the first step runs, and a report that wants scratch
//! files gets one directory that is removed on exit. The report body itself is
//! no longer assembled in shell — `formal-ai report body` renders it.

use crate::seed;

const COMMAND_PLACEHOLDER: &str = "{command}";
const MESSAGE_PLACEHOLDER: &str = "{message}";
/// Abort the whole script when a required program is missing, instead of
/// filing a partial report (#839, §7).
const PREFLIGHT: &str =
    "command -v {command} >/dev/null 2>&1 || { printf '%s\\n' {message} >&2; exit 1; }";
/// Fail on the first error and on an unset variable.
const STRICT_MODE: &str = "set -eu";
/// Shell variable holding the scratch directory of one report.
const SCRATCH_VARIABLE: &str = "report_dir";
/// One directory per report, removed however the script ends.
///
/// GNU `mktemp` only substitutes a trailing run of `X`s, so the suffixed
/// template of #838 produced the literal name `formal-ai-report.XXXXXX.lino`
/// in the filed gist. A directory keeps the template trailing and still lets
/// every artifact carry a real extension.
const SCRATCH_SETUP: &str = "report_dir=$(mktemp -d \"${TMPDIR:-/tmp}/formal-ai-report.XXXXXX\")\n\
                             trap 'rm -rf \"$report_dir\"' EXIT";

/// A shell script assembled from named steps.
pub(super) struct ReportScript {
    programs: Vec<String>,
    steps: Vec<String>,
    scratch: bool,
}

impl ReportScript {
    pub(super) const fn new() -> Self {
        Self {
            programs: Vec::new(),
            steps: Vec::new(),
            scratch: false,
        }
    }

    /// Add one step and remember the program it needs.
    pub(super) fn step(&mut self, program: &str, command: String) {
        if !self.programs.iter().any(|known| known == program) {
            self.programs.push(program.to_owned());
        }
        self.steps.push(command);
    }

    /// A shell-quoted path inside this report's scratch directory.
    pub(super) fn scratch(&mut self, name: &str) -> String {
        self.scratch = true;
        format!("\"${SCRATCH_VARIABLE}/{name}\"")
    }

    /// Render the script: strict mode, preflight, scratch setup, then steps.
    pub(super) fn render(&self) -> String {
        let mut lines = vec![String::from(STRICT_MODE)];
        lines.extend(self.programs.iter().map(|program| preflight(program)));
        if self.scratch {
            lines.push(String::from(SCRATCH_SETUP));
        }
        lines.extend(self.steps.iter().cloned());
        lines.join("\n")
    }
}

/// The `command -v` guard for one program.
fn preflight(program: &str) -> String {
    let message = seed::agent_info()
        .remove("issue_report_command_missing")
        .unwrap_or_default()
        .replace(COMMAND_PLACEHOLDER, program);
    PREFLIGHT
        .replace(COMMAND_PLACEHOLDER, program)
        .replace(MESSAGE_PLACEHOLDER, &shell_quote(&message))
}

/// Quote `value` as a single shell word.
pub(super) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
