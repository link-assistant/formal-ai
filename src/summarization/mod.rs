//! Formalize-summarize-deformalize pipeline for project descriptions, README
//! prose, conversation summaries, and chat titles.
//!
//! The module is intentionally deterministic: every transformation is a pure
//! function of its input plus the [`SummarizationConfig`]. No neural model
//! or external API is consulted. The pipeline has three explicit stages:
//!
//! 1. **Formalize.** Free-form prose, Markdown README content, dialog turns,
//!    or a curated list of [`crate::seed::ProjectStatement`]s is converted
//!    into a homogeneous `Vec<Statement>`. Each statement is one sentence
//!    with a coarse `kind` inferred from cue words (purpose, feature,
//!    install, …) and a numeric `weight` (0–100) that says how important it
//!    is.
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
//! Higher-level helpers chain the three stages together for the most common
//! callers:
//!
//! - [`describe_project`] — curated GitHub project → language-aware
//!   description.
//! - [`describe_readme`] — Markdown README text → language-aware description
//!   (badges, headings, and fenced code blocks are stripped before
//!   formalization).
//! - [`summarize_dialog`] — chat turns → short recap of the conversation.
//! - [`generate_chat_title`] — chat turns → 1–5 word chat title.
//! - [`summarize_repository_file`] — repository path + file content → file
//!   metadata, optional meta-language evidence, embedded Markdown grammars, and
//!   content summary.
//! - [`summarize_repository_resource`] — a [`RepositoryEntry`] tree (a file
//!   **or** a directory/folder of arbitrary depth) → a recursively composed
//!   summary. Directories decompose into children, summarize each child
//!   (recursing into subdirectories), and compose the child summaries behind an
//!   aggregate identity sentence, bounding depth via the mode ladder. This is
//!   the general entry point that subsumes [`summarize_repository_file`].
//!
//! See `ARCHITECTURE.md` § "Project lookups and summarization" for how
//! `project_lookup` chains the three stages together.

use crate::seed::{ProjectRecord, ProjectStatement};

/// Default cap on the number of retained statements per summary.
///
/// Applied when the caller does not supply an explicit `max_statements`
/// value. Mirrors the vision note in PR #174: "for example not more than
/// 30 statements (it should be configurable also)". Set via
/// [`SummarizationConfig::default`] callers that opt into the cap.
pub const DEFAULT_MAX_STATEMENTS: usize = 30;

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

    /// Map the slug of a `summary_classification_cue` meaning to its kind. The
    /// seven `summary_kind_*` leaves in `data/seed/meanings-summary.lino` carry
    /// the cue surfaces that [`classify_sentence`] scans; this resolves the
    /// meaning that matched back into a [`StatementKind`]. Unknown slugs fall
    /// back to [`StatementKind::Misc`] so the seed stays forward-compatible.
    #[must_use]
    pub fn from_slug(slug: &str) -> Self {
        match slug {
            "summary_kind_install" => Self::Install,
            "summary_kind_example" => Self::Example,
            "summary_kind_language" => Self::Language,
            "summary_kind_stars" => Self::Stars,
            "summary_kind_purpose" => Self::Purpose,
            "summary_kind_use_case" => Self::UseCase,
            "summary_kind_feature" => Self::Feature,
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
    /// One token — a legal identifier (function name, variable name, commit
    /// subject). The bottom rung of the ladder, one step shorter than
    /// [`Self::Topic`]; rendered by [`identifier::to_identifier`] under a length
    /// budget and a naming convention.
    Identifier,
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
            // Both label rungs are below the statement scale: they render a name,
            // not a percentage of the input.
            Self::Identifier | Self::Topic => 0,
            Self::Short => 20,
            Self::Standard => 50,
            Self::Full => 100,
            Self::Expand => 200,
        }
    }

    /// The next-shorter mode on the detail ladder, used to bound recursion when
    /// composing nested summaries (a directory describes its children one mode
    /// shorter than itself). `Identifier` is the fixed point: a single token
    /// cannot get shorter.
    #[must_use]
    pub const fn one_step_shorter(self) -> Self {
        match self {
            Self::Expand | Self::Full => Self::Standard,
            Self::Standard => Self::Short,
            Self::Short => Self::Topic,
            Self::Topic | Self::Identifier => Self::Identifier,
        }
    }

    /// Do the rungs below the statement scale — the ones that render a label
    /// rather than a body of prose?
    ///
    /// Call sites that special-cased `== Topic` before #844 must use this, or an
    /// `Identifier` request would fall through to the prose path and return a
    /// sentence where a name was asked for.
    #[must_use]
    pub const fn is_label_only(self) -> bool {
        matches!(self, Self::Topic | Self::Identifier)
    }
}

/// Render the label for a label-only rung ([`SummarizationMode::is_label_only`]).
///
/// `label` is the topic text the caller already computed (via [`to_topic`] or an
/// identity sentence). In [`SummarizationMode::Topic`] it is returned unchanged;
/// in [`SummarizationMode::Identifier`] it is shortened one more rung into a
/// legal `snake_case` name under the default budget. Callers needing another
/// convention or budget call [`identifier::to_identifier`] directly.
#[must_use]
pub fn label_for_mode(mode: SummarizationMode, label: &str) -> String {
    if mode == SummarizationMode::Identifier {
        return identifier::to_identifier(
            label,
            identifier::NamingConvention::SnakeCase,
            &identifier::IdentifierBudget::default(),
        );
    }
    label.to_string()
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
    /// Builder helper used by project lookup call sites.
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

    /// Builder helper to keep the boilerplate kinds (`install`, `example`) in
    /// the output.
    ///
    /// The default drops them because a compressed project summary should not
    /// carry setup steps. A merged multi-source context is the case that needs
    /// the opposite: when the sources are a question and its answers, the
    /// install command *is* the answer, and the evidence weight of
    /// [`importance::score`] — not the sentence's kind — is what decides its
    /// importance.
    #[must_use]
    pub const fn keeping_boilerplate(mut self) -> Self {
        self.drop_boilerplate = false;
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
            // The label rungs are rendered separately, but still return at most 1
            // statement when summarize() is asked to enforce it.
            SummarizationMode::Identifier | SummarizationMode::Topic => 1,
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
/// ends at `.`, `!`, `?`, `。`, `…`, the Devanagari danda `।` or double danda
/// `॥`, or a newline. Empty fragments are dropped.
///
/// The dandas matter because Hindi prose ends its sentences with them and not
/// with a full stop. Without them a Hindi page is one enormous statement, so
/// anything that ranks or trims sentences — the web-research extract of issue
/// #771, for one — degrades to returning the whole document.
///
/// A full stop glued to the next character ends nothing: `crates.io`, `docs.rs`
/// and `1.96` are single tokens, and splitting inside them turns one fact into
/// two fragments ("the crate is published on crates" and "io"), which a
/// multi-source merge then ranks and presents as separate facts. Only a full
/// stop followed by whitespace, punctuation or the end of the text closes a
/// sentence; the other terminators stay eager, because none of them appears
/// inside a word.
#[must_use]
pub fn formalize(text: &str) -> Vec<Statement> {
    let mut out = Vec::new();
    let mut buffer = String::new();
    let mut characters = text.chars().peekable();
    while let Some(ch) = characters.next() {
        let internal_period =
            ch == '.' && period_belongs_to_token(&buffer, characters.peek().copied());
        buffer.push(ch);
        if !internal_period && matches!(ch, '.' | '!' | '?' | '。' | '…' | '।' | '॥' | '\n')
        {
            push_sentence(&mut buffer, &mut out);
        }
    }
    push_sentence(&mut buffer, &mut out);
    out
}

/// Whether a full stop is part of the token being written, rather than a
/// sentence boundary.
///
/// This covers decimals, dotted names and abbreviations, and domain-like
/// tokens. In particular, both stops in `U.S.` stay with the following words:
/// the first joins adjacent letters, while the second closes an initialism.
fn period_belongs_to_token(buffer: &str, next: Option<char>) -> bool {
    let previous = buffer.chars().next_back();
    if previous.is_some_and(char::is_alphanumeric) && next.is_some_and(char::is_alphanumeric) {
        return true;
    }

    let token = buffer
        .split_whitespace()
        .next_back()
        .unwrap_or_default()
        .trim_matches(|character: char| !character.is_alphabetic() && character != '.');
    let mut segments = token.split('.');
    let first = segments.next().unwrap_or_default();
    if first.chars().count() != 1 || !first.chars().all(char::is_alphabetic) {
        return false;
    }
    let mut segment_count = 1;
    for segment in segments {
        if segment.chars().count() != 1 || !segment.chars().all(char::is_alphabetic) {
            return false;
        }
        segment_count += 1;
    }
    segment_count >= 2
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

/// Heuristic classifier for prose sentences.
///
/// Reasons over the seed registry rather than any hardcoded cue list: it walks
/// the meanings carrying [`crate::seed::ROLE_SUMMARY_CLASSIFICATION_CUE`] in
/// declaration order and returns the kind of the first meaning whose surface
/// fragments occur in the lowercased sentence as a raw substring. The `language`
/// kind is additionally length-guarded — a sentence that merely contains a
/// language cue but runs past twelve whitespace words is not a language line, so
/// it falls through to the later kinds. Every cue fragment, in every supported
/// language, lives in `data/seed/meanings-summary.lino`; nothing is hardcoded here.
#[must_use]
pub fn classify_sentence(sentence: &str) -> StatementKind {
    let lower = sentence.to_lowercase();
    let word_count = lower.split_whitespace().count();
    for meaning in
        crate::seed::lexicon().meanings_with_role(crate::seed::ROLE_SUMMARY_CLASSIFICATION_CUE)
    {
        if !meaning.words().any(|cue| lower.contains(cue)) {
            continue;
        }
        let kind = StatementKind::from_slug(&meaning.slug);
        // The `language` kind only applies to short identity-style sentences; a
        // long sentence that merely contains `is a …` keeps scanning so a later
        // feature/purpose cue can claim it (preserving the original
        // `&& word_count <= 12` guard that sat on the language arm).
        if kind == StatementKind::Language && word_count > 12 {
            continue;
        }
        return kind;
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
        .is_some_and(|c| matches!(c, '.' | '!' | '?' | '。' | '…' | '।' | '॥' | '」' | '"'))
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
    if config.mode.is_label_only() {
        let topic = to_topic(project.topic_for(&config.language), &statements);
        return label_for_mode(config.mode, &topic);
    }
    let summarized = summarize(&statements, config);
    deformalize(&summarized)
}

pub mod context;
pub mod dedup;
mod dialog;
mod file;
pub mod gathering;
pub mod identifier;
pub mod importance;
mod markdown;
pub mod recheck;
mod resource;
pub mod vocabulary;

pub use context::{merge_into_context, MergedContext};
pub use dedup::{
    deduplicate, Contradiction, DedupReport, MergeLink, MergedStatement, Polarity,
    SourcedStatement, StatementSignature, StatementVariant,
};
pub use dialog::{formalize_dialog, generate_chat_title, summarize_dialog, DialogTurn};
pub use file::{
    formalize_repository_file, summarize_repository_file, EmbeddedGrammarFormalization,
    MetaLanguageFormalization, RepositoryFileFormalization,
};
pub use gathering::{
    gather, FetchRecord, FetchedSource, GatheringPlan, GatheringReport, SourceCache, SourceProvider,
};
pub use identifier::{
    is_valid_identifier, to_identifier, IdentifierBudget, NamingConvention,
    DEFAULT_IDENTIFIER_MAX_LENGTH, DEFAULT_IDENTIFIER_MAX_WORDS,
};
pub use importance::{rank, to_statements_in, ImportanceScore, RankedStatement};
pub use markdown::{describe_readme, formalize_markdown, strip_markdown_noise};
pub use recheck::{recheck, RecheckReport, RecheckedStatement, Verdict};
pub use resource::{
    formalize_repository_directory, formalize_repository_resource, summarize_repository_resource,
    RepositoryDirectoryFormalization, RepositoryEntry, RepositoryResourceFormalization,
};
