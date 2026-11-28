use super::is_hosted_research_tool;
use crate::agentic_coding::planner::Capability;
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
pub(in crate::agentic_coding) fn tool_for<'a>(
    tool_names: &[&'a str],
    capability: Capability,
) -> Option<&'a str> {
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
pub(in crate::agentic_coding) fn shell_command_tool<'a>(tool_names: &[&'a str]) -> Option<&'a str> {
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
    let hosted = is_hosted_research_tool(name);
    let namespaced = name.to_ascii_lowercase().starts_with("mcp__");
    let client_scoped = crate::tool_scope::scope_of_tool_name(name).is_client_workspace();
    match (hosted, namespaced, client_scoped) {
        (false, true, true) => 0,
        (false, true, false) => 1,
        (false, false, _) => 2,
        (true, _, _) => 3,
    }
}

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

fn tool_matches_capability(
    name: &str,
    capability: Capability,
    registry: &[seed::AgenticToolCapability],
) -> bool {
    registry
        .iter()
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
pub(in crate::agentic_coding) fn classify_tool(name: &str) -> Option<Capability> {
    let registry = seed::agentic_tool_capabilities();
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
        if tool_matches_capability(name, capability, &registry) {
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
pub(in crate::agentic_coding) fn workspace_creation_tool<'a>(
    tool_names: &[&'a str],
) -> Option<&'a str> {
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
pub(in crate::agentic_coding) fn is_workspace_creation_tool(name: &str) -> bool {
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
