use formal_ai::agentic_coding::module_function::{
    ObservedCallableDisposition as FinalDisposition, observed_callable_request,
    plan_observed_callable_outcome,
};
use formal_ai::agentic_coding::planner::AgenticPlan;
use formal_ai::protocol::ChatMessage;
use serde_json::{Value, json};
const PROMPT: &str = "Add exported caption(pick) to out.mjs. Read first.mjs and second.mjs. Run the supplied gate.mjs command, node --test gate.mjs, and report results. Incidental numbers 188 and 558.";

#[test]
fn operands_are_bound_to_the_declared_destination_and_acceptance() {
    let request = observed_callable_request(PROMPT).expect("declared callable");
    assert_eq!(request.destination, "out.mjs");
    assert_eq!(request.name, "caption");
    assert_eq!(request.parameters, ["pick"]);
    assert_eq!(request.inputs, ["first.mjs", "second.mjs"]);
    assert_eq!(request.acceptance, ["gate.mjs"]);
    assert_eq!(request.command.as_deref(), Some("node --test gate.mjs"));
    assert!(
        observed_callable_request("Read existing f(write) from first.mjs. Read second.mjs.")
            .is_none()
    );
    assert!(
        observed_callable_request("Read existing f(x) from create.mjs. Read second.mjs.").is_none()
    );
    assert!(observed_callable_request("Add f(x) to a.mjs and b.mjs. Read input.mjs.").is_none());
    assert!(
        observed_callable_request("Implement module a.mjs. Read input.mjs with label «f(x)».")
            .is_none()
    );
}

fn messages(receipts: &[(&str, &str, bool)]) -> Vec<ChatMessage> {
    let mut data = vec![json!({"role":"user","content":PROMPT})];
    for (path, content, failed) in receipts {
        data.push(
            json!({"role":"assistant","content":"","tool_calls":[{"id":path,"type":"function",
                "function":{"name":"read","arguments":json!({"path":path}).to_string()}}]}),
        );
        data.push(json!({"role":"tool","tool_call_id":path,"content":content,"is_error":failed}));
    }
    serde_json::from_value(json!(data)).expect("valid transcript")
}

#[test]
fn observation_gap_keeps_source_identities_and_cannot_deliver_source() {
    let request = observed_callable_request(PROMPT).expect("declared callable");
    let messages = messages(&[
        ("out.mjs", "// existing\n", false),
        (
            "first.mjs",
            "export function first(x) { return x; }\n",
            false,
        ),
        (
            "second.mjs",
            "export function second(x) { return x; }\n",
            false,
        ),
        ("gate.mjs", "// immutable acceptance\n", false),
    ]);
    let outcome = plan_observed_callable_outcome(&request, &messages, &["read", "write"]);
    let plan = outcome.plan;
    let metadata = outcome.disposition;
    let AgenticPlan::Final(answer) = plan else {
        panic!("must report observed gap")
    };
    assert!(answer.starts_with("Callable discovery MissingContract:\n"));
    let discovery: Value =
        serde_json::from_str(answer.split_once('\n').expect("gap record").1).expect("typed record");
    assert_eq!(outcome.witness.as_ref(), Some(&discovery));
    assert_eq!(discovery["authored"], false);
    assert_eq!(discovery["verified"], false);
    assert_eq!(
        discovery["observations"][1]["contentId"],
        formal_ai::source_fetch::sha256_hex(b"export function first(x) { return x; }\n")
    );
    assert_eq!(
        metadata.expect("request-local result"),
        FinalDisposition::Gap
    );
}

#[test]
fn explicit_client_read_failure_never_becomes_an_empty_source() {
    let request = observed_callable_request(PROMPT).expect("declared callable");
    let messages = messages(&[("out.mjs", "not permitted", true)]);
    let outcome = plan_observed_callable_outcome(&request, &messages, &["read", "write"]);
    let plan = outcome.plan;
    let metadata = outcome.disposition;
    let AgenticPlan::Final(answer) = plan else {
        panic!("must report read failure")
    };
    assert!(answer.starts_with("Callable discovery ReadFailed:\n"));
    assert_eq!(metadata.expect("failure result"), FinalDisposition::Failure);
}

#[test]
fn complete_json_source_keys_do_not_own_discovery_transport_status() {
    let request = observed_callable_request(PROMPT).expect("declared callable");
    let source = json!({"error":"failed","is_error":true,"exit_code":9,"signal":"SIGTERM","content":"authored bytes"}).to_string();
    let mut messages = messages(&[
        ("out.mjs", "// destination", false),
        ("first.mjs", source.as_str(), false),
        ("second.mjs", "// source", false),
        ("gate.mjs", "// acceptance", false),
    ]);
    for message in &mut messages {
        if message.role == "tool" {
            message.source_read = Some(
                json!({"path":message.tool_call_id,"success":true,"complete":true,"format":"raw"}),
            );
        }
    }
    let outcome = plan_observed_callable_outcome(&request, &messages, &["read", "write"]);
    assert_eq!(outcome.disposition, Some(FinalDisposition::Gap));
    let witness = outcome.witness.expect("typed gap");
    assert_eq!(witness["reason"], "MissingContract");
    assert_eq!(
        witness["observations"][1]["contentId"],
        formal_ai::source_fetch::sha256_hex(source.as_bytes())
    );
    assert_eq!(witness["observations"][1]["complete"], true);
    assert_eq!(
        witness["observations"][1]["providerStatus"],
        "reported-success"
    );
    messages[4].source_read.as_mut().unwrap()["complete"] = json!(false);
    let partial = plan_observed_callable_outcome(&request, &messages, &["read"]);
    let witness = partial.witness.expect("partial gap");
    assert_eq!(partial.disposition, Some(FinalDisposition::Gap));
    assert_eq!(witness["observations"][1]["complete"], false);
    assert!(witness["observations"][1].get("catalog").is_none());
    assert_eq!(witness["detail"]["graphs"], json!([]));
}

const COMPACT_SOURCE_TASK: &str = "Implement exported renderChosen(pick) to nested/caption.mjs: return the compact summary of the selected candidate or null. Discover and compose observed left.mjs and right.mjs exports through imports using aliases chooseRecord as chosen, describeRecord as describe; preserve inputs and reuse semantics. Run node --test gate.test.mjs; keep source modules and tests unchanged.";
const COMPACT_SOURCE_SEED: &str = include_str!("../../../data/seed/source-authoring-grammar.lino");

#[test]
fn compact_source_frame_preserves_real_goal_spans_and_inferred_operation_needs() {
    use formal_ai::agentic_coding::module_function::{
        compact_source_request, derive_complete_source_request,
    };
    let request = compact_source_request(COMPACT_SOURCE_TASK, COMPACT_SOURCE_SEED)
        .expect("closed compact request");
    assert_eq!(request["destination"], "nested/caption.mjs");
    assert_eq!(request["inputs"], json!(["left.mjs", "right.mjs"]));
    assert_eq!(
        request["aliases"],
        json!([{"exported":"chooseRecord","local":"chosen"},{"exported":"describeRecord","local":"describe"}])
    );
    let observed =
        observed_callable_request(COMPACT_SOURCE_TASK).expect("compact discovery operands");
    assert_eq!(
        observed.command.as_deref(),
        Some("node --test gate.test.mjs")
    );
    let frame = derive_complete_source_request(COMPACT_SOURCE_TASK, COMPACT_SOURCE_SEED)
        .expect("complete compact frame");
    let goal = frame["clauses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["kind"] == "goal")
        .unwrap();
    assert_eq!(
        goal["sentence"],
        "return the compact summary of the selected candidate or null"
    );
    assert_eq!(goal["evidence"]["source"], goal["sentence"]);
    assert_eq!(
        frame["inferredOperationPrerequisites"][0]["authority"],
        "candidate-operation-invariant"
    );
    assert_eq!(frame["writeAuthority"], false);
    assert_eq!(frame["semanticAuthority"], false);
}

#[test]
fn compact_source_ownership_refuses_unknown_effects_conflicts_and_alias_ambiguity() {
    use formal_ai::agentic_coding::module_function::compact_source_request;
    let invalid = [
        format!("{COMPACT_SOURCE_TASK} Delete another.mjs."),
        COMPACT_SOURCE_TASK.replace(
            "preserve inputs and reuse semantics",
            "preserve inputs and reuse semantics and deploy the release",
        ),
        COMPACT_SOURCE_TASK.replace("; keep source modules and tests unchanged", ""),
        format!("{COMPACT_SOURCE_TASK} Run node --test other.mjs."),
        COMPACT_SOURCE_TASK.replace("left.mjs and right.mjs", "left.mjs and left.mjs"),
        COMPACT_SOURCE_TASK.replace("left.mjs and right.mjs", "nested/caption.mjs and right.mjs"),
        COMPACT_SOURCE_TASK.replace("left.mjs and right.mjs", "gate.test.mjs and right.mjs"),
        COMPACT_SOURCE_TASK.replace("left.mjs and right.mjs", "../left.mjs and right.mjs"),
        COMPACT_SOURCE_TASK.replace("nested/caption.mjs", "nested/caption.rs"),
        COMPACT_SOURCE_TASK.replace("describeRecord as describe", "chooseRecord as describe"),
        COMPACT_SOURCE_TASK.replace("describeRecord as describe", "describeRecord as chosen"),
        COMPACT_SOURCE_TASK.replace("chooseRecord as chosen", "chooseRecord as chosen()"),
        COMPACT_SOURCE_TASK.replace(
            "preserve inputs and reuse semantics",
            "\"preserve inputs and reuse semantics\"",
        ),
    ];
    for task in invalid {
        assert!(
            compact_source_request(&task, COMPACT_SOURCE_SEED).is_none(),
            "{task}"
        );
    }
}

#[test]
fn original_source_request_keeps_all_nine_explicit_clauses() {
    use formal_ai::agentic_coding::module_function::derive_complete_source_request;
    let archived: Value = serde_json::from_str(include_str!("../../../experiments/formal_ai_subagent/evidence/formal-ai-only-1188/g132/T1804-request.json")).unwrap();
    let original = archived["prompt"].as_str().unwrap();
    assert_eq!(original.len(), 955);
    let frame = derive_complete_source_request(original, COMPACT_SOURCE_SEED)
        .expect("original source frame");
    assert_eq!(frame["clauses"].as_array().unwrap().len(), 9);
    assert!(frame.get("inferredOperationPrerequisites").is_none());
}
