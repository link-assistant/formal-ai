//! Installed callback-value operation; source text never supplies a command capability.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::installed_source_commands::installed_source_command;
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::{Progress, ToolAttempt};
use super::tool_result::{
    SourceReadStatus, StepOutcome, incomplete_receipt, observed_payload, step_outcome,
};
use super::wording::agentic_message;
use crate::portable_string_contracts::quote_shell_word as quote;
use crate::protocol::ChatMessage;
use crate::source_fetch::sha256_hex;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};

/// The finite complete request owns a source path and callback-value operation only.
#[must_use]
pub fn source_unary_consolidation_request(task: &str) -> Option<Value> {
    if task.encode_utf16().count() > 4096 {
        return None;
    }
    let pattern=Regex::new(r"(?i)^Consolidate[ ]+(?:the[ ]+)?stateless[ ]+unary[ ]+callbacks[ ]+in[ ]+`?(js/agentic/[A-Za-z0-9_./-]+\.mjs)`?[ ]+(?:using|with)[ ]+(?:the[ ]+)?(?:shared[ ]+)?canonical[ ]+helpers\.[ ]+Preserve[ ]+callback[ ]+value[ ]+behavior[ ]+and[ ]+comments\.?$").ok()?;
    let normalized: String = task
        .trim_matches(crate::meta_reasoner::text::is_js_whitespace)
        .chars()
        .map(|character| {
            if crate::meta_reasoner::text::is_js_whitespace(character) {
                ' '
            } else {
                character
            }
        })
        .collect();
    let captures = pattern.captures(&normalized)?;
    let owner = captures.get(1)?.as_str();
    if owner.split('/').any(|part| matches!(part, "" | "." | "..")) {
        return None;
    }
    Some(
        json!({"owner":owner,"operation":"stateless-unary-callback-values","reflection":"not-preserved","independentBehaviorValidation":"required"}),
    )
}

fn gap(key: &str, result: &mut Option<FinalResult>) -> AgenticPlan {
    record(
        AgenticPlan::Final(agentic_message(key, &[])),
        FinalDisposition::Gap,
        "source-unary-consolidation-gap",
        result,
    )
}

fn exact_owner_read(attempt: &ToolAttempt, owner: &str) -> bool {
    if attempt.capability != Capability::Read {
        return false;
    }
    attempt
        .arguments
        .as_deref()
        .and_then(|arguments| serde_json::from_str::<Value>(arguments).ok())
        .is_some_and(|arguments| {
            arguments["path"] == owner
                && arguments["filePath"] == owner
                && arguments["file_path"] == owner
        })
}
fn exact_command(attempt: &ToolAttempt, command: &str) -> bool {
    attempt.capability == Capability::Run
        && attempt
            .arguments
            .as_deref()
            .and_then(|arguments| serde_json::from_str::<Value>(arguments).ok())
            .is_some_and(|arguments| arguments["command"] == command)
}

// A typed complete producer record rejects duplicate fields and extra top-level values.
#[derive(Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    schema: String,
    request: String,
    owner: String,
    before_sha256: String,
    after_sha256: String,
    files: u64,
    callbacks: u64,
    requires_independent_consumer_checks: bool,
    complete_semantic_equivalence: bool,
    independent_behavior_validation: String,
    autonomous_authorship_credit: u64,
}
fn receipt(payload: &str, task: &str, owner: &str, before: &str) -> Option<Receipt> {
    let parsed: Receipt = serde_json::from_str(payload).ok()?;
    if serde_json::to_string(&parsed).ok()?.as_str()
        != payload.trim_matches(crate::meta_reasoner::text::is_js_whitespace)
    {
        return None;
    }
    let hash = Regex::new(r"^[a-f0-9]{64}$").ok()?;
    (parsed.schema == "stateless-unary-request/v1"
        && parsed.request == task
        && parsed.owner == owner
        && parsed.before_sha256 == before
        && parsed.autonomous_authorship_credit == 0
        && !parsed.complete_semantic_equivalence
        && parsed.independent_behavior_validation == "pending"
        && parsed.callbacks <= 100_000
        && parsed.files <= 1
        && parsed.requires_independent_consumer_checks
        && hash.is_match(&parsed.after_sha256))
    .then_some(parsed)
}

/// A configured installed capability, exact preimage and fresh physical readback gate a finding.
/// The finding never certifies full consolidation coverage or semantic equivalence.
pub(super) fn plan_source_unary_consolidation_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let request = source_unary_consolidation_request(task)?;
    let owner = request["owner"].as_str()?;
    let Some(prefix) = installed_source_command("unary-consolidation") else {
        return Some(gap("source_unary_gap_1", result));
    };
    let progress = Progress::scan(messages);
    let read_tool = tool_for(tools, Capability::Read);
    let Some(read) = progress.source_read_for(owner) else {
        return Some(read_tool.map_or_else(
            || gap("source_unary_gap_2", result),
            |tool| plan_one(tool, super::workspace_change::read_arguments(owner)),
        ));
    };
    if read.status != SourceReadStatus::ReportedSuccess
        || !read.complete
        || read.absent
        || read.source.is_none()
    {
        return Some(gap("source_unary_gap_3", result));
    }
    let attempts = progress.research_attempts();
    let Some(source_attempt) = attempts
        .iter()
        .find(|attempt| exact_owner_read(attempt, owner) && attempt.source_observation().is_some())
    else {
        return Some(gap("source_unary_gap_4", result));
    };
    let Some(preimage) = source_attempt.source_observation() else {
        return Some(gap("source_unary_gap_4", result));
    };
    if preimage.status != SourceReadStatus::ReportedSuccess || !preimage.complete {
        return Some(gap("source_unary_gap_4", result));
    }
    let Some(source) = preimage.source.as_deref() else {
        return Some(gap("source_unary_gap_4", result));
    };
    let before = sha256_hex(source.as_bytes());
    let command = agentic_message(
        "installed_source_operation_command",
        &[
            ("node", &quote(&prefix[0])),
            ("script", &quote(&prefix[1])),
            ("task", &quote(task)),
            ("digest", &before),
        ],
    );
    let Some((run_index, run)) = attempts
        .iter()
        .enumerate()
        .rev()
        .find(|(_, attempt)| exact_command(attempt, &command))
    else {
        return Some(tool_for(tools, Capability::Run).map_or_else(
            || gap("source_unary_gap_12", result),
            |tool| plan_one(tool, json!({"command":command}).to_string()),
        ));
    };
    let Some(arguments) = run
        .arguments
        .as_deref()
        .and_then(|arguments| serde_json::from_str::<Value>(arguments).ok())
    else {
        return Some(gap("source_unary_gap_5", result));
    };
    let Some(observed) = progress.latest_attempt_for(Capability::Run, &arguments) else {
        return Some(gap("source_unary_gap_6", result));
    };
    if !observed.succeeded
        || incomplete_receipt(&observed.detail)
        || step_outcome(&observed.detail) != StepOutcome::Succeeded
    {
        return Some(gap("source_unary_gap_6", result));
    }
    let Some(payload) = observed_payload(&observed.detail) else {
        return Some(gap("source_unary_gap_7", result));
    };
    let Some(receipt) = receipt(&payload, task, owner, &before) else {
        return Some(gap("source_unary_gap_8", result));
    };
    let current_source = read.source.as_deref()?;
    let after_run = attempts[run_index + 1..].iter().any(|attempt| {
        exact_owner_read(attempt, owner)
            && attempt.source_observation().is_some_and(|observation| {
                observation.status == SourceReadStatus::ReportedSuccess
                    && observation.complete
                    && observation.source.as_deref() == Some(current_source)
            })
    });
    if !after_run {
        return Some(read_tool.map_or_else(
            || gap("source_unary_gap_9", result),
            |tool| plan_one(tool, super::workspace_change::read_arguments(owner)),
        ));
    }
    if sha256_hex(current_source.as_bytes()) != receipt.after_sha256 {
        return Some(gap("source_unary_gap_10", result));
    }
    Some(record(AgenticPlan::Final(json!({"sourceUnaryConsolidation":{"owner":owner,"callbacks":receipt.callbacks,"sourceReadbackVerified":true,
        "independentBehaviorValidation":"pending","completeSemanticEquivalence":false,"authorshipCredit":0}}).to_string()),FinalDisposition::Finding,"source-unary-consolidation-observed",result))
}
