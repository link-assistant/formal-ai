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
fn shared_shell_loop_prompt_supports_all_prompt_languages() {
    struct Case {
        language: &'static str,
        instruction: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            instruction: "make a loop of that (infinite), answer with only single line",
        },
        Case {
            language: "ru",
            instruction: "сделай из этого бесконечный цикл, ответь одной строкой",
        },
        Case {
            language: "hi",
            instruction: "इसे अनंत लूप बनाओ, केवल एक पंक्ति में उत्तर दो",
        },
        Case {
            language: "zh",
            instruction: "把它做成无限循环, 只用一行回答",
        },
    ];

    for case in cases {
        let response = solver().solve(&format!(
            "box@87ffc301f5eb:~$ sleep 30m && hive-cleanup -f\n\n{}",
            case.instruction
        ));

        assert_eq!(
            response.intent, "shell_command_transform",
            "language {} should route to shell command transform, got: {}",
            case.language, response.intent
        );
        assert_eq!(
            response.answer, "while true; do sleep 30m && hive-cleanup -f; done",
            "language {} should preserve the readable loop command",
            case.language
        );
    }
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

#[test]
fn shared_screen_followup_supports_all_prompt_languages() {
    struct Case {
        language: &'static str,
        followup: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            followup: "Use `screen -R auto-cleanup` to execute that line inside, answer in one line.",
        },
        Case {
            language: "ru",
            followup: "Используй `screen -R auto-cleanup`, выполни эту строку внутри, ответь одной строкой.",
        },
        Case {
            language: "hi",
            followup: "`screen -R auto-cleanup` का उपयोग करके उस पंक्ति को अंदर चलाओ, एक पंक्ति में उत्तर दो।",
        },
        Case {
            language: "zh",
            followup: "使用 `screen -R auto-cleanup` 在里面执行那一行, 只用一行回答。",
        },
    ];

    let prior_answer = "while true; do sleep 30m && hive-cleanup -f; done";
    for case in cases {
        let response = solver()
            .solve_with_history(case.followup, &[ConversationTurn::assistant(prior_answer)]);

        assert_eq!(
            response.intent, "shell_command_transform",
            "language {} should route to shell command transform, got: {}",
            case.language, response.intent
        );
        assert_eq!(
            response.answer,
            "screen -dmS auto-cleanup bash -c 'while true; do sleep 30m && hive-cleanup -f; done'",
            "language {} should preserve the readable screen command",
            case.language
        );
    }
}
