//! The scope a learned standing requirement belongs to (issue #1559).
//!
//! Every recorded exchange carries the scope it was made in as evidence
//! entries `scope:user=<id>`, `scope:repository=<host/owner/repo>` and
//! `scope:session=<id>`:
//!
//! - `user` is the client-declared end user (OpenAI `user`, Anthropic
//!   `metadata.user_id`) — a requirement that user states follows that user
//!   into every later conversation;
//! - `session` is recorded for an agent session (a request that carries
//!   tools): the client's first message configures one task, so what is
//!   learned there stays in that session unless sharing is requested with
//!   `FORMAL_AI_SHARE_STANDING_REQUIREMENTS=1`;
//! - `repository` names the repository an agent session works on, so a trace
//!   shows which task a requirement came from.
//!
//! An exchange with neither a user nor a session scope belongs to the owner of
//! the memory store, and what it teaches applies everywhere in that store, as
//! before. An amendment applies to a request only when every scope pair it
//! carries equals the request's own.
//!
//! Twin of `js/server/standing-scope.mjs`.

use std::borrow::Cow;

use serde_json::Value;

use crate::dreaming::bounds::is_session_instruction;
use crate::dreaming::lexicon::lexicon;
use crate::memory::MemoryEvent;
use crate::protocol::ChatMessage;

/// The evidence prefix of a scope entry.
pub const SCOPE_PREFIX: &str = "scope:";
/// The evidence entry of an amendment generalized from several scopes and
/// verified by replay.
pub const GENERALIZED_SCOPE: &str = "scope:generalized";
/// The scope keys in their canonical order.
pub const SCOPE_KEYS: [&str; 3] = ["user", "repository", "session"];
const SHARE_VARIABLE: &str = "FORMAL_AI_SHARE_STANDING_REQUIREMENTS";
const URL_TRAILING: [char; 13] = [
    ')', ']', '}', '>', '"', '\'', '`', ',', ';', ':', '!', '?', '.',
];

/// One `(key, value)` scope pair.
pub type ScopePair = (String, String);

/// The scope of one request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestScope {
    pub user: Option<String>,
    pub repository: Option<String>,
    pub session: Option<String>,
    /// The request carries tools: an agent works on one task.
    pub agent_session: bool,
    /// Sharing across sessions was requested by the operator.
    pub shared: bool,
}

impl RequestScope {
    fn value(&self, key: &str) -> Option<&str> {
        match key {
            "user" => self.user.as_deref(),
            "repository" => self.repository.as_deref(),
            "session" => self.session.as_deref(),
            _ => None,
        }
    }
}

/// The client-declared end user of a raw request body.
#[must_use]
pub fn request_user(body: &str) -> Option<String> {
    let object = serde_json::from_str::<Value>(body).ok()?;
    let object = object.as_object()?;
    [
        object.get("user"),
        object
            .get("metadata")
            .and_then(|metadata| metadata.get("user_id")),
    ]
    .into_iter()
    .flatten()
    .filter_map(Value::as_str)
    .map(str::trim)
    .find(|user| !user.is_empty())
    .map(str::to_owned)
}

fn stripped_url(raw: &str) -> &str {
    raw.trim_end_matches(|character| URL_TRAILING.contains(&character))
}

/// `host/owner/repo` of the first URL in `text` that points at an issue or a
/// pull request.
#[must_use]
pub fn repository_of(text: &str) -> Option<String> {
    let kinds = &lexicon().task_reference_words;
    text.split_whitespace().find_map(|raw| {
        let at = raw.find("://").filter(|at| *at > 0)?;
        let segments = stripped_url(&raw[at + 3..])
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();
        if segments.len() < 4 || !segments[0].contains('.') {
            return None;
        }
        if !kinds.contains(&segments[3].to_lowercase()) {
            return None;
        }
        let repository = segments[2].to_lowercase();
        let repository = repository.strip_suffix(".git").unwrap_or(&repository);
        Some(format!(
            "{}/{}/{repository}",
            segments[0].to_lowercase(),
            segments[1].to_lowercase()
        ))
    })
}

/// The operator asked agent sessions to share what they learn.
#[must_use]
pub fn sharing_requested() -> bool {
    std::env::var(SHARE_VARIABLE)
        .is_ok_and(|value| matches!(value.trim().to_lowercase().as_str(), "1" | "true" | "yes"))
}

fn role_is(message: &ChatMessage, roles: &[&str]) -> bool {
    roles
        .iter()
        .any(|role| message.role.eq_ignore_ascii_case(role))
}

/// The scope of one request from its raw body, chat messages and whether it
/// advertises tools.
#[must_use]
pub fn request_scope(body: &str, messages: &[ChatMessage], has_tools: bool) -> RequestScope {
    let configuring = messages
        .iter()
        .filter(|message| role_is(message, &["system", "developer", "user"]))
        .map(|message| message.content.plain_text())
        .collect::<Vec<_>>()
        .join("\n");
    let session = messages
        .iter()
        .find(|message| role_is(message, &["user"]))
        .map(|first| crate::engine::stable_id("session", &first.content.plain_text()));
    RequestScope {
        user: request_user(body),
        repository: repository_of(&configuring),
        session,
        agent_session: has_tools,
        shared: sharing_requested(),
    }
}

/// The scope entries a recorded exchange carries. A user scope wins; an agent
/// session is scoped to its session (and repository) unless sharing is
/// requested.
#[must_use]
pub fn recorded_scope_evidence(scope: &RequestScope) -> Vec<String> {
    if let Some(user) = &scope.user {
        return vec![format!("{SCOPE_PREFIX}user={user}")];
    }
    let Some(session) = scope.session.as_deref() else {
        return Vec::new();
    };
    if !scope.agent_session || scope.shared {
        return Vec::new();
    }
    let mut entries = Vec::new();
    if let Some(repository) = &scope.repository {
        entries.push(format!("{SCOPE_PREFIX}repository={repository}"));
    }
    entries.push(format!("{SCOPE_PREFIX}session={session}"));
    entries
}

/// The scope pairs in `evidence`, in canonical key order.
#[must_use]
pub fn scope_pairs(evidence: &[String]) -> Vec<ScopePair> {
    let mut pairs: Vec<ScopePair> = Vec::new();
    for entry in evidence {
        let Some(body) = entry.strip_prefix(SCOPE_PREFIX) else {
            continue;
        };
        let Some((key, value)) = body.split_once('=') else {
            continue;
        };
        if key.is_empty()
            || !SCOPE_KEYS.contains(&key)
            || pairs.iter().any(|(existing, _)| existing == key)
        {
            continue;
        }
        pairs.push((key.to_owned(), value.to_owned()));
    }
    pairs.sort_by_key(|(key, _)| SCOPE_KEYS.iter().position(|known| known == key));
    pairs
}

/// A canonical text key of scope pairs (empty when global).
#[must_use]
pub fn scope_key(pairs: &[ScopePair]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every pair equals the request's own; no scope sees only global pairs.
#[must_use]
pub fn in_scope(pairs: &[ScopePair], scope: Option<&RequestScope>) -> bool {
    pairs
        .iter()
        .all(|(key, value)| scope.and_then(|scope| scope.value(key)) == Some(value.as_str()))
}

/// Memory as one request sees it (issue #1559 §6): events recorded in another
/// scope, and amendments learned there, are dropped. With no request scope (an
/// internal caller) every scoped event is dropped.
#[must_use]
pub fn events_in_scope<'a>(
    events: &'a [MemoryEvent],
    scope: Option<&RequestScope>,
) -> Cow<'a, [MemoryEvent]> {
    if events
        .iter()
        .all(|event| scope_pairs(&event.evidence).is_empty())
    {
        return Cow::Borrowed(events);
    }
    Cow::Owned(
        events
            .iter()
            .filter(|event| in_scope(&scope_pairs(&event.evidence), scope))
            .cloned()
            .collect(),
    )
}

/// The recorded prompt configures the session rather than stating a user
/// requirement: it opens like a system prompt, or it is the first user message
/// of an agent session that carries no system message (a client delivering its
/// system prompt as the first user turn, issue #1559 §1).
#[must_use]
pub fn is_instruction_prompt(prompt: &str, messages: &[ChatMessage], scope: &RequestScope) -> bool {
    if is_session_instruction(prompt) {
        return true;
    }
    if !scope.agent_session
        || messages
            .iter()
            .any(|message| role_is(message, &["system", "developer"]))
    {
        return false;
    }
    messages
        .iter()
        .find(|message| role_is(message, &["user"]))
        .is_some_and(|first| {
            let prompt = prompt.trim();
            first.content.plain_text().trim() == prompt
                || first.content.user_request_text().trim() == prompt
        })
}
