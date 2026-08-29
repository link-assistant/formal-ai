//! Deterministic agentic planner for choosing the next tool or final answer from
//! a conversation and its advertised capabilities, without hidden neural state.

use serde_json::json;

pub(super) use super::capability_router::tool_for;
use super::code_task;
use super::comparison;
use super::conversation_recall;
use super::diagram;
use super::document_recipe::{
    plan_change_request_step, plan_diagram_step, plan_dreaming_audit_step,
    plan_explain_step, plan_google_trends_catalog_step, plan_google_trends_learning_step,
    plan_ledger_step, plan_meaning_detail_step, plan_question_catalog_step, plan_rebuild_step,
    plan_repair_strategy_step, plan_self_ast_step, plan_self_heal_step, plan_source_links_step,
};
use super::dreaming_audit;
use super::evidence_record;
use super::explain;
use super::file_read::{file_read_task_for, plan_file_read_step};
use super::formalization_recipe;
use super::general_execution::plan_general_change_step;
use super::general_planner::{
    compose_general_change_plan, has_authoritative_literal_write, objective_text,
};
use super::google_trends_catalog;
use super::google_trends_learning;
use super::intent_router;
use super::learning_report;
use super::ledger;
use super::local_search;
use super::meaning_detail;
use super::mutating_action;
use super::note_composition;
use super::procedure;
pub(super) use super::progress::Progress;
use super::question_catalog;
use super::rebuild_plan;
use super::repair_strategy;
use super::report_issue;
use super::self_ast;
use super::self_heal;
use super::shell_command;
use super::shell_file_fallback;
use super::source_links;
use super::statement_audit;
use super::structured_edit;
use super::task_structure;
use super::tool_result;
use super::web_research;
use super::{algorithm_learning, capability_router};
use super::{change_request, code_artifact};
use crate::conversation_control::is_conversation_control_prompt;
use crate::protocol::ChatMessage;
use crate::skill_compiler::looks_like_skill_description;

pub use super::formalization_recipe::{CANONICAL_SOURCE_URL, KB_PATH, SEARCH_QUERY};

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
    let received = latest_user_text(messages)?;
    trace_route("agentic_received", &received);
    // An *unmarked* harness preamble is still the caller talking (issue #907,
    // follow-up). `<session_context>`-style markup is stripped upstream in
    // `crate::protocol`, but Hive Mind's adapters concatenated their workflow
    // policy and the objective into one untagged user message, so the tell has
    // to be the objective delimiter the caller wrote instead of a tag:
    // everything before a line-anchored `Issue to solve:` / `Task:` / `Goal:`
    // lead is the caller's framing, and only the text after it is the request.
    //
    // Routing the whole message let the preamble win: "When running sudo
    // commands, run them in the background." paired a run verb with the `sudo`
    // shell token and planned bare `sudo`, and "Your prepared working
    // directory: …" planned `pwd`, in both cases dropping the repository work
    // that followed. The general planner already read the objective this way
    // (issue #904); every other route now reads it the same way, so one
    // boundary serves the whole router rather than one recipe.
    let task = objective_text(&received).to_owned();
    trace_route("agentic_task", &task);
    if is_conversation_control_prompt(&task) || looks_like_skill_description(&task) {
        return None;
    }
    // Issue #707: seed-defined computer-use plans own their exact multilingual
    // prompts before broad write/search routing. Each emitted primitive carries
    // explicit pre/postconditions and is executed by the advertising client.
    if let Some(plan) = crate::computer_use::plan_agentic_step(messages, tool_names) {
        return Some(plan);
    }
    // An explicit exact-content marker makes the following bytes authoritative.
    // Claim this narrow shape before edit/source semantics inspect the payload:
    // literal bytes may themselves say "rename X to Y" (issue #708). Broader
    // file-write requests remain below the semantic coding routes.
    if has_authoritative_literal_write(&task)
        && let Some(plan) = tool_for(tool_names, Capability::Write)
            .and_then(|_| compose_general_change_plan(&task))
            .map(|plan| plan_general_change_step(messages, tool_names, &plan))
        {
            return Some(plan);
        }
    // A learned workspace-change procedure owns grounded repository rewrites
    // and multi-file compositions before source creation or shell routing can
    // collapse them into one incomplete action.
    if let Some(plan) =
        super::workspace_change::plan_workspace_change_step(&task, messages, tool_names)
    {
        return Some(plan);
    }
    // A source-code description is not literal file content. Lower bounded
    // seed-backed source tasks before the broad literal-write parser so coding
    // requests produce executable bytes and verify those exact bytes.
    if let Some(plan) = code_task::plan_generated_source_step(&task, messages, tool_names) {
        return Some(plan);
    }
    if let Some(plan) = structured_edit::plan_structured_edit_step(&task, messages, tool_names) {
        return Some(plan);
    }
    // Resolve an unambiguous literal write before keyword recipes: arbitrary
    // filenames/payloads may legitimately contain "issue", "report", or "learning".
    // Unambiguous is the operative word: a request that also pins the target
    // file's opening line has not spelled its bytes out, and content recovered
    // from its prose would be written without that line (issue #1066).
    if let Some(plan) = tool_for(tool_names, Capability::Write)
        .and_then(|_| compose_general_change_plan(&task))
        .map(|plan| plan_general_change_step(messages, tool_names, &plan))
    {
        return Some(plan);
    }
    // Portable event logs own the independently validated trace-learning route.
    if let Some(task) = algorithm_learning::compile_task(&task) {
        return Some(algorithm_learning::plan_step(messages, tool_names, &task));
    }
    // A freely phrased procedure is one generalized compile → persist → verify
    // recipe on both the symbolic and Agent CLI surfaces.
    if let Some(procedure) = procedure::compile_task(&task) {
        return Some(procedure::plan_step(messages, tool_names, &procedure));
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
            statement_audit::command_for(&task),
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
    // "Find this out and leave the answer in FILE" (issue #1066). This sits ahead
    // of every route that reads a request's lone file-shaped token, because that
    // token is the *destination* here and opening it for reading ends the run with
    // the evidence file unwritten. It sits behind the literal-write routes above,
    // which own a request that spells its bytes out; this one owns the request
    // whose bytes still have to be found.
    if let Some(plan) = evidence_record::plan_evidence_record_step(&task, messages, tool_names) {
        return Some(plan);
    }
    if let Some(answer) = tool_result::follow_up_answer(messages, &task) {
        return Some(AgenticPlan::Final(answer));
    }
    if let Some(answer) = web_research::contextual_reference_clarification(&task) {
        return Some(AgenticPlan::Final(answer));
    }
    if web_research::is_definition_followup(&task) {
        if let Some(query) = web_research::definition_followup_topic(messages, &task) {
            if let Some(plan) = web_research::plan_web_research_step(messages, tool_names, &query) {
                return Some(plan);
            }
        } else {
            return Some(AgenticPlan::Final(
                web_research::definition_followup_clarification(&task),
            ));
        }
    }
    if let Some(plan) = intent_router::plan_edit_step(&task, messages, tool_names) {
        return Some(plan);
    }
    // Preserve the established stateful list/read recipe whenever the client
    // exposes its typed read capability. The shared read-many route remains
    // available for CLIs that advertise only a batch reader.
    if tool_for(tool_names, Capability::Read).is_some()
        && let Some(file_task) = file_read_task_for(&task) {
            return Some(plan_file_read_step(&file_task, messages, tool_names));
        }
    // A meanings-driven explicit local scope dominates generic search verbs.
    // This state machine observes each result and widens only after emptiness.
    if let Some(plan) = local_search::plan_local_search_step(messages, tool_names) {
        return Some(plan);
    }
    if let Some(plan) = comparison::plan_comparison_step(&task, messages, tool_names) {
        return Some(plan);
    }
    if let Some(plan) = capability_router::plan_shared_capability_step(&task, messages, tool_names)
    {
        return Some(plan);
    }
    if let Some(command) = shell_command::shell_command_for_task(&task) {
        if let Some(plan) = shell_file_fallback::plan_step(&task, messages, tool_names, &command) {
            return Some(plan);
        }
        // A command that changes the workspace answers by what the workspace
        // holds afterwards, so it is carried out as the verified recipe its seed
        // intent declares rather than issued once (issues #824 and #944).
        if let Some(plan) = mutating_action::plan_step(&command, messages, tool_names, &task) {
            return Some(plan);
        }
        return Some(plan_shell_step(messages, tool_names, &command));
    }
    if let Some(file_task) = file_read_task_for(&task) {
        return Some(plan_file_read_step(&file_task, messages, tool_names));
    }
    if formalization_recipe::is_formalization_task(&task) {
        return Some(formalization_recipe::plan_formalization_step(
            &task, messages, tool_names,
        ));
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
    // A request to look at the repository the agent was handed is answered by
    // reading that repository. It has to be resolved before the research
    // routers, which would otherwise claim it on the strength of its question
    // shape alone and look the answer up on the open web (issue #1066). The
    // subject rule inside `workspace_inspection_query_for_task` is what keeps a
    // genuinely external question out of this route.
    if !tool_result::has_latest_turn_result(messages)
        && let Some(query) = shell_command::workspace_inspection_query_for_task(&task)
            && let Some(tool) = tool_for(tool_names, Capability::Grep) {
                return Some(plan_one(
                    tool,
                    json!({ "query": query, "pattern": query }).to_string(),
                ));
            }
    // A question about how a task decomposes is answered by decomposing it. It
    // has to be resolved before the research routers for the same reason the
    // workspace inspection above does: the question shape alone would otherwise
    // send a task the web has never heard of to a web search (issue #1066).
    if let Some(plan) = task_structure::plan_task_structure_step(&task) {
        return Some(plan);
    }
    if let Some(query) = web_research::web_research_query_for(messages)
        && let Some(plan) = web_research::plan_web_research_step(messages, tool_names, &query) {
            return Some(plan);
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
    if !tool_result::has_latest_turn_result(messages)
        && let Some(query) = shell_command::code_search_query_for_task(&task)
            && let Some(tool) = tool_for(tool_names, Capability::Grep) {
                return Some(plan_one(
                    tool,
                    json!({ "query": query, "pattern": query }).to_string(),
                ));
            }
    if web_research::has_successful_search_result(messages)
        && let Some(query) = web_research::unresolved_web_research_query_for(messages)
            && let Some(plan) = web_research::plan_web_research_step(messages, tool_names, &query) {
                return Some(plan);
            }
    if let Some(answer) = tool_result::latest_turn_answer(messages, tool_names, &task) {
        return Some(AgenticPlan::Final(answer));
    }
    // A request that specifies what a document has to *cover* is answered by
    // composing that document. It sits here, after every route that could answer
    // one of the named parts outright, so a note is only composed once nothing
    // else claims the request -- and before the literal-write fallback, which
    // would otherwise write the specification instead of the document
    // (issue #1066).
    if let Some(plan) = note_composition::plan_note_composition_step(&task, messages) {
        return Some(plan);
    }
    if let Some(plan) = compose_general_change_plan(&task)
        .map(|plan| plan_general_change_step(messages, tool_names, &plan))
    {
        return Some(plan);
    }
    if let Some(query) = web_research::unresolved_web_research_query_for(messages)
        && let Some(plan) = web_research::plan_web_research_step(messages, tool_names, &query) {
            return Some(plan);
        }
    None
}

/// Run a shell command through the client-owned tool loop, then present its result.
fn plan_shell_step(messages: &[ChatMessage], tool_names: &[&str], command: &str) -> AgenticPlan {
    let progress = Progress::scan(messages);
    if progress.done(Capability::Run) {
        return AgenticPlan::Final(tool_result::render(
            command,
            progress.run_outputs.last().map_or("", String::as_str),
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
    crate::protocol::latest_user_request(messages)
}

/// Emit a `route=value` planner-routing trace line to stderr when
/// `FORMAL_AI_TRACE_REQUESTS=1`, mirroring the request tracing in
/// `crate::protocol`. Off by default; issue #956 asked for visibility into how
/// a received task was routed.
pub(super) fn trace_route(route: &str, value: &str) {
    if std::env::var("FORMAL_AI_TRACE_REQUESTS").as_deref() == Ok("1") {
        eprintln!("[trace] {route}={value}");
    }
}
