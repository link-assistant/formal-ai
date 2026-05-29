//! Core data types for the coding-task catalog: the language, task, template,
//! and resolved-spec records plus their execution metadata. These are plain
//! `Copy` records describing static data; the catalog tables in [`super`]
//! supply the values and the lookup helpers resolve a prompt onto a
//! [`ProgramSpec`].

#[derive(Clone, Copy)]
pub struct ProgramLanguage {
    pub slug: &'static str,
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub code_fence: &'static str,
    pub execution: ProgramExecution,
    pub source: &'static str,
    /// File name a novice should save the snippet as before running it (issue
    /// #330). The check/run commands above already reference this name.
    pub save_as: &'static str,
    /// One-line, novice-friendly hint for installing the toolchain (issue
    /// #330). URLs and shell commands stay canonical; only the surrounding
    /// prose is localized in `program_test_instructions`.
    pub setup_hint: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProgramTask {
    pub slug: &'static str,
    pub label: &'static str,
    pub aliases: &'static [&'static str],
    pub output: &'static str,
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
}

#[derive(Clone, Copy)]
pub struct ProgramExecution {
    pub status: ExecutionStatus,
    pub environment: &'static str,
    pub check_command: Option<&'static str>,
    pub run_command: &'static str,
    pub notes: &'static str,
}

#[derive(Clone, Copy)]
pub enum ExecutionStatus {
    Verified,
    Unavailable,
}

impl ExecutionStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Verified => "compiled and ran",
            Self::Unavailable => "not compiled or run",
        }
    }
}
