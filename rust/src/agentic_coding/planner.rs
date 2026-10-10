//! Deterministic agentic planner for choosing the next tool or final answer from
//! a conversation and its advertised capabilities, without hidden neural state.

use serde_json::json;
mod continuation;
mod obligations;
mod precedence;
mod steps;
use super::final_result::{FinalDisposition, FinalResult, ResolvedPlan, record};
use continuation::continued_agent_task;
pub use continuation::trace_route;
pub(in crate::agentic_coding) use continuation::{evidence_window_start, is_continuation_cue};
pub use precedence::checked_route_precedence;
use steps::{stop_repeated_call, stop_repeated_failure};

pub(super) use super::capability_router::tool_for;
use super::code_task;
use super::comparison;
use super::conversation_recall;
use super::diagram;
use super::document_recipe::{
    plan_change_request_step, plan_diagram_step, plan_dreaming_audit_step, plan_explain_step,
    plan_google_trends_catalog_step, plan_google_trends_learning_step, plan_ledger_step,
    plan_meaning_detail_step, plan_question_catalog_step, plan_rebuild_step,
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
    plan_owned_goal_step,
};
use super::git_commit;
use super::google_trends_catalog;
use super::google_trends_learning;
use super::harness_envelope;
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
use super::structured_document;
use super::structured_edit;
use super::task_obligations;
use super::task_structure;
use super::tool_result;
use super::web_research;
use super::workspace_inspection;
use super::workspace_search;
use super::{algorithm_learning, capability_router};
use super::{change_request, code_artifact};
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
    capability_router::classify_tool(name)
}

/// Plan the next agentic step from the conversation and advertised tools.
/// Returns [`None`] when neither a stored recipe nor a safe general plan applies.
#[must_use]
pub fn plan_chat_step(messages: &[ChatMessage], tool_names: &[&str]) -> Option<AgenticPlan> {
    plan_chat_step_resolved(messages, tool_names).map(|resolved| resolved.plan)
}

pub(super) fn plan_chat_step_resolved(
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<ResolvedPlan> {
    let mut result = None;
    let plan = plan_chat_step_inner(messages, tool_names, &mut result)?;
    Some(ResolvedPlan::new(plan, result))
}

fn plan_chat_step_inner(
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let received = crate::protocol::latest_user_request(messages)?;
    // A harness prompt that quotes the task to have it summarized is answered
    // with the summary, never by doing the task again (issue #1133).
    if let Some(summary) = harness_envelope::summarize_request(&received) {
        return Some(record(
            AgenticPlan::Final(summary),
            FinalDisposition::Finding,
            "harness_summary",
            result,
        ));
    }
    // A restart over prepared work (changed paths plus a pull request link)
    // is the whole turn's shape, not one arm of the precedence cascade: it is
    // decided on the same effective request the cascade reads, ahead of it,
    // and its plan still passes the repeated-call and repeated-failure stops.
    let effective = continued_agent_task(messages, &received);
    let restart = super::restart_feedback::plan_restart(
        effective.as_deref().unwrap_or(&received),
        messages,
        tool_names,
    );
    let plan = restart
        .or_else(|| plan_chat_step_routes(messages, tool_names, received.clone(), result))
        .filter(|plan| !super::file_read::read_policy_blocks_plan(&received, plan))?;
    let was_tool_calls = matches!(plan, AgenticPlan::ToolCalls(_));
    let stopped = stop_repeated_failure(stop_repeated_call(plan, messages), messages);
    Some(
        if was_tool_calls && matches!(stopped, AgenticPlan::Final(_)) {
            record(
                stopped,
                FinalDisposition::Failure,
                "repeated_step_stopped",
                result,
            )
        } else {
            stopped
        },
    )
}

fn plan_chat_step_routes(
    messages: &[ChatMessage],
    tool_names: &[&str],
    received: String,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    // Load-time precedence join (plan 10 leaf 18): first plan step of a run
    // proves the seed still names the cascade this function is about to walk.
    checked_route_precedence();
    trace_route("agentic_received", &received);
    // Agent compacts a long tool loop by asking the model to summarize it, then
    // starts the next request with the protocol turn "Continue if you have next
    // steps". The work is still present, but only inside the assistant's
    // `Conversation summary:` envelope. Treating the protocol turn as a fresh
    // request loses that work and sends the sentence itself to web search.
    // Recover only our own summary envelope, immediately before the exact
    // continuation turn; ordinary user requests that happen to say "continue"
    // keep their normal meaning.
    let effective = continued_agent_task(messages, &received).unwrap_or(received);
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
    let task = objective_text(&effective).to_owned();
    trace_route("agentic_task", &task);
    // A bare continuation cue with nothing to resume is still the cue here,
    // and the `agentic_continuation` conversation handler answers it; the
    // words of the cue are never a request (issue #1095).
    if crate::rule_interpreter::handler_matches("conversation_control", &task)
        || is_continuation_cue(&task)
        // An edit request's block is its payload: a `when … then` inside it is
        // text being written, not a skill being taught (PR #1188 T57).
        || (!has_authoritative_literal_write(&task)
            && super::general_planner::compose_edit_request(&task).is_none()
            && looks_like_skill_description(super::positional_edit::own_text(&task)))
    {
        return None;
    }
    // Issue #707: seed-defined computer-use plans own their exact multilingual
    // prompts before broad write/search routing. Each emitted primitive carries
    // explicit pre/postconditions and is executed by the advertising client.
    // Ahead of them, quotes that do not pair leave no telling the quoted text
    // from the instruction, so the request is declined before any arm reads its
    // payload as words to act on (PR #1188 G71).
    let source = code_task::verified_source_description(&task)
        .map(|_| steps::plan_verified_source_step(&task, messages, tool_names, result));
    if let Some(plan) = source.or_else(|| {
        let owned_goal = if evidence_record::has_typed_evidence_delivery(&task) {
            None
        } else {
            plan_owned_goal_step(&task, messages, tool_names, plan_chat_step_resolved, result)
        };
        owned_goal
            .or_else(|| {
                super::quote_nesting::request_fault_answer(
                    &task,
                    tool_for(tool_names, Capability::MultiEdit).is_some(),
                )
            })
            .or_else(|| crate::computer_use::plan_agentic_step(messages, tool_names))
            .map(Some)
    }) {
        return plan;
    }
    // An explicit exact-content marker makes the following bytes authoritative.
    // Claim this narrow shape before edit/source semantics inspect the payload:
    // literal bytes may themselves say "rename X to Y" (issue #708). Broader
    // file-write requests remain below the semantic coding routes.
    if has_authoritative_literal_write(&task)
        && capability_router::workspace_creation_tool(tool_names).is_some()
    {
        if let Some(nodes) = task_obligations::obligations(&task) {
            return obligations::plan_obligations_step(&task, messages, tool_names, &nodes, result);
        }
        if let Some(plan) = compose_general_change_plan(&task) {
            return Some(plan_general_change_step(
                messages, tool_names, &plan, result,
            ));
        }
    }
    // Bind a program's semantic operands before treating its source path as a
    // destination for a report about the rest of the request.
    if let Some(mut answer) =
        crate::coding::program_contract::answer(&task, &mut crate::event_log::EventLog::default())
    {
        if let Some(recipe) = answer.execution_recipe.as_mut()
            && super::ci_workflow::requested_in(&task)
        {
            super::ci_workflow::attach(recipe);
        }
        if let Some(plan) =
            super::command_reroute::plan_symbolic_command_reroute(messages, tool_names, &answer)
        {
            return Some(plan);
        }
    }
    // "Find this out and leave the answer in FILE" (issue #1066). This sits ahead
    // of every route that reads a request's lone file-shaped token, because that
    // token is the *destination* here and opening it for reading ends the run with
    // the evidence file unwritten. It sits behind the literal-write routes above,
    // which own a request that spells its bytes out; this one owns the request
    // whose bytes still have to be found.
    //
    // It sits ahead of the change routes below for the same reason it sits ahead
    // of the readers: a request can carry both halves. The ladder's leaf says
    // "Edit `src/engine_responses.rs` … Then create `agent-ladder-effects/…lino`
    // recording what you changed. Leave evidence in `.agent-ladder/…-proof.md`",
    // and the change routes answer `Final` for the whole request the moment the
    // edit lands, so the two records the caller verifies are never written and
    // the node fails `missing_proof` having done the work. This route peels one
    // delivery at a time and re-plans the residual (see `parse_obligation`), so
    // the change route still receives the edit -- with only the edit left in it.
    if let Some(plan) =
        evidence_record::plan_evidence_record_step(&task, messages, tool_names, result)
    {
        return Some(plan);
    }
    plan_settled_routes(&task, messages, tool_names, result)
}

/// Every route below the delivery peeling above, as one function.
///
/// [`plan_evidence_record_step`](evidence_record::plan_evidence_record_step)
/// peels a named destination off a request and re-plans the remainder, and that
/// is only ever the right reading when the destination is not already some
/// other route's whole answer. A registered recipe *is* named by its artifact —
/// `learning_report::route` matches a prompt precisely by finding its own
/// `path` in it — so peeling that path off leaves a residual that no longer
/// reaches the recipe, and the request is answered by whatever the remainder
/// happens to look like instead.
///
/// Splitting the tail out lets the peeling ask the question directly: plan the
/// *whole* request through the routes below and see whether one of them writes
/// the file. Nothing here re-enters the peeling, so asking cannot recurse.
pub(super) fn plan_settled_routes(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let instruction = super::general_planner::instruction_view_for_request(task);
    let owned = instruction.as_str();
    // A request to commit what is already in the tree is one shell step. It
    // is claimed first because its words ("review these changes and commit
    // them", with a `?? Main.scala` listing) read to later routes as a search
    // for the file; the route itself declines any request that also names the
    // work to do (issue #1133).
    if let Some(plan) = steps::explicit_shell_step(task, messages, tool_names, result)
        .or_else(|| {
            code_task::plan_verified_generated_source_step(task, messages, tool_names, result)
        })
        .or_else(|| git_commit::plan_commit_step(owned, messages, tool_names))
    {
        return Some(plan);
    }
    // A learned workspace-change procedure owns grounded repository rewrites
    // and multi-file compositions before source creation or shell routing can
    // collapse them into one incomplete action. A function and its test added
    // to existing modules (PR #1188 T1) is one such composition: read both
    // modules, write both, run the stated command.
    // A copy or move followed by edits of the file it makes is planned
    // sentence by sentence (PR #1188 G82).
    if let Some(plan) = super::request_sequence::plan_request_sequence_step(
        owned,
        messages,
        tool_names,
        plan_chat_step_resolved,
        result,
    )
    .or_else(|| {
        super::workspace_change::plan_workspace_change_step(owned, messages, tool_names, result)
    })
    .or_else(|| {
        super::module_function::plan_module_function_step(owned, messages, tool_names, result)
    })
    // A bug report with a stated expectation is checked before anything is
    // rewritten (PR #1188 T93).
    .or_else(|| {
        super::function_expectation::plan_function_expectation_step(
            owned, messages, tool_names, result,
        )
    })
    // An assertion of a stated call and value is added in the test
    // file's own form, and the file is run (PR #1188 G13).
    .or_else(|| {
        super::test_assertion::plan_test_assertion_step(owned, messages, tool_names, result)
    })
    // A test asked for with no expected result is a question (PR #1188
    // G25).
    .or_else(|| super::function_expectation::test_expectation_question(owned, result))
    {
        return Some(plan);
    }
    // A source-code description is not literal file content. Lower bounded
    // seed-backed source tasks before the broad literal-write parser so coding
    // requests produce executable bytes and verify those exact bytes.
    if let Some(plan) = code_task::plan_generated_source_step(owned, messages, tool_names, result) {
        return Some(plan);
    }
    if let Some(plan) =
        structured_edit::plan_structured_edit_step(owned, messages, tool_names, result)
    {
        return Some(plan);
    }
    // A source-backed structured document is a read/derive/write transaction.
    // It must win before the ordinary file reader, which would otherwise read
    // the input correctly and then mistake that intermediate observation for
    // the answer to the whole authored-artifact request.
    if let Some(plan) = structured_document::plan_step(owned, messages, tool_names) {
        return Some(plan);
    }
    // A repository audit names the artifact its CLI command will produce; that
    // filename is a destination, not literal content for the generic writer.
    // Keep the replayable recipe ahead of literal fallback so an observed CLI
    // result completes the audit instead of starting a redundant write plan.
    if statement_audit::is_statement_audit_task(task) {
        return Some(plan_shell_step(
            messages,
            tool_names,
            statement_audit::command_for(task),
            result,
        ));
    }
    // Resolve an unambiguous literal write before keyword recipes: arbitrary
    // filenames/payloads may legitimately contain "issue", "report", or "learning".
    // Unambiguous is the operative word: a request that also pins the target
    // file's opening line has not spelled its bytes out, and content recovered
    // from its prose would be written without that line (issue #1066).
    // Plan 16 L2g: a request that names a source-tree file (`js/app.js … to
    // typescript`) is the meta pivot's translate family, and must be claimed
    // before the literal-write and obligation composers read "write it" as an
    // artifact write of the goal text. The structural gate (a path token plus
    // the target's canonical spelling) is cheap; the family itself is decided
    // by the shared solver, whose bridge lowers it to one `translate` tool
    // call — or answers with the rendered gap on a client without the tool.
    if crate::meta_translate::source_tree_request(task).is_some() {
        match super::conversation_recall::plan_shared_solver_step(messages, tool_names) {
            super::conversation_recall::SharedSolverStep::Ready(plan) => return Some(plan),
            super::conversation_recall::SharedSolverStep::NotOurs
            | super::conversation_recall::SharedSolverStep::Defer => {}
        }
    }
    // An enumerated request is planned one obligation at a time, and is not
    // successfully finished until the runtime ledger has discharged every
    // node with at least one execution record (issues #1099 and #1138 B5).
    // Single-clause requests deliberately bypass this path and keep the
    // established whole-request composer below. So does a request the shell
    // executes as one command and no composer reads as an artifact write:
    // "show me the working directory: then list its files" splits into clauses
    // whose first half names no artifact, and the ledger reported a gap where
    // the shell cascade answers the whole request (issue #907). The semantic
    // resolver, not the general one: "copy" inside "a fresh repository copy"
    // matches a shell verb surface without the request being a command.
    let shell_owned = super::shell_command::semantic_shell_command_for_task(task).is_some()
        && compose_general_change_plan(task).is_none();
    if capability_router::workspace_creation_tool(tool_names).is_some()
        && !shell_owned
        && let Some(obligations) = task_obligations::obligations(task)
    {
        return obligations::plan_obligations_step(
            task,
            messages,
            tool_names,
            &obligations,
            result,
        );
    }
    if let Some(plan) = capability_router::workspace_creation_tool(tool_names)
        .and_then(|_| compose_general_change_plan(task))
        .map(|plan| plan_general_change_step(messages, tool_names, &plan, result))
    {
        return Some(plan);
    }
    // Portable event logs own the independently validated trace-learning route.
    if let Some(task) = algorithm_learning::compile_task(task) {
        return Some(algorithm_learning::plan_step(messages, tool_names, &task));
    }
    // A freely phrased procedure is one generalized compile → persist → verify
    // recipe on both the symbolic and Agent CLI surfaces.
    if let Some(procedure) = procedure::compile_task(task) {
        return Some(procedure::plan_step(messages, tool_names, &procedure));
    }
    // Specific self-inspection routes precede broad formalization. Associative
    // learning comes before self-healing because both accept auto-learning terms;
    // the requested artifact scope distinguishes their recipes.
    if let Some(report) = learning_report::route(task) {
        return Some(report.plan_step(messages, tool_names));
    }
    // Workspace mutations are grounded in client-owned file bytes. This route
    // follows the explicit learning recipes so their requested artifacts cannot
    // be mistaken for an edit, and precedes the generic edit/read/shell routers
    // below. Requests naming both a literal target and literal content are
    // already claimed by the write probe above.
    if let Some(plan) =
        code_artifact::plan_code_artifact_step(task, messages, tool_names).or_else(|| {
            (super::general_planner::owned_semantic_authoring_lead(task)
                && shell_command::semantic_shell_command_for_task(task).is_none()
                && compose_general_change_plan(task).is_none()
                && super::general_planner::compose_edit_request(task).is_none())
            .then(|| {
                record(
                    AgenticPlan::Final(super::general_planner::missing_implementation_contract(
                        task,
                    )),
                    FinalDisposition::Gap,
                    "semantic-authoring-missing-contract",
                    result,
                )
            })
        })
    {
        return Some(plan);
    }
    if self_heal::is_self_heal_task(task) {
        return Some(plan_self_heal_step(messages, tool_names));
    }
    if dreaming_audit::is_dreaming_audit_task(task) {
        return Some(plan_dreaming_audit_step(messages, tool_names));
    }
    if self_ast::is_self_ast_task(task) {
        return Some(plan_self_ast_step(messages, tool_names));
    }
    // The whole-repository source-links recipe: checked alongside the other
    // self-inspection recipes and before formalization, because its request
    // legitimately names "links" (its output format), which the broad
    // formalization keyword match below would otherwise capture.
    if source_links::is_source_links_task(task) {
        return Some(plan_source_links_step(messages, tool_names));
    }
    // The learning-ledger recipe: the promotion step that follows an approved repair
    // case. Checked after self-healing (which owns the "auto learning" keywords) and
    // before formalization, since its request legitimately names "Links Notation".
    if ledger::is_ledger_task(task) {
        return Some(plan_ledger_step(messages, tool_names));
    }
    // The grounded self-explanation recipe: answers "how does Formal AI work?" from
    // real source/data/test artifacts. Checked alongside the other self-inspection
    // recipes and before formalization, since its request legitimately names "Links
    // Notation" as the output format its document is rendered in.
    if explain::is_explain_task(task) {
        return Some(plan_explain_step(messages, tool_names));
    }
    // The user-initiated self-change recipe: turns a natural-language "change Formal AI
    // itself" request into a reviewable pull request through the same human-gated loop.
    // Checked alongside the other self-referential recipes and before formalization,
    // since its request legitimately names "Links Notation" as the output format.
    if change_request::is_change_request_task(task) {
        return Some(plan_change_request_step(messages, tool_names));
    }
    // The general repair-classification recipe: given an arbitrary failure trace, decide
    // whether the repair is a solver method, a data record, or a test, and compose the
    // grounded, human-gated strategy for each class. Checked alongside the other
    // self-referential recipes and before formalization, since its request legitimately
    // names "Links Notation" as the output format its strategies are rendered in. Its
    // keywords are disjoint from the self-healing recipe's ("repair case"/"repair loop"),
    // so ordering only guards a request that somehow names both.
    if repair_strategy::is_repair_strategy_task(task) {
        return Some(plan_repair_strategy_step(messages, tool_names));
    }
    // Rebuild-and-reattach recipe: once a change is accepted, recompile Formal AI and
    // reattach the improved WebAssembly worker to the UI (issue #558's `R558-06`).
    // Checked alongside the other self-referential recipes and before formalization,
    // since its request legitimately names "Links Notation" as the output format its plan
    // is rendered in. Its keywords key on "reattach" and are disjoint from the
    // source-links recipe's "recompile", so ordering only guards a request that somehow
    // names both.
    if rebuild_plan::is_rebuild_task(task) {
        return Some(plan_rebuild_step(messages, tool_names));
    }
    // The learning-frontier recipe (issues #498 + #558): route the trending prompts the
    // engine cannot yet resolve through the human-gated self-improvement loop. Checked
    // before the sibling catalog recipe because both legitimately name "Google Trends";
    // its keywords ("learning frontier", "self-improvement loop", "cannot … resolve") are
    // disjoint from the catalog recipe's (prompt/answer/catalog/test), so ordering only
    // guards a request that somehow names both.
    if google_trends_learning::is_google_trends_learning_task(task) {
        return Some(plan_google_trends_learning_step(messages, tool_names));
    }
    if google_trends_catalog::is_google_trends_catalog_task(task) {
        return Some(plan_google_trends_catalog_step(messages, tool_names));
    }
    // The question-catalog recipe (issue #527): enumerate every possible question
    // smallest-first, classify each grammatically and logically, and answer the
    // meaningful ones. Checked alongside the other self-referential recipes and before
    // formalization, since its request legitimately names "Links Notation" as the output
    // format its catalog is rendered in. Its keywords ("question catalog", "all possible
    // questions", …) are disjoint from the sibling recipes', so ordering only guards a
    // request that somehow names both.
    if question_catalog::is_question_catalog_task(task) {
        return Some(plan_question_catalog_step(messages, tool_names));
    }
    // A request to inspect named files and report what it finds is a file
    // analysis, not a request to open a repository issue. In particular, a
    // `.github/...` path supplies the report router's otherwise-valid subject
    // word. Let the typed read + audit object govern the output verb before the
    // conversation-level report wizard sees it (issue #1138 self-use).
    // After it, where a name or literal is used inside the workspace is a
    // content search (PR #1188 T90): grep it, ahead of the file-name locate arm
    // and web search.
    if let Some(plan) = file_read_task_for(task)
        .filter(super::file_read::FileReadTask::is_analysis)
        .map(|file_task| plan_file_read_step(&file_task, messages, tool_names, result))
        .or_else(|| workspace_search::plan_workspace_search_step(task, messages, tool_names))
        // What a named module exports is answered from its declarations (T99).
        .or_else(|| {
            super::module_exports::plan_module_exports_step(task, messages, tool_names, result)
        })
        // A summary of a named file summarizes the file's text (T100).
        .or_else(|| super::file_summary::plan_file_summary_step(task, messages, tool_names, result))
    {
        return Some(plan);
    }
    // Agent-mode counterpart of the web UI's report action (issues #687 + #822).
    // This is a conversation state machine: after the initial report intent it
    // continues across structured tool results or plain-text user choices.
    if let Some(plan) = report_issue::plan_report_flow(messages, tool_names) {
        return Some(plan);
    }
    match conversation_recall::plan_shared_solver_step(messages, tool_names) {
        conversation_recall::SharedSolverStep::Ready(plan) => return Some(plan),
        // The shared solver's typed recipe owns the turn and the client can run
        // it: stop routing and yield the request so the server's recipe path
        // delivers the verified write--check--run chain (issue #916) -- falling
        // through instead would let the research arms below steal the prompt
        // back (issue #936).
        conversation_recall::SharedSolverStep::Defer => return None,
        conversation_recall::SharedSolverStep::NotOurs => {}
    }
    if let Some(answer) = tool_result::follow_up_answer(messages, task) {
        return Some(AgenticPlan::Final(answer));
    }
    if let Some(answer) = web_research::contextual_reference_clarification(task) {
        return Some(AgenticPlan::Final(answer));
    }
    if web_research::is_definition_followup(task) {
        if let Some(query) = web_research::definition_followup_topic(messages, task) {
            if let Some(plan) =
                web_research::plan_web_research_step(messages, tool_names, &query, true)
            {
                return Some(plan);
            }
        } else {
            return Some(AgenticPlan::Final(
                web_research::definition_followup_clarification(task),
            ));
        }
    }
    if let Some(plan) = intent_router::plan_edit_step(task, messages, tool_names) {
        return Some(plan);
    }
    // Preserve the established stateful list/read recipe whenever the client
    // exposes its typed read capability. The shared read-many route remains
    // available for CLIs that advertise only a batch reader.
    if tool_for(tool_names, Capability::Read).is_some()
        && let Some(file_task) = file_read_task_for(task)
    {
        return Some(plan_file_read_step(
            &file_task, messages, tool_names, result,
        ));
    }
    // A meanings-driven explicit local scope dominates generic search verbs.
    // This state machine observes each result and widens only after emptiness.
    if let Some(plan) = local_search::plan_local_search_step(messages, tool_names) {
        return Some(plan);
    }
    if let Some(plan) = comparison::plan_comparison_step(task, messages, tool_names) {
        return Some(plan);
    }
    // Plan 10 leaf 11: the shared cue-phrase arm that stood here is retired, and its *position*
    // is kept -- because position is what the seven named capabilities still need. The decision
    // table decides them now by `(object, act, locus)` with no phrase scan, and it decides them
    // here, ahead of the shell cascade, so a request whose row names an advertised `grep`, `glob`,
    // `list_dir`, `read_many`, `multi_edit`, `todo` or `subagent` is not answered by the shell
    // lowering that same row declares as its fallback. A row the table cannot decide here (no
    // row, or a capability outside the seven) leaves the cascade exactly as it was.
    if let Some(plan) = capability_router::plan_named_capability_step(task, messages, tool_names) {
        return Some(plan);
    }
    if let Some(plan) = plan_shell_command_arm(task, messages, tool_names, result) {
        return Some(plan);
    }
    if let Some(file_task) = file_read_task_for(task) {
        return Some(plan_file_read_step(
            &file_task, messages, tool_names, result,
        ));
    }
    if formalization_recipe::is_formalization_task(task) {
        return Some(formalization_recipe::plan_formalization_step(
            task, messages, tool_names,
        ));
    }
    if meaning_detail::is_meaning_detail_task(task) {
        return Some(plan_meaning_detail_step(task, messages, tool_names));
    }
    if diagram::is_diagram_task(task) {
        return Some(plan_diagram_step(messages, tool_names));
    }
    // Plan 10 leaf 19 (issue #1138): the URL route that stood here is retired.
    // The decision table's five `url` rows — one per act — decide that request
    // class at the named-or-local stage below, with the same `fetch_arguments`
    // lowering this arm produced, so the arm was a second URL decision ahead of
    // its own table backstop. The table also guards what this arm never did: a
    // URL named while a research recipe is already under way is the recipe's
    // to continue, not a one-step fetch that ends it (issue #781).
    // A request to look at the repository the agent was handed is answered by
    // reading that repository. It has to be resolved before the research
    // routers, which would otherwise claim it on the strength of its question
    // shape alone and look the answer up on the open web (issue #1066). The
    // subject rule inside `workspace_inspection_search_for_task` is what keeps a
    // genuinely external question out of this route.
    if !tool_result::has_latest_turn_result(messages)
        && let Some(search) = workspace_inspection::workspace_inspection_search_for_task(task)
        && let Some(tool) = tool_for(tool_names, Capability::Grep)
    {
        let mut arguments = json!({
            "query": search.query,
            "pattern": search.pattern,
        });
        if let Some(include) = search.include {
            arguments["include"] = include.into();
        }
        return Some(plan_one(tool, arguments.to_string()));
    }
    // A question about how a task decomposes is answered by decomposing it. It
    // has to be resolved before the research routers for the same reason the
    // workspace inspection above does: the question shape alone would otherwise
    // send a task the web has never heard of to a web search (issue #1066).
    //
    // The route reads `messages` for the same reason its neighbour above does,
    // and it makes that judgement itself: a turn on which a tool has already run
    // is not one an answer composed from the request alone may claim.
    if let Some(plan) = task_structure::plan_task_structure_step(messages, task, result) {
        return Some(plan);
    }
    // The decision table of `data/seed/capability-routing.lino` (issue #1138 B10,
    // plan 10 leaves 9-11). Capability is a function of the object in the
    // request, the act asked for, and where the effect lands, and a triple with
    // no row declines here rather than guessing -- so the research routers below
    // answer only what the table did not claim.
    //
    // It sits *here*, and not ahead of the semantic routes above, because that
    // is the position the #745 and #758 misroutes were actually made from:
    // every one of them was a request the routes above declined and a research
    // router then claimed on the strength of its sentence shape alone. Ahead of
    // those routes the table preempts the ones that read the conversation and
    // the workspace, which was measured at 239 failing tests -- the recipe
    // driver, the ladder capability suite and the agentic surfaces among them.
    // Since leaf 11 removed the 280 memorized cues there is no second capability
    // decision behind this one; what the table declines is visible as an honest
    // gap in the event log rather than as a phrase match that never fired.
    // One more thing the table may not claim: a turn that is *mid-recipe*.
    // Once a search has produced usable output the conversation is inside the
    // multi-step research route -- search, then fetch each result, then compose
    // -- and the request text on that turn is still the original request. The
    // table reads that text, sees the object it named ("… на amazon.in" names a
    // registrable host), and answers the whole conversation with one step,
    // ending the recipe after its first (issue #781). Routing by object, act and
    // locus decides *which capability a request needs*; it does not decide that
    // a recipe already under way is finished. The `has_successful_search_result`
    // boundary is the one the research routes below already draw for themselves.
    if !web_research::has_successful_search_result(messages)
        && let Some(plan) = capability_router::plan_routed_capability_step(
            task,
            messages,
            tool_names,
            capability_router::RoutingStage::NamedOrLocal,
        )
    {
        return Some(plan);
    }
    // An addition that quotes no text earns a question naming what is missing
    // (PR #1188 G69).
    // An instruction that edits a named file is never a web question -- when
    // no edit route above could compose it, the honest answer is that nothing
    // was planned, not a search for the sentence (issues #1115, #1133).
    if super::positional_edit::unquoted_addition_path(task).is_some()
        || super::positional_edit::names_local_edit(task)
    {
        return steps::unquoted_addition_question(task);
    }
    if let Some(query) = web_research::web_research_query_for(messages)
        && let Some(plan) =
            web_research::plan_web_research_step(messages, tool_names, &query, false)
    {
        return Some(plan);
    }
    if let Some(plan) = intent_router::plan_web_search_step(task, messages, tool_names) {
        return Some(plan);
    }
    // A generic localized "find" cue can describe either an open-web lookup or
    // a workspace grep. The research routers above get first refusal whenever
    // the client exposes their tools; explicit local/repository searches were
    // already claimed by the capability router. This fallback therefore keeps
    // grep available to grep-only clients without letting an alphabetically
    // earlier local tool steal a web-research request.
    if !tool_result::has_latest_turn_result(messages)
        && let Some(query) = shell_command::code_search_query_for_task(task)
        && let Some(tool) = tool_for(tool_names, Capability::Grep)
    {
        return Some(plan_one(
            tool,
            json!({ "query": query, "pattern": query }).to_string(),
        ));
    }
    if web_research::has_successful_search_result(messages)
        && let Some(query) = web_research::mid_research_web_query_for(messages)
        && let Some(plan) =
            web_research::plan_web_research_step(messages, tool_names, &query, false)
    {
        return Some(plan);
    }
    if let Some(answer) = tool_result::latest_turn_answer(messages, tool_names, task) {
        return Some(AgenticPlan::Final(answer));
    }
    // A request that specifies what a document has to *cover* is answered by
    // composing that document. It sits here, after every route that could answer
    // one of the named parts outright, so a note is only composed once nothing
    // else claims the request -- and before the literal-write fallback, which
    // would otherwise write the specification instead of the document
    // (issue #1066).
    if let Some(plan) = note_composition::plan_note_composition_step(task, messages, result) {
        return Some(plan);
    }
    if let Some(plan) = compose_general_change_plan(task)
        .map(|plan| plan_general_change_step(messages, tool_names, &plan, result))
    {
        return Some(plan);
    }
    if let Some(query) = web_research::unresolved_web_research_query_for(messages)
        && let Some(plan) =
            web_research::plan_web_research_step(messages, tool_names, &query, false)
    {
        return Some(plan);
    }
    // The open-web half of the decision table, last: the research routers above
    // own a bare term the web has to answer, and the table speaks only for the
    // requests they declined -- which is where #745's "trawl the web for rust
    // ownership" ended in nothing at all.
    if let Some(plan) = capability_router::plan_routed_capability_step(
        task,
        messages,
        tool_names,
        capability_router::RoutingStage::OpenWeb,
    ) {
        return Some(plan);
    }
    None
}

/// The `shell_command` route arm of the cascade, or `None` to fall through.
///
/// A destructive shell intent read against a request about text inside a file
/// is declined with the seeded sentence, never composed (PR #1188); that
/// guard is this arm's own head, not a separately named route. Otherwise the
/// composed command runs through the file fallback, the verified mutating
/// recipe, or one shell step.
fn plan_shell_command_arm(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    if let Some(decline) = shell_command::destructive_edit_decline(task) {
        return Some(decline);
    }
    let command = shell_command::shell_command_for_task(task)?;
    if let Some(plan) = shell_file_fallback::plan_step(task, messages, tool_names, &command) {
        return Some(plan);
    }
    // A command that changes the workspace answers by what the workspace
    // holds afterwards, so it is carried out as the verified recipe its seed
    // intent declares rather than issued once (issues #824 and #944).
    if let Some(plan) = mutating_action::plan_step(&command, messages, tool_names, task, result) {
        return Some(plan);
    }
    Some(plan_shell_step(messages, tool_names, &command, result))
}

/// Run a shell command through the client-owned tool loop, then present its result.
fn plan_shell_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    command: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let progress = Progress::scan(messages);
    if progress.done(Capability::Run) {
        let raw = progress.run_outputs.last().map_or("", String::as_str);
        let disposition = if tool_result::step_outcome(raw) == tool_result::StepOutcome::Failed {
            FinalDisposition::Failure
        } else {
            FinalDisposition::Finding
        };
        return record(
            AgenticPlan::Final(tool_result::render(
                command,
                raw,
                crate::protocol::latest_user_request(messages)
                    .as_deref()
                    .unwrap_or_default(),
            )),
            disposition,
            "shell_result_observed",
            result,
        );
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
