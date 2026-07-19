//! Budget-driven search meta-algorithm grounding (issue #662, journey F4).
//!
//! `GOALS.md` (Universal Solver Goals) asks that, "when no reusable part exists,
//! combine reasoning, random search, and evolutionary search according to the
//! available compute budget instead of giving up".
//! `data/meta/budget-search-recipe.lino` is that recipe: it names every ordered
//! step, seed role, external Wikidata grounding, handler function, the loop
//! integration point (with its CLI flag and environment variable), and the
//! pinning tests that make up the budget-search synthesis stage.
//!
//! These tests keep the recipe *grounded*: they load it and assert the live
//! source still matches. If the recipe and the code drift apart, CI fails — so
//! the recipe is always an accurate, executable description of how the code was
//! produced, not stale documentation. The parser mirrors
//! `tests/unit/specification/document_verification_meta_algorithm.rs`.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/budget-search-recipe.lino";

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

#[test]
fn meta_recipe_steps_are_complete_and_ordered() {
    let records = records();
    let recipe = of_kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1, "exactly one meta_recipe header expected");
    assert_eq!(recipe[0].require("topic"), "budget_search");
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how to generalise the reach-a-target class"
    );
    assert!(
        !recipe[0].require("summary").is_empty(),
        "recipe must summarise the budget-search stage"
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
        (1..=9).collect::<Vec<_>>(),
        "the meta-algorithm must list nine contiguously ordered steps"
    );

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
    let roles = of_kind(&records, "meta_role");
    assert_eq!(
        roles.len(),
        4,
        "expected the three reachability trigger roles plus the operator-vocabulary role"
    );

    for role in roles {
        let value = role.require("role");
        let constant = role.require("const");
        let const_source = read(role.require("const_file"));
        let declaration = format!("pub const {constant}: &str = \"{value}\";");
        assert!(
            const_source.contains(&declaration),
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
        "expected the three reachability groundings plus the division/modulo generalisation proof"
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
        functions.len() >= 10,
        "expected the recognise/derive/generate/search/propose chain to be documented"
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
fn meta_recipe_integration_is_wired_into_the_loop_with_budget_controls() {
    let records = records();
    let integrations = of_kind(&records, "meta_integration");
    assert_eq!(
        integrations.len(),
        1,
        "expected one loop-integration record for the budget-search stage"
    );
    let integration = integrations[0];

    // The stage is called from the universal loop.
    let function = integration.require("function");
    let call_site = read(integration.require("call_site"));
    assert!(
        call_site.contains(&format!("{function}(")),
        "{} should call {function}",
        integration.require("call_site"),
    );

    // The compute budget is a real config field, CLI flag, and environment
    // variable, so the search depth is controllable.
    let solver = read("src/solver.rs");
    assert!(
        solver.contains(&format!("pub {}: u32", integration.require("budget_field"))),
        "src/solver.rs should expose the {} config field",
        integration.require("budget_field"),
    );
    let main = read("src/main.rs");
    let env_var = integration.require("env_var");
    assert!(
        main.contains(env_var),
        "src/main.rs should bind the {env_var} environment variable"
    );
    // clap derives the `--compute-budget` long flag from the snake_case field
    // name plus `#[arg(long, …)]`, so the grounded token is the field itself.
    let cli_field = integration.require("cli_flag").replace('-', "_");
    assert!(
        main.contains(&format!("{cli_field}: Option<u32>")),
        "src/main.rs should expose the --{} CLI argument as field {cli_field}",
        integration.require("cli_flag"),
    );
    assert!(
        main.contains("#[arg(long, env ="),
        "src/main.rs should expose the compute budget as a long flag bound to the env var"
    );
}

#[test]
fn meta_recipe_tests_pin_the_behaviour() {
    let records = records();
    let tests = of_kind(&records, "meta_test");
    assert!(
        tests.len() >= 2,
        "expected the unit suite and this grounding spec to be pinned"
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
        "data/meta/budget-search-recipe.lino",
        "tests/unit/specification/budget_search_meta_algorithm.rs",
        "budget-driven search",
    ] {
        assert!(
            doc.contains(needle),
            "docs/meta-algorithm.md should contain: {needle}"
        );
    }
}
