//! Promoted project lookup tests.
//!
//! These tests pin down the "what is <project>?" behavior for the curated
//! registry described in `data/seed/projects.lino`, generic repository URL
//! routing, promotion switches, and the
//! formalize-summarize-deformalize pipeline in `src/summarization.rs`.
//! Splitting them out of `prompt_variations.rs` keeps each test file under
//! the repository's 1000-line cap (see `scripts/check-file-size.rs`).

use formal_ai::{ConversationTurn, FormalAiEngine, SolverConfig, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn russian_hive_mind_prompt_prefers_link_assistant_project() {
    let response = answer("Что такое Hive Mind?");
    assert_eq!(
        response.intent, "project_lookup",
        "Hive Mind should not fall through to a generic concept or Wikipedia-style answer: {}",
        response.answer,
    );
    assert!(response.answer.contains("link-assistant/hive-mind"));
    // Russian localized description comes from data/seed/projects.lino via the
    // summarization pipeline ("Hive Mind — это ИИ, который ...").
    assert!(
        response.answer.contains("ИИ"),
        "Russian Hive Mind answer should carry the localized ИИ description, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project:promoted:")),
        "expected promoted project evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn english_hive_mind_prompt_prefers_link_assistant_project() {
    let response = answer("What is Hive Mind?");
    assert_eq!(
        response.intent, "project_lookup",
        "Hive Mind should resolve to the curated project answer, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(response.answer.contains("link-assistant/hive-mind"));
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("ai that controls ais")
            || lower.contains("orchestrates")
            || lower.contains("the ai"),
        "English Hive Mind answer should describe the project, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project:promoted:")),
        "expected promoted project evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn associative_project_promotion_can_be_disabled() {
    let solver = UniversalSolver::new(SolverConfig {
        associative_project_promotion: false,
        ..SolverConfig::default()
    });
    let response = solver.solve("What is Hive Mind?");
    assert_eq!(
        response.intent, "project_lookup",
        "disabling promotion should keep the generic project lookup path, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(
        !response.answer.contains("link-assistant/hive-mind"),
        "promotion-off answer must not privilege the Link Assistant repository, got {}",
        response.answer,
    );
    assert!(
        response.answer.contains("GitHub")
            && response.answer.contains("GitLab")
            && response.answer.contains("Bitbucket"),
        "generic project lookup should cover repository hosts, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project_lookup:promotion:disabled")),
        "expected disabled-promotion evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn curated_project_concept_prompt_routes_to_project_lookup() {
    let response = answer("What is link-cli?");
    assert_eq!(
        response.intent, "project_lookup",
        "curated link-cli concept lookup should route through project_lookup, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("link-foundation/link-cli"),
        "link-cli answer should link to the canonical repository, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project:promoted:")),
        "expected promoted project evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn curated_project_lookup_records_summarization_evidence() {
    let response = answer("What is command-stream?");
    assert_eq!(response.intent, "project_lookup");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("summarization:mode")),
        "project lookup should log a summarization mode event, got {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("summarization:language")),
        "project lookup should log a summarization language event, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn linksplatform_repository_is_promoted_by_default() {
    let response = answer("What is https://github.com/linksplatform/Documentation?");
    assert_eq!(response.intent, "project_lookup");
    assert!(
        response.answer.contains("linksplatform/Documentation"),
        "LinksPlatform repository should be listed as a promoted project, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project:promoted:linksplatform/Documentation")),
        "expected linksplatform promotion evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn explicit_formal_ai_repository_url_still_routes_to_project_lookup() {
    let response = answer("What is https://github.com/link-assistant/formal-ai?");
    assert_eq!(
        response.intent, "project_lookup",
        "explicit formal-ai repository URLs should not be treated as identity prompts",
    );
    assert!(
        response.answer.contains("link-assistant/formal-ai"),
        "formal-ai repository lookup should link to the repository, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project:promoted:link-assistant/formal-ai")),
        "expected formal-ai promotion evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn github_repository_url_routes_to_generic_project_lookup() {
    let response = answer("What is https://github.com/rust-lang/rust?");
    assert_eq!(
        response.intent, "project_lookup",
        "explicit GitHub repository URLs should route to project_lookup, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(response.answer.contains("rust-lang/rust"));
    assert!(response.answer.contains("GitHub"));
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project_lookup:repository:github:rust-lang/rust")),
        "expected GitHub repository evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn repository_url_review_request_beats_capability_question_prefix() {
    let response = answer("ты можешь сделать кодревью https://github.com/netkeep80/anum_docs ?");
    assert_eq!(
        response.intent, "project_lookup",
        "explicit repository review requests should not be swallowed by capability phrasing, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(response.answer.contains("netkeep80/anum_docs"));
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("project_lookup:repository:github:netkeep80/anum_docs")),
        "expected GitHub repository evidence, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn issue_556_repository_lookup_language_followup_reanswers_in_russian() {
    let solver = UniversalSolver::default();
    let previous_prompt = "ты можешь сделать кодревью https://github.com/netkeep80/anum_docs ?";
    let history = [
        ConversationTurn::user(previous_prompt),
        ConversationTurn::assistant(
            "This is a repository lookup for \
             [netkeep80/anum_docs](https://github.com/netkeep80/anum_docs) on GitHub.",
        ),
    ];

    let response =
        solver.solve_with_history("я не понимаю по английски, напиши по русски", &history);

    assert_eq!(
        response.intent, "project_lookup",
        "language retarget follow-up should replay the previous repository lookup, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("Это запрос о репозитории"),
        "Russian retarget should render the generic repository lookup in Russian, got {}",
        response.answer,
    );
    assert!(response.answer.contains("netkeep80/anum_docs"));
    assert!(
        !response.answer.contains("This is a repository lookup"),
        "reported English answer should not be repeated verbatim: {}",
        response.answer,
    );
    assert!(
        response
            .links_notation
            .contains("project_lookup:repository:github netkeep80/anum_docs"),
        "retargeted answer should preserve repository evidence, got {}",
        response.links_notation,
    );
    assert!(
        response.links_notation.contains("language_to ru"),
        "retargeted answer should record the requested language, got {}",
        response.links_notation,
    );
}

#[test]
fn issue_556_repository_lookup_language_followup_supports_seeded_languages() {
    let solver = UniversalSolver::default();
    let previous_prompt = "ты можешь сделать кодревью https://github.com/netkeep80/anum_docs ?";
    let history = [
        ConversationTurn::user(previous_prompt),
        ConversationTurn::assistant(
            "This is a repository lookup for \
             [netkeep80/anum_docs](https://github.com/netkeep80/anum_docs) on GitHub.",
        ),
    ];
    let cases = [
        (
            "I do not understand Russian, write in English",
            "This is a repository lookup",
            "language_to en",
        ),
        (
            "я не понимаю по английски, напиши по русски",
            "Это запрос о репозитории",
            "language_to ru",
        ),
        (
            "मुझे अंग्रेजी समझ नहीं आती, हिंदी में लिखो",
            "यह GitHub पर रिपॉजिटरी",
            "language_to hi",
        ),
        (
            "我不懂英语，用中文",
            "这是 GitHub 上的仓库查询",
            "language_to zh",
        ),
    ];

    for (follow_up, expected_fragment, language_trace) in cases {
        let response = solver.solve_with_history(follow_up, &history);
        assert_eq!(
            response.intent, "project_lookup",
            "follow-up {follow_up:?} should replay project_lookup, got {} -> {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.contains(expected_fragment),
            "follow-up {follow_up:?} should render requested language fragment {expected_fragment:?}, got {}",
            response.answer,
        );
        assert!(
            response.answer.contains("netkeep80/anum_docs"),
            "follow-up {follow_up:?} should preserve the repository slug, got {}",
            response.answer,
        );
        assert!(
            response
                .links_notation
                .contains("project_lookup:repository:github netkeep80/anum_docs"),
            "follow-up {follow_up:?} should preserve repository evidence, got {}",
            response.links_notation,
        );
        assert!(
            response.links_notation.contains(language_trace),
            "follow-up {follow_up:?} should record {language_trace}, got {}",
            response.links_notation,
        );
    }
}

#[test]
fn response_language_marker_with_current_object_does_not_replay_prior_repository_lookup() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user(
            "ты можешь сделать кодревью https://github.com/netkeep80/anum_docs ?",
        ),
        ConversationTurn::assistant(
            "This is a repository lookup for \
             [netkeep80/anum_docs](https://github.com/netkeep80/anum_docs) on GitHub.",
        ),
    ];

    let response = solver.solve_with_history("Tell me about Telegram Ads in Russian", &history);

    assert_eq!(
        response.intent, "concept_lookup",
        "a current-turn concept request with a response-language marker should not replay history, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("Реклама в Telegram"),
        "expected the current concept answer in Russian, got {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("netkeep80/anum_docs"),
        "current-turn concept request should not preserve prior repository slug, got {}",
        response.answer,
    );
}

#[test]
fn gitlab_repository_url_routes_to_generic_project_lookup() {
    let response = answer("Describe https://gitlab.com/gitlab-org/gitlab");
    assert_eq!(response.intent, "project_lookup");
    assert!(response.answer.contains("gitlab-org/gitlab"));
    assert!(response.answer.contains("GitLab"));
}

#[test]
fn bitbucket_repository_url_routes_to_generic_project_lookup() {
    let response = answer("Describe https://bitbucket.org/atlassian/python-bitbucket");
    assert_eq!(response.intent, "project_lookup");
    assert!(response.answer.contains("atlassian/python-bitbucket"));
    assert!(response.answer.contains("Bitbucket"));
}

#[test]
fn http_fetch_of_curated_github_url_describes_project_via_summarization() {
    let response = answer("fetch https://github.com/link-assistant/hive-mind");
    assert_eq!(
        response.intent, "http_fetch",
        "GitHub URL fetch should still resolve as http_fetch, got {} -> {}",
        response.intent, response.answer,
    );
    assert!(
        response.answer.contains("link-assistant/hive-mind")
            || response.answer.to_lowercase().contains("the ai")
            || response.answer.to_lowercase().contains("hive mind"),
        "curated-URL fetch should describe the project, got {}",
        response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("http_fetch:curated_project")),
        "curated GitHub URL fetch should log http_fetch:curated_project evidence, got {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("summarization:mode")),
        "curated GitHub URL fetch should record a summarization mode event, got {:?}",
        response.evidence_links,
    );
}

#[test]
fn http_fetch_of_unknown_url_skips_curated_project_summary() {
    let response = answer("fetch https://example.com");
    assert_eq!(response.intent, "http_fetch");
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("http_fetch:curated_project")),
        "non-curated URL fetch must not log a curated_project event, got {:?}",
        response.evidence_links,
    );
}
