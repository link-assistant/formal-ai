//! Agentic-coding meta-algorithm grounding (issue #468).
//!
//! The maintainer asked that "our Formal AI system should have enough skills
//! (meta algorithm, rust code) to actually call all the tools from any agentic
//! CLI, understand errors from tools, … to actually complete the task."
//! `data/meta/agentic-coding-recipe.lino` is that meta-algorithm written down:
//! it names every plan constant, advertised tool, capability, state-machine
//! stage, handler function, protocol primitive, loop bound, and exposure surface
//! that make up the deterministic agentic loop, plus the eight ordered steps that
//! generalise to a new task.
//!
//! These tests keep the recipe *grounded*: they load it and assert the live
//! source still matches. If the recipe and the code drift apart, CI fails — so
//! the recipe is always an accurate, executable description of how the loop was
//! produced, not stale documentation.

use std::fs;
use std::path::{Path, PathBuf};

const RECIPE: &str = "data/meta/agentic-coding-recipe.lino";

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

/// Collect the `order` fields of a record kind, sorted, as integers.
fn sorted_orders(records: &[Record], kind: &str) -> Vec<usize> {
    let mut orders: Vec<usize> = of_kind(records, kind)
        .iter()
        .map(|record| {
            record
                .require("order")
                .parse()
                .expect("order must be an integer")
        })
        .collect();
    orders.sort_unstable();
    orders
}

#[test]
fn meta_recipe_header_is_complete() {
    let records = records();
    let recipe = of_kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1, "exactly one meta_recipe header expected");
    assert_eq!(recipe[0].require("topic"), "agentic_coding");
    assert!(
        !recipe[0].require("summary").is_empty(),
        "recipe must summarise the agentic loop"
    );
    assert!(
        !recipe[0].require("generalization").is_empty(),
        "recipe must describe how to generalise to a new task"
    );
}

#[test]
fn meta_recipe_steps_are_complete_and_ordered() {
    let records = records();
    assert_eq!(
        sorted_orders(&records, "meta_step"),
        (1..=8).collect::<Vec<_>>(),
        "the meta-algorithm must list eight contiguously ordered steps"
    );
    // Every step points at a source file that actually exists.
    for step in of_kind(&records, "meta_step") {
        let seed = step.require("seed_file");
        assert!(
            repo_root().join(seed).exists(),
            "step {} references missing seed_file {seed}",
            step.require("id")
        );
    }
}

#[test]
fn meta_constants_are_declared_in_named_source() {
    let records = records();
    let constants = of_kind(&records, "meta_constant");
    assert_eq!(
        constants.len(),
        3,
        "SEARCH_QUERY, CANONICAL_SOURCE_URL, KB_PATH"
    );
    for constant in constants {
        let name = constant.require("constant");
        let source = read(constant.require("source_file"));
        assert!(
            source.contains(&format!("pub const {name}: &str")),
            "{} should declare pub const {name}: &str",
            constant.require("source_file")
        );
    }
}

#[test]
fn meta_tools_match_driver_planner_and_packages() {
    let records = records();
    let tools = of_kind(&records, "meta_tool");

    // The recipe lists exactly the four driver tools, in canonical recipe order.
    let advertised: Vec<&str> = tools.iter().map(|tool| tool.require("tool")).collect();
    assert_eq!(
        advertised,
        ["web_search", "web_fetch", "write_file", "run_command"],
        "the recipe must list the four DRIVER_TOOLS in order"
    );

    let driver = read("src/agentic_coding/driver.rs");
    let planner = read("src/agentic_coding/planner.rs");
    let packages = read("src/associative_package.rs");
    for tool in tools {
        let name = tool.require("tool");
        let capability = tool.require("capability");
        let permission = tool.require("permission");
        let package = tool.require("permission_package");

        assert!(
            driver.contains(&format!("\"{name}\"")),
            "driver.rs DRIVER_TOOLS should advertise {name}"
        );
        assert!(
            planner.contains(&format!("Capability::{capability}")),
            "planner.rs should classify into Capability::{capability}"
        );
        assert!(
            packages.contains(&format!("\"{permission}\"")),
            "associative_package.rs should grant permission {permission}"
        );
        assert!(
            packages.contains(&format!("\"{package}\"")),
            "associative_package.rs should declare package {package}"
        );
    }
}

#[test]
fn meta_stages_match_the_planner_state_machine() {
    let records = records();
    let planner = read("src/agentic_coding/planner.rs");

    // The documented state machine appears verbatim in the planner.
    assert!(
        planner.contains(
            "web_search → web_fetch → write_file(formalize) → run_command(verify) → final"
        ),
        "planner.rs should document the search → fetch → write → run → final state machine"
    );

    for stage in of_kind(&records, "meta_stage") {
        let order = stage.require("order");
        assert!(
            planner.contains(&format!("Step {order}:")),
            "planner.rs should mark `Step {order}:` for the {} stage",
            stage.require("stage")
        );
    }
    assert_eq!(
        sorted_orders(&records, "meta_stage"),
        (1..=5).collect::<Vec<_>>(),
        "the state machine must list five contiguously ordered stages"
    );
}

#[test]
fn meta_functions_exist_in_named_source() {
    let records = records();
    let functions = of_kind(&records, "meta_function");
    assert!(
        functions.len() >= 12,
        "the loop is built from at least a dozen named functions"
    );
    for function in functions {
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
fn meta_primitives_match_primitive_kinds() {
    let records = records();
    let formalize = read("src/agentic_coding/formalize.rs");
    assert!(
        formalize.contains("PRIMITIVE_KINDS: [&str; 9]"),
        "formalize.rs should declare the nine PRIMITIVE_KINDS"
    );
    for primitive in of_kind(&records, "meta_primitive") {
        let name = primitive.require("primitive");
        assert!(
            formalize.contains(&format!("\"{name}\"")),
            "PRIMITIVE_KINDS should include the {name} primitive"
        );
    }
    assert_eq!(
        sorted_orders(&records, "meta_primitive"),
        (1..=9).collect::<Vec<_>>(),
        "all nine protocol primitives must be listed, contiguously ordered"
    );
}

#[test]
fn meta_bound_caps_the_loop() {
    let records = records();
    let bounds = of_kind(&records, "meta_bound");
    assert_eq!(bounds.len(), 1, "the loop has exactly one turn cap");
    for bound in bounds {
        let name = bound.require("const");
        let value = bound.require("value");
        let source = read(bound.require("source_file"));
        assert!(
            source.contains(&format!("const {name}: usize = {value};")),
            "{} should cap the loop with const {name}: usize = {value};",
            bound.require("source_file")
        );
    }
}

#[test]
fn meta_surfaces_expose_the_loop() {
    let records = records();
    let surfaces = of_kind(&records, "meta_surface");
    assert_eq!(
        surfaces.len(),
        3,
        "the loop is exposed as a CLI subcommand, an example, and an integration test"
    );
    for surface in surfaces {
        let source = read(surface.require("source_file"));
        let needle = surface.require("needle");
        assert!(
            source.contains(needle),
            "{} ({}) should contain `{needle}`",
            surface.require("source_file"),
            surface.require("surface")
        );
    }
}

#[test]
fn agentic_meta_algorithm_doc_explains_the_recipe() {
    let doc = read("docs/meta-algorithm.md");
    for needle in [
        "## The agentic-coding meta-algorithm",
        "data/meta/agentic-coding-recipe.lino",
        "tests/unit/specification/agentic_meta_algorithm.rs",
        "agent_mode",
    ] {
        assert!(
            doc.contains(needle),
            "docs/meta-algorithm.md should contain: {needle}"
        );
    }
}
