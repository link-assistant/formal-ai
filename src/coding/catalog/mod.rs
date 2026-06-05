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

/// Surface forms (across every supported language) carried by the meaning whose
/// slug is `<prefix>_<slug>`, or an empty iterator when no such meaning exists.
///
/// The coding catalog keeps each language's and task's alias surfaces in the
/// language-independent meaning lexicon — the `program_language_<slug>` and
/// `program_task_<slug>` meanings (roles [`crate::seed::ROLE_PROGRAM_LANGUAGE_ALIAS`]
/// and [`crate::seed::ROLE_PROGRAM_TASK_ALIAS`]) — instead of an inline list, so the
/// matchers below name only the concept by slug while the words stay self-describing
/// seed data shared byte-for-byte with the JS worker (issue #386).
fn alias_surfaces(prefix: &str, slug: &str) -> impl Iterator<Item = &'static str> {
    crate::seed::lexicon()
        .meaning(&format!("{prefix}_{slug}"))
        .into_iter()
        .flat_map(crate::seed::Meaning::words)
}

#[must_use]
pub fn program_language_by_alias(normalized: &str) -> Option<&'static ProgramLanguage> {
    PROGRAM_LANGUAGES.iter().find(|language| {
        alias_surfaces("program_language", language.slug)
            .any(|alias| contains_token(normalized, alias))
    })
}

#[must_use]
pub fn program_task_by_alias(normalized: &str) -> Option<&'static ProgramTask> {
    PROGRAM_TASKS.iter().find(|task| {
        alias_surfaces("program_task", task.slug).any(|alias| contains_phrase(normalized, alias))
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

/// Devanagari text (Hindi, …) is written without spaces between words, so the
/// whitespace-based matchers never isolate a single word — exactly as for CJK.
/// When a surface form carries a Devanagari sign we fall back to a substring
/// test. This mirrors [`contains_cjk`] and lets a handler partition a role's
/// word forms by script (Devanagari vs. Han) straight from the seed, so the
/// head-final Hindi and Chinese extraction strategies never name a raw word.
pub fn contains_devanagari(text: &str) -> bool {
    text.chars()
        .any(|ch| (0x0900..=0x097F).contains(&(ch as u32)))
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
