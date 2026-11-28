//! Numbered source requirements used as the work for a verified delivery.

use std::sync::LazyLock;

use regex::Regex;

use super::code_task::render_seeded_change;
use super::final_result::{FinalDisposition, FinalResult, ResolvedPlan, record};
use super::planner::AgenticPlan;
use super::request_sequence::plan_bound_request_steps;
use super::requirement_resolution::{RequirementTarget, resolve_requirement_targets};
use super::structured_edit::{plan_declared_member_change, plan_seed_literal_change};
use super::write_request::{bare_surfaces, safe_relative_path};
use crate::normal_markov::quoted_segment_spans;
use crate::protocol::{ChatMessage, latest_user_request};
use crate::seed;

const MAXIMUM_REQUIREMENTS: usize = 64;
const MAXIMUM_PARAGRAPH_LENGTH: usize = 8192;
const SOURCE_CONTRACT_MISSING: &str = "coding-source-authoring-contract-missing";

static PARAGRAPH_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\r?\n[^\S\r\n]*\r?\n").expect("valid paragraph pattern"));
static CLAUSE_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|\s)\(([1-9][0-9]?)\)").expect("valid requirement marker pattern")
});

/// Read only the opening instruction paragraph, outside quoted payloads.
fn numbered_clauses(task: &str) -> Option<Vec<(usize, &str)>> {
    let task = task.trim_start();
    let head = PARAGRAPH_END.splitn(task, 2).next()?;
    if head.len() > MAXIMUM_PARAGRAPH_LENGTH {
        return None;
    }
    let quotes = quoted_segment_spans(head);
    let markers: Vec<_> = CLAUSE_MARKER
        .captures_iter(head)
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            let number = captures.get(1)?;
            let opener = number.start() - 1;
            (head[whole.end()..].starts_with(char::is_whitespace)
                && !quotes
                    .iter()
                    .any(|quote| (quote.start..quote.end).contains(&opener)))
            .then_some((whole.start(), whole.end(), number.as_str()))
        })
        .collect();
    let [(start, _, _), ..] = markers.as_slice() else {
        return None;
    };
    let lead = head[..*start].trim();
    if !(2..=MAXIMUM_REQUIREMENTS).contains(&markers.len())
        || (!lead.is_empty() && !lead.ends_with(':'))
    {
        return None;
    }
    markers
        .iter()
        .enumerate()
        .map(|(index, (_, end, number))| {
            let next = markers.get(index + 1).map_or(head.len(), |marker| marker.0);
            Some((number.parse().ok()?, head[*end..next].trim()))
        })
        .collect()
}

enum Change {
    Member(String),
    SeedLiteral { old: String, next: String },
}

fn literal(text: &str, spaces: bool) -> bool {
    (1..=96).contains(&text.len())
        && text.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || b"_-".contains(&byte) || (spaces && byte == b' ')
        })
}

fn source_clause(clause: &str) -> Option<(String, Change)> {
    let words: Vec<_> = clause
        .strip_suffix('.')
        .unwrap_or(clause)
        .split_whitespace()
        .collect();
    let [verb, value, relation, rest @ ..] = words.as_slice() else {
        return None;
    };
    let action = verb.to_lowercase();
    let relation = relation.to_lowercase();
    let new_leads = bare_surfaces(seed::ROLE_FILE_EDIT_NEW_LEAD_CUE);
    if bare_surfaces(seed::ROLE_CODING_MEMBER_ADD_ACTION).contains(&action)
        && bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE).contains(&action)
        && literal(value, false)
        && (new_leads.contains(&relation)
            || bare_surfaces(seed::ROLE_FILE_WRITE_TARGET_CUE).contains(&relation))
    {
        return Some((rest.join(" "), Change::Member((*value).to_owned())));
    }
    if !bare_surfaces(seed::ROLE_FILE_EDIT_ACTION_CUE).contains(&action) {
        return None;
    }
    let old_leads = bare_surfaces("file_edit_old_lead_cue");
    let from = words
        .iter()
        .enumerate()
        .skip(2)
        .find(|(_, word)| old_leads.contains(&word.to_lowercase()))?
        .0;
    let to = words
        .iter()
        .rposition(|word| new_leads.contains(&word.to_lowercase()))?;
    if to <= from + 1 {
        return None;
    }
    let old = words[from + 1..to].join(" ");
    let next = words[to + 1..].join(" ");
    if old == next || !literal(&old, true) || !literal(&next, true) {
        return None;
    }
    Some((words[1..from].join(" "), Change::SeedLiteral { old, next }))
}

fn bind_source_clause(
    descriptor: &str,
    change: &Change,
    target: RequirementTarget,
) -> Option<(String, String)> {
    if !matches!(target.kind.as_str(), "const" | "static") {
        return None;
    }
    let path = match change {
        Change::Member(_) => {
            let relative = target.module_path.strip_prefix("src/")?;
            let path = format!("rust/src/{relative}");
            (path.ends_with(".rs") && safe_relative_path(&path)).then_some(path)?
        }
        Change::SeedLiteral { .. } => {
            if target.module_path != "src/seed/embedded_registry.rs"
                || !descriptor
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                    .any(|token| token == target.symbol)
            {
                return None;
            }
            let seeds = seed::seed_files();
            let mut matching = seeds.into_iter().filter_map(|(path, _)| {
                let stem = path.rsplit('/').next()?.strip_suffix(".lino")?;
                (format!("{}_LINO", stem.to_ascii_uppercase().replace('-', "_")) == target.symbol)
                    .then_some(path)
            });
            let path = matching.next()?;
            if matching.next().is_some() {
                return None;
            }
            path.to_owned()
        }
    };
    Some((path, target.symbol))
}

fn gap(task: &str, result: &mut Option<FinalResult>) -> Option<AgenticPlan> {
    Some(record(
        AgenticPlan::Final(render_seeded_change(
            SOURCE_CONTRACT_MISSING,
            task,
            "",
            &[],
        )?),
        FinalDisposition::Gap,
        SOURCE_CONTRACT_MISSING,
        result,
    ))
}

pub(super) fn plan_numbered_source_requirements(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<ResolvedPlan> {
    let clauses = numbered_clauses(task)?;
    let inputs: Vec<_> = clauses
        .iter()
        .map(|(_, clause)| source_clause(clause))
        .collect();
    if inputs.iter().all(Option::is_none) {
        return None;
    }
    let targets = resolve_requirement_targets(
        inputs
            .iter()
            .map(|input| input.as_ref().map(|(descriptor, _)| descriptor.as_str())),
    );
    let bound_inputs: Vec<_> = inputs
        .into_iter()
        .zip(targets)
        .map(|(input, target)| {
            let (descriptor, change) = input?;
            Some((bind_source_clause(&descriptor, &change, target?)?, change))
        })
        .collect();
    if bound_inputs.iter().all(Option::is_none) {
        return None;
    }
    let mut result = None;
    let mut texts = Vec::with_capacity(clauses.len());
    let mut bound = Vec::with_capacity(texts.capacity());
    for ((number, text), target) in clauses.into_iter().zip(bound_inputs) {
        let Some(target) =
            target.filter(|_| number == texts.len() + 1 && !texts.iter().any(|part| part == text))
        else {
            let plan = gap(task, &mut result)?;
            return Some(ResolvedPlan::new(plan, result));
        };
        texts.push(text.to_owned());
        bound.push(target);
    }
    let plan = plan_bound_request_steps(
        &texts,
        messages,
        tool_names,
        |own, tools| {
            let part = latest_user_request(own)?;
            let index = texts.iter().position(|text| *text == part)?;
            let ((path, declaration), change) = &bound[index];
            let mut outcome = None;
            let plan = match change {
                Change::Member(value) => plan_declared_member_change(
                    &part,
                    own,
                    tools,
                    path,
                    declaration,
                    value,
                    &mut outcome,
                ),
                Change::SeedLiteral { old, next } => {
                    plan_seed_literal_change(&part, own, tools, path, old, next, &mut outcome)
                }
            }
            .or_else(|| gap(&part, &mut outcome))?;
            Some(ResolvedPlan::new(plan, outcome))
        },
        &mut result,
    )?;
    Some(ResolvedPlan::new(plan, result))
}
