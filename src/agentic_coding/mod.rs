//! Agentic-coding capability for the Formal AI server (issue #468).
//!
//! Issue #468 posed a text-formalization task and asked whether our Formal AI can
//! *solve such tasks in agentic mode* — i.e. be driven by an external agentic CLI
//! over the OpenAI-compatible server, emit tool calls, consume tool results,
//! react to errors, and loop to completion. The maintainer's framing is the
//! authority here:
//!
//! * *"for us everything is a link"* — so the knowledge base the task asks for is
//!   produced in Links Notation (our meta-language), **not** as typed-struct
//!   entities/ontologies. See [`formalize`].
//! * *"our Formal AI system should have enough skills … to actually call all the
//!   tools from any agentic CLI, understand errors from tools, and so on"* — so a
//!   deterministic planner turns the conversation-so-far into the next tool calls
//!   or a final answer, and an in-repo driver exercises the whole loop offline.
//!
//! This module hosts that capability. Neural inference remains a NON-GOAL:
//! extraction is grounded in a closed lexicon (see [`lexicon`]) and the planner is
//! a pure, deterministic function of the message history.

pub mod corpus;
pub mod diagram;
pub mod driver;
mod file_read;
pub mod formalize;
pub(crate) mod lexicon;
pub mod meaning_detail;
pub mod planner;
pub mod self_ast;

pub use diagram::{is_diagram_task, DIAGRAM_PATH, DIAGRAM_TASK};
pub use driver::{
    run_agentic_task, run_agentic_task_in, DriverOutcome, DriverToolStep, DRIVER_TOOLS,
};
pub use formalize::{
    coverage_line, formalize_text_to_links, FormalizationSummary, FormalizedKnowledgeBase,
    CANONICAL_FISHERMAN_SYNOPSIS, FISHERMAN_DOC_ID, PRIMITIVE_KINDS,
};
pub use meaning_detail::{
    concept_for_task, enrich_block, is_meaning_detail_task, MEANING_DETAIL_TASK, POTATO_DETAIL_TASK,
};
pub use planner::{
    plan_chat_step, AgenticPlan, PlannedToolCall, CANONICAL_SOURCE_URL, KB_PATH, SEARCH_QUERY,
};
pub use self_ast::{ast_census, is_self_ast_task, render_ast_document, AST_PATH, AST_TASK};
