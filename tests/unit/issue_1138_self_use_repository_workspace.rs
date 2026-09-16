//! Issue #1138, plan 14 **wave F leaf F-1** — self-use observations for plan 03.
//!
//! The five held-out repository prompts of plan 03, and its second held-out
//! family against a foreign non-Rust tree, were given to Formal AI through the
//! real `@link-assistant/agent` CLI and through `formal-ai chat`. Neither family
//! names a file or a constant, so reaching the target requires locating it.
//!
//! For the second family the CLI's throwaway workspace was seeded with the
//! three-file Python fixture at
//! `docs/case-studies/issue-1138/self-use/fixtures/python-timeout`, which
//! contains exactly one `DEFAULT_TIMEOUT = 30` and a test that asserts it —
//! the `literal_occurrence_locates_in_a_python_tree` shape plan 03 describes.
//!
//! Observations and prompts: `data/benchmarks/self-use-repository-workspace.lino`.
//! Transcripts: `docs/case-studies/issue-1138/self-use/`.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::solver::solve;

const CORPUS: &str = "data/benchmarks/self-use-repository-workspace.lino";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn unquote(raw: &str) -> String {
    let trimmed = raw.trim();
    let inner = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed);
    inner.replace("\"\"", "\"")
}

fn corpus() -> String {
    fs::read_to_string(repo_root().join(CORPUS))
        .unwrap_or_else(|error| panic!("{CORPUS} should be readable: {error}"))
}

fn family(id: &str) -> Vec<(String, String)> {
    let text = corpus();
    let mut out = Vec::new();
    let mut current = String::new();
    let mut language = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("id ") {
            current = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("language ") {
            language = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("prompt ") {
            if current == id {
                out.push((language.clone(), unquote(value)));
            }
        }
    }
    assert_eq!(
        out.len(),
        5,
        "family `{id}` must carry one prompt per registered language in {CORPUS}"
    );
    out
}

fn field(id: &str, name: &str) -> String {
    let text = corpus();
    let mut current = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("id ") {
            current = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix(&format!("{name} ")) {
            if current == id {
                return unquote(value);
            }
        }
    }
    panic!("{CORPUS} should carry `{name}` for family `{id}`");
}

/// **Wave F observation, plan 03 leaf F-1.** A prompt naming a forty-character
/// base commit is a clone spec. The commit is ignored in every language.
///
/// English and Russian go straight to a shell command in the throwaway workspace
/// and fail with `could not find Cargo.toml in /private/var/folders/…/tmp.…` —
/// the system assumed the repository was the current directory, never asked for
/// it, and never mentioned the commit it had been handed. Spanish web-searched
/// the sentence and opened an unrelated repository's documentation.
///
/// The minimum this test asks for is that the commit the prompt names survives
/// into the answer, in some form: as a clone the system says it will make, or as
/// a question about a repository it does not have.
#[test]
fn a_named_base_commit_is_not_silently_dropped() {
    let base = field("commit_spec_ignored", "base_commit");
    let short = &base[..12];
    let mut offenders = Vec::new();
    for (language, prompt) in family("commit_spec_ignored") {
        let answer = solve(&prompt).answer;
        if !answer.contains(&base) && !answer.contains(short) {
            let head: String = answer.trim().chars().take(140).collect();
            offenders.push(format!("{language}: {head}"));
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 03: a base commit in the prompt is a workspace spec, not decoration. \
         Answers that never mention `{short}`:\n{}",
        offenders.join("\n")
    );
}

/// **Wave F observation, plan 03.** The target declaration is never named.
///
/// The prompts name neither `WEB_SEARCH_PROVIDERS` nor `src/web_search_core.rs`;
/// that is the point — locating it is the task. Nothing in any of the five
/// answers names the declaration, the file, or any candidate for either, so
/// nothing was located and nothing could have been edited.
#[test]
fn the_target_declaration_is_located_and_named() {
    let declaration = field("commit_spec_ignored", "target_declaration");
    let file = field("commit_spec_ignored", "target_file");
    let mut offenders = Vec::new();
    for (language, prompt) in family("commit_spec_ignored") {
        let answer = solve(&prompt).answer;
        if !answer.contains(&declaration) && !answer.contains(&file) {
            offenders.push(language);
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 03: the prompt names neither `{declaration}` nor `{file}` — locating one \
         of them is the task. Languages that named neither: {}",
        offenders.join(", ")
    );
}

/// **Wave F observation, plan 03, `literal_occurrence_locates_in_a_python_tree`.**
/// The CLI's workspace held a three-file Python tree with exactly one
/// `DEFAULT_TIMEOUT = 30` and a test asserting it. Not located in any language,
/// and the search itself was malformed:
///
/// - English ran `find` **with no arguments** and got the usage message back.
///   That was the whole run.
/// - Russian ran `grep` with the prompt sentence as the pattern and got
///   `No files found`.
///
/// The request text was used as the search pattern instead of the thing the
/// request is about. `DEFAULT_TIMEOUT` is still `30` in every captured
/// workspace: nothing was edited and no test was run.
#[test]
fn a_single_declaration_in_a_foreign_tree_is_located() {
    let declaration = field("single_occurrence_not_located", "target_declaration");
    let file = field("single_occurrence_not_located", "target_file");
    let mut offenders = Vec::new();
    for (language, prompt) in family("single_occurrence_not_located") {
        let answer = solve(&prompt).answer;
        if !answer.contains(&declaration) && !answer.contains(&file) {
            offenders.push(language);
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 03: a tree with exactly one occurrence of `{declaration}` resolves. \
         Languages that named neither it nor `{file}`: {}",
        offenders.join(", ")
    );
}

/// **Wave F guard, plan 03.** The Python fixture the CLI was handed must keep its
/// committed value, so a later run that *does* edit it is visibly a change rather
/// than a fixture that had already drifted. `sixty seconds` is what the prompts
/// ask for; `30` is what the fixture holds.
#[test]
fn the_python_fixture_still_holds_the_value_the_prompts_ask_to_change() {
    let fixture = repo_root().join(field("single_occurrence_not_located", "fixture"));
    let source = fs::read_to_string(fixture.join("client.py"))
        .expect("the wave F python fixture should be readable");
    assert!(
        source.contains("DEFAULT_TIMEOUT = 30"),
        "the fixture must start at 30 so a run that raises it to 60 is visible as an edit"
    );
    assert_eq!(
        source.matches("DEFAULT_TIMEOUT =").count(),
        1,
        "the fixture must hold exactly one definition, or `LiteralOccurrence` is \
         ambiguous by construction and the case proves nothing"
    );
}
