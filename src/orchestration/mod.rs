//! Permission-gated orchestration of external agentic CLIs.
//!
//! The controller consumes the same seed registry as `formal-ai with`, binds
//! every grant to one workspace, records every process and filesystem effect,
//! and never retries an external process implicitly.

mod dispatch;
mod permission;
mod replay;
mod runner;
mod workspace;

pub use dispatch::{
    dispatch_agents, ComparisonEntry, ComparisonLedger, DispatchConfig, DispatchError,
    DispatchMode, DispatchReport,
};
pub use permission::AgentRunPermission;
pub use replay::{read_session, replay_session, write_session, ReplayError};
pub use runner::{
    run_agent, AgentCommand, AgentEvent, AgentRunConfig, AgentRunError, AgentSession, AgentStatus,
    AgentTarget, VerificationCommand, VerificationResult,
};
pub use workspace::{WorkspaceChange, WorkspaceChangeKind};
