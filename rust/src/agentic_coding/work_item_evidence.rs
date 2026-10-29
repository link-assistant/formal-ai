//! Bounded source observations before an unsupported repository task stops.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use crate::protocol::ChatMessage;
use serde_json::json;

/// Native twin of js/agentic/work_item_evidence.mjs. References grant read evidence only.
pub(super) fn plan_work_item_evidence(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    if task.len() > 32_768 {
        return None;
    }
    let pattern = regex::Regex::new(
        r"^[A-Za-z0-9_-][A-Za-z0-9_./-]*\.(md|toml|json|rs|mjs|js|ts|yml|yaml|lino)$",
    )
    .expect("StaticWorkItemPathPattern");
    let mut paths = Vec::new();
    for word in
        task.split(|character: char| character.is_whitespace() || "`\'\"()[],;".contains(character))
    {
        let path = word
            .trim_start_matches(['`', '\'', '"', '(', '['])
            .trim_end_matches(['`', '\'', '"', ')', ',', '.', ';', ':', ']']);
        if !pattern.is_match(path)
            || path.split('/').any(|part| matches!(part, "" | "." | ".."))
            || paths.contains(&path)
        {
            continue;
        }
        paths.push(path);
        if paths.len() == 8 {
            break;
        }
    }
    if paths.is_empty() {
        return None;
    }
    let progress = Progress::scan(messages);
    let mut observations = Vec::new();
    let mut finish = |reason: &str, observations: &[serde_json::Value]| {
        let payload = json!({
            "schema":"repository-work-item-evidence/v1", "reason":reason,
            "taskIdentity":crate::source_fetch::sha256_hex(task.as_bytes()),
            "requirements":crate::obligation_ledger::clauses_with_spans(task).into_iter()
                .map(|(text,(start,end))|json!({"text":text,"span":[start,end],"status":"unbound"})).collect::<Vec<_>>(),
            "decomposition":[
                {"step":"collect_repository_evidence", "status":if reason == "EvidenceCollected" {"observed"} else {"incomplete"}},
                {"step":"bind_requirements_to_source", "status":"unverified"},
                {"step":"implement_requested_change", "status":"not_executed"},
                {"step":"verify_artifact_and_requirements", "status":"not_executed"}
            ],
            "observations":observations, "coverage":"explicit-paths-only; at most eight files",
            "authored":false, "verified":false
        });
        record(
            AgenticPlan::Final(payload.to_string()),
            FinalDisposition::Gap,
            "repository_work_item_evidence_gap",
            result,
        )
    };
    for path in paths {
        let Some(observed) = progress.source_read_for(path) else {
            let Some(read) = tool_for(tools, Capability::Read) else {
                return Some(finish("MissingReadTool", &observations));
            };
            let step = plan_one(
                read,
                json!({"path":path,"filePath":path,"file_path":path}).to_string(),
            );
            return Some(if super::file_read::read_policy_blocks_plan(task, &step) {
                finish("ReadPolicyRefused", &observations)
            } else {
                step
            });
        };
        let complete = observed.error.is_none()
            && observed.complete
            && observed
                .source
                .as_ref()
                .is_some_and(|source| source.len() <= 262_144);
        let source = observed.source.as_deref().unwrap_or_default();
        observations.push(json!({"path":path,"complete":complete,
            "sourceIdentity":complete.then(||crate::source_fetch::sha256_hex(source.as_bytes())),
            "excerpt":complete.then(||source.chars().take(1024).collect::<String>())}));
        if !complete {
            return Some(finish("IncompleteSourceObservation", &observations));
        }
    }
    Some(finish("EvidenceCollected", &observations))
}

#[cfg(test)]
mod work_item_evidence_tests {
    use super::super::tool_result;
    use super::*;
    use crate::protocol::ToolCall;

    #[test]
    fn original_documentation_issue_collects_evidence_without_claiming_implementation() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../embedded/data/benchmarks/fixtures/work-item-1455/failure.json"
        ))
        .expect("public failure fixture");
        let task = fixture["objective"].as_str().expect("objective");
        let paths = [
            "docs/associative-tech-stack.md",
            "rust/Cargo.toml",
            "package.json",
            "js/web-search-component-entry.js",
        ];
        let mut messages = vec![ChatMessage::user(task)];
        let mut result = None;
        for path in paths {
            let Some(AgenticPlan::ToolCalls(calls)) =
                plan_work_item_evidence(task, &messages, &["read", "write"], &mut result)
            else {
                panic!("evidence must be read before a terminal answer")
            };
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].tool, "read");
            let arguments: serde_json::Value =
                serde_json::from_str(&calls[0].arguments).expect("read operands");
            assert_eq!(arguments["path"], path);
            let tool_call_identity = format!("evidence-{}", messages.len());
            messages.push(ChatMessage::assistant_tool_calls_with_content(
                String::new(),
                vec![ToolCall::function(
                    tool_call_identity.clone(),
                    "read".to_owned(),
                    calls[0].arguments.clone(),
                )],
            ));
            messages.push(ChatMessage::tool_result(
                tool_call_identity,
                "read",
                tool_result::complete_owned_source_read_frame("observed source\n"),
            ));
        }
        let Some(AgenticPlan::Final(answer)) =
            plan_work_item_evidence(task, &messages, &["read", "write"], &mut result)
        else {
            panic!("bounded evidence collection must terminate")
        };
        let evidence: serde_json::Value =
            serde_json::from_str(&answer).expect("formal observation");
        assert_eq!(evidence["reason"], "EvidenceCollected");
        assert_eq!(evidence["authored"], false);
        assert_eq!(evidence["verified"], false);
        assert_eq!(
            result.expect("qualified gap").disposition,
            FinalDisposition::Gap
        );
    }
}
