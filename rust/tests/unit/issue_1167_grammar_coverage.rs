//! Issue #1167 (E132), R2: every language the meta-language dependency
//! ships a grammar for AND that appears in `data/meta/hello-world-languages.lino`
//! has an entry in `data/seed/program-cst-grammars.lino`.
//!
//! The entries are generated, not hand-maintained:
//! `scripts/generate-program-cst-grammars.rs` derives the shipped set from
//! `cargo metadata` of the resolved meta-language package and fails `--check`
//! on drift. This test pins the same invariant from the data side — both
//! files are embedded/committed, so no network fetch is needed — and also
//! guards the other direction: a seed entry naming a slug the dependency
//! does not ship would assert a validation that never happens (the
//! `syntax_link_count` guard in `coding/cst.rs` exists for exactly that).

use formal_ai::seed::PROGRAM_CST_GRAMMARS_LINO;
use links_notation::LiNo;
use links_notation::parse_lino as parse_canonical_lino;

const HELLO_WORLD_LANGUAGES: &str = include_str!("../../../data/meta/hello-world-languages.lino");

/// The tracked grammar slugs the resolved meta-language 0.58.2 ships — the
/// pinned census `scripts/generate-program-cst-grammars.rs` derives live
/// from `cargo metadata`. Upstream shipping a new grammar is what changes
/// this list (and the seed), never a hand edit.
const SHIPPED_TRACKED_GRAMMAR_SLUGS: &[&str] = &[
    "c",
    "cpp",
    "csharp",
    "go",
    "java",
    "javascript",
    "kotlin",
    // tree-sitter-pascal; registered for the held-out Free Pascal path of
    // issue #1164 (R1164-9).
    "pascal",
    "php",
    "python",
    "r",
    "ruby",
    "rust",
    "scala",
    "swift",
    "typescript",
];

/// Every `name value` pair the canonical grammar found, in document order —
/// the same oracle walk `issue_715_renderer_artifacts` uses.
fn pairs(document: &str) -> Vec<(String, String)> {
    fn walk(node: &LiNo<String>, out: &mut Vec<(String, String)>) {
        let LiNo::Link { values, .. } = node else {
            return;
        };
        let refs: Vec<&str> = values
            .iter()
            .filter_map(|value| match value {
                LiNo::Ref(reference) => Some(reference.as_str()),
                LiNo::Link { .. } => None,
            })
            .collect();
        if let [name, value] = refs[..] {
            out.push((name.to_owned(), value.to_owned()));
        }
        for value in values {
            walk(value, out);
        }
    }
    let tree = parse_canonical_lino(document.trim())
        .unwrap_or_else(|error| panic!("document should parse: {error}"));
    let mut out = Vec::new();
    walk(&tree, &mut out);
    out
}

fn grammar_entry_slugs() -> Vec<String> {
    pairs(PROGRAM_CST_GRAMMARS_LINO)
        .into_iter()
        .filter(|(name, _)| name == "program_language")
        .map(|(_, slug)| slug)
        .collect()
}

fn hello_world_slugs() -> Vec<String> {
    pairs(HELLO_WORLD_LANGUAGES)
        .into_iter()
        .filter(|(name, _)| name == "slug")
        .map(|(_, slug)| slug)
        .collect()
}

#[test]
fn every_shipped_hello_world_language_has_a_grammar_entry() {
    // R2: Hello World languages with no shipped grammar (haskell, elixir,
    // clojure, fsharp, ocaml, erlang, julia) honestly carry no entry; every
    // other Hello World language must.
    let entries = grammar_entry_slugs();
    let missing: Vec<String> = hello_world_slugs()
        .into_iter()
        .filter(|slug| SHIPPED_TRACKED_GRAMMAR_SLUGS.contains(&slug.as_str()))
        .filter(|slug| !entries.contains(slug))
        .collect();
    assert!(
        missing.is_empty(),
        "these Hello World languages ship a meta-language grammar but have no \
         cst_grammar entry: {missing:?} — run `rust-script \
         scripts/generate-program-cst-grammars.rs --write`"
    );
}

#[test]
fn every_grammar_entry_names_a_shipped_grammar() {
    // The reverse guard: a seed entry the dependency cannot parse would be a
    // label, not a validation.
    let invented: Vec<String> = grammar_entry_slugs()
        .into_iter()
        .filter(|slug| !SHIPPED_TRACKED_GRAMMAR_SLUGS.contains(&slug.as_str()))
        .collect();
    assert!(
        invented.is_empty(),
        "these cst_grammar entries name slugs meta-language 0.58.2 does not ship: {invented:?}"
    );
}

#[test]
fn the_four_newly_registered_grammars_are_present() {
    // Kotlin and Scala sat on the declared-gap list since issue #921;
    // Swift and R were never covered at all. All four register now.
    let entries = grammar_entry_slugs();
    for slug in ["kotlin", "scala", "swift", "r"] {
        assert!(
            entries.iter().any(|entry| entry == slug),
            "`{slug}` must have a cst_grammar entry"
        );
    }
}

#[test]
fn grammar_entries_are_sorted_and_unique() {
    // The generator owns the order (sorted by slug); drift in ordering is
    // drift from the generator.
    let entries = grammar_entry_slugs();
    let mut sorted = entries.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        entries, sorted,
        "cst_grammar entries must be slug-sorted with no duplicates"
    );
}
