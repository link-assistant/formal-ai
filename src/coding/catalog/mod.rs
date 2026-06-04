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

#[cfg(test)]
mod seed_alias_coverage {
    //! The matchers ([`program_language_by_alias`] / [`program_task_by_alias`])
    //! read their surface words from the meaning lexicon — the
    //! `program_language_<slug>` / `program_task_<slug>` meanings (issue #386) —
    //! so the catalog tables name only the concept by slug. These guards lock the
    //! two sides together: every catalog slug must own its alias meaning (else
    //! that language or task would silently never match), and every alias meaning
    //! must name a real catalog slug (else it is dead seed data).
    use super::*;
    use crate::seed::{lexicon, ROLE_PROGRAM_LANGUAGE_ALIAS, ROLE_PROGRAM_TASK_ALIAS};

    #[test]
    fn every_language_slug_owns_an_alias_meaning() {
        for language in PROGRAM_LANGUAGES {
            let slug = format!("program_language_{}", language.slug);
            let meaning = lexicon()
                .meaning(&slug)
                .unwrap_or_else(|| panic!("language `{}` has no `{slug}` meaning", language.slug));
            assert!(
                meaning.has_role(ROLE_PROGRAM_LANGUAGE_ALIAS),
                "`{slug}` must carry the `{ROLE_PROGRAM_LANGUAGE_ALIAS}` role"
            );
            assert!(
                meaning.words().next().is_some(),
                "`{slug}` must lexicalise at least one alias surface"
            );
        }
    }

    #[test]
    fn every_task_slug_owns_an_alias_meaning() {
        for task in PROGRAM_TASKS {
            let slug = format!("program_task_{}", task.slug);
            let meaning = lexicon()
                .meaning(&slug)
                .unwrap_or_else(|| panic!("task `{}` has no `{slug}` meaning", task.slug));
            assert!(
                meaning.has_role(ROLE_PROGRAM_TASK_ALIAS),
                "`{slug}` must carry the `{ROLE_PROGRAM_TASK_ALIAS}` role"
            );
            assert!(
                meaning.words().next().is_some(),
                "`{slug}` must lexicalise at least one alias surface"
            );
        }
    }

    #[test]
    fn every_language_alias_meaning_names_a_catalog_slug() {
        for meaning in lexicon().meanings_with_role(ROLE_PROGRAM_LANGUAGE_ALIAS) {
            let slug = meaning.slug.strip_prefix("program_language_").unwrap_or_else(|| {
                panic!("`{}` carries the language-alias role but is not a `program_language_<slug>` meaning", meaning.slug)
            });
            assert!(
                program_language_by_slug(slug).is_some(),
                "alias meaning `{}` names language slug `{slug}`, absent from PROGRAM_LANGUAGES",
                meaning.slug
            );
        }
    }

    #[test]
    fn every_task_alias_meaning_names_a_catalog_slug() {
        for meaning in lexicon().meanings_with_role(ROLE_PROGRAM_TASK_ALIAS) {
            let slug = meaning.slug.strip_prefix("program_task_").unwrap_or_else(|| {
                panic!("`{}` carries the task-alias role but is not a `program_task_<slug>` meaning", meaning.slug)
            });
            assert!(
                program_task_by_slug(slug).is_some(),
                "alias meaning `{}` names task slug `{slug}`, absent from PROGRAM_TASKS",
                meaning.slug
            );
        }
    }

    #[test]
    fn every_language_resolves_from_its_seed_surfaces() {
        // End-to-end: each language's canonical slug spelling is a unique token
        // (no other language lexicalises it), so feeding it must resolve back to
        // that language through the seed — proving the `alias_surfaces` read and
        // the matcher wiring are live, not just structurally present.
        for language in PROGRAM_LANGUAGES {
            assert_eq!(
                program_language_by_alias(language.slug).map(|l| l.slug),
                Some(language.slug),
                "`{}` must resolve to itself through its seed surfaces",
                language.slug
            );
        }
    }

    #[test]
    fn every_task_resolves_through_the_seed() {
        // End-to-end wiring smoke for the `program_task_` branch: feeding each
        // task meaning's first surface must resolve to *some* task. (Priority and
        // substring matching mean a longer phrase can legitimately resolve to a
        // shorter-prefix task — e.g. a list_files_arg phrase to list_files — so we
        // assert reachability, not self-identity; tests/unit locks the exact map.)
        for task in PROGRAM_TASKS {
            let slug = format!("program_task_{}", task.slug);
            let first = lexicon()
                .meaning(&slug)
                .and_then(|m| m.words().next())
                .unwrap_or_else(|| panic!("`{slug}` must lexicalise at least one surface"));
            assert!(
                program_task_by_alias(first).is_some(),
                "task surface `{first}` (from `{slug}`) must resolve to a catalog task"
            );
        }
    }
}
