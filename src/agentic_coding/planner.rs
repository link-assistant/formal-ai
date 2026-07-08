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

use super::change_request;
use super::diagram;
use super::explain;
use super::file_read::{file_read_task_for, plan_file_read_step};
use super::formalize::{
    coverage_line, formalize_text_to_links, FormalizedKnowledgeBase, CANONICAL_FISHERMAN_SYNOPSIS,
    FISHERMAN_DOC_ID,
};
use super::google_trends_catalog;
use super::ledger;
use super::meaning_detail;
use super::question_catalog;
use super::rebuild_plan;
use super::repair_strategy;
use super::self_ast;
use super::self_heal;
use super::source_graph;
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
///
/// This is the single source of truth for "what kind of thing a tool does". Both
/// the planner (to pick which advertised tool to call for each recipe step) and
/// the server's permission gate (to decide whether an agentic client may drive a
/// tool of this kind) classify tool names through [`tool_capability`] — so the
/// two never drift and no per-tool-name special cases accumulate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Search,
    Fetch,
    Read,
    Write,
    Run,
}

impl Capability {
    /// The associative-package capability key that grants an agentic client the
    /// right to drive a tool of this kind, e.g. `tool:capability:write`. Grants
    /// are by *capability class*, not by tool name, so any CLI's naming
    /// (`write`, `write_file`, `edit`, `patch`, …) maps to the same permission.
    #[must_use]
    pub const fn permission_key(self) -> &'static str {
        match self {
            Self::Search => "tool:capability:search",
            Self::Fetch => "tool:capability:fetch",
            Self::Read => "tool:capability:read",
            Self::Write => "tool:capability:write",
            Self::Run => "tool:capability:run",
        }
    }
}

/// Classify an advertised tool name into the [`Capability`] it provides.
///
/// Returns [`None`] when the planner's recipe has no use for it
/// (list/grep/todo/…). Public so the permission gate classifies through the
/// *same* function the planner uses.
#[must_use]
pub fn tool_capability(name: &str) -> Option<Capability> {
    classify_tool(name)
}

/// Plan the next agentic step from the conversation so far and the tool names the
/// CLI advertised.
///
/// Returns [`None`] when the latest user turn is neither of the recipes the
/// planner knows (formalize a text — issue #468, or make a meaning more detailed
/// — issue #538) — the server then falls back to its ordinary solver text, so
/// unrelated requests are untouched.
#[must_use]
pub fn plan_chat_step(messages: &[ChatMessage], tool_names: &[&str]) -> Option<AgenticPlan> {
    let task = latest_user_text(messages)?;
    // The self-AST recipe is checked first because it is the most specific router
    // (it requires both an AST/CST intent word *and* a self-reference). A self-AST
    // request legitimately mentions "Links Notation" as its output format, which
    // would otherwise be captured by the broad formalization keyword match below.
    // The self-healing recipe is checked before self-AST: both are self-inspection
    // recipes, but self-healing has its own dedicated keywords (self-heal, repair
    // case, auto-learning) that never overlap the AST/CST keywords, so ordering only
    // guards against a request that names both.
    if self_heal::is_self_heal_task(&task) {
        return Some(plan_self_heal_step(messages, tool_names));
    }
    if self_ast::is_self_ast_task(&task) {
        return Some(plan_self_ast_step(messages, tool_names));
    }
    // The whole-repository source-graph recipe: checked alongside the other
    // self-inspection recipes and before formalization, because its request
    // legitimately names "links" (its output format), which the broad
    // formalization keyword match below would otherwise capture.
    if source_graph::is_source_graph_task(&task) {
        return Some(plan_source_graph_step(messages, tool_names));
    }
    // The learning-ledger recipe: the promotion step that follows an approved repair
    // case. Checked after self-healing (which owns the "auto learning" keywords) and
    // before formalization, since its request legitimately names "Links Notation".
    if ledger::is_ledger_task(&task) {
        return Some(plan_ledger_step(messages, tool_names));
    }
    // The grounded self-explanation recipe: answers "how does Formal AI work?" from
    // real source/data/test artifacts. Checked alongside the other self-inspection
    // recipes and before formalization, since its request legitimately names "Links
    // Notation" as the output format its document is rendered in.
    if explain::is_explain_task(&task) {
        return Some(plan_explain_step(messages, tool_names));
    }
    // The user-initiated self-change recipe: turns a natural-language "change Formal AI
    // itself" request into a reviewable pull request through the same human-gated loop.
    // Checked alongside the other self-referential recipes and before formalization,
    // since its request legitimately names "Links Notation" as the output format.
    if change_request::is_change_request_task(&task) {
        return Some(plan_change_request_step(messages, tool_names));
    }
    // The general repair-classification recipe: given an arbitrary failure trace, decide
    // whether the repair is a solver method, a data record, or a test, and compose the
    // grounded, human-gated strategy for each class. Checked alongside the other
    // self-referential recipes and before formalization, since its request legitimately
    // names "Links Notation" as the output format its strategies are rendered in. Its
    // keywords are disjoint from the self-healing recipe's ("repair case"/"repair loop"),
    // so ordering only guards a request that somehow names both.
    if repair_strategy::is_repair_strategy_task(&task) {
        return Some(plan_repair_strategy_step(messages, tool_names));
    }
    // Rebuild-and-reattach recipe: once a change is accepted, recompile Formal AI and
    // reattach the improved WebAssembly worker to the UI (issue #558's `R558-06`).
    // Checked alongside the other self-referential recipes and before formalization,
    // since its request legitimately names "Links Notation" as the output format its plan
    // is rendered in. Its keywords key on "reattach" and are disjoint from the
    // source-graph recipe's "recompile", so ordering only guards a request that somehow
    // names both.
    if rebuild_plan::is_rebuild_task(&task) {
        return Some(plan_rebuild_step(messages, tool_names));
    }
    if google_trends_catalog::is_google_trends_catalog_task(&task) {
        return Some(plan_google_trends_catalog_step(messages, tool_names));
    }
    // The question-catalog recipe (issue #527): enumerate every possible question
    // smallest-first, classify each grammatically and logically, and answer the
    // meaningful ones. Checked alongside the other self-referential recipes and before
    // formalization, since its request legitimately names "Links Notation" as the output
    // format its catalog is rendered in. Its keywords ("question catalog", "all possible
    // questions", …) are disjoint from the sibling recipes', so ordering only guards a
    // request that somehow names both.
    if question_catalog::is_question_catalog_task(&task) {
        return Some(plan_question_catalog_step(messages, tool_names));
    }
    if let Some(file_task) = file_read_task_for(&task) {
        return Some(plan_file_read_step(&file_task, messages, tool_names));
    }
    if let Some(command) = shell_command_for_task(&task) {
        return Some(plan_shell_step(messages, tool_names, &command));
    }
    if is_formalization_task(&task) {
        return Some(plan_formalization_step(messages, tool_names));
    }
    if meaning_detail::is_meaning_detail_task(&task) {
        return Some(plan_meaning_detail_step(&task, messages, tool_names));
    }
    if diagram::is_diagram_task(&task) {
        return Some(plan_diagram_step(messages, tool_names));
    }
    None
}

/// The issue-#607 shell recipe: ask the CLI's shell/run tool to execute a simple
/// directory listing, then summarize the tool result. Execution still happens in
/// the client-side agent workspace/permission model; this server only emits the
/// OpenAI-compatible `tool_calls` turn.
fn plan_shell_step(messages: &[ChatMessage], tool_names: &[&str], command: &str) -> AgenticPlan {
    let progress = Progress::scan(messages);
    if progress.done(Capability::Run) {
        return AgenticPlan::Final(shell_final_answer(
            command,
            progress.run_output.as_deref().unwrap_or_default(),
        ));
    }

    if let Some(tool) = tool_for(tool_names, Capability::Run) {
        return plan_one(tool, json!({ "command": command }).to_string());
    }

    AgenticPlan::Final(format!(
        "I can run `{command}` when the client advertises a shell tool such as `bash`, `shell`, or `run_command`."
    ))
}

/// The issue-#468 recipe: search → fetch → formalize → verify → final.
fn plan_formalization_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let search_tool = tool_for(tool_names, Capability::Search);
    let fetch_tool = tool_for(tool_names, Capability::Fetch);
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);

    // Step 1: search for the source text.
    if let Some(tool) = search_tool {
        if !progress.done(Capability::Search) {
            return plan_one(tool, json!({ "query": SEARCH_QUERY }).to_string());
        }
    }
    // Step 2: fetch the source text.
    if let Some(tool) = fetch_tool {
        if !progress.done(Capability::Fetch) {
            return plan_one(tool, fetch_arguments(CANONICAL_SOURCE_URL));
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
            return plan_one(tool, write_arguments(KB_PATH, &formalized.links_notation));
        }
    }
    // Step 4: verify by reading the file back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {KB_PATH}") });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 5: nothing left to do — answer with the knowledge base inline.
    AgenticPlan::Final(final_answer(&formalized))
}

/// The issue-#538 recipe: search → fetch (Wikidata lexemes) → write the enriched
/// meaning block → verify → final. Mirrors the formalization recipe but re-derives
/// the enriched meaning block from the fetched lexeme facts instead of formalizing
/// prose. The concept to enrich is routed from the request itself
/// ([`meaning_detail::concept_for_task`]), so the *same* recipe makes tomato,
/// potato, or any registered concept more detailed. Steps whose tool the CLI did
/// not advertise are skipped.
fn plan_meaning_detail_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> AgenticPlan {
    // Route to the concept the request names (default: tomato — the canonical task).
    let concept = meaning_detail::concept_for_task(task).unwrap_or(&meaning_detail::TOMATO);

    let search_tool = tool_for(tool_names, Capability::Search);
    let fetch_tool = tool_for(tool_names, Capability::Fetch);
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);

    // Step 1: search for the Wikidata lexeme data.
    if let Some(tool) = search_tool {
        if !progress.done(Capability::Search) {
            return plan_one(tool, json!({ "query": concept.search_query }).to_string());
        }
    }
    // Step 2: fetch the lexeme forms (where the missing plural is recovered).
    if let Some(tool) = fetch_tool {
        if !progress.done(Capability::Fetch) {
            return plan_one(tool, fetch_arguments(concept.source_url));
        }
    }

    // Re-derive the enriched block from the fetched lexeme facts (or the canonical
    // fallback when the fetch errored), exactly as the formalization recipe does.
    let block = meaning_detail::enrich_block(concept, progress.fetched_text.as_deref());

    // Step 3: write the enriched meaning block.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(concept.kb_path, &block));
        }
    }
    // Step 4: verify by reading the enriched block back (mirrors the formalization
    // recipe; `cat` is the allowlisted read the sandbox workspace supports).
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", concept.kb_path) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 5: nothing left to do — answer with the enriched block inline.
    AgenticPlan::Final(meaning_detail::final_answer_for(concept, &block))
}

/// The issue-#538 diagram recipe: write the generated mermaid document → verify →
/// final. Unlike the other two recipes it needs no web step — the diagrams are a
/// pure function of the planner's own recipe table ([`diagram::render_document`]),
/// so the loop *documents itself*. Steps whose tool the CLI did not advertise are
/// skipped.
fn plan_diagram_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = diagram::render_document();

    // Step 1: write the generated diagram document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(diagram::DIAGRAM_PATH, &document));
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", diagram::DIAGRAM_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(diagram::final_answer(&document))
}

/// The issue-#538 self-AST recipe: write the generated CST/AST-in-data document →
/// verify → final. Like the diagram recipe it needs no web step — the document is a
/// pure function of the planner's own source parsed through the meta-language links
/// network ([`self_ast::render_document`]), so the loop *inspects itself*. Steps
/// whose tool the CLI did not advertise are skipped.
fn plan_self_ast_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = self_ast::render_document();

    // Step 1: write the generated CST/AST document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(self_ast::AST_PATH, &document));
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", self_ast::AST_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(self_ast::final_answer(&document))
}

/// The issue-#558 self-healing recipe: write the generated repair-case document →
/// verify → final. Like the diagram and self-AST recipes it needs no web step — the
/// document is a pure function of the canonical self-healing case
/// ([`self_heal::render_document`]), so the loop *repairs itself*. Steps whose tool
/// the CLI did not advertise are skipped.
fn plan_self_heal_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = self_heal::render_document();

    // Step 1: write the generated repair-case document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(self_heal::SELF_HEAL_PATH, &document));
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", self_heal::SELF_HEAL_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(self_heal::final_answer(&document))
}

/// The issue-#558 source-graph recipe: write the generated whole-repository
/// source ↔ links projection document → verify → final. Like the diagram, self-AST,
/// and self-healing recipes it needs no web step — the document is a pure function
/// of the system's own embedded source projected through the meta-language links
/// network ([`source_graph::render_document`]), so the loop *translates itself*.
/// Steps whose tool the CLI did not advertise are skipped.
fn plan_source_graph_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = source_graph::render_document();

    // Step 1: write the generated projection document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(
                tool,
                write_arguments(source_graph::SOURCE_GRAPH_PATH, &document),
            );
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments =
                json!({ "command": format!("cat {}", source_graph::SOURCE_GRAPH_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(source_graph::final_answer(&document))
}

/// The issue-#558 learning-ledger recipe: write the generated approved-lesson ledger
/// document → verify → final. Like the other self-inspection recipes it needs no web
/// step — the document is a pure function of the canonical, human-approved ledger
/// ([`ledger::render_document`]). Steps whose tool the CLI did not advertise are
/// skipped.
fn plan_ledger_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = ledger::render_document();

    // Step 1: write the generated ledger document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(ledger::LEDGER_PATH, &document));
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", ledger::LEDGER_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(ledger::final_answer(&document))
}

/// The issue-#558 self-explanation recipe: write the generated grounded-explanation
/// document → verify → final. Like the other self-inspection recipes it needs no web
/// step — the document is a pure function of the system's own embedded source cited
/// through the owned manifest ([`explain::render_document`]), so the loop *explains
/// itself*. Steps whose tool the CLI did not advertise are skipped.
fn plan_explain_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = explain::render_document();

    // Step 1: write the generated grounded-explanation document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(explain::EXPLAIN_PATH, &document));
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", explain::EXPLAIN_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(explain::final_answer(&document))
}

/// The issue-#558 self-change recipe: write the generated reviewable pull-request
/// document → verify → final. Like the other self-referential recipes it needs no web
/// step — the document is a deterministic function of the request and its grounded
/// target ([`change_request::render_document`]), so the loop turns a user's request to
/// *change Formal AI itself* into a reviewable PR. Steps whose tool the CLI did not
/// advertise are skipped.
fn plan_change_request_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = change_request::render_document();

    // Step 1: write the generated reviewable pull-request document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(
                tool,
                write_arguments(change_request::CHANGE_PATH, &document),
            );
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", change_request::CHANGE_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(change_request::final_answer(&document))
}

/// The issue-#558 general repair-classification recipe: write the generated
/// repair-strategies document → verify → final. Like the other self-referential recipes
/// it needs no web step — the document is a deterministic function of the three
/// self-contained canonical failure traces ([`repair_strategy::render_document`]), so
/// the loop decides *which part* of itself to repair for every failure class. Steps
/// whose tool the CLI did not advertise are skipped.
fn plan_repair_strategy_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = repair_strategy::render_document();

    // Step 1: write the generated repair-strategies document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(
                tool,
                write_arguments(repair_strategy::REPAIR_STRATEGY_PATH, &document),
            );
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments =
                json!({ "command": format!("cat {}", repair_strategy::REPAIR_STRATEGY_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(repair_strategy::final_answer(&document))
}

/// The issue-#558 rebuild-and-reattach recipe: write the generated
/// rebuild-and-reattach plan → verify → final. Like the change-request and source-graph
/// recipes it needs no web step — the plan is a deterministic function of the accepted
/// change and the grounded UI artifacts ([`rebuild_plan::render_document`]), so the loop
/// turns an accepted change into the ordered, reversible plan to recompile Formal AI and
/// reattach the improved worker to the UI. Steps whose tool the CLI did not advertise are
/// skipped.
fn plan_rebuild_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = rebuild_plan::render_document();

    // Step 1: write the generated rebuild-and-reattach plan.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(tool, write_arguments(rebuild_plan::REBUILD_PATH, &document));
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments = json!({ "command": format!("cat {}", rebuild_plan::REBUILD_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated plan inline.
    AgenticPlan::Final(rebuild_plan::final_answer(&document))
}

/// The issue-#527 question-catalog recipe: write the generated question-catalog
/// document → verify → final. Like the other self-referential recipes it needs no web
/// step — the document is a deterministic function of the seed lexicon and the
/// deterministic engine ([`question_catalog::render_document`]), so the loop *generates
/// every possible question and answers it*. Steps whose tool the CLI did not advertise
/// are skipped.
fn plan_question_catalog_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = question_catalog::render_document();

    // Step 1: write the generated question-catalog document.
    if let Some(tool) = write_tool {
        if !progress.done(Capability::Write) {
            return plan_one(
                tool,
                write_arguments(question_catalog::QUESTION_CATALOG_PATH, &document),
            );
        }
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) = run_tool {
        if !progress.done(Capability::Run) {
            let arguments =
                json!({ "command": format!("cat {}", question_catalog::QUESTION_CATALOG_PATH) });
            return plan_one(tool, arguments.to_string());
        }
    }

    // Step 3: nothing left to do — answer with the generated catalog inline.
    AgenticPlan::Final(question_catalog::final_answer(&document))
}

fn plan_google_trends_catalog_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let write_tool = tool_for(tool_names, Capability::Write);
    let run_tool = tool_for(tool_names, Capability::Run);

    let progress = Progress::scan(messages);
    let document = google_trends_catalog::render_document();

    if let Some(tool) = write_tool.filter(|_| !progress.done(Capability::Write)) {
        return plan_one(
            tool,
            write_arguments(google_trends_catalog::GOOGLE_TRENDS_CATALOG_PATH, &document),
        );
    }

    if let Some(tool) = run_tool.filter(|_| !progress.done(Capability::Run)) {
        let command = google_trends_catalog::verification_command();
        return plan_one(tool, json!({ "command": command }).to_string());
    }

    AgenticPlan::Final(google_trends_catalog::final_answer(&document))
}

/// Which recipe capabilities the conversation already produced a result for.
struct Progress {
    /// Capabilities a prior `tool` result already answered.
    completed: Vec<Capability>,
    /// The latest non-errored fetch result's text, if any.
    fetched_text: Option<String>,
    /// The latest run/shell result's text, if any.
    run_output: Option<String>,
}

impl Progress {
    fn scan(messages: &[ChatMessage]) -> Self {
        let mut completed = Vec::new();
        let mut fetched_text = None;
        let mut run_output = None;
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
            if capability == Capability::Run {
                run_output = Some(message.content.plain_text());
            }
            if !completed.contains(&capability) {
                completed.push(capability);
            }
        }
        Self {
            completed,
            fetched_text,
            run_output,
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

/// Arguments for a write step that satisfy whichever key the advertised write
/// tool expects. Agentic CLIs disagree on the parameter name — the in-repo driver
/// reads `path`, the `@link-assistant/agent` CLI's `write` tool wants `filePath`,
/// others use `file_path`. All are emitted; a schema-validating CLI keeps the one
/// it declared and strips the rest, so the same plan drives any of them without a
/// per-CLI special case.
fn write_arguments(path: &str, content: &str) -> String {
    json!({
        "path": path,
        "filePath": path,
        "file_path": path,
        "content": content,
    })
    .to_string()
}

/// Arguments for a fetch step. Emits `url` (the universal key) plus `format`
/// set to `"text"` — the `@link-assistant/agent` CLI's `webfetch` tool declares
/// a required `format` enum (`"text" | "markdown" | "html"`) and zod refuses the
/// call otherwise (observed live: *"Invalid option: expected one of
/// \"text\"|\"markdown\"|\"html\""*). The in-repo driver reads only `url`, and
/// CLIs whose schemas don't declare `format` strip it, so one shape drives all
/// of them without a per-CLI special case.
fn fetch_arguments(url: &str) -> String {
    json!({
        "url": url,
        "format": "text",
    })
    .to_string()
}

/// The first advertised tool name that provides `capability`, if any.
fn tool_for<'a>(tool_names: &[&'a str], capability: Capability) -> Option<&'a str> {
    tool_names
        .iter()
        .copied()
        .find(|name| classify_tool(name) == Some(capability))
}

/// Classify a tool name into a [`Capability`] by substring, mirroring the naming
/// conventions agentic CLIs use (`web_search`, `web_fetch`, `read`, `write_file`,
/// `run_command`, `bash`, `websearch`, `webfetch`, …).
///
/// The recipe only wants five kinds of tool, and real CLIs expose *lookalikes*
/// that must not be mistaken for them: a `todowrite` scratchpad is not a file
/// writer, a `codesearch` is not a web search, and `edit`/`patch`
/// are not create-file tools. Those are ruled out first so that — even though
/// [`requested_tool_names`](super::super::protocol) hands the planner an
/// alphabetically sorted list — `todowrite` can never be picked ahead of `write`
/// nor `codesearch` ahead of `websearch`.
fn classify_tool(name: &str) -> Option<Capability> {
    let lower = name.to_ascii_lowercase();
    // Scratchpad / navigation tools that merely *look* like recipe tools.
    if lower.contains("todo") {
        return None;
    }
    if lower.contains("search") {
        // A code search is not the web search the recipe issues its query to.
        (!lower.contains("code")).then_some(Capability::Search)
    } else if lower == "read"
        || lower.contains("read_file")
        || lower.contains("read_local_file")
        || lower.contains("file_read")
        || lower.contains("open_file")
        || lower.contains("view_file")
    {
        Some(Capability::Read)
    } else if lower.contains("fetch")
        || lower.contains("open")
        || lower.contains("browse")
        || lower.contains("get_url")
        || lower.contains("read_url")
    {
        Some(Capability::Fetch)
    } else if lower.contains("write") || lower.contains("create_file") {
        // `write` / `write_file` create a file from scratch; `edit`/`patch`
        // mutate an existing one and take different arguments, so they are not
        // interchangeable with the recipe's write step and stay unclassified.
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

/// Whether `prompt` asks the CLI to run `ls` / list the current workspace.
///
/// This intentionally starts narrow: it resolves only read-only directory-listing
/// language to `ls`. Broader shell synthesis belongs in a richer command parser,
/// not in a one-off fallback that might accidentally execute an invented command.
fn shell_command_for_task(prompt: &str) -> Option<String> {
    let lower = prompt.to_ascii_lowercase();
    let mentions_ls = contains_word(&lower, "ls");
    let run_context = ["run", "execute", "command", "terminal", "shell"]
        .iter()
        .any(|word| contains_word(&lower, word));
    let listing_context = contains_any(
        &lower,
        &[
            "list files",
            "list the files",
            "list local files",
            "list directory",
            "files here",
            "current directory",
            "working directory",
        ],
    );

    if mentions_ls && (run_context || listing_context) {
        return Some(String::from("ls"));
    }

    let asks_for_listing = contains_any(
        &lower,
        &[
            "list files",
            "list the files",
            "list local files",
            "list directory",
            "directory listing",
            "directory contents",
            "folder contents",
            "contents of this directory",
            "contents of the current directory",
            "contents of this folder",
            "contents of the current folder",
        ],
    );
    let asks_which_files = contains_any(
        &lower,
        &[
            "what files",
            "which files",
            "files are in",
            "files exist",
            "files are here",
        ],
    );
    let local_scope = contains_any(
        &lower,
        &[
            "here",
            "current directory",
            "working directory",
            "current working directory",
            "this directory",
            "current folder",
            "this folder",
            "local files",
        ],
    );

    ((asks_for_listing || asks_which_files) && local_scope).then(|| String::from("ls"))
}

fn contains_any(text: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|phrase| text.contains(phrase))
}

fn contains_word(text: &str, word: &str) -> bool {
    text.split(|character: char| !character.is_ascii_alphanumeric())
        .any(|part| part == word)
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

fn shell_final_answer(command: &str, output: &str) -> String {
    let trimmed = output.trim_end();
    if trimmed.is_empty() {
        format!("The `{command}` command completed with no output.")
    } else {
        format!("The `{command}` command completed. Output:\n\n```text\n{trimmed}\n```")
    }
}
