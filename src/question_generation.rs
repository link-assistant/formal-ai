//! Lazy question generation and answering primitives for issue #527.
//!
//! The generator intentionally produces a stream, not a collection: the set of
//! token sequences is unbounded as word count grows. Each candidate is classified
//! before it leaves the iterator so callers can choose whether they want raw
//! question-like fragments, grammatical questions, or only logically meaningful
//! questions.

use std::collections::{BTreeMap, HashSet};
use std::sync::OnceLock;

use crate::engine::{FormalAiEngine, SymbolicAnswer};
use crate::seed::parser::{parse_lino, split_pipe_list};

const BASIS_POINTS_DENOMINATOR: usize = 10_000;

/// The language whose vocabulary and grammar roles the default generator reads.
///
/// Every record in the lexicon carries a `language` tag, so a second language is
/// added purely in data; the enumeration, tiering, and answering logic never
/// branch on the language.
const DEFAULT_LANGUAGE: &str = "en";

/// The frequency-ranked vocabulary and grammar role sets that drive generation,
/// lifted out of Rust literals into `data/seed/question-generation-lexicon.lino`.
///
/// Issue #527 requires the generator to draw from a frequency-tiered vocabulary
/// and to classify candidates grammatically and logically. Historically both the
/// words and the grammar lexicons (interrogative openers, auxiliary openers,
/// function words) lived as inline `matches!`/literal lists here, which branched
/// the *general* generation logic on specific English words. This struct holds
/// them as reviewable link data instead; the Rust code keeps only the structural
/// glue (lazy enumeration, tier curve, grammar/logic gates) and reads every word
/// and role from here. Mirrors the cue-lexicon migration (issue #559).
#[derive(Debug, Clone)]
struct QuestionLexicon {
    /// Grammar roles and frequency vocabulary keyed by language tag (`en`, `ru`,
    /// …). The generation and classification logic never branches on a specific
    /// word *or* language: it reads whichever language the caller selects, and the
    /// classifier recognizes an opener/auxiliary/function word in any seeded
    /// language (surfaces are script-distinct, so languages never collide).
    languages: BTreeMap<String, LanguageLexicon>,
    tier: TierCurve,
}

/// The per-language vocabulary and grammar role sets lifted from the seed data.
#[derive(Debug, Clone, Default)]
struct LanguageLexicon {
    words: Vec<QuestionWord>,
    openers: HashSet<String>,
    auxiliaries: HashSet<String>,
    function_words: HashSet<String>,
}

impl QuestionLexicon {
    /// The frequency vocabulary for `language`, or an empty slice when the
    /// language is not seeded.
    fn words_for(&self, language: &str) -> &[QuestionWord] {
        self.languages
            .get(language)
            .map_or(&[], |lexicon| lexicon.words.as_slice())
    }

    /// The frequency vocabulary for the default language (`en`).
    fn default_words(&self) -> &[QuestionWord] {
        self.words_for(DEFAULT_LANGUAGE)
    }

    /// Whether `token` fills `role` in *any* seeded language. The classifier is
    /// language-agnostic: a Russian opener is recognized exactly like an English
    /// one, because seeded surfaces never overlap across scripts.
    fn any_language_has_role(&self, token: &str, role: GrammarRole) -> bool {
        self.languages.values().any(|lexicon| {
            let set = match role {
                GrammarRole::InterrogativeOpener => &lexicon.openers,
                GrammarRole::AuxiliaryOpener => &lexicon.auxiliaries,
                GrammarRole::FunctionWord => &lexicon.function_words,
            };
            set.contains(token)
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum GrammarRole {
    InterrogativeOpener,
    AuxiliaryOpener,
    FunctionWord,
}

/// The frequency-tier selection curve: how large a slice of the ranked vocabulary
/// a candidate of a given word count may draw from. Stored in the lexicon's
/// `tier_policy` record so the "top 10%, then halve" rule is reviewable data, not a
/// magic number in code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TierCurve {
    base_basis_points: usize,
    halving_start_word_count: usize,
    max_halvings: u32,
    minimum_ranked_words: usize,
}

impl TierCurve {
    /// The basis-point fraction of the ranked vocabulary a `word_count`-word
    /// candidate may draw from: the base tier up to [`Self::halving_start_word_count`]
    /// words, then halved for each additional word (capped at [`Self::max_halvings`]).
    fn basis_points_for_word_count(self, word_count: usize) -> usize {
        if word_count <= self.halving_start_word_count {
            return self.base_basis_points;
        }
        let halvings = word_count
            .saturating_sub(self.halving_start_word_count)
            .min(self.max_halvings as usize);
        (self.base_basis_points >> halvings).max(1)
    }
}

const QUESTION_LEXICON_LINO: &str = include_str!("../data/seed/question-generation-lexicon.lino");

/// The question-generation lexicon, parsed once from the embedded link data.
fn question_lexicon() -> &'static QuestionLexicon {
    static CELL: OnceLock<QuestionLexicon> = OnceLock::new();
    CELL.get_or_init(load_question_lexicon)
}

fn load_question_lexicon() -> QuestionLexicon {
    let tree = parse_lino(QUESTION_LEXICON_LINO);
    let mut languages: BTreeMap<String, LanguageLexicon> = BTreeMap::new();
    let mut tier = TierCurve {
        base_basis_points: 1_000,
        halving_start_word_count: 2,
        max_halvings: 9,
        minimum_ranked_words: 4,
    };

    for record in &tree.children {
        match record.find_child_value("record_type") {
            "frequency_word" => {
                let language = record.find_child_value("language");
                if language.is_empty() {
                    continue;
                }
                let surface = record.find_child_value("surface");
                if surface.is_empty() {
                    continue;
                }
                let scores: Vec<f32> = split_pipe_list(record.find_child_value("frequency_scores"))
                    .iter()
                    .filter_map(|token| token.parse::<f32>().ok())
                    .collect();
                languages
                    .entry(language.to_string())
                    .or_default()
                    .words
                    .push(QuestionWord::from_corpus_scores(surface, &scores));
            }
            "grammar_role" => {
                let language = record.find_child_value("language");
                if language.is_empty() {
                    continue;
                }
                let members: Vec<String> = split_pipe_list(record.find_child_value("member"))
                    .into_iter()
                    .map(|member| member.to_ascii_lowercase())
                    .collect();
                let lexicon = languages.entry(language.to_string()).or_default();
                match record.find_child_value("role") {
                    "interrogative_opener" => lexicon.openers.extend(members),
                    "auxiliary_opener" => lexicon.auxiliaries.extend(members),
                    "function_word" => lexicon.function_words.extend(members),
                    _ => {}
                }
            }
            "tier_policy" => {
                if let Some(value) = parse_usize(record.find_child_value("base_basis_points")) {
                    tier.base_basis_points = value;
                }
                if let Some(value) =
                    parse_usize(record.find_child_value("halving_start_word_count"))
                {
                    tier.halving_start_word_count = value;
                }
                if let Some(value) = parse_usize(record.find_child_value("max_halvings"))
                    .and_then(|value| u32::try_from(value).ok())
                {
                    tier.max_halvings = value;
                }
                if let Some(value) = parse_usize(record.find_child_value("minimum_ranked_words")) {
                    tier.minimum_ranked_words = value;
                }
            }
            _ => {}
        }
    }

    QuestionLexicon { languages, tier }
}

fn parse_usize(value: &str) -> Option<usize> {
    value.trim().parse::<usize>().ok()
}

/// A read-only view of the seed lexicon the generator reads, so a grounding test
/// can pin the data to the behavior it drives (R13) without the generator having
/// to expose its internal tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionLexiconSummary {
    /// The language tag this summary describes (`en`, `ru`, `hi`, `zh`).
    pub language: String,
    /// Ranked vocabulary surfaces, most frequent first (ties broken alphabetically).
    pub vocabulary: Vec<String>,
    /// `interrogative_opener` role members, sorted.
    pub interrogative_openers: Vec<String>,
    /// `auxiliary_opener` role members, sorted.
    pub auxiliary_openers: Vec<String>,
    /// `function_word` role members, sorted.
    pub function_words: Vec<String>,
    /// Base frequency tier in basis points (1000 bp = the top 10%).
    pub tier_base_basis_points: usize,
    /// Minimum number of ranked words a tier may shrink to.
    pub tier_minimum_ranked_words: usize,
}

/// Summarize the seed lexicon the default generator reads (the `en` language).
/// Exposed for the issue-#527 grounding test.
#[must_use]
pub fn question_lexicon_summary() -> QuestionLexiconSummary {
    question_lexicon_summary_for_language(DEFAULT_LANGUAGE)
        .expect("the default language must be present in the seed lexicon")
}

/// Summarize the seed lexicon for a specific `language`.
///
/// Returns `None` when that language is not seeded. Every supported language is
/// grounded to its behavior by the issue-#527 tests, so a one-language regression
/// cannot land silently.
#[must_use]
pub fn question_lexicon_summary_for_language(language: &str) -> Option<QuestionLexiconSummary> {
    let lexicon = question_lexicon();
    let language_lexicon = lexicon.languages.get(language)?;
    let vocabulary = QuestionGenerationConfig::for_language(language)
        .words()
        .iter()
        .map(|word| word.surface.clone())
        .collect();
    Some(QuestionLexiconSummary {
        language: language.to_string(),
        vocabulary,
        interrogative_openers: sorted(&language_lexicon.openers),
        auxiliary_openers: sorted(&language_lexicon.auxiliaries),
        function_words: sorted(&language_lexicon.function_words),
        tier_base_basis_points: lexicon.tier.base_basis_points,
        tier_minimum_ranked_words: lexicon.tier.minimum_ranked_words,
    })
}

fn sorted(set: &HashSet<String>) -> Vec<String> {
    let mut members: Vec<String> = set.iter().cloned().collect();
    members.sort();
    members
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuestionWord {
    pub surface: String,
    pub average_frequency_score: f32,
    pub corpus_count: usize,
}

impl QuestionWord {
    #[must_use]
    pub fn from_corpus_scores(surface: impl Into<String>, scores: &[f32]) -> Self {
        let mut corpus_count = 0usize;
        let mut score_count = 0.0;
        let mut score_sum = 0.0;
        for score in scores.iter().copied().filter(|score| score.is_finite()) {
            corpus_count += 1;
            score_count += 1.0;
            score_sum += score;
        }
        let average_frequency_score = if corpus_count == 0 {
            0.0
        } else {
            score_sum / score_count
        };

        Self {
            surface: normalize_word_surface(&surface.into()),
            average_frequency_score,
            corpus_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionAcceptance {
    AnyQuestionLike,
    Grammatical,
    GrammaticalAndMeaningful,
}

impl QuestionAcceptance {
    fn accepts(self, question: &GeneratedQuestion) -> bool {
        match self {
            Self::AnyQuestionLike => true,
            Self::Grammatical => question.grammar == QuestionGrammarClass::Grammatical,
            Self::GrammaticalAndMeaningful => {
                question.class == GeneratedQuestionClass::GrammaticalAndMeaningful
            }
        }
    }

    const fn requires_grammatical_candidate(self) -> bool {
        matches!(self, Self::Grammatical | Self::GrammaticalAndMeaningful)
    }
}

/// How the generator restricts the ranked vocabulary as candidates grow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrequencySelectionPolicy {
    /// The default: draw from a frequency tier that shrinks with word count
    /// (top [`TierCurve::base_basis_points`], halving per extra word).
    FrequencyTiers,
    /// Draw from the whole ranked vocabulary regardless of word count.
    AllRankedWords,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuestionGenerationConfig {
    words: Vec<QuestionWord>,
    acceptance: QuestionAcceptance,
    frequency_policy: FrequencySelectionPolicy,
    tier: TierCurve,
    minimum_ranked_words: usize,
}

impl Default for QuestionGenerationConfig {
    fn default() -> Self {
        // The default vocabulary is the frequency-ranked word list from the seed
        // lexicon — no words are hardcoded here.
        Self::for_language(DEFAULT_LANGUAGE)
    }
}

impl QuestionGenerationConfig {
    /// Build a config from a seeded `language`'s frequency vocabulary. The
    /// enumeration, tiering, and classification logic is language-agnostic, so
    /// selecting a language changes only which words feed the stream. Falls back
    /// to the default language when `language` is not seeded.
    #[must_use]
    pub fn for_language(language: &str) -> Self {
        let words = question_lexicon().words_for(language);
        let words = if words.is_empty() {
            question_lexicon().default_words()
        } else {
            words
        };
        Self::from_words(words.iter().cloned())
    }

    #[must_use]
    pub fn from_words<I>(words: I) -> Self
    where
        I: IntoIterator<Item = QuestionWord>,
    {
        let mut ranked: Vec<QuestionWord> = words
            .into_iter()
            .filter(|word| !word.surface.trim().is_empty())
            .collect();
        ranked.sort_by(|left, right| {
            right
                .average_frequency_score
                .total_cmp(&left.average_frequency_score)
                .then_with(|| left.surface.cmp(&right.surface))
        });

        let mut seen = HashSet::new();
        ranked.retain(|word| seen.insert(word.surface.to_ascii_lowercase()));

        let tier = question_lexicon().tier;
        Self {
            words: ranked,
            acceptance: QuestionAcceptance::GrammaticalAndMeaningful,
            frequency_policy: FrequencySelectionPolicy::FrequencyTiers,
            tier,
            minimum_ranked_words: tier.minimum_ranked_words,
        }
    }

    #[must_use]
    pub const fn with_acceptance(mut self, acceptance: QuestionAcceptance) -> Self {
        self.acceptance = acceptance;
        self
    }

    #[must_use]
    pub const fn with_all_ranked_words(mut self) -> Self {
        self.frequency_policy = FrequencySelectionPolicy::AllRankedWords;
        self
    }

    #[must_use]
    pub const fn with_minimum_ranked_words(mut self, minimum_ranked_words: usize) -> Self {
        self.minimum_ranked_words = minimum_ranked_words;
        self
    }

    #[must_use]
    pub fn words(&self) -> &[QuestionWord] {
        &self.words
    }

    fn ranked_word_limit(&self, word_count: usize) -> usize {
        match self.frequency_policy {
            FrequencySelectionPolicy::AllRankedWords => self.words.len(),
            FrequencySelectionPolicy::FrequencyTiers => {
                let basis_points = self.tier.basis_points_for_word_count(word_count);
                let selected = self
                    .words
                    .len()
                    .saturating_mul(basis_points)
                    .saturating_add(BASIS_POINTS_DENOMINATOR - 1)
                    / BASIS_POINTS_DENOMINATOR;
                selected
                    .max(1)
                    .max(self.minimum_ranked_words.min(self.words.len()))
                    .min(self.words.len())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionGrammarClass {
    Fragment,
    Grammatical,
    Ungrammatical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalMeaningClass {
    Meaningful,
    OpenSlot,
    NotMeaningful,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedQuestionClass {
    GrammaticalAndMeaningful,
    GrammaticalOpenSlot,
    Fragment,
    Ungrammatical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedQuestion {
    pub text: String,
    pub words: Vec<String>,
    pub word_count: usize,
    pub grammar: QuestionGrammarClass,
    pub logical_meaning: LogicalMeaningClass,
    pub class: GeneratedQuestionClass,
}

#[derive(Debug, Clone)]
pub struct QuestionGenerator {
    config: QuestionGenerationConfig,
    word_count: usize,
    indices: Vec<usize>,
    exhausted: bool,
}

impl QuestionGenerator {
    #[must_use]
    pub fn new(config: QuestionGenerationConfig) -> Self {
        let exhausted = config.words.is_empty()
            || !config.words.iter().any(|word| {
                is_question_opener(&word.surface) || is_auxiliary_opener(&word.surface)
            });
        Self {
            config,
            word_count: 1,
            indices: vec![0],
            exhausted,
        }
    }
}

impl Default for QuestionGenerator {
    fn default() -> Self {
        Self::new(QuestionGenerationConfig::default())
    }
}

impl Iterator for QuestionGenerator {
    type Item = GeneratedQuestion;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        loop {
            let limit = self.config.ranked_word_limit(self.word_count);
            if limit == 0 {
                self.exhausted = true;
                return None;
            }
            if self.config.acceptance.requires_grammatical_candidate() && self.word_count > limit {
                self.exhausted = true;
                return None;
            }
            if self.indices.iter().any(|index| *index >= limit) {
                self.indices = vec![0; self.word_count];
            }

            let question = self.current_question();
            self.advance(limit);

            if let Some(question) = question {
                if self.config.acceptance.accepts(&question) {
                    return Some(question);
                }
            }
        }
    }
}

impl QuestionGenerator {
    fn current_question(&self) -> Option<GeneratedQuestion> {
        let tokens: Vec<String> = self
            .indices
            .iter()
            .filter_map(|index| self.config.words.get(*index))
            .map(|word| word.surface.clone())
            .collect();

        if tokens.len() != self.word_count || !is_question_like(&tokens) {
            return None;
        }

        Some(classify_question(tokens, &self.indices))
    }

    fn advance(&mut self, limit: usize) {
        for position in (0..self.indices.len()).rev() {
            if self.indices[position] + 1 < limit {
                self.indices[position] += 1;
                return;
            }
            self.indices[position] = 0;
        }

        self.word_count += 1;
        self.indices = vec![0; self.word_count];
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedQuestionAnswer {
    pub question: GeneratedQuestion,
    pub answer: SymbolicAnswer,
}

#[derive(Debug, Clone)]
pub struct GeneratedQuestionAnswerStream {
    questions: QuestionGenerator,
    engine: FormalAiEngine,
}

impl Iterator for GeneratedQuestionAnswerStream {
    type Item = GeneratedQuestionAnswer;

    fn next(&mut self) -> Option<Self::Item> {
        self.questions
            .next()
            .map(|question| GeneratedQuestionAnswer {
                answer: self.engine.answer(&question.text),
                question,
            })
    }
}

#[must_use]
pub fn generated_question_answers(
    config: QuestionGenerationConfig,
) -> GeneratedQuestionAnswerStream {
    GeneratedQuestionAnswerStream {
        questions: QuestionGenerator::new(config),
        engine: FormalAiEngine,
    }
}

fn classify_question(tokens: Vec<String>, indices: &[usize]) -> GeneratedQuestion {
    let word_count = tokens.len();
    let grammar = classify_grammar(&tokens, indices);
    let logical_meaning = classify_logical_meaning(&tokens, indices, grammar);
    let class = match (grammar, logical_meaning) {
        (QuestionGrammarClass::Grammatical, LogicalMeaningClass::Meaningful) => {
            GeneratedQuestionClass::GrammaticalAndMeaningful
        }
        (QuestionGrammarClass::Grammatical, LogicalMeaningClass::OpenSlot) => {
            GeneratedQuestionClass::GrammaticalOpenSlot
        }
        (QuestionGrammarClass::Fragment, _) => GeneratedQuestionClass::Fragment,
        (QuestionGrammarClass::Ungrammatical, _) | (_, LogicalMeaningClass::NotMeaningful) => {
            GeneratedQuestionClass::Ungrammatical
        }
    };

    GeneratedQuestion {
        text: format!("{}?", tokens.join(" ")),
        words: tokens,
        word_count,
        grammar,
        logical_meaning,
        class,
    }
}

fn classify_grammar(tokens: &[String], indices: &[usize]) -> QuestionGrammarClass {
    let Some(first) = tokens.first() else {
        return QuestionGrammarClass::Ungrammatical;
    };

    if tokens.len() == 1 {
        return if is_question_opener(first) || is_auxiliary_opener(first) {
            QuestionGrammarClass::Fragment
        } else {
            QuestionGrammarClass::Ungrammatical
        };
    }

    if !is_question_opener(first) && !is_auxiliary_opener(first) {
        return QuestionGrammarClass::Ungrammatical;
    }
    if has_duplicate_token(tokens) || !tail_indices_are_ordered(indices) {
        return QuestionGrammarClass::Ungrammatical;
    }
    if tokens
        .iter()
        .skip(1)
        .any(|token| is_question_pronoun(token))
    {
        return QuestionGrammarClass::Ungrammatical;
    }
    if tokens.len() == 2 {
        return QuestionGrammarClass::Fragment;
    }
    if is_question_pronoun(first) {
        if tokens
            .get(1)
            .is_some_and(|token| is_auxiliary_opener(token) || is_content_word(token))
        {
            return QuestionGrammarClass::Grammatical;
        }
        return QuestionGrammarClass::Ungrammatical;
    }
    if is_auxiliary_opener(first) && tokens.iter().skip(1).all(|token| is_content_word(token)) {
        return QuestionGrammarClass::Grammatical;
    }

    QuestionGrammarClass::Ungrammatical
}

fn classify_logical_meaning(
    tokens: &[String],
    indices: &[usize],
    grammar: QuestionGrammarClass,
) -> LogicalMeaningClass {
    match grammar {
        QuestionGrammarClass::Ungrammatical => LogicalMeaningClass::NotMeaningful,
        QuestionGrammarClass::Fragment => LogicalMeaningClass::OpenSlot,
        QuestionGrammarClass::Grammatical => {
            let content_count = tokens
                .iter()
                .skip(1)
                .filter(|token| is_content_word(token))
                .count();
            if content_count > 0
                && tokens.last().is_some_and(|token| is_content_word(token))
                && tail_indices_are_ordered(indices)
            {
                LogicalMeaningClass::Meaningful
            } else {
                LogicalMeaningClass::OpenSlot
            }
        }
    }
}

fn normalize_word_surface(surface: &str) -> String {
    surface
        .trim()
        .trim_end_matches('?')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn is_question_like(tokens: &[String]) -> bool {
    tokens
        .first()
        .is_some_and(|token| is_question_opener(token) || is_auxiliary_opener(token))
}

/// Whether `token` is an interrogative opener (a wh-word). Reads the
/// `interrogative_opener` role from the seed lexicon; never a hardcoded list.
fn is_question_opener(token: &str) -> bool {
    question_lexicon().any_language_has_role(
        &token.to_ascii_lowercase(),
        GrammarRole::InterrogativeOpener,
    )
}

fn is_question_pronoun(token: &str) -> bool {
    is_question_opener(token)
}

/// Whether `token` is an auxiliary/modal opener. Reads the `auxiliary_opener`
/// role from the seed lexicon; never a hardcoded list.
fn is_auxiliary_opener(token: &str) -> bool {
    question_lexicon()
        .any_language_has_role(&token.to_ascii_lowercase(), GrammarRole::AuxiliaryOpener)
}

/// Whether `token` carries standalone content — anything that is neither an
/// interrogative opener, an auxiliary opener, nor a closed-class `function_word`
/// (all three role sets come from the seed lexicon).
fn is_content_word(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    !is_question_pronoun(&lower)
        && !is_auxiliary_opener(&lower)
        && !question_lexicon().any_language_has_role(&lower, GrammarRole::FunctionWord)
}

fn has_duplicate_token(tokens: &[String]) -> bool {
    let mut seen = HashSet::new();
    tokens
        .iter()
        .any(|token| !seen.insert(token.to_ascii_lowercase()))
}

fn tail_indices_are_ordered(indices: &[usize]) -> bool {
    indices
        .iter()
        .skip(1)
        .zip(indices.iter().skip(2))
        .all(|(left, right)| left < right)
}
