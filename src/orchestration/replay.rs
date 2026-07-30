use super::{AgentEvent, AgentSession};
use serde_json::Error as JsonError;
use sha2::{Digest, Sha256};
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
    if canonical_bytes(&session)? != bytes {
        return Err(ReplayError::NonCanonical);
    }
    Ok(session)
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
        let expected = format!("{:x}", Sha256::digest(payload.as_bytes()));
        if event.sha256 != expected {
            return Err(ReplayError::EventDigest(sequence));
        }
        previous.clone_from(&event.sha256);
    }
    Ok(())
}
