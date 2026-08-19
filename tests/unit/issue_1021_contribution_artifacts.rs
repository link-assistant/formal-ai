//! Issue #1021: the process artifacts a contribution carries.
//!
//! Formal AI wrote code and stopped there. A changelog fragment and a
//! pull-request body that closes its issue are as much a part of landing a
//! change here as the code is — `scripts/check-changelog-fragment.rs` and
//! `scripts/check-pull-request-link.rs` are gates, not etiquette — and the
//! issue records that it produced neither, so a human finished every change by
//! hand.
//!
//! These tests **drive the generator** rather than compare it against a
//! committed sample. A sample only proves that one string was once correct; the
//! rules below are read out of the gate scripts themselves, so if a gate
//! changes its mind about what it accepts, these tests change with it.

use std::fs;
use std::path::PathBuf;

use formal_ai::contribution_artifacts::{compose, Contribution};
use formal_ai::seed::contribution_artifact_vocabulary;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// A contribution with everything an author would supply.
fn contribution() -> Contribution {
    Contribution {
        issue: 1021,
        repository: "link-assistant/formal-ai".to_owned(),
        slug: "behaviour_range".to_owned(),
        timestamp: "20260819_120000".to_owned(),
        bump: "minor".to_owned(),
        category: "fixed".to_owned(),
        title: "Route the reported behaviour range by generalization".to_owned(),
        problem: "Seven reported prompts routed to nothing, or to the wrong command.".to_owned(),
        cause: "Each route read English phrases written into the router itself.".to_owned(),
        change: "Every cue moved to seed data and the routes now compose it.".to_owned(),
        verification: vec![
            "cargo test --test unit issue_1021".to_owned(),
            "rust-script scripts/check-hardcoded-language.rs".to_owned(),
        ],
    }
}

/// Read a `const NAME: &[&str] = &[...]` list out of a gate script, so the rules
/// these tests enforce are the gate's own and not a second copy of them.
fn string_list(source: &str, name: &str) -> Vec<String> {
    let start = source
        .find(&format!("const {name}"))
        .unwrap_or_else(|| panic!("{name} is no longer declared in the gate"));
    let body = &source[start..];
    // Skip the type annotation (`&[&str]`) and open at the initializer.
    let assignment = body.find('=').expect("list is initialized");
    let body = &body[assignment..];
    let open = body.find('[').expect("list opens");
    let close = body.find(']').expect("list closes");
    body[open + 1..close]
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

/// `scripts/check-changelog-fragment.rs` accepts a fragment by its path, and
/// `scripts/collect-changelog.rs` reads its frontmatter and body. The generated
/// fragment has to satisfy both, and the shape of it is not this test's opinion:
/// `changelog.d/README.md` documents it and the fragments already in the tree
/// follow it.
#[test]
fn a_composed_fragment_is_one_the_changelog_gate_accepts() {
    let artifacts = compose(&contribution()).expect("a fully specified contribution composes");
    let path = &artifacts.changelog_fragment_path;

    // The gate's own predicate, transcribed from `is_changelog_fragment`.
    assert!(path.starts_with("changelog.d/"), "{path}");
    // The gate matches the extension byte for byte, so this does too; clippy's
    // case-insensitive suggestion would accept a name the gate rejects.
    assert!(
        std::path::Path::new(path).extension() == Some(std::ffi::OsStr::new("md")),
        "{path}"
    );
    assert!(!path.ends_with("README.md"), "{path}");

    // The name carries the timestamp README.md asks for, and the issue it
    // answers, so `scripts/check-fragment-release-map.rs` can trace it back.
    let name = path
        .trim_start_matches("changelog.d/")
        .trim_end_matches(".md");
    let (stamp, rest) = name.split_at("20260819_120000".len());
    assert_eq!(stamp, "20260819_120000");
    assert_eq!(rest, "_issue_1021_behaviour_range");

    // Frontmatter as `strip_frontmatter` in `scripts/collect-changelog.rs`
    // reads it: a fence, one `bump:` line, a closing fence.
    let fragment = &artifacts.changelog_fragment;
    let mut lines = fragment.lines();
    assert_eq!(lines.next(), Some("---"));
    assert_eq!(lines.next(), Some("bump: minor"));
    assert_eq!(lines.next(), Some("---"));

    assert!(fragment.contains("### Fixed"), "{fragment}");
    assert!(
        fragment.contains("- Route the reported behaviour range by generalization"),
        "{fragment}"
    );
    assert!(fragment.ends_with('\n'), "a fragment ends with a newline");
}

/// Driving the generator across every bump and category the seed defines, so a
/// category added to `data/seed/contribution-artifacts.lino` is covered the day
/// it lands rather than the day someone remembers to extend this test.
#[test]
fn every_seeded_bump_and_category_composes_its_own_heading() {
    let vocab = contribution_artifact_vocabulary();
    assert!(!vocab.bumps.is_empty(), "the seed defines bumps");
    assert!(!vocab.categories.is_empty(), "the seed defines categories");

    for bump in &vocab.bumps {
        for category in &vocab.categories {
            let mut input = contribution();
            input.bump.clone_from(bump);
            input.category.clone_from(&category.name);
            let artifacts = compose(&input).expect("a seeded bump and category compose");
            assert!(
                artifacts
                    .changelog_fragment
                    .contains(&format!("bump: {bump}")),
                "{}",
                artifacts.changelog_fragment
            );
            assert!(
                artifacts
                    .changelog_fragment
                    .contains(&format!("### {}", category.heading)),
                "{}",
                artifacts.changelog_fragment
            );
        }
    }
}

/// A bump the release scripts do not recognise is a release that silently does
/// not happen, and a category they do not recognise is an entry that lands under
/// no heading. Composing nothing is the honest answer to both.
#[test]
fn an_unrecognised_bump_or_category_composes_nothing() {
    for (bump, category) in [
        ("major-ish", "fixed"),
        ("", "fixed"),
        ("patch", "improved"),
        ("patch", ""),
    ] {
        let mut input = contribution();
        input.bump = bump.to_owned();
        input.category = category.to_owned();
        assert!(
            compose(&input).is_none(),
            "bump {bump}, category {category}"
        );
    }
    // A contribution with nothing to say has no title to render either.
    let mut untitled = contribution();
    untitled.title = "   ".to_owned();
    assert!(compose(&untitled).is_none());
}

/// The rule `scripts/check-pull-request-link.rs` enforces: a recognised closing
/// keyword, followed by a reference GitHub resolves, and none of the words that
/// read like a link and close nothing. Both word lists are read out of the gate
/// so this test cannot drift away from it.
#[test]
fn a_composed_body_closes_its_issue_by_the_gates_own_rules() {
    let gate = read("scripts/check-pull-request-link.rs");
    let closing = string_list(&gate, "CLOSING_KEYWORDS");
    let non_closing = string_list(&gate, "NON_CLOSING_KEYWORDS");
    let artifacts = compose(&contribution()).expect("composes");
    let body = &artifacts.pull_request_body;
    let lowered = body.to_lowercase();

    let first = body.lines().next().expect("a body opens with its link");
    let (keyword, reference) = first.split_once(' ').expect("keyword then reference");
    assert!(
        closing.contains(&keyword.to_lowercase()),
        "{keyword} is not one of {closing:?}"
    );
    assert_eq!(
        reference.trim(),
        "https://github.com/link-assistant/formal-ai/issues/1021"
    );

    for word in &non_closing {
        assert!(
            !lowered.contains(&format!("{word} https://github.com/")),
            "the body links its issue with {word}, which closes nothing"
        );
        assert!(
            !lowered.contains(&format!("{word} #")),
            "the body links its issue with {word}, which closes nothing"
        );
    }
}

/// The body a reviewer reads: what broke, why, what changed, and how to check
/// it. The headings come from the seed, so this drives the generator against
/// whatever the seed currently says rather than against four literals.
#[test]
fn a_composed_body_answers_every_seeded_section() {
    let vocab = contribution_artifact_vocabulary();
    let input = contribution();
    let artifacts = compose(&input).expect("composes");
    let body = &artifacts.pull_request_body;

    assert!(!vocab.sections.is_empty(), "the seed defines sections");
    for section in &vocab.sections {
        assert!(
            body.contains(&format!("## {}", section.heading)),
            "no section {}: {body}",
            section.heading
        );
    }
    for sentence in [&input.problem, &input.cause, &input.change] {
        assert!(body.contains(sentence.as_str()), "{sentence} is missing");
    }
    for command in &input.verification {
        assert!(
            body.contains(&format!("`{command}`")),
            "{command} is missing"
        );
    }
    assert_eq!(artifacts.pull_request_title, input.title);
}

/// A section the author left empty is left out rather than rendered as a heading
/// over nothing: an empty heading in a body reads as an unanswered question.
#[test]
fn an_unanswered_section_is_omitted_rather_than_left_empty() {
    let mut input = contribution();
    input.cause = String::new();
    input.verification.clear();
    let body = compose(&input).expect("composes").pull_request_body;
    assert!(!body.contains("## Why"), "{body}");
    assert!(!body.contains("## Verification"), "{body}");
    assert!(body.contains("## What is broken"), "{body}");
    assert!(!body.contains("\n\n\n"), "no gap is left behind: {body}");
}

/// R379 applied to the generator itself: it composes English without containing
/// any. The property here is stricter than the gate's — the gate asks that no
/// literal *read as* user-facing prose, this asks that no literal contain two
/// plain words in a row at all, once the format placeholders are removed.
#[test]
fn the_generator_composes_prose_without_containing_any() {
    let source = read("src/contribution_artifacts.rs");
    for (number, line) in source.lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("//") || code.starts_with("///") {
            continue;
        }
        for literal in string_literals(code) {
            let stripped = strip_placeholders(&literal);
            let words: Vec<&str> = stripped
                .split(|c: char| !c.is_alphabetic())
                .filter(|word| word.len() > 1)
                .collect();
            assert!(
                words.len() < 2,
                "line {}: the literal {literal:?} carries prose ({words:?}) that belongs in \
                 data/seed/contribution-artifacts.lino",
                number + 1
            );
        }
    }
}

/// String literals on one line of Rust source, without their quotes.
fn string_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut characters = line.chars();
    while let Some(character) = characters.next() {
        if character != '"' {
            continue;
        }
        let mut literal = String::new();
        while let Some(next) = characters.next() {
            match next {
                '\\' => {
                    characters.next();
                    literal.push(' ');
                }
                '"' => break,
                _ => literal.push(next),
            }
        }
        literals.push(literal);
    }
    literals
}

/// Remove `{placeholder}` spans, which name a value rather than say anything.
fn strip_placeholders(literal: &str) -> String {
    let mut stripped = String::new();
    let mut depth = 0usize;
    for character in literal.chars() {
        match character {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => stripped.push(character),
            _ => {}
        }
    }
    stripped
}
