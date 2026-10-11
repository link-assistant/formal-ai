//! Conditional request-derived prerequisites, without semantic write authority.
use super::discovery::{observed_callable_goal_ledger, observed_callable_request};
use crate::agentic_coding::tool_result::SourceReadStatus;
use crate::agentic_coding::{file_read, planner, progress::Progress, workspace_change};
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

/// Classify only Read kinds declared by the maintained goal ledger producer.
fn source_need_is_read(need: &Value) -> bool {
    matches!(
        need["kind"].as_str(),
        Some("read-destination" | "read-source")
    )
}

/// Mirrors sourceNeedPreflight; actual host message ownership remains required.
pub fn source_need_preflight(source: &str, messages: &[ChatMessage]) -> Option<Value> {
    if crate::normal_markov::quote_fault(source).is_some() {
        return None;
    }
    let user = messages
        .iter()
        .rev()
        .find(|message| message.role == "user")?;
    if user.content.user_request_text() != source {
        return None;
    }
    let request = observed_callable_request(source)?;
    let ledger = observed_callable_goal_ledger(&request, messages);
    let progress = Progress::scan(messages);
    let operands = std::iter::once((&request.destination, "destination"))
        .chain(request.inputs.iter().map(|path| (path, "source")))
        .chain(request.acceptance.iter().map(|path| (path, "acceptance")));
    let prerequisites = operands.map(|(path, role)| {
        let policy = file_read::read_policy_blocks_plan(source,
            &planner::plan_one("read", workspace_change::read_arguments(path)));
        let read = progress.source_read_for(path);
        let status = if policy { "policy-refused" } else {
            read.map_or("unattempted", |observation| {
                if let Some(error) = &observation.error {
                    if role == "destination" && error.split(|character: char|
                        !character.is_ascii_alphanumeric() && character != '_')
                        .any(|word| word == "ENOENT") {
                        "conditional-satisfied-absence"
                    } else { "failed" }
                } else if observation.complete && observation.source.is_some() {
                    "conditional-satisfied-read"
                } else { "incomplete" }
            })
        };
        let identity = read.filter(|value| value.error.is_none() && value.complete)
            .and_then(|value| value.source.as_ref())
            .map(|value| crate::source_fetch::sha256_hex(value.as_bytes()));
        let provider_status = read.map(|value| match value.status {
            SourceReadStatus::ReportedSuccess => "reported-success",
            SourceReadStatus::ReportedFailure => "reported-failure",
            SourceReadStatus::Unknown => "unknown",
        });
        json!({"path":path,"role":role,"status":status,"sourceIdentity":identity,
            "providerStatus":provider_status,"providerError":read.and_then(|value|value.error.as_ref()),
            "requiredBy":"maintained-observed-callable-request",
            "requestIdentity":crate::source_fetch::sha256_hex(source.as_bytes())})
    }).collect::<Vec<_>>();
    let unresolved = ledger["needs"]
        .as_array()?
        .iter()
        .filter(|need| !source_need_is_read(need))
        .cloned()
        .collect::<Vec<_>>();
    Some(
        json!({"request":{"name":request.name,"parameters":request.parameters,
        "destination":request.destination,"inputs":request.inputs,"acceptance":request.acceptance,
        "command":request.command},"ledger":ledger,"prerequisites":prerequisites,
        "unresolvedNeeds":unresolved,"independentClauses":ledger["clauses"],
        "semanticCoverage":"unbound","declarationAuthority":"conditional-request-classifier",
        "receiptTrust":"requires actual maintained host message boundary","moduleEffects":"unknown",
        "writeAuthority":false,"authored":false,"verified":false}),
    )
}
