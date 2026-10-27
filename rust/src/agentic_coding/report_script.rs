//! The shell script an agentic report hands to the harness (#839).
//!
//! Issue #838 was filed by a single `format!` heredoc: it created two temporary
//! files, exported a context, `printf`-ed one intro line, appended
//! `tail -c 12000` of the export and called `gh issue create`. Nothing in that
//! string could be tested, and every command in it was assumed to exist.
//!
//! This builder replaces the heredoc with a small, inspectable structure: each
//! step names the program it needs, every named program is checked with
//! `command -v` before the first step runs. Export directories survive so the
//! user can retrieve them. GitHub submission has its own named CLI action;
//! the shared renderer prepares its body outside the shell.

use crate::seed;

const COMMAND_PLACEHOLDER: &str = "{command}";
const MESSAGE_PLACEHOLDER: &str = "{message}";
/// Abort the whole script when a required program is missing, instead of
/// filing a partial report (#839, §7).
const PREFLIGHT: &str =
    "command -v {command} >/dev/null 2>&1 || { printf '%s\\n' {message} >&2; exit 1; }";
/// Fail on the first error and on an unset variable.
const STRICT_MODE: &str = "set -eu";
/// Shell variable holding the directory of one report's deliverable exports.
const EXPORT_VARIABLE: &str = "export_dir";
/// One directory per export, outside the caller's working directory (#945).
///
/// There is no removal trap: a harness or server log
/// export *is* the deliverable, so the directory must outlive the script. What
/// it must never do is land in the CWD, where a repo checkout would collect
/// untracked session dumps at its root.
const EXPORT_SETUP: &str = "export_dir=$(mktemp -d \"${TMPDIR:-/tmp}/formal-ai-export.XXXXXX\")";

/// A shell script assembled from named steps.
pub(super) struct ReportScript {
    programs: Vec<String>,
    steps: Vec<String>,
    export: bool,
}

impl ReportScript {
    pub(super) const fn new() -> Self {
        Self {
            programs: Vec::new(),
            steps: Vec::new(),
            export: false,
        }
    }

    /// Add one step and remember the program it needs.
    pub(super) fn step(&mut self, program: &str, command: String) {
        if !self.programs.iter().any(|known| known == program) {
            self.programs.push(program.to_owned());
        }
        self.steps.push(command);
    }

    /// A shell-quoted path inside this report's surviving export directory.
    pub(super) fn export(&mut self, name: &str) -> String {
        self.export = true;
        format!("\"${EXPORT_VARIABLE}/{name}\"")
    }

    /// Render the script: strict mode, preflight, directory setup, then steps.
    pub(super) fn render(&self) -> String {
        let mut lines = vec![String::from(STRICT_MODE)];
        lines.extend(self.programs.iter().map(|program| preflight(program)));
        if self.export {
            lines.push(String::from(EXPORT_SETUP));
        }
        lines.extend(self.steps.iter().cloned());
        lines.join("\n")
    }
}

/// The `command -v` guard for one program.
fn preflight(program: &str) -> String {
    let message = seed::agent_info_value("issue_report_command_missing")
        .unwrap_or_default()
        .replace(COMMAND_PLACEHOLDER, program);
    PREFLIGHT
        .replace(COMMAND_PLACEHOLDER, program)
        .replace(MESSAGE_PLACEHOLDER, &shell_quote(&message))
}

/// Quote `value` as a single shell word.
pub(super) use crate::portable_string_contracts::quote_shell_word as shell_quote;
