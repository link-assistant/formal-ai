//! Meta-algorithm grounding (issue #444).
//!
//! The maintainer asked for a meta-algorithm that can reproduce our Rust code on
//! a topic on demand, so we "learn from our own source code on how to produce
//! changes on the topic". `data/meta/procedural-howto-recipe.lino` is that
//! recipe: it names every seed role, handler function, evidence stage, JS parity
//! target, external-service toggle, and benchmark that make up the procedural
//! how-to topic, plus the eight ordered steps that generalise to any topic.
//!
//! These tests keep the recipe *grounded*: they load it and assert the live
//! source still matches. If the recipe and the code drift apart, CI fails — so
//! the recipe is always an accurate, executable description of how the code was
//! produced, not stale documentation.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/procedural-howto-recipe.lino";

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

fn read_worker_source() -> String {
    let mut source = read("src/web/formal_ai_worker.js");
    let worker_dir = repo_root().join("src/web/worker");
    if !worker_dir.exists() {
        return source;
    }

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

    for module in modules {
        let module_source = fs::read_to_string(&module)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", module.display()));
        source.push('\n');
        source.push_str(&module_source);
    }

    source
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

#[test]
fn meta_recipe_steps_are_complete_and_ordered() {
    let records = records();
    let recipe = of_kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1, "exactly one meta_recipe header expected");
    assert_eq!(recipe[0].require("topic"), "procedural_how_to");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how to generalise to other topics"
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
}

#[test]
fn meta_recipe_roles_match_live_constants_and_seed() {
    let records = records();
    let intent = read("src/seed/roles/intent.rs");
    let roles = of_kind(&records, "meta_role");
    assert_eq!(roles.len(), 11, "expected eleven documented lexicon roles");

    for role in roles {
        let value = role.require("role");
        let constant = role.require("const");
        let declaration = format!("pub const {constant}: &str = \"{value}\";");
        assert!(
            intent.contains(&declaration),
            "src/seed/roles/intent.rs should declare {declaration}"
        );
        let seed = read(role.require("seed_file"));
        assert!(
            seed.contains(&format!("role {value}")),
            "{} should declare `role {value}`",
            role.require("seed_file")
        );
    }
}

#[test]
fn meta_recipe_functions_exist_in_named_source() {
    let records = records();
    for function in of_kind(&records, "meta_function") {
        let name = function.require("function");
        let source = read(function.require("source_file"));
        assert!(
            source.contains(&format!("fn {name}")),
            "{} should define fn {name}",
            function.require("source_file")
        );
    }
}

#[test]
fn meta_recipe_dispatch_intents_are_wired() {
    let records = records();
    let dispatch = read("src/solver_dispatch.rs");
    for function in of_kind(&records, "meta_function") {
        if let Some(intent) = function.field("dispatch_intent") {
            let name = function.require("function");
            assert!(
                dispatch.contains(&format!("(\"{intent}\", {name})")),
                "src/solver_dispatch.rs should route (\"{intent}\", {name})"
            );
        }
    }
}

#[test]
fn meta_recipe_stages_match_emitted_plan() {
    let records = records();
    let handler = read("src/solver_handler_how.rs");
    let mut orders: Vec<usize> = Vec::new();
    for stage in of_kind(&records, "meta_stage") {
        let name = stage.require("stage");
        assert!(
            handler.contains(&format!("\"{name}\".to_owned()")),
            "src/solver_handler_how.rs should emit the {name} stage"
        );
        orders.push(stage.require("order").parse().expect("stage order integer"));
    }
    orders.sort_unstable();
    let expected_orders = (1..=orders.len()).collect::<Vec<_>>();
    assert_eq!(
        orders, expected_orders,
        "the discovery plan must list contiguously ordered stages"
    );
}

#[test]
fn meta_recipe_parity_targets_exist_in_worker() {
    let records = records();
    let handler = read("src/solver_handler_how.rs");
    let worker = read_worker_source();
    for parity in of_kind(&records, "meta_parity") {
        let rust = parity.require("rust_function");
        let js = parity.require("js_function");
        assert!(
            handler.contains(&format!("fn {rust}")),
            "src/solver_handler_how.rs should define fn {rust}"
        );
        assert!(
            worker.contains(&format!("function {js}")),
            "src/web/formal_ai_worker.js should mirror it as function {js}"
        );
    }
}

#[test]
fn meta_recipe_external_services_carry_settings_toggle() {
    let records = records();
    for service in of_kind(&records, "meta_external_service") {
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
fn meta_algorithm_doc_explains_and_links_the_recipe() {
    let doc = read("docs/meta-algorithm.md");
    for needle in [
        "# Meta-Algorithm",
        "data/meta/procedural-howto-recipe.lino",
        "tests/unit/specification/meta_algorithm.rs",
        "## The eight steps",
        "Generalising to a new topic",
    ] {
        assert!(
            doc.contains(needle),
            "docs/meta-algorithm.md should contain: {needle}"
        );
    }
}

#[test]
fn meta_recipe_benchmark_is_pinned_by_a_ratchet_test() {
    let records = records();
    for benchmark in of_kind(&records, "meta_benchmark") {
        let fixture = read(benchmark.require("fixture"));
        assert!(
            fixture.contains(benchmark.require("suite")),
            "fixture should declare suite {}",
            benchmark.require("suite")
        );
        let test_source = read("tests/unit/specification/procedural_howto_benchmarks.rs");
        assert!(
            test_source.contains(&format!("fn {}", benchmark.require("ratchet_test"))),
            "a ratchet test {} should pin the suite",
            benchmark.require("ratchet_test")
        );
    }
}
