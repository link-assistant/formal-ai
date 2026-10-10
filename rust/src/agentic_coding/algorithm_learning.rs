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
use super::planner::{AgenticPlan, Capability, plan_one, tool_capability, write_arguments};
use super::progress::Progress;
use super::tool_result::{
    StepOutcome, command_argument, observed_bytes_match, observed_payload, step_outcome,
};
use crate::algorithm_discovery::{
    AlgorithmCandidate, AlgorithmDiscoveryRun, ArgumentPattern, ExecutionTrace, TraceStep,
    discover_algorithms, traces_from_memory_events,
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
    let discovery = discovery_command();
    if !progress.has_run(&discovery) {
        return plan_one(run_tool, json!({ "command": discovery }).to_string());
    }
    let mut capture_selected = false;
    if command_payload(messages, &discovery, Some("algorithm-command-receipt/v1")).is_none() {
        if !bare_current_operation(messages, &discovery) {
            return AgenticPlan::Final(result_document(task, "artifact_verification_failed", ""));
        }
        let captured_discovery = capture_command(&discovery, &CaptureOptions::default())
            .expect("compiled algorithm command satisfies capture policy");
        if !progress.has_run(&captured_discovery) {
            return plan_one(
                run_tool,
                json!({ "command": captured_discovery }).to_string(),
            );
        }
        if command_payload(messages, &captured_discovery, None).is_none() {
            return AgenticPlan::Final(result_document(task, "artifact_verification_failed", ""));
        }
        capture_selected = true;
    }
    let readback = if capture_selected {
        capture_command(&readback_command(), &CaptureOptions::default())
            .expect("compiled algorithm command satisfies capture policy")
    } else {
        readback_command()
    };
    if !progress.has_run(&readback) {
        return plan_one(run_tool, json!({ "command": readback }).to_string());
    }
    let verified = command_payload(messages, &readback, Some("algorithm-command-receipt/v1"))
        .and_then(|output| AlgorithmCandidate::from_links_notation(&output).ok());
    if verified.as_ref() != Some(&task.candidate) {
        return AgenticPlan::Final(result_document(task, "artifact_verification_failed", ""));
    }
    let command = if capture_selected {
        capture_command(
            &conformance_command(&task.candidate),
            &CaptureOptions::default(),
        )
        .expect("compiled algorithm command satisfies capture policy")
    } else {
        conformance_command(&task.candidate)
    };
    if !progress.has_run(&command) {
        return plan_one(run_tool, json!({ "command": command }).to_string());
    }
    let expected = expected_conformance(&task.candidate);
    if command_payload(messages, &command, Some("algorithm-command-receipt/v1")).as_deref()
        != Some(expected.as_str())
    {
        return AgenticPlan::Final(result_document(task, "conformance_failed", ""));
    }
    AgenticPlan::Final(result_document(task, "conformance_passed", &expected))
}

/// Decode only a successful complete receipt bound to its current request call.
pub(super) fn command_payload(
    messages: &[ChatMessage],
    command: &str,
    operation_schema: Option<&str>,
) -> Option<String> {
    let start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .unwrap_or(0);
    let current = &messages[start..];
    for (index, message) in current.iter().enumerate().rev() {
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let Some(identity) = message.tool_call_id.as_deref() else {
            continue;
        };
        let Some(call) = current[..index]
            .iter()
            .rev()
            .flat_map(|prior| prior.tool_calls.iter().rev())
            .find(|call| call.id == identity)
        else {
            continue;
        };
        if tool_capability(&call.function.name) != Some(Capability::Run)
            || command_argument(&call.function.arguments).as_deref() != Some(command)
        {
            continue;
        }
        if message.is_error
            || message
                .name
                .as_deref()
                .is_some_and(|name| name != call.function.name)
        {
            return None;
        }
        let raw = message.content.plain_text();
        if let Ok(receipt) = serde_json::from_str::<serde_json::Value>(&raw)
            && receipt
                .get("command")
                .is_some_and(|value| value.as_str() != Some(command))
        {
            return None;
        }
        if let Ok(receipt) = serde_json::from_str::<serde_json::Value>(&raw)
            && operation_schema.is_some_and(|schema| receipt["schema"] == schema)
        {
            return (receipt["command"].as_str() == Some(command)
                && receipt["operation_success"].as_bool() == Some(true)
                && receipt
                    .get("exit_code")
                    .is_some_and(serde_json::Value::is_null)
                && receipt["complete"].as_bool() == Some(true)
                && receipt["truncated"].as_bool() == Some(false)
                && receipt["timed_out"].as_bool() == Some(false)
                && receipt["stderr"].as_str() == Some("")
                && receipt.get("error").is_some_and(serde_json::Value::is_null)
                && !["aborted", "is_error", "isError"]
                    .iter()
                    .any(|key| receipt[*key].as_bool() == Some(true))
                && receipt["stream_complete"].as_bool() != Some(false)
                && receipt.get("signal").is_none_or(serde_json::Value::is_null))
            .then(|| receipt["stdout"].as_str().map(str::to_owned))
            .flatten();
        }
        let payload = observed_payload(&raw)?;
        return observed_bytes_match(&raw, &payload).then_some(payload);
    }
    None
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

/// Bounded capture policy corresponding to captureProgram's JavaScript options.
#[derive(Clone, Copy)]
pub(super) struct CaptureOptions {
    pub timeout: u64,
    pub max_buffer: usize,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        Self {
            timeout: 60_000,
            max_buffer: 8_388_608,
        }
    }
}

/// Exact single-quoted shell operand, corresponding to shellQuote.
pub(super) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Readable Node receipt producer corresponding to captureProgram.
/// Valid UTF-8 strings correspond to the JavaScript Unicode-scalar input domain.
pub(super) fn capture_program(
    command: &str,
    options: &CaptureOptions,
) -> Result<String, &'static str> {
    if command.is_empty()
        || options.timeout == 0
        || options.timeout > 60_000
        || options.max_buffer == 0
        || options.max_buffer > 8_388_608
    {
        return Err("invalid_capture_policy");
    }
    let characters: Vec<char> = command.chars().collect();
    let mut lines = vec![
        "  const { spawnSync } = require('node:child_process');".to_owned(),
        "  const command = [".to_owned(),
    ];
    for chunk in characters.chunks(64) {
        let text: String = chunk.iter().collect();
        let literal = serde_json::to_string(&text).map_err(|_| "invalid_capture_policy")?;
        lines.push(format!("    {literal},"));
    }
    let suffix = [
        "  ].join('');",
        "  const result = spawnSync('/bin/sh', ['-c', command], {",
        "    timeout: TIMEOUT, maxBuffer: MAX_BUFFER,",
        "  });",
        "  const decoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });",
        "  let stdout = '', stderr = '', decodingError = null;",
        "  try { stdout = decoder.decode(result.stdout ?? Buffer.alloc(0)); }",
        "  catch { decodingError = 'InvalidUTF8Stdout'; }",
        "  try { stderr = decoder.decode(result.stderr ?? Buffer.alloc(0)); }",
        "  catch { decodingError ??= 'InvalidUTF8Stderr'; }",
        "  if (decodingError !== null) { stdout = ''; stderr = ''; }",
        "  const receipt = {",
        "    schema: 'command-execution-receipt/v1', stdout, stderr,",
        "    exit_code: result.status, signal: result.signal ?? null,",
        "    complete: !result.error && decodingError === null",
        "      && result.status !== null && result.signal === null,",
        "    truncated: result.error?.code === 'ENOBUFS',",
        "    timed_out: result.error?.code === 'ETIMEDOUT', aborted: false,",
        "    error: decodingError ?? result.error?.message ?? null,",
        "  };",
        "  process.stdout.write(JSON.stringify(receipt));",
        "  process.exitCode = decodingError !== null ? 1",
        "    : Number.isInteger(result.status) ? result.status : 1;",
    ];
    lines.extend(suffix.iter().map(|line| {
        line.replace("TIMEOUT", &options.timeout.to_string())
            .replace("MAX_BUFFER", &options.max_buffer.to_string())
    }));
    Ok(lines.join("\n"))
}

/// Shell request corresponding to captureCommand with the same bounded options.
pub(super) fn capture_command(
    command: &str,
    options: &CaptureOptions,
) -> Result<String, &'static str> {
    Ok(format!(
        "node -e {}",
        shell_quote(&capture_program(command, options)?)
    ))
}

fn bare_current_operation(messages: &[ChatMessage], command: &str) -> bool {
    let start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .unwrap_or(0);
    let current = &messages[start..];
    for (index, message) in current.iter().enumerate().rev() {
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let Some(identity) = message.tool_call_id.as_deref() else {
            continue;
        };
        let Some(call) = current[..index]
            .iter()
            .rev()
            .flat_map(|prior| prior.tool_calls.iter().rev())
            .find(|call| call.id == identity)
        else {
            continue;
        };
        if tool_capability(&call.function.name) != Some(Capability::Run)
            || command_argument(&call.function.arguments).as_deref() != Some(command)
        {
            continue;
        }
        if message.is_error
            || message
                .name
                .as_deref()
                .is_some_and(|name| name != call.function.name)
        {
            return false;
        }
        let raw = message.content.plain_text();
        return !raw.trim().is_empty()
            && step_outcome(&raw) == StepOutcome::Unreported
            && serde_json::from_str::<serde_json::Value>(&raw).is_err();
    }
    false
}
