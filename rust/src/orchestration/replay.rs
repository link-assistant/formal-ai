use super::{AgentEvent, AgentSession, AgentStatus};
use serde_json::Error as JsonError;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum ReplayError {
    Io(io::Error),
    Json(JsonError),
    NonCanonical,
    Schema,
    EventSequence(u64),
    EventChain(u64),
    EventDigest(u64),
    EventBinding(&'static str),
    BrokenAncestry,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "io:{error}"),
            Self::Json(error) => write!(formatter, "json:{error}"),
            Self::NonCanonical => formatter.write_str("non_canonical_session"),
            Self::Schema => formatter.write_str("unsupported_session_schema"),
            Self::EventSequence(sequence) => write!(formatter, "event_sequence:{sequence}"),
            Self::EventChain(sequence) => write!(formatter, "event_chain:{sequence}"),
            Self::EventDigest(sequence) => write!(formatter, "event_digest:{sequence}"),
            Self::EventBinding(field) => write!(formatter, "event_binding:{field}"),
            Self::BrokenAncestry => formatter.write_str("broken_ancestry"),
        }
    }
}

impl std::error::Error for ReplayError {}

pub fn write_session(path: &Path, session: &AgentSession) -> Result<(), ReplayError> {
    let rendered = canonical_bytes(session)?;
    fs::write(path, rendered).map_err(ReplayError::Io)
}

pub fn read_session(path: &Path) -> Result<AgentSession, ReplayError> {
    let bytes = fs::read(path).map_err(ReplayError::Io)?;
    replay_session(&bytes)
}

pub fn replay_session(bytes: &[u8]) -> Result<AgentSession, ReplayError> {
    let session: AgentSession = serde_json::from_slice(bytes).map_err(ReplayError::Json)?;
    if session.schema != "formal-ai-agent-session-v1" {
        return Err(ReplayError::Schema);
    }
    verify_events(&session.events)?;
    verify_bindings(&session)?;
    if canonical_bytes(&session)? != bytes {
        return Err(ReplayError::NonCanonical);
    }
    Ok(session)
}

/// Replay a correction session against the exact parent bytes it names.
///
/// Both sessions must replay on their own, and the child's continuation must
/// carry the SHA-256 of `parent_bytes`: a session resumed from another parent,
/// or replayed against an edited parent, is rejected as broken ancestry.
pub fn replay_continuation(
    parent_bytes: &[u8],
    child_bytes: &[u8],
) -> Result<AgentSession, ReplayError> {
    replay_session(parent_bytes)?;
    let child = replay_session(child_bytes)?;
    let parent_sha256 = crate::source_fetch::sha256_hex(parent_bytes);
    if child
        .continuation
        .as_ref()
        .is_some_and(|continuation| continuation.parent_session_sha256 == parent_sha256)
    {
        Ok(child)
    } else {
        Err(ReplayError::BrokenAncestry)
    }
}

fn canonical_bytes(session: &AgentSession) -> Result<Vec<u8>, ReplayError> {
    let mut rendered = serde_json::to_vec_pretty(session).map_err(ReplayError::Json)?;
    rendered.push(b'\n');
    Ok(rendered)
}

fn verify_events(events: &[AgentEvent]) -> Result<(), ReplayError> {
    let mut previous = "0".repeat(64);
    for (index, event) in events.iter().enumerate() {
        let sequence = index as u64;
        if event.sequence != sequence {
            return Err(ReplayError::EventSequence(sequence));
        }
        if event.previous_sha256 != previous {
            return Err(ReplayError::EventChain(sequence));
        }
        let payload = format!(
            "{}\0{}\0{}\0{}",
            event.sequence, event.kind, event.detail, event.previous_sha256
        );
        let expected = crate::source_fetch::sha256_hex(payload.as_bytes());
        if event.sha256 != expected {
            return Err(ReplayError::EventDigest(sequence));
        }
        previous.clone_from(&event.sha256);
    }
    Ok(())
}

/// The session fields the event chain stands for must agree with the chain.
///
/// The chain digests events only, so an edited `status` or `changes` list
/// would otherwise still replay. An agent run records exactly one terminal
/// process event, a composed-verifier run one composition outcome, every
/// workspace effect is an event in `changes` order, and a continuation is
/// present exactly when the run resumed a native session.
fn verify_bindings(session: &AgentSession) -> Result<(), ReplayError> {
    let composed = session
        .events
        .iter()
        .any(|event| event.kind == "composition_verification_started");
    let mut outcomes = session
        .events
        .iter()
        .filter_map(|event| terminal_status(&event.kind, composed));
    if outcomes.next().as_ref() != Some(&session.status) || outcomes.next().is_some() {
        return Err(ReplayError::EventBinding("status"));
    }
    let effects = session
        .events
        .iter()
        .filter(|event| event.kind == "workspace_effect")
        .map(|event| event.detail.as_str());
    if !effects.eq(session.changes.iter().map(|change| change.path.as_str())) {
        return Err(ReplayError::EventBinding("effects"));
    }
    let resumed = session
        .events
        .iter()
        .any(|event| event.kind == "native_session_resumed");
    if resumed != session.continuation.is_some() {
        return Err(ReplayError::EventBinding("continuation"));
    }
    Ok(())
}

fn terminal_status(kind: &str, composed: bool) -> Option<AgentStatus> {
    match (composed, kind) {
        (false, "process_succeeded") | (true, "composition_verification_passed") => {
            Some(AgentStatus::Succeeded)
        }
        (false, "process_failed") | (true, "composition_verification_failed") => {
            Some(AgentStatus::Failed)
        }
        (false, "process_timed_out") => Some(AgentStatus::TimedOut),
        _ => None,
    }
}
