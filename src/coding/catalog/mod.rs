//! Catalog of supported coding tasks, programming languages, and the code
//! templates that realize each task in each language.
//!
//! A `write_program` request is answered by resolving the prompt to a
//! [`ProgramSpec`] — a `(task, language, template)` triple — via the
//! alias-matching helpers below. The catalog is plain data: adding a language or
//! a task is a matter of extending [`PROGRAM_LANGUAGES`] / [`PROGRAM_TASKS`] and
//! supplying the matching templates, so coverage grows without the engine
//! changing. The matchers are script-aware (Latin/Cyrillic token boundaries,
//! CJK substring) so prompts in every supported language resolve.
//!
//! The catalog is split into cohesive, focused files to stay well under the
//! repository's per-file line limit: [`types`] (the records), [`languages`]
//! ([`PROGRAM_LANGUAGES`]), [`tasks`] ([`PROGRAM_TASKS`]), and the template
//! tables in [`templates_core`] / [`templates_extended`], concatenated here as
//! [`TEMPLATE_GROUPS`].

mod languages;
mod tasks;
mod templates_core;
mod templates_extended;
mod types;

pub use languages::PROGRAM_LANGUAGES;
pub use tasks::PROGRAM_TASKS;
pub use types::{
    ExecutionStatus, ProgramExecution, ProgramLanguage, ProgramSpec, ProgramTask, ProgramTemplate,
};

pub const WRITE_PROGRAM_INTENT: &str = "write_program";

/// Every program template, grouped by source file. The groups are split purely
/// to keep each file under the repository's per-file line limit; semantically
/// they form a single flat catalog, iterated via [`program_templates`].
const TEMPLATE_GROUPS: &[&[ProgramTemplate]] = &[
    templates_core::TEMPLATES_CORE,
    templates_extended::TEMPLATES_EXTENDED,
];

/// Iterate over every program template across all groups.
pub fn program_templates() -> impl Iterator<Item = &'static ProgramTemplate> {
    TEMPLATE_GROUPS.iter().copied().flatten()
}

/// Total number of templates in the catalog (used for diagnostics).
#[must_use]
pub fn program_template_count() -> usize {
    TEMPLATE_GROUPS.iter().map(|group| group.len()).sum()
}

#[must_use]
pub fn program_language_by_slug(slug: &str) -> Option<&'static ProgramLanguage> {
    PROGRAM_LANGUAGES
        .iter()
        .find(|language| language.slug == slug)
}

#[must_use]
pub fn program_task_by_slug(slug: &str) -> Option<&'static ProgramTask> {
    PROGRAM_TASKS.iter().find(|task| task.slug == slug)
}

#[must_use]
pub fn program_template(task_slug: &str, language_slug: &str) -> Option<&'static ProgramTemplate> {
    program_templates()
        .find(|template| template.task_slug == task_slug && template.language_slug == language_slug)
}

#[must_use]
pub fn program_spec(task_slug: &str, language_slug: &str) -> Option<ProgramSpec> {
    Some(ProgramSpec {
        task: program_task_by_slug(task_slug)?,
        language: program_language_by_slug(language_slug)?,
        template: program_template(task_slug, language_slug)?,
    })
}

#[must_use]
pub fn program_language_by_alias(normalized: &str) -> Option<&'static ProgramLanguage> {
    PROGRAM_LANGUAGES.iter().find(|language| {
        language
            .aliases
            .iter()
            .any(|alias| contains_token(normalized, alias))
    })
}

#[must_use]
pub fn program_task_by_alias(normalized: &str) -> Option<&'static ProgramTask> {
    PROGRAM_TASKS.iter().find(|task| {
        task.aliases
            .iter()
            .any(|alias| contains_phrase(normalized, alias))
    })
}

#[must_use]
pub fn supported_program_languages() -> String {
    PROGRAM_LANGUAGES
        .iter()
        .map(|language| language.slug)
        .collect::<Vec<_>>()
        .join(", ")
}

#[must_use]
pub fn supported_program_tasks() -> String {
    PROGRAM_TASKS
        .iter()
        .map(|task| task.slug)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Chinese (and other CJK) text is written without spaces between words, so the
/// whitespace-based token/phrase matchers below never see an isolated word. When
/// the expected alias itself contains a CJK ideograph we fall back to a plain
/// substring test, which is what "token boundaries" effectively mean for those
/// scripts. Latin and Cyrillic aliases keep strict boundary matching so short
/// tokens like `rust` never match inside `trust`.
pub fn contains_cjk(text: &str) -> bool {
    text.chars().any(|ch| {
        let cp = ch as u32;
        (0x3400..=0x4DBF).contains(&cp)
            || (0x4E00..=0x9FFF).contains(&cp)
            || (0xF900..=0xFAFF).contains(&cp)
            || (0x3040..=0x30FF).contains(&cp)
            || (0x3100..=0x312F).contains(&cp)
    })
}

fn contains_token(normalized: &str, expected: &str) -> bool {
    if contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized.split_whitespace().any(|token| token == expected)
}

fn contains_phrase(normalized: &str, expected: &str) -> bool {
    if contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized == expected
        || normalized.starts_with(&format!("{expected} "))
        || normalized.ends_with(&format!(" {expected}"))
        || normalized.contains(&format!(" {expected} "))
}

#[cfg(test)]
mod lino_parity {
    //! The portable `data/seed/hello-world-programs.lino` knowledge bundle is a
    //! hand-mirrored copy of this catalog. These tests lock the two together so
    //! a task or template added to the Rust catalog can never silently drift
    //! out of the downloadable seed (the divergence that previously left
    //! `list_files_arg` out of the seed). Regenerate the seed blocks with
    //! `experiments/issue-330-coding-tasks/generate_lino.py` when adding tasks.
    use super::*;

    /// Escape raw template code the way the `.lino` seed encodes a
    /// single-quoted `code '...'` payload: backslash -> `\\`, single-quote ->
    /// `\x27`, newline -> `\n`. Mirrors `generate_lino.py`.
    fn escape_for_lino(code: &str) -> String {
        code.replace('\\', "\\\\")
            .replace('\'', "\\x27")
            .replace('\n', "\\n")
    }

    #[test]
    fn lino_seed_tasks_line_lists_every_catalog_task() {
        let lino = crate::seed::HELLO_WORLD_PROGRAMS_LINO;
        let tasks_line = lino
            .lines()
            .find(|l| l.trim_start().starts_with("tasks \""))
            .expect("lino seed declares a tasks line");
        let listed: Vec<&str> = tasks_line
            .trim()
            .trim_start_matches("tasks \"")
            .trim_end_matches('"')
            .split('|')
            .collect();
        for task in PROGRAM_TASKS {
            assert!(
                listed.contains(&task.slug),
                "lino tasks line is missing `{}` (has {listed:?})",
                task.slug
            );
        }
    }

    #[test]
    fn lino_seed_mirrors_every_catalog_template() {
        let lino = crate::seed::HELLO_WORLD_PROGRAMS_LINO;
        for template in program_templates() {
            let needle = format!("code '{}'", escape_for_lino(template.code));
            assert!(
                lino.contains(&needle),
                "lino seed is missing the {}/{} template (escaped code not found)",
                template.task_slug,
                template.language_slug
            );
        }
    }

    #[test]
    fn lino_seed_has_no_extra_templates() {
        let lino = crate::seed::HELLO_WORLD_PROGRAMS_LINO;
        let seed_templates = lino.matches("\n  code '").count();
        assert_eq!(
            seed_templates,
            program_template_count(),
            "lino seed template count ({seed_templates}) must equal the catalog count ({})",
            program_template_count()
        );
    }
}
