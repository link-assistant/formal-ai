use super::{
    AgenticPlan, ChatMessage, PLAN_PATH, ToolCall, compose_general_change_plan,
    observed_plan_tools, plan_chat_step,
};

#[test]
fn command_capture_parent_file_is_a_real_failure_without_false_completion() {
    let prompt = "Run 'printf learned-output' and write its exact stdout to reports/learned.txt";
    let plan = compose_general_change_plan(prompt).expect("declared output plan");
    let mut workspace = observed_plan_tools::ToolWorkspace::new(prompt);
    workspace
        .write_initial(PLAN_PATH, "")
        .expect("empty auxiliary journal");
    workspace
        .write_initial("reports", "existing file")
        .expect("real obstructing parent");
    let tools = ["read", "write", "run_shell_command"];
    let mut messages = vec![ChatMessage::user(prompt)];
    for turn in 0..8 {
        match plan_chat_step(&messages, &tools).expect("planned output task") {
            AgenticPlan::ToolCalls(calls) => {
                let call = &calls[0];
                let id = format!("blocked-parent-{turn}");
                let observation = workspace.execute(&id, call);
                let failed = observation.is_error;
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    &id,
                    &call.tool,
                    call.arguments.clone(),
                )]));
                messages.push(observation);
                if failed {
                    assert!(workspace.read(&plan.target).is_err());
                    assert!(
                        matches!(plan_chat_step(&messages, &tools), Some(AgenticPlan::Final(answer)) if !answer.contains("Completed the general change request"))
                    );
                    return;
                }
            }
            AgenticPlan::Final(_) => {
                panic!("the blocked parent must be observed as a failed tool call")
            }
        }
    }
    panic!("bounded real capture did not observe the parent error");
}

#[test]
fn command_capture_cold_auxiliary_absence_is_not_the_parent_failure() {
    let prompt = "Run 'printf learned-output' and write its exact stdout to reports/learned.txt";
    let plan = compose_general_change_plan(prompt).expect("declared output plan");
    let mut workspace = observed_plan_tools::ToolWorkspace::new(prompt);
    workspace
        .write_initial("reports", "existing file")
        .expect("real obstructing parent");
    let tools = ["read", "write", "run_command"];
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut cold_auxiliary_absence = false;
    for turn in 0..8 {
        match plan_chat_step(&messages, &tools).expect("planned output task") {
            AgenticPlan::ToolCalls(calls) => {
                let call = &calls[0];
                let id = format!("blocked-parent-{turn}");
                let observation = workspace.execute(&id, call);
                let failed = observation.is_error;
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    &id,
                    &call.tool,
                    call.arguments.clone(),
                )]));
                messages.push(observation);
                if failed
                    && observation_path_is_auxiliary_missing(
                        messages.last().expect("actual observation"),
                    )
                {
                    assert!(!cold_auxiliary_absence);
                    cold_auxiliary_absence = true;
                    continue;
                }
                if failed {
                    assert!(cold_auxiliary_absence);
                    assert!(workspace.read(&plan.target).is_err());
                    assert!(
                        matches!(plan_chat_step(&messages, &tools), Some(AgenticPlan::Final(answer)) if !answer.contains("Completed the general change request"))
                    );
                    return;
                }
            }
            AgenticPlan::Final(_) => {
                panic!("the blocked parent must be observed as a failed tool call")
            }
        }
    }
    panic!("bounded real capture did not observe the parent error");
}

fn observation_path_is_auxiliary_missing(observation: &ChatMessage) -> bool {
    observation.source_read.as_ref().is_some_and(|read| {
        read["path"] == PLAN_PATH
            && read["success"] == false
            && read["complete"] == false
            && read["format"] == "raw"
            && read["error_code"] == "ENOENT"
    })
}
