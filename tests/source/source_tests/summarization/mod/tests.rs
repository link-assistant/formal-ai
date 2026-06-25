use super::*;

fn sample_statements() -> Vec<Statement> {
    vec![
        Statement::new(
            "X is the AI that controls AIs.",
            StatementKind::Purpose,
            100,
        ),
        Statement::new("X is written in JavaScript.", StatementKind::Language, 60),
        Statement::new(
            "X orchestrates multiple agents.",
            StatementKind::Feature,
            70,
        ),
        Statement::new("Install X with npm install x.", StatementKind::Install, 10),
        Statement::new("Run x --help for flags.", StatementKind::Example, 10),
    ]
}

#[test]
fn formalize_splits_on_punctuation() {
    let stmts = formalize("Foo is bar. Foo helps you ship! What is foo?");
    assert_eq!(stmts.len(), 3);
    assert!(stmts[0].text.ends_with('.'));
    assert!(stmts[1].text.ends_with('!'));
    assert!(stmts[2].text.ends_with('?'));
}

#[test]
fn classify_picks_install_for_npm_install() {
    assert_eq!(
        classify_sentence("Install foo with npm install foo."),
        StatementKind::Install
    );
}

#[test]
fn classify_scans_summary_cue_meanings_in_declaration_order() {
    // The classifier no longer hardcodes cue arrays: it walks the seven
    // `summary_kind_*` leaves carrying ROLE_SUMMARY_CLASSIFICATION_CUE in
    // declaration order, which must equal the original priority order.
    let slugs: Vec<&str> = crate::seed::lexicon()
        .meanings_with_role(crate::seed::ROLE_SUMMARY_CLASSIFICATION_CUE)
        .map(|m| m.slug.as_str())
        .collect();
    assert_eq!(
        slugs,
        vec![
            "summary_kind_install",
            "summary_kind_example",
            "summary_kind_language",
            "summary_kind_stars",
            "summary_kind_purpose",
            "summary_kind_use_case",
            "summary_kind_feature",
        ],
        "summary cue meanings must scan in the original classify priority order"
    );
    // from_slug resolves every leaf back to its kind (and is forward-safe).
    for (slug, kind) in [
        ("summary_kind_install", StatementKind::Install),
        ("summary_kind_example", StatementKind::Example),
        ("summary_kind_language", StatementKind::Language),
        ("summary_kind_stars", StatementKind::Stars),
        ("summary_kind_purpose", StatementKind::Purpose),
        ("summary_kind_use_case", StatementKind::UseCase),
        ("summary_kind_feature", StatementKind::Feature),
    ] {
        assert_eq!(StatementKind::from_slug(slug), kind, "from_slug({slug})");
    }
    assert_eq!(
        StatementKind::from_slug("not_a_summary_slug"),
        StatementKind::Misc
    );
}

#[test]
fn classify_recognizes_each_kind_through_seed_surfaces() {
    // Each kind is locked to a surface fragment carried by the seed, so an
    // accidental edit to data/seed/meanings-summary.lino fails this test.
    for (sentence, kind) in [
        ("To install, run the setup script.", StatementKind::Install),
        (
            "For example, see the snippet below.",
            StatementKind::Example,
        ),
        ("Written in Rust.", StatementKind::Language),
        ("It has 5000 github stars.", StatementKind::Stars),
        ("It helps you ship faster.", StatementKind::Purpose),
        (
            "Use it when you deploy to production.",
            StatementKind::UseCase,
        ),
        ("It supports plugins.", StatementKind::Feature),
        ("Just a plain sentence about nothing.", StatementKind::Misc),
    ] {
        assert_eq!(classify_sentence(sentence), kind, "classify({sentence})");
    }
}

#[test]
fn classify_language_guard_falls_through_on_long_sentences() {
    // A short `is a …` sentence is a language line; a long one that merely
    // contains the same cue keeps scanning so a later feature cue claims it
    // (the original `&& word_count <= 12` guard, now applied structurally).
    assert_eq!(classify_sentence("X is a tool."), StatementKind::Language);
    let long = "This project is a comprehensive toolkit that supports plugins \
                    and provides many helpful features for everyone.";
    assert!(long.split_whitespace().count() > 12);
    assert_eq!(classify_sentence(long), StatementKind::Feature);
}

#[test]
fn summarize_short_keeps_highest_weight() {
    let config = SummarizationConfig::default().with_mode(SummarizationMode::Short);
    let out = summarize(&sample_statements(), &config);
    assert!(!out.is_empty());
    assert_eq!(out[0].kind, StatementKind::Purpose);
    // Short mode + 3 retained statements after dropping boilerplate ⇒
    // effective_max_statements = max(1, round(3*0.2)) = 1.
    assert_eq!(out.len(), 1);
}

#[test]
fn summarize_drops_install_and_example_by_default() {
    let config = SummarizationConfig::default().with_mode(SummarizationMode::Full);
    let out = summarize(&sample_statements(), &config);
    assert!(out.iter().all(|s| s.kind != StatementKind::Install));
    assert!(out.iter().all(|s| s.kind != StatementKind::Example));
    assert_eq!(out.len(), 3);
}

#[test]
fn summarize_full_keeps_install_when_drop_boilerplate_false() {
    let mut config = SummarizationConfig::default().with_mode(SummarizationMode::Full);
    config.drop_boilerplate = false;
    let out = summarize(&sample_statements(), &config);
    assert!(out.iter().any(|s| s.kind == StatementKind::Install));
    assert!(out.iter().any(|s| s.kind == StatementKind::Example));
}

#[test]
fn summarize_expand_with_primes_grows_output() {
    let mut config = SummarizationConfig::default().with_mode(SummarizationMode::Expand);
    config.use_semantic_primes = true;
    let stmts = vec![Statement::new(
        "X orchestrates multiple agents.",
        StatementKind::Feature,
        70,
    )];
    let out = summarize(&stmts, &config);
    assert!(out.len() >= 2);
    assert!(out.iter().any(|s| s.text.contains("controls many")));
}

#[test]
fn summarize_topic_returns_at_most_one_statement() {
    let config = SummarizationConfig::default().with_mode(SummarizationMode::Topic);
    let out = summarize(&sample_statements(), &config);
    assert!(out.len() <= 1);
}

#[test]
fn deformalize_joins_statements_with_period_terminator() {
    let stmts = vec![
        Statement::new("Hello world", StatementKind::Identity, 100),
        Statement::new("Foo bars", StatementKind::Misc, 50),
    ];
    let rendered = deformalize(&stmts);
    assert_eq!(rendered, "Hello world. Foo bars.");
}

#[test]
fn to_topic_clamps_to_five_words() {
    let topic = to_topic("", &sample_statements());
    assert!(topic.split_whitespace().count() <= 5);
}

#[test]
fn to_topic_uses_explicit_topic_when_present() {
    let topic = to_topic("Hive Mind", &[]);
    assert_eq!(topic, "Hive Mind");
}

#[test]
fn apply_compound_words_shortens_english_phrases() {
    let result = apply_compound_words("Run in order to ship the user interface.", "en");
    assert!(result.contains("to ship"));
    assert!(result.contains("UI"));
}

#[test]
fn apply_semantic_primes_expands_orchestrates() {
    let result = apply_semantic_primes("X orchestrates agents.", "en");
    assert!(result.contains("controls many"));
}

#[test]
fn apply_semantic_primes_supports_russian() {
    let result = apply_semantic_primes("X автоматизация всего.", "ru");
    assert!(result.contains("когда машина делает"));
}

#[test]
fn describe_project_topic_returns_topic_label() {
    let registry = crate::seed::projects_registry();
    let hive = registry.by_alias("Hive Mind").expect("hive-mind present");
    let topic = describe_project(
        hive,
        &SummarizationConfig::default().with_mode(SummarizationMode::Topic),
    );
    assert_eq!(topic, "Hive Mind");
}

#[test]
fn describe_project_short_returns_purpose_statement() {
    let registry = crate::seed::projects_registry();
    let hive = registry.by_alias("Hive Mind").expect("hive-mind present");
    let description = describe_project(
        hive,
        &SummarizationConfig::default().with_mode(SummarizationMode::Short),
    );
    assert!(
        description.contains("AI"),
        "expected description to mention AI, got: {description}"
    );
    // Short mode drops boilerplate: install/example phrases must be absent.
    assert!(!description.to_lowercase().contains("npm install"));
}

#[test]
fn describe_project_russian_uses_localized_statements() {
    let registry = crate::seed::projects_registry();
    let hive = registry.by_alias("Hive Mind").expect("hive-mind present");
    let description = describe_project(
        hive,
        &SummarizationConfig::default()
            .with_mode(SummarizationMode::Short)
            .with_language("ru"),
    );
    assert!(
        description.contains("ИИ"),
        "expected Russian description to contain ИИ, got: {description}"
    );
}

#[test]
fn effective_max_statements_clamps_explicit_cap() {
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Standard)
        .with_max_statements(2);
    assert_eq!(config.effective_max_statements(10), 2);
    assert_eq!(config.effective_max_statements(0), 0);
}

#[test]
fn effective_max_statements_topic_returns_one() {
    let config = SummarizationConfig::default().with_mode(SummarizationMode::Topic);
    assert_eq!(config.effective_max_statements(10), 1);
    assert_eq!(config.effective_max_statements(0), 0);
}

#[test]
fn strip_markdown_noise_drops_headings_badges_and_code_blocks() {
    let md = "# Title\n\
                  \n\
                  [![ci](https://example.com/ci.svg)](https://example.com/ci)\n\
                  \n\
                  Hive Mind is the AI that controls AIs.\n\
                  \n\
                  ```bash\n\
                  npm install hive-mind\n\
                  ```\n\
                  \n\
                  > It orchestrates multiple agents.\n";
    let stripped = strip_markdown_noise(md);
    let lower = stripped.to_lowercase();
    assert!(lower.contains("hive mind"));
    assert!(lower.contains("orchestrates multiple agents"));
    assert!(
        !lower.contains("npm install"),
        "fenced code block should be dropped, got: {stripped}",
    );
    assert!(
        !lower.contains("[![ci]"),
        "badge line should be dropped, got: {stripped}",
    );
}

#[test]
fn strip_markdown_noise_keeps_heading_text_when_no_marker() {
    let md = "## Section\n\nBody sentence one. Body sentence two.";
    let stripped = strip_markdown_noise(md);
    assert!(stripped.contains("Section"));
    assert!(stripped.contains("Body sentence one"));
}

#[test]
fn strip_markdown_noise_skips_html_comments() {
    let md = "<!-- internal note -->\n\nReal sentence.";
    let stripped = strip_markdown_noise(md);
    assert!(!stripped.contains("internal note"));
    assert!(stripped.contains("Real sentence"));
}

#[test]
fn formalize_markdown_yields_statements_for_readme_prose() {
    let md = "# hive-mind\n\nHive Mind orchestrates multiple agents.\n\
                  Install with npm install hive-mind.\n";
    let stmts = formalize_markdown(md);
    assert!(!stmts.is_empty());
    assert!(stmts
        .iter()
        .any(|s| s.text.to_lowercase().contains("orchestrates")));
    assert!(stmts.iter().any(|s| s.kind == StatementKind::Install));
}

#[test]
fn describe_readme_strips_boilerplate_and_keeps_purpose() {
    let readme = "# command-stream\n\n\
                      ![ci](https://example.com/ci.svg)\n\n\
                      command-stream is the streaming shell helper used by hive-mind.\n\
                      Install with npm install command-stream.\n\
                      \n\
                      ```bash\n\
                      npm run example\n\
                      ```\n";
    let description = describe_readme(
        "link-foundation/command-stream",
        readme,
        &SummarizationConfig::default().with_mode(SummarizationMode::Short),
    );
    assert!(
        !description.to_lowercase().contains("npm install"),
        "Short mode should drop install boilerplate, got: {description}",
    );
    assert!(description.to_lowercase().contains("command-stream"));
}

#[test]
fn describe_readme_topic_mode_returns_repo_slug() {
    let topic = describe_readme(
        "link-assistant/hive-mind",
        "# Anything\n",
        &SummarizationConfig::default().with_mode(SummarizationMode::Topic),
    );
    assert!(topic.split_whitespace().count() <= 5);
    assert!(topic.contains("link-assistant/hive-mind") || topic.contains("hive-mind"));
}

#[test]
fn formalize_repository_file_markdown_records_embedded_grammars() {
    let markdown = "# File summary\n\n\
                    Formal AI is designed to summarize repository files.\n\n\
                    ```rust\n\
                    pub fn summarize_file() -> &'static str { \"ok\" }\n\
                    ```\n\n\
                    ```javascript\n\
                    export function renderSummary() { return 'ok'; }\n\
                    ```\n";
    let formalized = formalize_repository_file("docs/file-summary.md", markdown);
    assert_eq!(formalized.format, "markdown");
    assert_eq!(formalized.embedded_grammars.len(), 2);
    assert_eq!(formalized.embedded_grammars[0].language, "rust");
    assert_eq!(formalized.embedded_grammars[1].language, "javascript");
    let lino = formalized.links_notation();
    assert!(lino.contains("repository_file"));
    assert!(lino.contains("embedded_grammar"));
    assert!(lino.contains("language rust"));
    assert!(lino.contains("language javascript"));
}

#[test]
fn formalize_repository_file_markdown_closes_embedded_grammar_at_eof() {
    let markdown = "# File summary\n\n\
                    ```rust\n\
                    pub struct FileSummary;\n";
    let formalized = formalize_repository_file("docs/file-summary.md", markdown);
    assert_eq!(formalized.embedded_grammars.len(), 1);
    assert_eq!(formalized.embedded_grammars[0].language, "rust");
}

#[test]
fn formalize_repository_file_rust_records_meta_language_and_symbols() {
    let source = "pub struct FileSummary;\n\n\
                  pub fn summarize_file() -> &'static str {\n\
                  \"ok\"\n\
                  }\n";
    let formalized = formalize_repository_file("src/file_summary.rs", source);
    assert_eq!(formalized.format, "rust");
    assert!(formalized
        .statements
        .iter()
        .any(|statement| statement.text.contains("rust struct FileSummary")));
    assert!(formalized
        .statements
        .iter()
        .any(|statement| statement.text.contains("rust function summarize_file")));
    let meta = formalized
        .meta_language
        .as_ref()
        .expect("Rust files should be parsed through meta-language");
    assert_eq!(meta.label, "rust");
    assert!(meta.syntax_link_count > 0);
    assert!(meta.text_preserved);
}

#[test]
fn summarize_dialog_keeps_user_question_over_assistant_chatter() {
    let turns = vec![
        DialogTurn::user("What is Hive Mind?"),
        DialogTurn::assistant("Hive Mind is an AI orchestrator."),
        DialogTurn::user("How do I install it?"),
    ];
    let summary = summarize_dialog(
        &turns,
        &SummarizationConfig::default()
            .with_mode(SummarizationMode::Short)
            .with_max_statements(2),
    );
    let lower = summary.to_lowercase();
    assert!(
        lower.contains("hive mind") || lower.contains("install"),
        "expected dialog summary to surface user questions, got: {summary}",
    );
}

#[test]
fn generate_chat_title_returns_up_to_five_words() {
    let turns = vec![DialogTurn::user("What is the Hive Mind project about?")];
    let title = generate_chat_title(&turns, "en");
    assert!(!title.is_empty());
    assert!(
        title.split_whitespace().count() <= 5,
        "chat title must be at most 5 words, got: {title}",
    );
}

#[test]
fn default_max_statements_constant_is_thirty() {
    assert_eq!(DEFAULT_MAX_STATEMENTS, 30);
    // Smoke test: applying the documented cap to a long input does not
    // exceed it.
    let stmts: Vec<Statement> = (0..50)
        .map(|i| Statement::new(format!("Sentence {i}."), StatementKind::Feature, 50))
        .collect();
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Full)
        .with_max_statements(DEFAULT_MAX_STATEMENTS);
    let out = summarize(&stmts, &config);
    assert_eq!(out.len(), DEFAULT_MAX_STATEMENTS);
}
