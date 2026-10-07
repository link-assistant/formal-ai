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
//! tables in [`templates_core`] / [`templates_listing`] / [`templates_extended`] /
//! [`templates_stdin`] / [`templates_framework`], concatenated here as
//! [`TEMPLATE_GROUPS`].

mod languages;
mod tasks;
mod templates_core;
mod templates_extended;
mod templates_framework;
mod templates_listing;
mod templates_stdin;
mod types;

use std::borrow::Cow;

use crate::event_log::EventLog;
use crate::meta_algorithm_builder::{CodingSurface, MetaAlgorithmBuilder};

pub use languages::PROGRAM_LANGUAGES;
pub use tasks::PROGRAM_TASKS;
pub use types::{
    CompiledTemplate, ExecutionStatus, ProgramExecution, ProgramLanguage, ProgramSpec, ProgramTask,
    ProgramTemplate,
};

pub const WRITE_PROGRAM_INTENT: &str = "write_program";

pub fn record_algorithm_construction(log: &mut EventLog) {
    MetaAlgorithmBuilder::for_surface(CodingSurface::CodingCatalog).record(log);
}

/// Every compiled program template, grouped by source file. The groups are
/// split purely to keep each file under the repository's per-file line limit;
/// semantically they form a single flat table, read through
/// [`program_templates`].
const TEMPLATE_GROUPS: &[&[CompiledTemplate]] = &[
    templates_core::TEMPLATES_CORE,
    templates_listing::TEMPLATES_LISTING,
    templates_extended::TEMPLATES_EXTENDED,
    templates_stdin::TEMPLATES_STDIN,
    templates_framework::TEMPLATES_FRAMEWORK,
];

/// A catalog pair the documentation route answers (issue #1165 R1165-4/6):
/// the program it rediscovers and the row its run contract binds.
pub struct DocumentedPair {
    /// The task slug.
    pub task_slug: &'static str,
    /// The catalog row with the file and commands the documented program
    /// binds (`HelloWorldApp.java` and `javac HelloWorldApp.java`).
    pub language: ProgramLanguage,
    /// The rediscovered program, its contract and its deviation.
    pub program: crate::discovery_production::DocumentedProgram,
}

/// The catalog's runtime tables: every program it answers with, and the
/// pairs the documentation route answers.
struct CatalogTable {
    templates: Vec<ProgramTemplate>,
    documented: Vec<DocumentedPair>,
}

/// The catalog row a documented program binds: the base row with the file it
/// is saved as and its commands taken from the program's run contract.
fn bound_language(
    base: &ProgramLanguage,
    contract: &crate::discovery_production::DocumentedContract,
) -> ProgramLanguage {
    let command = |role: &str| {
        contract
            .commands
            .iter()
            .find(|command| command.role == role)
            .map(|command| Cow::Owned(command.command.clone()))
    };
    let mut language = base.clone();
    language.save_as = Cow::Owned(contract.save_as.clone());
    language.execution.check_command = command("check");
    if let Some(run) = command("run") {
        language.execution.run_command = run;
    }
    language
}

/// The catalog's tables, built once at runtime (issue #1165 R1165-4).
///
/// The compiled groups come first; then every pair the seed bundle retires to
/// the documentation route (`program_source "documentation_route"` in
/// `data/seed/hello-world-programs.lino`) takes the program the documentation
/// captures rediscover for it, and the row its run contract binds. A retired
/// pair whose captures yield no verified program has no template.
fn catalog_table() -> &'static CatalogTable {
    static TABLE: std::sync::OnceLock<CatalogTable> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut templates: Vec<ProgramTemplate> = TEMPLATE_GROUPS
            .iter()
            .copied()
            .flatten()
            .map(ProgramTemplate::from)
            .collect();
        let mut documented = Vec::new();
        for program in crate::discovery_production::documented_catalog_programs() {
            let (Some(task), Some(language), Ok(rediscovered)) = (
                program_task_by_slug(&program.task),
                program_language_by_slug(&program.language),
                program.rediscovered,
            ) else {
                continue;
            };
            let compiled = templates.iter().any(|template| {
                template.task_slug == task.slug && template.language_slug == language.slug
            });
            if compiled {
                continue;
            }
            templates.push(ProgramTemplate {
                task_slug: task.slug,
                language_slug: language.slug,
                code: Cow::Owned(rediscovered.recipe.entry.clone()),
            });
            documented.push(DocumentedPair {
                task_slug: task.slug,
                language: bound_language(language, &rediscovered.contract),
                program: rediscovered,
            });
        }
        CatalogTable {
            templates,
            documented,
        }
    })
}

/// Iterate over every program template the catalog answers with.
pub fn program_templates() -> impl Iterator<Item = &'static ProgramTemplate> {
    catalog_table().templates.iter()
}

/// Total number of templates in the catalog (used for diagnostics).
#[must_use]
pub fn program_template_count() -> usize {
    catalog_table().templates.len()
}

/// The pair the documentation route answers, when it answers this one.
#[must_use]
pub fn documented_pair(task_slug: &str, language_slug: &str) -> Option<&'static DocumentedPair> {
    catalog_table()
        .documented
        .iter()
        .find(|pair| pair.task_slug == task_slug && pair.language.slug == language_slug)
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

/// The `(task, language, template)` triple a request resolves to; a pair the
/// documentation route answers runs with the row its program binds.
#[must_use]
pub fn program_spec(task_slug: &str, language_slug: &str) -> Option<ProgramSpec> {
    let language = documented_pair(task_slug, language_slug)
        .map(|pair| &pair.language)
        .or_else(|| program_language_by_slug(language_slug))?;
    Some(ProgramSpec {
        task: program_task_by_slug(task_slug)?,
        language,
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

/// Does `normalized` name this implementation target by one of its surfaces?
fn names_target(normalized: &str, language: &ProgramLanguage) -> bool {
    alias_surfaces("program_language", language.slug).any(|alias| contains_token(normalized, alias))
}

#[must_use]
pub fn program_language_by_alias(normalized: &str) -> Option<&'static ProgramLanguage> {
    // A request that names both a framework and the language that framework is
    // written in — `напиши мне код на PHP Laravel`, issue #723 — names a single
    // implementation target, and it is the more specific of the two: answering
    // in the base language throws away the part of the request that was
    // hardest to satisfy. Framework rows are therefore consulted first. Nothing
    // else changes: with no framework named, this is the same first-match scan
    // over [`PROGRAM_LANGUAGES`] it has always been.
    PROGRAM_LANGUAGES
        .iter()
        .find(|language| language.is_framework() && names_target(normalized, language))
        .or_else(|| {
            PROGRAM_LANGUAGES
                .iter()
                .find(|language| names_target(normalized, language))
        })
}

/// The language a program composed for this request is written in, or `None`
/// when the request names no implementation target and inherits none.
///
/// A request names an *implementation target*, which may be a framework rather
/// than a language (`напиши мне код на PHP Laravel`, issue #723). Composing a
/// program out of the catalogued idioms is a question about the language rather
/// than about the target — Laravel adds no way of sorting a list that PHP does
/// not already have, and the artifact is a standalone script rather than a file
/// inside an application — so the target resolves through
/// [`ProgramLanguage::base_language`] here. The write-program path keeps the
/// target itself, and answers with the template, the file and the run command
/// that target actually asked for.
#[must_use]
pub fn composition_language(
    normalized: &str,
    inherited: Option<&'static ProgramLanguage>,
) -> Option<&'static ProgramLanguage> {
    Some(
        program_language_by_alias(normalized)
            .or(inherited)?
            .base_language(),
    )
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
    // A Latin language name can directly follow or precede Han characters in
    // an ordinary Chinese request (for example `翻译成Rust`). Han/Latin script
    // changes are token boundaries even when there is no whitespace. Keep
    // letter-to-letter boundaries strict so short aliases such as `rs` and `c`
    // still cannot match inside unrelated words.
    //
    // "Letter" here means alphabetic in *any* alphabetic script, not only
    // ASCII: asking `is_ascii_alphanumeric` instead read the `c` of the Spanish
    // `código` as an isolated token, because the `ó` after it is not ASCII and
    // so looked like a word boundary — and every Spanish coding request
    // mentioning code was answered in C (issue #1021). The scripts that are
    // written without word spaces stay boundaries, which is the same contract
    // [`contains_cjk`] and [`contains_devanagari`] draw everywhere else.
    if expected
        .chars()
        .all(|character| character.is_ascii_alphanumeric())
    {
        return normalized.match_indices(expected).any(|(index, _)| {
            let before = normalized[..index].chars().next_back();
            let after = normalized[index + expected.len()..].chars().next();
            let is_alias_continuation = |character: char| {
                let spaceless_script = contains_cjk(&character.to_string())
                    || contains_devanagari(&character.to_string());
                (character.is_alphanumeric() && !spaceless_script) || matches!(character, '+' | '#')
            };
            before.is_none_or(|character| !is_alias_continuation(character))
                && after.is_none_or(|character| !is_alias_continuation(character))
        });
    }
    normalized.split_whitespace().any(|token| token == expected)
}

/// A whitespace-bounded phrase.
///
/// A Han character beside a Latin phrase is a word boundary too, as it is for [`contains_token`]: an ordinary Chinese
/// request writes `写一个hello world程序` with no spaces around the phrase.
fn contains_phrase(normalized: &str, expected: &str) -> bool {
    if contains_cjk(expected) {
        return normalized.contains(expected);
    }
    let is_boundary =
        |character: char| character.is_whitespace() || contains_cjk(&character.to_string());
    normalized.match_indices(expected).any(|(index, _)| {
        let before = normalized[..index].chars().next_back();
        let after = normalized[index + expected.len()..].chars().next();
        before.is_none_or(is_boundary) && after.is_none_or(is_boundary)
    })
}
