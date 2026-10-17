//! Request/source observations around the shared generated maintenance decisions.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::{Progress, ToolAttempt};
use super::shell_command_policy::sentences;
use super::source_maintenance_observation::derive_source_maintenance;
use super::tool_result::SourceReadStatus;
use super::workspace_line_operation::named_target_and_payloads;
use crate::coding_research_generated::{self as generated, SourceMaintenanceDecision};
use crate::protocol::ChatMessage;
use crate::seed::parser::{LinoNode, parse_lino};
use regex::Regex;
use serde_json::{Value, json};

const GRAMMAR: &str = include_str!("../../embedded/data/meta/source-maintenance-grammar.lino");

pub(super) struct SourceMaintenanceRequest {
    target: String,
    language: String,
}

fn scalar_values(text: &str) -> Vec<String> {
    text.chars()
        .map(|character| character.to_string())
        .collect()
}

fn normalized(text: &str) -> String {
    generated::source_maintenance_normalize_cue_kernel(scalar_values(text))
}

fn unquoted_edge(text: &str, leading: bool) -> &str {
    let character = if leading {
        text.chars().next()
    } else {
        text.chars().next_back()
    };
    if let Some(character @ ('\'' | '"' | '`')) = character {
        if leading {
            &text[character.len_utf8()..]
        } else {
            &text[..text.len() - character.len_utf8()]
        }
    } else {
        text
    }
}

fn grammar_values(rules: &[LinoNode], name: &str) -> Vec<String> {
    rules
        .iter()
        .filter(|node| node.name == name)
        .map(|node| node.id.clone())
        .collect()
}

fn constraint_observations(rules: &[LinoNode], clauses: &[&str]) -> (bool, Vec<bool>) {
    let patterns = rules
        .iter()
        .filter(|node| node.name == "constraint")
        .map(|node| node.find_child_value("pattern"))
        .map(|pattern| Regex::new(&format!("^(?:{pattern})$")))
        .collect::<Result<Vec<_>, _>>();
    let Ok(patterns) = patterns else {
        return (false, vec![false]);
    };
    let observations = clauses
        .iter()
        .skip(1)
        .map(|clause| {
            let text = normalized(clause);
            patterns.iter().any(|pattern| pattern.is_match(&text))
        })
        .collect();
    (true, observations)
}

/// The same generated ownership kernel used by the JavaScript production route.
pub(super) fn owned_source_maintenance_request(task: &str) -> Option<SourceMaintenanceRequest> {
    let (target, payloads) = named_target_and_payloads(task)?;
    let sentences = sentences(task);
    let clauses = sentences
        .iter()
        .flat_map(|sentence| sentence.text.split(';'))
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .collect::<Vec<_>>();
    let first = *clauses.first()?;
    let position = first.find(&target)?;
    let prefix = normalized(unquoted_edge(&first[..position], false));
    let suffix = normalized(unquoted_edge(&first[position + target.len()..], true));
    let grammar = parse_lino(GRAMMAR);
    let root = grammar
        .children
        .iter()
        .find(|node| node.name == "source-maintenance-grammar")?;
    let (valid, constraints) = constraint_observations(&root.children, &clauses);
    let language = generated::source_maintenance_language_kernel(scalar_values(&target));
    let payload_count = payloads.len().to_string().parse::<f64>().ok()?;
    let owned = generated::source_maintenance_ownership_kernel(
        true,
        payload_count,
        language.clone(),
        prefix,
        suffix,
        valid,
        grammar_values(&root.children, "action"),
        grammar_values(&root.children, "property"),
        grammar_values(&root.children, "preposition"),
        grammar_values(&root.children, "scope"),
        constraints,
    );
    owned.then_some(SourceMaintenanceRequest { target, language })
}

fn matching_arguments(
    attempt: &ToolAttempt,
    target: &str,
    capability: Capability,
) -> Option<Value> {
    if attempt.capability != capability {
        return None;
    }
    let arguments: Value = serde_json::from_str(attempt.arguments.as_deref()?).ok()?;
    (arguments.get("filePath").and_then(Value::as_str) == Some(target)).then_some(arguments)
}

fn refusal(reason: String, failure: bool, result: &mut Option<FinalResult>) -> AgenticPlan {
    let disposition = if failure {
        FinalDisposition::Failure
    } else {
        FinalDisposition::Gap
    };
    record(
        AgenticPlan::Final(reason),
        disposition,
        "source-maintenance-observed",
        result,
    )
}

fn finding(
    target: &str,
    source: &str,
    edited: bool,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let evidence = json!({"sourceMaintenance":{"path":target,"property":"trailing-horizontal-whitespace",
        "verified":true,"written":edited,"sha256":crate::source_fetch::sha256_hex(source.as_bytes())}});
    record(
        AgenticPlan::Final(evidence.to_string()),
        FinalDisposition::Finding,
        "source-maintenance-observed",
        result,
    )
}

/// Bound provider observations feed canonical transitions; only selected effects become tool calls.
pub(super) fn plan_source_maintenance_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let contract = owned_source_maintenance_request(task)?;
    let progress = Progress::scan(messages);
    let read = tool_for(tools, Capability::Read);
    let edit = tool_for(tools, Capability::Edit);
    let observed = progress.source_read_for(&contract.target);
    let attempts = progress.research_attempts();
    let edits = attempts
        .iter()
        .enumerate()
        .filter_map(|(index, attempt)| {
            matching_arguments(attempt, &contract.target, Capability::Edit)
                .map(|arguments| (index, attempt, arguments))
        })
        .collect::<Vec<_>>();
    let latest = edits.last();
    let complete = observed.as_ref().is_some_and(|value| {
        value.status == SourceReadStatus::ReportedSuccess
            && value.complete
            && !value.absent
            && value.source.is_some()
    });
    let readback_seen = latest.is_some_and(|(index, _, _)| {
        attempts.iter().skip(index + 1).any(|attempt| {
            matching_arguments(attempt, &contract.target, Capability::Read).is_some()
        })
    });
    let source = observed
        .as_ref()
        .and_then(|value| value.source.as_deref())
        .unwrap_or_default();
    let readback_matches = latest.is_some_and(|(_, _, arguments)| {
        arguments.get("newString").and_then(Value::as_str) == Some(source)
    });
    let stage = generated::source_maintenance_stage_kernel(
        read.is_some(),
        observed.is_some(),
        complete,
        edits.iter().any(|(_, attempt, _)| !attempt.succeeded),
        latest.is_some(),
        readback_seen,
        readback_matches,
    );
    match stage {
        SourceMaintenanceDecision::Refused(reason, failure) => {
            return Some(refusal(reason, failure, result));
        }
        SourceMaintenanceDecision::Read => {
            return Some(plan_one(
                read?,
                json!({"filePath":contract.target}).to_string(),
            ));
        }
        SourceMaintenanceDecision::Derive => {}
        _ => return None,
    }
    let derived =
        derive_source_maintenance("trailing-horizontal-whitespace", &contract.language, source);
    let gap = derived.as_ref().err().cloned().unwrap_or_default();
    let same = derived.as_ref().is_ok_and(|value| value == source);
    let decision =
        generated::source_maintenance_effect_kernel(gap, same, latest.is_some(), edit.is_some());
    match decision {
        SourceMaintenanceDecision::Refused(reason, failure) => {
            Some(refusal(reason, failure, result))
        }
        SourceMaintenanceDecision::Finding => {
            Some(finding(&contract.target, source, latest.is_some(), result))
        }
        SourceMaintenanceDecision::Edit => Some(plan_one(
            edit?,
            json!({"filePath":contract.target,
            "oldString":source,"newString":derived.ok()?})
            .to_string(),
        )),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/fixtures/source-maintenance/native-route.rs"]
mod native_route_tests;
