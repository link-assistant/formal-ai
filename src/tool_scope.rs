//! Which resource an advertised tool's effect actually lands on (issue #1075).
//!
//! A capability answers *what* a tool does; it does not answer *where*. Codex
//! advertised `codex_apps__github.create_file` beside its own `apply_patch`,
//! and a router that matched the name substring `create_file` chose the remote
//! connector to create a file in the client's checkout. The call left the
//! workspace untouched and answered 404 — with a fabricated empty repository,
//! against no repository at all.
//!
//! Scope is therefore read from what the client advertises, not from the shape
//! of the name: first the required arguments (a tool that cannot be called
//! without naming a repository, a channel or a live process does not act on the
//! workspace), then the namespace the tool is addressed through. The vocabulary
//! for both lives in `data/seed/tool-resource-scopes.lino`.

use std::sync::OnceLock;

use serde_json::{Map, Value};

use crate::seed::{self, ToolResourceScopeVocabulary};

/// The seed vocabulary, parsed once. Routing consults it for every advertised
/// tool of every request.
fn vocabulary() -> &'static ToolResourceScopeVocabulary {
    static VOCABULARY: OnceLock<ToolResourceScopeVocabulary> = OnceLock::new();
    VOCABULARY.get_or_init(seed::tool_resource_scope_vocabulary)
}

/// The resource an advertised tool acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolResourceScope {
    /// The directory the client is running in — the only scope that satisfies a
    /// request to create or edit a file in the task workspace.
    ClientWorkspace,
    /// A service reached over the network, addressed by repository, project,
    /// channel or page. Its effect is invisible to the workspace.
    RemoteService,
    /// The standard input of a process the client already started.
    ProcessInput,
}

impl ToolResourceScope {
    /// Whether an effect in this scope lands in the client's own workspace.
    #[must_use]
    pub const fn is_client_workspace(self) -> bool {
        matches!(self, Self::ClientWorkspace)
    }
}

/// Scope from the executable name alone, for the routing decisions that see
/// names rather than schemas.
///
/// Only whole address segments are compared. `codex_apps__github.create_file`
/// is addressed through `github` and is remote; `read_github_file` is one
/// segment and is not.
#[must_use]
pub fn scope_of_tool_name(name: &str) -> ToolResourceScope {
    scope_of_tool_name_with(name, vocabulary())
}

#[must_use]
pub fn scope_of_tool_name_with(
    name: &str,
    vocabulary: &ToolResourceScopeVocabulary,
) -> ToolResourceScope {
    let segments = address_segments(name);
    if segments
        .iter()
        .any(|segment| vocabulary.remote_namespaces.iter().any(|ns| ns == segment))
    {
        return ToolResourceScope::RemoteService;
    }
    let leaf = segments.last().map(String::as_str).unwrap_or_default();
    if vocabulary.process_names.iter().any(|entry| entry == leaf) {
        return ToolResourceScope::ProcessInput;
    }
    ToolResourceScope::ClientWorkspace
}

/// Scope from the client's own advertisement: the required arguments first,
/// then the name. A tool whose schema requires `repository_full_name` is remote
/// however it is named, which is what makes this a statement about the effect
/// rather than about the spelling.
#[must_use]
pub fn scope_of_tool_definition(definition: &Value, name: &str) -> ToolResourceScope {
    let vocabulary = vocabulary();
    let required = required_argument_names(definition);
    if required
        .iter()
        .any(|argument| vocabulary.remote_arguments.iter().any(|a| a == argument))
    {
        return ToolResourceScope::RemoteService;
    }
    if required
        .iter()
        .any(|argument| vocabulary.process_arguments.iter().any(|a| a == argument))
    {
        return ToolResourceScope::ProcessInput;
    }
    scope_of_tool_name_with(name, vocabulary)
}

/// The scope of every advertised name, resolved through its definition when the
/// definition is available.
#[must_use]
pub fn scope_of_advertised_tool(definitions: &[Value], name: &str) -> ToolResourceScope {
    crate::protocol_policy::find_tool_definition(definitions, name)
        .map_or_else(|| scope_of_tool_name(name), |d| scope_of_tool_definition(d, name))
}

/// Whether `name` is an argument that *names* a resource rather than describing
/// one. A description can be derived from the request; an identity cannot be
/// invented, because an invented one addresses a different resource — or, when
/// it is the empty string, none at all.
#[must_use]
pub fn is_identity_argument(name: &str) -> bool {
    let lower = name.to_lowercase();
    vocabulary()
        .identity_arguments
        .iter()
        .any(|entry| *entry == lower)
}

/// The required identity arguments of `definition` that neither the planner nor
/// the request can supply.
///
/// This is the precondition check the issue asks for: a tool whose identity
/// arguments cannot be grounded has an unknown target, so calling it is a guess
/// dressed as an action.
#[must_use]
pub fn ungrounded_identity_arguments(
    definition: &Value,
    provided: &Map<String, Value>,
    context: &str,
) -> Vec<String> {
    required_argument_names(definition)
        .into_iter()
        .filter(|name| is_identity_argument(name))
        .filter(|name| {
            !provided
                .get(name)
                .is_some_and(|value| !matches!(value, Value::String(text) if text.is_empty()))
                && grounded_identity_argument(name, context).is_none()
        })
        .collect()
}

/// Ground an identity argument in what the request actually says.
///
/// The only identities a request states outright are the ones inside the URLs
/// it carries: `https://host/owner/repo/issues/7` names the repository, its
/// owner and the issue. Anything else stays ungrounded and the call is not made.
#[must_use]
pub fn grounded_identity_argument(name: &str, context: &str) -> Option<Value> {
    let (owner, repo, number) = repository_reference(context)?;
    match name.to_lowercase().as_str() {
        "repository_full_name" | "repository" => Some(Value::String(format!("{owner}/{repo}"))),
        "repo" | "repo_name" => Some(Value::String(repo)),
        "owner" | "org" | "organization" => Some(Value::String(owner)),
        "issue_number" | "pull_number" => number.map(Value::from),
        _ => None,
    }
}

/// The `owner`, `repository` and trailing number of the first repository URL in
/// `context`.
fn repository_reference(context: &str) -> Option<(String, String, Option<u64>)> {
    context.split("://").skip(1).find_map(|rest| {
        let rest = rest
            .split_whitespace()
            .next()?
            .trim_end_matches(|c: char| matches!(c, '.' | ',' | ')' | '"' | '\'' | '>'));
        let mut segments = rest.split('/').filter(|segment| !segment.is_empty());
        let _host = segments.next()?;
        let owner = segments.next()?;
        let repo = segments.next()?.trim_end_matches(".git");
        if owner.is_empty() || repo.is_empty() {
            return None;
        }
        let number = segments
            .nth(1)
            .and_then(|segment| segment.parse::<u64>().ok());
        Some((owner.to_owned(), repo.to_owned(), number))
    })
}

/// The address segments a client routes a call through: MCP's `__`, the
/// Responses namespace dot, and the path forms some connectors use.
fn address_segments(name: &str) -> Vec<String> {
    name.split("__")
        .flat_map(|part| part.split(['.', '/', ':']))
        .filter(|segment| !segment.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Every argument the client says it will not accept a call without.
fn required_argument_names(definition: &Value) -> Vec<String> {
    let Some(object) = definition.as_object() else {
        return Vec::new();
    };
    object
        .get("parameters")
        .or_else(|| object.get("input_schema"))
        .or_else(|| {
            object
                .get("function")
                .and_then(|function| function.get("parameters"))
        })
        .or_else(|| {
            object
                .get("function")
                .and_then(|function| function.get("input_schema"))
        })
        .and_then(|schema| schema.get("required"))
        .and_then(Value::as_array)
        .map(|required| {
            required
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_lowercase)
                .collect()
        })
        .unwrap_or_default()
}
