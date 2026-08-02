//! Issue #673: the self-AST census must cover the whole workspace, not one
//! pinned module.
//!
//! Before this change `data/meta/self-ast.lino` described exactly one file
//! (`src/agentic_coding/planner.rs`), so a self-coding planner could not
//! introspect anything else. The tests below assert the three acceptance
//! criteria of the issue:
//!
//! 1. `data/meta/self-ast/` holds a census for *every* `src/` module, each
//!    carrying its fidelity marker, and the workspace index resolves any
//!    `path:symbol` the method registry knows;
//! 2. the census regenerates deterministically and incrementally, and the drift
//!    check fails on a fixture whose census is stale;
//! 3. the general planner resolves an edit target in a module *other than*
//!    `planner.rs` through the census index.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::method_registry::MethodRegistry;
use formal_ai::protocol::ChatMessage;
use formal_ai::self_ast_census::{
    document_path_for, drift_report, workspace, CensusDrift, CensusFidelity, WorkspaceCensus,
    CENSUS_DIR, FULL_FIDELITY_PREFIX, INDEX_PATH,
};

/// The repository root, so the tests read the *committed* census rather than a
/// path relative to the (unspecified) working directory.
fn repository_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Regeneration hint reused by every drift assertion.
const REGENERATE: &str = "run `cargo run --example regenerate_self_ast_census`";

#[test]
fn every_owned_module_has_a_committed_census_with_its_fidelity_marker() {
    let census = workspace();
    assert!(
        census.module_count() > 100,
        "the workspace census still looks single-module: {} modules",
        census.module_count()
    );
    for module in &census.modules {
        let document = repository_root().join(module.document_path());
        let contents = fs::read_to_string(&document).unwrap_or_else(|error| {
            panic!("missing census for {}: {error} — {REGENERATE}", module.path)
        });
        let expected_fidelity = if module.path.starts_with(FULL_FIDELITY_PREFIX) {
            CensusFidelity::FullAst
        } else {
            CensusFidelity::Signature
        };
        assert_eq!(
            module.fidelity, expected_fidelity,
            "wrong fidelity tier for {}",
            module.path
        );
        assert!(
            contents.contains(&format!("  fidelity {}\n", expected_fidelity.slug())),
            "census for {} carries no fidelity marker",
            module.path
        );
        assert!(
            contents.contains(&format!("  target {}\n", module.path)),
            "census for {} does not name its target",
            module.path
        );
    }
    assert!(
        census.full_fidelity_count() > 0,
        "no module was censused at full AST fidelity"
    );
    assert!(
        census.full_fidelity_count() < census.module_count(),
        "every module was censused at full fidelity — the tiering is not doing its job"
    );
}

#[test]
fn committed_census_documents_match_what_the_sources_render() {
    // The drift guard from issue #559, applied to the census: what is committed
    // must be byte-for-byte what the live sources render.
    let root = repository_root();
    let expected = workspace().documents();
    let committed: Vec<(String, String)> = collect_committed(root)
        .into_iter()
        .map(|path| {
            let contents = fs::read_to_string(root.join(&path)).expect("read committed census");
            (path, contents)
        })
        .collect();
    let drift = drift_report(&expected, &committed);
    assert!(
        drift.is_empty(),
        "committed self-AST census has drifted ({REGENERATE}):\n{}",
        drift
            .iter()
            .map(CensusDrift::describe)
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        root.join(INDEX_PATH).is_file(),
        "the workspace census index is missing — {REGENERATE}"
    );
}

#[test]
fn drift_check_fails_on_a_fixture_with_a_stale_census() {
    // A fixture workspace of two modules, censused, then perturbed three ways:
    // a stale document, a missing one, and an orphan nobody owns.
    let census = WorkspaceCensus::compile(&[
        ("src/alpha.rs", "pub fn alpha() -> u8 {\n    1\n}\n"),
        ("src/beta.rs", "pub struct Beta;\n"),
    ]);
    let expected = census.documents();
    assert!(
        drift_report(&expected, &expected).is_empty(),
        "a census compared against itself must not drift"
    );

    let mut committed = expected.clone();
    committed[0].1.push_str("stale trailing line\n");
    let stale = drift_report(&expected, &committed);
    assert_eq!(
        stale.len(),
        1,
        "expected exactly one stale document, got {stale:?}"
    );
    assert!(
        matches!(&stale[0], CensusDrift::Stale { document } if *document == expected[0].0),
        "stale census not reported as stale: {stale:?}"
    );

    let missing = drift_report(&expected, &expected[1..]);
    assert!(
        matches!(&missing[0], CensusDrift::Missing { document } if *document == expected[0].0),
        "missing census not reported as missing: {missing:?}"
    );

    let mut with_orphan = expected.clone();
    with_orphan.push((
        document_path_for("src/gone.rs"),
        "self_ast_census\n  target src/gone.rs\n".to_owned(),
    ));
    let orphan = drift_report(&expected, &with_orphan);
    assert!(
        matches!(&orphan[0], CensusDrift::Orphan { document } if document.ends_with("gone.lino")),
        "orphaned census not reported as an orphan: {orphan:?}"
    );
}

#[test]
fn census_regenerates_deterministically_and_incrementally() {
    let files: [(&str, &str); 3] = [
        ("src/alpha.rs", "pub fn alpha() -> u8 {\n    1\n}\n"),
        ("src/beta.rs", "pub struct Beta;\n"),
        ("src/agentic_coding/gamma.rs", "pub fn gamma() {}\n"),
    ];
    let first = WorkspaceCensus::compile(&files).documents();
    let again = WorkspaceCensus::compile(&files).documents();
    assert_eq!(first, again, "census regeneration is not deterministic");

    // One changed module must rewrite exactly one module document (plus the
    // index, which summarises the workspace) — the incremental property.
    let mut changed = files;
    changed[1].1 = "pub struct Beta;\npub fn beta() {}\n";
    let after = WorkspaceCensus::compile(&changed).documents();
    let rewritten: Vec<&str> = first
        .iter()
        .zip(&after)
        .filter(|(before, now)| before.1 != now.1)
        .map(|(before, _)| before.0.as_str())
        .collect();
    assert_eq!(
        rewritten,
        vec![INDEX_PATH, document_path_for("src/beta.rs").as_str()],
        "a one-module change re-censused the wrong set of documents"
    );

    // Ordering is path-sorted regardless of input order, so regeneration never
    // shuffles the committed tree.
    let mut shuffled = files;
    shuffled.reverse();
    assert_eq!(
        WorkspaceCensus::compile(&shuffled).documents(),
        first,
        "census output depends on input order"
    );
}

#[test]
fn the_index_resolves_every_path_symbol_the_method_registry_knows() {
    let census = workspace();
    let source = dispatch_sources();
    let registry = MethodRegistry::from_dispatch();
    assert!(
        registry.methods.len() > 50,
        "method registry looks empty: {}",
        registry.methods.len()
    );
    let mut unresolved = Vec::new();
    for method in &registry.methods {
        let declared = |identifier: &str| {
            !identifier.is_empty()
                && census.modules_declaring(identifier).iter().any(|module| {
                    module
                        .symbol(identifier)
                        .is_some_and(|symbol| symbol.kind == "function")
                })
        };
        let Some(symbol) = entry_point(&source, &method.name, &declared) else {
            unresolved.push(format!("{} (no entry point)", method.name));
            continue;
        };
        let declaring = census.modules_declaring(&symbol);
        let Some(module) = declaring.first() else {
            unresolved.push(format!("{} -> {symbol} (undeclared)", method.name));
            continue;
        };
        let reference = format!("{}:{symbol}", module.path);
        let resolution = census
            .resolve(&reference)
            .unwrap_or_else(|| panic!("census index cannot resolve {reference}"));
        assert_eq!(resolution.module_path, module.path);
        let span = resolution
            .symbol
            .unwrap_or_else(|| panic!("{reference} resolved without a span"));
        assert!(
            span.start_line >= 1 && span.end_line >= span.start_line,
            "nonsensical span for {reference}: {span:?}"
        );
    }
    assert!(
        unresolved.is_empty(),
        "the census index does not resolve every registry method: {unresolved:?}"
    );
}

#[test]
fn the_planner_resolves_an_edit_target_outside_planner_rs_via_the_census() {
    // The whole point of the census for E35: an edit request that addresses the
    // workspace — by a module suffix with a directory component, or by a
    // `path:symbol` pair — is resolved to the module's real path through the
    // census index, with no hardcoded paths and nothing about `planner.rs`. Each
    // row names a *different* module and uses a different phrasing/language so a
    // pass proves the resolution is general (CONTRIBUTING rule 4).
    for (prompt, expected_path) in [
        (
            "In self_source_links.rs:owned_source_files, change hello to goodbye",
            "src/self_source_links.rs",
        ),
        (
            "Replace foo with bar in self_ast_census.rs:resolve",
            "src/self_ast_census.rs",
        ),
        (
            "замени привет на пока в файле agentic_coding/source_links.rs",
            "src/agentic_coding/source_links.rs",
        ),
    ] {
        assert_ne!(
            expected_path, "src/agentic_coding/planner.rs",
            "the point is to resolve a module other than the pinned planner"
        );
        assert!(
            workspace().module(expected_path).is_some(),
            "{expected_path} is not censused"
        );
        let messages = vec![ChatMessage::user(prompt)];
        let calls = match plan_chat_step(&messages, &["edit", "read_file"]) {
            Some(AgenticPlan::ToolCalls(calls)) => calls,
            other => panic!("expected a tool call for {prompt:?}, got {other:?}"),
        };
        assert_eq!(calls.len(), 1, "expected one call for {prompt:?}");
        let value: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
        assert_eq!(
            value["path"].as_str().unwrap(),
            expected_path,
            "edit target for {prompt:?} was not resolved through the census index"
        );
    }
}

#[test]
fn the_workspace_census_is_addressable_without_a_multi_megabyte_seed() {
    // "Scale honestly": every module is addressable, but only the agentic-coding
    // tier pays for a full AST, so the committed seed stays small enough to read.
    let root = repository_root();
    let documents = collect_committed(root);
    let total: usize = documents
        .iter()
        .map(|path| {
            usize::try_from(
                fs::metadata(root.join(path))
                    .expect("committed census document")
                    .len(),
            )
            .expect("census document size fits in usize")
        })
        .sum();
    assert!(
        total < 2 * 1024 * 1024,
        "the committed census grew to {total} bytes — the fidelity tiering is not holding"
    );
    for path in &documents {
        let lines = fs::read_to_string(root.join(path)).unwrap().lines().count();
        assert!(
            lines <= 1500,
            "{path} has {lines} lines, over the repository's `.lino` guard"
        );
    }
    // The pinned single-module artifact (R381) is untouched by the workspace
    // census; both live side by side.
    assert!(
        root.join("data/meta/self-ast.lino").is_file(),
        "the pinned R381 self-AST artifact disappeared"
    );
}

/// Every committed census document, repository-relative and sorted.
fn collect_committed(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    walk(&root.join(CENSUS_DIR), &mut found);
    let mut paths: BTreeSet<String> = BTreeSet::new();
    for absolute in found {
        paths.insert(
            absolute
                .strip_prefix(root)
                .expect("census document lives in the repository")
                .to_string_lossy()
                .replace('\\', "/"),
        );
    }
    paths.into_iter().collect()
}

fn walk(directory: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("lino") {
            out.push(path);
        }
    }
}

/// The dispatch sources the method registry is derived from. `solver_dispatch`
/// is `pub(crate)`, so the test reads the modules off disk exactly as the
/// registry's own tests do.
fn dispatch_sources() -> String {
    let mut joined = String::new();
    for path in ["src/solver_dispatch.rs", "src/meta_method_dispatch.rs"] {
        joined.push_str(
            &fs::read_to_string(repository_root().join(path)).expect("read dispatch source"),
        );
    }
    joined
}

/// The identifier the dispatch source binds to the method named `name`: the
/// first identifier after the quoted method name that the census knows as a
/// declared function.
fn entry_point(source: &str, name: &str, declared: &dyn Fn(&str) -> bool) -> Option<String> {
    let quoted = format!("\"{name}\"");
    let mut cursor = 0;
    while let Some(offset) = source[cursor..].find(&quoted) {
        let after = cursor + offset + quoted.len();
        let rest = source[after..].trim_start();
        let rest = rest
            .strip_prefix("=>")
            .or_else(|| rest.strip_prefix(','))
            .map(|rest| rest.trim_start().trim_start_matches('{').trim_start());
        if let Some(rest) = rest {
            let identifier: String = rest
                .chars()
                .take_while(|character| character.is_alphanumeric() || *character == '_')
                .collect();
            if declared(&identifier) {
                return Some(identifier);
            }
        }
        cursor = after;
    }
    None
}
