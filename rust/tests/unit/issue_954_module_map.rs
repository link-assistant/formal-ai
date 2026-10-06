//! Issue #954 (E102): the architecture docs carry a generated "Module map"
//! (`docs/architecture/module-map.md`, §18 of the ARCHITECTURE.md contents)
//! covering every `pub mod` in `rust/src/lib.rs`, pinned so drift fails CI
//! — a new module fails until documented, a removed one leaves a stale row
//! behind that this test also catches. The companion generator is
//! `scripts/generate-module-map.rs` (`--check` here, `--write` to refresh).
//!
//! Run: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_954_`

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits inside the repository")
        .to_path_buf()
}

fn read(path: &str) -> String {
    let full = repo_root().join(path);
    fs::read_to_string(&full).unwrap_or_else(|error| panic!("{} should be readable: {error}", path))
}

/// The architecture topic file that owns §18 (ARCHITECTURE.md links to it).
const MODULE_MAP: &str = "docs/architecture/module-map.md";
const BEGIN: &str = "<!-- module-map:begin";
const END: &str = "<!-- module-map:end -->";

fn declared_modules() -> Vec<String> {
    read("rust/src/lib.rs")
        .lines()
        .filter_map(|line| {
            line.strip_prefix("pub mod ")
                .and_then(|rest| rest.strip_suffix(';'))
        })
        .map(str::to_string)
        .collect()
}

/// The map's first column: every table row whose first cell is a bare
/// `` `name` `` between the generated markers.
fn mapped_modules() -> Vec<String> {
    let architecture = read(MODULE_MAP);
    let begin = architecture
        .find(BEGIN)
        .expect("the module map carries the module-map begin marker");
    let end = architecture
        .find(END)
        .expect("the module map carries the module-map end marker");
    architecture[begin..end]
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix('|')?.trim();
            if rest.starts_with("---") || rest.starts_with("<!--") || rest.starts_with("Module |") {
                return None;
            }
            let (name, _) = rest.split_once('|')?;
            let name = name.trim().trim_matches('`');
            let shape = !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_');
            shape.then(|| name.to_string())
        })
        .collect()
}

#[test]
fn the_module_map_equals_the_librs_mod_list() {
    let declared = declared_modules();
    let mapped = mapped_modules();
    assert!(
        declared.len() >= 150,
        "the crate documents a large module surface (found {})",
        declared.len()
    );
    let missing: Vec<&String> = declared
        .iter()
        .filter(|name| !mapped.contains(name))
        .collect();
    let stale: Vec<&String> = mapped
        .iter()
        .filter(|name| !declared.contains(name))
        .collect();
    assert!(
        missing.is_empty() && stale.is_empty(),
        "{MODULE_MAP} (ARCHITECTURE.md §18) is out of sync with rust/src/lib.rs -- missing {missing:?}, stale {stale:?}; run `rust-script scripts/generate-module-map.rs --write`"
    );
}

#[test]
fn every_row_carries_a_responsibility_and_an_owning_section() {
    let architecture = read(MODULE_MAP);
    let begin = architecture.find(BEGIN).expect("begin marker");
    let end = architecture.find(END).expect("end marker");
    let mut rows = 0usize;
    for line in architecture[begin..end].lines() {
        let rest = match line.trim().strip_prefix('|') {
            Some(rest) => rest.trim(),
            None => continue,
        };
        if rest.starts_with("---") || rest.starts_with("Module |") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').collect();
        if cells.len() >= 4 {
            let responsibility = cells[2].trim();
            let section = cells[3].trim();
            assert!(
                responsibility.chars().count() >= 8,
                "a one-sentence responsibility, not an empty cell: {line}"
            );
            assert!(
                section.starts_with('§'),
                "the owning doc section is a §-number: {line}"
            );
            let number: f32 = section
                .trim_start_matches('§')
                .parse()
                .unwrap_or_else(|_| panic!("§ followed by a number: {line}"));
            assert!((2.0..=15.0).contains(&number), "a real section: {line}");
            rows += 1;
        }
    }
    assert_eq!(
        rows,
        declared_modules().len(),
        "one row per declared module, no more, no fewer"
    );
}

#[test]
fn the_generator_exists_and_pins_the_same_property() {
    let generator = read("scripts/generate-module-map.rs");
    for needle in ["--check", "--write", "module-map:begin", "pub mod "] {
        assert!(
            generator.contains(needle),
            "the generator carries {needle}: check/write modes, the same markers, the same declaration prefix"
        );
    }
}

#[test]
fn the_reorganization_manifest_records_the_measured_starting_point() {
    let manifest = read("docs/module-reorganization.md");
    assert!(
        manifest.contains("273 top-level entries"),
        "the entry count the < 60 CI leg starts from is recorded, not re-derived"
    );
    assert!(
        manifest.contains("world_model.rs` 1000"),
        "the worst cap-pressure file is named with its measured size"
    );
    for family in ["`solver/`", "`world_model/`", "`dreaming/`", "`translate/`"] {
        assert!(
            manifest.contains(family),
            "the {family} family move is specified"
        );
    }
}
