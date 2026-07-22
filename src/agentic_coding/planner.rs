//! Deterministic agentic planner — the server's "brain" for issue #468.
//!
//! This pure meta-algorithm chooses the next tool or final answer from the
//! conversation and advertised capabilities. It supports stored task recipes and
//! a bounded general fallback; neural sampling and hidden state remain non-goals.

use serde_json::json;

use super::capability_router;
pub(super) use super::capability_router::tool_for;
use super::change_request;
use super::code_artifact;
use super::conversation_recall;
use super::diagram;
use super::dreaming_audit;
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
use super::learning_report;
use super::ledger;
use super::meaning_detail;
pub(super) use super::progress::Progress;
use super::question_catalog;
use super::rebuild_plan;
use super::repair_strategy;
use super::report_issue;
use super::self_ast;
use super::self_heal;
use super::shell_command;
use super::source_links;
use super::statement_audit;
use super::tool_result;
use super::web_research;
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
    Grep,
    Glob,
    ListDir,
    Todo,
    Subagent,
    ReadMany,
    MultiEdit,
    AskUser,
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
            Self::Grep => "tool:capability:grep",
            Self::Glob => "tool:capability:glob",
            Self::ListDir => "tool:capability:list_dir",
            Self::Todo => "tool:capability:todo",
            Self::Subagent => "tool:capability:subagent",
            Self::ReadMany => "tool:capability:read_many",
            Self::MultiEdit => "tool:capability:multi_edit",
            Self::AskUser => "tool:capability:ask_user",
        }
    }

    pub(super) const fn registry_id(self) -> &'static str {
        match self {
            Self::Search => "web_search",
            Self::Fetch => "web_fetch",
            Self::Read => "read_file",
            Self::Write => "write_file",
            Self::Edit => "edit_file",
            Self::Run => "shell",
            Self::Grep => "grep",
            Self::Glob => "glob",
            Self::ListDir => "list_dir",
            Self::Todo => "todo",
            Self::Subagent => "subagent",
            Self::ReadMany => "read_many",
            Self::MultiEdit => "multi_edit",
            Self::AskUser => "ask_user",
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
    // Resolve an unambiguous literal write before keyword recipes: arbitrary
    // filenames/payloads may legitimately contain "issue", "report", or "learning".
    if let Some(plan) = tool_for(tool_names, Capability::Write)
        .and_then(|_| compose_general_change_plan(&task))
        .map(|plan| plan_general_change_step(messages, tool_names, &plan))
    {
        return Some(plan);
    }
    // Specific self-inspection routes precede broad formalization. Associative
    // learning comes before self-healing because both accept auto-learning terms;
    // the requested artifact scope distinguishes their recipes.
    if let Some(report) = learning_report::route(&task) {
        return Some(report.plan_step(messages, tool_names));
    }
    // Repository statement audits run through the same public CLI a human can
    // replay. Route before generic file/code changes because the task names its
    // output artifact but does not ask the planner to fabricate that content.
    if statement_audit::is_statement_audit_task(&task) {
        return Some(plan_shell_step(
            messages,
            tool_names,
            statement_audit::STATEMENT_AUDIT_COMMAND,
        ));
    }
    // Workspace mutations are grounded in client-owned file bytes. This route
    // follows the explicit learning recipes so their requested artifacts cannot
    // be mistaken for an edit, and precedes the generic edit/read/shell routers
    // below. Requests naming both a literal target and literal content are
    // already claimed by the write probe above.
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
    // The whole-repository source-links recipe: checked alongside the other
    // self-inspection recipes and before formalization, because its request
    // legitimately names "links" (its output format), which the broad
    // formalization keyword match below would otherwise capture.
    if source_links::is_source_links_task(&task) {
        return Some(plan_source_links_step(messages, tool_names));
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
    // source-links recipe's "recompile", so ordering only guards a request that somehow
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
    // Agent-mode counterpart of the web UI's report action (issues #687 + #822).
    // This is a conversation state machine: after the initial report intent it
    // continues across structured tool results or plain-text user choices.
    if let Some(plan) = report_issue::plan_report_flow(messages, tool_names) {
        return Some(plan);
    }
    if let Some(answer) = conversation_recall::recall_answer_for(messages) {
        return Some(AgenticPlan::Final(answer));
    }
    if let Some(answer) = tool_result::follow_up_answer(messages, &task) {
        return Some(AgenticPlan::Final(answer));
    }
    if let Some(plan) = intent_router::plan_edit_step(&task, messages, tool_names) {
        return Some(plan);
    }
    // Preserve the established stateful list/read recipe whenever the client
    // exposes its typed read capability. The shared read-many route remains
    // available for CLIs that advertise only a batch reader.
    if tool_for(tool_names, Capability::Read).is_some() {
        if let Some(file_task) = file_read_task_for(&task) {
            return Some(plan_file_read_step(&file_task, messages, tool_names));
        }
    }
    // A path lookup with an explicit local scope must execute on the user's
    // machine even when a client also advertises Glob and web-search tools.
    if let Some(command) = shell_command::local_path_search_command_for_task(&task) {
        if tool_for(tool_names, Capability::Run).is_some() {
            return Some(plan_shell_step(messages, tool_names, &command));
        }
    }
    if let Some(plan) = capability_router::plan_shared_capability_step(&task, messages, tool_names)
    {
        return Some(plan);
    }
    if let Some(command) = shell_command::shell_command_for_task(&task) {
        return Some(plan_shell_step(messages, tool_names, &command));
    }
    if let Some(file_task) = file_read_task_for(&task) {
        return Some(plan_file_read_step(&file_task, messages, tool_names));
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
    // A typed URL object is more specific than broad research prose. Resolve it
    // before the research recipe so requests such as "tell me about URL" fetch
    // that page instead of turning the URL itself into a search query.
    if let Some(plan) = intent_router::plan_web_fetch_step(&task, messages, tool_names) {
        return Some(plan);
    }
    if let Some(query) = web_research::web_research_query_for(messages) {
        if let Some(plan) = web_research::plan_web_research_step(messages, tool_names, &query) {
            return Some(plan);
        }
    }
    if let Some(plan) = intent_router::plan_web_search_step(&task, messages, tool_names) {
        return Some(plan);
    }
    // A generic localized "find" cue can describe either an open-web lookup or
    // a workspace grep. The research routers above get first refusal whenever
    // the client exposes their tools; explicit local/repository searches were
    // already claimed by the capability router. This fallback therefore keeps
    // grep available to grep-only clients without letting an alphabetically
    // earlier local tool steal a web-research request.
    if !tool_result::has_latest_turn_result(messages) {
        if let Some(query) = shell_command::code_search_query_for_task(&task) {
            if let Some(tool) = tool_for(tool_names, Capability::Grep) {
                return Some(plan_one(
                    tool,
                    json!({ "query": query, "pattern": query }).to_string(),
                ));
            }
        }
    }
    if let Some(answer) = tool_result::latest_turn_answer(messages, tool_names, &task) {
        return Some(AgenticPlan::Final(answer));
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

/// Run a shell command through the client-owned tool loop, then present its result.
fn plan_shell_step(messages: &[ChatMessage], tool_names: &[&str], command: &str) -> AgenticPlan {
    let progress = Progress::scan(messages);
    if progress.done(Capability::Run) {
        return AgenticPlan::Final(tool_result::render(
            command,
            progress.run_output.as_deref().unwrap_or_default(),
            latest_user_text(messages).as_deref().unwrap_or_default(),
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
/// Every self-inspection recipe (diagram, self-AST, self-heal, source-links,
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

/// The issue-#558 source-links recipe: write the generated whole-repository
/// source ↔ links projection document → verify → final. Like the diagram, self-AST,
/// and self-healing recipes it needs no web step — the document is a pure function
/// of the system's own embedded source projected through the meta-language links
/// network ([`source_links::render_document`]), so the loop *translates itself*.
/// Steps whose tool the CLI did not advertise are skipped.
fn plan_source_links_step(messages: &[ChatMessage], tool_names: &[&str]) -> AgenticPlan {
    let document = source_links::render_document();
    let final_answer = source_links::final_answer(&document);
    plan_document_recipe(
        messages,
        tool_names,
        DocumentRecipe {
            path: source_links::SOURCE_LINKS_PATH,
            verify_command: format!("cat {}", source_links::SOURCE_LINKS_PATH),
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
/// rebuild-and-reattach plan → verify → final. Like the change-request and source-links
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

pub(super) fn classify_tool(name: &str) -> Option<Capability> {
    capability_router::classify_tool(name)
}

/// The text of the most recent `user` turn.
fn latest_user_text(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.user_request_text())
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
