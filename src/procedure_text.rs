//! Retrieved page → ordered step list with provenance (issue #1138, plan 04 L9,
//! extended by plan 02 L10–L11).
//!
//! One "ordered step with provenance" record for the whole tree:
//! [`ProcedureStepRecord`] is what a synthesised guide step
//! (`how_to_guide::GuideStep::to_step_record`) and a captured page both become,
//! and `formalization::procedures::ExtractedProcedure::from_step_records` is its
//! only consumer-side constructor. HTML handling is delegated to the existing
//! `how_to_guide::extract` helpers so there is one HTML extractor in the tree.

use crate::event_log::EventLog;
use crate::seed::SourceRecord;
use crate::source_fetch::{CachedSourceClient, SourceCapture, SourceTransport};
use crate::source_walk::LookupBounds;

/// Fewer than this many items is not a procedure; a one-step guess is refused.
pub const MIN_PROCEDURE_STEPS: usize = 2;

/// One instruction recovered from a retrieved page, with the bytes it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStepRecord {
    pub ordinal: usize,
    pub text: String,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
    pub license_name: String,
    pub license_url: String,
    pub depth: usize,
}

/// How a page's procedure was recovered, recorded so a replay is auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepShape {
    /// `<ol>` / `<li>` items, the wikiHow and Wikipedia "Algorithm" shape.
    OrderedList,
    /// Numbered prose lines ("1." / "Step 2:"), the Stack Exchange shape.
    NumberedProse,
    /// A pseudocode block delimited by `<pre>` / `<code>`.
    PseudocodeBlock,
    /// A definition sentence that states a recurrence or closed form.
    DefinitionSentence,
}

/// Extract an ordered procedure from one capture. Returns `None` rather than a
/// one-step guess: fewer than [`MIN_PROCEDURE_STEPS`] items is not a procedure.
pub fn steps_from_capture(
    _capture: &SourceCapture,
    _source: &SourceRecord,
    _bounds: &LookupBounds,
) -> Option<(StepShape, Vec<ProcedureStepRecord>)> {
    todo!("plan 02 leaf L10")
}

/// Whether the bytes this step was read from may be emitted verbatim, or only
/// their abstract shape reused. A share-alike capture is always `ShapeOnly`.
#[must_use]
pub fn reuse_mode(_step: &ProcedureStepRecord) -> crate::coding::program_ir::ReuseMode {
    todo!("plan 02 leaf L10")
}

/// Retrieve and extract for one need phrase across the registry's coding
/// sources that declare the need kind, in the registry's consultation order.
pub fn retrieve_procedure<T: SourceTransport>(
    _phrase: &str,
    _prose_language: &str,
    _client: &CachedSourceClient<T>,
    _bounds: &LookupBounds,
    _log: &mut EventLog,
) -> Vec<(StepShape, Vec<ProcedureStepRecord>)> {
    todo!("plan 02 leaf L11")
}
