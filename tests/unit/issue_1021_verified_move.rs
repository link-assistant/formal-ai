//! Issue #1021 / #944 (E92): a move is *performed and verified*, not issued.
//!
//! Issue #824 reported *"Move <dir> to <dir>"* being refused. The routing half
//! of that was answered on this branch — the request lowers to a concrete `mv
//! SOURCE DESTINATION` — and issue #944 asks for the other half: "verify source
//! exists, verify/create destination parent (mkdir -p), perform the move, verify
//! the result, and confirm to the user".
//!
//! So these tests do not assert that a command was *planned*. They drive the
//! planner's own tool loop with the results a shell would return and assert the
//! ordered recipe it walks, that a check which fails stops the recipe before the
//! action, and that the answer afterwards names what it rests on. The same
//! recipe is exercised end to end against a real filesystem by the write-effect
//! ladder rungs `824.L1`-`824.L5`.

use formal_ai::agentic_coding::mutating_action::verified_recipe;
use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::{ChatMessage, ToolCall};

const TOOLS: [&str; 2] = ["write_file", "run_shell_command"];

/// The qwen-code shell envelope, the shape the #902-#909 corpus recorded and the
/// one the write-effect ladder replays.
fn shell_result(command: &str, stdout: &str, exit_code: i32) -> String {
    format!(
        "Command: {command}\nDirectory: (root)\nOutput: {}\nError: (none)\nExit Code: {exit_code}\nSignal: 0\nProcess Group PGID: 0",
        if stdout.is_empty() { "(empty)" } else { stdout }
    )
}

fn next_call(messages: &[ChatMessage]) -> Option<PlannedToolCall> {
    match plan_chat_step(messages, &TOOLS)? {
        AgenticPlan::ToolCalls(mut calls) if calls.len() == 1 => Some(calls.remove(0)),
        AgenticPlan::Final(_) => None,
        other @ AgenticPlan::ToolCalls(_) => {
            panic!("expected one tool call or a final answer, got {other:?}")
        }
    }
}

fn final_answer(messages: &[ChatMessage]) -> String {
    match plan_chat_step(messages, &TOOLS) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected a final answer, got {other:?}"),
    }
}

fn record(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

fn command_of(call: &PlannedToolCall) -> String {
    let arguments: serde_json::Value =
        serde_json::from_str(&call.arguments).expect("tool arguments");
    arguments["command"]
        .as_str()
        .unwrap_or_else(|| panic!("a shell step should carry a command: {}", call.arguments))
        .to_owned()
}

/// Drive the planner until it answers, returning every command it ran and the
/// answer it finished with. `exit_codes` supplies the status of each step in
/// order; a missing entry means the step succeeded.
fn drive(prompt: &str, exit_codes: &[i32]) -> (Vec<String>, String) {
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut commands = Vec::new();
    for step in 0..10 {
        let Some(call) = next_call(&messages) else {
            break;
        };
        let command = command_of(&call);
        let code = exit_codes.get(step).copied().unwrap_or(0);
        record(&mut messages, &call, &shell_result(&command, "", code));
        commands.push(command);
    }
    (commands, final_answer(&messages))
}

/// The recipe is derived from the seed effect table, so it is the same list the
/// planner walks and the same list a maintainer edits.
#[test]
fn a_move_expands_into_preconditions_preparation_action_and_postconditions() {
    assert_eq!(
        verified_recipe("mv report.txt archive/report.txt"),
        Some(vec![
            String::from("test -e report.txt"),
            String::from("test ! -e archive/report.txt"),
            String::from("mkdir -p -- archive"),
            String::from("mv report.txt archive/report.txt"),
            String::from("test -e archive/report.txt"),
            String::from("test ! -e report.txt"),
        ])
    );
}

/// A copy keeps its source, and nothing in the planner names `cp`: the
/// difference is one line of `data/seed/shell-intents.lino`.
#[test]
fn a_copy_declares_the_same_shape_with_a_source_that_survives() {
    assert_eq!(
        verified_recipe("cp a.txt backup/a.txt"),
        Some(vec![
            String::from("test -e a.txt"),
            String::from("test ! -e backup/a.txt"),
            String::from("mkdir -p -- backup"),
            String::from("cp a.txt backup/a.txt"),
            String::from("test -e backup/a.txt"),
            String::from("test -e a.txt"),
        ])
    );
}

/// A destination with no directory component still has a parent, and `mkdir -p
/// -- .` is the no-op that keeps the recipe one shape instead of two.
#[test]
fn a_destination_in_the_working_directory_prepares_the_working_directory() {
    let recipe = verified_recipe("mv old.txt new.txt").expect("recipe");
    assert!(
        recipe.contains(&String::from("mkdir -p -- .")),
        "{recipe:?}"
    );
}

/// Read-only intents keep their single-shot path: the effect table is what makes
/// a command a recipe, and nothing declares one for `ls` or `cat`.
#[test]
fn a_command_with_no_declared_effect_is_not_a_recipe() {
    for command in ["ls", "cat note.txt", "git status", "rm old.txt"] {
        assert_eq!(verified_recipe(command), None, "{command}");
    }
}

#[test]
fn a_requested_move_runs_its_checks_around_the_action() {
    let (commands, answer) = drive("move the file report.txt to archive/report.txt", &[]);

    assert_eq!(
        commands,
        vec![
            String::from("test -e report.txt"),
            String::from("test ! -e archive/report.txt"),
            String::from("mkdir -p -- archive"),
            String::from("mv report.txt archive/report.txt"),
            String::from("test -e archive/report.txt"),
            String::from("test ! -e report.txt"),
        ]
    );
    assert!(
        answer.contains("mv report.txt archive/report.txt"),
        "the answer should name the action it took: {answer}"
    );
    assert!(
        answer.contains("test -e archive/report.txt"),
        "and the checks the claim rests on: {answer}"
    );
}

/// The conflict case: the destination is occupied, so the recipe stops on the
/// precondition and the action never runs.
#[test]
fn a_move_onto_an_occupied_destination_stops_before_it_acts() {
    let (commands, answer) = drive("move the file report.txt to archive/report.txt", &[0, 1]);

    assert_eq!(
        commands,
        vec![
            String::from("test -e report.txt"),
            String::from("test ! -e archive/report.txt"),
        ],
        "a failed precondition ends the recipe where it stopped"
    );
    assert!(
        answer.contains("test ! -e archive/report.txt"),
        "the answer should name the check that stopped it: {answer}"
    );
    assert!(
        answer.contains('1'),
        "and the status the workspace answered with: {answer}"
    );
    assert!(
        !answer.to_lowercase().contains("completed the"),
        "a blocked move must not claim completion: {answer}"
    );
}

/// A missing source is the other precondition, and it stops the recipe on step
/// one — before `mkdir -p` has created anything.
#[test]
fn a_move_of_a_missing_source_stops_on_the_first_check() {
    let (commands, answer) = drive("move the file report.txt to archive/report.txt", &[1]);

    assert_eq!(commands, vec![String::from("test -e report.txt")]);
    assert!(!answer.to_lowercase().contains("completed the"), "{answer}");
}
