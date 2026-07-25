//! The capability menu must never be the answer to a question the user asked
//! about something else (issue #842, ladder nodes 827.L3.b / 827.L4.b / 826.L2.b).
//!
//! The menu is a correct answer to "what can you do?" and to nothing else. The
//! ladder found it printed for a bare demonstrative follow-up ("так что это
//! такое то?") and the Russian loanword phrasing of the report intent
//! ("Зарепорти баг") simply because those surfaces had been filed under the
//! `capability_query` meaning.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use formal_ai::solver::{SolverConfig, UniversalSolver};

/// The opening line of the Russian capability menu, as seeded in
/// `data/seed/multilingual-responses.lino`.
const RU_MENU_MARKER: &str = "Вот что я умею";

fn answer(prompt: &str) -> String {
    UniversalSolver::new(SolverConfig::default())
        .solve(prompt)
        .answer
}

#[test]
fn a_bare_demonstrative_question_asks_what_is_meant() {
    for prompt in [
        "Так что это такое то?",
        "Что означает слово это в предложении: так что это такое то?",
    ] {
        let answer = answer(prompt);
        assert!(!answer.contains(RU_MENU_MARKER), "{prompt}: {answer}");
        assert!(!answer.contains("Приветствия"), "{prompt}: {answer}");
        assert!(answer.contains("«это»"), "{prompt}: {answer}");
    }
}

/// "What can you do?" itself must keep its menu — the fix removes one wrong
/// surface, not the intent.
#[test]
fn the_capability_query_still_gets_the_menu() {
    assert!(answer("Что ты умеешь?").contains(RU_MENU_MARKER));
}

/// Russian loanword phrasings of the report intent must reach the same place
/// the English and native-Russian ones do.
///
/// The report route lives in agent mode, so this goes through the HTTP API with
/// a tool set advertised, exactly as the ladder does. `Зарепорти баг` (826.L2.b)
/// is the ladder node that fell through to the unknown-prompt refusal while its
/// sibling `Сообщи об ошибке` (826.L3.b) and the English `Report a bug`
/// (826.L4.b) both reached the report menu.
#[test]
fn russian_report_phrasings_agree_with_the_english_one() {
    for prompt in ["Report a bug", "Сообщи об ошибке", "Зарепорти баг"] {
        let answer = agent_answer(prompt);
        assert!(!answer.contains(RU_MENU_MARKER), "{prompt}: {answer}");
        assert!(!answer.contains("не смог определить"), "{prompt}: {answer}");
        assert!(
            !answer.contains("could not determine"),
            "{prompt}: {answer}"
        );
    }
}

/// The assistant reply to `prompt` in agent mode, with the ladder's tool set
/// advertised.
fn agent_answer(prompt: &str) -> String {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}],
        "tools": [serde_json::json!({"type": "function", "function": {
            "name": "bash",
            "parameters": {
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"]
            }
        }})]
    });
    enable_http_agent_mode_for_current_process();
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: serde_json::Value = serde_json::from_str(&response.body).expect("JSON response");
    response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}
