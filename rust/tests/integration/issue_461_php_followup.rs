//! Issue #461: after the Russian capabilities answer advertises Hello World code
//! generation, the follow-up "На php не получится написать?" must inherit that
//! Hello World task instead of falling to unknown.
//!
//! The guarantee issue #461 asked for is the *inheritance*: a follow-up that
//! names only a language still resolves to the task the previous turn
//! advertised. Which route answers it is a detail of what the catalog covers,
//! and that changed under issue #1021, which catalogued PHP so the Russian
//! request of issue #723 would be answered by generalization rather than by a
//! rule written for its wording. PHP therefore graduated from the coding oracle
//! to the catalog the way Kotlin did under issue #921 — see
//! `issue_412_oracle_languages.rs`, where the same graduation is asserted rather
//! than deleted — so this test pins the catalog route and the answer it carries.

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
        response.intent, "write_program",
        "the follow-up must inherit the Hello World task and take the catalog \
         route PHP graduated to, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        response.answer.contains("```php"),
        "answer must include a PHP code fence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("<?php"),
        "answer must carry the catalogued PHP template, got: {}",
        response.answer
    );
    // A real `php` toolchain verified the catalogued templates, so the answer
    // reports execution rather than borrowing an unverified claim.
    assert!(
        response.answer.contains("Вывод:"),
        "answer must report the program's output, got: {}",
        response.answer
    );
}
