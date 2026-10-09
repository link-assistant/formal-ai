//! Render exact source observations without parsing file content as transport.
use super::super::final_result::{
    FinalDisposition, FinalPayloadRole, FinalResult, record_with_role,
};
use super::super::{progress::Progress, tool_result};
use super::{AgenticPlan, FileReadMode, ToolResultRecord, file_read_final_answer};

pub(super) fn read_observation(
    records: &[ToolResultRecord],
    progress: &Progress,
    path: &str,
) -> Option<tool_result::SourceReadObservation> {
    if let Some(read) = progress.source_read_for(path) {
        return Some(tool_result::SourceReadObservation {
            status: read.status,
            complete: read.complete,
            source: read.source.clone(),
            error: read.error.clone(),
        });
    }
    let legacy = super::records::read_record_for_path(records, path)?;
    let mut read =
        tool_result::source_read_observation(&legacy.content, legacy.is_error, None, path);
    read.complete = false;
    Some(read)
}
pub(super) fn source_read_answer(
    paths: &[String],
    mode: &FileReadMode,
    records: &[ToolResultRecord],
    progress: &Progress,
    request: &str,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let reads = paths
        .iter()
        .map(|path| read_observation(records, progress, path))
        .collect::<Vec<_>>();
    for (path, read) in paths.iter().zip(&reads) {
        if let Some(error) = read.as_ref().and_then(|read| read.error.as_deref()) {
            return Some(record_with_role(
                AgenticPlan::Final(tool_result::render_failure(path, error, request)),
                FinalDisposition::Failure,
                "file_read_observed",
                FinalPayloadRole::Finding,
                result,
            ));
        }
    }
    let reads = reads.into_iter().collect::<Option<Vec<_>>>()?;
    let complete = reads.iter().all(|read| read.complete);
    let contents = paths
        .iter()
        .zip(reads)
        .map(|(path, read)| (path.clone(), read.source.unwrap_or_default()))
        .collect::<Vec<_>>();
    Some(record_with_role(
        AgenticPlan::Final(file_read_final_answer(mode, &contents, request)),
        if complete {
            FinalDisposition::Finding
        } else {
            FinalDisposition::Unknown
        },
        "file_read_observed",
        if mode == &FileReadMode::Audit {
            FinalPayloadRole::AuditReport
        } else {
            FinalPayloadRole::Finding
        },
        result,
    ))
}
