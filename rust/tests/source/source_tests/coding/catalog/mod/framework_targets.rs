//! A catalog row is an *implementation target*: either a language, or a
//! framework written in one of the catalogued languages (issue #723, which
//! asked for Laravel and was answered in PHP). These guards keep that widening
//! honest — a framework must name a base language that actually exists, must
//! resolve ahead of that base language when a request names both, and must not
//! change what the base language answers on its own.
use super::*;

#[test]
fn every_framework_names_a_catalogued_base_language() {
    for language in PROGRAM_LANGUAGES {
        let Some(base_slug) = language.framework_of else {
            continue;
        };
        let base = program_language_by_slug(base_slug).unwrap_or_else(|| {
            panic!(
                "framework `{}` is a framework of `{base_slug}`, absent from PROGRAM_LANGUAGES",
                language.slug
            )
        });
        assert!(
            !base.is_framework(),
            "framework `{}` names `{base_slug}` as its base, which is itself a framework — \
             `base_language` resolves one hop, so a chain would silently stop short",
            language.slug
        );
    }
}

#[test]
fn base_language_is_the_row_itself_for_a_language() {
    for language in PROGRAM_LANGUAGES {
        if language.is_framework() {
            continue;
        }
        assert_eq!(
            language.base_language().slug,
            language.slug,
            "`{}` is not a framework, so it is its own base language",
            language.slug
        );
    }
}

#[test]
fn a_framework_resolves_to_its_own_row_and_the_base_language_to_its_own() {
    // The exact shape issue #723 reported: `напиши мне код на PHP Laravel`
    // names both, and the framework is the more specific of the two. Answering
    // in the base language throws away the part of the request that was
    // hardest to satisfy, so the framework wins — while a request that names
    // only the base language is untouched by the framework row existing.
    let laravel = program_language_by_slug("laravel").expect("laravel is catalogued");
    assert!(laravel.is_framework(), "laravel is a framework target");
    assert_eq!(laravel.base_language().slug, "php");

    assert_eq!(
        program_language_by_alias("напиши мне код на php laravel").map(|target| target.slug),
        Some("laravel")
    );
    assert_eq!(
        program_language_by_alias("write me php code").map(|target| target.slug),
        Some("php")
    );
}

#[test]
fn every_framework_target_carries_its_own_execution_surface() {
    // A framework inherits its base language's grammar and composable idioms,
    // but never the answers the request actually asked for: the file to save
    // and the command to run are the framework's own, or the answer is just
    // the base language wearing a different name.
    for language in PROGRAM_LANGUAGES {
        let Some(base_slug) = language.framework_of else {
            continue;
        };
        let base = program_language_by_slug(base_slug).expect("checked above");
        assert_ne!(
            language.save_as, base.save_as,
            "framework `{}` saves to the same file as `{base_slug}`",
            language.slug
        );
        assert_ne!(
            language.execution.run_command, base.execution.run_command,
            "framework `{}` runs the same command as `{base_slug}`",
            language.slug
        );
        assert_ne!(
            language.setup_hint, base.setup_hint,
            "framework `{}` needs the same setup as `{base_slug}`",
            language.slug
        );
    }
}
