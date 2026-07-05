//! Lazy question generation and answering primitives for issue #527.
//!
//! The generator intentionally produces a stream, not a collection: the set of
//! token sequences is unbounded as word count grows. Each candidate is classified
//! before it leaves the iterator so callers can choose whether they want raw
//! question-like fragments, grammatical questions, or only logically meaningful
//! questions.

use std::collections::HashSet;

use crate::engine::{FormalAiEngine, SymbolicAnswer};

const DEFAULT_MIN_RANKED_WORDS: usize = 4;
const BASIS_POINTS_DENOMINATOR: usize = 10_000;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrequencySelectionPolicy {
    Issue527Percentages,
    AllRankedWords,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuestionGenerationConfig {
    words: Vec<QuestionWord>,
    acceptance: QuestionAcceptance,
    frequency_policy: FrequencySelectionPolicy,
    minimum_ranked_words: usize,
}

impl Default for QuestionGenerationConfig {
    fn default() -> Self {
        Self::from_words([
            QuestionWord::from_corpus_scores("what", &[0.99, 0.98, 0.97]),
            QuestionWord::from_corpus_scores("is", &[0.98, 0.97, 0.96]),
            QuestionWord::from_corpus_scores("who", &[0.95, 0.94, 0.93]),
            QuestionWord::from_corpus_scores("formal", &[0.90, 0.88, 0.86]),
            QuestionWord::from_corpus_scores("ai", &[0.89, 0.87, 0.85]),
            QuestionWord::from_corpus_scores("where", &[0.84, 0.82, 0.81]),
            QuestionWord::from_corpus_scores("when", &[0.83, 0.81, 0.80]),
            QuestionWord::from_corpus_scores("why", &[0.82, 0.80, 0.79]),
            QuestionWord::from_corpus_scores("how", &[0.81, 0.79, 0.78]),
            QuestionWord::from_corpus_scores("does", &[0.80, 0.78, 0.77]),
            QuestionWord::from_corpus_scores("can", &[0.79, 0.77, 0.76]),
            QuestionWord::from_corpus_scores("you", &[0.78, 0.76, 0.75]),
            QuestionWord::from_corpus_scores("work", &[0.70, 0.69, 0.68]),
            QuestionWord::from_corpus_scores("answer", &[0.69, 0.68, 0.67]),
            QuestionWord::from_corpus_scores("question", &[0.68, 0.67, 0.66]),
            QuestionWord::from_corpus_scores("language", &[0.67, 0.66, 0.65]),
        ])
    }
}

impl QuestionGenerationConfig {
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

        Self {
            words: ranked,
            acceptance: QuestionAcceptance::GrammaticalAndMeaningful,
            frequency_policy: FrequencySelectionPolicy::Issue527Percentages,
            minimum_ranked_words: DEFAULT_MIN_RANKED_WORDS,
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
            FrequencySelectionPolicy::Issue527Percentages => {
                let basis_points = frequency_basis_points_for_word_count(word_count);
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

fn frequency_basis_points_for_word_count(word_count: usize) -> usize {
    if word_count <= 2 {
        return 1_000;
    }
    let halvings = word_count.saturating_sub(2).min(9);
    (1_000 / (1_usize << halvings)).max(1)
}

fn is_question_like(tokens: &[String]) -> bool {
    tokens
        .first()
        .is_some_and(|token| is_question_opener(token) || is_auxiliary_opener(token))
}

fn is_question_opener(token: &str) -> bool {
    matches!(
        token,
        "what" | "who" | "when" | "where" | "why" | "how" | "which"
    )
}

fn is_question_pronoun(token: &str) -> bool {
    is_question_opener(token)
}

fn is_auxiliary_opener(token: &str) -> bool {
    matches!(
        token,
        "is" | "are"
            | "am"
            | "was"
            | "were"
            | "do"
            | "does"
            | "did"
            | "can"
            | "could"
            | "should"
            | "would"
            | "will"
            | "has"
            | "have"
    )
}

fn is_content_word(token: &str) -> bool {
    !is_question_pronoun(token)
        && !is_auxiliary_opener(token)
        && !matches!(
            token,
            "a" | "an" | "the" | "to" | "of" | "for" | "in" | "on"
        )
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
