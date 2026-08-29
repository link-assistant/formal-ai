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

use super::planner::{AgenticPlan, trace_route};
use crate::engine::{FormalAiEngine, normalize_prompt};
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
pub(super) fn plan_task_structure_step(task: &str) -> Option<AgenticPlan> {
    if !looks_like_task_decomposition(&normalize_prompt(task)) {
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
