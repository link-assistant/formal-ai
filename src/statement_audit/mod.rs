//! Evidence-weighted audit of statements in a repository corpus.
//!
//! The reasoning core is deliberately pure: callers supply a corpus snapshot
//! and captured evidence, and receive a deterministic, replayable links network.
//! Filesystem discovery and network capture stay at the boundary.

mod audit;
mod extract;
mod model;
mod repository;

pub use audit::audit_corpus;
pub use extract::requirement_claim;
pub use model::{
    AttachedEvidence, AuditConfig, AuditFinding, AuditedStatement, Claim, Contradiction,
    EvidenceCapture, EvidenceSelector, FindingKind, RepositoryAudit, RepositoryCorpus,
    RepositoryDocument, SourceKind, SourceLocation,
};
