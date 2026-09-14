use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const SLICE: usize = 20;
// These upstream entry points are also ordinary language/operators used
// throughout the runtime; forbidding them would flag prose rather than
// benchmark knowledge. Compound/specific callable names remain forbidden.
const GENERIC_FUNCTION_NAMES: [&str; 3] = ["longest", "max", "sum"];

#[test]
fn upstream_slice_names_and_sentences_are_absent_from_runtime_and_seed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cache = benchmark_cache(root);
    let humaneval = cache.join("humaneval.jsonl");
    let mbpp = cache.join("mbpp.jsonl");
    if !humaneval.is_file() || !mbpp.is_file() {
        eprintln!(
            "SKIP no-memorization corpus gate: cache both {} and {} with the external benchmark runner",
            humaneval.display(),
            mbpp.display()
        );
        return;
    }

    let mut forbidden_names = BTreeSet::new();
    let mut forbidden_sentences = BTreeSet::new();
    for value in json_lines(&humaneval) {
        if let Some(name) = value.get("entry_point").and_then(serde_json::Value::as_str) {
            forbidden_names.insert(name.to_owned());
        }
        if let Some(prompt) = value.get("prompt").and_then(serde_json::Value::as_str) {
            for sentence in docstring_sentences(prompt) {
                if sentence.split_whitespace().count() > 4 {
                    forbidden_sentences.insert(sentence);
                }
            }
        }
    }
    for value in json_lines(&mbpp) {
        if let Some(text) = value.get("text").and_then(serde_json::Value::as_str) {
            forbidden_sentences.insert(normalize(text));
        }
        if let Some(tests) = value.get("test_list").and_then(serde_json::Value::as_array) {
            for test in tests {
                if let Some(name) = test
                    .as_str()
                    .and_then(|line| line.trim().strip_prefix("assert "))
                    .and_then(|assertion| assertion.split_once('(').map(|(name, _)| name.trim()))
                {
                    forbidden_names.insert(name.to_owned());
                }
            }
        }
    }
    forbidden_names.retain(|name| !GENERIC_FUNCTION_NAMES.contains(&name.as_str()));

    let mut violations = Vec::new();
    for path in source_and_seed_files(root) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let normalized = normalize(&text);
        for name in &forbidden_names {
            if contains_identifier(&text, name) {
                violations.push(format!(
                    "{} contains upstream entry point {name}",
                    path.display()
                ));
            }
        }
        for sentence in &forbidden_sentences {
            if !sentence.is_empty() && normalized.contains(sentence) {
                violations.push(format!(
                    "{} contains upstream task sentence {sentence:?}",
                    path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "benchmark-specific knowledge entered runtime or seed:\n{}",
        violations.join("\n")
    );
}

fn benchmark_cache(root: &Path) -> PathBuf {
    let repository_cache = root.join("target/formal-ai-benchmarks");
    if repository_cache.is_dir() {
        return repository_cache;
    }
    std::env::var_os("CARGO_TARGET_DIR").map_or(repository_cache, |target| {
        PathBuf::from(target).join("formal-ai-benchmarks")
    })
}

fn json_lines(path: &Path) -> Vec<serde_json::Value> {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(SLICE)
        .map(|line| serde_json::from_str(line).expect("upstream JSONL record"))
        .collect()
}

fn docstring_sentences(prompt: &str) -> Vec<String> {
    let Some((_, rest)) = prompt.split_once("\"\"\"") else {
        return Vec::new();
    };
    let Some((docstring, _)) = rest.split_once("\"\"\"") else {
        return Vec::new();
    };
    docstring
        .split(['.', '!', '?', '\n'])
        .map(normalize)
        .filter(|sentence| !sentence.is_empty() && !sentence.starts_with(">>>"))
        .collect()
}

fn source_and_seed_files(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.join("src"), root.join("data/seed")];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            let mut children = fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect::<Vec<_>>();
            children.sort();
            pending.extend(children);
        } else if path.is_file() {
            files.push(path);
        }
    }
    files
}

fn contains_identifier(text: &str, needle: &str) -> bool {
    text.match_indices(needle).any(|(start, found)| {
        let before = text[..start].chars().next_back();
        let after = text[start + found.len()..].chars().next();
        !before.is_some_and(identifier_character) && !after.is_some_and(identifier_character)
    })
}

const fn identifier_character(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '_' {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
