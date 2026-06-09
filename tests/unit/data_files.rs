use std::fs;
use std::path::Path;

use formal_ai::json_lino::{json_cache_file, json_cache_projection, lino_to_json};
use links_notation::parse_lino as parse_canonical_lino;
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

        parse_canonical_lino(content.trim()).unwrap_or_else(|error| {
            panic!(
                "{} contains invalid canonical Links Notation: {error}",
                path.display()
            );
        });
    }

    assert!(
        checked_files >= 3,
        "expected checked-in Links Notation seed data files"
    );
}

#[test]
fn lino_data_files_avoid_jsonish_and_unresolved_tokens() {
    let pipe_id_blob =
        Regex::new(r"\b[QLP][0-9]+(?:\|[QLP][0-9]+)+\b").expect("pipe id regex should compile");
    let jsonish_colon_value =
        Regex::new(r"^\s*[A-Za-z0-9_.-]+:\s+\S").expect("jsonish colon regex should compile");
    let colon_comment =
        Regex::new(r"^\s*[A-Za-z0-9_.-]+:\s+#").expect("colon comment regex should compile");

    for path in data_lino_paths() {
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        for (index, line) in content.lines().enumerate() {
            // Strip the trailing comment first, then any quoted scalar spans, so
            // structural checks only inspect the Links Notation skeleton and not
            // human-readable values (which may legitimately contain `[]`, ids,
            // etc. when they quote source code or prose).
            let stripped = strip_quoted_spans(strip_lino_comment(line));
            assert!(
                !stripped.contains("unformalized-raw"),
                "{}:{} still contains unresolved raw surface ids: {line}",
                path.display(),
                index + 1
            );
            assert!(
                !stripped.contains("[]"),
                "{}:{} still contains invalid empty array syntax: {line}",
                path.display(),
                index + 1
            );
            let stripped = stripped.as_str();
            assert!(
                !pipe_id_blob.is_match(stripped),
                "{}:{} still contains a pipe-separated id blob: {line}",
                path.display(),
                index + 1
            );
            if path_has_component(&path, "cache") {
                assert!(
                    !jsonish_colon_value.is_match(stripped),
                    "{}:{} still uses JSON-style `key: value`: {line}",
                    path.display(),
                    index + 1
                );
            }
            assert!(
                !colon_comment.is_match(line),
                "{}:{} still uses an empty colon definition with comment-only data: {line}",
                path.display(),
                index + 1
            );
        }
    }
}

#[test]
fn seed_lino_files_have_no_codepoint_byte_dumps() {
    // Issue #398 review (comment 4660584608): seed data must be human readable.
    // Codepoint byte-dumps such as `answer codepoints 72 105 44 ...` are banned;
    // text must be grounded references or, as a last resort, quoted strings.
    let bare_integer_run =
        Regex::new(r"(?:^|\s)\d+(?:\s+\d+){3,}(?:\s|$)").expect("integer run regex should compile");
    for path in seed_lino_paths() {
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        for (index, line) in content.lines().enumerate() {
            let line_number = index + 1;
            let skeleton = strip_quoted_spans(strip_lino_comment(line));
            for token in skeleton.split_whitespace() {
                assert!(
                    token != "codepoints" && token != "unformalized-raw",
                    "{}:{line_number} still encodes text as a codepoint byte-dump: {line}",
                    path.display()
                );
            }
            assert!(
                !bare_integer_run.is_match(&skeleton),
                "{}:{line_number} contains a bare integer run used as text; \
                 use a grounded reference or a quoted string: {line}",
                path.display()
            );
        }
    }
}

#[test]
fn seed_lino_files_have_no_synthetic_surface_ids() {
    // Issue #398 review (comment 4660584608): a surface is the text (and facets)
    // recorded under a language; opaque hashed `seed-surface-<hash>` ids carried
    // no meaning and existed only to give the node an id. They are banned so the
    // seed stays legible. `scripts/clean-seed-readability.rs` performs the
    // migration that removes them.
    for path in seed_lino_paths() {
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        for (index, line) in content.lines().enumerate() {
            let skeleton = strip_quoted_spans(strip_lino_comment(line));
            assert!(
                !skeleton.contains("seed-surface-"),
                "{}:{} reintroduces a synthetic surface id; a surface is the text \
                 under a language, not a minted `seed-surface-<hash>` id: {line}",
                path.display(),
                index + 1,
            );
        }
    }
}

#[test]
fn seed_lino_files_have_no_keyword_restating_comments() {
    // Issue #398 review (comment 4660584608): a comment must add information (the
    // human meaning of an opaque id like `Q146786 # plural`), never restate the
    // keyword already on the line. These trailing comments only repeat the
    // keyword and are banned. `scripts/clean-seed-readability.rs` strips them.
    const NOISE_COMMENTS: &[&str] = &[
        "language",
        "definition-link",
        "semantic-role",
        "facet",
        "seed lexical surface",
        "source-id",
        "action",
    ];
    for path in seed_lino_paths() {
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        for (index, line) in content.lines().enumerate() {
            let Some(comment) = lino_comment_body(line) else {
                continue;
            };
            assert!(
                !NOISE_COMMENTS.contains(&comment.trim()),
                "{}:{} has a keyword-restating noise comment `# {}`; \
                 drop it or replace it with the human meaning of an opaque id: {line}",
                path.display(),
                index + 1,
                comment.trim(),
            );
        }
    }
}

#[test]
fn seed_lino_values_never_pipe_pack_multi_values() {
    // Issue #398 review (comment 4660584608), defect #4: a multi-value field must
    // be a sequence of separate references — `aliases ("a" "b" "c")` — not a
    // single string with an in-band `|` separator. This guard is exhaustive: it
    // fails on *any* seed value containing `|`, whether bare (`tasks a|b`) or
    // quoted (`aliases "a|b"`), across every field. `scripts/migrate-pipe-lists.rs`
    // performs the migration into `(...)` reference lists.
    //
    // `code` is the sole exemption: it carries verbatim source listings where `|`
    // is legitimate syntax (Rust closures `|x|`, `||` short-circuits, shell pipes).
    const PROSE_FIELDS: &[&str] = &["code"];
    for path in seed_lino_paths() {
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        for (index, line) in content.lines().enumerate() {
            let code = strip_lino_comment(line).trim();
            let Some((keyword, value)) = code.split_once(char::is_whitespace) else {
                continue;
            };
            if PROSE_FIELDS.contains(&keyword) {
                continue;
            }
            assert!(
                !value.contains('|'),
                "{}:{} packs the `{keyword}` multi-value with `|`; use a \
                 reference list `{keyword} (\"a\" \"b\")` instead: {line}",
                path.display(),
                index + 1,
            );
        }
    }
}

#[test]
fn seed_lino_files_have_no_empty_redefinition_fields() {
    // Issue #398 review (comment 4663407299), defect #1: a semantic facet (or any
    // field) must use the native `subject predicate` form — two references on one
    // line, e.g. `notation word_surface`, `denotation lexical_sense`. The banned
    // shape is an *empty* colon redefinition: `word_surface:` with no body, which
    // merely restates a concept already defined elsewhere.
    //
    // The earlier guard (`lino_data_files_avoid_jsonish_and_unresolved_tokens`)
    // only caught `key: # comment`; it missed a bare `word_surface:`. This check
    // is the broad version: it walks the *entire* seed tree and fails on any
    // colon line whose body is empty — i.e. it has no deeper-indented child. A
    // real definition header (`result:` followed by indented `defined-by …`) has
    // a body and passes; only the valueless redefinition fails.
    // `scripts/migrate-empty-facet-fields.rs` performs the migration.
    let empty_colon =
        Regex::new(r"^[A-Za-z0-9_.-]+:$").expect("empty colon regex should compile");
    for path in seed_lino_paths() {
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        let lines: Vec<&str> = content.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let skeleton = strip_quoted_spans(strip_lino_comment(line));
            let body = skeleton.trim();
            if !empty_colon.is_match(body) {
                continue;
            }
            let indent = line.chars().take_while(|character| *character == ' ').count();
            // A genuine block header is followed by a deeper-indented child; an
            // empty redefinition is not.
            let has_child = lines[index + 1..]
                .iter()
                .find(|next| !strip_lino_comment(next).trim().is_empty())
                .is_some_and(|next| {
                    next.chars().take_while(|character| *character == ' ').count() > indent
                });
            assert!(
                has_child,
                "{}:{} is an empty colon redefinition `{body}`; use the native \
                 `subject predicate` form (two references on one line) instead: {line}",
                path.display(),
                index + 1,
            );
        }
    }
}

#[test]
fn wikidata_cache_uses_compact_native_lino() {
    let cache_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/cache/wikidata");
    let encoded_text = Regex::new(r"\bu-[0-9A-Fa-f]{2}(?:-[0-9A-Fa-f]{2})+\b")
        .expect("hex reference regex should compile");
    let generated_array_id = Regex::new(r"\bat-[0-9]{4}\b").expect("array id regex should compile");
    let scalar_string_tag =
        Regex::new(r#"\bstring\s+["']"#).expect("string tag regex should compile");
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
            assert!(
                !generated_array_id.is_match(trimmed),
                "{}:{} still contains generated array id syntax: {line}",
                path.display(),
                index + 1
            );
            assert!(
                !scalar_string_tag.is_match(trimmed),
                "{}:{} still contains a JSON scalar `string` tag: {line}",
                path.display(),
                index + 1
            );
        }
        assert!(
            !encoded_text.is_match(&content),
            "{} still stores raw source strings as encoded codepoint atoms",
            path.display()
        );
    }

    let reference_cache = fs::read_to_string(wikidata_cache_path(&cache_dir, "Q181593", "lino"))
        .expect("Q181593 cache file should exist");
    assert!(
        reference_cache.contains("\"sorting algorithm\""),
        "raw cached strings with spaces should stay quoted and searchable"
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
                    id.starts_with('L') || id.starts_with("seed-surface-"),
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
        let path = wikidata_cache_path(&cache_dir, id, "lino");
        assert!(
            path.is_file(),
            "missing cached Wikidata file {}",
            path.display()
        );
        let json_path = wikidata_cache_path(&cache_dir, id, "json");
        assert!(
            json_path.is_file(),
            "missing raw Wikidata JSON file {}",
            json_path.display()
        );
    }

    for path in WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("lino"))
    {
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("cache file should have an id stem");
        let json_path = path.with_extension("json");
        assert!(
            json_path.is_file(),
            "{} must have a raw JSON snapshot beside it",
            path.display()
        );
        let raw_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&json_path).expect("raw json cache file should be UTF-8"),
        )
        .unwrap_or_else(|error| panic!("{} contains invalid JSON: {error}", json_path.display()));

        let content = fs::read_to_string(&path).expect("cache file should be UTF-8");
        assert_eq!(
            content,
            json_cache_file(id, &raw_json),
            "{} is not the canonical LiNo projection of {}",
            path.display(),
            json_path.display()
        );
        let json = lino_to_json(&content)
            .unwrap_or_else(|error| panic!("{} failed to decode: {error}", path.display()));
        assert_eq!(
            json_cache_projection(id, &raw_json),
            json,
            "{} decoded projection does not match raw JSON source fields",
            path.display()
        );
    }
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

fn data_lino_paths() -> Vec<std::path::PathBuf> {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    WalkDir::new(data_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("lino"))
        .collect()
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

/// The body of a line's trailing `#` comment (everything after the `#`), or
/// `None` when the line has no comment. Mirrors [`strip_lino_comment`]'s
/// quote-aware boundary detection.
fn lino_comment_body(line: &str) -> Option<&str> {
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
            return Some(&line[index + 1..]);
        }
        previous_was_space = character.is_whitespace();
    }
    None
}

/// Remove quoted scalar spans (`"..."`, `'...'`, `` `...` ``) from a line so
/// structural assertions only inspect the Links Notation skeleton. Backslash
/// escapes are honoured inside double-quote and backtick spans (matching the
/// seed parser), so an escaped delimiter does not prematurely close the span.
fn strip_quoted_spans(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(character) = chars.next() {
        if matches!(character, '"' | '\'' | '`') {
            let delimiter = character;
            let escapes = delimiter != '\'';
            while let Some(inner) = chars.next() {
                if escapes && inner == '\\' {
                    chars.next();
                    continue;
                }
                if inner == delimiter {
                    break;
                }
            }
            out.push(' ');
        } else {
            out.push(character);
        }
    }
    out
}

fn wikidata_cache_path(root: &Path, id: &str, extension: &str) -> std::path::PathBuf {
    let kind = match id.chars().next() {
        Some('L') => "lexeme",
        Some('P') => "property",
        Some('Q') => "entity",
        _ => panic!("unexpected Wikidata id `{id}`"),
    };
    root.join(kind).join(format!("{id}.{extension}"))
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
