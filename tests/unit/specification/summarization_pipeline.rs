//! Specification tests for the formalize → summarize → deformalize pipeline
//! covered by the curated-project registry work in issue #159.
//!
//! These tests pin down the public surface exposed from `src/summarization.rs`
//! (README ingestion, dialog summarization, chat-title generation) so the
//! pipeline contract is visible alongside the existing project-lookup tests.

use formal_ai::summarization::{
    apply_compound_words, apply_semantic_primes, deformalize, describe_readme, formalize,
    formalize_dialog, formalize_markdown, generate_chat_title, strip_markdown_noise,
    summarize_dialog, summarize_repository_file, DialogTurn, StatementKind, SummarizationConfig,
    SummarizationMode, DEFAULT_MAX_STATEMENTS,
};

#[test]
fn default_max_statements_is_thirty() {
    assert_eq!(
        DEFAULT_MAX_STATEMENTS, 30,
        "default cap on retained statements must match the documented vision (30)",
    );
}

#[test]
fn summarization_mode_target_percent_matches_vision() {
    // Vision (PR #174 comment 2026-05-20T11:25:48Z): topic = 1-5 words,
    // short ~ 20%, standard ~ 50%, full = 100%, expand = 200%.
    assert_eq!(SummarizationMode::Topic.target_percent(), 0);
    assert_eq!(SummarizationMode::Short.target_percent(), 20);
    assert_eq!(SummarizationMode::Standard.target_percent(), 50);
    assert_eq!(SummarizationMode::Full.target_percent(), 100);
    assert_eq!(SummarizationMode::Expand.target_percent(), 200);
}

#[test]
fn strip_markdown_noise_drops_badges_html_comments_and_code_blocks() {
    let markdown = "# Title\n\n[![ci](https://example.com/ci.svg)](https://example.com/ci)\n\n<!-- internal -->\n\nFormal AI is a deterministic symbolic engine.\n\n```bash\nnpm install formal-ai\n```\n\nIt runs offline.";
    let stripped = strip_markdown_noise(markdown);
    assert!(
        !stripped.contains("![ci]"),
        "badge should be dropped, got: {stripped}",
    );
    assert!(
        !stripped.contains("<!--"),
        "HTML comment should be dropped, got: {stripped}",
    );
    assert!(
        !stripped.contains("npm install"),
        "fenced code block should be dropped, got: {stripped}",
    );
    assert!(
        stripped.contains("Formal AI is a deterministic symbolic engine."),
        "prose should be kept, got: {stripped}",
    );
}

#[test]
fn formalize_markdown_classifies_install_and_purpose_lines() {
    let markdown = "Hive Mind is the AI that controls AIs.\n\nInstall with `cargo add hive-mind`.";
    let statements = formalize_markdown(markdown);
    assert!(
        statements
            .iter()
            .any(|stmt| stmt.kind == StatementKind::Purpose),
        "README formalization should detect a purpose statement, got {statements:?}",
    );
    assert!(
        statements
            .iter()
            .any(|stmt| stmt.kind == StatementKind::Install),
        "README formalization should detect the install line, got {statements:?}",
    );
}

#[test]
fn describe_readme_short_keeps_purpose_drops_install() {
    let markdown = "Formal AI is a deterministic symbolic engine.\n\nInstall with `cargo add formal-ai`.\n\nRun `formal-ai --help` for flags.";
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Short)
        .with_language("en");
    let summary = describe_readme("link-assistant/formal-ai", markdown, &config);
    assert!(
        summary.to_lowercase().contains("deterministic")
            || summary.to_lowercase().contains("symbolic engine"),
        "short README summary should keep the purpose, got: {summary}",
    );
    assert!(
        !summary.contains("cargo add"),
        "short README summary should drop install lines by default, got: {summary}",
    );
}

#[test]
fn describe_readme_topic_returns_repo_slug() {
    let markdown = "Formal AI is a deterministic symbolic engine.";
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Topic)
        .with_language("en");
    let topic = describe_readme("link-assistant/formal-ai", markdown, &config);
    assert!(
        !topic.is_empty(),
        "topic-mode README summary should not be empty",
    );
    assert!(
        topic.split_whitespace().count() <= 5,
        "topic-mode README summary must stay within 5 words, got: {topic}",
    );
}

#[test]
fn formalize_dialog_biases_user_turns_above_assistant_turns() {
    let turns = vec![
        DialogTurn::assistant("Hi, how may I help you?"),
        DialogTurn::user("What is 2 + 2?"),
    ];
    let statements = formalize_dialog(&turns);
    let user_stmt = statements
        .iter()
        .find(|s| s.text.contains("2 + 2"))
        .expect("user turn should be formalized");
    let assistant_stmt = statements
        .iter()
        .find(|s| s.text.to_lowercase().contains("how may i help"))
        .expect("assistant turn should be formalized");
    assert!(
        user_stmt.weight > assistant_stmt.weight,
        "user turns should outweigh assistant turns: user={} assistant={}",
        user_stmt.weight,
        assistant_stmt.weight,
    );
}

#[test]
fn summarize_dialog_short_keeps_user_questions() {
    let turns = vec![
        DialogTurn::user("Tell me about Hive Mind."),
        DialogTurn::assistant(
            "Of course! Hive Mind orchestrates many subordinate agents and assigns goals.",
        ),
    ];
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Short)
        .with_language("en");
    let summary = summarize_dialog(&turns, &config);
    assert!(
        summary.to_lowercase().contains("hive mind"),
        "dialog summary should mention the user's topic, got: {summary}",
    );
}

#[test]
fn generate_chat_title_returns_five_or_fewer_words() {
    let turns = vec![
        DialogTurn::user("Tell me about Hive Mind."),
        DialogTurn::assistant("Hive Mind orchestrates subordinate agents."),
    ];
    let title = generate_chat_title(&turns, "en");
    assert!(!title.is_empty(), "chat title should not be empty");
    assert!(
        title.split_whitespace().count() <= 5,
        "chat title should stay within 5 words, got: {title}",
    );
}

#[test]
fn formalize_summarize_deformalize_round_trip_keeps_meaning() {
    let prose = "Formal AI is a symbolic engine. It runs offline. \
                 Install with cargo. Run with cargo run.";
    let statements = formalize(prose);
    assert!(!statements.is_empty(), "formalize should not return empty");
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Full)
        .with_language("en");
    let kept = formal_ai::summarization::summarize(&statements, &config);
    let prose_again = deformalize(&kept);
    assert!(
        prose_again.contains("Formal AI is a symbolic engine"),
        "full-mode round trip should preserve the purpose, got: {prose_again}",
    );
}

#[test]
fn compound_words_and_semantic_primes_are_reversible_by_size() {
    let prose = "the AI orchestrates multiple agents";
    let compressed = apply_compound_words(prose, "en");
    assert!(
        compressed.split_whitespace().count() <= prose.split_whitespace().count(),
        "compound-word compression should not grow the text: {compressed}",
    );
    let expanded = apply_semantic_primes(prose, "en");
    assert!(
        expanded.split_whitespace().count() >= prose.split_whitespace().count(),
        "NSM semantic primes should not shrink the text: {expanded}",
    );
}

#[test]
fn repository_file_summary_recurses_into_markdown_embedded_grammars() {
    let markdown = "# Summarization\n\n\
                    Formal AI summarizes repository files.\n\n\
                    ```rust\n\
                    pub fn summarize_file() -> &'static str { \"ok\" }\n\
                    ```\n\n\
                    ```javascript\n\
                    export function renderSummary() { return 'ok'; }\n\
                    ```\n";
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Standard)
        .with_language("en");
    let summary = summarize_repository_file("docs/example.md", markdown, &config);

    assert!(
        summary.contains("Markdown"),
        "summary should name the repository file format, got: {summary}",
    );
    assert!(
        summary.contains("embedded grammar blocks: rust, javascript"),
        "summary should recurse into fenced Markdown grammars, got: {summary}",
    );
    assert!(
        summary.contains("summarizes repository files"),
        "summary should keep the prose content, got: {summary}",
    );
}
