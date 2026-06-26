//! Issue #461: after the Russian capabilities answer advertises Hello World code
//! generation, the follow-up "На php не получится написать?" must inherit that
//! Hello World task and use the cached PHP oracle instead of falling to unknown.

use formal_ai::{ConversationTurn, UniversalSolver};

#[test]
fn russian_capabilities_follow_up_can_request_php_hello_world() {
    let solver = UniversalSolver::default();
    let capabilities_prompt = "Что ты умеешь делать?";
    let capabilities = solver.solve(capabilities_prompt);
    assert_eq!(capabilities.intent, "capabilities");
    assert!(
        capabilities.answer.contains("Hello World"),
        "setup should advertise Hello World generation, got: {}",
        capabilities.answer
    );

    let history = [
        ConversationTurn::user(capabilities_prompt),
        ConversationTurn::assistant(capabilities.answer),
    ];
    let response = solver.solve_with_history("На php не получится написать?", &history);

    assert_eq!(
        response.intent, "write_program_oracle_hello_world_php",
        "PHP follow-up should route through the cached coding oracle, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```php"),
        "answer must include a PHP code fence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Hello World Collection"),
        "answer must keep source attribution, got: {}",
        response.answer
    );
}
