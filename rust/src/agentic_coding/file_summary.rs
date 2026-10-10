//! Summarizing a named file (PR #1188 T100, gap G27).
//!
//! "Summarize README.md in one sentence." answered "Read 1 file(s): README.md:
//! # Demo". A request with the seeded summarization action
//! (`text_summarization_action`, every registered language) that names one
//! file reads it and hands the request, the file's text in place of its name,
//! to the shared solver's summarization handlers. Twin of
//! `js/agentic/file_summary.mjs`.

use super::final_result::{FinalDisposition, FinalResult, record};
use super::module_function::read_source;
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use crate::protocol::ChatMessage;
use crate::seed;

const SUMMARY_ROLE: &str = "text_summarization_action";
const SUMMARY_INTENT_PREFIX: &str = "summarization";
const CLAUSE_ENDS: &[char] = &['.', '!', '?', '。', '！', '？', '।'];

/// The one file a summarization request names.
fn summarized_file(task: &str) -> Option<String> {
    if !seed::lexicon().mentions_role(SUMMARY_ROLE, &crate::engine::normalize_prompt(task)) {
        return None;
    }
    let paths = super::file_read::owned_read_paths(task, SUMMARY_ROLE);
    match paths.as_slice() {
        [only] => Some(only.clone()),
        _ => None,
    }
}

/// The request with the file's text in place of its name.
fn summary_request(task: &str, path: &str, text: &str) -> String {
    let words: Vec<&str> = task
        .split_whitespace()
        .filter(|word| !word.contains(path))
        .collect();
    let request = words.join(" ");
    format!("{}: {}", request.trim_end_matches(CLAUSE_ENDS), text.trim())
}

/// The arm: read the named file, then answer with the solver's summary of it.
pub(super) fn plan_file_summary_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let path = summarized_file(task)?;
    let current_turn = &messages[super::planner::evidence_window_start(messages)..];
    let Some(source) = read_source(current_turn, &path) else {
        let read = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(
            read,
            super::workspace_change::read_arguments(&path),
        ));
    };
    if source.trim().is_empty() {
        return None;
    }
    let answer =
        crate::solver::UniversalSolver::default().solve(&summary_request(task, &path, &source));
    (answer.intent.starts_with(SUMMARY_INTENT_PREFIX) && !answer.answer.is_empty()).then(|| {
        record(
            AgenticPlan::Final(answer.answer),
            FinalDisposition::Finding,
            &answer.intent,
            result,
        )
    })
}
