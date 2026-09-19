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
        if language == "en" {
            assert_eq!(
                answer,
                "This request routes to the `list_dir` capability, but this chat surface does not expose the required `shell` tool. Use an agent client that advertises it.\nRequest anchors preserved for that client: `74875c1b9b6e36bee9b942343ba295541fdb6997`."
            );
        }
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

/// The isolated chat call has no repository bytes. It must hand the request to
/// a workspace-capable client rather than fabricate a location. The actual
/// five-language location contract, with an adopted repository, is exercised by
/// `issue_1138_locate_targets::census_locates_a_declaration_the_prompt_never_names`.
#[test]
fn chat_hands_repository_location_to_a_workspace_capable_client() {
    let declaration = field("commit_spec_ignored", "target_declaration");
    let file = field("commit_spec_ignored", "target_file");
    let mut offenders = Vec::new();
    for (language, prompt) in family("commit_spec_ignored") {
        let response = solve(&prompt);
        let answer = response.answer;
        if language == "en" {
            assert_eq!(
                answer,
                "This request routes to the `list_dir` capability, but this chat surface does not expose the required `shell` tool. Use an agent client that advertises it.\nRequest anchors preserved for that client: `74875c1b9b6e36bee9b942343ba295541fdb6997`."
            );
        }
        if response.intent != "capability_gap" || !answer.contains("`shell`") {
            let head: String = answer.chars().take(140).collect();
            offenders.push(format!(
                "{language}: no typed workspace handoff (intent {}, answer {head:?})",
                response.intent
            ));
        }
        if answer.contains(&declaration) || answer.contains(&file) {
            offenders.push(format!("{language}: fabricated an unobserved location"));
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 03: chat has no repository bytes, so it must preserve the request and \
         request workspace capability, not claim `{declaration}` or `{file}`.\n{}",
        offenders.join(", ")
    );
}

/// An isolated chat request also cannot see the separately captured Python
/// fixture. It requests the workspace tool and makes no location claim. The
/// executable protocol over that fixture is pinned by
/// `issue_1138_locate_targets::literal_occurrence_locates_in_a_python_tree`.
#[test]
fn chat_does_not_invent_a_declaration_in_an_unseen_foreign_tree() {
    let declaration = field("single_occurrence_not_located", "target_declaration");
    let file = field("single_occurrence_not_located", "target_file");
    let mut offenders = Vec::new();
    for (language, prompt) in family("single_occurrence_not_located") {
        let response = solve(&prompt);
        let answer = response.answer;
        if language == "en" {
            assert_eq!(
                answer,
                "This request routes to the `grep` capability, but this chat surface does not expose the required `shell` tool. Use an agent client that advertises it."
            );
        }
        if response.intent != "capability_gap" || !answer.contains("`shell`") {
            let head: String = answer.chars().take(140).collect();
            offenders.push(format!(
                "{language}: no typed workspace handoff (intent {}, answer {head:?})",
                response.intent
            ));
        }
        if answer.contains(&declaration) || answer.contains(&file) {
            offenders.push(format!("{language}: fabricated an unobserved location"));
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 03: an unseen tree cannot honestly resolve `{declaration}` or `{file}`; \
         chat must request the workspace capability.\n{}",
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
