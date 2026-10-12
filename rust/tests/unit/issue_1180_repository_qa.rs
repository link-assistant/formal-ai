//! Issue #1180 R10: status and definition questions answered from this
//! repository's history and its self-AST census.
//!
//! "Why does the self-development status fail?" is answered from the rule's
//! checking script (and the issue that introduced it), the commit range the
//! script measures, and the floor and target the self-hosting ledger records.
//! "What does `evaluate_calculation` do?" is answered by locating the symbol in
//! `data/meta/self-ast`, quoting its documentation comment from the source,
//! and naming the commit that first wrote its name (`git log -S`). Both cues
//! claim only with their subject present (issue #1175).

use std::path::{Path, PathBuf};

use formal_ai::FormalAiEngine;
use formal_ai::history_context::{
    self, CensusSymbol, HistoryRules, definition_subjects, status_subject, symbol_doc,
};

fn repo_root() -> PathBuf {
    std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("..")).expect("repo root")
}

fn shallow(root: &Path) -> bool {
    let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--is-shallow-repository"])
        .current_dir(root)
        .output()
    else {
        return true;
    };
    String::from_utf8_lossy(&output.stdout).trim() == "true"
}

#[test]
fn the_seed_names_the_status_rule_and_the_census() {
    let rules = HistoryRules::load(None);
    let qa = &rules.repository_qa;
    assert_eq!(
        qa.status_script,
        "scripts/check-self-development-release.rs"
    );
    assert_eq!(qa.status_ledger, "data/meta/self-hosting-ledger.lino");
    assert_eq!(qa.status_tag_match, "v[0-9]*");
    assert_eq!(qa.census_dir, "data/meta/self-ast/src");
    assert_eq!(qa.source_root, "rust");
    assert_eq!(qa.doc_prefix, "///");
    assert_eq!(qa.attribute_prefix, "#[");
    let languages = |cues: &[(String, Vec<String>)]| -> Vec<String> {
        cues.iter().map(|(language, _)| language.clone()).collect()
    };
    assert_eq!(
        languages(&qa.status_cues),
        vec!["en", "ru", "hi", "zh", "es"]
    );
    assert_eq!(
        languages(&qa.definition_cues),
        vec!["en", "ru", "hi", "zh", "es"]
    );
}

#[test]
fn a_cue_alone_never_names_a_subject() {
    let rules = HistoryRules::load(None);
    assert_eq!(
        status_subject("Why does the self-development status fail?", &rules),
        Some("self-development status".to_owned())
    );
    assert_eq!(status_subject("Why does the build fail?", &rules), None);
    assert_eq!(
        definition_subjects("What does `evaluate_calculation` do?", &rules),
        vec!["evaluate_calculation".to_owned()]
    );
    assert_eq!(
        definition_subjects("Что делает handleWordProblem?", &rules),
        vec!["handleWordProblem".to_owned()]
    );
    // A plain word is a census name too ("parse", "new"), so it is not read
    // as a symbol; nor is an identifier without a definition cue.
    assert_eq!(
        definition_subjects("What does parse do?", &rules),
        Vec::<String>::new()
    );
    assert_eq!(
        definition_subjects("Explain evaluate_calculation", &rules),
        Vec::<String>::new()
    );
}

#[test]
fn the_documentation_comment_above_the_item_is_quoted() {
    let rules = HistoryRules::load(None);
    let source = "fn before() {}\n\n/// Add two numbers.\n/// Overflow wraps.\n///\n/// Details nobody needs.\n#[must_use]\npub fn add(a: u8, b: u8) -> u8 {\n    a.wrapping_add(b)\n}\n";
    assert_eq!(
        symbol_doc(source, 8, &rules),
        Some("Add two numbers. Overflow wraps.".to_owned())
    );
    assert_eq!(symbol_doc(source, 1, &rules), None);
}

#[test]
fn the_census_locates_evaluate_calculation() {
    let root = repo_root();
    let rules = HistoryRules::load(None);
    let found: Vec<CensusSymbol> = history_context::census_symbols(&root, &rules)
        .into_iter()
        .filter(|symbol| symbol.name == "evaluate_calculation")
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].target, "src/calculation.rs");
    assert_eq!(found[0].kind, "function");
}

#[test]
fn chat_answers_what_a_function_does_from_the_census() {
    let root = repo_root();
    if shallow(&root) {
        eprintln!("skip: shallow clone has no searchable history");
        return;
    }
    let rules = HistoryRules::load(None);
    let symbol = history_context::census_symbols(&root, &rules)
        .into_iter()
        .find(|symbol| symbol.name == "evaluate_calculation")
        .expect("the census records evaluate_calculation");
    let expected = history_context::definition_answer(&root, &symbol, "en", &rules);
    let response = FormalAiEngine.answer("What does `evaluate_calculation` do?");
    assert_eq!(response.intent, "repository_definition");
    assert_eq!(response.answer, expected);
    let located = format!(
        "`evaluate_calculation` is a function in rust/src/calculation.rs, lines {} to {}. Its documentation comment reads: Evaluate an expression, delegating calculator-supported syntax to `link-calculator` and preserving the in-repo evaluator as a fallback for syntax the upstream crate does not support yet.",
        symbol.start, symbol.end
    );
    assert_eq!(&expected[..located.len()], located);
}

#[test]
fn chat_explains_why_the_self_development_status_fails() {
    let root = repo_root();
    if shallow(&root) {
        eprintln!("skip: shallow clone has no release tags to range from");
        return;
    }
    let rules = HistoryRules::load(None);
    let Some(expected) =
        history_context::status_answer(&root, "self-development status", "en", &rules)
    else {
        eprintln!("skip: no release tag is reachable from HEAD");
        return;
    };
    let response = FormalAiEngine.answer("Why does the self-development status fail?");
    assert_eq!(response.intent, "repository_status_explanation");
    assert_eq!(response.answer, expected);
    assert_eq!(
        &expected[..92],
        "The self-development status is checked by scripts/check-self-development-release.rs, introdu"
    );
    assert!(
        expected.contains("introduced for issue #1014."),
        "the rule's source issue comes from the script's introducing commit: {expected}"
    );
}
