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
    /// One-line, novice-friendly hint for installing the toolchain (issue
    /// #330). URLs and shell commands stay canonical; only the surrounding
    /// prose is localized in `program_test_instructions`.
    pub setup_hint: &'static str,
}

#[derive(Clone, Copy)]
pub struct ProgramTask {
    pub slug: &'static str,
    pub label: &'static str,
    pub output: &'static str,
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
