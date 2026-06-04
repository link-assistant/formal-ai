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
        let slug = meaning
            .slug
            .strip_prefix("program_task_")
            .unwrap_or_else(|| {
                panic!(
                    "`{}` carries the task-alias role but is not a `program_task_<slug>` meaning",
                    meaning.slug
                )
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
