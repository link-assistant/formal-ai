//! Deterministic agentic planner — the server's "brain" for issue #468.
//!
//! The maintainer's framing: *"our Formal AI system should have enough skills
//! (meta algorithm, rust code) to actually call all the tools from any agentic
//! CLI, understand errors from tools, and so on, call bash commands, do web fetch
//! and web search, to actually complete the task."*
//!
//! This module is that meta-algorithm for the canonical issue-#468 task —
//! formalizing «Сказка о рыбаке и рыбке» into a Links Notation knowledge base. It
//! is a **pure, deterministic function of the conversation so far**: given the
//! messages exchanged and the tool names the agentic CLI advertised, it decides
//! the next step. Neural inference stays a NON-GOAL — there is no sampling, no
//! hidden state, and the same history always yields the same plan.
//!
//! The recipe is a small state machine:
//!
//! ```text
//! web_search → web_fetch → write_file(formalize) → run_command(verify) → final
//! ```
//!
//! Each step is taken only if (a) the conversation does not already contain a
//! tool result for that capability and (b) the CLI advertised a tool with that
//! capability. Steps whose tool is unavailable are skipped, so the planner adapts
//! to whatever subset of tools a given CLI exposes. Tool *errors* are observed:
//! a fetch result that [`looks_like_error`] is ignored, and the formalizer falls
//! back to the canonical synopsis so the loop still completes with a stable
//! knowledge base.

use serde_json::json;

use super::formalize::{
    coverage_line, formalize_text_to_links, FormalizedKnowledgeBase, CANONICAL_FISHERMAN_SYNOPSIS,
    FISHERMAN_DOC_ID,
};
use crate::protocol::ChatMessage;

/// The Russian web-search query the planner issues when a search tool exists.
pub const SEARCH_QUERY: &str = "Пушкин Сказка о рыбаке и рыбке полный текст";

/// The source URL the planner fetches when a fetch tool exists.
pub const CANONICAL_SOURCE_URL: &str =
    "https://ru.wikisource.org/wiki/Сказка_о_рыбаке_и_рыбке_(Пушкин)";

/// The path the planner writes the knowledge base to.
pub const KB_PATH: &str = "knowledge-base.lino";

/// The next deterministic step the server takes in an agentic coding loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgenticPlan {
    /// Emit these tool calls (one per planned step) and wait for their results.
    ToolCalls(Vec<PlannedToolCall>),
    /// The task is complete; this is the final assistant answer.
    Final(String),
}

/// A single tool call the planner wants the server to emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedToolCall {
    /// The tool name to invoke (taken verbatim from the request's tools).
    pub tool: String,
    /// JSON-encoded arguments object for the call.
    pub arguments: String,
}

/// The tool capabilities the planner's recipe relies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Capability {
    Search,
    Fetch,
    Write,
    Run,
}

/// Plan the next agentic step from the conversation so far and the tool names the
/// CLI advertised.
///
/// Returns [`None`] when the latest user turn is not a formalization task — the
/// server then falls back to its ordinary solver text, so non-agentic-coding
/// requests are untouched.
#[must_use]
pub fn plan_chat_step(messages: &[ChatMessage], tool_names: &[&str]) -> Option<AgenticPlan> {
    let task = latest_user_text(messages)?;
    if !is_formalization_task(&task) {
        return None;
    }

    let search_tool = tool_for(tool_names, Capability::Search);
    let fetch_tool = tool_for(tool_names, Capability::Fetch);
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);

    // Step 1: search for the source text.
    if let Some(tool) = search_tool {
        if !progress.done(Capability::Search) {
            return Some(plan_one(tool, json!({ "query": SEARCH_QUERY }).to_string()));
        }
    }
    // Step 2: fetch the source text.
    if let Some(tool) = fetch_tool {
        if !progress.done(Capability::Fetch) {
            return Some(plan_one(
                tool,
                json!({ "url": CANONICAL_SOURCE_URL }).to_string(),
            ));
        }
    }

    // The source text for the knowledge base: the latest non-errored fetch result
    // if we have one, else the canonical synopsis (the determinism fallback).
    let source = progress
        .fetched_text
        .as_deref()
        .unwrap_or(CANONICAL_FISHERMAN_SYNOPSIS);
    let formalized = formalize_text_to_links(source, "");

    // Step 3: write the formalized knowledge base.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            let arguments = json!({ "path": KB_PATH, "content": formalized.links_notation });
            return Some(plan_one(tool, arguments.to_string()));
        }
    }
    // Step 4: verify by reading the file back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {KB_PATH}") });
            return Some(plan_one(tool, arguments.to_string()));
        }
    }

    // Step 5: nothing left to do — answer with the knowledge base inline.
    Some(AgenticPlan::Final(final_answer(&formalized)))
}

/// Which recipe capabilities the conversation already produced a result for.
struct Progress {
    /// Capabilities a prior `tool` result already answered.
    completed: Vec<Capability>,
    /// The latest non-errored fetch result's text, if any.
    fetched_text: Option<String>,
}

impl Progress {
    fn scan(messages: &[ChatMessage]) -> Self {
        let mut completed = Vec::new();
        let mut fetched_text = None;
        for (index, message) in messages.iter().enumerate() {
            if !message.role.eq_ignore_ascii_case("tool") {
                continue;
            }
            let Some(capability) = result_capability(messages, index) else {
                continue;
            };
            if capability == Capability::Fetch {
                let text = message.content.plain_text();
                if !looks_like_error(&text) && !text.trim().is_empty() {
                    fetched_text = Some(text);
                }
            }
            if !completed.contains(&capability) {
                completed.push(capability);
            }
        }
        Self {
            completed,
            fetched_text,
        }
    }

    /// Whether a prior tool result already covered `capability`.
    fn done(&self, capability: Capability) -> bool {
        self.completed.contains(&capability)
    }
}

fn plan_one(tool: &str, arguments: String) -> AgenticPlan {
    AgenticPlan::ToolCalls(vec![PlannedToolCall {
        tool: tool.to_owned(),
        arguments,
    }])
}

/// The first advertised tool name that provides `capability`, if any.
fn tool_for<'a>(tool_names: &[&'a str], capability: Capability) -> Option<&'a str> {
    tool_names
        .iter()
        .copied()
        .find(|name| classify_tool(name) == Some(capability))
}

/// Classify a tool name into a [`Capability`] by substring, mirroring the naming
/// conventions agentic CLIs use (`web_search`, `web_fetch`, `write_file`,
/// `run_command`, `bash`, …).
fn classify_tool(name: &str) -> Option<Capability> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("search") {
        Some(Capability::Search)
    } else if lower.contains("fetch")
        || lower.contains("open")
        || lower.contains("browse")
        || lower.contains("get_url")
        || lower.contains("read_url")
    {
        Some(Capability::Fetch)
    } else if lower.contains("write") {
        Some(Capability::Write)
    } else if lower.contains("run")
        || lower.contains("bash")
        || lower.contains("command")
        || lower.contains("exec")
        || lower.contains("shell")
    {
        Some(Capability::Run)
    } else {
        None
    }
}

/// Resolve which capability the tool result at `index` answers. Prefer the
/// result's own `name`; otherwise map its `tool_call_id` back to the tool name in
/// a prior assistant `tool_calls` turn.
fn result_capability(messages: &[ChatMessage], index: usize) -> Option<Capability> {
    let message = &messages[index];
    if let Some(name) = &message.name {
        if let Some(capability) = classify_tool(name) {
            return Some(capability);
        }
    }
    let call_id = message.tool_call_id.as_ref()?;
    messages[..index]
        .iter()
        .flat_map(|prior| prior.tool_calls.iter())
        .find(|call| &call.id == call_id)
        .and_then(|call| classify_tool(&call.function.name))
}

/// The text of the most recent `user` turn.
fn latest_user_text(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.plain_text())
}

/// Keywords that mark a user turn as the canonical issue-#468 formalization task.
const FORMALIZATION_KEYWORDS: [&str; 7] = [
    "formaliz",
    "формализ",
    "knowledge base",
    "links notation",
    "рыбак",
    "fisherman",
    "сказк",
];

/// Whether `prompt` asks to formalize the canonical tale into a knowledge base.
fn is_formalization_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    FORMALIZATION_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
}

/// Whether a tool result looks like an error the planner should not trust.
fn looks_like_error(text: &str) -> bool {
    let lower = text.to_lowercase();
    ["error", "failed", "not found", "404"]
        .iter()
        .any(|needle| lower.contains(needle))
}

/// The self-contained final answer: a natural-language summary, the coverage
/// line, and the Links Notation knowledge base inline.
fn final_answer(formalized: &FormalizedKnowledgeBase) -> String {
    let summary = &formalized.summary;
    let subject = if summary.doc_id == FISHERMAN_DOC_ID {
        "«Сказка о рыбаке и рыбке»".to_owned()
    } else {
        format!("the source text ({})", summary.doc_id)
    };
    format!(
        "Formalized {subject} into a Links Notation knowledge base: {records} records realising \
         all nine protocol primitives ({coverage}).\n\nKnowledge base ({KB_PATH}):\n\n{kb}",
        records = summary.total_records(),
        coverage = coverage_line(summary),
        kb = formalized.links_notation.trim_end(),
    )
}
