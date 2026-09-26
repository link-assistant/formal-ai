//! Adapters from Formal AI's existing ordered-data surfaces.

use std::collections::BTreeMap;

use super::{ExecutionTrace, TraceStep};
use crate::event_log::EventLog;
use crate::memory::MemoryEvent;
use crate::skill_procedure::CompiledProcedure;

/// Adapt the universal solver's event log to the common trace model.
#[must_use]
pub fn trace_from_event_log(id: impl Into<String>, log: &EventLog) -> ExecutionTrace {
    ExecutionTrace::new(
        id,
        log.events()
            .iter()
            .map(|event| {
                TraceStep::new(event.kind).with_arguments([("payload", event.payload.as_str())])
            })
            .collect(),
    )
}

/// Project a compiled natural-language guide onto the common trace model. A
/// collection of independently compiled guides can therefore be mined with the
/// same held-out protocol as runtime observations.
#[must_use]
pub fn trace_from_compiled_procedure(procedure: &CompiledProcedure) -> ExecutionTrace {
    ExecutionTrace::new(
        procedure.id.clone(),
        procedure
            .steps
            .iter()
            .map(|step| {
                let mut arguments = step
                    .objects
                    .iter()
                    .enumerate()
                    .map(|(index, object)| (format!("object_{}", index + 1), object.clone()))
                    .collect::<BTreeMap<_, _>>();
                if let Some(language) = &step.target_language {
                    arguments.insert(String::from("target_language"), language.clone());
                }
                TraceStep::new(&step.kind).with_arguments(arguments)
            })
            .collect(),
    )
}

/// Group portable memory events by conversation and project tool invocations
/// (or, when absent, event kinds) into execution traces.
#[must_use]
pub fn traces_from_memory_events(events: &[MemoryEvent]) -> Vec<ExecutionTrace> {
    let mut grouped: BTreeMap<String, Vec<TraceStep>> = BTreeMap::new();
    for event in events {
        if event.kind.as_deref() == Some("algorithm_learning_candidate") {
            continue;
        }
        let Some(operation) = event.tool.as_deref().or(event.kind.as_deref()) else {
            continue;
        };
        let id = event
            .conversation_id
            .clone()
            .unwrap_or_else(|| String::from("ungrouped"));
        let arguments = event
            .inputs
            .as_deref()
            .map(parse_arguments)
            .unwrap_or_default();
        grouped
            .entry(id)
            .or_default()
            .push(TraceStep::new(operation).with_arguments(arguments));
    }
    grouped
        .into_iter()
        .map(|(id, steps)| ExecutionTrace::new(id, steps))
        .collect()
}

fn parse_arguments(input: &str) -> BTreeMap<String, String> {
    if let Ok(serde_json::Value::Object(object)) = serde_json::from_str(input) {
        return object
            .into_iter()
            .map(|(key, value)| {
                let value = match value {
                    serde_json::Value::String(value) => value,
                    other => other.to_string(),
                };
                (key, value)
            })
            .collect();
    }
    input
        .split([';', ','])
        .filter_map(|part| part.trim().split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .filter(|(key, _)| !key.is_empty())
        .collect()
}
