//! Manual native projection of the bounded source-documentation controller.
//! Source only: no native compilation or verified translation credit claimed.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::{Progress, ToolAttempt};
use super::shell_command_policy::sentences;
use crate::normal_markov::{quote_fault, quoted_segment_spans};
use crate::obligation_ledger::clauses_with_spans;
use crate::protocol::ChatMessage;
use crate::seed::parser::parse_lino;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};

const GRAMMAR: &str = include_str!("../../embedded/data/meta/source-documentation.lino");

fn concept_words(text: &str) -> Vec<String> {
    let camel = regex::Regex::new(r"([a-z0-9])([A-Z])").expect("StaticConceptBoundary");
    let words = regex::Regex::new(r"[^\p{L}\p{N}]+").expect("StaticConceptWords");
    let normalized = camel.replace_all(text, "$1 $2").to_lowercase();
    words
        .split(&normalized)
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

fn documentation_authority(
    task: &str,
    guide_path: &str,
    actions: &[String],
    preserve: &[String],
    negations: &[String],
    prefixes: &[String],
) -> Option<&'static str> {
    if quote_fault(task).is_some() {
        return Some("DocumentationActionUnowned");
    }
    let mut masked = task.as_bytes().to_vec();
    for span in quoted_segment_spans(task) {
        for byte in &mut masked[span.start..span.end] {
            if !matches!(*byte, b'\r' | b'\n') {
                *byte = b' ';
            }
        }
    }
    let masked = String::from_utf8(masked).expect("MaskedQuoteUtf8");
    if !preserve
        .iter()
        .any(|cue| concept_words(&masked).contains(cue))
    {
        return None;
    }
    let mut mentioned = false;
    for clause in sentences(&masked) {
        if !task[clause.span.clone()].contains(guide_path) {
            continue;
        }
        let words = concept_words(clause.text);
        for action in actions {
            let Some(index) = words.iter().position(|word| word == action) else {
                continue;
            };
            mentioned = true;
            let before = words[..index].join(" ");
            if negations.iter().any(|negation| {
                format!(" {before} ").contains(&format!(" {} ", concept_words(negation).join(" ")))
            }) {
                return Some("NegatedDocumentationAction");
            }
            if index == 0 || prefixes.contains(&before) {
                return Some("owned");
            }
        }
    }
    let raw = concept_words(task);
    (mentioned || actions.iter().any(|action| raw.contains(action)))
        .then_some("DocumentationActionUnowned")
}

fn argument_alias(args: &Value, names: &[&str]) -> Option<String> {
    let object = args.as_object()?;
    let values: Vec<_> = names.iter().filter_map(|name| object.get(*name)).collect();
    let first = values.first()?.as_str()?;
    values
        .iter()
        .all(|value| value.as_str() == Some(first))
        .then(|| first.to_owned())
}
fn attempt_path(attempt: &ToolAttempt) -> Option<String> {
    let args: Value = serde_json::from_str(attempt.arguments.as_deref()?).ok()?;
    argument_alias(&args, &["filePath", "file_path", "path"])
}
fn complete(attempt: &ToolAttempt) -> Option<&str> {
    let observed = attempt.source_observation()?;
    let source = observed.source.as_deref()?;
    (attempt.succeeded && observed.error.is_none() && observed.complete && source.len() <= 262_144)
        .then_some(source)
}
fn reads_for<'a>(
    attempts: &'a [ToolAttempt],
    path: &str,
    before: usize,
    after: Option<usize>,
) -> Vec<(usize, &'a ToolAttempt)> {
    attempts
        .iter()
        .enumerate()
        .filter(|(index, attempt)| {
            *index < before
                && after.is_none_or(|after| *index > after)
                && attempt.capability == Capability::Read
                && attempt_path(attempt).as_deref() == Some(path)
        })
        .collect()
}
fn property_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

/// Bind one requested concept to a scalar source field; ambiguity remains unresolved.
pub(super) fn derive_source_documentation(
    task: &str,
    source_path: &str,
    source: &str,
    guide: &str,
    template: &str,
) -> Value {
    let Ok(value) = serde_json::from_str::<Value>(source) else {
        return json!({"gap":"InvalidSourceJson"});
    };
    let Some(object) = value.as_object() else {
        return json!({"gap":"SourceObjectRequired"});
    };
    let requested = concept_words(task);
    let bindings: Vec<_> = object
        .iter()
        .filter(|(key, value)| {
            let words = concept_words(key);
            !words.is_empty()
                && words.iter().all(|word| requested.contains(word))
                && property_text(value).is_some()
        })
        .collect();
    if bindings.len() != 1 {
        return json!({"gap":if bindings.is_empty() {"MissingPropertyBinding"} else {"AmbiguousPropertyBinding"}});
    }
    let (key, value) = bindings[0];
    let text = property_text(value).expect("ScalarBinding");
    if text.contains(['\r', '\n', '`']) || source_path.contains(['\r', '\n', '`']) {
        return json!({"gap":"UnsupportedDocumentationValue"});
    }
    let paragraph = template
        .replace("{source}", source_path)
        .replace("{concept}", &concept_words(key).join(" "))
        .replace("{value}", &text);
    let content = if guide.contains(&paragraph) {
        guide.to_owned()
    } else {
        format!(
            "{}\n\n{paragraph}\n",
            guide.strip_suffix('\n').unwrap_or(guide)
        )
    };
    json!({"key":key,"value":value,"paragraph":paragraph,"sourceIdentity":sha256_hex(source.as_bytes()),"content":content})
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

fn read_step(
    task: &str,
    path: &str,
    tools: &[&str],
    state: &Value,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let Some(read) = tool_for(tools, Capability::Read) else {
        return finish(state, "MissingReadTool", false, result);
    };
    let step = plan_one(read, json!({"filePath":path}).to_string());
    if super::file_read::read_policy_blocks_plan(task, &step) {
        finish(state, "ReadPolicyRefused", false, result)
    } else {
        step
    }
}

/// Same read/candidate/edit/readback and no-op contract as the JavaScript route.
pub(super) fn plan_source_documentation_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: Option<&str>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let parsed = parse_lino(GRAMMAR);
    let root = parsed
        .children
        .iter()
        .find(|node| node.name == "source-documentation")?;
    let values = |name: &str| {
        root.children
            .iter()
            .filter(|node| node.name == name)
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    };
    let template = values("template").into_iter().next()?;
    // Integration exposes this existing helper as pub(super); no duplicate path parser.
    let paths = super::work_item_evidence::work_item_evidence_paths(task);
    let sources: Vec<_> = paths
        .iter()
        .copied()
        .filter(|path| path.ends_with(".json"))
        .collect();
    let guides: Vec<_> = paths
        .iter()
        .copied()
        .filter(|path| path.ends_with(".md"))
        .collect();
    if paths.len() != 2 || sources.len() != 1 || guides.len() != 1 {
        return None;
    }
    let source_path = sources[0];
    let guide_path = guides[0];
    let authority = documentation_authority(
        task,
        guide_path,
        &values("action"),
        &values("preserve"),
        &values("negation"),
        &values("prefix"),
    )?;
    let progress = Progress::scan(messages);
    let attempts = progress.research_attempts();
    let mut requirements: Vec<_>=clauses_with_spans(task).into_iter().map(|(text,(start,end))|json!({"text":text,"span":[start,end],"kind":"source-documentation","status":"unverified"})).collect();
    if requirements.is_empty() {
        requirements.push(json!({"text":task,"span":[0,task.len()],"kind":"source-documentation","status":"unverified"}));
    }
    let mut state = json!({"schema":"source-documentation-controller/v1","task":task,"taskIdentity":sha256_hex(task.as_bytes()),"workItem":work_item,"requirements":requirements,"source":source_path,"target":guide_path,"observations":[],"candidateEdits":[],"checks":[],"effects":[],"remainingWork":["independent-requirement-acceptance"],"authored":false,"verified":false,"complete":false});
    if authority != "owned" {
        return Some(finish(&state, authority, false, result));
    }
    if attempts
        .iter()
        .any(|attempt| attempt.capability == Capability::Read && attempt_path(attempt).is_none())
    {
        return Some(finish(&state, "AmbiguousReadPathAttempt", true, result));
    }
    let mutations: Vec<_> = attempts
        .iter()
        .enumerate()
        .filter(|(_, attempt)| matches!(attempt.capability, Capability::Edit | Capability::Write))
        .collect();
    if mutations.iter().any(|(_, attempt)| {
        attempt.capability != Capability::Edit
            || attempt_path(attempt).as_deref() != Some(guide_path)
    }) {
        return Some(finish(
            &state,
            "UnrelatedOrAmbiguousMutationAttempt",
            true,
            result,
        ));
    }
    if mutations.len() > 1 {
        return Some(finish(&state, "MultipleEditAttempts", true, result));
    }
    for path in [source_path, guide_path] {
        let reads = reads_for(attempts, path, attempts.len(), None);
        let Some((_, observation)) = reads.last() else {
            return Some(read_step(task, path, tools, &state, result));
        };
        let Some(content) = complete(observation) else {
            return Some(finish(
                &state,
                "IncompleteOrOversizedSourceObservation",
                false,
                result,
            ));
        };
        state["observations"].as_array_mut()?.push(
            json!({"path":path,"sourceIdentity":sha256_hex(content.as_bytes()),"complete":true}),
        );
    }
    if let Some((index, edit)) = mutations.first().copied() {
        let source_reads = reads_for(attempts, source_path, index, None);
        let guide_reads = reads_for(attempts, guide_path, index, None);
        let original_source = source_reads
            .iter()
            .find_map(|(_, attempt)| complete(attempt));
        let original_guide = guide_reads
            .last()
            .and_then(|(_, attempt)| complete(attempt));
        let (Some(original_source), Some(original_guide)) = (original_source, original_guide)
        else {
            return Some(finish(
                &state,
                "EditPreconditionObservationMissing",
                true,
                result,
            ));
        };
        let expected = derive_source_documentation(
            task,
            source_path,
            original_source,
            original_guide,
            &template,
        );
        if let Some(gap) = expected.get("gap").and_then(Value::as_str) {
            return Some(finish(&state, gap, true, result));
        }
        let args: Value = serde_json::from_str(edit.arguments.as_deref()?).ok()?;
        let old = argument_alias(&args, &["oldString", "old_string", "old_str", "old"]);
        let next = argument_alias(&args, &["newString", "new_string", "new_str", "new"]);
        state["binding"] = json!({"key":expected["key"],"value":expected["value"],"sourceIdentity":expected["sourceIdentity"]});
        state["candidateEdits"].as_array_mut()?.push(json!({"path":guide_path,"preimageIdentity":sha256_hex(original_guide.as_bytes()),"postimageIdentity":sha256_hex(expected["content"].as_str()?.as_bytes())}));
        if old.as_deref() != Some(original_guide)
            || next.as_deref() != expected["content"].as_str()
            || next == old
        {
            return Some(finish(&state, "UnrelatedEditCandidate", true, result));
        }
        if !edit.succeeded {
            return Some(finish(&state, "CandidateEditFailed", true, result));
        }
        let source_reads = reads_for(attempts, source_path, attempts.len(), Some(index));
        let Some((source_index, source_read)) = source_reads.last().copied() else {
            return Some(read_step(task, source_path, tools, &state, result));
        };
        let observed_source = complete(source_read);
        if observed_source.is_none_or(|source| {
            sha256_hex(source.as_bytes()) != expected["sourceIdentity"].as_str().unwrap_or_default()
        }) {
            return Some(finish(&state, "SourceFreshnessMismatch", true, result));
        }
        let guide_reads = reads_for(attempts, guide_path, attempts.len(), Some(source_index));
        let Some((_, guide_read)) = guide_reads.last() else {
            return Some(read_step(task, guide_path, tools, &state, result));
        };
        let guide = complete(guide_read);
        if guide != expected["content"].as_str() {
            return Some(finish(&state, "ReadbackMismatch", true, result));
        }
        state["checks"] = json!([{"kind":"source-freshness","passed":true},{"kind":"exact-readback","passed":true}]);
        state["effects"] =
            json!([{"path":guide_path,"sourceIdentity":sha256_hex(guide?.as_bytes())}]);
        return Some(finish(
            &state,
            "UsefulCandidateReadbackVerified",
            false,
            result,
        ));
    }
    let source_reads = reads_for(attempts, source_path, attempts.len(), None);
    let guide_reads = reads_for(attempts, guide_path, attempts.len(), None);
    let source = complete(source_reads.last()?.1)?;
    let guide = complete(guide_reads.last()?.1)?;
    let derived = derive_source_documentation(task, source_path, source, guide, &template);
    if let Some(gap) = derived.get("gap").and_then(Value::as_str) {
        return Some(finish(&state, gap, false, result));
    }
    state["binding"] = json!({"key":derived["key"],"value":derived["value"],"sourceIdentity":derived["sourceIdentity"]});
    if derived["content"].as_str()? == guide {
        state["checks"] = json!([{"kind":"source-bound-noop","passed":true}]);
        return Some(finish(&state, "VerifiedNoOp", false, result));
    }
    Some(if let Some(edit) = tool_for(tools, Capability::Edit) {
        plan_one(
            edit,
            json!({"filePath":guide_path,"oldString":guide,"newString":derived["content"]})
                .to_string(),
        )
    } else {
        finish(&state, "MissingEditTool", false, result)
    })
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/source_documentation.rs"]
mod tests;
