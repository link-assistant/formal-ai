//! Permission-gated orchestration of external agentic CLIs.
//!
//! The controller consumes the same seed registry as `formal-ai with`, binds
//! every grant to one workspace, records every process and filesystem effect,
//! and never retries an external process implicitly.

mod analysis;
mod attribution;
mod dispatch;
mod incremental;
mod permission;
mod replay;
mod runner;
pub(crate) mod workspace;

pub use analysis::{
    AgentSynthesisClaim, AgentSynthesisContradiction, AgentSynthesisError, AgentSynthesisReport,
    AgentSynthesisSource, VerifiedTranslation, apply_verified_translation, extract_agent_result,
    observe_orchestration_session, synthesize_sessions,
};
pub use dispatch::{
    ComparisonEntry, ComparisonLedger, DispatchConfig, DispatchError, DispatchMode, DispatchReport,
    dispatch_agents,
};
pub use incremental::{IncrementalProposal, IncrementalSplit, IncrementalStep, IncrementalTrace};
pub use permission::AgentRunPermission;
pub use replay::{ReplayError, read_session, replay_session, write_session};
pub use runner::{
    AgentCommand, AgentContinuation, AgentEvent, AgentRunConfig, AgentRunError, AgentSession,
    AgentStatus, AgentTarget, CorrectionRequest, NativeAgentSession, VerificationCommand,
    VerificationResult, resume_agent, run_agent, session_sha256,
};
pub use workspace::{WorkspaceChange, WorkspaceChangeKind};
