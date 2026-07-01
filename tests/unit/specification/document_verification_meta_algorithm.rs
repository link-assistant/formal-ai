//! Document-verification meta-algorithm grounding (issue #535).
//!
//! Issue #535 asked us to fully support attached-document verification across
//! every interface surface, generalise it to the whole verification class
//! (originality, plagiarism, authenticity, veracity, fact-check) in all
//! languages "expressed recursively through meanings (meta language)", ground
//! "all meanings … in external data sources", and "fully use
//! github.com/link-foundation/relative-meta-logic for relative statements
//! probability". `data/meta/document-verification-recipe.lino` is that recipe:
//! it names every ordered step, seed role, external Wikidata grounding, handler
//! function, Rust↔JS parity target, trusted external source, and pinning test
//! that make up the document-verification topic.
//!
//! These tests keep the recipe *grounded*: they load it and assert the live
//! source still matches. If the recipe and the code drift apart, CI fails — so
//! the recipe is always an accurate, executable description of how the code was
//! produced, not stale documentation. The parser mirrors
//! `tests/unit/specification/meta_algorithm.rs`.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/document-verification-recipe.lino";

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
    assert_eq!(recipe[0].require("topic"), "document_verification");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how to generalise across the verification class"
    );
    assert!(
        !recipe[0].require("summary").is_empty(),
        "recipe must summarise the document-verification handler"
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
    let intent_roles = read("src/seed/roles/intent.rs");
    let roles = of_kind(&records, "meta_role");
    assert_eq!(
        roles.len(),
        3,
        "expected the three verification trigger roles (action, subject, document)"
    );

    for role in roles {
        let value = role.require("role");
        let constant = role.require("const");
        let declaration = format!("pub const {constant}: &str = \"{value}\";");
        assert!(
            intent_roles.contains(&declaration),
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
        groundings.len() >= 6,
        "expected the verification-class groundings (verification, plagiarism, \
         originality, authenticity, fact-checking, document)"
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
        "expected the recogniser/ingest/statement/relative-meta-logic chain to be documented"
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
fn meta_recipe_dispatch_intents_are_wired() {
    let records = records();
    let dispatch = read("src/solver_dispatch.rs");
    let mut wired = 0;
    for function in of_kind(&records, "meta_function") {
        if let Some(intent) = function.field("dispatch_intent") {
            let name = function.require("function");
            assert!(
                dispatch.contains(&format!("(\"{intent}\", {name})")),
                "src/solver_dispatch.rs should route (\"{intent}\", {name})"
            );
            wired += 1;
        }
    }
    assert!(
        wired >= 1,
        "the document-verification handler must be wired into the dispatch table"
    );
}

#[test]
fn meta_recipe_parity_targets_exist_in_both_runtimes() {
    let records = records();
    let parities = of_kind(&records, "meta_parity");
    assert!(
        parities.len() >= 7,
        "expected every handler and statement-verification function to have a JS parity target"
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
fn meta_recipe_external_service_carries_settings_toggle() {
    let records = records();
    let services = of_kind(&records, "meta_external_service");
    assert!(
        !services.is_empty(),
        "the trusted original-journalism source must be recorded"
    );
    for service in services {
        let registry = read(service.require("registry"));
        let source = service.require("source");
        let settings_key = service.require("settings_key");
        assert!(
            registry.contains(&format!("source {source}")),
            "registry should declare source {source}"
        );
        assert!(
            registry.contains(&format!("settings_key {settings_key}")),
            "registry should bind {source} to settings_key {settings_key}"
        );
    }
}

#[test]
fn meta_recipe_tests_pin_the_behaviour() {
    let records = records();
    let tests = of_kind(&records, "meta_test");
    assert!(
        tests.len() >= 4,
        "expected unit, relative-meta-logic, statement-verification, and e2e pins"
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
        "data/meta/document-verification-recipe.lino",
        "tests/unit/specification/document_verification_meta_algorithm.rs",
        "document verification",
    ] {
        assert!(
            doc.contains(needle),
            "docs/meta-algorithm.md should contain: {needle}"
        );
    }
}
