use super::*;

pub(super) fn plan_chat_step_routes(
    messages: &[ChatMessage],
    tool_names: &[&str],
    received: String,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    // Load-time precedence join (plan 10 leaf 18): first plan step of a run
    // proves the seed still names the cascade this function is about to walk.
    checked_route_precedence();
    trace_route("agentic_received", &received);
    // Agent compacts a long tool loop by asking the model to summarize it, then
    // starts the next request with the protocol turn "Continue if you have next
    // steps". The work is still present, but only inside the assistant's
    // `Conversation summary:` envelope. Treating the protocol turn as a fresh
    // request loses that work and sends the sentence itself to web search.
    // Recover only our own summary envelope, immediately before the exact
    // continuation turn; ordinary user requests that happen to say "continue"
    // keep their normal meaning.
    let effective = continued_agent_task(messages, &received).unwrap_or(received);
    // An *unmarked* harness preamble is still the caller talking (issue #907,
    // follow-up). `<session_context>`-style markup is stripped upstream in
    // `crate::protocol`, but Hive Mind's adapters concatenated their workflow
    // policy and the objective into one untagged user message, so the tell has
    // to be the objective delimiter the caller wrote instead of a tag:
    // everything before a line-anchored `Issue to solve:` / `Task:` / `Goal:`
    // lead is the caller's framing, and only the text after it is the request.
    //
    // Routing the whole message let the preamble win: "When running sudo
    // commands, run them in the background." paired a run verb with the `sudo`
    // shell token and planned bare `sudo`, and "Your prepared working
    // directory: …" planned `pwd`, in both cases dropping the repository work
    // that followed. The general planner already read the objective this way
    // (issue #904); every other route now reads it the same way, so one
    // boundary serves the whole router rather than one recipe.
    let task = objective_text(&effective).to_owned();
    trace_route("agentic_task", &task);
    if let Some(plan) =
        super::super::document_recipe::plan_structured_repair(&task, messages, tool_names)
    {
        return Some(plan);
    }
    // A bare continuation cue with nothing to resume is still the cue here,
    // and the `agentic_continuation` conversation handler answers it; the
    // words of the cue are never a request (issue #1095).
    if let steps::SourceOrOwnedGoalStep::Claimed(plan) =
        steps::plan_request_preflight(&task, messages, tool_names, result)
    {
        return plan;
    }
    if
    // An edit request's block is its payload: a `when … then` inside it is
    // text being written, not a skill being taught (PR #1188 T57).
    !has_authoritative_literal_write(&task)
        && super::super::general_planner::compose_edit_request(&task).is_none()
        && looks_like_skill_description(super::super::positional_edit::own_text(&task))
    {
        return None;
    }
    // Explicit computer-use plans precede broad write/search routing. Each primitive carries
    // explicit pre/postconditions and is executed by the advertising client.
    // Ahead of them, quotes that do not pair leave no telling the quoted text
    // from the instruction, so the request is declined before any arm reads its
    // payload as words to act on (PR #1188 G71).
    if let steps::SourceOrOwnedGoalStep::Claimed(plan) =
        steps::plan_source_or_owned_goal_step(&task, messages, tool_names, result)
    {
        return plan;
    }
    // An explicit exact-content marker makes the following bytes authoritative.
    // Claim this narrow shape before edit/source semantics inspect the payload:
    // literal bytes may themselves say "rename X to Y" (issue #708). Broader
    // file-write requests remain below the semantic coding routes.
    if has_authoritative_literal_write(&task)
        && !super::super::general_planner::owns_complete_edit_request(&task)
        && capability_router::workspace_creation_tool(tool_names).is_some()
    {
        if let Some(nodes) = task_obligations::obligations(&task) {
            return obligations::plan_obligations_step(&task, messages, tool_names, &nodes, result);
        }
        if let Some(plan) = compose_general_change_plan(&task) {
            return Some(plan_general_change_step(
                messages, tool_names, &plan, result,
            ));
        }
    }
    if task.lines().any(|line| line.trim_end() == "```lino")
        && let Some(report) = learning_report::route(&task)
    {
        return Some(report.plan_step(&task, messages, tool_names));
    }
    // Bind a program's semantic operands before treating its source path as a
    // destination for a report about the rest of the request.
    if let Some(mut answer) =
        crate::coding::program_contract::answer(&task, &mut crate::event_log::EventLog::default())
    {
        if let Some(recipe) = answer.execution_recipe.as_mut()
            && super::ci_workflow::requested_in(&task)
        {
            super::ci_workflow::attach(recipe, &task);
        }
        if let Some(plan) =
            super::command_reroute::plan_symbolic_command_reroute(messages, tool_names, &answer)
        {
            return Some(plan);
        }
    }
    // "Find this out and leave the answer in FILE" (issue #1066). This sits ahead
    // of every route that reads a request's lone file-shaped token, because that
    // token is the *destination* here and opening it for reading ends the run with
    // the evidence file unwritten. It sits behind the literal-write routes above,
    // which own a request that spells its bytes out; this one owns the request
    // whose bytes still have to be found.
    //
    // It sits ahead of the change routes below for the same reason it sits ahead
    // of the readers: a request can carry both halves. The ladder's leaf says
    // "Edit `src/engine_responses.rs` … Then create `agent-ladder-effects/…lino`
    // recording what you changed. Leave evidence in `.agent-ladder/…-proof.md`",
    // and the change routes answer `Final` for the whole request the moment the
    // edit lands, so the two records the caller verifies are never written and
    // the node fails `missing_proof` having done the work. This route peels one
    // delivery at a time and re-plans the residual (see `parse_obligation`), so
    // the change route still receives the edit -- with only the edit left in it.
    if let Some(plan) =
        evidence_record::plan_evidence_record_step(&task, messages, tool_names, result)
    {
        return Some(plan);
    }
    plan_settled_routes(&task, messages, tool_names, result)
}
