//! Plan 16 L5 (issue #1138): the round trip the pivot can prove today. A
//! medium is faithful exactly when anything that crosses it comes back
//! identical, so every committed ES source round-trips through the pivot
//! *document* — serialize, parse, render to each target, extract back —
//! and the declared-projection registry in the seed stays in agreement
//! with the leg table the code answers from, so neither surface can name
//! a leaf that has already landed and disclaimed the leg.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::es_meta;
use formal_ai::meta_translate::{self, SourceRoot, TranslationOutcome};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

fn collect_owned(directory: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_owned(&path, extension, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            out.push(path);
        }
    }
    out.sort();
}

fn rendered(outcome: TranslationOutcome, context: &str) -> String {
    match outcome {
        TranslationOutcome::Rendered { target, .. } => target,
        TranslationOutcome::Refused { refusals } => {
            panic!("{context} must not refuse over the committed corpus: {refusals:?}")
        }
        TranslationOutcome::Pending { plan_leaf } => {
            panic!("{context} must be live, not pending on {plan_leaf}")
        }
        TranslationOutcome::Invalid { reason } => {
            panic!("{context} must normalize into the pivot: {reason}")
        }
    }
}

/// The leaf's own sentence, executed at the fidelity each quadrant owns:
/// for every committed js and ts source, `lino → ts → lino` and
/// `lino → js → lino` are CST-identical — the lino being the rendered
/// pivot document of the committed source. CST equality is kind, text and
/// shape (spans ignored, per the L2 definition) plus `token_count`; the
/// header fields `target` and `language` are provenance and survive whole
/// only on the identity legs. Corpus-wide is the point: the bundle files
/// carry strings, escapes and template literals, so the serialize/parse
/// pair is proven on the worst corpus the repository has.
#[test]
fn every_committed_es_source_round_trips_through_the_pivot_document() {
    let root = root();
    for (dir, extension, from) in [
        ("js", "js", SourceRoot::JavaScript),
        ("ts", "ts", SourceRoot::TypeScript),
    ] {
        let mut sources = Vec::new();
        collect_owned(&root.join(dir), extension, &mut sources);
        assert!(
            sources.len() >= 60,
            "the committed {dir} corpus is never empty: {count} files",
            count = sources.len()
        );
        for source_path in &sources {
            let display = source_path
                .strip_prefix(&root)
                .expect("the source lives under the repository root")
                .to_string_lossy()
                .replace('\\', "/");
            let source = fs::read_to_string(source_path).expect("the committed source is readable");
            let lino = rendered(
                meta_translate::translate(from, SourceRoot::Meta, &display, &source),
                &display,
            );
            let document = es_meta::parse_document(&lino).unwrap_or_else(|error| {
                panic!("{display}: the rendered pivot must parse: {error}")
            });
            for to in [SourceRoot::JavaScript, SourceRoot::TypeScript] {
                let target = rendered(
                    meta_translate::translate(SourceRoot::Meta, to, &display, &lino),
                    &display,
                );
                let back = rendered(
                    meta_translate::translate(to, SourceRoot::Meta, &display, &target),
                    &display,
                );
                let back_document = es_meta::parse_document(&back).unwrap_or_else(|error| {
                    panic!("{display}: the re-extracted pivot must parse: {error}")
                });
                assert_eq!(
                    document.trees, back_document.trees,
                    "{display}: the {to:?} render moved the token tree"
                );
                assert_eq!(
                    document.token_count, back_document.token_count,
                    "{display}: the {to:?} render changed the token count"
                );
                if to == from {
                    assert_eq!(
                        document, back_document,
                        "{display}: the identity leg preserves the whole document"
                    );
                }
            }
        }
    }
}

/// The seed's declared-projection registry and the code's leg table are two
/// surfaces of one truth: every directed pair of distinct roots declares
/// exactly one row, live rows are live in code, owed rows name the same
/// leaf `pending_leg` names, and the fidelity spellings are known. A
/// skipped or drifted row is a missing row here, so seed drift is red.
#[test]
fn the_seed_registry_and_the_leg_table_agree() {
    let projections = meta_translate::root_projections();
    assert_eq!(
        projections.len(),
        12,
        "every distinct directed pair declares exactly one row"
    );
    for (from, to, pending) in meta_translate::directions() {
        let row = projections
            .iter()
            .find(|row| row.from == from && row.to == to)
            .unwrap_or_else(|| panic!("the seed must declare {from:?} → {to:?}"));
        match pending {
            None => assert_eq!(
                row.owed_by, None,
                "{from:?} → {to:?} is live in code, so the seed must say live"
            ),
            Some(leaf) => assert_eq!(
                row.owed_by.as_deref(),
                Some(leaf),
                "{from:?} → {to:?} owes {leaf} in code, so the seed must name it"
            ),
        }
        assert!(
            matches!(
                row.fidelity.as_str(),
                "signature" | "token_tree" | "network"
            ),
            "known fidelity spellings only: {}",
            row.fidelity
        );
    }
    // The pointer history, kept honest: the legs that render source text of
    // another grammar name the leaf that owes them — L7 inherited the
    // quadrant from L3 (the dogfood loop) and L5 (verification), landed the
    // meta → rust network leg, and passed the grammar-projection legs to L8.
    // L8's rule table now answers for every js/ts corpus kind — ruled,
    // spliced or declared no-form — so the js/ts → rust renders are live:
    // they render the plain subset and refuse objects, classes and the
    // dynamic family by name. The rust → es legs stay owed: the table
    // still leaves a type-position tail.
    for (from, to) in [
        (SourceRoot::Rust, SourceRoot::JavaScript),
        (SourceRoot::Rust, SourceRoot::TypeScript),
    ] {
        assert_eq!(
            meta_translate::pending_leg(from, to),
            Some("L8"),
            "{from:?} → {to:?} is owed by the grammar projection rules"
        );
    }
    for (from, to) in [
        (SourceRoot::JavaScript, SourceRoot::Rust),
        (SourceRoot::TypeScript, SourceRoot::Rust),
    ] {
        assert_eq!(
            meta_translate::pending_leg(from, to),
            None,
            "{from:?} → {to:?} is live through the L8 grammar projection"
        );
    }
    // The rust row states the honest fidelity: the self-AST census is a
    // signature projection, live, and no rust row renders es source text
    // live — the rust → es legs exist only as rows owed to L7.
    let rust_row = projections
        .iter()
        .find(|row| row.from == SourceRoot::Rust && row.to == SourceRoot::Meta)
        .expect("rust → meta is declared");
    assert_eq!(rust_row.fidelity, "signature");
    assert_eq!(rust_row.owed_by, None);
    // L7 landed the reverse leg at full network fidelity: the lossless
    // serialization in, reconstructed rust out.
    let meta_rust_row = projections
        .iter()
        .find(|row| row.from == SourceRoot::Meta && row.to == SourceRoot::Rust)
        .expect("meta → rust is declared");
    assert_eq!(meta_rust_row.fidelity, "network");
    assert_eq!(meta_rust_row.owed_by, None);
    assert!(
        projections
            .iter()
            .filter(|row| row.from == SourceRoot::Rust)
            .all(|row| row.to == SourceRoot::Meta || row.owed_by.is_some()),
        "no row may claim rust renders es source text live"
    );
}

/// The registry is parsed from the seed, not a copy of it — the L2 pattern
/// restated for the root rows: whatever the data says is what the reader
/// returns.
#[test]
fn the_registry_moves_with_the_seed_not_a_copy_of_it() {
    let seed = concat!(
        "language_projection\n",
        "  root_projection probe_leg\n",
        "    from js\n",
        "    to rust\n",
        "    fidelity token_tree\n",
        "    status owed_by L7\n",
    );
    let rules = es_meta::projection_rules_from(seed);
    assert_eq!(rules.roots.len(), 1, "the probe row is the only root row");
    assert_eq!(rules.roots[0].from, "js");
    assert_eq!(rules.roots[0].to, "rust");
    assert_eq!(rules.roots[0].fidelity, "token_tree");
    assert_eq!(rules.roots[0].status, "owed_by L7");
    // A row with an empty field is not a declaration — the reader skips it
    // rather than half-carrying it, and the agreement check above is what
    // makes the skip loud.
    let broken = concat!(
        "language_projection\n",
        "  root_projection broken_leg\n",
        "    from js\n",
        "    to rust\n",
        "    status live\n",
    );
    assert!(
        es_meta::projection_rules_from(broken).roots.is_empty(),
        "a row missing its fidelity declares nothing"
    );
}
