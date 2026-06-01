use std::fs;
use std::path::Path;

use lino_objects_codec::format::parse_indented;
use walkdir::WalkDir;

const MAX_LINO_LINES: usize = 1_500;

#[test]
fn lino_data_files_are_parseable_human_readable_and_bounded() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    assert!(data_dir.is_dir(), "data directory should exist");

    let mut checked_files = 0_usize;
    for entry in WalkDir::new(&data_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("lino") {
            continue;
        }

        checked_files += 1;
        let content = fs::read_to_string(path).expect("lino file should be UTF-8 text");
        let line_count = content.lines().count();
        assert!(
            line_count <= MAX_LINO_LINES,
            "{} has {line_count} lines, exceeding {MAX_LINO_LINES}",
            path.display()
        );
        assert!(
            !content.contains("(str ") && !content.contains("(object "),
            "{} should use indented human-readable Links Notation, not typed object encoding",
            path.display()
        );

        for record in split_records(&content) {
            parse_indented(record).unwrap_or_else(|error| {
                panic!(
                    "{} contains invalid Links Notation: {error}",
                    path.display()
                );
            });
        }
    }

    assert!(
        checked_files >= 3,
        "expected checked-in Links Notation seed data files"
    );
}

fn split_records(content: &str) -> Vec<&str> {
    content
        .split("\n\n")
        .map(str::trim)
        .filter(|record| !record.is_empty())
        .collect()
}

/// The browser demo ships a copy of the canonical Links Notation seed under
/// `src/web/seed/`, generated from `data/seed/*.lino` by `scripts/sync-seed.sh`.
/// The deploy and E2E pipelines re-run that copy before serving, which keeps the
/// *served* site correct but cannot catch a *committed* mirror that has silently
/// drifted from the canonical source — and the per-commit CI gate skips the E2E
/// re-sync entirely on docs-only commits. Issue #386 required the "apply the fix
/// in every place" invariant to be mechanically enforced so this class of drift
/// can never regress (the cancel-sort fix had to land in `data/seed/`,
/// `src/web/seed/`, and the worker at once). This test is the committed-tree
/// equivalent of `scripts/sync-seed.sh --check`, run in the PR-gated unit suite
/// on every OS so a forgotten re-sync fails fast instead of shipping stale data.
#[test]
fn web_seed_mirror_matches_canonical_data_seed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let canonical = root.join("data/seed");
    let mirror = root.join("src/web/seed");
    assert!(canonical.is_dir(), "data/seed/ should exist");
    assert!(mirror.is_dir(), "src/web/seed/ should exist");

    // sync-seed.sh copies the *top-level* `*.lino` files only — its glob does
    // not recurse into `data/seed/api-cache/`, which has no web mirror — so the
    // guard mirrors exactly that contract (read_dir + is_file skips subdirs).
    let mut canonical_names = Vec::new();
    let mut problems = Vec::new();

    for entry in fs::read_dir(&canonical).expect("read data/seed/") {
        let entry = entry.expect("data/seed/ entry");
        let path = entry.path();
        if !entry.file_type().expect("file type").is_file() {
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("lino") {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        canonical_names.push(name.clone());

        match fs::read(mirror.join(&name)) {
            Ok(mirror_bytes) => {
                let canonical_bytes = fs::read(&path).expect("read canonical seed");
                if canonical_bytes != mirror_bytes {
                    problems.push(format!("out of sync: {name}"));
                }
            }
            Err(_) => problems.push(format!("missing in src/web/seed/: {name}")),
        }
    }

    // Orphans: a mirror file with no canonical counterpart would silently ship.
    for entry in fs::read_dir(&mirror).expect("read src/web/seed/") {
        let entry = entry.expect("src/web/seed/ entry");
        let path = entry.path();
        if !entry.file_type().expect("file type").is_file() {
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("lino") {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !canonical.join(&name).is_file() {
            problems.push(format!("orphan in src/web/seed/: {name}"));
        }
    }

    assert!(
        !canonical_names.is_empty(),
        "expected canonical seed files under data/seed/"
    );
    assert!(
        problems.is_empty(),
        "src/web/seed/ has drifted from data/seed/ — run scripts/sync-seed.sh:\n{}",
        problems.join("\n")
    );
}
