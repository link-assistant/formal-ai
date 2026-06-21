//! Replays captured shared-dialog requirements from issue #552.

use formal_ai::{ConversationTurn, ExecutionSurface, SolverConfig, UniversalSolver};

fn solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

#[test]
fn chatgpt_shared_shell_loop_prompt_returns_readable_single_line() {
    let response = solver().solve(
        "box@87ffc301f5eb:~$ sleep 30m && hive-cleanup -f\n\n\
         make a loop of that (infinite), answer with only single line",
    );

    assert_eq!(response.intent, "shell_command_transform");
    assert_eq!(
        response.answer,
        "while true; do sleep 30m && hive-cleanup -f; done"
    );
    assert!(!response.answer.contains('\n'));
    assert!(response.answer.contains("; do sleep 30m"));
}

#[test]
fn chatgpt_shared_screen_followup_uses_previous_loop_command() {
    let user_prompt = "box@87ffc301f5eb:~$ sleep 30m && hive-cleanup -f\n\n\
                       make a loop of that (infinite), answer with only single line";
    let prior_answer = "while true; do sleep 30m && hive-cleanup -f; done";
    let response = solver().solve_with_history(
        "Can we use `screen -R auto-cleanup` to execute that line inside, also making it all using single line?",
        &[
            ConversationTurn::user(user_prompt),
            ConversationTurn::assistant(prior_answer),
        ],
    );

    assert_eq!(response.intent, "shell_command_transform");
    assert_eq!(
        response.answer,
        "screen -dmS auto-cleanup bash -c 'while true; do sleep 30m && hive-cleanup -f; done'"
    );
    assert!(!response.answer.contains('\n'));
}
