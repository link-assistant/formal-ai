//! Formalize-summarize-deformalize pipeline for project descriptions, README
//! prose, and conversation summaries.
//!
//! The module is intentionally deterministic: every transformation is a pure
//! function of its input plus the [`SummarizationConfig`]. No neural model
//! or external API is consulted. The pipeline has three explicit stages:
//!
//! 1. **Formalize.** Free-form prose (or a curated list of
//!    [`crate::seed::ProjectStatement`]s) is converted into a homogeneous
//!    `Vec<Statement>`. Each statement is one sentence with a coarse `kind`
//!    inferred from cue words (purpose, feature, install, …) and a numeric
//!    `weight` (0–100) that says how important it is.
//! 2. **Summarize.** [`summarize`] applies the configured [`SummarizationMode`]
//!    and `max_statements` limit. Compressing keeps the highest-weighted
//!    statements; expanding *adds* paraphrases generated from the NSM
//!    semantic-prime expansion below.
//! 3. **Deformalize.** [`deformalize`] renders the surviving statements back
//!    into a single block of text suitable for display.
//!
//! The `apply_semantic_primes` and `apply_compound_words` helpers implement
//! the configurable "simplify with semantic primes / shorten with compound
//! words" requirement from PR #174. Both are vocabulary-driven so they can be
//! extended without touching call sites.
//!
//! See `ARCHITECTURE.md` § "Project lookups and summarization" for how the
//! Hive Mind handler chains the three stages together.

use crate::seed::{ProjectRecord, ProjectStatement};

/// Coarse classification used by the summarizer to decide which statements
/// survive a compression pass. Mirrors the `kind "..."` field accepted by
/// `data/seed/projects.lino`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatementKind {
    /// "X is Y" — the bare identity of the project / subject.
    Identity,
    /// Why the project exists / what problem it solves.
    Purpose,
    /// Programming language or runtime.
    Language,
    /// Star count or other social proof.
    Stars,
    /// A concrete capability the project offers.
    Feature,
    /// When the reader should reach for the project.
    UseCase,
    /// Installation / setup instructions.
    Install,
    /// Example invocation, code snippet, command-line usage.
    Example,
    /// Anything else (treated as low-weight by default).
    #[default]
    Misc,
}

impl StatementKind {
    /// Parse a kind label from a seed `kind "..."` field. Unknown labels
    /// fall back to [`StatementKind::Misc`] so the data file remains forward-
    /// compatible with new kinds added in code.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "identity" => Self::Identity,
            "purpose" => Self::Purpose,
            "language" => Self::Language,
            "stars" => Self::Stars,
            "feature" => Self::Feature,
            "use_case" | "usecase" | "use-case" => Self::UseCase,
            "install" => Self::Install,
            "example" => Self::Example,
            _ => Self::Misc,
        }
    }

    /// `true` when the statement carries information that survives the
    /// tightest "what is X?" responses (identity, purpose, language, stars).
    #[must_use]
    pub const fn is_essential(self) -> bool {
        matches!(
            self,
            Self::Identity | Self::Purpose | Self::Language | Self::Stars
        )
    }

    /// `true` when the statement is README boilerplate (install / example)
    /// that should be omitted from compressed answers.
    #[must_use]
    pub const fn is_boilerplate(self) -> bool {
        matches!(self, Self::Install | Self::Example)
    }
}

/// A single normalized statement participating in the summarization pipeline.
#[derive(Debug, Clone)]
pub struct Statement {
    pub text: String,
    pub kind: StatementKind,
    pub weight: u8,
}

impl Statement {
    /// Build a statement from explicit fields.
    #[must_use]
    pub fn new(text: impl Into<String>, kind: StatementKind, weight: u8) -> Self {
        Self {
            text: text.into(),
            kind,
            weight,
        }
    }

    /// Build a statement from a seed [`ProjectStatement`], inferring the
    /// numeric kind from the seed's text label and clamping the weight.
    #[must_use]
    pub fn from_seed(seed: &ProjectStatement) -> Self {
        Self {
            text: seed.text.clone(),
            kind: StatementKind::parse(&seed.kind),
            weight: seed.weight,
        }
    }
}

/// Compression / expansion target for [`summarize`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SummarizationMode {
    /// 1–5 words — return just the project / topic name. Used for chat titles
    /// and topic labels.
    Topic,
    /// ~20% of the source — one or two essential statements.
    Short,
    /// ~50% of the source — keep all essential statements plus the highest-
    /// weighted features.
    #[default]
    Standard,
    /// 100% — every statement, in weight order.
    Full,
    /// ~200% — every statement plus NSM-style paraphrases that expand
    /// compound words into semantic primes.
    Expand,
}

impl SummarizationMode {
    /// Target size relative to the input statement count, expressed as a
    /// percentage. Used by [`SummarizationConfig::effective_max_statements`]
    /// when the caller does not pin an explicit cap. Integer math keeps the
    /// pipeline free of floating-point casts.
    #[must_use]
    pub const fn target_percent(self) -> u32 {
        match self {
            Self::Topic => 0,
            Self::Short => 20,
            Self::Standard => 50,
            Self::Full => 100,
            Self::Expand => 200,
        }
    }
}

/// Configuration for the summarization pipeline. Every knob has a sensible
/// default so the simplest call site can be
/// `summarize(&statements, &SummarizationConfig::default())`.
#[derive(Debug, Clone)]
pub struct SummarizationConfig {
    pub mode: SummarizationMode,
    /// Hard cap on output statements. `None` lets [`SummarizationMode`] pick.
    pub max_statements: Option<usize>,
    /// Language slug (`en` / `ru` / `hi` / `zh`). Drives compound-word and
    /// semantic-prime substitution lists.
    pub language: String,
    /// Replace compound words with shorter compound forms (default `false`).
    /// Useful for chat titles where the result should fit in 1–5 words.
    pub use_compound_words: bool,
    /// Expand compound or rare words into NSM semantic primes when the mode
    /// is `Expand`. Off by default to keep `Topic`/`Short`/`Standard` terse.
    pub use_semantic_primes: bool,
    /// Strip boilerplate kinds (`install`, `example`) from the output.
    /// `true` by default — compressed answers should never carry setup steps.
    pub drop_boilerplate: bool,
}

impl Default for SummarizationConfig {
    fn default() -> Self {
        Self {
            mode: SummarizationMode::Standard,
            max_statements: None,
            language: "en".to_string(),
            use_compound_words: false,
            use_semantic_primes: false,
            drop_boilerplate: true,
        }
    }
}

impl SummarizationConfig {
    /// Builder helper used by Hive Mind handler call sites.
    #[must_use]
    pub const fn with_mode(mut self, mode: SummarizationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Builder helper to pin the language.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Builder helper to clamp the number of statements.
    #[must_use]
    pub const fn with_max_statements(mut self, cap: usize) -> Self {
        self.max_statements = Some(cap);
        self
    }

    /// Effective statement cap for the given input size. Combines
    /// [`SummarizationMode::target_percent`] with the optional explicit cap and
    /// guarantees at least one statement for any non-empty input.
    #[must_use]
    pub fn effective_max_statements(&self, input_count: usize) -> usize {
        if input_count == 0 {
            return 0;
        }
        let ratio_target = match self.mode {
            // Topic mode is rendered separately, but still returns at most 1
            // statement when summarize() is asked to enforce it.
            SummarizationMode::Topic => 1,
            SummarizationMode::Full | SummarizationMode::Expand => input_count,
            other => {
                // Round-to-nearest using only integer math:
                //   suggested = round(input_count * percent / 100)
                let percent = other.target_percent() as usize;
                let suggested = (input_count * percent + 50) / 100;
                suggested.max(1)
            }
        };
        self.max_statements
            .map_or_else(|| ratio_target.max(1), |cap| cap.min(ratio_target).max(1))
    }
}

/// Split a paragraph of free-form text into [`Statement`]s. Each sentence
/// ends at `.`, `!`, `?`, `。`, `…` or a newline. Empty fragments are dropped.
#[must_use]
pub fn formalize(text: &str) -> Vec<Statement> {
    let mut out = Vec::new();
    let mut buffer = String::new();
    for ch in text.chars() {
        buffer.push(ch);
        if matches!(ch, '.' | '!' | '?' | '。' | '…' | '\n') {
            push_sentence(&mut buffer, &mut out);
        }
    }
    push_sentence(&mut buffer, &mut out);
    out
}

fn push_sentence(buffer: &mut String, out: &mut Vec<Statement>) {
    let sentence: String = buffer
        .chars()
        .filter(|c| !matches!(c, '\n'))
        .collect::<String>()
        .trim()
        .to_string();
    buffer.clear();
    if sentence.is_empty() {
        return;
    }
    let kind = classify_sentence(&sentence);
    let weight = weight_for_kind(kind);
    out.push(Statement::new(sentence, kind, weight));
}

/// Heuristic classifier for prose sentences. Looks for cue words that signal
/// each kind. The cue lists are intentionally short — the seed registry is
/// the long-term source of truth.
#[must_use]
pub fn classify_sentence(sentence: &str) -> StatementKind {
    let lower = sentence.to_lowercase();
    if contains_any(
        &lower,
        &[
            "to install",
            "install with",
            "installation",
            "npm install",
            "cargo install",
            "pip install",
        ],
    ) {
        return StatementKind::Install;
    }
    if contains_any(
        &lower,
        &[
            "for example",
            "example:",
            "e.g.",
            "run --",
            "$ ",
            "```",
            "выполни",
        ],
    ) {
        return StatementKind::Example;
    }
    if contains_any(
        &lower,
        &[
            "written in ",
            "language:",
            "is a ",
            "написан на ",
            "на языке ",
            "build with ",
        ],
    ) && lower.split_whitespace().count() <= 12
    {
        return StatementKind::Language;
    }
    if contains_any(
        &lower,
        &[
            " stars",
            "github stars",
            "★",
            "stargazers",
            "звёзды",
            "звезд",
        ],
    ) {
        return StatementKind::Stars;
    }
    if contains_any(
        &lower,
        &[
            "is for ",
            "is the ai",
            "is an ai",
            "is used to",
            "is meant to",
            "is designed to",
            "helps you",
            "lets you",
            "это ии",
            "предназнач",
        ],
    ) {
        return StatementKind::Purpose;
    }
    if contains_any(
        &lower,
        &[
            "use it when",
            "use this when",
            "when you need",
            "ideal for",
            "useful when",
        ],
    ) {
        return StatementKind::UseCase;
    }
    if contains_any(
        &lower,
        &[
            " supports ",
            " provides ",
            " offers ",
            " exposes ",
            " ships ",
            " предоставляет ",
            " поддерживает ",
            " orchestrates ",
        ],
    ) {
        return StatementKind::Feature;
    }
    StatementKind::Misc
}

const fn weight_for_kind(kind: StatementKind) -> u8 {
    match kind {
        StatementKind::Purpose => 100,
        StatementKind::Identity => 90,
        StatementKind::Language => 60,
        StatementKind::Stars => 55,
        StatementKind::Feature => 70,
        StatementKind::UseCase => 65,
        StatementKind::Install => 10,
        StatementKind::Example => 15,
        StatementKind::Misc => 30,
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles
        .iter()
        .any(|needle| !needle.is_empty() && haystack.contains(needle))
}

/// Apply [`SummarizationConfig`] to a slice of statements.
///
/// Returns a new vector ordered by weight (descending), capped at the effective
/// max. Boilerplate is stripped before ranking when `drop_boilerplate` is set,
/// and `Expand` mode appends NSM paraphrases for the surviving statements.
#[must_use]
pub fn summarize(statements: &[Statement], config: &SummarizationConfig) -> Vec<Statement> {
    if statements.is_empty() {
        return Vec::new();
    }
    let mut filtered: Vec<Statement> = statements
        .iter()
        .filter(|s| !(config.drop_boilerplate && s.kind.is_boilerplate()))
        .cloned()
        .collect();
    filtered.sort_by_key(|stmt| core::cmp::Reverse(stmt.weight));
    let cap = config.effective_max_statements(filtered.len());
    filtered.truncate(cap);

    if config.mode == SummarizationMode::Expand {
        // Double the surviving set with NSM paraphrases so the result lands
        // near the requested ~200% target ratio.
        let mut expanded: Vec<Statement> = Vec::with_capacity(filtered.len() * 2);
        for stmt in &filtered {
            expanded.push(stmt.clone());
            if config.use_semantic_primes {
                let mut paraphrase = stmt.clone();
                paraphrase.text = apply_semantic_primes(&stmt.text, &config.language);
                paraphrase.weight = stmt.weight.saturating_sub(5);
                if paraphrase.text != stmt.text {
                    expanded.push(paraphrase);
                }
            }
        }
        return expanded;
    }

    if config.use_compound_words {
        for stmt in &mut filtered {
            stmt.text = apply_compound_words(&stmt.text, &config.language);
        }
    }

    filtered
}

/// Render a slice of statements as a single block of text. Statements are
/// joined with single spaces (after re-punctuation) so the result reads as
/// continuous prose.
#[must_use]
pub fn deformalize(statements: &[Statement]) -> String {
    statements
        .iter()
        .map(|s| {
            let trimmed = s.text.trim();
            if trimmed.is_empty() {
                String::new()
            } else if ends_with_terminal_punct(trimmed) {
                trimmed.to_string()
            } else {
                format!("{trimmed}.")
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn ends_with_terminal_punct(text: &str) -> bool {
    text.chars()
        .last()
        .is_some_and(|c| matches!(c, '.' | '!' | '?' | '。' | '…' | '」' | '"'))
}

/// Render the topic label (1–5 words) for the supplied statements.
///
/// When `explicit_topic` is non-empty (e.g. `project.topic`) it is returned
/// verbatim. Otherwise the first content noun of the highest-weight
/// statement is used.
#[must_use]
pub fn to_topic(explicit_topic: &str, statements: &[Statement]) -> String {
    let candidate = explicit_topic.trim();
    if !candidate.is_empty() {
        return clamp_words(candidate, 5);
    }
    statements
        .iter()
        .max_by_key(|s| s.weight)
        .map(|s| clamp_words(&s.text, 5))
        .unwrap_or_default()
}

fn clamp_words(text: &str, max_words: usize) -> String {
    text.split_whitespace()
        .take(max_words)
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches(['.', ',', '!', '?', ';', ':', '…', '」', '"'])
        .to_string()
}

/// Substitute a few common compound forms with shorter equivalents.
/// Vocabulary is intentionally tiny; extending it is a single-line addition.
#[must_use]
pub fn apply_compound_words(text: &str, language: &str) -> String {
    let pairs: &[(&str, &str)] = match language {
        "ru" => &[
            ("в которой ", "где "),
            ("для того чтобы ", "чтобы "),
            ("к примеру", "например"),
        ],
        _ => &[
            ("in order to ", "to "),
            ("for the purpose of ", "for "),
            ("a number of ", "several "),
            ("user interface", "UI"),
            ("command line interface", "CLI"),
            ("artificial intelligence", "AI"),
        ],
    };
    let mut out = text.to_string();
    for (long, short) in pairs {
        out = out.replace(long, short);
    }
    out
}

/// Substitute compound or rare words with NSM semantic primes.
///
/// See <https://en.wikipedia.org/wiki/Natural_semantic_metalanguage>. This is a
/// best-effort heuristic — the vocabulary is short and additive, so callers
/// always see *some* simplification even when the prime is only an
/// approximation.
#[must_use]
pub fn apply_semantic_primes(text: &str, language: &str) -> String {
    let pairs: &[(&str, &str)] = match language {
        "ru" => &[
            ("автоматизация", "когда машина делает"),
            ("оркестрирует", "управляет вместе"),
            ("делегирование", "передача работы"),
            ("детерминированный", "всегда одинаковый"),
        ],
        _ => &[
            ("orchestrates", "controls many"),
            (
                "automation of automation",
                "machine that makes other machines do",
            ),
            ("automation", "machine doing"),
            ("delegating", "giving work to"),
            ("deterministic", "always the same"),
            ("multilingual", "in many languages"),
            ("symbolic", "rule-based"),
        ],
    };
    let mut out = text.to_string();
    for (compound, prime) in pairs {
        out = out.replace(compound, prime);
    }
    out
}

/// Build a description from the curated project record.
///
/// Centralizes the "look up project → pick statements for language →
/// summarize → deformalize" pipeline so callers can request `Topic` / `Short`
/// / `Standard` / `Full` / `Expand` length with one call.
#[must_use]
pub fn describe_project(project: &ProjectRecord, config: &SummarizationConfig) -> String {
    let seed_statements = project.statements_for(&config.language);
    let statements: Vec<Statement> = seed_statements.iter().map(Statement::from_seed).collect();
    if config.mode == SummarizationMode::Topic {
        return to_topic(project.topic_for(&config.language), &statements);
    }
    let summarized = summarize(&statements, config);
    deformalize(&summarized)
}

#[cfg(test)]
mod tests {
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
}
