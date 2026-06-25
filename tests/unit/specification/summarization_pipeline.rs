//! Specification tests for the formalize → summarize → deformalize pipeline
//! covered by the curated-project registry work in issue #159.
//!
//! These tests pin down the public surface exposed from `src/summarization.rs`
//! (README ingestion, dialog summarization, chat-title generation) so the
//! pipeline contract is visible alongside the existing project-lookup tests.

use formal_ai::summarization::{
    apply_compound_words, apply_semantic_primes, deformalize, describe_readme, formalize,
    formalize_dialog, formalize_markdown, formalize_repository_resource, generate_chat_title,
    strip_markdown_noise, summarize_dialog, summarize_repository_file,
    summarize_repository_resource, DialogTurn, RepositoryEntry, RepositoryResourceFormalization,
    StatementKind, SummarizationConfig, SummarizationMode, DEFAULT_MAX_STATEMENTS,
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

// --- issue #563: generalize from files to any repository resource (folders too).

/// A small fixture tree used by the resource/folder summarization tests: a
/// directory holding two files and one nested subdirectory.
fn sample_tree() -> RepositoryEntry {
    RepositoryEntry::directory(
        "src/summarization",
        vec![
            RepositoryEntry::file(
                "src/summarization/mod.rs",
                "//! Summarization pipeline.\npub fn summarize() {}\n",
            ),
            RepositoryEntry::file(
                "src/summarization/file.rs",
                "//! File summary.\npub struct RepositoryFileFormalization;\n\
                 pub fn formalize_repository_file() {}\n",
            ),
            RepositoryEntry::directory(
                "src/summarization/nested",
                vec![RepositoryEntry::file(
                    "src/summarization/nested/readme.md",
                    "# Nested\n\nThis explains nested things.\n\n```rust\npub fn x() {}\n```\n",
                )],
            ),
        ],
    )
}

#[test]
fn summarize_repository_resource_subsumes_file_summarization() {
    // The general entry point must reproduce the specialized file summarizer
    // exactly when handed a file: generalization adds folders without changing
    // the established file behavior.
    let markdown = "# Summarization\n\nFormal AI summarizes repository files.\n";
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Standard)
        .with_language("en");
    let via_file = summarize_repository_file("docs/example.md", markdown, &config);
    let via_resource =
        summarize_repository_resource(&RepositoryEntry::file("docs/example.md", markdown), &config);
    assert_eq!(
        via_file, via_resource,
        "summarize_repository_resource must match summarize_repository_file for file inputs",
    );
}

#[test]
fn directory_topic_summary_is_a_single_identity_sentence() {
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Topic)
        .with_language("en");
    let summary = summarize_repository_resource(&sample_tree(), &config);
    assert!(
        summary.starts_with(
            "src/summarization is a repository directory with 2 files and 1 subdirectory"
        ),
        "topic-mode directory summary should be the aggregate identity sentence, got: {summary}",
    );
    assert!(
        !summary.contains("Contents:"),
        "topic-mode directory summary must not expand children, got: {summary}",
    );
}

#[test]
fn directory_summary_reports_recursive_aggregate_counts() {
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Topic)
        .with_language("en");
    let summary = summarize_repository_resource(&sample_tree(), &config);
    // 2 direct files + 1 nested file = 3 files total; 12 lines total.
    assert!(
        summary.contains("12 lines total across 3 files"),
        "directory identity should aggregate lines and files over the whole subtree, got: {summary}",
    );
}

#[test]
fn directory_short_summary_bounds_listed_children() {
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Short)
        .with_language("en");
    let summary = summarize_repository_resource(&sample_tree(), &config);
    assert!(
        summary.contains("Contents:"),
        "short-mode directory summary should expand into a Contents section, got: {summary}",
    );
    assert!(
        summary.contains("1 more entry omitted for brevity."),
        "short-mode directory summary should cap children and note the remainder, got: {summary}",
    );
}

#[test]
fn directory_summary_recurses_with_one_step_shorter_mode() {
    // A Full directory describes its child *directory* in Standard detail, which
    // in turn expands its own Contents — evidence that recursion depth is bounded
    // by the mode ladder rather than by a fixed depth limit.
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Full)
        .with_language("en");
    let summary = summarize_repository_resource(&sample_tree(), &config);
    assert!(
        summary.contains("src/summarization/nested is a repository directory"),
        "full-mode summary should describe the nested subdirectory, got: {summary}",
    );
    assert!(
        summary.contains("src/summarization/nested/readme.md is a Markdown file"),
        "nested directory should itself be summarized one mode shorter, got: {summary}",
    );
}

#[test]
fn formalize_repository_resource_distinguishes_files_and_directories() {
    let formal = formalize_repository_resource(&sample_tree());
    match formal {
        RepositoryResourceFormalization::Directory(dir) => {
            assert_eq!(dir.direct_file_count, 2);
            assert_eq!(dir.direct_directory_count, 1);
            assert_eq!(dir.total_file_count, 3);
            assert_eq!(dir.total_directory_count, 1);
            assert!(
                dir.children
                    .iter()
                    .any(RepositoryResourceFormalization::is_directory),
                "formalized directory should retain a directory child",
            );
        }
        RepositoryResourceFormalization::File(_) => {
            panic!("a directory entry must formalize to a directory resource")
        }
    }
}

#[test]
fn directory_links_notation_lists_children_by_kind() {
    let formal = formalize_repository_resource(&sample_tree());
    let lino = formal.links_notation();
    assert!(
        lino.starts_with("repository_directory"),
        "directory Links Notation should open a repository_directory block, got: {lino}",
    );
    assert!(
        lino.contains("file src/summarization/mod.rs"),
        "directory Links Notation should list file children, got: {lino}",
    );
    assert!(
        lino.contains("directory src/summarization/nested"),
        "directory Links Notation should list subdirectory children, got: {lino}",
    );
}
