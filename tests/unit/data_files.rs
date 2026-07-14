use std::fs;
use std::path::Path;

use formal_ai::json_lino::{json_cache_file, lino_to_json};
use links_notation::parse_lino as parse_canonical_lino;
use regex::Regex;
use serde_json::{Map, Value};
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
        // The typed-object-encoding guard targets the Links Notation skeleton,
        // not quoted prose: a cached definition may legitimately read
        // `"(object pronoun) …"`. Strip quoted scalar spans first so the check
        // only inspects structure, matching the sibling jsonish guard below.
        let skeleton: String = content
            .lines()
            .map(|line| strip_quoted_spans(line) + "\n")
            .collect();
        assert!(
            !skeleton.contains("(str ") && !skeleton.contains("(object "),
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
    // Issue #398 review (comment 4664274427, defect 4 / CI check 4): every
    // `^\s*[\w-]+:\s*$` line is an empty colon redefinition — a YAML-style
    // header (`monday:`) whose trailing colon restates the slug as a valueless
    // key. Links Notation has no such form: a node is just an indented name,
    // exactly like its own `surface` / `lexeme en` children, which already carry
    // no colon. The native definition header drops the colon (`monday`), with
    // the definition living in the deeper-indented children.
    //
    // This is the reviewer's exact regex applied tree-wide: it fails on *any*
    // bare-identifier line that ends in a colon, whether or not it has a body.
    // Stripping the colon is parse-equivalent — `parse_colon_definition`
    // (`src/seed/parser.rs`) turns `monday:` into `(name = "monday", id = "")`,
    // identical to the bare node `monday`. `scripts/migrate-empty-redefinition-fields.rs`
    // performs the whole-tree migration and regenerates the browser worker embed.
    let empty_colon = Regex::new(r"^[A-Za-z0-9_.-]+:$").expect("empty colon regex should compile");
    for path in seed_lino_paths() {
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        for (index, line) in content.lines().enumerate() {
            let body = strip_quoted_spans(strip_lino_comment(line));
            let body = body.trim();
            assert!(
                !empty_colon.is_match(body),
                "{}:{} is an empty colon redefinition `{body}`; drop the trailing \
                 colon so it is a native Links Notation node (`{}`): {line}",
                path.display(),
                index + 1,
                body.trim_end_matches(':'),
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

/// Independent empty-collection normalization (issue #398 review, defect #5):
/// drop JSON `null`, empty arrays and empty objects. Defined here in the test
/// — *not* reused from the codec under test — so the round-trip assertion below
/// can never be circular. An empty collection is an absent default that the
/// Links Notation cache never stores, so the only difference allowed between
/// the raw JSON and the JSON rebuilt from Links Notation is the absence of
/// these empties.
fn strip_json_empties(value: &Value) -> Option<Value> {
    match value {
        Value::Null => None,
        Value::Array(items) => {
            let kept: Vec<Value> = items.iter().filter_map(strip_json_empties).collect();
            (!kept.is_empty()).then_some(Value::Array(kept))
        }
        Value::Object(object) => {
            let mut kept = Map::new();
            for (key, value) in object {
                if let Some(value) = strip_json_empties(value) {
                    kept.insert(key.clone(), value);
                }
            }
            (!kept.is_empty()).then_some(Value::Object(kept))
        }
        scalar => Some(scalar.clone()),
    }
}

#[test]
fn wikidata_lino_cache_rebuilds_full_json_losslessly() {
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

    let mut checked = 0;
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
        let raw_json: Value = serde_json::from_str(
            &fs::read_to_string(&json_path).expect("raw json cache file should be UTF-8"),
        )
        .unwrap_or_else(|error| panic!("{} contains invalid JSON: {error}", json_path.display()));

        let content = fs::read_to_string(&path).expect("cache file should be UTF-8");

        // The checked-in LiNo is the canonical encoding of the raw JSON, so the
        // file stays byte-stable under regeneration.
        assert_eq!(
            content,
            json_cache_file(id, &raw_json),
            "{} is not the canonical LiNo encoding of {}",
            path.display(),
            json_path.display()
        );

        // The headline guarantee (defect #1): rebuild the *entire* original
        // JSON — every form, sense, claim and key — from the LiNo alone, and
        // assert it equals the raw `.json` (modulo the empty defaults that are
        // never stored). The expected value is normalized by this test's own
        // `strip_json_empties`, never by the codec, so the assertion proves a
        // real round-trip rather than comparing the codec against itself.
        let rebuilt = lino_to_json(&content)
            .unwrap_or_else(|error| panic!("{} failed to decode: {error}", path.display()));
        let expected = strip_json_empties(&raw_json).unwrap_or_else(|| Value::Object(Map::new()));
        assert_eq!(
            rebuilt,
            expected,
            "{} does not rebuild the full raw JSON {}",
            path.display(),
            json_path.display()
        );
        checked += 1;
    }

    assert!(
        checked >= required.len(),
        "expected to round-trip every cached Wikidata file, only checked {checked}"
    );

    // Guard the specific snapshot the review called out: L3412's raw JSON keeps
    // forms, senses and claims, and all three must survive the round-trip.
    let l3412_lino = fs::read_to_string(wikidata_cache_path(&cache_dir, "L3412", "lino"))
        .expect("L3412 cache should be UTF-8");
    let l3412 = lino_to_json(&l3412_lino).expect("L3412 should decode");
    let entity = l3412
        .get("entities")
        .and_then(|entities| entities.get("L3412"))
        .expect("L3412 entity should be present");
    for key in ["forms", "senses", "claims"] {
        let value = entity
            .get(key)
            .unwrap_or_else(|| panic!("L3412 lost `{key}`"));
        assert!(
            !value.as_array().is_some_and(Vec::is_empty)
                && !value.as_object().is_some_and(Map::is_empty),
            "L3412 `{key}` round-tripped empty"
        );
    }
}

#[test]
fn wiktionary_cache_is_pretty_printed_and_rebuilds_full_json() {
    let cache_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/cache/wiktionary");
    let mut checked = 0;
    for path in WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("lino"))
    {
        let json_path = path.with_extension("json");
        assert!(
            json_path.is_file(),
            "{} must have a raw JSON snapshot beside it",
            path.display()
        );
        let raw_text = fs::read_to_string(&json_path).expect("wiktionary json should be UTF-8");
        let raw_json: Value = serde_json::from_str(&raw_text)
            .unwrap_or_else(|error| panic!("{} is invalid JSON: {error}", json_path.display()));

        // Defect #2: the snapshot must be pretty-printed (multi-line), one entry
        // per grounded surface — never the original single-line blob.
        assert!(
            raw_text.lines().count() > 1,
            "{} must be pretty-printed multi-line, not a single-line blob",
            json_path.display()
        );

        // The LiNo rebuilds the full Wiktionary JSON (every meaning, definition,
        // phonetic and license) modulo the empty defaults that are never stored.
        let content = fs::read_to_string(&path).expect("wiktionary cache should be UTF-8");
        let rebuilt = lino_to_json(&content)
            .unwrap_or_else(|error| panic!("{} failed to decode: {error}", path.display()));
        let expected = strip_json_empties(&raw_json).unwrap_or_else(|| Value::Object(Map::new()));
        assert_eq!(
            rebuilt,
            expected,
            "{} does not rebuild the full raw JSON {}",
            path.display(),
            json_path.display()
        );
        checked += 1;
    }

    assert!(
        checked > 0,
        "expected at least one Wiktionary cache record to round-trip"
    );
}

/// The number of cached Wiktionary entries must never regress below this floor.
/// Issue #398 (deep review of `92a29b0`, open item #1) flags that the Wiktionary
/// cache was a single placeholder entry while the seed grounds 140+ meanings:
/// "Wiktionary should be used heavily … per grounded surface, in every language
/// we claim." This ratchet records progress toward that goal so each batch can
/// only raise it. Bump it (and never lower it) whenever
/// `scripts/ground-wiktionary.py` caches more grounded surfaces. The matching
/// lossless `.lino`/`.json` snapshots keep every entry checked in and verified
/// by `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json`.
const WIKTIONARY_ENTRY_FLOOR: usize = 156;

#[test]
fn wiktionary_cache_breadth_does_not_regress() {
    let cache_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/cache/wiktionary");
    let entries = WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("lino"))
        .count();

    assert!(
        entries >= WIKTIONARY_ENTRY_FLOOR,
        "Wiktionary cache breadth regressed: {entries} entries cached, floor is \
         {WIKTIONARY_ENTRY_FLOOR}. Wiktionary grounding is append-only — re-run \
         scripts/ground-wiktionary.py rather than removing cached entries."
    );
}

#[test]
fn meaning_definitions_are_unique() {
    // Issue #398 review (comment 4664274427, CI check 7): no two meanings may
    // share an identical definition. If two definitions are byte-for-byte the
    // same they are the same meaning (and must be merged); if they are meant to
    // be distinct they must be differentiated (a different genus, role, facet,
    // grounding, or surface set). The signature is the *full* definition body —
    // including the per-language `lexeme` surfaces — so genuinely distinct
    // siblings that share a genus (`monday` and `tuesday`, both
    // `defined-by calendar_day`) stay distinct via their surfaces.
    let mut by_signature: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (path, slug, body) in meaning_definitions() {
        by_signature
            .entry(body)
            .or_default()
            .push(format!("{}:{slug}", path.display()));
    }
    let mut clashes: Vec<(String, Vec<String>)> = by_signature
        .into_iter()
        .filter(|(_, owners)| owners.len() > 1)
        .collect();
    clashes.sort();
    assert!(
        clashes.is_empty(),
        "meanings with byte-identical definitions must be merged or differentiated: {:?}",
        clashes
            .iter()
            .map(|(_, owners)| owners.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn meaning_slugs_are_globally_unique() {
    // A meaning slug names exactly one meaning. The same slug defined twice (in
    // one file or across files) is a cross-reference hazard: `defined-by foo`
    // becomes ambiguous. This guards the dataset's referential integrity.
    let mut owners: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (path, slug, _body) in meaning_definitions() {
        owners
            .entry(slug)
            .or_default()
            .push(path.display().to_string());
    }
    let mut duplicates: Vec<(String, Vec<String>)> = owners
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect();
    duplicates.sort();
    assert!(
        duplicates.is_empty(),
        "meaning slugs must be globally unique; duplicated: {duplicates:?}"
    );
}

#[test]
fn cache_source_json_is_pretty_printed() {
    // Issue #398 review (comment 4664274427, CI check 2): every cached source
    // snapshot is pretty-printed (multi-line) so diffs are reviewable. A
    // single-line blob is the placeholder shape the review rejects. This guards
    // the whole `data/cache` tree, not just Wiktionary.
    let cache_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/cache");
    let mut checked = 0;
    for path in WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
    {
        let text = fs::read_to_string(&path).expect("cache json should be UTF-8");
        assert!(
            text.lines().count() > 1,
            "{} must be pretty-printed multi-line, not a single-line blob",
            path.display()
        );
        checked += 1;
    }
    assert!(checked > 0, "expected checked-in cache JSON snapshots");
}

/// The number of meanings carrying a `grounded-in <id>` anchor must never
/// regress below this floor. Issue #398 (review comment 4664274427, CI check 3)
/// wants *every* meaning grounded to a real Wikidata/Wiktionary id; this ratchet
/// records progress toward that goal so each batch can only raise it. Bump it
/// (and never lower it) whenever `scripts/ground-meanings.rs` or the
/// `formal-ai import lexemes` bulk importer (issue #660) grounds more meanings.
/// The matching `data/cache` snapshots keep the closure checked in.
const GROUNDED_MEANING_FLOOR: usize = 349;

#[test]
fn grounded_meaning_coverage_does_not_regress() {
    let definitions = meaning_definitions();
    let total = definitions.len();
    let grounded = definitions
        .iter()
        .filter(|(_, _, body)| {
            body.lines()
                .any(|line| line.trim().starts_with("grounded-in "))
        })
        .count();

    assert!(
        grounded >= GROUNDED_MEANING_FLOOR,
        "grounded-meaning coverage regressed: {grounded}/{total} grounded, \
         floor is {GROUNDED_MEANING_FLOOR}. Grounding is append-only — re-run \
         scripts/ground-meanings.rs rather than removing grounded-in anchors."
    );
}

/// The number of meanings carrying full lexical detail — a `source-lexeme`
/// reference with its part of speech (`lexical-category`) and at least one
/// `form` + `feature` sourced from the lexeme — must never regress. Issue #398
/// (deep review of `f1f78dc`, defect 6 / CI check 6) wants *every* grounded word
/// to expose its parts of speech and all forms from Wikidata/Wiktionary rather
/// than hand-authored surfaces; this ratchet records progress toward that goal.
/// Bump it (and never lower it) whenever `scripts/ground-lexemes.py` enriches
/// more words.
const LEXICAL_COMPLETENESS_FLOOR: usize = 6;

#[test]
fn lexical_completeness_does_not_regress() {
    let cache_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/cache/wikidata");
    let mut complete = 0;
    for (path, slug, body) in meaning_definitions() {
        let has_source_lexeme = body
            .lines()
            .any(|line| line.trim().starts_with("source-lexeme "));
        let has_category = body
            .lines()
            .any(|line| line.trim().starts_with("lexical-category "));
        let has_form = body.lines().any(|line| line.trim().starts_with("form "));
        let has_feature = body.lines().any(|line| line.trim().starts_with("feature "));
        if !(has_source_lexeme && has_category && has_form && has_feature) {
            continue;
        }
        // Every referenced source lexeme must resolve to a checked-in cache file
        // so the part of speech and forms are actually sourced, not invented.
        for line in body.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("source-lexeme ") {
                let lid = rest.split_whitespace().next().unwrap_or_default();
                let lexeme_path = wikidata_cache_path(&cache_dir, lid, "lino");
                assert!(
                    lexeme_path.is_file(),
                    "{slug} in {} references uncached lexeme {lid}",
                    path.display()
                );
            }
        }
        complete += 1;
    }

    assert!(
        complete >= LEXICAL_COMPLETENESS_FLOOR,
        "lexical-completeness coverage regressed: {complete} meanings carry sourced \
         part-of-speech + forms, floor is {LEXICAL_COMPLETENESS_FLOOR}. Lexical \
         grounding is append-only — re-run scripts/ground-lexemes.py rather than \
         removing source-lexeme detail."
    );
}

/// Every top-level meaning across the `data/seed/meanings*.lino` tree as
/// `(file, slug, full-definition-body)`. The body is the verbatim block of
/// deeper-indented child lines (comments stripped), used for uniqueness checks.
fn meaning_definitions() -> Vec<(std::path::PathBuf, String, String)> {
    let mut out = Vec::new();
    for path in seed_lino_paths() {
        let is_meaning_file = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("meanings"));
        if !is_meaning_file {
            continue;
        }
        let content = fs::read_to_string(&path).expect("lino file should be UTF-8 text");
        let lines: Vec<&str> = content.lines().collect();
        let mut index = 0;
        while index < lines.len() {
            let line = lines[index];
            let indent = line.chars().take_while(|c| *c == ' ').count();
            let trimmed = strip_lino_comment(line).trim();
            // A top-level meaning header sits at indent 2, is a bare slug (no
            // value on the line), and is not the `meanings` container itself.
            let is_header = indent == 2
                && !trimmed.is_empty()
                && trimmed != "meanings"
                && !trimmed.contains(char::is_whitespace);
            if !is_header {
                index += 1;
                continue;
            }
            let mut body = Vec::new();
            let mut cursor = index + 1;
            while cursor < lines.len() {
                let next = lines[cursor];
                if next.trim().is_empty() {
                    cursor += 1;
                    continue;
                }
                let next_indent = next.chars().take_while(|c| *c == ' ').count();
                if next_indent <= indent {
                    break;
                }
                body.push(strip_lino_comment(next).trim_end().to_string());
                cursor += 1;
            }
            out.push((path.clone(), trimmed.to_string(), body.join("\n")));
            index = cursor;
        }
    }
    out
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
