//! Manual native projection of owned source-repair intake; CI-only qualification.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability};
use super::shell_command_policy::sentences;
use super::source_dictionary_binding::derive_own_key_dictionary_repair;
use super::source_documentation::concept_words;
use super::work_item_file_io::FileIoAttempt;
use super::work_item_file_io::{
    FileIoPlan, WorkItemFileProgress, file_io_effect_step, file_io_pending_step, file_io_read_step,
};
use crate::normal_markov::{quote_fault, quoted_segment_spans};
use crate::obligation_ledger::clauses_with_spans;
use crate::protocol::ChatMessage;
use crate::seed::parser::parse_lino;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
const POLICY: &str = include_str!("../../embedded/data/meta/owned-source-repair.lino");
pub(super) fn source_repair_contract(task: &str) -> Option<Value> {
    if task.len() > 262_144 || quote_fault(task).is_some() {
        return None;
    }
    let parsed = parse_lino(POLICY);
    let root = parsed
        .children
        .iter()
        .find(|node| node.name == "owned-source-repair")?;
    let values = |name: &str| {
        root.children
            .iter()
            .filter(|node| node.name == name)
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    };
    let actions = values("action");
    let prefixes = values("prefix");
    let negations = values("negation");
    let mut masked = task.as_bytes().to_vec();
    for span in quoted_segment_spans(task) {
        for byte in &mut masked[span.start..span.end] {
            if !matches!(*byte, b'\r' | b'\n') {
                *byte = b' ';
            }
        }
    }
    let masked = String::from_utf8(masked).expect("MaskedSourceRepairUtf8");
    let paths: Vec<_> = super::work_item_evidence::work_item_evidence_paths(task)
        .into_iter()
        .filter(|path| path.ends_with(".mjs") || path.ends_with(".js"))
        .collect();
    if paths.is_empty() {
        return None;
    }
    let keys = values("key-cue");
    let bindings = values("binding-cue");
    let dictionary_scopes = values("dictionary-scope-cue");
    let dictionary_relations = values("dictionary-relation-cue");
    let mut repair_goals = Vec::new();
    let mut owned = false;
    let mut negated = false;
    let mut operation = None;
    for clause in sentences(&masked) {
        let words = concept_words(clause.text);
        for action in &actions {
            let Some(index) = words.iter().position(|word| word == action) else {
                continue;
            };
            let before = words[..index].join(" ");
            if negations.iter().any(|cue| {
                format!(" {before} ").contains(&format!(" {} ", concept_words(cue).join(" ")))
            }) {
                negated = true;
            }
            if index == 0 || prefixes.contains(&before) {
                owned = true;
                repair_goals.push(
                    dictionary_scopes.iter().any(|cue| words.contains(cue))
                        && words.iter().enumerate().any(|(key_index, word)| {
                            keys.contains(word)
                                && words.iter().enumerate().any(|(relation_index, relation)| {
                                    relation_index > key_index
                                        && dictionary_relations.contains(relation)
                                        && words.iter().enumerate().any(
                                            |(binding_index, binding)| {
                                                binding_index > relation_index
                                                    && bindings.contains(binding)
                                            },
                                        )
                                })
                        }),
                );
            }
        }
    }
    if !owned && !negated {
        return None;
    }
    if !repair_goals.is_empty() && repair_goals.iter().all(|goal| *goal) {
        operation = Some("own-key-storage");
    }
    Some(
        json!({"operation":operation,"owner":if paths.len()==1 { Some(paths[0]) } else { None },"authority":if negated {"NegatedSourceRepairAction"} else if paths.len()==1 {"owned"} else {"AmbiguousSourceRepairOwner"}}),
    )
}
fn argument_path(arguments: Option<&str>) -> Option<String> {
    let object: Value = serde_json::from_str(arguments?).ok()?;
    let values: Vec<_> = ["filePath", "file_path", "path"]
        .iter()
        .filter_map(|name| object.get(*name))
        .collect();
    let first = values.first()?.as_str()?;
    values
        .iter()
        .all(|value| value.as_str() == Some(first))
        .then(|| first.to_owned())
}
fn complete(attempt: &FileIoAttempt) -> Option<&str> {
    let observed = attempt.source_observation()?;
    let source = observed.source.as_deref()?;
    (attempt.succeeded && observed.error.is_none() && observed.complete && source.len() <= 262_144)
        .then_some(source)
}
fn argument_alias(arguments: &Value, names: &[&str]) -> Option<String> {
    let object = arguments.as_object()?;
    let values: Vec<_> = names.iter().filter_map(|name| object.get(*name)).collect();
    let first = values.first()?.as_str()?;
    values
        .iter()
        .all(|value| value.as_str() == Some(first))
        .then(|| first.to_owned())
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
        "owned-source-repair",
        result,
    )
}
fn io(plan: FileIoPlan, state: &Value, result: &mut Option<FinalResult>) -> AgenticPlan {
    match plan {
        FileIoPlan::Pending(step) => step,
        FileIoPlan::Gap(reason) => finish(state, &reason, false, result),
    }
}
pub(super) fn plan_owned_source_repair(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: Option<&str>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let contract = source_repair_contract(task)?;
    let progress = WorkItemFileProgress::scan(messages);
    let attempts = progress.research_attempts();
    let mut requirements: Vec<_>=clauses_with_spans(task).into_iter().map(|(text,(start,end))|json!({"text":text,"span":[start,end],"kind":"source-repair","status":"unverified"})).collect();
    if requirements.is_empty() {
        requirements.push(
            json!({"text":task,"span":[0,task.len()],"kind":"source-repair","status":"unverified"}),
        );
    }
    let mut state = json!({"schema":"owned-source-repair/v1","task":task,"taskIdentity":sha256_hex(task.as_bytes()),"workItem":work_item,"source":contract["owner"],"target":contract["owner"],"requirements":requirements,"observations":[],"candidateEdits":[],"checks":[],"effects":[],"remainingWork":["source-repair-operation","independent-requirement-acceptance"],"authored":false,"verified":false,"complete":false});
    if contract["authority"] != "owned" {
        return Some(finish(
            &state,
            contract["authority"]
                .as_str()
                .unwrap_or("SourceRepairActionUnowned"),
            false,
            result,
        ));
    }
    let owner = contract["owner"].as_str()?;
    if let Some(pending) = file_io_pending_step(&progress, tools) {
        return Some(io(pending, &state, result));
    }
    state["transportDiagnostics"] = json!(progress.transport_diagnostics());
    if attempts.iter().any(|attempt| {
        attempt.capability == Capability::Read
            && argument_path(attempt.arguments.as_deref()).is_none()
    }) {
        return Some(finish(&state, "AmbiguousSourceRepairRead", true, result));
    }
    let effects: Vec<_> = attempts
        .iter()
        .enumerate()
        .filter(|(_, attempt)| matches!(attempt.capability, Capability::Write | Capability::Edit))
        .collect();
    if effects.len() > 1 {
        return Some(finish(&state, "MultipleSourceRepairEffects", true, result));
    }
    if effects.iter().any(|(_, attempt)| {
        attempt.capability != Capability::Edit
            || argument_path(attempt.arguments.as_deref()).as_deref() != Some(owner)
    }) {
        return Some(finish(&state, "UnrelatedSourceRepairEffect", true, result));
    }
    let effect = effects.first().copied();
    let reads = |before: usize, after: Option<usize>| {
        attempts
            .iter()
            .enumerate()
            .filter(|(index, attempt)| {
                *index < before
                    && after.is_none_or(|after| *index > after)
                    && attempt.capability == Capability::Read
                    && argument_path(attempt.arguments.as_deref()).as_deref() == Some(owner)
            })
            .collect::<Vec<_>>()
    };
    let prior = reads(effect.map_or(attempts.len(), |(index, _)| index), None);
    if prior.is_empty() {
        return Some(io(
            file_io_read_step(messages, tools, owner, task),
            &state,
            result,
        ));
    }
    let original = prior.iter().find_map(|(_, attempt)| complete(attempt));
    let latest = prior.last().and_then(|(_, attempt)| complete(attempt));
    let (Some(original), Some(latest)) = (original, latest) else {
        return Some(finish(
            &state,
            "SourceRepairCompleteObservationRequired",
            false,
            result,
        ));
    };
    if original != latest {
        return Some(finish(&state, "SourceRepairPreimageChanged", true, result));
    }
    state["observations"] =
        json!([{"path":owner,"sourceIdentity":sha256_hex(original.as_bytes()),"complete":true}]);
    if contract["operation"].is_null() {
        return Some(finish(
            &state,
            "SourceRepairOperatorUnavailable",
            false,
            result,
        ));
    }
    let derived = match derive_own_key_dictionary_repair(owner, original) {
        Ok(value) => value,
        Err(reason) => return Some(finish(&state, &reason, effect.is_some(), result)),
    };
    let candidate = &derived["candidate"];
    let next = candidate["source"].as_str();
    if derived["schema"] != "observed-own-key-dictionary-repair/v1"
        || derived["owner"] != owner
        || candidate["path"] != owner
        || candidate["beforeSha256"] != sha256_hex(original.as_bytes())
        || next.is_none_or(|next| {
            next == original
                || next.len() > 262_144
                || candidate["afterSha256"] != sha256_hex(next.as_bytes())
        })
        || derived["verification"]["completeConcreteResidual"] != true
        || derived["verification"]["independentGrammarReparse"] != true
        || derived["verification"]["exactInverse"] != true
    {
        return Some(finish(&state, "SourceRepairCandidateInvalid", true, result));
    }
    let next = next.expect("CheckedCandidateSource");
    state["binding"] = derived["binding"].clone();
    state["sourceProof"] = derived["verification"].clone();
    state["knowledge"] = derived["knowledge"].clone();
    state["candidateEdits"] = json!([{"path":owner,"preimageIdentity":sha256_hex(original.as_bytes()),"postimageIdentity":sha256_hex(next.as_bytes())}]);
    let Some((effect_index, effect)) = effect else {
        return Some(io(
            file_io_effect_step(messages, tools, owner, original, next),
            &state,
            result,
        ));
    };
    let arguments: Value = match effect
        .arguments
        .as_deref()
        .and_then(|raw| serde_json::from_str(raw).ok())
    {
        Some(value) => value,
        None => {
            return Some(finish(
                &state,
                "SourceRepairEffectArgumentsInvalid",
                true,
                result,
            ));
        }
    };
    if argument_alias(&arguments, &["oldString", "old_string", "old_str", "old"]).as_deref()
        != Some(original)
        || argument_alias(&arguments, &["newString", "new_string", "new_str", "new"]).as_deref()
            != Some(next)
    {
        return Some(finish(&state, "SourceRepairEffectNotDerived", true, result));
    }
    if !effect.succeeded {
        return Some(finish(&state, "SourceRepairEditFailed", true, result));
    }
    let fresh = reads(attempts.len(), Some(effect_index));
    let Some((_, fresh)) = fresh.last() else {
        return Some(io(
            file_io_read_step(messages, tools, owner, task),
            &state,
            result,
        ));
    };
    if complete(fresh) != Some(next) {
        return Some(finish(&state, "SourceRepairReadbackMismatch", true, result));
    }
    state["effects"] = json!([{"path":owner,"kind":"edit","observedPostimageIdentity":sha256_hex(next.as_bytes())}]);
    state["checks"] = json!([{"kind":"derived-source-proof-and-exact-readback","passed":true}]);
    state["remainingWork"] = json!(["independent-requirement-acceptance"]);
    Some(finish(
        &state,
        "UsefulSourceRepairReadbackVerified",
        false,
        result,
    ))
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/source_repair.rs"]
mod tests;
