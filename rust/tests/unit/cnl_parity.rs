//! Cross-runtime contract for the generalized CNL engine (issue #1584): the
//! JavaScript reference writes one fixture per language with
//! `node experiments/issue-1584/parity-fixture.mjs --write`; the Rust port must
//! reproduce every projection digest and every controlled-dialogue turn.
//! JavaScript root: experiments/issue-1584/parity-fixture.mjs.
use ::formal_ai::cnl_output::grammar::{Graph, Slots, graph_key};
use ::formal_ai::cnl_output::{
    ControlledDialogue, OutputConfig, OutputDocument, OutputPart, OutputPolicy, cnl_policy,
    controlled_dialogue, output_configuration, project_output, validate_output,
};
use ::formal_ai::{ConversationTurn, source_fetch::sha256_hex};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/parity/issue-1584-cnl")
}

fn fixtures() -> Vec<Value> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(fixture_dir())
        .expect("parity fixture directory")
        .map(|entry| entry.expect("fixture entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no parity fixtures");
    files
        .iter()
        .map(|path| {
            let text = std::fs::read_to_string(path).expect("fixture text");
            serde_json::from_str(&text).expect("fixture json")
        })
        .collect()
}

/// Every fact, its converse and both polarities, as statements and questions.
fn fact_graphs(policy: &OutputPolicy) -> Vec<Graph> {
    let mut graphs: Vec<Graph> = Vec::new();
    for fact in &policy.shared.facts {
        let frame = policy
            .shared
            .frames
            .iter()
            .find(|item| item.id == fact.frame)
            .expect("fact frame");
        let reversed: Vec<String> = fact.args.iter().rev().cloned().collect();
        for args in [&fact.args, &reversed] {
            for polarity in ["positive", "negative"] {
                for mood in ["statement", "question"] {
                    let mut slots = Slots::default();
                    for (slot, value) in frame.slots.iter().zip(args) {
                        slots.insert(slot, value);
                    }
                    let graph = Graph::new(&frame.id, slots, polarity, mood);
                    let key = graph_key(&graph);
                    // A JavaScript Map keeps the first insertion position and the last value.
                    match graphs.iter_mut().find(|known| graph_key(known) == key) {
                        Some(known) => *known = graph,
                        None => graphs.push(graph),
                    }
                }
            }
        }
    }
    graphs
}

/// Dialogue frames that every output style must also realize.
fn dialogue_graphs() -> Vec<Graph> {
    vec![
        Graph::new("reply", Slots::default(), "positive", "statement"),
        Graph::new("reply", Slots::default(), "negative", "statement"),
        Graph::new(
            "clarify",
            Slots::from([("count", "2")]),
            "positive",
            "statement",
        ),
        Graph::new("choose", Slots::default(), "positive", "request"),
    ]
}

fn projection_rows(
    language: &str,
    profile: &str,
    densities: &[String],
    policy: &OutputPolicy,
) -> Vec<String> {
    let mut rows = Vec::new();
    for graph in fact_graphs(policy).into_iter().chain(dialogue_graphs()) {
        for density in densities {
            let configuration =
                output_configuration(Some(profile), Some(density), policy).expect("configuration");
            let document = OutputDocument {
                language: language.to_owned(),
                source: String::new(),
                parts: vec![OutputPart::Graph {
                    graph: graph.clone(),
                    surface: None,
                }],
            };
            let text = project_output(&document, configuration, policy);
            let report = validate_output(&text, language, configuration, policy, &());
            rows.push(
                [
                    graph_key(&graph),
                    density.clone(),
                    text,
                    report.compliant.to_string(),
                    report.violations.join(","),
                    report.unchecked.join(","),
                ]
                .join("\t"),
            );
        }
    }
    rows
}

#[test]
fn projection_digests_match_the_javascript_reference() {
    let policy = cnl_policy();
    let mut mismatches = Vec::new();
    for fixture in fixtures() {
        let language = fixture["language"].as_str().expect("language");
        assert!(
            policy.grammar(language).is_some(),
            "{language}: the Rust engine did not load this grammar"
        );
        let densities: Vec<String> = fixture["densities"]
            .as_array()
            .expect("densities")
            .iter()
            .map(|item| item.as_str().expect("density").to_owned())
            .collect();
        for (profile, expected) in fixture["projections"].as_object().expect("projections") {
            let rows = projection_rows(language, profile, &densities, policy);
            let actual = sha256_hex(format!("{}\n", rows.join("\n")).as_bytes());
            if actual != expected.as_str().expect("digest") {
                eprintln!(
                    "{language}/{profile}: digest mismatch; compare with \
                     `node experiments/issue-1584/parity-fixture.mjs --rows {language} {profile}`:\n{}",
                    rows.join("\n")
                );
                mismatches.push(format!("{language}/{profile}"));
            }
        }
    }
    assert!(mismatches.is_empty(), "digest mismatches: {mismatches:?}");
}

fn configuration_of(value: &Value, policy: &OutputPolicy) -> OutputConfig {
    output_configuration(
        value["output_standard"].as_str(),
        value["lexical_density"].as_str(),
        policy,
    )
    .expect("configuration")
}

fn history_of(value: &Value) -> Vec<ConversationTurn> {
    value
        .as_array()
        .expect("history")
        .iter()
        .map(|turn| {
            let content = turn["content"].as_str().expect("content");
            match turn["role"].as_str() {
                Some("user") => ConversationTurn::user(content),
                Some("assistant") => ConversationTurn::assistant(content),
                role => panic!("unexpected role {role:?}"),
            }
        })
        .collect()
}

fn turn_json(turn: Option<ControlledDialogue>) -> Value {
    match turn {
        None => Value::Null,
        Some(ControlledDialogue::Task { language, prompt }) => {
            json!({"intent": "cnl_task", "language": language, "prompt": prompt})
        }
        Some(ControlledDialogue::Rendered {
            intent,
            language,
            content,
            output_projection,
        }) => json!({
            "intent": intent,
            "language": language,
            "content": content,
            "compliant": output_projection.report.compliant,
        }),
    }
}

#[test]
fn controlled_dialogue_turns_match_the_javascript_reference() {
    let policy = cnl_policy();
    let mut failures = Vec::new();
    let mut total = 0;
    for fixture in fixtures() {
        let language = fixture["language"].as_str().expect("language");
        for case in fixture["dialogue"].as_array().expect("dialogue") {
            total += 1;
            let prompt = case["prompt"].as_str().expect("prompt");
            let history = history_of(&case["history"]);
            let configuration = configuration_of(&case["configuration"], policy);
            let actual = turn_json(controlled_dialogue(prompt, &history, configuration, policy));
            if actual != case["expected"] {
                failures.push(format!(
                    "{language}: {prompt:?} {}\n  expected {}\n  actual   {actual}",
                    case["configuration"], case["expected"]
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {total} dialogue turns differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
