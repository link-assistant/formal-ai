//! Bounded source observations before an unsupported repository task stops.
//! Native twin of `js/agentic/work_item_evidence.mjs`.
//!
//! Each file name the work item mentions is resolved through
//! `work_item_references` (issue #1558): the exact path first, then a search
//! by file name, then ranked candidates. A failed read is recorded and the
//! next action differs from it; a reference nothing resolves becomes an
//! explicit unresolved item while the remaining evidence is still read.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::{Progress, ToolAttempt};
use super::source_manifest_facts::observed_manifest_facts;
use super::work_item_references::{
    MAXIMUM_CANDIDATE_READS, ReadOutcome, WorkItemReference, attempt_for, product_name,
    ranked_candidates, read_attempt_for, read_outcome, same_path, search_actions, search_matches,
    work_item_references, workspace_roots,
};
use crate::protocol::{ChatMessage, client_working_directory};
use serde_json::json;

/// Parse bounded explicit source references without granting effect authority.
pub(super) fn work_item_evidence_paths(task: &str) -> Vec<&str> {
    super::work_item_references::work_item_evidence_paths(task, false)
}

/// The host root a relative read falls back to when the client names no workspace.
fn host_root() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|root| root.into_os_string().into_string().ok())
}

/// What the collector does next: plan one action, or finish with a reason.
enum Step {
    Plan(AgenticPlan),
    Finish(&'static str),
}

/// The evidence gathered so far, reported in the final payload.
#[derive(Default)]
struct Evidence {
    observations: Vec<serde_json::Value>,
    unresolved: Vec<serde_json::Value>,
    terms: Vec<String>,
}

impl Evidence {
    fn leave_unresolved(
        &mut self,
        reference: &WorkItemReference,
        reason: &str,
        extra: Option<(&str, serde_json::Value)>,
    ) {
        let mut row = json!({"reference":reference.name,"target":reference.target,"reason":reason});
        if let Some((key, value)) = extra {
            row[key] = value;
        }
        self.unresolved.push(row);
    }

    /// Record the read of `path` for `reference` and return its outcome.
    fn observe(
        &mut self,
        reference: &WorkItemReference,
        path: &str,
        attempt: &ToolAttempt,
        resolved_by: &str,
    ) -> ReadOutcome {
        let outcome = read_outcome(attempt);
        let failed = outcome == ReadOutcome::Failed;
        let source = attempt
            .source_observation()
            .filter(|observed| !failed && observed.complete)
            .and_then(|observed| observed.source.as_deref())
            .filter(|source| source.len() <= 262_144);
        let complete = source.is_some();
        let mut observation = json!({"path":path,"complete":complete,
            "sourceIdentity":source.map(|source|crate::source_fetch::sha256_hex(source.as_bytes())),
            "excerpt":source.map(|source|source.chars().take(1024).collect::<String>())});
        if reference.name != path {
            observation["reference"] = json!(reference.name);
        }
        if resolved_by != "exact" {
            observation["resolvedBy"] = json!(resolved_by);
        }
        if failed {
            observation["readFailed"] = json!(true);
        }
        if let Some(source) = source
            && (path.ends_with(".toml") || path.ends_with(".json"))
        {
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
        self.observations.push(observation);
        if failed {
            ReadOutcome::Failed
        } else if complete {
            ReadOutcome::Complete
        } else {
            ReadOutcome::Incomplete
        }
    }
}

fn read_step(read: &str, path: &str) -> AgenticPlan {
    plan_one(
        read,
        json!({"path":path,"filePath":path,"file_path":path}).to_string(),
    )
}

fn plan_read(task: &str, read: &str, path: &str) -> Step {
    let step = read_step(read, path);
    if super::file_read::read_policy_blocks_plan(task, &step) {
        Step::Finish("ReadPolicyRefused")
    } else {
        Step::Plan(step)
    }
}

/// Walk every reference in order and return the first action still owed, or the terminal reason.
fn next_step(
    task: &str,
    references: &[WorkItemReference],
    progress: &Progress,
    roots: &[String],
    tools: &[&str],
    evidence: &mut Evidence,
) -> Step {
    let read = tool_for(tools, Capability::Read);
    let mut incomplete = false;
    for reference in references {
        let Some(target) = reference.path.as_deref().filter(|_| !reference.external) else {
            evidence.leave_unresolved(
                reference,
                "OutsideRepository",
                Some(("repository", json!(reference.repository))),
            );
            continue;
        };
        let Some(read) = read else {
            return Step::Finish("MissingReadTool");
        };
        let searches = search_actions(reference, tools);
        let product = product_name(target);
        // The exact path is read first unless the name is bare and a name search can find it without a failed read.
        let exact_first = !product
            && (target.contains('/')
                || !searches
                    .iter()
                    .any(|action| action.capability == Capability::Glob));
        let mut failed: Vec<&str> = Vec::new();
        if exact_first {
            let Some(attempt) = read_attempt_for(progress, target, roots) else {
                return plan_read(task, read, target);
            };
            match evidence.observe(reference, target, attempt, "exact") {
                ReadOutcome::Failed => failed.push(target),
                outcome => {
                    incomplete |= outcome == ReadOutcome::Incomplete;
                    continue;
                }
            }
        } else if super::file_read::read_policy_blocks_plan(task, &read_step(read, target)) {
            return Step::Finish("ReadPolicyRefused");
        }
        let mut matches = None;
        for action in &searches {
            let Some(attempt) = attempt_for(progress, action) else {
                return Step::Plan(plan_one(action.tool, action.arguments.clone()));
            };
            let found = search_matches(&attempt.detail, reference, roots);
            if !found.is_empty() || attempt.succeeded {
                matches = Some(found);
                break;
            }
        }
        let Some(matches) = matches else {
            if product {
                evidence.terms.push(reference.name.clone());
            } else {
                let reason = if searches.is_empty() {
                    "SearchUnavailable"
                } else {
                    "SearchFailed"
                };
                evidence.leave_unresolved(reference, reason, None);
            }
            continue;
        };
        let readable = matches
            .iter()
            .filter(|path| {
                !failed
                    .iter()
                    .any(|prior| path.as_str() == *prior || same_path(path, prior, roots))
            })
            .cloned()
            .collect::<Vec<_>>();
        let candidates = ranked_candidates(&readable, reference);
        if candidates.is_empty() {
            if product && matches.is_empty() {
                evidence.terms.push(reference.name.clone());
            } else {
                let reason = if matches.is_empty() {
                    "NotFound"
                } else {
                    "CandidatesUnreadable"
                };
                evidence.leave_unresolved(reference, reason, None);
            }
            continue;
        }
        let resolved_by = if candidates.len() == 1 {
            "unique-name-match"
        } else {
            "ranked-name-match"
        };
        let tried = &candidates[..candidates.len().min(MAXIMUM_CANDIDATE_READS)];
        let mut resolved = false;
        for candidate in tried {
            let Some(attempt) = read_attempt_for(progress, candidate, roots) else {
                return plan_read(task, read, candidate);
            };
            let outcome = evidence.observe(reference, candidate, attempt, resolved_by);
            if outcome != ReadOutcome::Failed {
                incomplete |= outcome == ReadOutcome::Incomplete;
                resolved = true;
                break;
            }
        }
        if !resolved {
            evidence.leave_unresolved(
                reference,
                "CandidatesUnreadable",
                Some(("candidates", json!(tried))),
            );
        }
    }
    if !evidence.unresolved.is_empty() {
        Step::Finish("UnresolvedReferences")
    } else if incomplete {
        Step::Finish("IncompleteSourceObservation")
    } else {
        Step::Finish("EvidenceCollected")
    }
}

/// Native twin of `js/agentic/work_item_evidence.mjs`. References grant read evidence only.
pub(super) fn plan_work_item_evidence(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: &str,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    if work_item_evidence_paths(task).is_empty() {
        return None;
    }
    let references = work_item_references(task, work_item);
    let progress = Progress::scan(messages);
    let roots = workspace_roots(client_working_directory(messages), host_root());
    let mut evidence = Evidence::default();
    let reason = match next_step(task, &references, &progress, &roots, tools, &mut evidence) {
        Step::Plan(plan) => return Some(plan),
        Step::Finish(reason) => reason,
    };
    let payload = json!({
        "schema":"repository-work-item-evidence/v1", "reason":reason,"workItem":work_item,
        "taskIdentity":crate::source_fetch::sha256_hex(task.as_bytes()),
        "requirements":crate::obligation_ledger::clauses_with_spans(task).into_iter()
            .map(|(text,(start,end))|json!({"text":text,"span":[start,end],"status":"unbound"})).collect::<Vec<_>>(),
        "decomposition":[
            {"step":"collect_repository_evidence", "status":if reason == "EvidenceCollected" {"observed"} else {"incomplete"}},
            {"step":"bind_requirements_to_source", "status":"unverified"},
            {"step":"implement_requested_change", "status":"not_executed"},
            {"step":"verify_artifact_and_requirements", "status":"not_executed"}
        ],
        "observations":evidence.observations, "unresolved":evidence.unresolved, "terms":evidence.terms,
        "coverage":"explicit-references; exact path, then file-name search; at most eight files",
        "authored":false, "verified":false
    });
    Some(record(
        AgenticPlan::Final(payload.to_string()),
        FinalDisposition::Gap,
        "repository_work_item_evidence_gap",
        result,
    ))
}
