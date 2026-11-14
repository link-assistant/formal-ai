//! Bounded source observations before an unsupported repository task stops.
use super::final_result::{FinalDisposition, FinalResult};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use super::source_manifest_facts::observed_manifest_facts;
use super::work_item_report::{refusal, work_item_record_final};
use crate::protocol::ChatMessage;
use serde_json::json;

/// Parse bounded explicit source references without granting effect authority.
pub(super) fn work_item_evidence_paths(task: &str) -> Vec<&str> {
    if task.len() > 32_768 {
        return Vec::new();
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
    paths
}

/// Native twin of `js/agentic/work_item_evidence.mjs`. References grant read evidence only.
pub(super) fn plan_work_item_evidence(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: &str,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let paths = work_item_evidence_paths(task);
    if paths.is_empty() {
        return None;
    }
    let progress = Progress::scan(messages);
    let mut observations = Vec::new();
    // The work item is split into requirements before any evidence is read, so
    // a refusal names the requirement it concerns (issue #1560).
    let requirements = super::work_item_requirements::work_item_requirements(task)
        .into_iter()
        .map(|(text, (start, end))| json!({"text":text,"span":[start,end],"status":"unbound"}))
        .collect::<Vec<_>>();
    let mut finish = |reason: &str,
                      observations: &[serde_json::Value],
                      refused: Option<serde_json::Value>| {
        let mut payload = json!({
            "schema":"repository-work-item-evidence/v1", "reason":reason,"workItem":work_item,
            "taskIdentity":crate::source_fetch::sha256_hex(task.as_bytes()),
            "requirements":requirements,
            "decomposition":[
                {"step":"collect_repository_evidence", "status":if reason == "EvidenceCollected" {"observed"} else {"incomplete"}},
                {"step":"bind_requirements_to_source", "status":"unverified"},
                {"step":"implement_requested_change", "status":"not_executed"},
                {"step":"verify_artifact_and_requirements", "status":"not_executed"}
            ],
            "observations":observations, "coverage":"explicit-paths-only; at most eight files",
            "authored":false, "verified":false
        });
        if let Some(refused) = refused {
            payload["refused"] = refused;
        }
        work_item_record_final(
            payload,
            task,
            FinalDisposition::Gap,
            "repository_work_item_evidence_gap",
            result,
        )
    };
    for path in paths {
        let Some(observed) = progress.source_read_for(path) else {
            let Some(read) = tool_for(tools, Capability::Read) else {
                return Some(finish("MissingReadTool", &observations, None));
            };
            let step = plan_one(
                read,
                json!({"path":path,"filePath":path,"file_path":path}).to_string(),
            );
            return Some(
                if super::file_read::user_read_policy_blocks_plan(messages, task, &step) {
                    finish(
                        "ReadPolicyRefused",
                        &observations,
                        Some(refusal(&requirements, path)),
                    )
                } else {
                    step
                },
            );
        };
        let complete = observed.error.is_none()
            && observed.complete
            && observed
                .source
                .as_ref()
                .is_some_and(|source| source.len() <= 262_144);
        let source = observed.source.as_deref().unwrap_or_default();
        let mut observation = json!({"path":path,"complete":complete,
            "sourceIdentity":complete.then(||crate::source_fetch::sha256_hex(source.as_bytes())),
            "excerpt":complete.then(||source.chars().take(1024).collect::<String>())});
        if complete && (path.ends_with(".json") || path.ends_with(".toml")) {
            match observed_manifest_facts(path, source) {
                Ok(facts) => {
                    observation["manifestFacts"] = json!(facts);
                    observation["factScope"] =
                        json!("literal-declarations-only; executable use remains unverified");
                }
                Err(_) => {
                    observation["manifestFacts"] = json!([]);
                    observation["factGap"] = json!("UnsupportedOrInvalidManifest");
                }
            }
        }
        observations.push(observation);
        if !complete {
            return Some(finish("IncompleteSourceObservation", &observations, None));
        }
    }
    Some(finish("EvidenceCollected", &observations, None))
}
