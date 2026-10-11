//! PR #1188, R1188-U7: the concise lexeme form reads as the long form.
//!
//! `lexeme en "read" "read the file"`, with the fields every surface shares as
//! its children and long word lists continued on `words` lines, is expanded
//! by `seed::expand_concise_lexemes` before `seed::parse_lino` builds the tree,
//! so every loader sees one `surface` per word (`docs/links-notation-style.md`).
//! The fixtures are shared with `rust/tests/web/concise-lexemes.test.mjs`,
//! which pins the JavaScript twin in `js/seed_loader.js`.

use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use formal_ai::lino_adapters::links_notation::LinoNode;
use formal_ai::seed::{expand_concise_lexemes, parse_lino};

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/concise-lexemes")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// One line per node, children indented: the structure, nothing lexical.
fn canonical(node: &LinoNode, depth: usize) -> String {
    let mut out = format!("{}{}\t{}\n", "  ".repeat(depth), node.name, node.id);
    for child in &node.children {
        out.push_str(&canonical(child, depth + 1));
    }
    out
}

#[test]
fn the_concise_and_the_long_form_parse_to_one_tree() {
    let concise = parse_lino(&fixture("concise.lino"));
    let long = parse_lino(&fixture("long.lino"));
    assert_eq!(canonical(&concise, 0), canonical(&long, 0));
}

#[test]
fn every_word_becomes_a_surface_with_the_shared_fields() {
    let tree = parse_lino(&fixture("concise.lino"));
    let actor = &tree.children[0].children[0];
    let english = actor
        .children
        .iter()
        .find(|child| child.name == "lexeme" && child.id == "en")
        .expect("the English lexeme");
    let surface = &english.children[0];
    assert_eq!(surface.find_child_value("text"), "actor");
    assert_eq!(surface.find_child_value("part_of_speech"), "noun");
    let spanish = actor
        .children
        .iter()
        .find(|child| child.name == "lexeme" && child.id == "es")
        .expect("the Spanish lexeme");
    let words: Vec<&str> = spanish
        .children
        .iter()
        .map(|surface| surface.find_child_value("text"))
        .collect();
    assert_eq!(words, ["uno", "dos", "tres", "cuatro"]);
}

#[test]
fn a_document_without_the_concise_form_is_read_as_written() {
    let long = fixture("long.lino");
    assert!(matches!(expand_concise_lexemes(&long), Cow::Borrowed(_)));
    let lexicon = "lexeme old\n  work tale\n  kind entity\n";
    assert_eq!(expand_concise_lexemes(lexicon), lexicon);
}
