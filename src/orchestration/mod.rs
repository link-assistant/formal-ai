//! Permission-gated orchestration of external agentic CLIs.
//!
//! The controller consumes the same seed registry as `formal-ai with`, binds
//! every grant to one workspace, records every process and filesystem effect,
//! and never retries an external process implicitly.

mod analysis;
mod dispatch;
mod permission;
mod replay;
mod runner;
mod workspace;

pub use analysis::{
    apply_verified_translation, extract_agent_result, observe_orchestration_session,
    synthesize_sessions, AgentSynthesisClaim, AgentSynthesisContradiction, AgentSynthesisError,
    AgentSynthesisReport, AgentSynthesisSource, VerifiedTranslation,
};
pub use dispatch::{
    dispatch_agents, ComparisonEntry, ComparisonLedger, DispatchConfig, DispatchError,
    DispatchMode, DispatchReport,
};
pub use permission::AgentRunPermission;
pub use replay::{read_session, replay_session, write_session, ReplayError};
pub use runner::{
    resume_agent, run_agent, session_sha256, AgentCommand, AgentContinuation, AgentEvent,
    AgentRunConfig, AgentRunError, AgentSession, AgentStatus, AgentTarget, CorrectionRequest,
    NativeAgentSession, VerificationCommand, VerificationResult,
};
pub use workspace::{WorkspaceChange, WorkspaceChangeKind};
