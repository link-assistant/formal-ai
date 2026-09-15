//! Rebuild the browser's derived recurrence cache from trusted replay fixtures.
//!
//! Usage:
//!   cargo run --example generate_recurrence_source_cache
//!   cargo run --example generate_recurrence_source_cache -- --write

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::coding_function_catalog::wikifunctions::{
    fetch_abstract_implementation, fetch_function, fetch_function_descriptors, fetch_testers,
};
use formal_ai::coding_recurrence::{Expression, Operation, Recurrence, formalize};
use formal_ai::{CachedSourceClient, FetchError, SourceTransport};
use serde_json::Value;
use sha2::{Digest, Sha256};

const TARGET: &str = "src/web/source-cache/wikifunctions-recurrences.lino";
const WIKIDATA_FIBONACCI: &str = "https://www.wikidata.org/wiki/Special:EntityData/Q23835349.json";
const WIKIDATA_FACTORIAL: &str = "https://www.wikidata.org/wiki/Special:EntityData/Q120976.json";

#[derive(Clone)]
struct FixtureTransport {
    root: PathBuf,
}

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let file = if url.contains("zids=Z13835") {
            "fetch-Z13835-en.json"
        } else if url.contains("zids=Z13864") {
            "fetch-Z13864-abstract-en.json"
        } else if url.contains("zids=Z13667") {
            "fetch-Z13667-en.json"
        } else if url.contains("zids=Z13863") {
            "fetch-Z13863-abstract-en.json"
        } else if url.contains("zids=Z802%7CZ13695%7CZ13521%7CZ13582%7CZ13569%7CZ13522%7CZ13539") {
            "fetch-recurrence-operators-en.json"
        } else if url.contains("zids=Z13869%7CZ17391%7CZ17398%7CZ34276") {
            "fetch-recurrence-testers-en.json"
        } else if url.contains("zids=Z13840%7CZ13865%7CZ13866%7CZ13867") {
            "fetch-factorial-testers-en.json"
        } else {
            return Err(FetchError::Transport(format!(
                "unexpected recurrence fixture URL: {url}"
            )));
        };
        fs::read(self.root.join(file)).map_err(|error| FetchError::Transport(error.to_string()))
    }
}

fn line(out: &mut String, indent: usize, name: &str, value: Option<&str>) {
    out.push_str(&" ".repeat(indent));
    out.push_str(name);
    if let Some(value) = value {
        out.push(' ');
        out.push_str(&serde_json::to_string(value).expect("serialize Links value"));
    }
    out.push('\n');
}

fn expression(out: &mut String, indent: usize, value: &Expression) {
    match value {
        Expression::Parameter(name) => line(out, indent, "parameter", Some(name)),
        Expression::Literal(value) => line(out, indent, "literal", Some(&value.to_string())),
        Expression::Recur(argument) => {
            line(out, indent, "recur", None);
            expression(out, indent + 2, argument);
        }
        Expression::Apply(operation, operands) => {
            let name = match operation {
                Operation::Conditional => "conditional",
                Operation::LessEqual => "less_equal",
                Operation::Equal => "equal",
                Operation::Add => "add",
                Operation::Multiply => "multiply",
                Operation::Subtract => "subtract",
                Operation::SubtractOne => "subtract_one",
            };
            line(out, indent, name, None);
            for operand in operands {
                expression(out, indent + 2, operand);
            }
        }
    }
}

fn aliases(root: &Path, qid: &str) -> (Vec<(String, String)>, String) {
    let path = root.join(format!("fetch-{qid}.json"));
    let bytes = fs::read(&path).expect("read Wikidata fixture");
    let value: Value = serde_json::from_slice(&bytes).expect("parse Wikidata fixture");
    let entity = &value["entities"][qid];
    let mut values = BTreeSet::new();
    for language in ["en", "ru", "hi", "zh"] {
        if let Some(label) = entity["labels"][language]["value"].as_str() {
            values.insert((language.to_owned(), label.to_owned()));
        }
        if let Some(items) = entity["aliases"][language].as_array() {
            for alias in items {
                if let Some(text) = alias["value"].as_str() {
                    values.insert((language.to_owned(), text.to_owned()));
                }
            }
        }
    }
    let digest = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    (values.into_iter().collect(), digest)
}

fn captured_at(root: &Path) -> String {
    fs::read_to_string(root.join("capture-manifest.lino"))
        .expect("read capture manifest")
        .lines()
        .find_map(|line| line.trim().strip_prefix("fetched_at "))
        .map(|value| value.trim_matches('"').to_owned())
        .expect("capture manifest has fetched_at")
}

fn record(
    out: &mut String,
    recurrence: &Recurrence,
    source_aliases: &[(String, String)],
    concept_qid: &str,
    concept_sha256: &str,
    tests: &[formal_ai::coding_function_catalog::wikifunctions::FunctionTest],
) {
    line(out, 2, "recurrence", Some(&recurrence.source_function_zid));
    line(out, 4, "label", Some(&recurrence.source_label));
    line(
        out,
        4,
        "identifier",
        Some(&recurrence.suggested_identifier()),
    );
    for (language, text) in source_aliases {
        line(out, 4, "alias", None);
        line(out, 6, "language", Some(language));
        line(out, 6, "text", Some(text));
    }
    line(out, 4, "parameter", Some(&recurrence.parameter));
    line(out, 4, "expression", None);
    expression(out, 6, &recurrence.expression);
    line(out, 4, "termination_measure", Some(&recurrence.parameter));
    for offset in &recurrence.predecessor_offsets {
        line(out, 4, "predecessor_offset", Some(&offset.to_string()));
    }
    for test in tests
        .iter()
        .filter(|test| test.function_zid == recurrence.source_function_zid)
    {
        line(out, 4, "source_test", Some(&test.zid));
        for argument in &test.arguments {
            line(out, 6, "argument", Some(argument));
        }
        line(out, 6, "expected", Some(&test.expected));
        line(out, 6, "source_url", Some(&test.source_url));
        line(out, 6, "sha256", Some(&test.sha256));
    }
    line(
        out,
        4,
        "abstract_implementation",
        Some(&recurrence.source_implementation_zid),
    );
    line(out, 4, "source_url", Some(&recurrence.source_url));
    line(out, 4, "sha256", Some(&recurrence.source_sha256));
    line(out, 4, "fetched_at", Some(&recurrence.fetched_at));
    line(out, 4, "license", Some(&recurrence.license));
    line(out, 4, "concept", Some(concept_qid));
    line(
        out,
        4,
        "concept_source_url",
        Some(if concept_qid == "Q23835349" {
            WIKIDATA_FIBONACCI
        } else {
            WIKIDATA_FACTORIAL
        }),
    );
    line(out, 4, "concept_sha256", Some(concept_sha256));
    line(out, 4, "concept_license", Some("CC0-1.0"));
}

fn generate(root: &Path) -> String {
    let cache = std::env::temp_dir().join(format!(
        "formal-ai-recurrence-cache-generator-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&cache);
    let client = CachedSourceClient::new(
        &cache,
        FixtureTransport {
            root: root.to_owned(),
        },
    )
    .with_online(true)
    .with_clock(|| 1_789_344_000);
    let operator_ids = [
        "Z802", "Z13695", "Z13521", "Z13582", "Z13569", "Z13522", "Z13539",
    ]
    .map(str::to_owned);
    let operators = fetch_function_descriptors(&client, &operator_ids).expect("operators");

    let fibonacci_definition = fetch_function(&client, "Z13835").expect("Fibonacci function");
    let fibonacci_abstract =
        fetch_abstract_implementation(&client, "Z13864").expect("Fibonacci abstract");
    let mut fibonacci = formalize(&fibonacci_definition, &fibonacci_abstract, &operators)
        .expect("Fibonacci formalization");
    let fibonacci_tests = fetch_testers(
        &client,
        &[
            "Z13869".to_owned(),
            "Z17391".to_owned(),
            "Z17398".to_owned(),
            "Z34276".to_owned(),
        ],
    )
    .expect("Fibonacci tests");

    let factorial_definition = fetch_function(&client, "Z13667").expect("factorial function");
    let factorial_abstract =
        fetch_abstract_implementation(&client, "Z13863").expect("factorial abstract");
    let mut factorial = formalize(&factorial_definition, &factorial_abstract, &operators)
        .expect("factorial formalization");
    let factorial_tests = fetch_testers(
        &client,
        &[
            "Z13840".to_owned(),
            "Z13865".to_owned(),
            "Z13866".to_owned(),
            "Z13867".to_owned(),
        ],
    )
    .expect("factorial tests");

    let (fibonacci_aliases, fibonacci_concept_sha256) = aliases(root, "Q23835349");
    let (factorial_aliases, factorial_concept_sha256) = aliases(root, "Q120976");
    let captured_at = captured_at(root);
    fibonacci.fetched_at.clone_from(&captured_at);
    factorial.fetched_at = captured_at;
    let mut out = String::new();
    line(&mut out, 0, "recurrence_source_cache", None);
    line(
        &mut out,
        2,
        "generated_by",
        Some("cargo run --example generate_recurrence_source_cache -- --write"),
    );
    record(
        &mut out,
        &fibonacci,
        &fibonacci_aliases,
        "Q23835349",
        &fibonacci_concept_sha256,
        &fibonacci_tests,
    );
    record(
        &mut out,
        &factorial,
        &factorial_aliases,
        "Q120976",
        &factorial_concept_sha256,
        &factorial_tests,
    );
    let _ = fs::remove_dir_all(cache);
    out
}

fn main() {
    let root = Path::new("tests/fixtures/coding-discovery/wikifunctions");
    let generated = generate(root);
    if std::env::args().any(|argument| argument == "--write") {
        if let Some(parent) = Path::new(TARGET).parent() {
            fs::create_dir_all(parent).expect("create recurrence cache directory");
        }
        fs::write(TARGET, generated).expect("write recurrence source cache");
        return;
    }
    let current = fs::read_to_string(TARGET).unwrap_or_default();
    assert_eq!(
        current, generated,
        "{TARGET} is stale; run the generator with --write"
    );
}
