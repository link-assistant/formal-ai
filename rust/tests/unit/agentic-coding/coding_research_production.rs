//! Hosted native production route: synthetic transport and in-memory effects, never live discovery proof.
use formal_ai::agentic_coding::coding_research::{derive_coding_procedure, independent_acceptance};
use formal_ai::agentic_coding::module_function::{ModuleFunctionRequest, module_function_request};
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const TOOLS: [&str; 5] = ["read_file", "write_file", "bash", "web_search", "web_fetch"];
const TAP: &str =
    "TAP version 13\n1..1\n# tests 1\n# pass 1\n# fail 0\n# skipped 0\n# cancelled 0\n";

fn rows() -> Vec<Value> {
    serde_json::from_str(include_str!("../../fixtures/coding-research/held-out.json"))
        .expect("shared declarative held-out fixtures")
}
fn request(row: &Value) -> ModuleFunctionRequest {
    let prompt = row["prompt"].as_str().expect("prompt");
    let mut request = module_function_request(
        &prompt.replace(
            row["module"].as_str().expect("module"),
            &row["module"]
                .as_str()
                .expect("module")
                .replace(".ts", ".mjs"),
        ),
    )
    .expect("same declaration and path request grammar");
    row["language"]
        .as_str()
        .expect("language")
        .clone_into(&mut request.language);
    row["module"]
        .as_str()
        .expect("module")
        .clone_into(&mut request.module);
    request
}
fn page(row: &Value, source: &str) -> Value {
    json!({"url":row["url"],"query":"synthetic-query","sha256":"synthetic-digest",
        "codeBlocks":[{"language":row["language"],"text":source}]})
}
struct Observation {
    calls: Vec<String>,
    final_answer: String,
}
fn drive(row: &Value, files: &mut BTreeMap<String, String>, receipt: &str) -> Observation {
    let mut messages = vec![ChatMessage::user(row["prompt"].as_str().expect("prompt"))];
    let mut observed = Vec::new();
    for index in 0..16 {
        let plan =
            plan_chat_step(&messages, &TOOLS).expect("production planner returns a decision");
        let AgenticPlan::ToolCalls(calls) = plan else {
            let AgenticPlan::Final(final_answer) = plan else {
                unreachable!()
            };
            return Observation {
                calls: observed,
                final_answer,
            };
        };
        assert_eq!(calls.len(), 1, "bounded one-effect route");
        let call = &calls[0];
        let args: Value = serde_json::from_str(&call.arguments).expect("tool arguments");
        let path = ["path", "filePath", "file_path"]
            .iter()
            .find_map(|key| args[*key].as_str())
            .unwrap_or_default();
        let mut metadata = None;
        let output = match call.tool.as_str() {
            "read_file" => {
                if let Some(source) = files.get(path) {
                    metadata =
                        Some(json!({"path":path,"success":true,"complete":true,"format":"raw"}));
                    source.clone()
                } else {
                    metadata = Some(
                        json!({"path":path,"success":false,"complete":false,"format":"raw",
                        "error":"ENOENT","error_code":"ENOENT","exit_code":1}),
                    );
                    "ENOENT: file not found".to_owned()
                }
            }
            "web_search" => {
                assert!(
                    !args["query"]
                        .as_str()
                        .expect("query")
                        .contains(row["prompt"].as_str().unwrap())
                );
                row["url"].as_str().expect("url").to_owned()
            }
            "web_fetch" => {
                assert_eq!(args["url"], row["url"]);
                format!(
                    "<html><body><pre><code class=\"language-{}\">{}</code></pre></body></html>",
                    row["language"].as_str().unwrap(),
                    row["source"].as_str().unwrap()
                )
            }
            "write_file" => {
                assert_eq!(path, row["module"].as_str().unwrap());
                files.insert(
                    path.to_owned(),
                    args["content"].as_str().expect("source").to_owned(),
                );
                json!({"exit_code":0,"complete":true,"output":""}).to_string()
            }
            "bash" => {
                assert_eq!(args["command"], "node --test acceptance.test.mjs");
                receipt.to_owned()
            }
            other => panic!("unexpected effect: {other}"),
        };
        observed.push(call.tool.clone());
        let id = format!("synthetic-{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        let mut result = ChatMessage::tool_result(id, call.tool.clone(), output);
        result.source_read = metadata;
        messages.push(result);
    }
    panic!("production research route exceeded its finite test budget")
}

const HELD_OUT_PRODUCTION_CASES: [(usize, &str); 3] = [
    (0, "novel-js-identity"),
    (1, "novel-ts-literal"),
    (2, "unseen-zero-arg-greeting"),
];

fn assert_cold_research_and_reuse(case: (usize, &str)) {
    let fixtures = rows();
    assert_eq!(
        fixtures.len(),
        HELD_OUT_PRODUCTION_CASES.len(),
        "complete held-out schedule"
    );
    let mut indices = std::collections::BTreeSet::new();
    for (position, (index, name)) in HELD_OUT_PRODUCTION_CASES.iter().enumerate() {
        assert_eq!(*index, position, "ordered fixture index coverage");
        assert!(indices.insert(*index), "unique fixture index coverage");
        assert_eq!(
            fixtures[*index]["name"].as_str(),
            Some(*name),
            "stable fixture name"
        );
    }
    assert_eq!(
        indices.len(),
        fixtures.len(),
        "every fixture scheduled once"
    );
    assert_eq!(
        HELD_OUT_PRODUCTION_CASES[case.0], case,
        "registered fixture identity"
    );
    let row = fixtures[case.0].clone();
    let mut files = BTreeMap::from([(
        "acceptance.test.mjs".to_owned(),
        row["acceptance"].as_str().unwrap().to_owned(),
    )]);
    let receipt = json!({"exit_code":0,"complete":true,"output":TAP}).to_string();
    let first = drive(&row, &mut files, &receipt);
    assert!(first.calls.iter().any(|tool| tool == "web_search"));
    assert!(first.calls.iter().any(|tool| tool == "web_fetch"));
    assert_eq!(
        first
            .calls
            .iter()
            .filter(|tool| *tool == "write_file")
            .count(),
        1
    );
    assert!(first.calls.iter().any(|tool| tool == "bash"));
    assert_eq!(
        files.get(row["module"].as_str().unwrap()).unwrap(),
        row["expected"].as_str().unwrap()
    );
    assert!(!first.final_answer.contains("VerificationReceiptIncomplete"));
    let reuse = drive(&row, &mut files, &receipt);
    assert!(!reuse.calls.iter().any(|tool| tool == "write_file"));
    assert!(reuse.calls.iter().any(|tool| tool == "bash"));
    assert!(!reuse.final_answer.contains("VerificationReceiptIncomplete"));
}

#[test]
fn canonical_native_production_route_handles_cold_held_out_sources_and_reuses_without_writes() {
    assert_cold_research_and_reuse(HELD_OUT_PRODUCTION_CASES[0]);
}

#[test]
fn canonical_native_production_route_handles_cold_held_out_typescript_and_reuses_without_writes() {
    assert_cold_research_and_reuse(HELD_OUT_PRODUCTION_CASES[1]);
}

#[test]
fn canonical_native_production_route_handles_cold_held_out_zero_argument_and_reuses_without_writes()
{
    assert_cold_research_and_reuse(HELD_OUT_PRODUCTION_CASES[2]);
}

#[test]
fn native_research_does_not_accept_missing_or_failed_verification() {
    let row = &rows()[0];
    for receipt in [
        json!({"exit_code":0,"complete":true,"output":""}),
        json!({"exit_code":0,"complete":true,"output":"# tests 0\n# pass 0\n# fail 0\n"}),
        json!({"exit_code":1,"complete":true,"output":"# tests 1\n# pass 0\n# fail 1\n"}),
    ] {
        let mut files = BTreeMap::from([(
            "acceptance.test.mjs".to_owned(),
            row["acceptance"].as_str().unwrap().to_owned(),
        )]);
        let observed = drive(row, &mut files, &receipt.to_string());
        assert!(
            observed
                .final_answer
                .contains("VerificationReceiptIncomplete")
                || observed.final_answer.contains("NoTestsExecuted")
                || observed.final_answer.to_lowercase().contains("fail")
        );
    }
}

#[test]
fn native_catalog_and_acceptance_use_the_shared_conservative_decisions() {
    for row in rows() {
        let request = request(&row);
        assert!(independent_acceptance(
            &request,
            row["acceptance"].as_str().unwrap()
        ));
        assert!(!independent_acceptance(
            &request,
            "import assert from 'node:assert/strict'; assert.equal(1, 1);"
        ));
        let derived =
            derive_coding_procedure(&request, &page(&row, row["source"].as_str().unwrap()))
                .expect("shared pure-return fixture derives");
        assert_eq!(derived["source"], row["expected"]);
        assert_eq!(derived["provenance"]["verified"], false);
        assert!(
            derive_coding_procedure(
                &request,
                &page(
                    &row,
                    "function unlisted(seed) { console.log(seed); return seed; }"
                )
            )
            .is_none()
        );
        assert!(
            derive_coding_procedure(&request, &page(&row, "function unlisted( { return ;"))
                .is_none()
        );
        let mut unrelated = page(&row, "malformed {{{");
        unrelated["codeBlocks"][0]["language"] = json!("rust");
        assert!(derive_coding_procedure(&request, &unrelated).is_none());
        let mut ambiguous = page(&row, row["source"].as_str().unwrap());
        let duplicate = ambiguous["codeBlocks"][0].clone();
        ambiguous["codeBlocks"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(derive_coding_procedure(&request, &ambiguous).is_none());
    }
}

#[test]
fn native_generated_query_consumes_the_same_declarative_query_vectors() {
    let vectors: Vec<Value> =
        serde_json::from_str(include_str!("../../fixtures/coding-research-query.json"))
            .expect("shared query vectors");
    for vector in vectors {
        let request = ModuleFunctionRequest {
            name: vector["name"].as_str().unwrap().to_owned(),
            parameters: vector["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_owned())
                .collect(),
            at: 0,
            module: "held-out.mjs".to_owned(),
            test: None,
            language: "javascript".to_owned(),
            clause: vector["clause"].as_str().unwrap().to_owned(),
            command: None,
        };
        assert_eq!(
            formal_ai::agentic_coding::coding_research::coding_research_query(&request),
            vector["expected"].as_str().unwrap(),
            "{}",
            vector["label"]
        );
    }
}

#[test]
fn native_production_research_observes_url_standard_eligibility() {
    let vectors: Vec<Value> = serde_json::from_str(include_str!(
        "../../fixtures/coding-research/url-standard.json"
    ))
    .expect("shared URL eligibility vectors");
    for vector in vectors {
        let mut row = rows()[0].clone();
        row["url"] = vector["url"].clone();
        let mut files = BTreeMap::from([(
            "acceptance.test.mjs".to_owned(),
            row["acceptance"].as_str().expect("acceptance").to_owned(),
        )]);
        let receipt = json!({"exit_code":0,"complete":true,"output":TAP}).to_string();
        let observed = drive(&row, &mut files, &receipt);
        let permitted = vector["fetch"].as_bool().expect("expected eligibility");
        assert_eq!(
            observed.calls.iter().any(|tool| tool == "web_fetch"),
            permitted,
            "{}",
            vector["url"]
        );
        if !permitted {
            assert!(
                !observed
                    .calls
                    .iter()
                    .any(|tool| tool == "write_file" || tool == "bash")
            );
            assert!(observed.final_answer.contains("NoValidatedSourceProcedure"));
            assert!(!files.contains_key(row["module"].as_str().unwrap()));
        }
    }
}
