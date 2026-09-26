//! Placement rules for the modules issue #1138 plan 02 adds
//! (`src/coding/program_ir.rs` and the modules that ship with it).
//!
//! Three rules, each of which the repository already enforces elsewhere and
//! which a new module is easiest to break on day one:
//!
//! * a module stays under the 1,000-line Rust ceiling
//!   (`scripts/check-file-size.rs`),
//! * its unit tests live in `tests/source/source_tests/<module>/`, never inline
//!   in the implementation file, and
//! * it is registered in `src/coding/mod.rs` and reachable from `src/lib.rs`,
//!   so nothing is dead code the census cannot see.
//!
//! The fourth assertion is the one that is red until wave I4 lands: a module in
//! the tree carries no unimplemented leaf.

use std::fs;
use std::path::{Path, PathBuf};

/// The modules plan 02 adds, in the order the plan's leaves add them.
const NEW_MODULES: [&str; 6] = [
    "rust/src/coding/program_ir.rs",
    "rust/src/coding/fragment_catalog.rs",
    "rust/src/coding/composition_search.rs",
    "rust/src/coding/ir_lowering/mod.rs",
    "rust/src/coding/ir_lowering/python.rs",
    "rust/src/coding/ir_lowering/rust.rs",
];

/// `scripts/check-file-size.rs:21-27`.
const RUST_FILE_CEILING: usize = 1_000;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repository_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn every_new_module_stays_under_the_rust_file_ceiling() {
    for relative in NEW_MODULES {
        let lines = read(relative).lines().count();
        assert!(
            lines <= RUST_FILE_CEILING,
            "{relative} has {lines} lines, over the {RUST_FILE_CEILING}-line ceiling"
        );
    }
}

#[test]
fn no_new_module_keeps_its_unit_tests_inline() {
    for relative in NEW_MODULES {
        let source = read(relative);
        assert!(
            !source.contains("#[cfg(test)]"),
            "{relative} keeps tests inline; they belong in tests/source/source_tests/"
        );
    }
}

#[test]
fn every_new_module_is_registered_and_reachable() {
    let coding = read("rust/src/coding/mod.rs");
    let library = read("rust/src/lib.rs");
    for module in [
        "program_ir",
        "fragment_catalog",
        "composition_search",
        "ir_lowering",
    ] {
        assert!(
            coding.contains(&format!("pub mod {module};")),
            "`{module}` is not registered in rust/src/coding/mod.rs"
        );
        assert!(
            library.contains(&format!("pub use coding::{module};")),
            "`{module}` is not reachable from the crate root"
        );
    }
}

#[test]
fn no_new_module_ships_an_unimplemented_leaf() {
    let mut pending = Vec::new();
    for relative in NEW_MODULES {
        let source = read(relative);
        for line in source.lines() {
            if line.trim().starts_with("todo!(") {
                pending.push(format!("{relative}: {}", line.trim()));
            }
        }
    }
    assert!(
        pending.is_empty(),
        "these leaves are declared but not implemented:\n{}",
        pending.join("\n")
    );
}
