//! Target-bound post-write observations and conservative mutation invalidation.
use super::{Capability, Progress, ToolAttempt, source_read_path};
use crate::agentic_coding::general_planner::shell_quote;
use crate::agentic_coding::tool_result::command_argument;

pub(in crate::agentic_coding) struct PostWriteCommandReceipt<'a> {
    pub(in crate::agentic_coding) raw: &'a str,
    pub(in crate::agentic_coding) qualified: bool,
    pub(in crate::agentic_coding) succeeded: bool,
}

pub fn owned_creation_readback_command(target: &str) -> String {
    crate::seed::agentic_message(
        "literal-create-shell-readback",
        &[("path", &shell_quote(target))],
    )
}

pub(in crate::agentic_coding) fn read_only_capability(capability: Capability) -> bool {
    matches!(
        capability,
        Capability::Read
            | Capability::ReadMany
            | Capability::Search
            | Capability::Fetch
            | Capability::Grep
            | Capability::Glob
            | Capability::ListDir
            | Capability::AskUser
    )
}

fn read_only_attempt(progress: &Progress, attempt: &ToolAttempt) -> bool {
    if read_only_capability(attempt.capability) {
        return true;
    }
    if attempt.capability != Capability::Run {
        return false;
    }
    let Some(arguments) = attempt.arguments.as_deref() else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(arguments) else {
        return false;
    };
    if value.as_object().is_none_or(|object| object.len() != 1) {
        return false;
    }
    let Some(command) = value.get("command").and_then(serde_json::Value::as_str) else {
        return false;
    };
    let (Some(raw), Some(tool)) = (attempt.raw_receipt.as_deref(), attempt.tool.as_deref()) else {
        return false;
    };
    if !progress
        .qualified_command_attempt(command)
        .is_some_and(|frame| frame.matches_receipt(tool, raw))
    {
        return false;
    }
    progress.research_attempts().iter().any(|write| {
        write.capability == Capability::Write
            && write.succeeded
            && write
                .arguments
                .as_deref()
                .and_then(source_read_path)
                .is_some_and(|path| owned_creation_readback_command(&path) == command)
    })
}

impl Progress {
    /// A recognized exact readback cannot invalidate another file's observation.
    pub(in crate::agentic_coding) fn latest_post_write_read_attempt_for(
        &self,
        path: &str,
    ) -> Option<&ToolAttempt> {
        self.attempts_after_latest_write(path)?
            .iter()
            .rev()
            .take_while(|attempt| read_only_attempt(self, attempt))
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

    /// Only an exact fresh command frame can witness a post-write shell readback.
    pub(in crate::agentic_coding) fn post_write_command_receipt_for(
        &self,
        path: &str,
        command: &str,
    ) -> Option<PostWriteCommandReceipt<'_>> {
        for attempt in self.attempts_after_latest_write(path)?.iter().rev() {
            if attempt.capability == Capability::Run
                && attempt
                    .arguments
                    .as_deref()
                    .and_then(command_argument)
                    .as_deref()
                    == Some(command)
            {
                let raw = attempt.raw_receipt.as_deref().unwrap_or("");
                let qualified = self
                    .qualified_command_attempt(command)
                    .is_some_and(|frame| {
                        frame.succeeded
                            && frame
                                .matches_receipt(attempt.tool.as_deref().unwrap_or_default(), raw)
                    });
                return Some(PostWriteCommandReceipt {
                    raw,
                    qualified,
                    succeeded: attempt.succeeded,
                });
            }
            if !read_only_attempt(self, attempt) {
                return None;
            }
        }
        None
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

pub(in crate::agentic_coding) struct PostMutationReadState<'a> {
    pub(in crate::agentic_coding) current: bool,
    pub(in crate::agentic_coding) attempt: Option<&'a ToolAttempt>,
}

impl Progress {
    pub(in crate::agentic_coding) fn attempted_mutation_for(
        &self,
        path: &str,
        capability: Capability,
    ) -> bool {
        self.qualified_attempts
            .iter()
            .any(|frame| frame.capability() == capability && frame.targets_path(path))
    }
    /// Actual preimage before the target Edit, without an intervening potential effect.
    pub(in crate::agentic_coding) fn source_read_before_latest_mutation_for(
        &self,
        path: &str,
        capability: Capability,
    ) -> Option<&crate::agentic_coding::tool_result::SourceReadObservation> {
        let index = self.attempts.iter().rposition(|attempt| {
            attempt.capability == capability
                && attempt
                    .arguments
                    .as_deref()
                    .and_then(source_read_path)
                    .as_deref()
                    == Some(path)
        })?;
        for attempt in self.attempts[..index].iter().rev() {
            if !read_only_attempt(self, attempt) {
                return None;
            }
            if attempt.capability == Capability::Read
                && attempt
                    .arguments
                    .as_deref()
                    .and_then(source_read_path)
                    .as_deref()
                    == Some(path)
            {
                let observed = attempt.source_observation()?;
                if !attempt.succeeded || observed.error.is_some() || !observed.complete {
                    return None;
                }
                let mutation = &self.attempts[index];
                let mutation_arguments: serde_json::Value =
                    serde_json::from_str(mutation.arguments.as_deref()?).ok()?;
                let end = self.qualified_attempts.iter().rposition(|frame| {
                    frame.capability() == capability
                        && frame.arguments_match(&mutation_arguments)
                        && mutation
                            .tool
                            .as_deref()
                            .is_some_and(|tool| frame.matches_receipt(tool, &mutation.detail))
                })?;
                let read_arguments: serde_json::Value =
                    serde_json::from_str(attempt.arguments.as_deref()?).ok()?;
                let start = self.qualified_attempts[..end].iter().rposition(|frame| {
                    frame.capability() == Capability::Read
                        && frame.arguments_match(&read_arguments)
                        && attempt
                            .tool
                            .as_deref()
                            .is_some_and(|tool| frame.matches_receipt(tool, &attempt.detail))
                })?;
                if self.qualified_attempts[start + 1..end]
                    .iter()
                    .any(|frame| !frame.has_receipt() && !read_only_capability(frame.capability()))
                {
                    return None;
                }
                return Some(observed);
            }
        }
        None
    }

    /// Pending or observed foreign mutations cannot lend a later target Read authority.
    pub(in crate::agentic_coding) fn post_mutation_read_state_for(
        &self,
        path: &str,
        capability: Capability,
        arguments: &serde_json::Value,
    ) -> PostMutationReadState<'_> {
        let unavailable = || PostMutationReadState {
            current: false,
            attempt: None,
        };
        let Some(frame) = self.latest_attempt_for(capability, arguments) else {
            return unavailable();
        };
        let Some(index) = self
            .qualified_attempts
            .iter()
            .position(|candidate| std::ptr::eq(candidate, frame))
        else {
            return unavailable();
        };
        if self.qualified_attempts[index + 1..]
            .iter()
            .any(|candidate| {
                !candidate.has_receipt() && !read_only_capability(candidate.capability())
            })
        {
            return unavailable();
        }
        let mut read = None;
        for attempt in self.attempts.iter().rev() {
            if read_only_attempt(self, attempt) {
                if read.is_none()
                    && attempt.capability == Capability::Read
                    && attempt
                        .arguments
                        .as_deref()
                        .and_then(source_read_path)
                        .as_deref()
                        == Some(path)
                {
                    read = Some(attempt);
                }
                continue;
            }
            let actual = attempt
                .arguments
                .as_deref()
                .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok());
            let current = attempt.capability == capability
                && attempt.succeeded
                && actual.as_ref().is_some_and(|actual| {
                    crate::agentic_coding::qualified_tool_observation::argument_values_equal(
                        actual, arguments,
                    )
                })
                && frame.succeeded
                && attempt
                    .tool
                    .as_deref()
                    .is_some_and(|tool| frame.matches_receipt(tool, &attempt.detail));
            return PostMutationReadState {
                current,
                attempt: if current { read } else { None },
            };
        }
        unavailable()
    }
}
