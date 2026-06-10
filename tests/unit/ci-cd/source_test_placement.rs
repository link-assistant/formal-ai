// Issue #398 review (comment 4660584608), rule 7: production code under `src/`
// must not carry inline unit tests. Tests belong in `tests/` so the shipped
// crate stays free of `#[test]`/`#[cfg(test)]` scaffolding and so the test suite
// is discovered in one place.
//
// The guard scans every `.rs` file under `src/` for a *real* test attribute or a
// `mod tests` declaration. Attributes are anchored at the start of the trimmed
// line, so a test-shaped fragment that only appears inside a string literal
// (e.g. the sample sorting answer in `src/solver_helpers.rs`) is not flagged.

use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn src_rust_paths() -> Vec<PathBuf> {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .collect()
}

// A real attribute begins the (trimmed) line. String-literal occurrences such as
// `"#[test]\n..."` start the trimmed line with `"`, so they are not matched.
fn is_inline_test_marker(trimmed: &str) -> bool {
    if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("#[cfg(test)") {
        return true;
    }
    if trimmed.starts_with("mod tests") {
        return true;
    }
    // `#[test]`, `#[tokio::test]`, `#[test(...)]`, etc.
    if let Some(rest) = trimmed.strip_prefix("#[") {
        let attr = rest.trim_start();
        if attr == "test]" || attr.starts_with("test]") || attr.starts_with("test(") {
            return true;
        }
        if let Some(suffix) = attr.split_once("::") {
            if suffix.1.starts_with("test]") || suffix.1.starts_with("test(") {
                return true;
            }
        }
    }
    false
}

#[test]
fn src_has_no_inline_unit_tests() {
    let mut violations = Vec::new();
    for path in src_rust_paths() {
        let content = fs::read_to_string(&path).expect("source file should be UTF-8 text");
        for (index, line) in content.lines().enumerate() {
            if is_inline_test_marker(line.trim_start()) {
                violations.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "production code under src/ must not contain inline tests; move them to tests/:\n{}",
        violations.join("\n")
    );
}
