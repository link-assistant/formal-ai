//! Plan 16 L7 (issue #1138): rust rendering from the pivot, landed at the
//! fidelity the engine already guarantees. The meta-language dependency
//! ships a lossless network serialization (`to_lino`/`from_lino`), and
//! `reconstruct_text` is byte-identical (issue #558), so the meta → rust
//! leg is wiring: every owned Rust module survives
//! source → network lino → source byte-for-byte through the public
//! translate surface, and a document in any other lino dialect — the
//! self-AST census, the ES token-tree pivot — is refused with the expected
//! dialect named, never mis-parsed into invented source. The legs that
//! render another grammar's source text (rust → js/ts, js/ts → rust) stay
//! owed to L8 until grammar projection rules exist.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::meta_translate::{self, SourceRoot, TranslationOutcome};

/// A small, ordinary Rust module the dialect probes round-trip — real
/// syntax, not a special case.
const PROBE: &str =
    "fn dialect_probe(sealed: bool) -> u32 {\n    if sealed { 7 } else { 41 + 1 }\n}\n";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

/// The owned corpus is the census set, derived: every committed census
/// document names its source module one-to-one, so walking the census tree
/// is walking `rust/src` exactly as the repository defines ownership.
fn collect_owned(directory: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_owned(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("lino") {
            out.push(path);
        }
    }
    out.sort();
}

fn rendered(outcome: TranslationOutcome, context: &str) -> String {
    match outcome {
        TranslationOutcome::Rendered { target, .. } => target,
        TranslationOutcome::Refused { refusals } => {
            panic!("{context} must not refuse: {refusals:?}")
        }
        TranslationOutcome::Pending { plan_leaf } => {
            panic!("{context} must be live, not pending on {plan_leaf}")
        }
        TranslationOutcome::Invalid { reason } => {
            panic!("{context} must render from the network document: {reason}")
        }
    }
}

/// The leaf's own sentence: the full-fidelity rust leg is the lossless
/// serialization the engine already guarantees, proven corpus-wide. Every
/// owned module — derived from the committed census tree, not selected —
/// goes source → `to_lino` → `from_lino` + `reconstruct_text` through the
/// public translate surface and must come back byte-identical. The biggest
/// modules are the point, not an exception: a serialization that drops a
/// span anywhere fails on the file that has the most of them.
#[test]
fn every_owned_rust_module_round_trips_through_the_network_serialization() {
    let root = root();
    let mut census = Vec::new();
    collect_owned(
        &root.join("data").join("meta").join("self-ast"),
        &mut census,
    );
    assert!(
        census.len() >= 600,
        "the owned corpus is the census set and is never empty: {count} files",
        count = census.len()
    );
    for census_path in &census {
        let relative = census_path
            .strip_prefix(root.join("data").join("meta").join("self-ast"))
            .expect("the census document lives under the census tree")
            .with_extension("rs");
        let source_path = root.join("rust").join(&relative);
        let display = relative.to_string_lossy().replace('\\', "/");
        let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
            panic!("rust/{display}: every census document names a source module: {error}")
        });
        let lino = formal_ai::agentic_coding::self_ast::network_lino(&source);
        let back = rendered(
            meta_translate::translate(SourceRoot::Meta, SourceRoot::Rust, &display, &lino),
            &display,
        );
        assert_eq!(
            back, source,
            "rust/{display}: the network round trip must be byte-identical"
        );
    }
}

/// The Meta root carries three lino dialects and the rust leg reads exactly
/// one: a census document (what rust → meta renders) and a token-tree pivot
/// document (what the ES legs carry) are both refused with the expected
/// dialect named — the leaf's honest-gap contract.
#[test]
fn meta_to_rust_refuses_every_other_lino_dialect_by_name() {
    assert_eq!(
        meta_translate::pending_leg(SourceRoot::Meta, SourceRoot::Rust),
        None
    );
    let census = rendered(
        meta_translate::translate(SourceRoot::Rust, SourceRoot::Meta, "probe.rs", PROBE),
        "the census probe",
    );
    let pivot = rendered(
        meta_translate::translate(
            SourceRoot::JavaScript,
            SourceRoot::Meta,
            "probe.js",
            "value;\n",
        ),
        "the pivot probe",
    );
    for (dialect, document) in [("census", &census), ("token-tree pivot", &pivot)] {
        match meta_translate::translate(SourceRoot::Meta, SourceRoot::Rust, "probe", document) {
            TranslationOutcome::Invalid { reason } => {
                assert!(
                    reason.contains("network serialization"),
                    "the {dialect} refusal must name the expected dialect: {reason}"
                );
            }
            other => panic!("the {dialect} document must be refused, not {other:?}"),
        }
    }
}

/// The registry row for the landed leg — `network` fidelity, live — and the
/// leg it declares really renders through the public surface.
#[test]
fn meta_to_rust_is_declared_network_and_live() {
    let row = meta_translate::root_projections()
        .into_iter()
        .find(|row| row.from == SourceRoot::Meta && row.to == SourceRoot::Rust)
        .expect("meta → rust is declared");
    assert_eq!(row.fidelity, "network");
    assert_eq!(row.owed_by, None);
    let lino = formal_ai::agentic_coding::self_ast::network_lino(PROBE);
    let back = rendered(
        meta_translate::translate(SourceRoot::Meta, SourceRoot::Rust, "probe.rs", &lino),
        "the probe",
    );
    assert_eq!(back, PROBE);
}
