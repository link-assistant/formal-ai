use std::fs;
use std::path::Path;

use formal_ai::json_lino::{json_to_lino, lino_to_json};
use lino_objects_codec::format::parse_indented;
use regex::Regex;
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
        if !path_has_component(path, "cache") {
            assert!(
                line_count <= MAX_LINO_LINES,
                "{} has {line_count} lines, exceeding {MAX_LINO_LINES}",
                path.display()
            );
        }
        assert!(
            !content.contains("(str ") && !content.contains("(object "),
            "{} should use indented human-readable Links Notation, not typed object encoding",
            path.display()
        );

        let parseable_content = strip_lino_comments(&content);
        for record in split_records(&parseable_content) {
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

#[test]
fn seed_lino_files_have_no_double_quoted_data() {
    for path in seed_lino_paths() {
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        assert!(
            !content.contains('"'),
            "{} contains a double quote; seed text must be codepoints or ids",
            path.display()
        );
    }
}

#[test]
fn wikidata_cache_uses_compact_native_lino() {
    let cache_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/cache/wikidata");
    let encoded_text = Regex::new(r"\bu-[0-9A-Fa-f]{2}(?:-[0-9A-Fa-f]{2})+\b")
        .expect("hex reference regex should compile");
    let forbidden = [
        "json-object",
        "json-array",
        "json-string",
        "json-number",
        "json-boolean",
        "json-null",
        "member ",
        "item ",
    ];

    for path in WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("lino"))
    {
        let content = fs::read_to_string(&path).expect("cache file should be UTF-8");
        for (index, line) in content.lines().enumerate() {
            let trimmed = strip_lino_comment(line).trim_start();
            for token in forbidden {
                assert!(
                    !trimmed.starts_with(token),
                    "{}:{} still contains noisy structural token `{token}`",
                    path.display(),
                    index + 1
                );
            }
        }
        assert!(
            !encoded_text.is_match(&content),
            "{} still stores raw source strings as encoded codepoint atoms",
            path.display()
        );
    }

    let reference_cache =
        fs::read_to_string(cache_dir.join("L41576.lino")).expect("L41576 cache file should exist");
    assert!(
        reference_cache.contains("\"reference\""),
        "raw cached strings should stay quoted and searchable"
    );
}

#[test]
fn meaning_seed_uses_id_fact_format() {
    let seed_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/seed");
    for path in WalkDir::new(&seed_dir)
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
    {
        let content = fs::read_to_string(&path).expect("meaning seed file should be UTF-8");
        for (index, line) in content.lines().enumerate() {
            let line_number = index + 1;
            let trimmed = strip_lino_comment(line).trim_start().to_string();
            assert!(
                !forbidden_meaning_key(&trimmed),
                "{}:{line_number} uses an old meaning/gloss/description/word key: {line}",
                path.display()
            );
            if let Some(id) = trimmed.strip_prefix("grounded-in ") {
                assert!(
                    id.starts_with('Q') || id.starts_with('P'),
                    "{}:{line_number} has non-entity grounding `{id}`",
                    path.display()
                );
            }
            if let Some(id) = trimmed.strip_prefix("surface ") {
                assert!(
                    id.starts_with('L')
                        || id.starts_with("seed-surface-")
                        || id.starts_with("unformalized-raw "),
                    "{}:{line_number} has bare-word surface `{id}`",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn wikidata_lino_cache_roundtrips_json_values() {
    let cache_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/cache/wikidata");
    let required = [
        "Q121769", "L5785", "L41576", "L166084", "L166085", "Q1084", "Q24905", "Q1860", "Q110786",
        "Q146786", "L3744", "L3743", "L3412", "L5848",
    ];
    for id in required {
        let path = cache_dir.join(format!("{id}.lino"));
        assert!(
            path.is_file(),
            "missing cached Wikidata file {}",
            path.display()
        );
    }

    for path in WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("lino"))
    {
        let content = fs::read_to_string(&path).expect("cache file should be UTF-8");
        let json = lino_to_json(&content)
            .unwrap_or_else(|error| panic!("{} failed to decode: {error}", path.display()));
        let roundtrip = lino_to_json(&json_to_lino(&json)).unwrap_or_else(|error| {
            panic!(
                "{} failed to decode after re-encode: {error}",
                path.display()
            )
        });
        assert_eq!(
            json,
            roundtrip,
            "{} lost data in json<->lino roundtrip",
            path.display()
        );
    }
}

fn split_records(content: &str) -> Vec<&str> {
    content
        .split("\n\n")
        .map(str::trim)
        .filter(|record| !record.is_empty())
        .collect()
}

fn seed_lino_paths() -> Vec<std::path::PathBuf> {
    let seed_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/seed");
    WalkDir::new(seed_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("lino"))
        .collect()
}

fn strip_lino_comments(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        out.push_str(strip_lino_comment(line));
        out.push('\n');
    }
    out
}

fn strip_lino_comment(line: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    let mut previous_was_space = true;
    let mut characters = line.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if let Some(quote_character) = quote {
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
                && characters.peek().is_some_and(|(_, next)| *next == '\'')
            {
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
            previous_was_space = false;
            continue;
        }
        if character == '#' && previous_was_space {
            return &line[..index];
        }
        previous_was_space = character.is_whitespace();
    }
    line
}

fn forbidden_meaning_key(trimmed: &str) -> bool {
    trimmed.starts_with("meaning ")
        || trimmed.starts_with("gloss ")
        || trimmed.starts_with("description ")
        || trimmed.starts_with("word ")
}

fn path_has_component(path: &Path, component: &str) -> bool {
    path.components().any(|path_component| {
        path_component
            .as_os_str()
            .to_str()
            .is_some_and(|part| part == component)
    })
}
