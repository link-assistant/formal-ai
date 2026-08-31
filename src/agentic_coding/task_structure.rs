//! Answering a question about a task's own structure from Formal AI's
//! decomposition of it, rather than from the open web (issue #1066).
//!
//! "Work out whether migrating the billing database can be split further" is a
//! question with no external answer. Nothing on the web knows the task, and the
//! recursion that does know it is
//! [`crate::solver_handlers::task_decomposition`], which has answered the three
//! questions about a task -- is it atomic, what is its first step, what does it
//! split into -- since issue #847. The agentic planner had no route to it, so a
//! request of that shape fell past every tool-using route to the research
//! routers and came back with a search for its own words.
//!
//! That is what the issue-#1028 ladder asks at every one of its thirty interior
//! nodes, and it is the reason those nodes produced no usable evidence. The
//! route is not about the ladder: it recognises the question with the very
//! predicate the symbolic handler routes on
//! ([`looks_like_task_decomposition`]), so every phrasing and every language the
//! handler already reads reaches it, and no phrase is spelled out here.
//!
//! Placed ahead of the research routers and behind every route that inspects
//! the workspace: a task-structure question is answered by thinking about the
//! task, and needs no tool at all, which is why the plan is always
//! [`AgenticPlan::Final`].
//!
//! That last property is also why the route answers only on a turn on which no
//! tool has run yet. An answer composed from the request alone is the same
//! answer on every turn, so a route that can always answer would answer forever
//! -- and, worse, would answer *over* work already done. A leaf that asks the
//! agent to look at the repository is planned as a search, and the turn after
//! the search only has to report what came back; this route claimed that turn
//! and reported a decomposition of the instructions instead, which is how
//! thirty-one of the issue-#1028 ladder's thirty-two leaves came back with a
//! four-step template where their evidence should have been (issue #1066).

use super::planner::{AgenticPlan, trace_route};
use super::tool_result;
use crate::engine::{FormalAiEngine, normalize_prompt};
use crate::protocol::ChatMessage;
use crate::solver_handlers::looks_like_task_decomposition;

/// Plan the answer to a question about how a task decomposes.
///
/// Declines on anything the symbolic engine does not conclude
/// ([`crate::engine::SymbolicAnswer::is_inconclusive`]), so a prompt that only
/// looks like a decomposition question keeps falling through to the routes that
/// would have handled it before.
///
/// It declines for the same reason on an answer that only describes the search
/// it would run ([`crate::engine::SymbolicAnswer::defers_to_the_open_web`]).
/// This route exists because a task's structure has no answer on the web, so an
/// answer that defers there has not answered the question -- and returning it as
/// [`AgenticPlan::Final`] would end the turn with prose about a search engine
/// where the caller asked what the task splits into.
///
/// The last check is the one issue #1066 is named for: an answer that announces
/// an enumeration and enumerates nothing
/// ([`crate::engine::SymbolicAnswer::announces_a_list_it_does_not_make`]) is
/// hollow, and hollow evidence is worse than none, because the caller's harness
/// scores it green.
pub(super) fn plan_task_structure_step(
    messages: &[ChatMessage],
    task: &str,
) -> Option<AgenticPlan> {
    if !nothing_has_been_observed_yet(messages) {
        return None;
    }
    if !looks_like_task_decomposition(&normalize_prompt(task)) {
        return None;
    }
    // A request that specifies a document to compose states a deliverable, and
    // a deliverable is work to do rather than a task to classify. The ladder's
    // thirty-second and last leaf arrives under an "Atomic task" label whose
    // words alone carry the atomicity predicate and the task noun this route
    // reads, and only then states the deliverable -- "Produce a final evidence
    // note containing the selected tree level, node outcomes, test results, and
    // session id." So the route answered the question the *label* posed,
    // truthfully ("yes, that is atomic"), and the note the sentence after the
    // colon asked for was never composed (issue #1066).
    //
    // The test is [`super::note_composition::composed_document_specification_span`],
    // the same recogniser that keeps a specification from being transcribed as
    // literal bytes: a composition action applied to a document kind with two or
    // more named parts. Nothing about headings or labels is spelled out here --
    // what decides is that the caller named something to produce.
    if super::note_composition::composed_document_specification_span(task).is_some() {
        return None;
    }
    let answer = FormalAiEngine.answer(task);
    if answer.is_inconclusive()
        || answer.defers_to_the_open_web()
        || answer.announces_a_list_it_does_not_make()
    {
        return None;
    }
    let text = answer.answer.trim();
    if text.is_empty() {
        return None;
    }
    trace_route("task_structure", &answer.intent);
    Some(AgenticPlan::Final(text.to_owned()))
}

/// Whether the turn is still one this route may answer.
///
/// It is the same question [`super::shell_command::workspace_inspection_search_for_task`]
/// is asked at its call site, and this route has a stronger reason to ask it.
/// Every other route here plans a tool call and stands aside once that call has
/// been made; this one plans no call at all, so its answer does not change from
/// one turn to the next and it would keep claiming the turn for as long as the
/// conversation ran. What it displaces is worse than a repeat: the turn after a
/// search exists to report what the search found, and an answer reached without
/// looking has no standing to overrule one reached by looking.
fn nothing_has_been_observed_yet(messages: &[ChatMessage]) -> bool {
    !tool_result::has_latest_turn_result(messages)
}
