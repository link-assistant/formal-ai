//! Declaration-scoped operands and observations, not synthesized code.
use crate::agentic_coding::final_result::{FinalDisposition, FinalResult, record};
use crate::agentic_coding::planner::{AgenticPlan, Capability};
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub(super) struct ObservedCallableRequest {
    name: String,
    parameters: Vec<String>,
    destination: String,
    inputs: Vec<String>,
    acceptance: Vec<String>,
    command: Option<String>,
}

fn instruction_clause(part: &str) -> String {
    let mut instruction = part.to_owned();
    for path in super::paths_in(part) {
        instruction = instruction.replace(&path, " ");
    }
    while let Some(stated) = super::signature(&instruction) {
        let end = stated.at
            + instruction[stated.at..]
                .find(')')
                .expect("signature closes")
            + 1;
        instruction.replace_range(stated.at..end, " ");
    }
    instruction
}

/// Mirrors observedCallableRequest: destination belongs to the declaration clause.
pub(super) fn observed_callable_request(task: &str) -> Option<ObservedCallableRequest> {
    let outside = super::outside_quotes(task);
    let parts = super::clauses(&outside);
    let stated = super::signature(&outside)?;
    let at = parts
        .iter()
        .position(|part| super::signature(part).is_some_and(|sig| sig.name == stated.name))?;
    let lexicon = crate::seed::lexicon();
    let authoring = |part: &str| {
        let normalized = crate::engine::normalize_prompt(&instruction_clause(part)).to_lowercase();
        lexicon.mentions_role("coding_request_verb", &normalized)
            || lexicon.mentions_role("coding_member_add_action", &normalized)
    };
    let mut declaration = parts[at].as_str();
    if !authoring(declaration) {
        let previous = parts.get(at.checked_sub(1)?)?;
        if !authoring(previous)
            || !lexicon.mentions_role(
                "coding-source-artifact-kind",
                &crate::engine::normalize_prompt(&instruction_clause(previous)).to_lowercase(),
            )
        {
            return None;
        }
        declaration = previous;
    }
    let own = super::paths_in(declaration);
    if own.len() != 1 || super::extension_language(&own[0]).is_none() {
        return None;
    }
    let command = super::stated_command(&outside);
    let acceptance = command.as_deref().map_or_else(Vec::new, super::paths_in);
    if acceptance.contains(&own[0]) {
        return None;
    }
    let inputs: Vec<String> = super::paths_in(&outside)
        .into_iter()
        .filter(|path| path != &own[0] && !acceptance.contains(path))
        .collect();
    if inputs.is_empty()
        || !lexicon.mentions_role(
            "file_read_action_cue",
            &crate::engine::normalize_prompt(&instruction_clause(&outside)).to_lowercase(),
        )
    {
        return None;
    }
    Some(ObservedCallableRequest {
        name: stated.name,
        parameters: stated.parameters,
        destination: own[0].clone(),
        inputs,
        acceptance,
        command,
    })
}

fn finish(
    request: &ObservedCallableRequest,
    observations: &[Value],
    reason: &str,
    disposition: FinalDisposition,
    detail: &Value,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let discovery = json!({"reason":reason,"request":{"name":request.name,"parameters":request.parameters,
        "destination":request.destination,"inputs":request.inputs,"acceptance":request.acceptance,"command":request.command},
        "observations":observations,"detail":detail,
        "missingContracts":request.inputs.iter().map(|path| json!({"path":path,
            "required":["inputs","result","effects","imports","optional-guard"]})).collect::<Vec<_>>(),
        "authored":false,"verified":false});
    let root = crate::seed::parser::parse_lino(include_str!(
        "../../../embedded/data/meta/agentic-messages.lino"
    ));
    let template = root
        .children
        .first()
        .and_then(|root| {
            root.children
                .iter()
                .find(|node| node.name == "message" && node.id == "callable-discovery-outcome")
        })
        .expect("callable discovery message is installed")
        .find_child_value("text");
    let answer = template
        .replace("\\n", "\n")
        .replace(concat!("{", "reason", "}"), reason)
        .replace(concat!("{", "discovery", "}"), &discovery.to_string());
    record(
        AgenticPlan::Final(answer),
        disposition,
        "observed-callable-discovery",
        result,
    )
}

/// Mirrors planObservedCallableStep: read all operands before unsupported-contract failure.
pub(super) fn plan_observed_callable_step(
    request: &ObservedCallableRequest,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    use crate::agentic_coding::{
        capability_router, code_artifact, planner, tool_result, workspace_change,
    };
    let current = &messages[planner::evidence_window_start(messages)..];
    let mut observations = Vec::new();
    let operands = std::iter::once((&request.destination, "destination"))
        .chain(request.inputs.iter().map(|path| (path, "source")))
        .chain(request.acceptance.iter().map(|path| (path, "acceptance")));
    for (path, role) in operands {
        let Some((receipt, explicitly_failed)) = read_receipt(current, path) else {
            return capability_router::tool_for(tool_names, Capability::Read).map_or_else(
                || {
                    finish(
                        request,
                        &observations,
                        "MissingReadTool",
                        FinalDisposition::Gap,
                        &json!({"path":path,"role":role}),
                        result,
                    )
                },
                |tool| planner::plan_one(tool, workspace_change::read_arguments(path)),
            );
        };
        let agent_source = code_artifact::source_from_agent_read_result(&receipt);
        let failure = if explicitly_failed || agent_source.is_none() {
            tool_result::failure_message(&receipt, explicitly_failed, false)
        } else {
            None
        };
        if let Some(error) = failure {
            if role == "destination"
                && error
                    .split(|c: char| !c.is_alphanumeric())
                    .any(|word| word == "ENOENT")
            {
                observations.push(
                    json!({"path":path,"role":role,"state":"absent","contentId":null,"bytes":null}),
                );
                continue;
            }
            return finish(
                request,
                &observations,
                "ReadFailed",
                FinalDisposition::Failure,
                &json!({"path":path,"role":role,"error":error}),
                result,
            );
        }
        let content =
            agent_source.unwrap_or_else(|| code_artifact::source_from_read_result(&receipt));
        observations.push(json!({"path":path,"role":role,"state":"observed",
            "contentId":crate::source_fetch::sha256_hex(content.as_bytes()),"bytes":content.len()}));
    }
    finish(
        request,
        &observations,
        "MissingContract",
        FinalDisposition::Gap,
        &Value::Null,
        result,
    )
}

fn read_receipt(messages: &[ChatMessage], path: &str) -> Option<(String, bool)> {
    use crate::agentic_coding::capability_router::classify_tool;
    for (index, message) in messages.iter().enumerate().rev() {
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let Some(id) = message.tool_call_id.as_deref() else {
            continue;
        };
        let call = messages[..index]
            .iter()
            .rev()
            .flat_map(|prior| prior.tool_calls.iter().rev())
            .find(|call| call.id == id);
        let Some(call) = call else {
            continue;
        };
        if classify_tool(&call.function.name) != Some(Capability::Read) {
            continue;
        }
        let Ok(args) = serde_json::from_str::<Value>(&call.function.arguments) else {
            continue;
        };
        if ["path", "filePath", "file_path"]
            .iter()
            .any(|key| args.get(key).and_then(Value::as_str) == Some(path))
        {
            return Some((message.content.plain_text(), message.is_error));
        }
    }
    None
}

#[cfg(test)]
mod tests {
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
            observed_callable_request("Read existing f(x) from create.mjs. Read second.mjs.")
                .is_none()
        );
        assert!(
            observed_callable_request("Add f(x) to a.mjs and b.mjs. Read input.mjs.").is_none()
        );
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
            data.push(
                json!({"role":"tool","tool_call_id":path,"content":content,"is_error":failed}),
            );
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
        let plan =
            plan_observed_callable_step(&request, &messages, &["read", "write"], &mut metadata);
        let AgenticPlan::Final(answer) = plan else {
            panic!("must report observed gap")
        };
        assert!(answer.starts_with("Callable discovery MissingContract:\n"));
        let discovery: Value = serde_json::from_str(answer.split_once('\n').expect("gap record").1)
            .expect("typed record");
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
        let plan =
            plan_observed_callable_step(&request, &messages, &["read", "write"], &mut metadata);
        let AgenticPlan::Final(answer) = plan else {
            panic!("must report read failure")
        };
        assert!(answer.starts_with("Callable discovery ReadFailed:\n"));
        assert_eq!(
            metadata.expect("failure result").disposition,
            FinalDisposition::Failure
        );
    }
}
