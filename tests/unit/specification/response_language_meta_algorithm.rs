//! Response-language follow-up meta-algorithm grounding (issue #556).
//!
//! Issue #556 asked us to "generalize to all similar requests (the whole class
//! of similar questions) in all languages" using "our general and universal meta
//! algorithm and actual recursive reasoning steps, expressed in meta language",
//! with "all meanings … grounded in external data sources" and "every finest
//! detail … tested". `data/meta/response-language-followup-recipe.lino` is that
//! recipe: it names every ordered step, seed role, external grounding, handler
//! function, forced-language seam, JS parity target, and pinning test that make
//! up the response-language follow-up.
//!
//! These tests keep the recipe *grounded*: they load it and assert the live
//! source still matches. If the recipe and the code drift apart, CI fails — so
//! the recipe is always an accurate, executable description of how the code was
//! produced, not stale documentation. The parser mirrors
//! `tests/unit/specification/meta_algorithm.rs`.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/response-language-followup-recipe.lino";

struct Record {
    kind: String,
    fields: Vec<(String, String)>,
}

impl Record {
    fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn require(&self, name: &str) -> &str {
        self.field(name)
            .unwrap_or_else(|| panic!("{} record missing field `{name}`", self.kind))
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
}

fn records() -> Vec<Record> {
    let text = read(RECIPE);
    let mut records = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(parse_record(&current));
            current.clear();
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(parse_record(&current));
    }
    records
}

fn parse_record(lines: &[&str]) -> Record {
    let mut kind = String::new();
    let mut fields = Vec::new();
    for line in lines.iter().skip(1) {
        let trimmed = line.trim();
        if let Some((name, raw)) = trimmed.split_once(' ') {
            let value = unquote(raw.trim());
            if name == "record_type" {
                kind = value;
            } else {
                fields.push((name.to_owned(), value));
            }
        }
    }
    Record { kind, fields }
}

fn unquote(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
}

fn of_kind<'a>(records: &'a [Record], kind: &str) -> Vec<&'a Record> {
    records
        .iter()
        .filter(|record| record.kind == kind)
        .collect()
}

/// Concatenate every JS worker module so parity assertions see the whole
/// browser surface regardless of which module a function lives in.
fn read_worker_source() -> String {
    let worker_dir = repo_root().join("src/web/worker");
    let mut modules: Vec<PathBuf> = fs::read_dir(&worker_dir)
        .unwrap_or_else(|error| panic!("src/web/worker should be readable: {error}"))
        .map(|entry| {
            entry
                .expect("worker module entry should be readable")
                .path()
        })
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("js"))
        .collect();
    modules.sort();

    let mut source = String::new();
    for module in modules {
        let module_source = fs::read_to_string(&module)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", module.display()));
        source.push('\n');
        source.push_str(&module_source);
    }
    source
}

#[test]
fn meta_recipe_steps_are_complete_and_ordered() {
    let records = records();
    let recipe = of_kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1, "exactly one meta_recipe header expected");
    assert_eq!(recipe[0].require("topic"), "response_language_followup");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how to generalise to other re-answer constraints"
    );
    assert!(
        !recipe[0].require("summary").is_empty(),
        "recipe must summarise the response-language follow-up"
    );

    let mut orders: Vec<usize> = of_kind(&records, "meta_step")
        .iter()
        .map(|step| {
            step.require("order")
                .parse()
                .expect("step order must be an integer")
        })
        .collect();
    orders.sort_unstable();
    assert_eq!(
        orders,
        (1..=8).collect::<Vec<_>>(),
        "the meta-algorithm must list eight contiguously ordered steps"
    );

    // Every step must point at a real file it was produced from.
    for step in of_kind(&records, "meta_step") {
        let seed_file = step.require("seed_file");
        assert!(
            repo_root().join(seed_file).exists(),
            "meta_step {} seed_file {seed_file} should exist",
            step.require("id"),
        );
    }
}

#[test]
fn meta_recipe_roles_match_live_constants_and_seed() {
    let records = records();
    let language_roles = read("src/seed/roles/language.rs");
    let roles = of_kind(&records, "meta_role");
    assert_eq!(
        roles.len(),
        2,
        "expected the two trigger roles (response-language + comprehension-failure)"
    );

    for role in roles {
        let value = role.require("role");
        let constant = role.require("const");
        let declaration = format!("pub const {constant}: &str = \"{value}\";");
        assert!(
            language_roles.contains(&declaration),
            "{} should declare {declaration}",
            role.require("const_file"),
        );
        let seed = read(role.require("seed_file"));
        assert!(
            seed.contains(&format!("role {value}")),
            "{} should declare `role {value}`",
            role.require("seed_file"),
        );
    }
}

#[test]
fn meta_recipe_groundings_resolve_to_cached_external_entities() {
    let records = records();
    let groundings = of_kind(&records, "meta_grounding");
    assert!(
        groundings.len() >= 5,
        "expected at least the comprehension-failure grounding plus the four seeded languages"
    );

    for grounding in groundings {
        let wikidata = grounding.require("wikidata");
        let cache_file = grounding.require("cache_file");
        assert!(
            cache_file.ends_with(&format!("{wikidata}.lino")),
            "grounding cache_file {cache_file} should name entity {wikidata}"
        );
        assert!(
            repo_root().join(cache_file).exists(),
            "grounding {} should have a cached external entity at {cache_file}",
            grounding.require("meaning"),
        );
        // The meaning must actually be grounded in that Q-id in the seed lexicon.
        let seed = read(grounding.require("seed_file"));
        assert!(
            seed.contains(wikidata),
            "{} should ground a meaning in {wikidata}",
            grounding.require("seed_file"),
        );
    }
}

#[test]
fn meta_recipe_functions_exist_in_named_source() {
    let records = records();
    let functions = of_kind(&records, "meta_function");
    assert!(
        functions.len() >= 8,
        "expected the full recogniser/seam/handler chain to be documented"
    );
    for function in functions {
        let name = function.require("function");
        let source = read(function.require("source_file"));
        assert!(
            source.contains(&format!("fn {name}")),
            "{} should define fn {name}",
            function.require("source_file"),
        );
    }
}

#[test]
fn meta_recipe_seam_tokens_exist_in_both_runtimes() {
    let records = records();
    let seams = of_kind(&records, "meta_seam");
    assert!(!seams.is_empty(), "the forced-language seam must be recorded");
    let worker = read_worker_source();
    for seam in seams {
        let rust = read(seam.require("rust_file"));
        assert!(
            rust.contains(seam.require("rust_token")),
            "{} should hold the {} seam token",
            seam.require("rust_file"),
            seam.require("rust_token"),
        );
        assert!(
            worker.contains(seam.require("js_token")),
            "the JS worker should mirror the {} seam token",
            seam.require("js_token"),
        );
    }
}

#[test]
fn meta_recipe_parity_targets_exist_in_both_runtimes() {
    let records = records();
    let parities = of_kind(&records, "meta_parity");
    assert!(
        parities.len() >= 6,
        "expected every handler function to have a JS parity target"
    );
    let worker = read_worker_source();
    for parity in parities {
        let rust = parity.require("rust_function");
        let js = parity.require("js_function");
        let rust_source = read(parity.require("rust_file"));
        assert!(
            rust_source.contains(&format!("fn {rust}")),
            "{} should define fn {rust}",
            parity.require("rust_file"),
        );
        assert!(
            worker.contains(&format!("function {js}")),
            "the JS worker should mirror it as function {js}"
        );
    }
}

#[test]
fn meta_recipe_tests_pin_the_behaviour() {
    let records = records();
    let tests = of_kind(&records, "meta_test");
    assert!(
        tests.len() >= 3,
        "expected Rust generalization, project-lookup, and JS-parity pins"
    );
    for test in tests {
        let test_file = test.require("test_file");
        assert!(
            repo_root().join(test_file).exists(),
            "meta_test {} should reference an existing file {test_file}",
            test.require("suite"),
        );
        assert!(
            !test.require("pins").is_empty(),
            "meta_test {} should describe what it pins",
            test.require("suite"),
        );
    }
}

#[test]
fn meta_algorithm_doc_explains_and_links_the_recipe() {
    let doc = read("docs/meta-algorithm.md");
    for needle in [
        "# Meta-Algorithm",
        "data/meta/response-language-followup-recipe.lino",
        "tests/unit/specification/response_language_meta_algorithm.rs",
        "response-language follow-up",
    ] {
        assert!(
            doc.contains(needle),
            "docs/meta-algorithm.md should contain: {needle}"
        );
    }
}
