use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::seed::lexicon;
use links_notation::parse_lino as parse_canonical_lino;
use regex::Regex;
use walkdir::WalkDir;

const LINKS_ROOT_SEED: &str = "data/seed/meanings-links-root.lino";

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
fn seed_and_source_wikidata_ids_have_checked_in_cache_records() {
    let root = repo_root();
    let wikidata_cache_dir = root.join("data/cache/wikidata");
    let references = wikidata_references_from_paths(seed_and_rust_source_paths(&root));

    assert!(
        !references.is_empty(),
        "semantic grounding should find checked-in Q/L/P source ids"
    );
    let missing: Vec<String> = references
        .into_iter()
        .flat_map(|(id, locations)| {
            missing_wikidata_cache_files(&wikidata_cache_dir, &id, &locations)
        })
        .collect();

    assert!(
        missing.is_empty(),
        "seed/source Wikidata ids are missing checked-in cache files:\n{}",
        missing.join("\n")
    );
}

#[test]
fn wikidata_cache_records_cover_recursive_grounding_closure() {
    let root = repo_root();
    let wikidata_cache_dir = root.join("data/cache/wikidata");
    let initial_references = wikidata_references_from_paths(seed_and_rust_source_paths(&root));
    let mut queue: Vec<String> = initial_references.keys().cloned().collect();
    let mut checked = BTreeSet::new();
    let mut missing = Vec::new();

    for (id, locations) in initial_references {
        missing.extend(missing_wikidata_cache_files(
            &wikidata_cache_dir,
            &id,
            &locations,
        ));
    }

    while let Some(id) = queue.pop() {
        if !checked.insert(id.clone()) {
            continue;
        }

        let cache_path = wikidata_cache_path(&wikidata_cache_dir, &id, "lino");
        if !cache_path.is_file() {
            continue;
        }
        let content = fs::read_to_string(&cache_path).expect("wikidata cache should be UTF-8");
        for referenced_id in wikidata_ids_outside_quotes(&content) {
            let location = vec![cache_path.display().to_string()];
            let referenced_cache_path =
                wikidata_cache_path(&wikidata_cache_dir, &referenced_id, "lino");
            if !referenced_cache_path.is_file() {
                missing.extend(missing_wikidata_cache_files(
                    &wikidata_cache_dir,
                    &referenced_id,
                    &location,
                ));
                continue;
            }
            if !checked.contains(&referenced_id) {
                queue.push(referenced_id);
            }
        }
    }

    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "recursive Wikidata grounding closure is missing checked-in cache files:\n{}",
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
        parse_canonical_lino(content.trim())
            .unwrap_or_else(|error| panic!("{} is invalid LiNo: {error}", path.display()));
    }

    let mut missing = Vec::new();
    for path in meaning_seed_paths(&root.join("data/seed")) {
        let content = fs::read_to_string(&path).expect("meaning seed file should be readable");
        for (index, line) in content.lines().enumerate() {
            let trimmed = strip_comment(line).trim();
            if let Some(id) = trimmed.strip_prefix("wiktionary ") {
                let cache_path = wiktionary_cache_path(&cache_dir, id);
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

fn seed_and_rust_source_paths(root: &Path) -> Vec<PathBuf> {
    ["data/seed", "src"]
        .into_iter()
        .flat_map(|directory| {
            WalkDir::new(root.join(directory))
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file())
                .map(walkdir::DirEntry::into_path)
                .filter(|path| {
                    matches!(
                        path.extension().and_then(|extension| extension.to_str()),
                        Some("lino" | "rs")
                    )
                })
        })
        .collect()
}

fn wikidata_references_from_paths(paths: Vec<PathBuf>) -> BTreeMap<String, Vec<String>> {
    let source_id = Regex::new(r"\b[QLP][0-9]+\b").expect("source id regex should compile");
    let mut references: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for path in paths {
        let content = fs::read_to_string(&path).expect("source file should be readable");
        for (index, line) in content.lines().enumerate() {
            for id in source_id.find_iter(line).map(|matched| matched.as_str()) {
                references.entry(id.to_string()).or_default().push(format!(
                    "{}:{}",
                    path.display(),
                    index + 1
                ));
            }
        }
    }

    references
}

fn wikidata_ids_outside_quotes(content: &str) -> BTreeSet<String> {
    let source_id = Regex::new(r"\b[QLP][0-9]+\b").expect("source id regex should compile");
    let mut references = BTreeSet::new();

    for line in content.lines() {
        let stripped = strip_comment(line);
        let unquoted = remove_quoted_segments(stripped);
        for id in source_id
            .find_iter(&unquoted)
            .map(|matched| matched.as_str())
        {
            references.insert(id.to_string());
        }
    }

    references
}

fn remove_quoted_segments(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let mut quote = None;
    let mut escaped = false;
    let mut characters = line.chars().peekable();

    while let Some(character) = characters.next() {
        if let Some(quote_character) = quote {
            output.push(' ');
            if escaped {
                escaped = false;
                continue;
            }
            if quote_character == '"' && character == '\\' {
                escaped = true;
                continue;
            }
            if quote_character == '\''
                && character == '\''
                && characters.peek().is_some_and(|next| *next == '\'')
            {
                output.push(' ');
                characters.next();
                continue;
            }
            if character == quote_character {
                quote = None;
            }
            continue;
        }

        if matches!(character, '"' | '\'') {
            quote = Some(character);
            output.push(' ');
        } else {
            output.push(character);
        }
    }

    output
}

fn missing_wikidata_cache_files(root: &Path, id: &str, locations: &[String]) -> Vec<String> {
    let mut missing = Vec::new();
    for extension in ["lino", "json"] {
        let cache_path = wikidata_cache_path(root, id, extension);
        if !cache_path.is_file() {
            missing.push(format!("{id}.{extension} -> {}", locations.join(", ")));
        }
    }
    missing
}

fn top_level_colon_definition(line: &str) -> Option<(String, String)> {
    let trimmed = strip_comment(line);
    if !trimmed.starts_with("  ") || trimmed.starts_with("    ") {
        return None;
    }
    let (slug, body) = trimmed.trim().split_once(':')?;
    Some((slug.trim().to_string(), body.trim().to_string()))
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

fn wikidata_cache_path(root: &Path, id: &str, extension: &str) -> PathBuf {
    let kind = match id.chars().next() {
        Some('L') => "lexeme",
        Some('P') => "property",
        Some('Q') => "entity",
        _ => panic!("unexpected Wikidata id `{id}`"),
    };
    root.join(kind).join(format!("{id}.{extension}"))
}

fn wiktionary_cache_path(root: &Path, id: &str) -> PathBuf {
    if let Some(rest) = id.strip_prefix("WT-") {
        if let Some((language, page)) = rest.split_once('-') {
            return root.join(language).join(format!("{page}.lino"));
        }
    }
    root.join(format!("{id}.lino"))
}
