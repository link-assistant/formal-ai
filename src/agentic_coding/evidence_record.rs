//! Recording what an investigation found into a file the caller named (#1066).
//!
//! A caller can ask for two things at once: *find something out*, and *leave the
//! answer at a named place*. The Agent-CLI ladder does exactly that — "Inspect
//! the decomposition data model … Leave observable evidence in
//! `.agent-ladder/node-1.2-proof.md`. The first line must be exactly
//! `node_path=1.2`" — and so does any harness that reads a run's result from a
//! file rather than from the transcript.
//!
//! The literal-write route ([`super::general_planner`]) declines this shape: it
//! composes a write only when the request spells the bytes out, and here the
//! bytes are the *outcome* of work that has not happened yet. Where the prose is
//! close enough that its broad content parser recovers something anyway --
//! "produce a note *containing* the node outcomes" -- what keeps the route from
//! claiming the request is the pinned first line, which composed bytes lifted
//! out of the prose never carry; see
//! [`super::write_request::honours_pinned_first_line`].
//!
//! Left to the remaining routes, the named path is the only file-shaped token in
//! the request, so it was opened for reading and the run ended with the
//! evidence file never written.
//!
//! This module closes that gap without duplicating either half. It splits the
//! request into the delivery obligation (where the answer goes, and what its
//! first line must be) and the residual investigation, re-plans the residual
//! through the ordinary router, and turns the answer that router eventually
//! produces into the write the caller asked for. When the router cannot answer
//! the residual, this route declines too — an evidence file is never invented.

use super::capability_router::tool_for;
use super::planner::{
    AgenticPlan, Capability, plan_chat_step, plan_one, trace_route, write_arguments,
};
use super::progress::Progress;
use super::shell_command_policy::sentences;
use super::write_request::{pinned_first_line, stated_write_target, states_write_action};
use crate::protocol::{ChatMessage, MessageContent};

/// The delivery half of a request, separated from the work it asks for.
struct Obligation {
    /// The path the answer has to be written to.
    target: String,
    /// The exact opening line the caller pinned, when it pinned one.
    first_line: Option<String>,
    /// Everything the request says that is not about delivery.
    residual: String,
}

/// Split a request into its delivery obligation and the investigation left over.
///
/// The obligation is read one sentence at a time, and a sentence carries it only
/// when it applies a seed-defined write action *to* a seed-cued target path. Both
/// halves are required and both must be in the same sentence: "Read the file
/// `Cargo.toml`. Record what you find in `notes/report.md`." cues two paths, and
/// only the second one is being written to. Scoping the pair to a sentence is
/// what tells them apart, and it is the same scoping
/// [`super::shell_command`] uses to tell a command that is named from one that
/// is ordered (issue #907).
///
/// Declines when the residual is empty: a request whose every sentence is about
/// delivery states no work to do, so there is nothing to record.
fn parse_obligation(request: &str) -> Option<Obligation> {
    let mut target = None;
    let mut first_line = None;
    let mut residual = String::new();
    for sentence in sentences(request) {
        if let Some(line) = pinned_first_line(sentence.text) {
            first_line = first_line.or(Some(line));
            continue;
        }
        if target.is_none()
            && states_write_action(sentence.text)
            && let Some(named) = stated_write_target(sentence.text)
        {
            target = Some(named);
            continue;
        }
        residual.push_str(&request[sentence.span]);
    }
    let residual = residual.trim().to_owned();
    (!residual.is_empty())
        .then_some(())
        .and(target)
        .map(|target| Obligation {
            target,
            first_line,
            residual,
        })
}

/// Plan the next step of a "find this out and record it at PATH" request.
///
/// Three states, in the order they occur:
///
/// * the write has already been attempted — report what happened, truthfully;
/// * the router still wants tool calls for the residual — pass them through, so
///   the investigation runs under the ordinary routes;
/// * the router has an answer — write it to the named path, under the pinned
///   first line when the caller pinned one.
pub(super) fn plan_evidence_record_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let obligation = parse_obligation(task)?;
    let write_tool = tool_for(tool_names, Capability::Write)?;
    let progress = Progress::scan(messages);
    if progress.attempted_write_for(&obligation.target) {
        trace_route("evidence_record", "already_written");
        return Some(AgenticPlan::Final(if progress
            .successful_write_for(&obligation.target)
        {
            format!("Recorded the findings in `{}`.", obligation.target)
        } else {
            format!(
                "The findings could not be recorded in `{}`: the write step failed.",
                obligation.target
            )
        }));
    }
    let residual_messages = with_residual_request(messages, &obligation.residual)?;
    match plan_chat_step(&residual_messages, tool_names)? {
        plan @ AgenticPlan::ToolCalls(_) => {
            trace_route("evidence_record", "investigating");
            Some(plan)
        }
        AgenticPlan::Final(answer) => {
            trace_route("evidence_record", &obligation.target);
            let content = obligation.first_line.map_or_else(
                || format!("{}\n", answer.trim_end()),
                |line| format!("{line}\n\n{}\n", answer.trim_end()),
            );
            Some(plan_one(
                write_tool,
                write_arguments(&obligation.target, &content),
            ))
        }
    }
}

/// The same conversation with the latest user turn reduced to `residual`.
///
/// Only the request text changes: every tool result the investigation has
/// already collected stays in place, so the residual is re-planned with the
/// progress it has actually made rather than from scratch.
fn with_residual_request(messages: &[ChatMessage], residual: &str) -> Option<Vec<ChatMessage>> {
    let index = messages.iter().rposition(|message| message.role == "user")?;
    let mut residual_messages = messages.to_vec();
    residual_messages[index].content = MessageContent::Text(residual.to_owned());
    Some(residual_messages)
}
