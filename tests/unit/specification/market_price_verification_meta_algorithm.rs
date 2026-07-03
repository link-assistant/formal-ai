//! Market-price verification meta-algorithm grounding (issue #493).
//!
//! Issue #493 asked us not merely to catch one false screenshot claim
//! (`ETH in 2024: $1,700`) but to "support the entire class of similar
//! questions" — any checkable numeric market-price claim, generalised across
//! assets, periods, and every supported language, grounded in external data and
//! weighed under github.com/link-foundation/relative-meta-logic.
//! `data/meta/market-price-verification-recipe.lino` is that recipe: it names
//! every ordered step, external Wikidata grounding, data source, handler
//! function, Rust-JS parity target, and pinning test that make up the
//! market-price-verification sub-algorithm of document verification.
//!
//! These tests keep the recipe *grounded*: they load it and assert the live
//! source still matches. If the recipe and the code drift apart, CI fails — so
//! the recipe is always an accurate, executable description of how the code was
//! produced, not stale documentation. The parser mirrors
//! `tests/unit/specification/document_verification_meta_algorithm.rs`.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/market-price-verification-recipe.lino";

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
    assert_eq!(recipe[0].require("topic"), "market_price_verification");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how to generalise across the market-price class"
    );
    assert!(
        !recipe[0].require("summary").is_empty(),
        "recipe must summarise the market-price fact check"
    );

    let steps = of_kind(&records, "meta_step");
    let mut orders: Vec<usize> = steps
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
        (1..=steps.len()).collect::<Vec<_>>(),
        "the meta-algorithm steps must be contiguously ordered from 1"
    );

    // Every step must have a non-empty detail and point at a real file it was
    // produced from.
    for step in &steps {
        assert!(
            !step.require("detail").is_empty(),
            "meta_step {} must describe what it does",
            step.require("id"),
        );
        let seed_file = step.require("seed_file");
        assert!(
            repo_root().join(seed_file).exists(),
            "meta_step {} seed_file {seed_file} should exist",
            step.require("id"),
        );
    }
}

#[test]
fn meta_recipe_groundings_resolve_to_cached_external_entities() {
    let records = records();
    let groundings = of_kind(&records, "meta_grounding");
    assert!(
        groundings.len() >= 2,
        "expected the ETH and BTC asset groundings"
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
        // The asset must actually be grounded in that Q-id in the seed registry.
        let seed = read(grounding.require("seed_file"));
        assert!(
            seed.contains(wikidata),
            "{} should ground an asset in {wikidata}",
            grounding.require("seed_file"),
        );
    }
}

#[test]
fn meta_recipe_data_source_exists_and_declares_registry() {
    let records = records();
    let data = of_kind(&records, "meta_data");
    assert!(
        !data.is_empty(),
        "the data-driven reference registry must be recorded"
    );
    for record in data {
        let data_file = record.require("data_file");
        assert!(
            repo_root().join(data_file).exists(),
            "meta_data {} should reference an existing file {data_file}",
            record.require("id"),
        );
        let contents = read(data_file);
        let needle = record.require("contains");
        assert!(
            contents.contains(needle),
            "{data_file} should contain the registry root `{needle}`",
        );
    }
}

#[test]
fn meta_recipe_functions_exist_in_named_source() {
    let records = records();
    let functions = of_kind(&records, "meta_function");
    assert!(
        functions.len() >= 8,
        "expected the registry/recogniser/split/parse/weigh/trace chain to be documented"
    );
    for function in &functions {
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
fn meta_recipe_parity_targets_exist_in_both_runtimes() {
    let records = records();
    let parities = of_kind(&records, "meta_parity");
    assert!(
        parities.len() >= 5,
        "expected the registry loader and the extract/assess chain to have JS parity targets"
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
        "expected unit, statement-verification, and e2e pins"
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
        "data/meta/market-price-verification-recipe.lino",
        "tests/unit/specification/market_price_verification_meta_algorithm.rs",
        "market-price",
    ] {
        assert!(
            doc.contains(needle),
            "docs/meta-algorithm.md should contain: {needle}"
        );
    }
}
