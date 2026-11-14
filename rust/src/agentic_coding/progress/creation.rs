//! Scoped creation receipts retain their real mutation capability.
#[cfg(test)]
#[path = "../../../tests/fixtures/scoped-creation-evidence.rs"]
mod tests;
use super::creation_path::{
    creation_path_matches, projected_creation_path, workspace_relative_creation_path,
};
use super::source_freshness::{PostWriteCommandReceipt, read_only_attempt, read_only_capability};
use super::{Capability, Progress, ToolAttempt, source_read_path};
use crate::agentic_coding::capability_router::is_workspace_creation_tool;
use crate::agentic_coding::qualified_tool_observation::{
    Binding, QualifiedToolAttempt, canonical_creation,
};
use crate::agentic_coding::tool_result::{SourceReadObservation, command_argument};
use serde_json::Value;

pub(super) struct CreationDeclaration {
    target: String,
    content: String,
    span: (usize, usize),
}
pub(super) fn standing_request(messages: &[crate::protocol::ChatMessage]) -> Option<String> {
    let received = crate::protocol::latest_user_request(messages)?;
    Some(
        crate::agentic_coding::planner::continued_agent_task(messages, &received)
            .unwrap_or(received),
    )
}
fn owned_creation_declarations(request: &str, host_root: Option<&str>) -> Vec<CreationDeclaration> {
    let mut declarations: Vec<CreationDeclaration> = Vec::new();
    for (clause, span) in crate::obligation_ledger::clauses_with_spans(request) {
        let Some((target, content)) =
            crate::agentic_coding::general_planner::owned_declared_create_frame(&clause)
        else {
            continue;
        };
        if span.0 >= span.1
            || request.get(span.0..span.1).is_none()
            || declarations
                .iter()
                .any(|declaration| creation_path_matches(&declaration.target, &target, host_root))
        {
            return Vec::new();
        }
        declarations.push(CreationDeclaration {
            target,
            content,
            span,
        });
    }
    declarations
}
fn declared_creation(
    declarations: &[CreationDeclaration],
    mutation: &CreationMutation<'_>,
    host_root: Option<&str>,
) -> bool {
    declarations.iter().any(|declaration| {
        declaration.span.0 < declaration.span.1
            && creation_path_matches(&declaration.target, &mutation.path, host_root)
            && declaration.content == mutation.content
    })
}
fn disjoint_creation_targets(left: &str, right: &str) -> bool {
    let left_parts = left.replace('\\', "/");
    let right_parts = right.replace('\\', "/");
    let valid = |path: &str| {
        let bytes = path.as_bytes();
        let drive_prefix = bytes.first().is_some_and(|byte| byte.is_ascii_alphabetic())
            && bytes.get(1) == Some(&b':');
        !drive_prefix && path.split('/').all(|part| part != "." && part != "..")
    };
    let key = |path: &str| {
        path.split('/')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("/")
            .to_lowercase()
    };
    let rooted = |path: &str| path.starts_with('/') || path.starts_with('\\');
    valid(&left_parts)
        && valid(&right_parts)
        && rooted(left) == rooted(right)
        && key(&left_parts) != key(&right_parts)
}
fn declared_disjoint_mutation(
    progress: &Progress,
    mutation: &CreationMutation<'_>,
    attempt: &ToolAttempt,
    index: usize,
) -> bool {
    let host_root = progress.workspace_root.as_deref();
    let declarations = progress.creation_declarations.get_or_init(|| {
        progress
            .creation_request
            .as_deref()
            .map(|request| owned_creation_declarations(request, host_root))
            .unwrap_or_default()
    });
    if !declared_creation(declarations, mutation, host_root) {
        return false;
    }
    let mut candidates = progress.qualified_attempts.iter().filter(|frame| {
        bound_attempt(progress, frame).is_some_and(|(_, actual)| std::ptr::eq(actual, attempt))
    });
    let Some(frame) = candidates.next() else {
        return false;
    };
    if candidates.next().is_some() {
        return false;
    }
    let Some((path, _)) = creation_arguments(frame) else {
        return false;
    };
    let left = projected_creation_path(&mutation.observed_path, host_root)
        .unwrap_or_else(|| mutation.observed_path.clone());
    let right = projected_creation_path(&path, host_root).unwrap_or_else(|| path.clone());
    if !disjoint_creation_targets(&left, &right) {
        return false;
    }
    let Some(other) = progress.creation_mutation_for(&path) else {
        return false;
    };
    other.qualified
        && other.succeeded
        && other.index == Some(index)
        && declared_creation(declarations, &other, host_root)
}

pub(in crate::agentic_coding) struct CreationMutation<'a> {
    pub(in crate::agentic_coding) path: String,
    pub(in crate::agentic_coding) observed_path: String,
    pub(in crate::agentic_coding) workspace_relative_path: Option<String>,
    pub(in crate::agentic_coding) content: String,
    pub(in crate::agentic_coding) frame: &'a QualifiedToolAttempt,
    pub(in crate::agentic_coding) attempt: Option<&'a ToolAttempt>,
    pub(in crate::agentic_coding) index: Option<usize>,
    pub(in crate::agentic_coding) receipt_present: bool,
    pub(in crate::agentic_coding) qualified: bool,
    pub(in crate::agentic_coding) succeeded: bool,
}
pub(in crate::agentic_coding) struct CreationPreflight<'a> {
    pub(in crate::agentic_coding) attempted: bool,
    pub(in crate::agentic_coding) observation: Option<&'a SourceReadObservation>,
}
pub(in crate::agentic_coding) struct CreationReadback<'a> {
    pub(in crate::agentic_coding) current: bool,
    pub(in crate::agentic_coding) read_attempt: Option<&'a ToolAttempt>,
    pub(in crate::agentic_coding) command_receipt: Option<PostWriteCommandReceipt<'a>>,
    pub(in crate::agentic_coding) pending_read: bool,
    pub(in crate::agentic_coding) needs_fresh_read: bool,
}
fn empty_readback<'a>(current: bool) -> CreationReadback<'a> {
    CreationReadback {
        current,
        read_attempt: None,
        command_receipt: None,
        pending_read: false,
        needs_fresh_read: false,
    }
}
fn creation_arguments(frame: &QualifiedToolAttempt) -> Option<(String, String)> {
    if !is_workspace_creation_tool(frame.declared_tool()) {
        return None;
    }
    if frame.capability() == Capability::Edit {
        let creation = canonical_creation(frame.declaration_arguments())?;
        return Some((creation.path.to_owned(), creation.content));
    }
    if frame.capability() != Capability::Write {
        return None;
    }
    let path = source_read_path(frame.declaration_arguments())?;
    let object = frame.arguments_value().as_object()?;
    let mut content = None;
    for key in ["content", "contents", "text", "new_string"] {
        if let Some(value) = object.get(key) {
            let value = value.as_str()?;
            if content.is_some_and(|previous| previous != value) {
                return None;
            }
            content = Some(value);
        }
    }
    Some((path, content?.to_owned()))
}
fn bound_attempt<'a>(
    progress: &'a Progress,
    frame: &QualifiedToolAttempt,
) -> Option<(usize, &'a ToolAttempt)> {
    let mut matches = progress.attempts.iter().enumerate().filter(|(_, attempt)| {
        let arguments = attempt.arguments.as_deref().unwrap_or_default();
        let value = serde_json::from_str::<Value>(arguments)
            .unwrap_or_else(|_| Value::String(arguments.to_owned()));
        attempt.capability == frame.capability()
            && attempt.tool.as_deref() == Some(frame.declared_tool())
            && frame.arguments_match(&value)
            && attempt.raw_receipt.as_deref().unwrap_or(&attempt.detail) == frame.detail
    });
    let found = matches.next()?;
    matches.next().is_none().then_some(found)
}
impl Progress {
    pub(in crate::agentic_coding) fn creation_mutation_for(
        &self,
        path: &str,
    ) -> Option<CreationMutation<'_>> {
        for frame in self.qualified_attempts.iter().rev() {
            let Some((target, content)) = creation_arguments(frame) else {
                continue;
            };
            if !creation_path_matches(path, &target, self.workspace_root.as_deref()) {
                continue;
            }
            let bound = bound_attempt(self, frame);
            let qualified =
                frame.binding == Binding::Exact && frame.has_receipt() && bound.is_some();
            return Some(CreationMutation {
                path: path.to_owned(),
                workspace_relative_path: workspace_relative_creation_path(
                    &target,
                    self.workspace_root.as_deref(),
                ),
                observed_path: target,
                content,
                frame,
                attempt: bound.map(|(_, attempt)| attempt),
                index: bound.map(|(index, _)| index),
                receipt_present: frame.has_receipt(),
                qualified,
                succeeded: qualified
                    && frame.succeeded
                    && bound.is_some_and(|(_, attempt)| attempt.succeeded),
            });
        }
        None
    }
    pub(in crate::agentic_coding) fn creation_preflight_for(
        &self,
        path: &str,
    ) -> CreationPreflight<'_> {
        let unknown = |attempted| CreationPreflight {
            attempted,
            observation: None,
        };
        if self
            .qualified_attempts
            .iter()
            .any(|frame| !frame.has_receipt() && !read_only_capability(frame.capability()))
        {
            return unknown(true);
        }
        for frame in self.qualified_attempts.iter().rev() {
            if !read_only_capability(frame.capability()) {
                return unknown(false);
            }
            if frame.capability() != Capability::Read
                || !source_read_path(frame.declaration_arguments()).is_some_and(|observed| {
                    creation_path_matches(path, &observed, self.workspace_root.as_deref())
                })
            {
                continue;
            }
            let bound = bound_attempt(self, frame);
            return if frame.binding == Binding::Exact && frame.has_receipt() && bound.is_some() {
                CreationPreflight {
                    attempted: true,
                    observation: bound.and_then(|(_, attempt)| attempt.source_read.as_ref()),
                }
            } else {
                unknown(true)
            };
        }
        unknown(false)
    }
    pub(in crate::agentic_coding) fn creation_readback_for<'a>(
        &'a self,
        mutation: &CreationMutation<'a>,
        command: &str,
    ) -> CreationReadback<'a> {
        let (Some(index), Some(attempt)) = (mutation.index, mutation.attempt) else {
            return empty_readback(false);
        };
        let Some(frame_index) = self
            .qualified_attempts
            .iter()
            .position(|frame| std::ptr::eq(frame, mutation.frame))
        else {
            return empty_readback(false);
        };
        if !mutation.succeeded
            || self
                .attempts
                .get(index)
                .is_none_or(|actual| !std::ptr::eq(actual, attempt))
            || self
                .qualified_attempts
                .iter()
                .any(|frame| !frame.has_receipt() && !read_only_capability(frame.capability()))
        {
            return empty_readback(false);
        }
        let mut state = empty_readback(true);
        state.pending_read = self.qualified_attempts[frame_index + 1..]
            .iter()
            .any(|frame| {
                frame.capability() == Capability::Read
                    && !frame.has_receipt()
                    && source_read_path(frame.declaration_arguments()).is_some_and(|observed| {
                        creation_path_matches(
                            &mutation.path,
                            &observed,
                            self.workspace_root.as_deref(),
                        )
                    })
            });
        for (offset, attempt) in self.attempts[index + 1..].iter().enumerate().rev() {
            if attempt.capability == Capability::Run
                && attempt
                    .arguments
                    .as_deref()
                    .and_then(command_argument)
                    .as_deref()
                    == Some(command)
            {
                let raw = attempt.raw_receipt.as_deref().unwrap_or_default();
                let frame = self.qualified_command_attempt(command);
                let qualified = frame.is_some_and(|frame| {
                    frame.matches_receipt(attempt.tool.as_deref().unwrap_or_default(), raw)
                });
                state
                    .command_receipt
                    .get_or_insert(PostWriteCommandReceipt {
                        raw,
                        qualified,
                        succeeded: qualified
                            && attempt.succeeded
                            && frame.is_some_and(|frame| frame.succeeded),
                    });
                if !state
                    .command_receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.qualified)
                {
                    return empty_readback(false);
                }
                continue;
            }
            if !read_only_attempt(self, attempt) {
                if declared_disjoint_mutation(self, mutation, attempt, index + 1 + offset) {
                    continue;
                }
                let qualified = self
                    .qualified_attempts
                    .iter()
                    .filter(|frame| {
                        frame.binding == Binding::Exact
                            && frame.has_receipt()
                            && bound_attempt(self, frame)
                                .is_some_and(|(_, actual)| std::ptr::eq(actual, attempt))
                    })
                    .count()
                    == 1;
                let needs_fresh_read = qualified
                    && !state.pending_read
                    && state.read_attempt.is_none()
                    && state.command_receipt.is_none();
                state.current = false;
                state.needs_fresh_read = needs_fresh_read;
                return state;
            }
            if state.read_attempt.is_none()
                && attempt.capability == Capability::Read
                && attempt
                    .arguments
                    .as_deref()
                    .and_then(source_read_path)
                    .is_some_and(|observed| {
                        creation_path_matches(
                            &mutation.path,
                            &observed,
                            self.workspace_root.as_deref(),
                        )
                    })
            {
                state.read_attempt = Some(attempt);
            }
        }
        state
    }
}
