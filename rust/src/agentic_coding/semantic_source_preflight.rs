//! Bounded lexical source discovery; observations never certify semantic ownership.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use super::tool_result::{SourceReadStatus, incomplete_receipt};
use super::wording::agentic_message;
use crate::protocol::{ChatMessage, client_working_directory};
use crate::source_fetch::sha256_hex;
use regex::Regex;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Exact request spans own lexical candidates, never implementation bindings.
#[must_use]
pub fn semantic_source_candidates(task: &str) -> Vec<Value> {
    if task.len() > 16_384 {
        return Vec::new();
    }
    let token =
        Regex::new(r"(?-u:\b)[A-Za-z_$][A-Za-z0-9_$]*(?-u:\b)").expect("StaticIdentifierPattern");
    let underscore =
        Regex::new(r"^[A-Za-z][A-Za-z0-9]*_[A-Za-z0-9_]+$").expect("StaticUnderscorePattern");
    let camel = Regex::new(r"[a-z][A-Z]").expect("StaticCamelPattern");
    let mut seen = BTreeSet::new();
    token.find_iter(task).filter_map(|matched| {
        let text = matched.as_str();
        ((underscore.is_match(text) || camel.is_match(text)) && seen.insert(text)).then(|| {
            json!({"text":text,"span":[matched.start(),matched.end()],"authority":"lexical-candidate-only"})
        })
    }).take(8).collect()
}

fn finish(
    task: &str,
    candidates: &[Value],
    observations: &[Value],
    reason: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let clauses: Vec<_> = crate::obligation_ledger::clauses_with_spans(task)
        .into_iter()
        .map(|(text, (start, end))| json!({"text":text,"span":[start,end],"status":"unbound"}))
        .collect();
    let discovery = json!({"reason":reason,"taskIdentity":sha256_hex(task.as_bytes()),"clauses":clauses,
        "candidates":candidates,"observations":observations,"inferredWork":"mechanical-source-discovery",
        "coverage":"bounded-candidates-and-paths-not-exhaustive","implementation":"Unknown","validation":"Unknown","authored":false,"verified":false});
    let payload = discovery.to_string();
    record(
        AgenticPlan::Final(agentic_message(
            "semantic_source_preflight_gap_output",
            &[("discovery", &payload)],
        )),
        FinalDisposition::Gap,
        "semantic-source-preflight-gap",
        result,
    )
}

fn source_status(status: SourceReadStatus) -> &'static str {
    match status {
        SourceReadStatus::ReportedSuccess => "reported-success",
        SourceReadStatus::ReportedFailure => "reported-failure",
        SourceReadStatus::Unknown => "unknown",
    }
}

/// Real current-window Grep and Read prerequisites end in an explicit semantic gap.
pub(super) fn plan_semantic_source_preflight(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let candidates = semantic_source_candidates(task);
    if candidates.is_empty() {
        return None;
    }
    let progress = Progress::scan(messages);
    let mut observations = Vec::new();
    let mut paths: Vec<String> = Vec::new();
    let path_pattern = Regex::new(r"^[A-Za-z0-9_./-]+$").expect("StaticRelativePathPattern");
    for candidate in &candidates {
        let text = candidate["text"].as_str()?;
        let arguments = json!({"path":".","pattern":format!(r"\b{}\b",regex::escape(text))});
        let Some(observed) = progress.latest_attempt_for(Capability::Grep, &arguments) else {
            return Some(tool_for(tools, Capability::Grep).map_or_else(
                || finish(task, &candidates, &observations, "MissingGrepTool", result),
                |tool| plan_one(tool, arguments.to_string()),
            ));
        };
        let complete = observed.succeeded
            && observed.detail.len() <= 65_536
            && !incomplete_receipt(&observed.detail)
            && !observed.detail.contains("Results truncated")
            && !observed.detail.contains("preview truncated");
        observations.push(json!({"candidate":candidate,"arguments":arguments,"providerSucceeded":observed.succeeded,"complete":complete,"detail":observed.detail}));
        if !complete {
            return Some(finish(
                task,
                &candidates,
                &observations,
                "IncompleteSourceSearch",
                result,
            ));
        }
        let root = client_working_directory(messages);
        for line in observed.detail.lines() {
            if !line.ends_with(':')
                || line
                    .chars()
                    .next()
                    .is_some_and(crate::meta_reasoner::text::is_js_whitespace)
            {
                continue;
            }
            let mut path = &line[..line.len() - 1];
            if let Some(root) = root.as_deref() {
                path = path.strip_prefix(&format!("{root}/")).unwrap_or(path);
            }
            path = path.strip_prefix("./").unwrap_or(path);
            if !path_pattern.is_match(path)
                || path.starts_with('/')
                || path.split('/').any(|part| matches!(part, "" | "." | ".."))
            {
                continue;
            }
            if paths.len() < 8 && !paths.iter().any(|known| known == path) {
                paths.push(path.to_owned());
            }
        }
    }
    for path in paths {
        let Some(observed) = progress.source_read_for(&path) else {
            return Some(tool_for(tools, Capability::Read).map_or_else(
                || finish(task, &candidates, &observations, "MissingReadTool", result),
                |tool| plan_one(tool, super::workspace_change::read_arguments(&path)),
            ));
        };
        observations.push(json!({"path":path,"sourceIdentity":observed.source.as_ref().map(|source|sha256_hex(source.as_bytes())),
            "complete":observed.complete,"status":source_status(observed.status),"error":observed.error,
            "bytes":observed.source.as_ref().map(String::len)}));
        if observed.error.is_some() || !observed.complete {
            return Some(finish(
                task,
                &candidates,
                &observations,
                "IncompleteCandidateSource",
                result,
            ));
        }
    }
    Some(finish(
        task,
        &candidates,
        &observations,
        "SourceBoundImplementationAndIndependentValidationMissing",
        result,
    ))
}
