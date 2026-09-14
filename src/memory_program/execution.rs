//! Bounded interpreter for compiled memory programs.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{CompiledMemoryProgram, MemoryProgramPermission, MemoryProgramStep};
use crate::engine::stable_id;
use crate::link_store::memory_event_to_link_record;
use crate::links_format::push_lino_node;
use crate::memory::{MemoryEvent, MemoryStore, isoformat_now};

/// The standard three-state permission gate for memory-program effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryProgramAuthorization {
    ReadOnly,
    Write,
    DestructiveConfirmed,
}

/// Why program execution stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryProgramHalt {
    Complete,
    Fixpoint,
    MatchLimit { matched: usize, max_matches: usize },
    IterationLimit { max_iterations: usize },
    PermissionDenied { required: String },
    ProgramGap { primitive: String },
}

/// Immutable result and audit trace of one bounded execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryProgramOutcome {
    pub program_id: String,
    pub matched: usize,
    pub changed: usize,
    pub iterations: usize,
    pub halt: MemoryProgramHalt,
    pub matched_event_ids: Vec<String>,
}

impl MemoryProgramOutcome {
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "memory_program_execution", None);
        push_lino_node(&mut out, 2, "program", Some(&self.program_id));
        let _ = writeln!(out, "  matched {}", self.matched);
        let _ = writeln!(out, "  changed {}", self.changed);
        let _ = writeln!(out, "  iterations {}", self.iterations);
        match &self.halt {
            MemoryProgramHalt::Complete => push_trace_token(&mut out, "halt", "complete"),
            MemoryProgramHalt::Fixpoint => push_trace_token(&mut out, "halt", "fixpoint"),
            MemoryProgramHalt::MatchLimit {
                matched,
                max_matches,
            } => {
                push_trace_token(&mut out, "halt", "match_limit");
                let _ = writeln!(
                    out,
                    "  reason matched {matched} exceeds max_matches {max_matches}"
                );
            }
            MemoryProgramHalt::IterationLimit { max_iterations } => {
                push_trace_token(&mut out, "halt", "iteration_limit");
                let _ = writeln!(out, "  reason max_iterations {max_iterations} reached");
            }
            MemoryProgramHalt::PermissionDenied { required } => {
                push_trace_token(&mut out, "halt", "permission_denied");
                push_lino_node(&mut out, 2, "required", Some(required));
                if required == "destructive" {
                    push_trace_token(
                        &mut out,
                        "policy",
                        "destructive_action_requires_confirmation",
                    );
                }
            }
            MemoryProgramHalt::ProgramGap { primitive } => {
                push_trace_token(&mut out, "halt", "program_gap");
                push_lino_node(&mut out, 2, "primitive", Some(primitive));
            }
        }
        for id in &self.matched_event_ids {
            push_lino_node(&mut out, 2, "matched_event", Some(id));
        }
        out.trim_end().to_owned()
    }
}

fn push_trace_token(out: &mut String, name: &str, value: &str) {
    out.push_str("  ");
    out.push_str(name);
    out.push(' ');
    out.push_str(value);
    out.push('\n');
}

struct InterpreterState {
    selection: Vec<usize>,
    projection: BTreeMap<String, String>,
    matched: usize,
    changed: usize,
    matched_event_ids: BTreeSet<String>,
}

/// Execute a complete program, enforcing its effect permissions and bounds.
#[must_use]
pub fn execute_memory_program(
    program: &CompiledMemoryProgram,
    store: &mut MemoryStore,
    authorization: MemoryProgramAuthorization,
) -> MemoryProgramOutcome {
    if let Some(required) = denied_permission(program, authorization) {
        return outcome(
            program,
            0,
            0,
            0,
            MemoryProgramHalt::PermissionDenied {
                required: required.to_owned(),
            },
            BTreeSet::new(),
        );
    }

    let bounded = program
        .steps
        .iter()
        .any(|step| step.primitive == "bounded_iterate_to_fixpoint");
    let iteration_bound = if bounded {
        program.limits.max_iterations
    } else {
        1
    };
    let mut state = InterpreterState {
        selection: Vec::new(),
        projection: BTreeMap::new(),
        matched: 0,
        changed: 0,
        matched_event_ids: BTreeSet::new(),
    };

    for iteration in 1..=iteration_bound {
        let changes_before = state.changed;
        for step in &program.steps {
            if matches!(
                step.primitive.as_str(),
                "sequential_compose" | "bounded_iterate_to_fixpoint"
            ) {
                continue;
            }
            if step.primitive == "match" {
                state.selection = matched_indices(store, &step.arguments);
                state.projection.clear();
                state.matched = state.matched.max(state.selection.len());
                if state.selection.len() > program.limits.max_matches {
                    return outcome(
                        program,
                        state.selection.len(),
                        state.changed,
                        iteration,
                        MemoryProgramHalt::MatchLimit {
                            matched: state.selection.len(),
                            max_matches: program.limits.max_matches,
                        },
                        state.matched_event_ids,
                    );
                }
                remember_matches(store, &state.selection, &mut state.matched_event_ids);
                continue;
            }
            match execute_step(step, store, &mut state.selection, &mut state.projection) {
                Some(changed) => state.changed += changed,
                None => {
                    return outcome(
                        program,
                        state.matched,
                        state.changed,
                        iteration,
                        MemoryProgramHalt::ProgramGap {
                            primitive: step.primitive.clone(),
                        },
                        state.matched_event_ids,
                    );
                }
            }
        }
        if bounded && state.changed == changes_before {
            return outcome(
                program,
                state.matched,
                state.changed,
                iteration,
                MemoryProgramHalt::Fixpoint,
                state.matched_event_ids,
            );
        }
        if !bounded {
            return outcome(
                program,
                state.matched,
                state.changed,
                iteration,
                MemoryProgramHalt::Complete,
                state.matched_event_ids,
            );
        }
    }
    outcome(
        program,
        state.matched,
        state.changed,
        iteration_bound,
        MemoryProgramHalt::IterationLimit {
            max_iterations: iteration_bound,
        },
        state.matched_event_ids,
    )
}

fn denied_permission(
    program: &CompiledMemoryProgram,
    authorization: MemoryProgramAuthorization,
) -> Option<&'static str> {
    let permissions = program.required_permissions();
    if permissions.contains(&MemoryProgramPermission::Destructive)
        && authorization != MemoryProgramAuthorization::DestructiveConfirmed
    {
        return Some("destructive");
    }
    if permissions.contains(&MemoryProgramPermission::Write)
        && authorization == MemoryProgramAuthorization::ReadOnly
    {
        return Some("write");
    }
    None
}

fn outcome(
    program: &CompiledMemoryProgram,
    matched: usize,
    changed: usize,
    iterations: usize,
    halt: MemoryProgramHalt,
    matched_event_ids: BTreeSet<String>,
) -> MemoryProgramOutcome {
    MemoryProgramOutcome {
        program_id: program.id.clone(),
        matched,
        changed,
        iterations,
        halt,
        matched_event_ids: matched_event_ids.into_iter().collect(),
    }
}

fn execute_step(
    step: &MemoryProgramStep,
    store: &mut MemoryStore,
    selection: &mut Vec<usize>,
    projection: &mut BTreeMap<String, String>,
) -> Option<usize> {
    match step.primitive.as_str() {
        "filter" => {
            filter_selection(store, selection, &step.arguments);
            Some(0)
        }
        "map_matches" => {
            projection.clone_from(&step.arguments);
            Some(0)
        }
        "create" => Some(create_events(store, selection, projection, &step.arguments)),
        "update" => Some(update_events(store, selection, &step.arguments)),
        "delete_with_retraction" => Some(append_retractions(
            store,
            selection,
            step.arguments
                .get("reason")
                .map_or("memory_program", String::as_str),
        )),
        _ => None,
    }
}

fn active_indices(store: &MemoryStore) -> Vec<usize> {
    let retracted = store
        .events()
        .iter()
        .filter(|event| event.kind.as_deref() == Some("memory_retraction"))
        .filter_map(|event| event.inputs.as_deref())
        .collect::<BTreeSet<_>>();
    store
        .events()
        .iter()
        .enumerate()
        .filter(|(index, event)| {
            event.kind.as_deref() != Some("memory_retraction")
                && !retracted.contains(source_id(event, *index).as_str())
        })
        .map(|(index, _)| index)
        .collect()
}

fn matched_indices(store: &MemoryStore, arguments: &BTreeMap<String, String>) -> Vec<usize> {
    active_indices(store)
        .into_iter()
        .filter(|&index| event_matches(&store.events()[index], arguments))
        .collect()
}

fn event_matches(event: &MemoryEvent, arguments: &BTreeMap<String, String>) -> bool {
    arguments.iter().all(|(name, value)| match name.as_str() {
        "contains" => contains_case_insensitive(event_text(event), value),
        "kind" => event.kind.as_deref() == Some(value),
        "sent_at" => event
            .sent_at
            .as_deref()
            .is_some_and(|sent_at| sent_at.starts_with(value)),
        "period" if value == "this_week" => {
            event.sent_at.as_deref().is_some_and(is_in_current_utc_week)
        }
        "field" => event_field(event, value).is_some(),
        _ => false,
    })
}

fn filter_selection(
    store: &MemoryStore,
    selection: &mut Vec<usize>,
    arguments: &BTreeMap<String, String>,
) {
    let duplicate_contents = duplicate_contents(store, selection);
    selection.retain(|&index| {
        let event = &store.events()[index];
        arguments.iter().all(|(name, value)| match name.as_str() {
            "role" => event.role.as_deref() == Some(value),
            "kind" => event.kind.as_deref() == Some(value),
            "missing" => !event_contains_marker(event, &label_marker(value)),
            "duplicate" => value == "true" && duplicate_contents.contains(event_text(event)),
            "links" => value == "none" && event.evidence.is_empty(),
            _ => false,
        })
    });
}

fn create_events(
    store: &mut MemoryStore,
    selection: &[usize],
    projection: &BTreeMap<String, String>,
    arguments: &BTreeMap<String, String>,
) -> usize {
    let kind = arguments
        .get("kind")
        .map_or("memory_program_result", String::as_str);
    if projection
        .get("aggregate")
        .is_some_and(|aggregate| aggregate == "count")
    {
        let group_field = projection.get("group").map_or("topic", String::as_str);
        let mut counts = BTreeMap::<String, usize>::new();
        for &index in selection {
            let event = &store.events()[index];
            let group = match group_field {
                "topic" => event.intent.as_deref().unwrap_or("unclassified"),
                "contributor" => event.role.as_deref().unwrap_or("unknown"),
                _ => "unknown",
            };
            *counts.entry(group.to_owned()).or_default() += 1;
        }
        let content = counts
            .iter()
            .map(|(group, count)| format!("{group}={count}"))
            .collect::<Vec<_>>()
            .join(", ");
        return usize::from(append_derived_event(
            store,
            kind,
            &format!("{kind}:{content}"),
            None,
            None,
            &content,
        ));
    }

    if projection.get("copy").is_some_and(|copy| copy == "true") {
        let collection = arguments.get("collection").map(String::as_str);
        let mut changed = 0;
        for &index in selection {
            let (target, content) = {
                let event = &store.events()[index];
                (
                    source_id(event, index),
                    event
                        .content
                        .clone()
                        .unwrap_or_else(|| event_text(event).to_owned()),
                )
            };
            changed += usize::from(append_derived_event(
                store,
                "collection_member",
                &format!(
                    "collection_member:{}:{target}",
                    collection.unwrap_or_default()
                ),
                Some(&target),
                collection,
                &content,
            ));
        }
        return changed;
    }

    let mut changed = 0;
    for &index in selection {
        let target = source_id(&store.events()[index], index);
        changed += usize::from(append_derived_event(
            store,
            kind,
            &format!("{kind}:{target}"),
            Some(&target),
            None,
            &format!("memory_program_result:{kind}:{target}"),
        ));
    }
    changed
}

fn append_derived_event(
    store: &mut MemoryStore,
    kind: &str,
    identity: &str,
    target: Option<&str>,
    output: Option<&str>,
    content: &str,
) -> bool {
    let id = stable_id("memory_program_result", identity);
    if store.events().iter().any(|event| event.id == id) {
        return false;
    }
    store.append(MemoryEvent {
        id,
        kind: Some(kind.to_owned()),
        inputs: target.map(ToOwned::to_owned),
        outputs: output.map(ToOwned::to_owned),
        content: Some(content.to_owned()),
        sent_at: Some(isoformat_now()),
        evidence: vec![String::from("memory_program")],
        ..MemoryEvent::default()
    });
    true
}

fn update_events(
    store: &mut MemoryStore,
    selection: &[usize],
    arguments: &BTreeMap<String, String>,
) -> usize {
    let mut changed = 0;
    for &index in selection {
        let Some(event) = store.events_mut().get_mut(index) else {
            continue;
        };
        // Each of the four edits reports for itself whether it actually
        // changed anything; the write count moves only if at least one did.
        let replaced = if let (Some(old), Some(new), Some(content)) = (
            arguments.get("old"),
            arguments.get("new"),
            event.content.as_mut(),
        ) && let Some(replaced) = replace_case_insensitive(content, old, new)
        {
            *content = replaced;
            true
        } else {
            false
        };
        let retagged = if let Some(value) = arguments.get("value")
            && event.kind.as_deref() != Some(value)
        {
            event.kind = Some(value.clone());
            true
        } else {
            false
        };
        let appended = if let Some(value) = arguments.get("append") {
            let content = event.content.get_or_insert_with(String::new);
            if content.split_whitespace().any(|part| part == value) {
                false
            } else {
                if !content.is_empty() {
                    content.push(' ');
                }
                content.push_str(value);
                true
            }
        } else {
            false
        };
        let normalized = if arguments
            .get("normalize")
            .is_some_and(|value| value == "whitespace")
            && let Some(content) = event.content.as_mut()
        {
            let squeezed = content.split_whitespace().collect::<Vec<_>>().join(" ");
            let changed = *content != squeezed;
            *content = squeezed;
            changed
        } else {
            false
        };
        if replaced || retagged || appended || normalized {
            event.write_count = event.write_count.max(1).saturating_add(1);
            changed += 1;
        }
    }
    changed
}

fn append_retractions(store: &mut MemoryStore, selection: &[usize], reason: &str) -> usize {
    let targets = selection
        .iter()
        .map(|&index| source_id(&store.events()[index], index))
        .collect::<Vec<_>>();
    let existing = store
        .events()
        .iter()
        .filter(|event| event.kind.as_deref() == Some("memory_retraction"))
        .filter_map(|event| event.inputs.clone())
        .collect::<BTreeSet<_>>();
    let mut changed = 0;
    for target in targets {
        if existing.contains(&target) {
            continue;
        }
        store.append(MemoryEvent {
            id: stable_id("memory_retraction", &format!("{target}:{reason}")),
            kind: Some(String::from("memory_retraction")),
            inputs: Some(target.clone()),
            outputs: Some(reason.to_owned()),
            content: Some(format!("memory_retraction:{target}")),
            sent_at: Some(isoformat_now()),
            evidence: vec![String::from("policy:append_only_retraction")],
            ..MemoryEvent::default()
        });
        changed += 1;
    }
    changed
}

fn remember_matches(store: &MemoryStore, indices: &[usize], output: &mut BTreeSet<String>) {
    for &index in indices {
        output.insert(source_id(&store.events()[index], index));
    }
}

fn source_id(event: &MemoryEvent, index: usize) -> String {
    if event.id.is_empty() {
        memory_event_to_link_record(event, index).source_id
    } else {
        event.id.clone()
    }
}

fn event_text(event: &MemoryEvent) -> &str {
    event
        .content
        .as_deref()
        .or(event.outputs.as_deref())
        .or(event.inputs.as_deref())
        .unwrap_or_default()
}

fn event_field<'a>(event: &'a MemoryEvent, field: &str) -> Option<&'a str> {
    match field {
        "label" | "content" => event.content.as_deref(),
        "inputs" => event.inputs.as_deref(),
        "outputs" => event.outputs.as_deref(),
        _ => None,
    }
}

fn label_marker(value: &str) -> String {
    value.replace('_', ":")
}

fn event_contains_marker(event: &MemoryEvent, marker: &str) -> bool {
    [
        event.content.as_deref(),
        event.inputs.as_deref(),
        event.outputs.as_deref(),
        event.demo_label.as_deref(),
    ]
    .into_iter()
    .flatten()
    .chain(event.evidence.iter().map(String::as_str))
    .any(|value| contains_case_insensitive(value, marker))
}

fn contains_case_insensitive(text: &str, pattern: &str) -> bool {
    text.to_lowercase().contains(&pattern.to_lowercase())
}

fn replace_case_insensitive(text: &str, old: &str, new: &str) -> Option<String> {
    if old.is_empty() {
        return None;
    }
    if text.contains(old) {
        return Some(text.replace(old, new));
    }
    if !old.is_ascii() || !text.is_ascii() {
        return None;
    }
    let lowercase = text.to_ascii_lowercase();
    let old_lowercase = old.to_ascii_lowercase();
    lowercase
        .contains(&old_lowercase)
        .then(|| replace_ascii_case_insensitive(text, &lowercase, &old_lowercase, new))
}

fn replace_ascii_case_insensitive(
    text: &str,
    lowercase: &str,
    old_lowercase: &str,
    new: &str,
) -> String {
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    while let Some(relative) = lowercase[cursor..].find(old_lowercase) {
        let start = cursor + relative;
        out.push_str(&text[cursor..start]);
        out.push_str(new);
        cursor = start + old_lowercase.len();
    }
    out.push_str(&text[cursor..]);
    out
}

fn duplicate_contents<'a>(store: &'a MemoryStore, selection: &[usize]) -> BTreeSet<&'a str> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for &index in selection {
        *counts
            .entry(event_text(&store.events()[index]))
            .or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(content, count)| (count > 1).then_some(content))
        .collect()
}

fn is_in_current_utc_week(timestamp: &str) -> bool {
    let Some(date) = timestamp.get(..10).and_then(parse_iso_date) else {
        return false;
    };
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return false;
    };
    let today = i64::try_from(duration.as_secs() / 86_400).unwrap_or(i64::MAX);
    let weekday_from_monday = (today + 3).rem_euclid(7);
    let week_start = today - weekday_from_monday;
    (week_start..week_start + 7).contains(&date)
}

fn parse_iso_date(date: &str) -> Option<i64> {
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}
