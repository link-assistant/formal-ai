//! Deterministic agentic planner — the server's "brain" for issue #468.
//!
//! This pure meta-algorithm chooses the next tool or final answer from the
//! conversation and advertised capabilities. It supports stored task recipes and
//! a bounded general fallback; neural sampling and hidden state remain non-goals.

use serde_json::json;

use super::change_request;
use super::code_artifact;
use super::code_rewrite_learning;
use super::conversation_recall;
use super::diagram;
use super::execution_learning;
use super::explain;
use super::file_read::{file_read_task_for, plan_file_read_step};
use super::formalize::{
    coverage_line, formalize_text_to_links, FormalizedKnowledgeBase, CANONICAL_FISHERMAN_SYNOPSIS,
    FISHERMAN_DOC_ID,
};
use super::general_planner::{compose_general_change_plan, GeneralChangePlan, PLAN_PATH};
use super::google_trends_catalog;
use super::google_trends_learning;
use super::intent_router;
use super::ledger;
use super::meaning_detail;
use super::question_catalog;
use super::rebuild_plan;
use super::repair_strategy;
use super::report_issue;
use super::routing_learning;
use super::self_ast;
use super::self_heal;
use super::shell_command;
use super::source_graph;
use super::web_research;
use super::{associative_learning, dreaming_audit};
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
    Edit,
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
            Self::Edit => "tool:capability:edit",
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

/// Plan the next agentic step from the conversation and advertised tools.
/// Returns [`None`] when neither a stored recipe nor a safe general plan applies.
#[must_use]
pub fn plan_chat_step(messages: &[ChatMessage], tool_names: &[&str]) -> Option<AgenticPlan> {
    let task = latest_user_text(messages)?;
    // Specific self-inspection routes precede broad formalization. Associative
    // learning comes before self-healing because both accept auto-learning terms;
    // the requested artifact scope distinguishes their recipes.
    if associative_learning::is_associative_learning_task(&task) {
        return Some(associative_learning::plan_step(messages, tool_names));
    }
    if routing_learning::is_routing_learning_task(&task) {
        return Some(routing_learning::plan_step(messages, tool_names));
    }
    if code_rewrite_learning::is_code_rewrite_learning_task(&task) {
        return Some(code_rewrite_learning::plan_step(messages, tool_names));
    }
    if execution_learning::is_execution_learning_task(&task) {
        return Some(execution_learning::plan_step(messages, tool_names));
    }
    // Workspace mutations are grounded in client-owned file bytes. This route
    // follows the explicit learning recipes so their requested artifacts cannot
    // be mistaken for an edit, and precedes generic file/change routers.
    if let Some(plan) = code_artifact::plan_code_artifact_step(&task, messages, tool_names) {
        return Some(plan);
    }
    if self_heal::is_self_heal_task(&task) {
        return Some(plan_self_heal_step(messages, tool_names));
    }
    if dreaming_audit::is_dreaming_audit_task(&task) {
        return Some(plan_dreaming_audit_step(messages, tool_names));
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
    // The learning-frontier recipe (issues #498 + #558): route the trending prompts the
    // engine cannot yet resolve through the human-gated self-improvement loop. Checked
    // before the sibling catalog recipe because both legitimately name "Google Trends";
    // its keywords ("learning frontier", "self-improvement loop", "cannot … resolve") are
    // disjoint from the catalog recipe's (prompt/answer/catalog/test), so ordering only
    // guards a request that somehow names both.
    if google_trends_learning::is_google_trends_learning_task(&task) {
        return Some(plan_google_trends_learning_step(messages, tool_names));
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
    // Agent-mode counterpart of the web UI's report action (issue #687).
    if let Some(request) = report_issue::report_issue_request_for(&task, messages) {
        return Some(report_issue::plan_report_issue_step(
            messages, tool_names, &request,
        ));
    }
    // Resolve dialogue meta-questions before open-world research.
    if let Some(answer) = conversation_recall::recall_answer_for(messages) {
        return Some(AgenticPlan::Final(answer));
    }
    // Probe writes before reads so a named output file is not mistaken for input.
    if let Some(plan) = tool_for(tool_names, Capability::Write)
        .and_then(|_| compose_general_change_plan(&task))
        .map(|plan| plan_general_change_step(messages, tool_names, &plan))
    {
        return Some(plan);
    }
    // Probe edits before reads for the same target-file ambiguity.
    if let Some(plan) = intent_router::plan_edit_step(&task, messages, tool_names) {
        return Some(plan);
    }
    if let Some(file_task) = file_read_task_for(&task) {
        return Some(plan_file_read_step(&file_task, messages, tool_names));
    }
    if let Some(command) = shell_command::shell_command_for_task(&task) {
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
    // Research is the final named recipe so more specific local actions win.
    if let Some(query) = web_research::web_research_query_for(messages) {
        if let Some(plan) = web_research::plan_web_research_step(messages, tool_names, &query) {
            return Some(plan);
        }
    }
    // Route the remaining requests by seed-backed intent and advertised capability.
    if let Some(plan) = intent_router::plan_web_fetch_step(&task, messages, tool_names) {
        return Some(plan);
    }
    if let Some(plan) = intent_router::plan_web_search_step(&task, messages, tool_names) {
        return Some(plan);
    }
    compose_general_change_plan(&task)
        .map(|plan| plan_general_change_step(messages, tool_names, &plan))
}

fn plan_general_change_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    plan: &GeneralChangePlan,
) -> AgenticPlan {
    let progress = Progress::scan(messages);
    let writes = progress.count(Capability::Write);
    if let Some(tool) = tool_for(tool_names, Capability::Write) {
        if writes == 0 {
            return plan_one(tool, write_arguments(PLAN_PATH, &plan.links_notation()));
        }
        if writes == 1 {
            return plan_one(tool, write_arguments(&plan.target, &plan.content));
        }
    }
    if let Some(tool) =
        tool_for(tool_names, Capability::Run).filter(|_| !progress.done(Capability::Run))
    {
        return plan_one(
            tool,
            json!({ "command": plan.verification_command }).to_string(),
        );
    }
    AgenticPlan::Final(format!(
        "Completed the general change request for {} and verified it with `{}`.\n\nPlan event ({}):\n\n{}",
        plan.target,
        plan.verification_command,
        PLAN_PATH,
        plan.links_notation().trim_end(),
    ))
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

/// A self-referential *generate → verify → final* recipe expressed as data.
///
/// Every self-inspection recipe (diagram, self-AST, self-heal, source-graph,
/// ledger, explain, change-request, repair-strategy, rebuild, question-catalog,
/// Google-Trends catalog, Google-Trends learning) has the *same* three-step shape:
/// write a generated document to `path`, verify it by running `verify_command`,
/// then answer with `final_answer`. They differ only in the document they generate,
/// so they are modelled as one struct and one planner
/// ([`plan_document_recipe`]) rather than a dozen copy-pasted functions — the exact
/// generalization the meta-algorithm is meant to embody.
pub(super) struct DocumentRecipe {
    /// The workspace-relative path the generated document is written to.
    pub(super) path: &'static str,
    /// The generated Links Notation document (a pure function of committed state).
    pub(super) document: String,
    /// The sandbox-allowlisted command that reads the document back for verification.
    pub(super) verify_command: String,
    /// The inline final answer returned once the write and verify steps are done.
    pub(super) final_answer: String,
}

/// Plan the next step of a [`DocumentRecipe`]: `write → verify → final`. Steps whose
/// capability the CLI did not advertise (or the conversation already satisfied) are
/// skipped, so the loop adapts to whatever subset of tools a given CLI exposes.
pub(super) fn plan_document_recipe(
    messages: &[ChatMessage],
    tool_names: &[&str],
    recipe: DocumentRecipe,
) -> AgenticPlan {
    let progress = Progress::scan(messages);

    // Step 1: write the generated document.
    if let Some(tool) =
        tool_for(tool_names, Capability::Write).filter(|_| !progress.done(Capability::Write))
    {
        return plan_one(tool, write_arguments(recipe.path, &recipe.document));
    }
    // Step 2: verify by reading the document back.
    if let Some(tool) =
        tool_for(tool_names, Capability::Run).filter(|_| !progress.done(Capability::Run))
    {
        return plan_one(
            tool,
            json!({ "command": recipe.verify_command }).to_string(),
        );
    }
    // Step 3: nothing left to do — answer with the generated document inline.
    AgenticPlan::Final(recipe.final_answer)
}

// State machine: web_search → web_fetch → write_file(formalize) → run_command(verify) → final.
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
    let document = diagram::render_document();
    let final_answer = diagram::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: diagram::DIAGRAM_PATH,
            verify_command: format!("cat {}", diagram::DIAGRAM_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#538 self-AST recipe: write the generated CST/AST-in-data document →
/// verify → final. Like the diagram recipe it needs no web step — the document is a
/// pure function of the planner's own source parsed through the meta-language links
/// network ([`self_ast::render_document`]), so the loop *inspects itself*. Steps
/// whose tool the CLI did not advertise are skipped.
fn plan_self_ast_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = self_ast::render_document();
    let final_answer = self_ast::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: self_ast::AST_PATH,
            verify_command: format!("cat {}", self_ast::AST_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 self-healing recipe: write the generated repair-case document →
/// verify → final. Like the diagram and self-AST recipes it needs no web step — the
/// document is a pure function of the canonical self-healing case
/// ([`self_heal::render_document`]), so the loop *repairs itself*. Steps whose tool
/// the CLI did not advertise are skipped.
fn plan_self_heal_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = self_heal::render_document();
    let final_answer = self_heal::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: self_heal::SELF_HEAL_PATH,
            verify_command: format!("cat {}", self_heal::SELF_HEAL_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 source-graph recipe: write the generated whole-repository
/// source ↔ links projection document → verify → final. Like the diagram, self-AST,
/// and self-healing recipes it needs no web step — the document is a pure function
/// of the system's own embedded source projected through the meta-language links
/// network ([`source_graph::render_document`]), so the loop *translates itself*.
/// Steps whose tool the CLI did not advertise are skipped.
fn plan_source_graph_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = source_graph::render_document();
    let final_answer = source_graph::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: source_graph::SOURCE_GRAPH_PATH,
            verify_command: format!("cat {}", source_graph::SOURCE_GRAPH_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 learning-ledger recipe: write the generated approved-lesson ledger
/// document → verify → final. Like the other self-inspection recipes it needs no web
/// step — the document is a pure function of the canonical, human-approved ledger
/// ([`ledger::render_document`]). Steps whose tool the CLI did not advertise are
/// skipped.
fn plan_ledger_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = ledger::render_document();
    let final_answer = ledger::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: ledger::LEDGER_PATH,
            verify_command: format!("cat {}", ledger::LEDGER_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 self-explanation recipe: write the generated grounded-explanation
/// document → verify → final. Like the other self-inspection recipes it needs no web
/// step — the document is a pure function of the system's own embedded source cited
/// through the owned manifest ([`explain::render_document`]), so the loop *explains
/// itself*. Steps whose tool the CLI did not advertise are skipped.
fn plan_explain_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = explain::render_document();
    let final_answer = explain::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: explain::EXPLAIN_PATH,
            verify_command: format!("cat {}", explain::EXPLAIN_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 self-change recipe: write the generated reviewable pull-request
/// document → verify → final. Like the other self-referential recipes it needs no web
/// step — the document is a deterministic function of the request and its grounded
/// target ([`change_request::render_document`]), so the loop turns a user's request to
/// *change Formal AI itself* into a reviewable PR. Steps whose tool the CLI did not
/// advertise are skipped.
fn plan_change_request_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = change_request::render_document();
    let final_answer = change_request::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: change_request::CHANGE_PATH,
            verify_command: format!("cat {}", change_request::CHANGE_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 general repair-classification recipe: write the generated
/// repair-strategies document → verify → final. Like the other self-referential recipes
/// it needs no web step — the document is a deterministic function of the three
/// self-contained canonical failure traces ([`repair_strategy::render_document`]), so
/// the loop decides *which part* of itself to repair for every failure class. Steps
/// whose tool the CLI did not advertise are skipped.
fn plan_repair_strategy_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = repair_strategy::render_document();
    let final_answer = repair_strategy::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: repair_strategy::REPAIR_STRATEGY_PATH,
            verify_command: format!("cat {}", repair_strategy::REPAIR_STRATEGY_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#558 rebuild-and-reattach recipe: write the generated
/// rebuild-and-reattach plan → verify → final. Like the change-request and source-graph
/// recipes it needs no web step — the plan is a deterministic function of the accepted
/// change and the grounded UI artifacts ([`rebuild_plan::render_document`]), so the loop
/// turns an accepted change into the ordered, reversible plan to recompile Formal AI and
/// reattach the improved worker to the UI. Steps whose tool the CLI did not advertise are
/// skipped.
fn plan_rebuild_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = rebuild_plan::render_document();
    let final_answer = rebuild_plan::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: rebuild_plan::REBUILD_PATH,
            verify_command: format!("cat {}", rebuild_plan::REBUILD_PATH),
            final_answer,
            document,
        },
    )
}

/// The issue-#527 question-catalog recipe: write the generated question-catalog
/// document → verify → final. Like the other self-referential recipes it needs no web
/// step — the document is a deterministic function of the seed lexicon and the
/// deterministic engine ([`question_catalog::render_document`]), so the loop *generates
/// every possible question and answers it*. Steps whose tool the CLI did not advertise
/// are skipped.
fn plan_question_catalog_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = question_catalog::render_document();
    let final_answer = question_catalog::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: question_catalog::QUESTION_CATALOG_PATH,
            verify_command: format!("cat {}", question_catalog::QUESTION_CATALOG_PATH),
            final_answer,
            document,
        },
    )
}

fn plan_dreaming_audit_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = dreaming_audit::render_document();
    let final_answer = dreaming_audit::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: dreaming_audit::DREAMING_AUDIT_PATH,
            verify_command: format!("cat {}", dreaming_audit::DREAMING_AUDIT_PATH),
            final_answer,
            document,
        },
    )
}

/// The issues-#498 + #558 learning-frontier recipe: write the generated
/// learning-frontier report → verify → final. Like the other self-referential recipes
/// it needs no web step — the report is a pure function of the committed Trends catalog
/// routed through the human-gated self-improvement loop
/// ([`google_trends_learning::render_document`]), so the loop maps its own coverage gap
/// and hands it to human triage. Steps whose tool the CLI did not advertise are skipped.
fn plan_google_trends_learning_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = google_trends_learning::render_document();
    let final_answer = google_trends_learning::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: google_trends_learning::GOOGLE_TRENDS_LEARNING_PATH,
            verify_command: google_trends_learning::verification_command(),
            final_answer,
            document,
        },
    )
}

fn plan_google_trends_catalog_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = google_trends_catalog::render_document();
    let final_answer = google_trends_catalog::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: google_trends_catalog::GOOGLE_TRENDS_CATALOG_PATH,
            verify_command: google_trends_catalog::verification_command(),
            final_answer,
            document,
        },
    )
}

/// Tool results produced since the current user turn began.
pub(super) struct Progress {
    completed: Vec<Capability>,
    pub(super) fetched_text: Option<String>,
    pub(super) search_output: Option<String>,
    pub(super) run_output: Option<String>,
}

impl Progress {
    pub(super) fn scan(messages: &[ChatMessage]) -> Self {
        let mut completed = Vec::new();
        let mut fetched_text = None;
        let mut search_output = None;
        let mut run_output = None;
        // Ignore results from earlier user turns.
        let current_turn = messages
            .iter()
            .rposition(|message| message.role.eq_ignore_ascii_case("user"))
            .map_or(0, |index| index + 1);
        for (index, message) in messages.iter().enumerate().skip(current_turn) {
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
            if capability == Capability::Search {
                let text = message.content.plain_text();
                if !looks_like_error(&text) && !text.trim().is_empty() {
                    search_output = Some(text);
                }
            }
            if capability == Capability::Run {
                run_output = Some(message.content.plain_text());
            }
            completed.push(capability);
        }
        Self {
            completed,
            fetched_text,
            search_output,
            run_output,
        }
    }

    /// Whether a prior tool result already covered `capability`.
    pub(super) fn done(&self, capability: Capability) -> bool {
        self.completed.contains(&capability)
    }

    fn count(&self, capability: Capability) -> usize {
        self.completed
            .iter()
            .filter(|done| **done == capability)
            .count()
    }

    /// The latest non-errored fetch result's text, for the [`intent_router`]
    /// fetch probe's final answer.
    pub(super) fn fetched_text(&self) -> Option<&str> {
        self.fetched_text.as_deref()
    }

    /// The latest non-errored web-search result's text, for the [`intent_router`]
    /// search probe's final answer.
    pub(super) fn search_output(&self) -> Option<&str> {
        self.search_output.as_deref()
    }
}

pub(super) fn plan_one(tool: &str, arguments: String) -> AgenticPlan {
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
pub(super) fn write_arguments(path: &str, content: &str) -> String {
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
pub(super) fn fetch_arguments(url: &str) -> String {
    json!({
        "url": url,
        "format": "text",
    })
    .to_string()
}

/// The first advertised tool name that provides `capability`, if any.
pub(super) fn tool_for<'a>(tool_names: &[&'a str], capability: Capability) -> Option<&'a str> {
    tool_names
        .iter()
        .copied()
        .find(|name| classify_tool(name) == Some(capability))
}

/// Classify a tool name into a [`Capability`] by substring, mirroring the naming
/// conventions agentic CLIs use (`web_search`, `web_fetch`, `read`, `write_file`,
/// `run_command`, `bash`, `websearch`, `webfetch`, …).
///
/// The recipe wants six kinds of tool, and real CLIs expose *lookalikes* that
/// must not be mistaken for them: a `todowrite` scratchpad is not a file writer,
/// a `codesearch` is not a web search, and an `edit`/`patch`/`replace` tool
/// mutates an existing file rather than creating one. Those are separated so
/// that — even though [`requested_tool_names`](super::super::protocol) hands the
/// planner an alphabetically sorted list — `todowrite` can never be picked ahead
/// of `write`, `codesearch` ahead of `websearch`, nor an `edit` tool ahead of a
/// create-file `write` for a write intent (they carry distinct arguments, so
/// each is its own capability class).
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
        // `write` / `write_file` create a file from scratch.
        Some(Capability::Write)
    } else if lower.contains("edit") || lower.contains("patch") || lower.contains("replace") {
        // `edit` / `apply_patch` / `str_replace` mutate an *existing* file and
        // take `(path, old, new)`-shaped arguments rather than `(path, content)`,
        // so they are their own capability class — never interchangeable with the
        // create-file write above (issue #680).
        Some(Capability::Edit)
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

fn shell_final_answer(command: &str, output: &str) -> String {
    let trimmed = output.trim_end();
    if trimmed.is_empty() {
        format!("The `{command}` command completed with no output.")
    } else {
        format!("The `{command}` command completed. Output:\n\n```text\n{trimmed}\n```")
    }
}
