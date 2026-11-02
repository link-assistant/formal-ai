//! Target-bound post-write source observations and mutation invalidation.
use super::{Capability, Progress, ToolAttempt, source_read_path};

impl Progress {
    /// Exact target read after the most recent potentially mutating attempt.
    fn latest_post_write_read_attempt_for(&self, path: &str) -> Option<&ToolAttempt> {
        self.attempts_after_latest_write(path)?
            .iter()
            .rev()
            .take_while(|attempt| {
                matches!(
                    attempt.capability,
                    Capability::Read
                        | Capability::ReadMany
                        | Capability::Search
                        | Capability::Fetch
                        | Capability::Grep
                        | Capability::Glob
                        | Capability::ListDir
                        | Capability::AskUser
                )
            })
            .find(|attempt| {
                attempt.capability == Capability::Read
                    && attempt
                        .arguments
                        .as_deref()
                        .and_then(source_read_path)
                        .as_deref()
                        == Some(path)
            })
    }

    pub(in crate::agentic_coding) fn attempted_read_after_latest_write_for(
        &self,
        path: &str,
    ) -> bool {
        self.latest_post_write_read_attempt_for(path).is_some()
    }

    /// Failed or missing latest receipts cannot reuse an earlier observation.
    pub(in crate::agentic_coding) fn source_read_after_latest_write_for(
        &self,
        path: &str,
    ) -> Option<&crate::agentic_coding::tool_result::SourceReadObservation> {
        self.latest_post_write_read_attempt_for(path)
            .and_then(|attempt| attempt.source_read.as_ref())
    }
}
