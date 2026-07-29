//! Executable grounding for the topic-neutral grounded-action recipe (#840).

use std::fs;
use std::path::Path;
use std::process::Command;

const RECIPE: &str = "data/meta/grounded-action-recipe.lino";

#[derive(Debug)]
struct Record {
    kind: String,
    fields: Vec<(String, String)>,
}

impl Record {
    fn field(&self, name: &str) -> &str {
        self.fields
            .iter()
            .find_map(|(key, value)| (key == name).then_some(value.as_str()))
            .unwrap_or_else(|| panic!("{} record is missing {name}", self.kind))
    }
}

fn read(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

fn records() -> Vec<Record> {
    let text = read(RECIPE);
    let mut groups: Vec<Vec<&str>> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if !line.starts_with(char::is_whitespace) {
            groups.push(Vec::new());
        }
        groups.last_mut().expect("record header").push(line);
    }
    groups
        .into_iter()
        .map(|lines| {
            let mut fields = Vec::new();
            for line in lines.into_iter().skip(1) {
                let (key, raw) = line.trim().split_once(' ').expect("field and value");
                fields.push((key.to_owned(), raw.trim().trim_matches('"').to_owned()));
            }
            let kind = fields
                .iter()
                .find_map(|(key, value)| (key == "record_type").then_some(value.clone()))
                .expect("record_type");
            Record { kind, fields }
        })
        .collect()
}

fn kind<'a>(records: &'a [Record], wanted: &str) -> Vec<&'a Record> {
    records
        .iter()
        .filter(|record| record.kind == wanted)
        .collect()
}

#[test]
fn grounded_action_recipe_has_eight_ordered_topic_neutral_steps() {
    let records = records();
    let recipe = kind(&records, "meta_recipe");
    assert_eq!(recipe.len(), 1);
    assert_eq!(recipe[0].field("topic"), "grounded_action");
    assert!(recipe[0]
        .field("generalization")
        .contains("any tool-backed task"));

    let mut orders = kind(&records, "meta_step")
        .into_iter()
        .map(|step| step.field("order").parse::<usize>().expect("order"))
        .collect::<Vec<_>>();
    orders.sort_unstable();
    assert_eq!(orders, (1..=8).collect::<Vec<_>>());
}

#[test]
fn grounded_action_recipe_roles_are_live_seed_contracts() {
    let records = records();
    let constants = read("src/seed/roles/intent.rs");
    for role in kind(&records, "meta_role") {
        let value = role.field("role");
        assert!(
            constants.contains(&format!(
                "pub const {}: &str = \"{value}\";",
                role.field("const")
            )),
            "{value} must have a Rust role constant"
        );
        assert!(
            read(role.field("seed_file")).contains(&format!("role {value}")),
            "{value} must be declared in seed data"
        );
    }
}

#[test]
fn grounded_action_recipe_functions_and_runtime_parity_exist() {
    let records = records();
    for function in kind(&records, "meta_function") {
        assert!(
            read(function.field("source_file"))
                .contains(&format!("fn {}", function.field("function"))),
            "{function:?}"
        );
    }
    for parity in kind(&records, "meta_parity") {
        assert!(read(parity.field("rust_source"))
            .contains(&format!("fn {}", parity.field("rust_function"))));
        assert!(read(parity.field("js_source"))
            .contains(&format!("function {}", parity.field("js_function"))));
    }
}

#[test]
fn grounded_action_recipe_benchmarks_are_executable_tests() {
    let records = records();
    let benchmarks = kind(&records, "meta_benchmark");
    assert_eq!(benchmarks.len(), 3);
    for benchmark in benchmarks {
        let test = benchmark.field("test");
        assert!(
            read(benchmark.field("source_file")).contains(&format!("fn {test}")),
            "{test} must remain an executable test"
        );
        assert!(!benchmark.field("coverage").is_empty());
    }
}

#[test]
fn grounded_action_self_authorship_is_measured_and_preserved() {
    let records = records();
    let evidence = kind(&records, "meta_evidence");
    assert_eq!(evidence.len(), 1);
    let session = evidence[0].field("session");
    let stream = read(evidence[0].field("evidence_file"));
    assert!(
        stream.contains(session),
        "{session} is absent from Agent evidence"
    );

    let invariant = read(evidence[0].field("source_file"));
    let generated =
        read("docs/case-studies/issue-840/self-hosting/grounded-action-authored-invariant.lino");
    assert_eq!(invariant.trim_end(), generated.trim_end());

    let decomposition = read("docs/case-studies/issue-840/self-hosting/decomposition.lino");
    assert_eq!(decomposition.matches("issue_840_smallest_leaf_").count(), 5);
    assert_eq!(
        decomposition
            .matches("authorship formal_ai_agent_cli")
            .count(),
        1
    );
    assert!(decomposition.contains("formal_ai_authored_percent 20"));
}

#[test]
fn grounded_action_browser_parity_executes() {
    let output = Command::new("node")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/issue_840_js_parity.mjs"))
        .output()
        .expect("Node.js is required by the browser-worker test suite");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("8/8 JS routing cases passed"),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
