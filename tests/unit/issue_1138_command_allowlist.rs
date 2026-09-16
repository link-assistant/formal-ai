//! Issue #1138 B7 (plan 03, L3): the repository allowlist is reviewable data.
//!
//! The default-deny arm survives verbatim; what changes is that widening the set
//! is an edit to `data/seed/repository-command-allowlist.lino` rather than an
//! edit to a hard-coded list in Rust. A program with no row is refused, and so
//! is a listed program used with an unlisted subcommand.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::repository_workspace::outcome::{
    CommandOutcome, describe_outcome, seed_sentence, seeded_languages,
};
use formal_ai::repository_workspace::{allows, command_allowlist};

const SEED: &str = "data/seed/repository-command-allowlist.lino";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// The programs the seed document names, read by the test rather than by the
/// code under test.
fn seed_programs() -> BTreeSet<String> {
    let text = fs::read_to_string(repo_root().join(SEED))
        .unwrap_or_else(|error| panic!("{SEED} should be readable: {error}"));
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("program ").map(|value| {
                value
                    .trim()
                    .trim_matches('"')
                    .to_owned()
            })
        })
        .collect()
}

/// A program with no row is refused, exactly as it is today.
#[test]
fn an_unlisted_program_is_still_refused() {
    assert!(
        !allows("curl", &["https://example.org"]),
        "the default-deny arm must survive: curl has no allowlist row"
    );
    assert!(
        !seed_programs().contains("curl"),
        "the seed table must not quietly gain a network fetcher"
    );
}

/// `git clone` is allowed and `git push` is not: a row is a program *and* a
/// subcommand, never a bare program.
#[test]
fn an_unlisted_git_subcommand_is_refused() {
    assert!(
        allows("git", &["clone", "--no-checkout", "owner/name", "/tmp/root"]),
        "cloning is how a repository task starts"
    );
    assert!(
        !allows("git", &["push", "origin", "main"]),
        "a repository task may never push; the allowlist is per subcommand"
    );
}

/// The set of allowed programs comes from the seed table, not from Rust.
#[test]
fn allowlist_rows_come_from_seed_not_from_rust() {
    let from_code: BTreeSet<String> = command_allowlist()
        .into_iter()
        .map(|row| row.program)
        .collect();
    assert_eq!(
        from_code,
        seed_programs(),
        "the allowlist the code enforces must be exactly the seed table's programs"
    );
    assert!(
        !from_code.is_empty(),
        "an empty allowlist would make the protocol unable to clone anything"
    );
    for row in command_allowlist() {
        assert!(
            row.subcommand.is_some(),
            "row `{}` must name a subcommand so the grant is not a whole program",
            row.id
        );
    }
}

/// **Wave F defect, plan 06 family 3 / plan 03 leaf L3.** The Russian prompt
/// *"Запусти это и скажи точно, что оно печатает: print(sum(range(1, 11)))"*
/// had its leading verb stripped and the remainder handed to `/bin/sh -c`. The
/// transcript records the shell answering
/// `/bin/sh: -c: line 0: syntax error near unexpected token '('`.
///
/// Prose is not a command. The first word of a stripped sentence names no
/// program, so under a default-deny table scoped to program *and* subcommand
/// *and* argument shape, nothing reaches a shell to fail there — in any of the
/// five languages the same prompt arrives in.
#[test]
fn prose_is_never_a_command() {
    const STRIPPED: &[(&str, &str)] = &[
        (
            "en",
            "this and tell me exactly what it prints: print(sum(range(1, 11)))",
        ),
        (
            "ru",
            "это и скажи точно, что оно печатает: print(sum(range(1, 11)))",
        ),
        (
            "hi",
            "इसे और मुझे ठीक-ठीक बताओ कि यह क्या छापता है: print(sum(range(1, 11)))",
        ),
        ("zh", "这个并准确告诉我它打印了什么：print(sum(range(1, 11)))"),
        (
            "es",
            "esto y dime exactamente qué imprime: print(sum(range(1, 11)))",
        ),
    ];

    for (language, prose) in STRIPPED {
        let tokens: Vec<&str> = prose.split_whitespace().collect();
        let (program, argv) = tokens
            .split_first()
            .expect("the observed prose is not empty");
        assert!(
            !allows(program, argv),
            "{language}: a sentence may never be run as a command; `{program}` names no program"
        );
        assert!(
            !seed_programs().contains(*program),
            "{language}: the seed table must not name a word of prose as a program"
        );
    }
}

/// **Wave F defect.** The same transcript called the failing command
/// `выполнена` — completed. A non-zero exit did not complete, and no surface may
/// say otherwise in any language.
#[test]
fn a_non_zero_exit_is_never_reported_as_completed() {
    const OBSERVED: &str = "/bin/sh: -c: line 0: syntax error near unexpected token `('";

    for language in ["en", "ru", "hi", "zh", "es"] {
        let failed = CommandOutcome::of_exit(Some(2), OBSERVED);
        assert!(
            !failed.is_completion(),
            "{language}: exit 2 is not a completion"
        );

        let sentence = describe_outcome(&failed, language);
        let completion = describe_outcome(
            &CommandOutcome::Completed {
                output: String::from(OBSERVED),
            },
            language,
        );
        assert!(
            !sentence.trim().is_empty(),
            "{language}: the honest sentence must be seeded, not improvised"
        );
        assert_ne!(
            sentence, completion,
            "{language}: a failure may not be rendered with the completion sentence"
        );
        assert!(
            sentence.contains('2'),
            "{language}: the observed exit status is quoted: {sentence}"
        );
        assert!(
            sentence.contains("syntax error"),
            "{language}: what the shell actually printed is shown: {sentence}"
        );
    }
}

/// The honest sentences are seed rows in five languages, not a `match` in Rust.
#[test]
fn the_outcome_sentences_are_seeded_in_five_languages() {
    for id in [
        "prose_is_not_a_command",
        "command_did_not_complete",
        "command_completed",
        "command_did_not_run",
        "command_timed_out",
        "unverified_execution",
    ] {
        let languages = seeded_languages(id);
        for language in ["en", "ru", "hi", "zh", "es"] {
            assert!(
                languages.iter().any(|seeded| seeded == language),
                "`{id}` must carry an `{language}` row; it carries {languages:?}"
            );
            assert!(
                !seed_sentence(id, language).trim().is_empty(),
                "`{id}`'s `{language}` row must say something"
            );
        }
    }
}
