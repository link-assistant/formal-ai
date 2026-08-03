//! Agent-CLI path for discovering reusable algorithms from supplied traces.
//!
//! Routing is based on the portable `demo_memory` data itself: if the supplied
//! task contains enough observations to yield a held-out-validated episode, the
//! planner runs the public CLI, reads its artifact back, and performs the same
//! side-effect-free conformance check a human can replay.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;

use super::capability_router::tool_for;
use super::driver::DriverOutcome;
use super::planner::{plan_one, write_arguments, AgenticPlan, Capability};
use super::progress::Progress;
use crate::algorithm_discovery::{
    discover_algorithms, traces_from_memory_events, AlgorithmCandidate, AlgorithmDiscoveryRun,
    ArgumentPattern, ExecutionTrace, TraceStep,
};
use crate::links_format::push_lino_node;
use crate::memory::MemoryStore;
use crate::protocol::ChatMessage;

pub const OBSERVATIONS_PATH: &str = "algorithm-observations.lino";
pub const DISCOVERY_PATH: &str = "discovered-algorithms.lino";
pub const CONFORMANCE_TRIGGER: &str = "agent-cli-conformance";

#[derive(Debug, Clone)]
pub struct AlgorithmLearningTask {
    pub observations: String,
    pub discovery: AlgorithmDiscoveryRun,
    pub candidate: AlgorithmCandidate,
}

/// Recognize actual event data, not request wording. This lets translations and
/// novel phrasings take the identical route while prose that merely mentions
/// learning remains unclaimed.
#[must_use]
pub fn compile_task(task: &str) -> Option<AlgorithmLearningTask> {
    let mut store = MemoryStore::new();
    store.replace_from_links_notation(task);
    if store.is_empty() {
        return None;
    }
    let discovery = discover_algorithms(&traces_from_memory_events(store.events()));
    let candidate = discovery.validated_candidates().into_iter().next()?.clone();
    Some(AlgorithmLearningTask {
        observations: store.export_links_notation(),
        discovery,
        candidate,
    })
}

pub(super) fn plan_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    task: &AlgorithmLearningTask,
) -> AgenticPlan {
    let progress = Progress::scan(messages);
    let write_tool = tool_for(tool_names, Capability::Write);
    if let Some(tool) = write_tool.filter(|_| !progress.done(Capability::Write)) {
        return plan_one(tool, write_arguments(OBSERVATIONS_PATH, &task.observations));
    }
    if write_tool.is_none() {
        return AgenticPlan::Final(result_document(task, "observation_write_unavailable", ""));
    }

    let run_tool = tool_for(tool_names, Capability::Run);
    let Some(run_tool) = run_tool else {
        return AgenticPlan::Final(result_document(task, "shell_unavailable", ""));
    };
    match progress.run_outputs.len() {
        0 => plan_one(
            run_tool,
            json!({ "command": discovery_command() }).to_string(),
        ),
        1 => plan_one(
            run_tool,
            json!({ "command": readback_command() }).to_string(),
        ),
        2 => {
            let verified = progress
                .run_outputs
                .get(1)
                .and_then(|output| AlgorithmCandidate::from_links_notation(output).ok());
            if verified.as_ref() != Some(&task.candidate) {
                return AgenticPlan::Final(result_document(
                    task,
                    "artifact_verification_failed",
                    "",
                ));
            }
            plan_one(
                run_tool,
                json!({ "command": conformance_command(&task.candidate) }).to_string(),
            )
        }
        _ => {
            let expected = expected_conformance(&task.candidate);
            let observed = progress.run_outputs.get(2).map(|output| output.trim());
            if observed != Some(expected.trim()) {
                return AgenticPlan::Final(result_document(task, "conformance_failed", ""));
            }
            AgenticPlan::Final(result_document(task, "conformance_passed", &expected))
        }
    }
}

fn discovery_command() -> String {
    [
        "formal-ai",
        "learn",
        "algorithms",
        "--from",
        OBSERVATIONS_PATH,
        "--output",
        DISCOVERY_PATH,
    ]
    .join(" ")
}

fn readback_command() -> String {
    ["cat", DISCOVERY_PATH].join(" ")
}

#[must_use]
pub fn conformance_bindings(candidate: &AlgorithmCandidate) -> BTreeMap<String, String> {
    candidate
        .steps
        .iter()
        .flat_map(|step| step.arguments.values())
        .filter_map(|pattern| match pattern {
            ArgumentPattern::Parameter(name) => Some(name.clone()),
            ArgumentPattern::Constant(_) => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|name| (name, String::from("agent-cli-value")))
        .collect()
}

#[must_use]
pub fn expected_conformance(candidate: &AlgorithmCandidate) -> String {
    candidate
        .conformance_links_notation(CONFORMANCE_TRIGGER, &conformance_bindings(candidate))
        .expect("the generated bindings cover every inferred parameter")
}

#[must_use]
pub fn conformance_command(candidate: &AlgorithmCandidate) -> String {
    let mut parts = vec![
        String::from("formal-ai"),
        String::from("algorithm"),
        String::from("conformance"),
        String::from("--artifact"),
        String::from(DISCOVERY_PATH),
        String::from("--trigger"),
        String::from(CONFORMANCE_TRIGGER),
    ];
    for (name, value) in conformance_bindings(candidate) {
        parts.push(String::from("--binding"));
        parts.push(format!("{name}={value}"));
    }
    parts.join(" ")
}

/// Project an executed Agent-CLI transcript into the shared observation model.
///
/// This closes the self-learning loop without treating tool output as an
/// instruction: only requested tool names and their structured inputs are
/// mined.
#[must_use]
pub fn trace_from_driver_outcome(id: impl Into<String>, outcome: &DriverOutcome) -> ExecutionTrace {
    ExecutionTrace::new(
        id,
        outcome
            .steps
            .iter()
            .map(|step| {
                let arguments = serde_json::from_str::<serde_json::Value>(&step.arguments)
                    .ok()
                    .and_then(|value| value.as_object().cloned())
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(name, value)| {
                        let value = value
                            .as_str()
                            .map_or_else(|| value.to_string(), ToOwned::to_owned);
                        (name, value)
                    });
                TraceStep::new(&step.tool).with_arguments(arguments)
            })
            .collect(),
    )
}

fn result_document(task: &AlgorithmLearningTask, status: &str, conformance: &str) -> String {
    let mut output = String::new();
    push_lino_node(
        &mut output,
        0,
        "agent_algorithm_learning",
        Some(&task.candidate.id),
    );
    push_lino_node(&mut output, 2, "status", Some(status));
    push_lino_node(&mut output, 2, "observations", Some(OBSERVATIONS_PATH));
    push_lino_node(&mut output, 2, "artifact", Some(DISCOVERY_PATH));
    push_lino_node(&mut output, 2, "human_gated", Some("true"));
    push_lino_node(
        &mut output,
        2,
        "execution_mode",
        Some("proposal_conformance"),
    );
    push_lino_node(
        &mut output,
        2,
        "held_out_tests",
        Some(&task.candidate.held_out.len().to_string()),
    );
    push_lino_node(
        &mut output,
        2,
        "associative_compression_lossless",
        Some(if task.discovery.associative_compression_lossless {
            "true"
        } else {
            "false"
        }),
    );
    if !conformance.is_empty() {
        push_lino_node(&mut output, 2, "conformance", Some(conformance));
    }
    output
}
