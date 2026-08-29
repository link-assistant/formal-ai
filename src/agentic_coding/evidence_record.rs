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
//! bytes are the *outcome* of work that has not happened yet.
//!
//! Left to the remaining routes, the named path is the only file-shaped token in
//! the request, so it was opened for reading and the run ended with the
//! evidence file never written.
//!
//! This module closes that gap without duplicating either half. It splits the
//! request into the delivery obligation (where the answer goes, and what its
//! first line must be) and the residual investigation, re-plans the residual
//! through the ordinary router, and turns the answer that router eventually
//! produces into the write the caller asked for.
//!
//! The agentic router is not the only thing that can answer the residual, and
//! for this shape of request it is usually not the thing that does. "Complete
//! recursive decomposition node 1.1.1, covering the four atomic tasks it names"
//! needs no tool: it is a question about task structure, which is what Formal
//! AI's symbolic engine answers. Delivering only what the *agentic* router produces
//! drops the obligation on every residual of that kind, and the request ends
//! with nothing written. So when the router has no plan, the residual is put to
//! [`crate::engine::FormalAiEngine`] and its answer delivered instead.
//!
//! An evidence file is still never invented. The engine's own verdict on
//! whether it reached a conclusion ([`crate::engine::SymbolicAnswer::is_inconclusive`],
//! [`crate::engine::SymbolicAnswer::defers_to_the_open_web`]) decides: an
//! unknown prompt, an ill-formed one, every clarification request and every
//! answer that only describes the web search it would run leave this route
//! declining exactly as before.

use super::capability_router::tool_for;
use super::planner::{
    AgenticPlan, Capability, plan_chat_step, plan_one, trace_route, write_arguments,
};
use super::progress::Progress;
use super::shell_command::carries_authoring_task;
use super::shell_command_policy::sentences;
use super::write_request::{
    bare_surfaces, first_action_cue_start, pinned_first_line, stated_write_target,
    states_write_action, tokens,
};
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
/// A sentence that asks for an artifact to be authored is never the delivery
/// half, however write-shaped it looks. "Today's date is Sunday. Create a file
/// `main.py` that prints Hello, world!" applies a write action to a cued path,
/// and reading it as delivery inverts the request: the caller's work becomes the
/// destination and their passing statement becomes the investigation, so the
/// Python file was written with prose about the date. The authoring test is
/// [`super::shell_command::carries_authoring_task`], the one requirement 3 of
/// issue #907 already states in exactly these terms.
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
            && !carries_authoring_task(&crate::engine::normalize_prompt(sentence.text))
            && states_write_action(sentence.text)
            && let Some(named) = stated_write_target(sentence.text)
        {
            target = Some(named);
            if let Some(work) = work_before_delivery(sentence.text) {
                residual.push_str(work);
                residual.push_str(". ");
            }
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

/// The part of a delivery sentence that is not about delivery.
///
/// Two sentences are the tidy way to ask for work and its delivery, and the
/// ladder's own nodes are written that way. English does not require it: "Break
/// the customer import rewrite into sub-tasks and record what you work out in
/// `import-split.md`" coordinates both halves into one sentence, and the
/// delivery half starts at the write action cue. Reading the sentence as
/// delivery and stopping there drops the work along with it, the residual comes
/// out empty, and the request is answered in the transcript with nothing
/// written -- which is the same silence issue #1066 is about, arrived at from
/// the other side.
///
/// Returns [`None`] when the sentence opens with its cue, which is the shape
/// where there is genuinely no work in front of the delivery.
fn work_before_delivery(sentence: &str) -> Option<&str> {
    let start = first_action_cue_start(&tokens(sentence))?;
    let work = without_trailing_separator(sentence.get(..start)?.trim());
    work.chars().any(char::is_alphanumeric).then_some(work)
}

/// `work` with the connective that introduced the delivery clause removed.
///
/// The connective belongs to the clause it opens, so it is delivery's, not the
/// work's. Which words those are is not a fact about writing files: it is the
/// vocabulary that ends one clause and starts the next, which the seed already
/// spells out in every registered language under
/// [`crate::seed::ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR`].
///
/// The separator has to be a whole word. "Run the import command record it in
/// `x.md`" ends its work on *command*, and a match on the last three letters
/// would hand back "Run the import comm".
fn without_trailing_separator(work: &str) -> &str {
    let lowered = work.to_lowercase();
    bare_surfaces(crate::seed::ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR)
        .iter()
        .filter(|separator| lowered.ends_with(separator.as_str()))
        .filter_map(|separator| work.get(..work.len() - separator.len()))
        .filter(|kept| kept.ends_with(char::is_whitespace))
        .map(str::trim_end)
        .min_by_key(|kept| kept.len())
        .unwrap_or(work)
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
    let answer = match plan_chat_step(&residual_messages, tool_names) {
        Some(plan @ AgenticPlan::ToolCalls(_)) => {
            trace_route("evidence_record", "investigating");
            return Some(plan);
        }
        Some(AgenticPlan::Final(answer)) => answer,
        None => {
            trace_route("evidence_record", "symbolic_residual");
            symbolic_answer(&obligation.residual)?
        }
    };
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

/// What Formal AI answers about the residual, when it reaches a conclusion.
///
/// The layering is the one [`super::general_execution`] already uses: the
/// agentic planner is a client of the symbolic engine, not a replacement for
/// it. Declining on an inconclusive answer is what keeps the delivery honest --
/// the caller asked for what was found out, and "nothing was" is not something
/// to write to a file and call evidence.
///
/// An answer that defers to the open web
/// ([`crate::engine::SymbolicAnswer::defers_to_the_open_web`]) is declined for
/// the same reason, and it is the sharper of the two cases because the text
/// reads like prose about the subject. "Today's date is Sunday. Create a file
/// `main.py` that prints Hello, world!" put the first sentence to the engine,
/// which described the search it would run for it; delivering that description
/// wrote a paragraph about `DuckDuckGo` into the caller's Python file. Where the
/// engine would search, this route has nothing to record and stands aside, and
/// the request goes on to the routes that recognise it.
///
/// An answer that announces an enumeration and enumerates nothing
/// ([`crate::engine::SymbolicAnswer::announces_a_list_it_does_not_make`]) is
/// declined last, and it is the case this whole route is most exposed to: the
/// text is non-empty, so it survives every check made of the file afterwards,
/// and a harness reading the file finds a heading with no list and calls the
/// node proved.
fn symbolic_answer(residual: &str) -> Option<String> {
    let answer = crate::engine::FormalAiEngine.answer(residual);
    if answer.is_inconclusive()
        || answer.defers_to_the_open_web()
        || answer.announces_a_list_it_does_not_make()
    {
        return None;
    }
    let text = answer.answer.trim().to_owned();
    (!text.is_empty()).then_some(text)
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
