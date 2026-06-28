//! GitHub repository traffic visibility questions (issue #497).
//!
//! The reported Russian prompt asks whether someone can know if anybody visited
//! the assistant's GitHub repository. These tests pin the whole class across the
//! supported chat languages: GitHub exposes aggregate repository traffic to
//! authorized repository users, but not the identities of individual visitors.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn assert_github_repository_traffic_answer(prompt: &str) {
    let response = answer(prompt);
    assert_eq!(
        response.intent, "github_repository_traffic",
        "prompt {prompt:?} should route to github_repository_traffic, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("GitHub"),
        "prompt {prompt:?} should keep the repository host explicit, got {}",
        response.answer,
    );
    assert!(
        response.answer.contains("docs.github.com"),
        "prompt {prompt:?} should cite official GitHub docs, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("source:https://docs.github.com")),
        "prompt {prompt:?} should record official GitHub documentation, got {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("github_repository_traffic:privacy")),
        "prompt {prompt:?} should record the individual-identity limitation, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn github_repository_traffic_questions_are_multilingual() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    for case in [
        Case {
            language: "en",
            prompt: "Can I know who visited my GitHub repo?",
        },
        Case {
            language: "ru",
            prompt: "можно ли узнать заходил ли кто либо в твое репо на github?",
        },
        Case {
            language: "hi",
            prompt: "क्या मैं जान सकता हूँ कि मेरे GitHub रेपो में कौन आया?",
        },
        Case {
            language: "zh",
            prompt: "能知道谁访问过我的 GitHub 仓库吗？",
        },
    ] {
        assert!(
            ["en", "ru", "hi", "zh"].contains(&case.language),
            "unexpected test language {}",
            case.language,
        );
        assert_github_repository_traffic_answer(case.prompt);
    }
}
