use super::{
    CodingResearchApproval, CodingResearchError, CodingResearchExecution,
    ResearchedCodingProcedure, error,
};

/// Accept an extracted procedure as a second typed input shape, under the same
/// execution + review gate the researched path already passes through (issue
/// #1138, plan 04 L10).
///
/// A procedure that has not been executed, or whose license forbids commercial
/// reuse, is shown to the user but refused for promotion; the refusal names the
/// license it refused on.
pub fn adopt_extracted_procedure(
    procedure: &crate::formalization::procedures::ExtractedProcedure,
    execution: Option<&CodingResearchExecution>,
    approval: &CodingResearchApproval,
) -> Result<ResearchedCodingProcedure, CodingResearchError> {
    if license_forbids_commercial_reuse(&procedure.license_name) {
        return Err(error(format!(
            "coding_research_license_forbids_promotion:{}",
            procedure.license_name
        )));
    }
    let Some(execution) = execution else {
        return Err(error("coding_research_execution_missing"));
    };
    if !approval.granted || approval.reviewer.trim().is_empty() {
        return Err(error("coding_research_human_approval_required"));
    }

    // The legacy execution record names a researched workspace rewrite, not
    // the individual ordered steps it ran. Accepting it as proof for a
    // different extracted procedure would manufacture verification. Until the
    // shared execution-evidence record can name every step, require exact
    // content identity and otherwise keep the candidate proposal-only.
    if execution.procedure_id != procedure.id {
        return Err(error("coding_research_execution_procedure_mismatch"));
    }
    Err(error("coding_research_execution_step_evidence_missing"))
}

/// Where the examples behind a decomposed procedure were read from: the
/// capture provenance and license the adoption gate requires of every step
/// (issue #1164 R7).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DecomposedProcedureSource {
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
    pub license_name: String,
    pub license_url: String,
}

/// Promote a procedure generalized from decomposed code examples (issue
/// #1164 R7) through the same execution-and-approval gate as
/// [`adopt_extracted_procedure`].
///
/// The decomposer leaves every step's provenance and license empty on
/// purpose; this fills them from the capture the examples came from, builds
/// the typed extracted procedure, and hands it to the gate, so the license
/// refusal, the execution requirement and the named review all still apply.
/// A procedure whose steps cannot form an extracted procedure (fewer than
/// two shared parts, or incomplete provenance) is refused before the gate.
#[cfg(feature = "meta-language")]
pub fn adopt_decomposed_procedure(
    procedure: &crate::code_example_knowledge::GeneralizedProcedure,
    goal: &str,
    source: &DecomposedProcedureSource,
    execution: Option<&CodingResearchExecution>,
    approval: &CodingResearchApproval,
) -> Result<ResearchedCodingProcedure, CodingResearchError> {
    let records = procedure
        .procedure_step_records()
        .into_iter()
        .map(|mut record| {
            record.source_url.clone_from(&source.source_url);
            record.sha256.clone_from(&source.sha256);
            record.fetched_at.clone_from(&source.fetched_at);
            record.license_name.clone_from(&source.license_name);
            record.license_url.clone_from(&source.license_url);
            record
        })
        .collect::<Vec<_>>();
    let extracted = crate::formalization::procedures::ExtractedProcedure::from_step_records(
        goal, &records, "en",
    )
    .ok_or_else(|| error("coding_research_decomposed_procedure_incomplete"))?;
    adopt_extracted_procedure(&extracted, execution, approval)
}

fn license_forbids_commercial_reuse(license: &str) -> bool {
    license
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|component| component.eq_ignore_ascii_case("nc"))
}
