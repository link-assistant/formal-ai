//! Core data types for the coding-task catalog: the language, task, template,
//! and resolved-spec records plus their execution metadata. These are plain
//! `Copy` records describing static data; the catalog tables in [`super`]
//! supply the values and the lookup helpers resolve a prompt onto a
//! [`ProgramSpec`].
//!
//! Neither record carries its alias surfaces inline: the words a prompt must
//! contain to resolve a language or a task live in the language-independent
//! meaning lexicon — the `program_language_<slug>` / `program_task_<slug>`
//! meanings (issue #386) — and [`super::program_language_by_alias`] /
//! [`super::program_task_by_alias`] read them by slug. A record names only the
//! concept (its `slug`); the translatable words stay self-describing seed data.

#[derive(Clone, Copy)]
pub struct ProgramLanguage {
    pub slug: &'static str,
    pub name: &'static str,
    pub code_fence: &'static str,
    pub execution: ProgramExecution,
    pub source: &'static str,
    /// File name a novice should save the snippet as before running it (issue
    /// #330). The check/run commands above already reference this name.
    pub save_as: &'static str,
    /// The catalogued language this row is a *framework of*, or `None` when the
    /// row is a language in its own right.
    ///
    /// Issue #723 asked for Laravel and was answered in PHP, because the catalog
    /// had one axis where the request names two things: the language a program
    /// is written in, and the framework it is written against. Rather than a
    /// rule that recognises Laravel, the axis is widened — a row is an
    /// *implementation target*, and a target may be a framework of another
    /// target. Everything that is a property of the language rather than of the
    /// target (its grammar, its composable idioms) is read through
    /// [`ProgramLanguage::base_language`], so a framework inherits it without
    /// restating it, while everything the request actually asked for — the
    /// template, the file to save, the command to run — stays the framework's
    /// own (issue #1021).
    pub framework_of: Option<&'static str>,
}

impl ProgramLanguage {
    /// The language this target is written in: the row itself for a language,
    /// the base language for a framework.
    ///
    /// A framework whose `framework_of` names no catalogued row falls back to
    /// itself, which keeps this total; the catalog is checked for that in
    /// `tests/source/source_tests/coding/catalog/mod/framework_targets.rs`.
    #[must_use]
    pub fn base_language(&'static self) -> &'static Self {
        self.framework_of
            .and_then(super::program_language_by_slug)
            .unwrap_or(self)
    }

    /// Is this target a framework rather than a language?
    #[must_use]
    pub const fn is_framework(&self) -> bool {
        self.framework_of.is_some()
    }

    /// One-line, novice-friendly hint for installing the toolchain (issue
    /// #330). URLs and shell commands stay canonical; only the surrounding
    /// prose is localized in `program_test_instructions`.
    ///
    /// Issue #1138 plan 06 leaf L3: this was fourteen `&'static str` constants
    /// in `languages.rs`, which is knowledge the runtime could not reach — the
    /// install path could not read them and the publisher lookup could not
    /// replace them. They are now rows in `data/seed/toolchains.lino`, beside
    /// the probe that says whether the toolchain is actually here.
    #[must_use]
    pub fn setup_hint(&self) -> String {
        crate::prerequisite::probe::seed_setup_hint(self.slug)
    }

    /// The environment the recorded verification ran in.
    #[must_use]
    pub fn environment(&self) -> String {
        crate::prerequisite::probe::seed_environment(self.slug)
    }

    /// Whether the output shown beside this language's programs was observed.
    ///
    /// Read from `data/seed/toolchains.lino`, which records what a harness run
    /// actually observed, rather than from a constant in this file. A caller
    /// holding a live [`crate::prerequisite::probe::ProbeVerdict`] derives the
    /// status from it through [`ExecutionStatus::from_verdict`] instead; the
    /// seed row is what a caller with no observation of its own may say.
    #[must_use]
    pub fn execution_status(&self) -> ExecutionStatus {
        match crate::prerequisite::probe::seed_execution_status(self.slug).as_str() {
            "verified" => ExecutionStatus::Verified,
            "unavailable" => ExecutionStatus::Unavailable,
            _ => ExecutionStatus::NotProbed,
        }
    }

    /// Probe this language's toolchain on this machine, right now.
    #[must_use]
    // Read by plan 06 leaf L13 (the Telegram execution surface) and leaf L15
    // (the browser probe), which hold a live verdict rather than a seed row.
    #[allow(dead_code)]
    pub fn probe_now(&self, root: &std::path::Path) -> crate::prerequisite::probe::ProbeVerdict {
        crate::prerequisite::probe::seed_probe_for_language(self.slug).map_or_else(
            || crate::prerequisite::probe::ProbeVerdict::NotProbed {
                reason: String::from("no probe declared for this language"),
            },
            |probe| crate::prerequisite::probe::probe_command(&probe, root),
        )
    }
}

#[derive(Clone, Copy)]
pub struct ProgramTask {
    pub slug: &'static str,
    pub label: &'static str,
    pub output: &'static str,
    /// Standard input the task is defined against, or `""` when the program
    /// reads none.
    ///
    /// Issue #863 asked for *copy stdin to stdout*, and a task whose whole
    /// subject is standard input has no verifiable output until its input is
    /// fixed: `output` alone would claim a result the reader cannot reproduce.
    /// The fixture therefore belongs to the task, next to the output it
    /// produces, rather than to whichever harness happens to run it — which is
    /// what lets [`ProgramSpec::run_command_line`] and the issue-330
    /// verification harness feed the same bytes without agreeing on anything
    /// but the catalog.
    pub input: &'static str,
}

impl ProgramTask {
    #[must_use]
    pub fn output_for_language(&self, language: &ProgramLanguage) -> String {
        list_files_sample_output(self.slug, language.save_as)
            .unwrap_or_else(|| self.output.to_owned())
    }
}

#[derive(Clone, Copy)]
pub struct ProgramTemplate {
    pub task_slug: &'static str,
    pub language_slug: &'static str,
    pub code: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProgramSpec {
    pub language: &'static ProgramLanguage,
    pub task: &'static ProgramTask,
    pub template: &'static ProgramTemplate,
}

impl ProgramSpec {
    #[must_use]
    pub fn response_link(self) -> String {
        format!(
            "response:write_program:{}:{}",
            self.task.slug, self.language.slug
        )
    }

    #[must_use]
    pub fn parameter_summary(self) -> String {
        format!(
            "write_program(language={}, task={})",
            self.language.slug, self.task.slug
        )
    }

    #[must_use]
    pub fn legacy_intent(self) -> String {
        if self.task.slug == "hello_world" {
            format!("hello_world_{}", self.language.slug)
        } else {
            format!("write_program_{}_{}", self.task.slug, self.language.slug)
        }
    }

    #[must_use]
    pub fn expected_output(self) -> String {
        self.task.output_for_language(self.language)
    }

    /// The stdin fixture this task is defined against, if it reads any.
    #[must_use]
    pub fn stdin_fixture(self) -> Option<&'static str> {
        (!self.task.input.is_empty()).then_some(self.task.input)
    }

    /// The run command as a reader must actually type it.
    ///
    /// A program that reads standard input is not run by naming it alone: with
    /// no redirection it waits for a terminal that, in an answer, nobody is
    /// typing into. The fixture is therefore piped in on the command line
    /// rather than left to a file the answer would also have to describe, so
    /// the command is copy-pasteable and carries the input it was verified
    /// against (issue #863).
    #[must_use]
    pub fn run_command_line(self) -> String {
        let run_command = self.language.execution.run_command;
        self.stdin_fixture().map_or_else(
            || run_command.to_owned(),
            |input| format!("printf '{}' | {run_command}", shell_escaped(input)),
        )
    }
}

/// `input` with the characters a single-quoted `printf` format cannot carry
/// written as the escapes `printf` expands back into them.
fn shell_escaped(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\\' => escaped.push_str("\\\\"),
            '%' => escaped.push_str("%%"),
            '\'' => escaped.push_str("'\\''"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn list_files_sample_output(task_slug: &str, save_as: &str) -> Option<String> {
    let reverse = match task_slug {
        "list_files" | "list_files_arg" => false,
        "list_files_reverse_sort" | "list_files_arg_reverse_sort" => true,
        _ => return None,
    };
    let mut files = ["README.md", "data.txt", save_as];
    files.sort_unstable();
    if reverse {
        files.reverse();
    }
    Some(files.join("\n"))
}

/// How a catalogued program is checked and run.
///
/// Issue #1138 plan 06 leaf L3 removed `status` and `environment` from this
/// record. They were the two fields a human wrote by hand to say whether the
/// program had been observed working, in a record the runtime could not update;
/// they are rows of `data/seed/toolchains.lino` now, read through
/// [`ProgramLanguage::execution_status`] and [`ProgramLanguage::environment`],
/// beside the probe that observes the toolchain on this machine.
#[derive(Clone, Copy)]
pub struct ProgramExecution {
    pub check_command: Option<&'static str>,
    pub run_command: &'static str,
    pub notes: &'static str,
}

/// Whether the output shown beside a program was observed.
///
/// Issue #1138 plan 06 leaf L2 added the third variant. Two variants could say
/// "it worked" and "it did not", and had no way at all to say **we have not
/// looked** — so every row was a human's guess presented with the authority of
/// an observation, and `execution_output_label` relabelled unobserved output as
/// "Expected output after verification" from a constant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// The program was compiled and run, and the output shown is what it printed.
    Verified,
    /// The toolchain is not here, so nothing could be compiled or run.
    Unavailable,
    /// Nothing looked. This is not a synonym for `Unavailable`.
    NotProbed,
}

impl ExecutionStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Verified => "compiled and ran",
            Self::Unavailable => "not compiled or run",
            Self::NotProbed => "not probed in this environment",
        }
    }

    /// Derive the status from a live probe rather than from a constant.
    ///
    /// A probe that observed the toolchain present says the toolchain is
    /// present; it does **not** by itself say the program shown was run, which
    /// is why a caller that has not executed anything keeps `NotProbed` until it
    /// has an observation to point at.
    #[must_use]
    // Read by plan 06 leaf L13 and leaf L15, and by the `environments --probe`
    // command of leaf L10's benchmark list, each of which observes the
    // toolchain itself instead of reading what a harness run recorded.
    #[allow(dead_code)]
    pub const fn from_verdict(verdict: &crate::prerequisite::probe::ProbeVerdict) -> Self {
        match verdict {
            crate::prerequisite::probe::ProbeVerdict::Present { .. } => Self::Verified,
            crate::prerequisite::probe::ProbeVerdict::Missing { .. }
            | crate::prerequisite::probe::ProbeVerdict::Unusable { .. } => Self::Unavailable,
            crate::prerequisite::probe::ProbeVerdict::NotProbed { .. } => Self::NotProbed,
        }
    }
}
