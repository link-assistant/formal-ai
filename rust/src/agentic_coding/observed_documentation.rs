//! Shared observed-source documentation custody. Manual native projection only.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability};
use super::work_item_file_io::{
    FileIoAttempt, FileIoPlan, WorkItemFileProgress, file_io_effect_step, file_io_pending_step,
    file_io_read_step,
};
use crate::obligation_ledger::clauses_with_spans;
use crate::protocol::ChatMessage;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
use std::collections::BTreeSet;

const MAXIMUM_BYTES: usize = 262_144;
fn arguments_of(attempt: &FileIoAttempt) -> Option<Value> {
    serde_json::from_str(attempt.arguments.as_deref()?).ok()
}
fn argument_alias(value: &Value, names: &[&str]) -> Option<String> {
    let object = value.as_object()?;
    let values: Vec<_> = names.iter().filter_map(|name| object.get(*name)).collect();
    let first = values.first()?.as_str()?;
    values
        .iter()
        .all(|value| value.as_str() == Some(first))
        .then(|| first.to_owned())
}
fn owner_of(attempt: &FileIoAttempt) -> Option<String> {
    argument_alias(&arguments_of(attempt)?, &["filePath", "file_path", "path"])
}
fn complete(attempt: &FileIoAttempt) -> Option<&str> {
    let read = attempt.source_observation()?;
    let source = read.source.as_deref()?;
    (attempt.succeeded && read.error.is_none() && read.complete && source.len() <= MAXIMUM_BYTES)
        .then_some(source)
}
fn owned_path(path: &str) -> bool {
    path.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        && path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}
fn reads<'a>(
    attempts: &'a [FileIoAttempt],
    path: &str,
    before: usize,
    after: Option<usize>,
) -> Vec<(usize, &'a FileIoAttempt)> {
    attempts
        .iter()
        .enumerate()
        .filter(|(index, attempt)| {
            *index < before
                && after.is_none_or(|after| *index > after)
                && attempt.capability == Capability::Read
                && owner_of(attempt).as_deref() == Some(path)
        })
        .collect()
}
fn latest_read<'a>(
    attempts: &'a [FileIoAttempt],
    path: &str,
    before: usize,
    after: Option<usize>,
) -> Option<(usize, &'a FileIoAttempt)> {
    reads(attempts, path, before, after).into_iter().next_back()
}
fn finish(
    state: &Value,
    reason: &str,
    failure: bool,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let mut payload = state.clone();
    payload["reason"] = json!(reason);
    record(
        AgenticPlan::Final(payload.to_string()),
        if failure {
            FinalDisposition::Failure
        } else {
            FinalDisposition::Gap
        },
        "source-documentation-controller",
        result,
    )
}
fn resolve_plan(
    planned: FileIoPlan,
    state: &Value,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    match planned {
        FileIoPlan::Pending(step) => step,
        FileIoPlan::Gap(reason) => finish(state, &reason, false, result),
    }
}
fn read_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    path: &str,
    state: &Value,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    resolve_plan(
        file_io_read_step(messages, tools, path, task),
        state,
        result,
    )
}

/// Shared custody only. Derivation cannot grant authority or independent acceptance.
pub(super) fn plan_observed_documentation(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    contract: &Value,
    derive: impl Fn(&[Value], &str) -> Value,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let progress = WorkItemFileProgress::scan(messages);
    let attempts = progress.research_attempts();
    let mut requirements: Vec<_> = clauses_with_spans(task).into_iter()
        .map(|(text, (start, end))| json!({"text":text,"span":[start,end],"kind":"source-documentation","status":"unverified"}))
        .collect();
    if requirements.is_empty() {
        requirements.push(json!({"text":task,"span":[0,task.len()],"kind":"source-documentation","status":"unverified"}));
    }
    let sources_value = &contract["sources"];
    let target_value = &contract["target"];
    let single_source = sources_value
        .as_array()
        .filter(|sources| sources.len() == 1)
        .map(|sources| sources[0].clone())
        .unwrap_or(Value::Null);
    let schema = contract
        .get("schema")
        .filter(|value| !value.is_null())
        .cloned()
        .unwrap_or_else(|| json!("source-documentation-controller/v1"));
    let mut state = json!({"schema":schema,"task":task,"taskIdentity":sha256_hex(task.as_bytes()),
        "workItem":contract.get("workItem").cloned().unwrap_or(Value::Null),"requirements":requirements,
        "source":single_source,"sources":sources_value,"target":target_value,
        "observations":[],"candidateEdits":[],"checks":[],"effects":[],
        "remainingWork":["independent-requirement-acceptance"],
        "transportDiagnostics":progress.transport_diagnostics(),"authored":false,"verified":false,"complete":false});
    for key in ["sources", "target"] {
        if contract.get(key).is_none() {
            state
                .as_object_mut()
                .expect("DocumentationState")
                .remove(key);
        }
    }
    let sources = sources_value
        .as_array()
        .and_then(|values| values.iter().map(Value::as_str).collect::<Option<Vec<_>>>());
    let target = target_value.as_str();
    let valid = sources.as_ref().is_some_and(|sources| {
        !sources.is_empty()
            && sources.len() <= 16
            && sources.iter().all(|path| owned_path(path))
            && target.is_some_and(owned_path)
            && sources
                .iter()
                .copied()
                .chain(target)
                .collect::<BTreeSet<_>>()
                .len()
                == sources.len() + 1
    });
    if !valid {
        return finish(&state, "DocumentationContractInvalid", true, result);
    }
    let sources = sources.unwrap_or_default();
    let target = target.unwrap_or_default();
    if contract["authority"] != "owned" {
        return finish(
            &state,
            contract["authority"]
                .as_str()
                .unwrap_or("DocumentationActionUnowned"),
            false,
            result,
        );
    }
    if let Some(planned) = file_io_pending_step(&progress, tools) {
        return resolve_plan(planned, &state, result);
    }
    if attempts
        .iter()
        .any(|attempt| attempt.capability == Capability::Read && owner_of(attempt).is_none())
    {
        return finish(&state, "AmbiguousReadPathAttempt", true, result);
    }
    let effects: Vec<_> = attempts
        .iter()
        .enumerate()
        .filter(|(_, attempt)| matches!(attempt.capability, Capability::Edit | Capability::Write))
        .collect();
    if effects.iter().any(|(_, attempt)| {
        attempt.capability != Capability::Edit || owner_of(attempt).as_deref() != Some(target)
    }) {
        return finish(&state, "UnrelatedOrAmbiguousMutationAttempt", true, result);
    }
    if effects.len() > 1 {
        return finish(&state, "MultipleEditAttempts", true, result);
    }
    let effect = effects.first().copied();
    for path in sources.iter().copied().chain(std::iter::once(target)) {
        let Some((_, observed)) = latest_read(attempts, path, attempts.len(), None) else {
            return read_step(task, messages, tools, path, &state, result);
        };
        let Some(content) = complete(observed) else {
            return finish(
                &state,
                "IncompleteOrOversizedSourceObservation",
                false,
                result,
            );
        };
        state["observations"].as_array_mut().expect("DocumentationObservations").push(
            json!({"path":path,"sourceIdentity":sha256_hex(content.as_bytes()),"complete":true}));
    }
    let cutoff = effect.map_or(attempts.len(), |(index, _)| index);
    let frozen_sources: Vec<_> = sources.iter().map(|path| {
        let source = reads(attempts, path, cutoff, None).into_iter().find_map(|(_, attempt)| complete(attempt));
        json!({"path":path,"source":source,"sourceIdentity":source.map(|source|sha256_hex(source.as_bytes()))})
    }).collect();
    let guide =
        latest_read(attempts, target, cutoff, None).and_then(|(_, attempt)| complete(attempt));
    if guide.is_none() || frozen_sources.iter().any(|row| row["source"].is_null()) {
        return finish(
            &state,
            "EditPreconditionObservationMissing",
            effect.is_some(),
            result,
        );
    }
    let guide = guide.unwrap_or_default();
    for source in &frozen_sources {
        let path = source["path"].as_str().unwrap_or_default();
        let latest =
            latest_read(attempts, path, cutoff, None).and_then(|(_, attempt)| complete(attempt));
        if latest.is_none_or(|content| source["sourceIdentity"] != sha256_hex(content.as_bytes())) {
            return finish(&state, "SourceFreshnessMismatch", true, result);
        }
    }
    // The supplied native derivation is a typed pure function; it reports expected
    // unsupported evidence through {gap}. Panics remain bugs, not observations.
    let derived = derive(&frozen_sources, guide);
    let remaining_valid = derived.get("remainingWork").is_none_or(|remaining| {
        remaining
            .as_array()
            .is_some_and(|rows| rows.len() <= 64 && rows.iter().all(Value::is_string))
    });
    let facts_valid = derived
        .get("facts")
        .is_none_or(|facts| facts.as_array().is_some_and(|rows| rows.len() <= 256));
    if !derived.is_object() || !remaining_valid || !facts_valid {
        return finish(&state, "DocumentationDerivationInvalid", true, result);
    }
    if derived.get("gap").is_some_and(|gap| {
        gap.as_str()
            .is_none_or(|value| value.is_empty() || value.chars().count() > 128)
    }) {
        return finish(&state, "DocumentationDerivationInvalid", true, result);
    }
    if let Some(gap) = derived
        .get("gap")
        .and_then(Value::as_str)
        .filter(|gap| !gap.is_empty())
    {
        return finish(&state, gap, effect.is_some(), result);
    }
    let Some(content) = derived
        .get("content")
        .and_then(Value::as_str)
        .filter(|content| content.len() <= MAXIMUM_BYTES)
    else {
        return finish(&state, "DocumentationCandidateBudgetExceeded", true, result);
    };
    state["binding"] = derived.get("binding").filter(|value| !value.is_null()).cloned()
        .unwrap_or_else(|| json!({"key":derived["key"],"value":derived["value"],"sourceIdentity":derived["sourceIdentity"]}));
    if let Some(facts) = derived.get("facts") {
        state["facts"] = facts.clone();
    }
    if let Some(remaining) = derived.get("remainingWork").and_then(Value::as_array) {
        for item in remaining {
            let items = state["remainingWork"]
                .as_array_mut()
                .expect("DocumentationRemainingWork");
            if !items.contains(item) {
                items.push(item.clone());
            }
        }
    }
    if let Some((index, edit)) = effect {
        let argument_values = arguments_of(edit).unwrap_or(Value::Null);
        let old = argument_alias(
            &argument_values,
            &["oldString", "old_string", "old_str", "old"],
        );
        let next = argument_alias(
            &argument_values,
            &["newString", "new_string", "new_str", "new"],
        );
        state["candidateEdits"].as_array_mut().expect("DocumentationCandidateEdits").push(
            json!({"path":target,"preimageIdentity":sha256_hex(guide.as_bytes()),"postimageIdentity":sha256_hex(content.as_bytes())}));
        if old.as_deref() != Some(guide) || next.as_deref() != Some(content) || next == old {
            return finish(&state, "UnrelatedEditCandidate", true, result);
        }
        if !edit.succeeded {
            return finish(&state, "CandidateEditFailed", true, result);
        }
        let mut last_fresh_index = index;
        for source in &frozen_sources {
            let path = source["path"].as_str().unwrap_or_default();
            let Some((fresh_index, fresh)) =
                latest_read(attempts, path, attempts.len(), Some(index))
            else {
                return read_step(task, messages, tools, path, &state, result);
            };
            let fresh_content = complete(fresh);
            if fresh_content.is_none_or(|fresh_content| {
                source["sourceIdentity"] != sha256_hex(fresh_content.as_bytes())
            }) {
                return finish(&state, "SourceFreshnessMismatch", true, result);
            }
            last_fresh_index = last_fresh_index.max(fresh_index);
        }
        let Some((_, observed)) =
            latest_read(attempts, target, attempts.len(), Some(last_fresh_index))
        else {
            return read_step(task, messages, tools, target, &state, result);
        };
        let actual = complete(observed);
        if actual != Some(content) {
            return finish(&state, "ReadbackMismatch", true, result);
        }
        state["effects"]
            .as_array_mut()
            .expect("DocumentationEffects")
            .push(json!({"path":target,"kind":"edit",
            "reportedSuccess":true,"observedPostimageIdentity":sha256_hex(content.as_bytes())}));
        state["checks"]
            .as_array_mut()
            .expect("DocumentationChecks")
            .push(json!({"kind":"source-freshness-and-exact-postimage","passed":true}));
        return finish(&state, "UsefulCandidateReadbackVerified", false, result);
    }
    if content == guide {
        state["checks"]
            .as_array_mut()
            .expect("DocumentationChecks")
            .push(json!({"kind":"source-bound-noop","passed":true}));
        return finish(&state, "VerifiedNoOp", false, result);
    }
    resolve_plan(
        file_io_effect_step(messages, tools, target, Some(guide), content),
        &state,
        result,
    )
}
