//! Ordered source text → [`ExtractedProcedure`] (issue #1138, plan 04 L9–L10).
//!
//! A step is `verified` only when an execution record says so; extraction never
//! sets it. An extracted procedure reaches the #919 ledger through
//! [`ExtractedProcedure::to_coding_procedure_source`], under the existing
//! execution and review gate rather than beside it.

use crate::procedure_text::ProcedureStepRecord;

/// An ordered, provenance-bearing procedure extracted from a trusted source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedProcedure {
    pub id: String,
    pub goal: String,
    pub language: String,
    pub steps: Vec<ProcedureStep>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub license_name: String,
    pub license_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStep {
    pub position: usize,
    pub imperative: String,
    pub object: Option<String>,
    /// `"<doc_id>@<start>:<end>"`, the exact span the step was read from.
    pub source_span: String,
    /// Set only by an execution record, never by extraction.
    pub verified: bool,
}

/// Extract an ordered procedure from any captured ordered text: a how-to
/// guide's steps, a documentation page's numbered list, an answer's ordered
/// block.
#[must_use]
pub fn procedure_from_steps(
    _goal: &str,
    _steps: &[ProcedureStepRecord],
    _language: &str,
) -> Option<ExtractedProcedure> {
    todo!("plan 04 leaf L9")
}

impl ExtractedProcedure {
    /// The only constructor. `GuideStep::to_step_record()` in
    /// `src/how_to_guide.rs` is how a synthesised guide reaches it.
    #[must_use]
    pub fn from_step_records(
        _goal: &str,
        _steps: &[ProcedureStepRecord],
        _language: &str,
    ) -> Option<Self> {
        todo!("plan 04 leaf L9")
    }

    /// Render into the versioned shape `coding_research_learning` already
    /// gates, so an extracted procedure enters the ledger through the existing
    /// execution and review boundary.
    #[must_use]
    pub fn to_coding_procedure_source(&self) -> String {
        todo!("plan 04 leaf L10")
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 04 leaf L9")
    }
}
