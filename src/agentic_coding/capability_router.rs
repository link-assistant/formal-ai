//! Capability-first routing for the shared agentic CLI tool set (issue #758).

use serde_json::json;

use super::general_planner::compose_edit_request;
use super::planner::{AgenticPlan, Capability, plan_one};
use crate::capability_routing::RoutingOutcome;
use crate::protocol::ChatMessage;
use crate::seed;

/// The advertised tool name that provides `capability`. Client-executed MCP
/// research tools take precedence over protocol-native hosted search/fetch
/// tools, because an MCP result returns through the CLI. Other capabilities use
/// canonical aliases first: a generic namespaced `_execute` helper must not
/// outrank Codex's purpose-built `exec_command`. Compatibility classification
/// remains the final fallback for namespaced tools used by other harnesses.
///
/// Every candidate is filtered by *where* its effect lands before any of that
/// runs (issue #1075). A capability that acts on the task workspace is only
/// satisfied by a tool that acts on the client's workspace: Codex advertises
/// `codex_apps__github.create_file` beside `apply_patch`, and choosing the
/// remote connector to create a workspace file produced a 404 against an empty
/// repository while the checkout stayed untouched.
pub(super) fn tool_for<'a>(tool_names: &[&'a str], capability: Capability) -> Option<&'a str> {
    let in_scope: Vec<&'a str> = tool_names
        .iter()
        .copied()
        .filter(|name| acts_in_capability_scope(name, capability))
        .collect();
    let tool_names: &[&'a str] = &in_scope;
    if matches!(capability, Capability::Search | Capability::Fetch)
        && let Some(name) = tool_names
            .iter()
            .copied()
            .filter(|name| classify_tool(name) == Some(capability))
            .min_by_key(|name| research_tool_rank(name))
    {
        return Some(name);
    }
    let registry = seed::agentic_tool_capabilities();
    let entry = registry
        .iter()
        .find(|entry| entry.id == capability.registry_id())?;
    entry
        .aliases
        .iter()
        .find_map(|alias| {
            tool_names
                .iter()
                .copied()
                .find(|name| alias.eq_ignore_ascii_case(name))
        })
        .or_else(|| {
            tool_names
                .iter()
                .copied()
                .find(|name| classify_tool(name) == Some(capability))
        })
}

/// A client-workspace execution tool whose schema accepts a shell command.
///
/// The general `Run` capability also includes interactive executors such as
/// computer use and code interpreters.  Those tools can perform work, but they
/// do not promise the `{ "command": ... }` contract used by `gh`, so the
/// narrower contract is declared beside the aliases in the seed registry.
pub(super) fn shell_command_tool<'a>(tool_names: &[&'a str]) -> Option<&'a str> {
    let registry = seed::agentic_tool_capabilities();
    let shell = registry
        .iter()
        .find(|entry| entry.id == Capability::Run.registry_id())?;
    shell.command_aliases.iter().find_map(|alias| {
        tool_names.iter().copied().find(|name| {
            acts_in_capability_scope(name, Capability::Run)
                && tool_leaf_name(name).eq_ignore_ascii_case(alias)
        })
    })
}

/// The provider-independent alias portion of a namespaced tool name.
fn tool_leaf_name(name: &str) -> &str {
    let name = name.rsplit("__").next().unwrap_or(name);
    name.rsplit(['.', '/', ':']).next().unwrap_or(name)
}

/// Whether a tool's effect lands where `capability` needs it to.
///
/// Research capabilities are about reaching *out*, so any scope answers them.
/// Everything else in the recipe -- reading, writing, editing, listing, running
/// -- is about the checkout the task is being done in, and only a tool scoped to
/// the client's workspace can do it there.
fn acts_in_capability_scope(name: &str, capability: Capability) -> bool {
    if matches!(
        capability,
        Capability::Search | Capability::Fetch | Capability::Todo | Capability::AskUser
    ) {
        return true;
    }
    crate::tool_scope::scope_of_tool_name(name).is_client_workspace()
}

/// Order among the tools that can answer a research capability.
///
/// A namespaced MCP research tool comes first. It is present only because the
/// client was deliberately configured to expose and permit it, so it is the
/// one alias the run is known to be able to execute. A client's own research
/// alias (`WebFetch`, `webfetch`) comes next, then the protocol-native hosted
/// tool (`web_search`, `web_fetch`), whose result the client never sees.
///
/// Issue #781: Claude Code advertises `WebSearch` alongside a wired-up
/// `mcp__issue781__websearch` but grants permission only for the latter, and
/// ranking its own alias first made every run stop at "Claude requested
/// permissions to use `WebSearch`, but you have not granted it yet".
///
/// Issue #1133: only a *research* MCP tool may outrank a client alias --
/// browser automation classifies as no capability at all, so
/// `mcp__playwright__browser_click` can
/// no longer win a fetch, which is what drove 547 identical empty-selector
/// calls until the context window filled.
fn research_tool_rank(name: &str) -> u8 {
    let hosted = HOSTED_RESEARCH_TOOLS
        .iter()
        .any(|hosted| hosted.eq_ignore_ascii_case(name));
    let namespaced = name.to_ascii_lowercase().starts_with("mcp__");
    let client_scoped = crate::tool_scope::scope_of_tool_name(name).is_client_workspace();
    match (hosted, namespaced, client_scoped) {
        (false, true, true) => 0,
        (false, true, false) => 1,
        (false, false, _) => 2,
        (true, _, _) => 3,
    }
}

/// Protocol-native research tools a provider executes on its own servers.
const HOSTED_RESEARCH_TOOLS: [&str; 5] = [
    "web_search",
    "web_fetch",
    "web_search_preview",
    "file_search",
    "computer_use_preview",
];

/// A browser-automation tool acts on a page the client is *driving*; it does
/// not retrieve a document. `browser_click`, `browser_type`, `browser_snapshot`
/// and their siblings all carry the substring `browse`, which the compatibility
/// fallback below read as a fetch (issue #1133). None of them is a capability
/// the planner has a use for, so they classify as nothing at all.
fn is_browser_interaction_tool(lower: &str) -> bool {
    BROWSER_INTERACTION_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

const BROWSER_INTERACTION_MARKERS: [&str; 11] = [
    "browser_",
    "click",
    "hover",
    "drag",
    "snapshot",
    "screenshot",
    "press_key",
    "fill_form",
    "select_option",
    "handle_dialog",
    "file_upload",
];

fn tool_matches_capability(name: &str, capability: Capability) -> bool {
    seed::agentic_tool_capabilities()
        .into_iter()
        .find(|entry| entry.id == capability.registry_id())
        .is_some_and(|entry| {
            entry
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(name))
        })
}

/// Classify a tool name through the shared alias registry, with legacy
/// substring matching as a compatibility fallback.
pub(super) fn classify_tool(name: &str) -> Option<Capability> {
    for capability in [
        Capability::Search,
        Capability::Fetch,
        Capability::Read,
        Capability::Write,
        Capability::Edit,
        Capability::Run,
        Capability::Grep,
        Capability::Glob,
        Capability::ListDir,
        Capability::Todo,
        Capability::Subagent,
        Capability::ReadMany,
        Capability::MultiEdit,
        Capability::AskUser,
    ] {
        if tool_matches_capability(name, capability) {
            return Some(capability);
        }
    }
    let lower = name.to_ascii_lowercase();
    if lower.contains("todo") || is_browser_interaction_tool(&lower) {
        return None;
    }
    if matches!(lower.as_str(), "computer_use" | "code_interpreter") {
        Some(Capability::Run)
    } else if lower.contains("search") {
        (lower.contains("web") && lower != "tool_search").then_some(Capability::Search)
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
    } else if lower == "write"
        || lower.ends_with("__write")
        || lower.contains("write_file")
        || lower.contains("file_write")
        || lower.contains("create_file")
    {
        Some(Capability::Write)
    } else if lower.contains("edit") || lower.contains("patch") || lower.contains("replace") {
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

/// The advertised tool that creates a workspace file: the write capability, or
/// a patch tool whose add-file form does the same (Codex's `apply_patch`).
///
/// Routes that gate on "can this client create a file" ask this rather than
/// [`tool_for`] with [`Capability::Write`]: gating on the write alias alone
/// skipped the repository work-item route for Codex entirely, and the issue it
/// should have planned from was narrated as a tool result instead (issue #1133).
pub(super) fn workspace_creation_tool<'a>(tool_names: &[&'a str]) -> Option<&'a str> {
    tool_for(tool_names, Capability::Write).or_else(|| {
        tool_names
            .iter()
            .copied()
            .find(|name| is_workspace_creation_tool(name))
    })
}

/// Whether a tool can create the source file that starts an execution recipe.
///
/// Most clients expose a dedicated write capability. Codex instead exposes its
/// patch grammar as an edit capability, but an add-file patch satisfies the
/// same recipe step. Process-input tools such as `write_stdin` satisfy neither.
pub(super) fn is_workspace_creation_tool(name: &str) -> bool {
    // A remote connector's `create_file` creates a file in a repository on a
    // server, not in the workspace the recipe is building (issue #1075).
    if !crate::tool_scope::scope_of_tool_name(name).is_client_workspace() {
        return false;
    }
    if classify_tool(name) == Some(Capability::Write) {
        return true;
    }
    let leaf = name.rsplit("__").next().unwrap_or(name);
    classify_tool(name) == Some(Capability::Edit) && leaf.to_ascii_lowercase().contains("patch")
}

/// Where in the planner a routed request is answered.
///
/// The decision table speaks about every request, but not at the same moment
/// about all of them. A request whose effect lands in the *workspace*, or whose
/// object is a URL, a path, a container or a literal, names something the
/// research routers cannot know better -- and letting them claim it on the
/// strength of the sentence's shape is precisely the #745 and #758 defect. Such
/// a request is answered before them. Everything else -- a bare term the open
/// web has to answer -- is the research routers' own subject, and the table
/// answers it only once they have declined, so a multi-turn research recipe is
/// never cut short by a single search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RoutingStage {
    /// Before the research routers: the workspace and the named objects.
    NamedOrLocal,
    /// After them: a bare term whose answer is on the open web.
    OpenWeb,
}

/// Whether the request names a filesystem container and asks nothing of it.
///
/// `acts` appends `retrieve` to every request, because a request that evidences
/// no narrower act is a retrieval -- which is the right default for an object
/// the request *named*, and the wrong one for a container it merely mentioned.
/// "Execute everything in the workspace" names a workspace and asks for no act
/// the seed knows, and answering it with a bare `ls` is passing prose through
/// as a command (issue #907). A retrieval the request *asked for* in the
/// seed's own words -- "busca X en mi escritorio", "… में खोजिए" -- is not
/// that case, however little else the sentence names: the #840 ladder's hi/es
/// folder nodes must reach the same `list_dir` lowering the English node does
/// (plan 10 leaf 21).
fn names_a_container_without_an_act(task: &str) -> bool {
    use crate::capability_routing::{Act, ObjectType, acts, evidences_retrieve_act, object_type};
    object_type(task).first().copied() == Some(ObjectType::PathScope)
        && acts(task) == [Act::Retrieve]
        && !evidences_retrieve_act(task)
}

/// Which stage a request belongs to, read from the derivations rather than from
/// the capability the table happens to name.
fn stage_of(task: &str) -> RoutingStage {
    use crate::capability_routing::{Locus, ObjectType, locus, object_type};
    let highest = object_type(task).first().copied().unwrap_or_default();
    let named = matches!(
        highest,
        ObjectType::Url
            | ObjectType::Path
            | ObjectType::PathScope
            | ObjectType::QuotedContent
            | ObjectType::Pattern
            | ObjectType::PathSet
            | ObjectType::TaskList
            | ObjectType::Delegation
    );
    if named || locus(task) == Locus::Workspace {
        RoutingStage::NamedOrLocal
    } else {
        RoutingStage::OpenWeb
    }
}

/// The capability slugs `data/seed/capability-routing.lino` names, paired with
/// the client capability that answers each. A slug with no pairing here is one
/// the decision table routes to but no advertised tool provides -- the calendar,
/// the response-language demonstration, the measurement lookup -- and the table
/// reports an honest gap for it rather than a tool call.
///
/// Plan 10 leaf 11: the seven request-side cue families joined this table, so
/// every capability the seed registry declares is decided by a
/// `(object, act, locus)` row and none by a memorized phrase.
const ROUTED_CAPABILITIES: [(&str, Capability); 12] = [
    ("web_fetch", Capability::Fetch),
    ("web_search", Capability::Search),
    ("read_file", Capability::Read),
    ("write_file", Capability::Write),
    ("list_dir", Capability::ListDir),
    ("grep", Capability::Grep),
    ("shell", Capability::Run),
    ("glob", Capability::Glob),
    ("read_many", Capability::ReadMany),
    ("multi_edit", Capability::MultiEdit),
    ("todo", Capability::Todo),
    ("subagent", Capability::Subagent),
];

/// Route one request through the decision table of
/// `data/seed/capability-routing.lino` (issue #1138 B10, plan 10 leaves 9-11).
///
/// Since leaf 11 this is the only request-side capability decision there is:
/// the 280 memorized cue phrases and the `task_matches` phrase scan are gone,
/// and every capability the seed registry declares is chosen by an
/// `(object, act, locus)` row. A triple with no row returns [`None`] and the
/// older routes see the request, which is how a class with no row yet stays
/// visible instead of silently guessed.
///
/// Only the capabilities a tool can actually satisfy are advertised to
/// [`crate::capability_routing::route`], so a row naming a capability this
/// client has no tool for produces an honest gap there rather than a tool call
/// here.
pub(super) fn plan_routed_capability_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    stage: RoutingStage,
) -> Option<AgenticPlan> {
    plan_routed_capability_step_in(task, messages, tool_names, stage, &[])
}

/// The capability slugs the retired cue arm used to decide, by slug.
///
/// Plan 10 leaf 11: the phrases are gone, but these seven rows have to be
/// decided at the arm's old position -- ahead of the shell cascade and the
/// file-read fallback -- because each of their rows names a specialized tool
/// with a `fallback shell`, and a request the table sends to an advertised
/// `grep_search`, `list_directory`, `glob`, `read_many_files`, `multi_edit`,
/// `todo_write` or `task` must not be answered by the shell lowering the same
/// row declares (issue #758's specialized-first policy, now carried by the
/// rows' `fallback` fields and this position instead of a phrase scan).
const NAMED_CAPABILITY_SLUGS: [&str; 7] = [
    "grep",
    "glob",
    "list_dir",
    "read_many",
    "multi_edit",
    "todo",
    "subagent",
];

/// The seven named capabilities, decided by the table at the position the cue
/// arm held: after the routes that read the conversation, before the shell
/// cascade that would otherwise lower the same request to `bash`.
pub(super) fn plan_named_capability_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    plan_routed_capability_step_in(
        task,
        messages,
        tool_names,
        RoutingStage::NamedOrLocal,
        &NAMED_CAPABILITY_SLUGS,
    )
}

fn plan_routed_capability_step_in(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    stage: RoutingStage,
    only: &[&str],
) -> Option<AgenticPlan> {
    // The capability belongs to the stated request, not to a later harness
    // block that only places the worker. Every agentic research route uses the
    // same block boundary; applying the table to the whole envelope let "work
    // only in this checkout" turn a live exchange-rate query into a local
    // directory request (issue #1066).
    let routed_task = super::stated_request::request_blocks(task)
        .into_iter()
        .next()
        .unwrap_or(task);
    if !crate::capability_routing::table_routing_enabled() || stage_of(routed_task) != stage {
        return None;
    }
    // At the open-web position every route that reads the conversation and the
    // workspace has declined, and a request that never named the open web is
    // one the symbolic engine should still answer. "What is Links Notation?"
    // is not a search just because nothing above claimed it (issue #989): the
    // seed's concept lookup resolves it, so the engine speaks; a computation
    // ("What is 480 divided by 15?") is held the same way. A definition the
    // seed's own lookup leaves unresolved is the honest unknown that only a
    // trusted external source answers, whatever language asks it -- "Что
    // такое фуфломицин?" searched in the ladder's committed baseline -- and a
    // current-fact request the table itself routed to `(bare_term, retrieve,
    // web)` ("Verify the current exchange rate between the euro and the yen",
    // the news class) is that same admission in the table's own words (plan
    // 10 leaf 21, issue #840). A measurement the table honestly gaps is still
    // refused below by its own outcome, never downgraded to a search.
    let engine_answerable_concept = crate::concepts::extract_concept_query(routed_task).is_some()
        && !super::web_research::concept_lookup_leaves_unknown(routed_task);
    // The concept half above answers definitions; the fact store answers
    // questions its records name -- "What is the capital of France?" is the
    // solver's own `fact_lookup` row, not a search (issue #1138: the gate
    // read only the concept lookup, so a fact the engine owned leaked to the
    // open web as soon as no concept matched).
    let engine_answerable_fact = crate::solver_handlers::fact_store_resolves(routed_task);
    if stage == RoutingStage::OpenWeb
        && !crate::capability_routing::names_open_web(routed_task)
        && (engine_answerable_concept || engine_answerable_fact)
    {
        return None;
    }
    let advertised: Vec<&str> = ROUTED_CAPABILITIES
        .iter()
        .filter(|(_, capability)| tool_for(tool_names, *capability).is_some())
        .map(|(slug, _)| *slug)
        .collect();
    let (slug, lowered_from) = match crate::capability_routing::route(routed_task, &advertised) {
        RoutingOutcome::Routed { capability } => (capability, None),
        RoutingOutcome::Lowered {
            preferred,
            capability,
        } => (capability, Some(preferred)),
        RoutingOutcome::HonestGap { .. } | RoutingOutcome::Ask { .. } => return None,
    };
    // A positional restriction reads the capability the row *decided on* -- the
    // preferred one when the row lowered, the row's own otherwise. A shell
    // outcome reached without a named preferred capability (a shell row) is
    // never a named-capability decision and stays out of the restricted call.
    let decided = lowered_from.as_deref().unwrap_or(&slug);
    if !only.is_empty() && !only.contains(&decided) {
        return None;
    }
    let capability = ROUTED_CAPABILITIES
        .iter()
        .find(|(name, _)| *name == slug)
        .map(|(_, capability)| *capability)?;
    if names_a_container_without_an_act(routed_task) {
        return None;
    }
    // A batch read is a request to *read*. A request the shell-intent
    // vocabulary recognizes as mutating — a requesting sentence names the
    // cue of an intent that declares an effect, whether or not the operands
    // resolve safely enough to build the command — is not a batch read, and
    // this arm must not answer it, not even from the latest turn's result:
    // a mutating request is carried out as the verified recipe its seed
    // intent declares (issues #824 and #944), and the table may not end a
    // recipe already under way (issue #781). The boundary is the seed's own
    // cue and effect declarations, so every mutating intent in every
    // language the vocabulary covers defers here without being named in
    // Rust, while a genuine batch read ("read all of these files: a.txt and
    // b.md") names no mutating cue and keeps its route (issue #1021:
    // "copy a.txt to b.txt" planned `cat`, and a traversal the safety rule
    // refused answered as a one-file cat of the operand that survived).
    if (capability == Capability::ReadMany
        || (capability == Capability::Run && lowered_from.as_deref() == Some("read_many")))
        && super::shell_command::names_mutating_shell_intent(routed_task)
    {
        return None;
    }
    if super::tool_result::has_latest_turn_result(messages) {
        return super::tool_result::latest_turn_answer(messages, tool_names, task)
            .map(AgenticPlan::Final);
    }
    // A sentence that *governs* commands is not one that requests one: "Never
    // run chmod on files outside the workspace" names a workspace and a file and
    // asks for neither (issues #907, #916). The shell route already draws this
    // boundary and it is drawn here for the same capabilities, not a second way.
    if capability == Capability::Run
        && super::shell_command_policy::governs_commands_rather_than_requesting_one(routed_task)
    {
        return None;
    }
    // A lowering to the generic listing yields to a more specific shell intent:
    // "What is current directory?" asks *where the session is*, not what the
    // place holds, and the seed's shell-intent vocabulary answers that with
    // `pwd` (issue #989) — a question the container row's `ls` cannot. The
    // #907 rule above keeps containers the request merely mentioned out of
    // the table; this one keeps the session's own place out of it too, and
    // lets every genuine listing ("show me the files in the folder") through,
    // because only an intent the vocabulary resolves *differently* defers.
    if capability == Capability::Run
        && lowered_from.as_deref() == Some("list_dir")
        // The comparison target is the very command the row's lowering runs,
        // so the defer tracks the seed's listing command instead of naming it
        // here (the literal-predicate ratchet, issue #1085 D1).
        && let Some(listing) = shell_fallback(Capability::ListDir, routed_task)
        && super::shell_command::semantic_shell_command_for_task(routed_task)
            .is_some_and(|command| !command.starts_with(&listing))
    {
        return None;
    }
    // A file the request asks to *create* cannot be read: the target does not
    // exist yet, so reading it is always the wrong tool (issue #681). This is
    // the rule `file_read_task_for` already applies, reused rather than
    // re-derived.
    // The rule is about a file the request asks to *create*, so it needs the
    // file as well as the verb. A request that states a write verb and names no
    // destination is asking for something to be shown, not made: Spanish
    // "escribe el contenido de sample.txt" and Chinese "输出内容 sample.txt" both
    // head their read with a verb the seed also knows as a write action, and
    // refusing the read on the verb alone left them with no route at all.
    if capability == Capability::Read
        && super::write_request::states_write_action(routed_task)
        && super::write_request::stated_write_target(routed_task).is_some()
    {
        return None;
    }
    let tool = tool_for(tool_names, capability)?;
    let arguments = routed_arguments(capability, lowered_from.as_deref(), routed_task)?;
    Some(plan_one(tool, arguments))
}

/// The command a `grep` lowered to the shell runs when the shell vocabulary
/// does not recognise the request as a search.
///
/// The table has already decided this is code navigation -- the object is a
/// bare term and the locus is the workspace -- so the cue that
/// [`super::shell_command::shell_command_for_task`] looks for has nothing left
/// to decide, and a code-shaped token is enough. Without this, a Spanish code
/// search fell through to a web search, which is the #758 defect in the one
/// language the cue vocabulary had not been given (issue #1138 B10).
fn lowered_search_command(preferred: Capability, task: &str) -> Option<String> {
    if preferred != Capability::Grep {
        return None;
    }
    let query = super::shell_command::code_shaped_query(task)
        .or_else(|| super::shell_command::code_search_query_for_task(task))?;
    Some(format!("rg -n {}", shell_quote(&query)))
}

/// The call the routed capability is made with.
///
/// A lowering builds the command for the capability it lowered *from*, so
/// `list_dir` lowered to the shell still runs `ls` and code navigation lowered
/// to the shell still greps -- the shape #758 asked for, now stated by the row's
/// `fallback` field rather than by a Rust cascade.
fn routed_arguments(
    capability: Capability,
    lowered_from: Option<&str>,
    task: &str,
) -> Option<String> {
    if capability == Capability::Run {
        let preferred = lowered_from
            .and_then(|slug| {
                ROUTED_CAPABILITIES
                    .iter()
                    .find(|(name, _)| *name == slug)
                    .map(|(_, capability)| *capability)
            })
            .unwrap_or(Capability::Run);
        let command = shell_fallback(preferred, task)
            .or_else(|| super::shell_command::shell_command_for_task(task))
            .or_else(|| lowered_search_command(preferred, task))?;
        return Some(json!({ "command": command }).to_string());
    }
    match capability {
        Capability::Fetch => {
            let url = crate::capability_routing::first_url(task)?;
            Some(super::planner::fetch_arguments(&url))
        }
        Capability::Search => {
            let query = super::stated_request::request_blocks(task)
                .into_iter()
                .find_map(super::web_research::open_web_query_for_block)
                .unwrap_or_else(|| {
                    crate::solver_handlers::web_search_intent::clean_search_query(task)
                });
            Some(json!({ "query": query }).to_string())
        }
        Capability::Read => {
            let path = crate::capability_routing::first_path(task)?;
            Some(json!({"path": path, "filePath": path, "file_path": path}).to_string())
        }
        Capability::Write => {
            let path = crate::capability_routing::first_path(task)?;
            let content = crate::capability_routing::explicit_content(task)?;
            Some(super::planner::write_arguments(&path, &content))
        }
        Capability::MultiEdit => {
            // The table routes a set of files transformed in place here; the
            // edit itself still has to be composable. A request whose edit
            // cannot be composed is declined rather than answered with
            // placeholder bytes (the guard `task_matches` used to apply before
            // the capability was chosen at all).
            let (path, old, new) = compose_edit_request(task)?;
            let paths = file_tokens(task);
            Some(
                json!({
                    "path": path,
                    "paths": paths,
                    "edits": [{
                        "old": old,
                        "new": new,
                        "old_string": old,
                        "new_string": new,
                    }],
                })
                .to_string(),
            )
        }
        Capability::Grep => {
            // The table's `(bare_term, retrieve, workspace)` row and the
            // workspace-inspection route answer one request class, and the
            // named-capability position now reaches it first. The inspection
            // subject rule is the stricter, seed-backed admission, so its
            // canonical literal/fact bindings name the query; without this a
            // bare-term retrieval grep fell back to the raw request sentence
            // (issue #1066).
            if let Some(search) =
                super::workspace_inspection::workspace_inspection_search_for_task(task)
            {
                let mut arguments = json!({
                    "query": search.query,
                    "pattern": search.pattern,
                });
                if let Some(include) = search.include {
                    arguments["include"] = include.into();
                }
                Some(arguments.to_string())
            } else {
                Some(arguments_for(capability, task))
            }
        }
        _ => Some(arguments_for(capability, task)),
    }
}

/// The call a capability that needs no per-request derivation is made with.
///
/// Plan 10 leaf 11: this was the cue capabilities' argument builder behind
/// `plan_shared_capability_step`; the routed step now derives the
/// request-shaped arguments itself (`routed_arguments`) and only these
/// schema-shaped defaults remain.
fn arguments_for(capability: Capability, task: &str) -> String {
    match capability {
        Capability::Grep => {
            let query = super::shell_command::code_search_query_for_task(task)
                .unwrap_or_else(|| task.to_owned());
            json!({"query": query, "pattern": query}).to_string()
        }
        Capability::Glob => {
            let pattern = wildcard_token(task).unwrap_or("*");
            json!({"pattern": pattern, "path": "."}).to_string()
        }
        Capability::ListDir => json!({"path": "."}).to_string(),
        Capability::Todo => json!({
            "todos": [{"content": task, "status": "pending"}],
            "plan": [{"step": task, "status": "pending"}],
        })
        .to_string(),
        Capability::Subagent => json!({
            "description": task,
            "prompt": task,
            "input": task,
            "subagent_type": "general",
        })
        .to_string(),
        Capability::AskUser => String::new(),
        Capability::ReadMany => {
            let paths = file_tokens(task);
            json!({"paths": paths, "file_paths": paths}).to_string()
        }
        _ => json!({"prompt": task}).to_string(),
    }
}

fn wildcard_token(task: &str) -> Option<&str> {
    task.split_whitespace()
        .map(|token| token.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '`' | '"' | '\'')))
        .find(|token| token.contains('*') || token.contains('?') || token.contains('['))
}

fn file_tokens(task: &str) -> Vec<&str> {
    task.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '`' | '"' | '\'' | '(' | ')'))
        })
        .filter(|token| {
            token.contains('.')
                && !token.starts_with('.')
                && !token.ends_with('.')
                && !token.contains("//")
        })
        .collect()
}

fn shell_fallback(capability: Capability, task: &str) -> Option<String> {
    match capability {
        Capability::Grep => super::shell_command::shell_command_for_task(task),
        Capability::Glob => {
            let pattern = wildcard_token(task).unwrap_or("*").replace('\'', "'\\''");
            let mut command = String::from("find");
            command.push_str(" .");
            command.push_str(" -path ");
            command.push('\'');
            command.push_str(&pattern);
            command.push('\'');
            Some(command)
        }
        Capability::ListDir => Some(String::from("ls")),
        Capability::ReadMany => {
            let paths = file_tokens(task);
            (!paths.is_empty()).then(|| {
                let paths = paths
                    .into_iter()
                    .map(shell_quote)
                    .collect::<Vec<_>>()
                    .join(" ");
                let mut command = String::from("cat");
                command.push(' ');
                command.push_str(&paths);
                command
            })
        }
        _ => None,
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
