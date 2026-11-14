//! Native counterpart of work_item_boundary_candidate.mjs. The controller owns
//! the whole task ledger; this module handles one seeded structural leaf only.
use super::comparison_boundary::observed_comparison_boundary_operation;
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability};
use super::shell_command_policy::sentences;
use super::tool_result::{SourceReadObservation, SourceReadStatus};
use super::work_item_file_io::{
    FileIoPlan, WorkItemFileProgress as Progress, file_io_effect_step, file_io_pending_step,
    file_io_read_step,
};
use crate::protocol::ChatMessage;
use crate::seed::parser::parse_lino;
use serde_json::{Value, json};

const GRAMMAR: &str = include_str!("../../embedded/data/meta/comparison-boundary-grammar.lino");
fn words(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|character| {
            if "`'\"()[],;:".contains(character) {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .split(crate::meta_reasoner::text::is_js_whitespace)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
fn owner(path: &str) -> bool {
    path.chars().next().is_some_and(|character| {
        character.is_ascii_alphanumeric() || character == '_' || character == '-'
    }) && path
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "_./-".contains(character))
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}
fn complete(observation: Option<&SourceReadObservation>) -> Option<&str> {
    observation
        .filter(|value| value.error.is_none() && value.complete && !value.absent)
        .and_then(|value| value.source.as_deref())
}
fn digest(text: &str) -> String {
    crate::source_fetch::sha256_hex(text.as_bytes())
}

/// Path discovery is passed in from the owning work-item module. This avoids
/// circular module ownership and keeps a single repository path parser.
pub(super) fn boundary_work_item_contract(
    task: &str,
    paths_for_clause: impl Fn(&str) -> Vec<String>,
) -> Option<Value> {
    if task.len() > 32768 {
        return None;
    }
    let tree = parse_lino(GRAMMAR);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "comparison-boundary-grammar")?;
    let values = |key: &str| {
        root.children
            .iter()
            .filter(|node| node.name == key)
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    };
    let mentions = |text: &str, key: &str| {
        values(key)
            .iter()
            .any(|value| format!(" {text} ").contains(&format!(" {} ", words(value))))
    };
    let mut owned = Vec::new();
    for clause in sentences(task) {
        let text = words(clause.text);
        let regression = mentions(&text, "regression-action");
        let repair = mentions(&text, "repair-action");
        if (!regression && !repair)
            || mentions(&text, "negated-property")
            || mentions(&text, "negated-action")
        {
            continue;
        }
        let upper = mentions(&text, "upper-bound");
        let lower = mentions(&text, "lower-bound");
        let inclusive = mentions(&text, "inclusive");
        let exclusive = mentions(&text, "exclusive");
        if upper == lower || inclusive == exclusive {
            continue;
        }
        let references = paths_for_clause(clause.text)
            .into_iter()
            .filter(|path| owner(path))
            .collect::<Vec<_>>();
        let tests = references
            .iter()
            .filter(|path| {
                values("test-suffix")
                    .iter()
                    .any(|suffix| path.ends_with(suffix))
            })
            .collect::<Vec<_>>();
        let sources = references
            .iter()
            .filter(|path| {
                values("source-suffix")
                    .iter()
                    .any(|suffix| path.ends_with(suffix))
                    && !tests.contains(path)
            })
            .collect::<Vec<_>>();
        if sources.len() != 1 || tests.len() != usize::from(regression) {
            continue;
        }
        let tokens = clause
            .text
            .split(crate::meta_reasoner::text::is_js_whitespace)
            .filter(|part| !part.is_empty())
            .map(|value| {
                value
                    .trim_start_matches(['`', '\'', '"', '('])
                    .trim_end_matches(['`', '\'', '"', ')', ',', '.', ';', ':'])
            })
            .collect::<Vec<_>>();
        let mut declaration: Option<&str> = None;
        let mut ambiguous = false;
        for pair in tokens.windows(2) {
            if !values("declaration-marker")
                .iter()
                .any(|marker| words(marker) == words(pair[0]))
            {
                continue;
            }
            let next = pair[1];
            if next.is_empty()
                || !next.chars().enumerate().all(|(index, character)| {
                    character.is_ascii_alphabetic()
                        || character == '_'
                        || character == '$'
                        || index > 0 && character.is_ascii_digit()
                })
                || declaration.is_some_and(|name| name != next)
            {
                ambiguous = true;
                break;
            }
            declaration = Some(next);
        }
        if ambiguous {
            continue;
        }
        let target = if regression { tests[0] } else { sources[0] };
        let start = clause.span.start + task[clause.span.clone()].find(clause.text)?;
        owned.push(json!({"schema":"work-item-comparison-contract/v1","mode":if regression{"regression"}else{"repair"},
            "owner":sources[0],"target":target,"declaration":declaration,"bound":if upper{"upper"}else{"lower"},"inclusive":inclusive,
            "sourceClause":clause.text,"sourceSpan":[start,start+clause.text.len()],"evidencePaths":[sources[0]],
            "targetObservationRequired":true,"allowMissing":if regression{vec![target]}else{Vec::<&String>::new()},
            "independentAcceptanceRequired":true,"scope":"finite-number-interval-only"}));
    }
    if owned.len() == 1 { owned.pop() } else { None }
}

/// Mirrors `boundaryCandidateObservations`: keep the latest matching read before
/// the first owned effect. An untyped latest read remains unknown; it cannot
/// expose an older successful read as the current observation.
pub(super) fn boundary_candidate_observations<'a>(
    contract: &Value,
    progress: &'a Progress,
) -> std::collections::BTreeMap<String, Option<&'a SourceReadObservation>> {
    let Some(owner) = contract["owner"].as_str() else {
        return std::collections::BTreeMap::new();
    };
    let Some(target) = contract["target"].as_str() else {
        return std::collections::BTreeMap::new();
    };
    let attempts = progress.research_attempts();
    let first_effect = attempts.iter().position(|attempt| {
        matches!(attempt.capability, Capability::Write | Capability::Edit)
            && attempt
                .arguments
                .as_deref()
                .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
                .is_some_and(|arguments| bound_path(&arguments, target))
    });
    let before_attempts = &attempts[..first_effect.unwrap_or(attempts.len())];
    let observed = |path: &str| -> Option<&'a SourceReadObservation> {
        before_attempts
            .iter()
            .rev()
            .find(|attempt| {
                attempt.capability == Capability::Read
                    && attempt
                        .arguments
                        .as_deref()
                        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
                        .is_some_and(|arguments| bound_path(&arguments, path))
            })
            .and_then(|attempt| attempt.source_observation())
    };
    [
        (owner.to_owned(), observed(owner)),
        (target.to_owned(), observed(target)),
    ]
    .into_iter()
    .collect()
}

/// Construct a candidate using current, scanner-qualified source observations.
/// The registered native operation uses native complete CSTs and generated core.
pub(super) fn derive_boundary_work_item_candidate(contract: &Value, progress: &Progress) -> Value {
    let Some(owner) = contract["owner"].as_str() else {
        return json!({"gap":"BoundaryContractInvalid"});
    };
    let Some(target) = contract["target"].as_str() else {
        return json!({"gap":"BoundaryContractInvalid"});
    };
    let observations = boundary_candidate_observations(contract, progress);
    let source_observation = observations.get(owner).copied().flatten();
    let target_observation = observations.get(target).copied().flatten();
    let Some(source) = complete(source_observation) else {
        return json!({"gap":"BoundaryCompleteSourceRequired","missingEvidence":[owner]});
    };
    if source.len() > 131072 {
        return json!({"gap":"BoundarySourceInputLimit"});
    }
    let Some(target_seen) = target_observation else {
        return json!({"gap":"BoundaryTargetObservationRequired","missingEvidence":[target]});
    };
    let regression = contract["mode"].as_str() == Some("regression");
    let before = complete(Some(target_seen));
    if !regression && before.is_none() {
        return json!({"gap":"BoundaryCompleteTargetRequired"});
    }
    if regression && before.is_none() && !target_seen.absent {
        return json!({"gap":"BoundaryTargetAvailabilityUnresolved"});
    }
    let request = json!({"source":source,"owner":owner,"declaration":contract["declaration"],"bound":contract["bound"],"inclusive":contract["inclusive"],"mode":contract["mode"],"testOwner":target});
    let result = match observed_comparison_boundary_operation(&request) {
        Ok(result) => result,
        Err(reason) => return json!({"gap":reason}),
    };
    let Some(after) = result["candidate"]["source"].as_str() else {
        return json!({"gap":"BoundaryCandidateIdentityMismatch"});
    };
    if result["schema"] != "observed-comparison-boundary-operation/v1"
        || result["candidate"]["path"] != target
        || result["candidate"]["afterSha256"] != digest(after)
    {
        return json!({"gap":"BoundaryCandidateIdentityMismatch"});
    }
    if !regression && result["candidate"]["beforeSha256"] != digest(source) {
        return json!({"gap":"BoundaryCandidatePreimageMismatch"});
    }
    if regression && result["candidate"]["sourceOwnerSha256"] != digest(source) {
        return json!({"gap":"BoundaryCandidateSourceMismatch"});
    }
    if regression && before.is_some_and(|text| text != after) {
        return json!({"gap":"BoundaryExistingRegressionPreserved"});
    }
    json!({"candidate":{"kind":"structural-comparison-boundary","mode":contract["mode"],"path":target,
        "before":before,"after":after,"requiresAbsent":before.is_none(),"sourceOwner":owner,"sourceIdentity":digest(source),
        "beforeIdentity":before.map(digest),"afterIdentity":digest(after),"noOp":before==Some(after),
        "sourceClause":contract["sourceClause"],"sourceSpan":contract["sourceSpan"],
        "preconditions":[{"kind":"complete-current-source","path":owner,"sha256":digest(source)},
            if let Some(text)=before{json!({"kind":"exact-target-preimage","path":target,"sha256":digest(text)})}else{json!({"kind":"absent-target","path":target})}],
        "checks":[{"kind":"structural-contract","status":"passed","detail":result["verification"]},
            {"kind":"physical-readback","status":"pending"},{"kind":"independent-task-acceptance","status":"pending"}],
        "rule":result.get("rule").cloned().unwrap_or(Value::Null),"sourceModel":result["sourceModel"],
        "suppliedMechanism":true,"autonomousAuthorshipCredit":0,"independentlyAccepted":false}})
}

pub(super) fn verify_boundary_candidate_readback(
    candidate: &Value,
    observation: Option<&SourceReadObservation>,
) -> Value {
    let Some(source) = complete(observation) else {
        return json!({"verified":false,"reason":"BoundaryCompleteReadbackRequired"});
    };
    if candidate["after"] != source || candidate["afterIdentity"] != digest(source) {
        return json!({"verified":false,"reason":"BoundaryPhysicalReadbackMismatch"});
    }
    json!({"verified":true,"noOp":candidate["noOp"],"implementationChanged":candidate["noOp"]!=true,
        "sourceIdentity":digest(source),"independentTaskAcceptance":false})
}

pub(super) enum BoundaryEffectOutcome {
    Pending(AgenticPlan),
    Effect(Value),
    Gap(String),
    Failure { reason: String, detail: String },
}
fn bound_path(arguments: &Value, path: &str) -> bool {
    let values = ["path", "filePath", "file_path"]
        .iter()
        .filter_map(|key| arguments.get(*key))
        .collect::<Vec<_>>();
    !values.is_empty() && values.iter().all(|value| value.as_str() == Some(path))
}
fn boundary_attempt_guard(progress: &Progress, target: &str) -> Option<Value> {
    let attempts = progress.research_attempts();
    let mutations = attempts
        .iter()
        .filter(|attempt| matches!(attempt.capability, Capability::Write | Capability::Edit))
        .collect::<Vec<_>>();
    let gap = if mutations.len() > 1 {
        Some("BoundaryEffectAttemptBudgetExceeded")
    } else if mutations.iter().any(|attempt| {
        attempt
            .arguments
            .as_deref()
            .and_then(|text| serde_json::from_str::<Value>(text).ok())
            .is_none_or(|arguments| !bound_path(&arguments, target))
    }) {
        Some("BoundaryUnrelatedOrAmbiguousMutationAttempt")
    } else if attempts.iter().any(|attempt| {
        if attempt.capability != Capability::Read {
            return false;
        }
        let Some(arguments) = attempt
            .arguments
            .as_deref()
            .and_then(|text| serde_json::from_str::<Value>(text).ok())
        else {
            return true;
        };
        let paths = ["path", "filePath", "file_path", "absolute_path"]
            .iter()
            .filter_map(|key| arguments.get(*key))
            .collect::<Vec<_>>();
        paths.is_empty()
            || paths
                .iter()
                .any(|path| path.as_str().is_none() || path.as_str() != paths[0].as_str())
    }) {
        Some("BoundaryAmbiguousReadPathAttempt")
    } else {
        None
    };
    gap.map(|reason| json!({"gap":reason,"budget":{"maximum":1,"observed":mutations.len()}}))
}

/// Operate a persisted candidate; never recompute its preimage after a write.
pub(super) fn plan_boundary_candidate_effect(
    candidate: &Value,
    messages: &[ChatMessage],
    tools: &[&str],
    task: &str,
) -> BoundaryEffectOutcome {
    use BoundaryEffectOutcome::{Effect, Failure, Gap};
    let Some(path) = candidate["path"].as_str() else {
        return Gap("BoundaryContractInvalid".to_owned());
    };
    let progress = Progress::scan(messages);
    if let Some(pending) = file_io_pending_step(&progress, tools) {
        return lower_file_io(pending);
    }
    if let Some(guard) = boundary_attempt_guard(&progress, path) {
        return Gap(guard["gap"]
            .as_str()
            .unwrap_or("BoundaryAttemptUnresolved")
            .to_owned());
    }
    let attempts = progress.research_attempts();
    let effects = attempts
        .iter()
        .enumerate()
        .filter_map(|(index, attempt)| {
            let arguments: Value = serde_json::from_str(attempt.arguments.as_deref()?).ok()?;
            (matches!(attempt.capability, Capability::Write | Capability::Edit)
                && bound_path(&arguments, path))
            .then_some((index, attempt, arguments))
        })
        .collect::<Vec<_>>();
    if effects.len() > 1 {
        return Gap("BoundaryEffectAttemptBudgetExceeded".to_owned());
    }
    if let Some((index, effect, arguments)) = effects.last() {
        let after = arguments
            .get("content")
            .or_else(|| arguments.get("newString"))
            .or_else(|| arguments.get("new_string"));
        if after != Some(&candidate["after"]) {
            return Gap("BoundaryUnrelatedTargetEffect".to_owned());
        }
        if !effect.succeeded {
            return Failure {
                reason: "BoundaryEffectFailed".to_owned(),
                detail: effect.detail.clone(),
            };
        }
        let observed = attempts[index + 1..].iter().rev().find_map(|attempt| {
            let arguments: Value = serde_json::from_str(attempt.arguments.as_deref()?).ok()?;
            (attempt.capability == Capability::Read && bound_path(&arguments, path))
                .then(|| attempt.source_observation())
                .flatten()
        });
        let Some(observed) = observed else {
            return lower_file_io(file_io_read_step(messages, tools, path, task));
        };
        let mut checked = verify_boundary_candidate_readback(candidate, Some(observed));
        if checked["verified"] != true {
            return Failure {
                reason: checked["reason"]
                    .as_str()
                    .unwrap_or("BoundaryReadbackFailed")
                    .to_owned(),
                detail: String::new(),
            };
        }
        checked["path"] = json!(path);
        checked["authorshipBoundary"] = json!("candidate-output-only");
        checked["independentlyAccepted"] = json!(false);
        return Effect(checked);
    }
    let target = progress.source_read_for(path);
    if candidate["noOp"] == true {
        let mut checked = verify_boundary_candidate_readback(candidate, target);
        if checked["verified"] != true {
            return Gap(checked["reason"]
                .as_str()
                .unwrap_or("BoundaryReadbackFailed")
                .to_owned());
        }
        checked["path"] = json!(path);
        checked["authorshipBoundary"] = json!("existing-artifact-no-authorship");
        checked["independentlyAccepted"] = json!(false);
        return Effect(checked);
    }
    let Some(after) = candidate["after"].as_str() else {
        return Gap("BoundaryContractInvalid".to_owned());
    };
    if candidate["requiresAbsent"] == true {
        if !target.is_some_and(|value| value.absent) {
            return Gap("BoundaryTargetPreconditionChanged".to_owned());
        }
        return lower_file_io(file_io_effect_step(messages, tools, path, None, after));
    }
    let before = complete(target);
    if before != candidate["before"].as_str()
        || before.map(digest).as_deref() != candidate["beforeIdentity"].as_str()
    {
        return Gap("BoundaryTargetPreconditionChanged".to_owned());
    }
    lower_file_io(file_io_effect_step(messages, tools, path, before, after))
}

fn lower_file_io(plan: FileIoPlan) -> BoundaryEffectOutcome {
    match plan {
        FileIoPlan::Pending(step) => BoundaryEffectOutcome::Pending(step),
        FileIoPlan::Gap(reason) => BoundaryEffectOutcome::Gap(reason),
    }
}

/// Native whole-route wrapper. The work-item router invokes this before its
/// generic latest-failure/read fallback. Whole-task status remains Gap until a
/// separate independent checker, outside this candidate, accepts requirements.
pub(super) fn plan_work_item_boundary_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: Option<&str>,
    paths_for_clause: impl Fn(&str) -> Vec<String>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let contract = boundary_work_item_contract(task, paths_for_clause)?;
    let progress = Progress::scan(messages);
    let mut observations = Vec::new();
    let finish = |reason: &str,
                  observations: &[Value],
                  candidate: Option<&Value>,
                  effect: Option<&Value>,
                  detail: Value,
                  result: &mut Option<FinalResult>| {
        let authored = effect.is_some_and(|value| value["implementationChanged"] == true);
        let source_start = contract["sourceSpan"][0].as_u64().unwrap_or(u64::MAX);
        let source_end = contract["sourceSpan"][1].as_u64().unwrap_or(u64::MAX);
        let requirements=crate::obligation_ledger::clauses_with_spans(task).into_iter().map(|(text,(start,end))|json!({
            "text":text,"span":[start,end],"status":if (start as u64)<=source_start&&(end as u64)>=source_end{"independent_check_pending"}else{"unresolved"}
        })).collect::<Vec<_>>();
        let candidate_view=candidate.map(|candidate| {
            let mut checks=candidate["checks"].as_array().cloned().unwrap_or_default();
            for check in &mut checks {if check["kind"]=="physical-readback"{check["status"]=json!(if effect.is_some_and(|value|value["verified"]==true){"passed"}else{"pending"});}}
            json!({"kind":candidate["kind"],"mode":candidate["mode"],"path":candidate["path"],
                "beforeIdentity":candidate["beforeIdentity"],"afterIdentity":candidate["afterIdentity"],
                "sourceIdentity":candidate["sourceIdentity"],"sourceOwner":candidate["sourceOwner"],
                "preconditions":candidate["preconditions"],"noOp":candidate["noOp"],
                "sourceRepresentation":{"kind":"complete-readable-cst","identity":digest(&candidate["sourceModel"].to_string())},"checks":checks})
        });
        let checks = candidate_view
            .as_ref()
            .map(|value| value["checks"].clone())
            .unwrap_or_else(|| json!([]));
        let remaining=requirements.iter().map(|value|json!({"text":value["text"],"span":value["span"],"reason":if value["status"]=="independent_check_pending"{"independent-requirement-acceptance-required"}else{"unsupported-or-unbound-obligation"}})).collect::<Vec<_>>();
        let payload = json!({"schema":"repository-work-item-controller/v1","transportDiagnostics":progress.transport_diagnostics(),"reason":reason,"workItem":work_item,
            "taskIdentity":digest(task),"requirements":requirements,"supportedSubset":"finite-number-interval-boundary-and-regression",
            "boundContract":contract,"observations":observations,"candidate":candidate_view,"checks":checks,
            "effects":effect.into_iter().collect::<Vec<_>>(),"authored":authored,"verified":false,"independentlyAccepted":false,"complete":false,
            "physicalReadbackVerified":effect.is_some_and(|value|value["verified"]==true),"noOp":effect.is_some_and(|value|value["noOp"]==true),
            "usefulChange":{"observed":authored,"independentlyAccepted":false,"paths":if authored{vec![contract["target"].clone()]}else{Vec::new()}},
            "authorshipBoundary":{"mechanism":"supplied-generic-structural-operation","targetOutputObserved":authored,"autonomousCredit":0,"laterReviewerWork":"not-observed"},
            "remainingWork":remaining,"budget":{"maximumEffects":1,"maximumEvidenceFiles":2},"detail":detail});
        record(
            AgenticPlan::Final(payload.to_string()),
            FinalDisposition::Gap,
            "repository_work_item_boundary",
            result,
        )
    };
    if let Some(pending) = file_io_pending_step(&progress, tools) {
        return Some(match pending {
            FileIoPlan::Pending(step) => step,
            FileIoPlan::Gap(reason) => {
                finish(&reason, &observations, None, None, Value::Null, result)
            }
        });
    }
    if let Some(guard) = boundary_attempt_guard(&progress, contract["target"].as_str()?) {
        return Some(finish(
            guard["gap"].as_str().unwrap_or("BoundaryAttemptUnresolved"),
            &observations,
            None,
            None,
            guard.clone(),
            result,
        ));
    }
    let mut paths = vec![contract["owner"].as_str()?, contract["target"].as_str()?];
    paths.dedup();
    for path in paths {
        let Some(observed) = progress.source_read_for(path) else {
            return Some(match file_io_read_step(messages, tools, path, task) {
                FileIoPlan::Pending(step) => step,
                FileIoPlan::Gap(reason) => {
                    finish(&reason, &observations, None, None, Value::Null, result)
                }
            });
        };
        let source = complete(Some(observed));
        let allowed_absent = contract["allowMissing"]
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value == path))
            && observed.absent;
        let status = match observed.status {
            SourceReadStatus::ReportedSuccess => "reported-success",
            SourceReadStatus::ReportedFailure => "reported-failure",
            SourceReadStatus::Unknown => "unknown",
        };
        observations.push(json!({"path":path,"complete":source.is_some(),"absent":observed.absent,"error":observed.error,"status":status,"sourceIdentity":source.map(digest)}));
        if source.is_none() && !allowed_absent {
            return Some(finish(
                "IncompleteSourceObservation",
                &observations,
                None,
                None,
                Value::Null,
                result,
            ));
        }
    }
    let proposed = derive_boundary_work_item_candidate(&contract, &progress);
    let Some(candidate) = proposed.get("candidate") else {
        return Some(finish(
            proposed["gap"]
                .as_str()
                .unwrap_or("BoundaryCandidateUnavailable"),
            &observations,
            None,
            None,
            proposed.clone(),
            result,
        ));
    };
    match plan_boundary_candidate_effect(candidate, messages, tools, task) {
        BoundaryEffectOutcome::Pending(step) => {
            Some(if super::file_read::read_policy_blocks_plan(task, &step) {
                finish(
                    "ReadPolicyRefused",
                    &observations,
                    Some(candidate),
                    None,
                    Value::Null,
                    result,
                )
            } else {
                step
            })
        }
        BoundaryEffectOutcome::Effect(effect) => Some(finish(
            if effect["noOp"] == true {
                "VerifiedNoOpAwaitingIndependentAcceptance"
            } else {
                "UsefulCandidateAwaitingIndependentAcceptance"
            },
            &observations,
            Some(candidate),
            Some(&effect),
            Value::Null,
            result,
        )),
        BoundaryEffectOutcome::Gap(reason) => Some(finish(
            &reason,
            &observations,
            Some(candidate),
            None,
            json!({"gap":reason}),
            result,
        )),
        BoundaryEffectOutcome::Failure { reason, detail } => Some(finish(
            &reason,
            &observations,
            Some(candidate),
            None,
            json!({"failure":reason,"detail":detail}),
            result,
        )),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/work_item_boundary_candidate_routes.rs"]
mod route_tests;
