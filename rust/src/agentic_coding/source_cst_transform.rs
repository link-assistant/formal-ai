//! Qualified parameter edits delegate to the installed complete-CST rule engine.
//!
//! The engine reparses, verifies inverse/preimage and owns the physical write;
//! this planner binds a current owner Read and never synthesizes source edits.
use super::final_result::{FinalDisposition, FinalResult, record};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use crate::meta_reasoner::text::is_js_whitespace as request_whitespace;
use crate::protocol::ChatMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCstTransformRequest {
    pub owner: String,
    pub declaration: String,
    pub binding: String,
    pub target: String,
}
/// The same bounded, declaration-qualified request grammar as the JS route.
#[must_use]
pub fn source_cst_transform_request(task: &str) -> Option<SourceCstTransformRequest> {
    static GRAMMAR: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    if task.len() > 4 * 4096 || task.encode_utf16().count() > 4096 {
        return None;
    }
    let grammar = GRAMMAR.get_or_init(|| {
        let whitespace = r"[\x09-\x0d\x20\u{00a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]";
        let source = r"(?i)^Rename\s+(?:the\s+)?parameter\s+`?([A-Za-z_$][A-Za-z0-9_$]*)`?\s+(?:of|in)\s+(?:the\s+)?function\s+`?([A-Za-z_$][A-Za-z0-9_$]*)`?\s+to\s+`?([A-Za-z_$][A-Za-z0-9_$]*)`?\s+in\s+`?([A-Za-z0-9_./-]+\.(?:mjs|js|rs|mts|ts))`?\.?\z";
        regex::Regex::new(&source.replace(r"\s", whitespace)).expect("SourceCstRequestGrammarInvariant")
    });
    let captures = grammar.captures(task.trim_matches(request_whitespace))?;
    let owner = captures.get(4)?.as_str();
    if owner.starts_with('/') || owner.split('/').any(|part| matches!(part, "" | "." | "..")) {
        return None;
    }
    Some(SourceCstTransformRequest {
        owner: owner.to_owned(),
        declaration: captures.get(2)?.as_str().to_owned(),
        binding: captures.get(1)?.as_str().to_owned(),
        target: captures.get(3)?.as_str().to_owned(),
    })
}
fn refused(key: &str, result: &mut Option<FinalResult>) -> AgenticPlan {
    record(
        AgenticPlan::Final(super::wording::agentic_message(key, &[])),
        FinalDisposition::Gap,
        "source-cst-transform-refused",
        result,
    )
}
/// Live planner route: only an installed command and complete owner observation
/// can authorize requesting the shared engine's own guarded transform.
pub(super) fn plan_source_cst_transform_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let request = source_cst_transform_request(task)?;
    let Some(installed) =
        super::installed_source_commands::installed_source_command("source-cst-transform")
    else {
        return Some(refused("source-cst-host-missing", result));
    };
    if tool_for(tools, Capability::Run).is_none() {
        return Some(refused("source-cst-run-missing", result));
    }
    let current = &messages[super::planner::evidence_window_start(messages)..];
    let progress = Progress::scan(current);
    let Some(attempt) = progress
        .research_attempts()
        .iter()
        .rev()
        .find(|attempt| attempt.capability == Capability::Read)
    else {
        return Some(tool_for(tools, Capability::Read).map_or_else(|| refused("source-cst-read-missing", result), |tool| plan_one(tool, serde_json::json!({"path":request.owner,"filePath":request.owner,"file_path":request.owner}).to_string())));
    };
    let arguments = attempt
        .arguments
        .as_deref()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok());
    let Some(arguments) = arguments.and_then(|value| value.as_object().cloned()) else {
        return Some(refused("source-cst-read-arguments", result));
    };
    let paths: Vec<_> = ["path", "filePath", "file_path"]
        .iter()
        .filter_map(|name| arguments.get(*name))
        .collect();
    if paths.is_empty()
        || paths
            .iter()
            .any(|value| value.as_str() != Some(request.owner.as_str()))
    {
        return Some(refused("source-cst-read-owner", result));
    }
    let Some(read) = attempt
        .source_observation()
        .filter(|read| attempt.succeeded && read.complete && read.error.is_none())
    else {
        return Some(refused("source-cst-read-failed", result));
    };
    let Some(source) = read
        .source
        .as_deref()
        .filter(|source| source.len() <= 1024 * 1024)
    else {
        return Some(refused("source-cst-read-failed", result));
    };
    let quote = crate::portable_string_contracts::quote_shell_word;
    let command = super::wording::agentic_message(
        "installed-source-operation-command",
        &[
            ("node", &quote(&installed[0])),
            ("script", &quote(&installed[1])),
            ("task", &quote(task)),
            (
                "digest",
                &crate::source_fetch::sha256_hex(source.as_bytes()),
            ),
        ],
    );
    Some(super::prerequisite_recovery::plan_shell_step(
        current, tools, &command, result,
    ))
}
