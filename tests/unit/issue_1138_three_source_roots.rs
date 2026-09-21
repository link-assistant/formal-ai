//! Repository layout gate for issue #1138 (plan 16, leaf L1).
//!
//! The js→ts→rust development cycle needs three source roots: `./js` is
//! iterated by hand and by agent, `./ts` is generated from it, `./rust` is
//! the crate root that today sits at the repository top level. This test
//! pins the layout and the invariants that make the split honest while the
//! cycle is partial:
//!
//! - `./rust` holds `Cargo.toml`, `src/`, `tests/`, `benches/`, `examples/`;
//! - `./js` holds the hand-iterated first citizens (the worker mirrors and
//!   the seed loader that today live under `src/web/`);
//! - no path outside its own root claims to be a source of any layer, so
//!   the top level stops being an implicit fourth root;
//! - every reference that used to point at `src/web` resolves inside `./js`.
//!
//! This module is registered only when leaf L1 lands; before that it is an
//! inert draft recording the plan's shape.

use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .map(PathBuf::from)
        .expect("the crate root sits one level below the repository root")
}

#[test]
fn the_three_source_roots_exist() {
    let root = repository_root();
    for layer in ["js", "ts", "rust"] {
        let layer_root = root.join(layer);
        assert!(
            layer_root.is_dir(),
            "{layer}/ must exist as a source root of the development cycle"
        );
    }
}

#[test]
fn the_crate_lives_under_rust() {
    let rust = repository_root().join("rust");
    for required in ["Cargo.toml", "src", "tests"] {
        assert!(
            rust.join(required).exists(),
            "rust/{required} must move with the crate root"
        );
    }
    for optional in ["benches", "examples"] {
        // Present today; the gate does not demand they stay forever, only
        // that whatever exists of the crate lives inside rust/.
        if repository_root().join(optional).exists() {
            panic!("{optional}/ still sits at the repository top level");
        }
    }
}

#[test]
fn the_top_level_is_no_longer_a_source_root() {
    let root = repository_root();
    assert!(
        !root.join("Cargo.toml").exists(),
        "Cargo.toml must live in rust/, not at the top level"
    );
    assert!(
        !root.join("src").exists(),
        "src/ must live in rust/, not at the top level"
    );
}

#[test]
fn the_javascript_first_citizens_live_under_js() {
    let js = repository_root().join("js");
    for required in [
        "worker/formal_ai_worker.js",
        "seed_loader.js",
        "seed-files.js",
    ] {
        assert!(
            js.join(required).exists(),
            "js/{required} must move with the src/web first citizens"
        );
    }
    assert!(
        !repository_root().join("rust/src/web").exists(),
        "src/web must not survive inside rust/ after the move"
    );
}

#[test]
fn no_stale_src_web_references_remain() {
    let root = repository_root();
    let mut offenders: Vec<String> = Vec::new();
    for name in [".github", "scripts", "js"] {
        let directory = root.join(name);
        if !directory.is_dir() {
            continue;
        }
        for entry in walkdir_files(&directory) {
            let contents = fs::read_to_string(&entry).unwrap_or_default();
            if contents.contains("src/web") {
                offenders.push(
                    entry
                        .strip_prefix(&root)
                        .map(Path::display)
                        .unwrap_or_else(|_| entry.display())
                        .to_string(),
                );
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "files still referencing src/web after the move: {}",
        offenders.join(", ")
    );
}

fn walkdir_files(directory: &Path) -> Vec<PathBuf> {
    fn walk(directory: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(directory, &mut files);
    files
}
