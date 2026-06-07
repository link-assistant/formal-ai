use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::seed::lexicon;
use lino_objects_codec::format::parse_indented;
use walkdir::WalkDir;

const LINKS_ROOT_SEED: &str = "data/seed/meanings-links-root.lino";
const SOURCE_KEYS: &[&str] = &[
    "grounded-in",
    "wikidata",
    "source-lexeme",
    "related-lexeme",
    "surface",
    "language",
    "lexical-category",
    "feature",
    "form",
    "sense",
];

#[test]
fn links_root_seed_uses_colon_definitions() {
    let root = repo_root();
    let path = root.join(LINKS_ROOT_SEED);
    let content = fs::read_to_string(&path).expect("links root seed should be readable");

    assert!(
        !content.contains("defined-by") && !content.contains("defined_by"),
        "{} must put core definitions in colon bodies, not defined-by child lines",
        path.display()
    );
    assert!(
        !content.contains("unformalized-raw"),
        "{} must not contain unresolved raw surfaces",
        path.display()
    );

    for old_name in [
        "reference_action",
        "link_action",
        "any_of_reference",
        "repeatable_from_zero",
        "self_equation",
    ] {
        assert!(
            !content.contains(old_name),
            "{} still contains underscore root symbol `{old_name}`",
            path.display()
        );
    }

    let mut links_root_bodies = Vec::new();
    let mut current: Option<(String, String)> = None;
    for line in content.lines() {
        if let Some((slug, body)) = top_level_colon_definition(line) {
            current = Some((slug, body));
            continue;
        }
        if strip_comment(line).trim() == "role links_root" {
            let Some((slug, body)) = current.clone() else {
                panic!(
                    "{} has role links_root outside a concept: {line}",
                    path.display()
                );
            };
            links_root_bodies.push((slug, body));
        }
    }

    assert!(
        links_root_bodies.len() >= 40,
        "{} should declare the recursive Links Theory root concepts",
        path.display()
    );
    for (slug, body) in links_root_bodies {
        assert!(
            !body.trim().is_empty(),
            "{} concept `{slug}` must have a non-empty colon definition body",
            path.display()
        );
        assert!(
            !body.contains('_'),
            "{} concept `{slug}` colon body still contains underscores: {body}",
            path.display()
        );
    }

    assert!(
        content.contains("  not: not (not not) # concept not"),
        "the core negation fixed-point definition must stay explicit"
    );
}

#[test]
fn semantic_definition_graph_is_closed() {
    let lex = lexicon();
    let slugs: BTreeSet<&str> = lex
        .meanings
        .iter()
        .map(|meaning| meaning.slug.as_str())
        .collect();
    let mut missing = Vec::new();

    for meaning in &lex.meanings {
        if meaning.defined_by.is_empty() {
            missing.push(format!("{} has no definition targets", meaning.slug));
            continue;
        }
        for target in &meaning.defined_by {
            if !slugs.contains(target.as_str()) {
                missing.push(format!("{} -> {target}", meaning.slug));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "semantic definition graph has dangling references:\n{}",
        missing.join("\n")
    );
}

#[test]
fn semantic_source_ids_have_checked_in_cache_records() {
    let root = repo_root();
    let seed_dir = root.join("data/seed");
    let wikidata_cache_dir = root.join("data/cache/wikidata");
    let mut missing = Vec::new();

    for path in meaning_seed_paths(&seed_dir) {
        let content = fs::read_to_string(&path).expect("meaning seed file should be readable");
        for (index, line) in content.lines().enumerate() {
            let Some(id) = source_id_from_line(line) else {
                continue;
            };
            let cache_path = wikidata_cache_dir.join(format!("{id}.lino"));
            if !cache_path.is_file() {
                missing.push(format!("{}:{} -> {}", path.display(), index + 1, id));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "semantic source ids are missing Wikidata cache files:\n{}",
        missing.join("\n")
    );
}

#[test]
fn wiktionary_cache_records_are_present_and_parseable() {
    let root = repo_root();
    let cache_dir = root.join("data/cache/wiktionary");
    assert!(
        cache_dir.is_dir(),
        "semantic grounding needs a checked-in Wiktionary cache directory"
    );

    let cache_files: Vec<PathBuf> = WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("lino"))
        .collect();
    assert!(
        !cache_files.is_empty(),
        "semantic grounding needs checked-in Wiktionary cache records"
    );

    for path in &cache_files {
        let content = fs::read_to_string(path).expect("wiktionary cache should be UTF-8");
        parse_indented(strip_lino_comments(&content).trim())
            .unwrap_or_else(|error| panic!("{} is invalid LiNo: {error}", path.display()));
    }

    let mut missing = Vec::new();
    for path in meaning_seed_paths(&root.join("data/seed")) {
        let content = fs::read_to_string(&path).expect("meaning seed file should be readable");
        for (index, line) in content.lines().enumerate() {
            let trimmed = strip_comment(line).trim();
            if let Some(id) = trimmed.strip_prefix("wiktionary ") {
                let cache_path = cache_dir.join(format!("{id}.lino"));
                if !cache_path.is_file() {
                    missing.push(format!("{}:{} -> {}", path.display(), index + 1, id));
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "semantic Wiktionary ids are missing cache files:\n{}",
        missing.join("\n")
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn meaning_seed_paths(seed_dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(seed_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("lino"))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("meanings"))
        })
        .collect()
}

fn source_id_from_line(line: &str) -> Option<String> {
    let trimmed = strip_comment(line).trim();
    let (key, raw) = trimmed.split_once(' ')?;
    if !SOURCE_KEYS.contains(&key) {
        return None;
    }
    let id = raw.split_whitespace().next()?;
    if id.starts_with("seed-surface-") || id.starts_with("WT-") {
        return None;
    }
    source_root_id(id)
}

fn source_root_id(id: &str) -> Option<String> {
    let mut chars = id.chars();
    let prefix = chars.next()?;
    if !matches!(prefix, 'L' | 'P' | 'Q') {
        return None;
    }
    let numeric: String = chars
        .take_while(|character| character.is_ascii_digit())
        .collect();
    if numeric.is_empty() {
        return None;
    }
    Some(format!("{prefix}{numeric}"))
}

fn top_level_colon_definition(line: &str) -> Option<(String, String)> {
    let trimmed = strip_comment(line);
    if !trimmed.starts_with("  ") || trimmed.starts_with("    ") {
        return None;
    }
    let (slug, body) = trimmed.trim().split_once(':')?;
    Some((slug.trim().to_string(), body.trim().to_string()))
}

fn strip_lino_comments(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        out.push_str(strip_comment(line));
        out.push('\n');
    }
    out
}

fn strip_comment(line: &str) -> &str {
    let mut previous_was_space = true;
    for (index, character) in line.char_indices() {
        if character == '#' && previous_was_space {
            return &line[..index];
        }
        previous_was_space = character.is_whitespace();
    }
    line
}
