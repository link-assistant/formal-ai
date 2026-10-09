use super::*;
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
    let mut metadata = None;
    let plan = plan_observed_callable_step(&request, &messages, &["read", "write"], &mut metadata);
    let AgenticPlan::Final(answer) = plan else {
        panic!("must report observed gap")
    };
    assert!(answer.starts_with("Callable discovery MissingContract:\n"));
    let discovery: Value =
        serde_json::from_str(answer.split_once('\n').expect("gap record").1).expect("typed record");
    assert_eq!(discovery["authored"], false);
    assert_eq!(discovery["verified"], false);
    assert_eq!(
        discovery["observations"][1]["contentId"],
        crate::source_fetch::sha256_hex(b"export function first(x) { return x; }\n")
    );
    assert_eq!(
        metadata.expect("request-local result").disposition,
        FinalDisposition::Gap
    );
}

#[test]
fn explicit_client_read_failure_never_becomes_an_empty_source() {
    let request = observed_callable_request(PROMPT).expect("declared callable");
    let messages = messages(&[("out.mjs", "not permitted", true)]);
    let mut metadata = None;
    let plan = plan_observed_callable_step(&request, &messages, &["read", "write"], &mut metadata);
    let AgenticPlan::Final(answer) = plan else {
        panic!("must report read failure")
    };
    assert!(answer.starts_with("Callable discovery ReadFailed:\n"));
    assert_eq!(
        metadata.expect("failure result").disposition,
        FinalDisposition::Failure
    );
}
