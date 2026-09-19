//! Issue #1138 B6 (plan 06, leaf 13): Telegram execution is observed or refused.

use std::time::Duration;

use formal_ai::execution_box::ExecutionBackend;
use formal_ai::telegram_runtime::{
    TELEGRAM_EXECUTION_HARD_LIMIT, TelegramCodeExecution, execute_telegram_code_request,
};

const PROMPTS: &[(&str, &str)] = &[
    (
        "en",
        "Run this and tell me exactly what it prints: print(sum(range(1, 11)))",
    ),
    (
        "ru",
        "Запусти это и скажи точно, что оно печатает: print(sum(range(1, 11)))",
    ),
    (
        "hi",
        "इसे चलाओ और मुझे ठीक-ठीक बताओ कि यह क्या छापता है: print(sum(range(1, 11)))",
    ),
    (
        "zh",
        "运行这个并准确告诉我它打印了什么：print(sum(range(1, 11)))",
    ),
    (
        "es",
        "Ejecuta esto y dime exactamente qué imprime: print(sum(range(1, 11)))",
    ),
];

#[test]
fn telegram_without_a_backend_is_honest_in_five_languages() {
    for (language, prompt) in PROMPTS {
        let outcome = execute_telegram_code_request(prompt, None, Duration::from_secs(1));
        let TelegramCodeExecution::Refused { answer } = outcome else {
            panic!("{language}: an absent backend must refuse, got {outcome:?}");
        };
        assert!(
            !answer.trim().is_empty(),
            "{language}: the seed must provide an honesty sentence"
        );
        assert!(
            !answer.contains("55"),
            "{language}: an unobserved result may never be shown: {answer}"
        );
    }
}

#[test]
fn telegram_with_a_backend_reports_observed_output_and_evidence() {
    for (language, prompt) in PROMPTS {
        let outcome = execute_telegram_code_request(
            prompt,
            Some(ExecutionBackend::HostSandbox),
            Duration::from_secs(2),
        );
        let TelegramCodeExecution::Observed {
            answer,
            evidence,
            ladder,
        } = outcome
        else {
            panic!("{language}: a configured backend must record the run, got {outcome:?}");
        };
        assert!(
            answer.contains("55"),
            "{language}: the answer shows the observed bytes"
        );
        assert_eq!(evidence.exit_code, Some(0), "{language}");
        assert_eq!(evidence.for_need, "telegram_code_execution", "{language}");
        assert_eq!(evidence.produced_by, "telegram_execution_box", "{language}");
        assert!(answer.contains(&evidence.evidence_id), "{language}");
        assert!(
            ladder.is_none(),
            "{language}: ordinary source is run once, not rewritten"
        );
    }
}

#[test]
fn telegram_policy_has_the_required_ten_minute_hard_failure() {
    assert_eq!(TELEGRAM_EXECUTION_HARD_LIMIT, Duration::from_secs(600));
}

#[test]
fn prose_without_an_execution_request_never_reaches_the_box() {
    assert_eq!(
        execute_telegram_code_request(
            "Explain what print(sum(range(1, 11))) means.",
            Some(ExecutionBackend::HostSandbox),
            Duration::from_secs(1),
        ),
        TelegramCodeExecution::NotRequested,
    );
}
