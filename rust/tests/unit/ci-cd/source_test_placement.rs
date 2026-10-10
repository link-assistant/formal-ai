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
        if let Some(suffix) = attr.split_once("::")
            && (suffix.1.starts_with("test]") || suffix.1.starts_with("test("))
        {
            return true;
        }
    }
    false
}

#[derive(Debug)]
struct SourceToken<'a> {
    text: &'a str,
    start: usize,
    end: usize,
    depth: usize,
}

// Comments and literals are atomic: their braces and test-shaped text cannot
// establish an item boundary or a registration's structural depth.
fn source_tokens(source: &str) -> Option<Vec<SourceToken<'_>>> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut delimiters = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if source[index..].starts_with("//") {
            index = source[index..]
                .find('\n')
                .map_or(bytes.len(), |n| index + n + 1);
            continue;
        }
        if source[index..].starts_with("/*") {
            index += 2;
            let mut comments = 1;
            while comments > 0 {
                if index >= bytes.len() {
                    return None;
                }
                if source[index..].starts_with("/*") {
                    comments += 1;
                    index += 2;
                } else if source[index..].starts_with("*/") {
                    comments -= 1;
                    index += 2;
                } else {
                    index += source[index..].chars().next()?.len_utf8();
                }
            }
            continue;
        }
        let start = index;
        let depth = delimiters.len();
        let raw_start = if source[index..].starts_with("br") {
            index + 2
        } else if bytes[index] == b'r' {
            index + 1
        } else {
            index
        };
        let mut raw_quote = raw_start;
        while raw_quote < bytes.len() && bytes[raw_quote] == b'#' {
            raw_quote += 1;
        }
        if raw_start > index && bytes.get(raw_quote) == Some(&b'"') {
            let closing = format!("\"{}", "#".repeat(raw_quote - raw_start));
            index = raw_quote + 1;
            index += source[index..].find(&closing)? + closing.len();
        } else if bytes[index] == b'"' {
            index += 1;
            loop {
                match bytes.get(index)? {
                    b'\\' => {
                        index += 1;
                        index += source[index..].chars().next()?.len_utf8();
                    }
                    b'"' => {
                        index += 1;
                        break;
                    }
                    _ => index += source[index..].chars().next()?.len_utf8(),
                }
            }
        } else if bytes[index] == b'\'' {
            // A lifetime has no closing quote. A character (including escaped
            // punctuation) is consumed atomically rather than as a delimiter.
            let mut end = index + 1;
            if bytes.get(end) == Some(&b'\\') {
                end += 1;
                if bytes.get(end) == Some(&b'u') && bytes.get(end + 1) == Some(&b'{') {
                    end += source[end..].find('}')? + 1;
                } else if bytes.get(end) == Some(&b'x') {
                    end += 3;
                } else {
                    end += source[end..].chars().next()?.len_utf8();
                }
            } else {
                end += source[end..].chars().next()?.len_utf8();
            }
            index = if bytes.get(end) == Some(&b'\'') {
                end + 1
            } else {
                index + 1
            };
        } else {
            let first = source[index..].chars().next()?;
            index += first.len_utf8();
            if first == '_' || first.is_alphanumeric() {
                while index < bytes.len() {
                    let next = source[index..].chars().next()?;
                    if next != '_' && !next.is_alphanumeric() {
                        break;
                    }
                    index += next.len_utf8();
                }
            } else if matches!(first, '(' | '[' | '{') {
                delimiters.push(first);
            } else if matches!(first, ')' | ']' | '}') {
                let expected = match first {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                if delimiters.pop()? != expected {
                    return None;
                }
            }
        }
        tokens.push(SourceToken {
            text: &source[start..index],
            start,
            end: index,
            depth,
        });
    }
    delimiters.is_empty().then_some(tokens)
}

fn external_test_file(
    crate_root: &Path,
    source_path: &Path,
    literal: &str,
) -> Option<(PathBuf, String)> {
    use sha2::{Digest, Sha256};
    use std::path::Component;

    let relative = literal.strip_prefix('"')?.strip_suffix('"')?;
    if relative.is_empty()
        || relative.contains('\\')
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == ".")
    {
        return None;
    }
    let crate_root = crate_root.canonicalize().ok()?;
    let tests_root = crate_root.join("tests");
    let mut target = source_path.parent()?.canonicalize().ok()?;
    if !target.starts_with(&crate_root) {
        return None;
    }
    let mut entered_path = false;
    for component in Path::new(relative).components() {
        match component {
            Component::ParentDir if !entered_path && target != crate_root => {
                target.pop();
            }
            Component::Normal(part) => {
                entered_path = true;
                target.push(part);
                if fs::symlink_metadata(&target).ok()?.file_type().is_symlink() {
                    return None;
                }
            }
            _ => return None,
        }
    }
    if !target.starts_with(&tests_root)
        || target.extension()?.to_str()? != "rs"
        || !fs::symlink_metadata(&tests_root).ok()?.is_dir()
        || fs::symlink_metadata(&tests_root)
            .ok()?
            .file_type()
            .is_symlink()
        || !fs::symlink_metadata(&target).ok()?.is_file()
        || target.canonicalize().ok()? != target
    {
        return None;
    }
    let content = fs::read(&target).ok()?;
    let digest = formal_ai::source_fetch::hex_lower(&Sha256::digest(&content));
    if formal_ai::source_fetch::hex_lower(&Sha256::digest(fs::read(&target).ok()?)) != digest {
        return None;
    }
    Some((target, digest))
}

fn external_test_registrations(
    crate_root: &Path,
    source_path: &Path,
    source: &str,
) -> Vec<(usize, usize, PathBuf, String)> {
    let Some(tokens) = source_tokens(source) else {
        return Vec::new();
    };
    let mut registrations = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.depth != 0 || (index > 0 && !matches!(tokens[index - 1].text, ";" | "}")) {
            continue;
        }
        let Some(item) = tokens.get(index..index + 16) else {
            continue;
        };
        let fixed = [
            (0, "#"),
            (1, "["),
            (2, "cfg"),
            (3, "("),
            (4, "test"),
            (5, ")"),
            (6, "]"),
            (7, "#"),
            (8, "["),
            (9, "path"),
            (10, "="),
            (12, "]"),
            (13, "mod"),
            (15, ";"),
        ];
        if fixed
            .iter()
            .any(|(offset, expected)| item[*offset].text != *expected)
        {
            continue;
        }
        let name = item[14].text;
        if name.is_empty()
            || !name
                .chars()
                .next()
                .is_some_and(|c| c == '_' || c.is_alphabetic())
            || !name.chars().all(|c| c == '_' || c.is_alphanumeric())
        {
            continue;
        }
        let Some((path, digest)) = external_test_file(crate_root, source_path, item[11].text)
        else {
            continue;
        };
        registrations.push((token.start, item[15].end, path, digest));
    }
    registrations
}

fn inline_test_violations(crate_root: &Path, path: &Path, content: &str) -> Vec<String> {
    let registrations = external_test_registrations(crate_root, path, content);
    let mut violations = Vec::new();
    let mut offset = 0;
    for (index, chunk) in content.split_inclusive('\n').enumerate() {
        let line = chunk.trim_end_matches(['\r', '\n']);
        let external = registrations
            .iter()
            .any(|(start, end, _, _)| offset + line.len() > *start && offset < *end);
        if is_inline_test_marker(line.trim_start()) && !external {
            violations.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
        }
        offset += chunk.len();
    }
    violations
}

#[test]
fn src_has_no_inline_unit_tests() {
    let mut violations = Vec::new();
    for path in src_rust_paths() {
        let content = fs::read_to_string(&path).expect("source file should be UTF-8 text");
        violations.extend(inline_test_violations(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            &path,
            &content,
        ));
    }
    assert!(
        violations.is_empty(),
        "production code under src/ must not contain inline tests; move them to tests/:\n{}",
        violations.join("\n")
    );
}

#[test]
fn external_registration_proofs_preserve_the_original_physical_fixtures() {
    use sha2::{Digest, Sha256};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (source, fixture) in [
        (
            "src/agentic_coding/function_expectation.rs",
            "tests/fixtures/test-target-ownership.rs",
        ),
        (
            "src/agentic_coding/driver.rs",
            "tests/fixtures/algorithm-operation-receipts.rs",
        ),
    ] {
        let source_path = root.join(source);
        let content = fs::read_to_string(&source_path).expect("physical source");
        let registrations = external_test_registrations(root, &source_path, &content);
        let fixture_path = root.join(fixture).canonicalize().expect("physical fixture");
        let bytes = fs::read(&fixture_path).expect("complete fixture bytes");
        let expected = formal_ai::source_fetch::hex_lower(&Sha256::digest(bytes));
        assert_eq!(registrations.len(), 1, "one complete external declaration");
        assert_eq!(registrations[0].2, fixture_path);
        assert_eq!(registrations[0].3, expected, "whole fixture byte binding");
    }
}

#[test]
fn external_registration_is_structural_and_refuses_unproven_paths_or_bodies() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "external-test-registration-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).expect("fresh isolated root");
    fs::create_dir_all(root.join("src/nested")).expect("source directory");
    fs::create_dir(root.join("tests")).expect("test directory");
    let source_path = root.join("src/nested/module.rs");
    let fixture = "#[test]\nfn physical_assertion() { assert_eq!(2 + 2, 4); }\n";
    for name in ["renamed-alpha.rs", "renamed-beta.rs"] {
        fs::write(root.join("tests").join(name), fixture).expect("complete test fixture");
        let declaration =
            format!("#[cfg(test)]\n#[path = \"../../tests/{name}\"]\nmod arbitrary_name;");
        let registrations = external_test_registrations(&root, &source_path, &declaration);
        assert_eq!(registrations.len(), 1, "renamed physical registration");
        assert_eq!(fs::read_to_string(&registrations[0].2).unwrap(), fixture);
        assert_eq!(
            inline_test_violations(&root, &source_path, &declaration).len(),
            0
        );
        let contextual = format!(
            "const TEXT: &str = r###\"{{ #[cfg(test)] }}\"###;\n/* {{ nested /* }} */ }} */\n{declaration}"
        );
        assert_eq!(
            external_test_registrations(&root, &source_path, &contextual).len(),
            1
        );
    }
    for source in [
        "#[cfg(test)]\nfn inline() {}",
        "#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod inline {}",
        "#[unknown]\n#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod hidden;",
        "#[cfg(test)]\n#[unknown]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod hidden;",
        "mod nested {\n#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod hidden;\n}",
        "#[cfg(test)]\n#[path = \"../../tests/missing.rs\"]\nmod absent;",
        "#[cfg(test)]\n#[path = \"../../../escaped.rs\"]\nmod escaped;",
        "#[cfg(test)]\n#[path = \"../../tests/../tests/renamed-alpha.rs\"]\nmod redundant;",
        "#[cfg(test)]\n#[path = \"../../tests/./renamed-alpha.rs\"]\nmod redundant;",
        "#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod incomplete",
        "#[cfg(test)]\n#[path = \"../../tests/renamed-alpha.rs\"]\nmod hidden;\n}",
        "#[test]\nfn actual_inline_test() {}",
        "#[tokio::test]\nasync fn actual_inline_test() {}",
        "mod tests { fn body() {} }",
    ] {
        assert!(
            external_test_registrations(&root, &source_path, source).is_empty(),
            "must refuse {source}"
        );
        assert!(
            !inline_test_violations(&root, &source_path, source).is_empty(),
            "original detector must reject {source}"
        );
    }
    fs::remove_dir_all(root).expect("remove isolated fixtures");
}

#[cfg(unix)]
#[test]
fn external_registration_refuses_a_symlink_fixture() {
    let root = std::env::temp_dir().join(format!("external-test-symlink-{}", std::process::id()));
    fs::create_dir(&root).expect("fresh isolated root");
    fs::create_dir_all(root.join("src/nested")).expect("source directory");
    fs::create_dir(root.join("tests")).expect("test directory");
    fs::write(root.join("tests/actual.rs"), "#[test] fn physical() {}\n").unwrap();
    std::os::unix::fs::symlink(root.join("tests/actual.rs"), root.join("tests/alias.rs")).unwrap();
    let source = "#[cfg(test)]\n#[path = \"../../tests/alias.rs\"]\nmod alias;";
    assert_eq!(
        external_test_registrations(&root, &root.join("src/nested/module.rs"), source).len(),
        0
    );
    assert_ne!(
        inline_test_violations(&root, &root.join("src/nested/module.rs"), source).len(),
        0
    );
    fs::remove_dir_all(root).unwrap();
}
