use std::collections::BTreeSet;

use crate::associative_persistence::AssociativeMemory;
use crate::relative_meta_logic::{SourceTier, Stance, StatementAssessment};

/// One text document in an immutable repository snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryDocument {
    pub path: String,
    pub content: String,
}

impl RepositoryDocument {
    #[must_use]
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }
}

/// A repository snapshot and its complete tracked-path index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositoryCorpus {
    pub documents: Vec<RepositoryDocument>,
    pub tracked_paths: BTreeSet<String>,
    pub skipped_paths: Vec<String>,
}

impl RepositoryCorpus {
    #[must_use]
    pub fn from_documents(documents: Vec<RepositoryDocument>) -> Self {
        let tracked_paths = documents
            .iter()
            .map(|document| document.path.clone())
            .collect();
        Self {
            documents,
            tracked_paths,
            skipped_paths: Vec::new(),
        }
    }
}

/// Statement-bearing syntax recognized at a source location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceKind {
    Prose,
    CodeComment,
    Structured,
}

impl SourceKind {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Prose => "prose",
            Self::CodeComment => "code_comment",
            Self::Structured => "structured",
        }
    }
}

/// Stable origin of an extracted statement.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceLocation {
    pub path: String,
    pub line: usize,
    pub kind: SourceKind,
}

/// A symbolic fact. An exclusive claim accepts one value per subject/predicate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Claim {
    pub subject: String,
    pub predicate: String,
    pub value: String,
    pub exclusive: bool,
}

impl Claim {
    #[must_use]
    pub fn exclusive(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            value: value.into(),
            exclusive: true,
        }
    }
}

/// How a replayable evidence capture selects statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceSelector {
    StatementText(String),
    Claim {
        subject: String,
        predicate: String,
        value: Option<String>,
    },
}

/// Evidence captured outside the pure audit core, with provenance for replay.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCapture {
    pub selector: EvidenceSelector,
    pub source_label: String,
    pub source_url: String,
    pub tier: SourceTier,
    pub stance: Stance,
    pub strength: f64,
    pub captured_at: String,
    pub sha256: String,
}

impl EvidenceCapture {
    #[must_use]
    pub fn for_statement(
        statement: impl Into<String>,
        source_label: impl Into<String>,
        source_url: impl Into<String>,
        tier: SourceTier,
        stance: Stance,
        strength: f64,
    ) -> Self {
        Self {
            selector: EvidenceSelector::StatementText(statement.into()),
            source_label: source_label.into(),
            source_url: source_url.into(),
            tier,
            stance,
            strength,
            captured_at: String::new(),
            sha256: String::new(),
        }
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn for_claim(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        value: Option<String>,
        source_label: impl Into<String>,
        source_url: impl Into<String>,
        tier: SourceTier,
        stance: Stance,
        strength: f64,
    ) -> Self {
        Self {
            selector: EvidenceSelector::Claim {
                subject: subject.into(),
                predicate: predicate.into(),
                value,
            },
            source_label: source_label.into(),
            source_url: source_url.into(),
            tier,
            stance,
            strength,
            captured_at: String::new(),
            sha256: String::new(),
        }
    }

    #[must_use]
    pub fn with_capture(
        mut self,
        captured_at: impl Into<String>,
        sha256: impl Into<String>,
    ) -> Self {
        self.captured_at = captured_at.into();
        self.sha256 = sha256.into();
        self
    }
}

/// Captured provenance paired with the mass it contributed to the posterior.
#[derive(Debug, Clone, PartialEq)]
pub struct AttachedEvidence {
    pub capture: EvidenceCapture,
    pub effective_mass: f64,
}

/// One extracted and assessed repository statement.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditedStatement {
    pub id: String,
    pub text: String,
    pub location: SourceLocation,
    pub claim: Option<Claim>,
    pub evidence: Vec<AttachedEvidence>,
    pub assessment: StatementAssessment,
    pub relative_weight: f32,
}

/// Two or more incompatible values for an exclusive claim key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contradiction {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub statement_ids: Vec<String>,
    pub proposed_resolution: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    ImprobableClaim,
    RequirementContradiction,
}

impl FindingKind {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::ImprobableClaim => "improbable_claim",
            Self::RequirementContradiction => "requirement_contradiction",
        }
    }
}

/// An append-only audit observation linked to its supporting statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFinding {
    pub id: String,
    pub kind: FindingKind,
    pub statement_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AuditConfig {
    pub temperature: f32,
    pub diagnostics: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            diagnostics: false,
        }
    }
}

/// Complete deterministic result, including its learned associative network.
#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryAudit {
    pub statements: Vec<AuditedStatement>,
    pub contradictions: Vec<Contradiction>,
    pub findings: Vec<AuditFinding>,
    pub learning: AssociativeMemory,
    pub skipped_paths: Vec<String>,
}
