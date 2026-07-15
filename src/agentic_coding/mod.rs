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

pub mod associative_learning;
pub mod change_request;
pub(crate) mod command_reroute;
mod conversation_recall;
pub mod corpus;
pub mod diagram;
pub mod dreaming_audit;
pub mod driver;
pub mod execution_learning;
pub mod explain;
mod file_read;
pub mod formalize;
pub mod general_planner;
pub mod google_trends_catalog;
pub mod google_trends_learning;
mod intent_router;
pub mod ledger;
pub(crate) mod lexicon;
pub mod meaning_detail;
pub mod planner;
pub mod question_catalog;
pub mod rebuild_plan;
pub mod repair_strategy;
mod report_issue;
pub mod routing_learning;
pub mod self_ast;
pub mod self_heal;
mod shell_command;
pub mod source_graph;
mod web_research;

pub use associative_learning::{
    is_associative_learning_task, ASSOCIATIVE_LEARNING_PATH, ASSOCIATIVE_LEARNING_TASK,
};
pub use change_request::{is_change_request_task, CHANGE_PATH, CHANGE_TASK};
pub use diagram::{is_diagram_task, DIAGRAM_PATH, DIAGRAM_TASK};
pub use dreaming_audit::{is_dreaming_audit_task, DREAMING_AUDIT_PATH, DREAMING_AUDIT_TASK};
pub use driver::{
    run_agentic_task, run_agentic_task_in, DriverOutcome, DriverToolStep, DRIVER_TOOLS,
};
pub use execution_learning::{
    is_execution_learning_task, EXECUTION_LEARNING_PATH, EXECUTION_LEARNING_TASK,
};
pub use explain::{is_explain_task, EXPLAIN_PATH, EXPLAIN_TASK};
pub use formalize::{
    coverage_line, formalize_text_to_links, FormalizationSummary, FormalizedKnowledgeBase,
    CANONICAL_FISHERMAN_SYNOPSIS, FISHERMAN_DOC_ID, PRIMITIVE_KINDS,
};
pub use google_trends_catalog::{
    is_google_trends_catalog_task, GOOGLE_TRENDS_CATALOG_PATH, GOOGLE_TRENDS_CATALOG_TASK,
};
pub use google_trends_learning::{
    is_google_trends_learning_task, GOOGLE_TRENDS_LEARNING_PATH, GOOGLE_TRENDS_LEARNING_TASK,
};
pub use ledger::{is_ledger_task, LEDGER_PATH, LEDGER_TASK};
pub use meaning_detail::{
    concept_for_task, enrich_block, is_meaning_detail_task, MEANING_DETAIL_TASK, POTATO_DETAIL_TASK,
};
pub use planner::{
    plan_chat_step, AgenticPlan, PlannedToolCall, CANONICAL_SOURCE_URL, KB_PATH, SEARCH_QUERY,
};
pub use question_catalog::{
    is_question_catalog_task, QUESTION_CATALOG_PATH, QUESTION_CATALOG_TASK,
};
pub use rebuild_plan::{is_rebuild_task, REBUILD_PATH, REBUILD_TASK};
pub use repair_strategy::{is_repair_strategy_task, REPAIR_STRATEGY_PATH, REPAIR_STRATEGY_TASK};
pub use routing_learning::{
    is_routing_learning_task, ROUTING_LEARNING_PATH, ROUTING_LEARNING_TASK,
};
pub use self_ast::{ast_census, is_self_ast_task, render_ast_document, AST_PATH, AST_TASK};
pub use self_heal::{is_self_heal_task, SELF_HEAL_PATH, SELF_HEAL_TASK};
pub use source_graph::{is_source_graph_task, SOURCE_GRAPH_PATH, SOURCE_GRAPH_TASK};
