//! Issue #527: the question generator's vocabulary and grammar roles are grounded
//! link data, not Rust literals.
//!
//! Issue #527 asks Formal AI to generate every possible question from a
//! frequency-tiered vocabulary and classify each candidate grammatically and
//! logically. That generation logic must stay general — it may never branch on a
//! specific word. These tests pin the two tables the logic reads
//! (`data/seed/question-generation-lexicon.lino`) to the behavior they drive, so
//! the data can never silently drift from the code (mirrors the cue-lexicon
//! grounding tests for issue #559):
//!
//! 1. the seed vocabulary and grammar roles load with the expected members;
//! 2. the grammar roles actually gate classification (an opener/auxiliary/function
//!    word is treated as such by the generator);
//! 3. the frequency-tier curve stored in data reproduces the "top 10%, then halve"
//!    behavior the issue specifies.

use formal_ai::{
    question_lexicon_summary, question_lexicon_summary_for_language, GeneratedQuestionClass,
    LogicalMeaningClass, QuestionAcceptance, QuestionGenerationConfig, QuestionGenerator,
    QuestionGrammarClass,
};

#[test]
fn seed_lexicon_loads_expected_vocabulary_and_grammar_roles() {
    let summary = question_lexicon_summary();

    // The frequency-ranked default vocabulary comes entirely from the seed data.
    assert_eq!(
        summary.vocabulary,
        vec![
            "what", "is", "who", "formal", "ai", "where", "when", "why", "how", "does", "can",
            "you", "work", "answer", "question", "language",
        ],
        "the default vocabulary must be the frequency-ranked seed word list, not a Rust literal",
    );

    assert_eq!(
        summary.interrogative_openers,
        vec!["how", "what", "when", "where", "which", "who", "why"],
        "the interrogative-opener role must come from the seed lexicon",
    );
    assert_eq!(
        summary.auxiliary_openers,
        vec![
            "am", "are", "can", "could", "did", "do", "does", "has", "have", "is", "should", "was",
            "were", "will", "would",
        ],
        "the auxiliary-opener role must come from the seed lexicon",
    );
    assert_eq!(
        summary.function_words,
        vec!["a", "an", "for", "in", "of", "on", "the", "to"],
        "the function-word role must come from the seed lexicon",
    );

    // The frequency-tier curve is data, too: 1000 basis points = the top 10%.
    assert_eq!(summary.tier_base_basis_points, 1_000);
    assert_eq!(summary.tier_minimum_ranked_words, 4);
}

#[test]
fn seed_grammar_roles_gate_classification() {
    // Build a config from words whose grammatical behavior depends on the seed
    // roles: `which` (interrogative opener), `have` (auxiliary opener), `the`
    // (function word), and two content words.
    let config = QuestionGenerationConfig::from_words([
        formal_ai::QuestionWord::from_corpus_scores("which", &[1.0]),
        formal_ai::QuestionWord::from_corpus_scores("have", &[0.9]),
        formal_ai::QuestionWord::from_corpus_scores("robots", &[0.8]),
        formal_ai::QuestionWord::from_corpus_scores("dreams", &[0.7]),
        formal_ai::QuestionWord::from_corpus_scores("the", &[0.6]),
    ])
    .with_acceptance(QuestionAcceptance::AnyQuestionLike)
    .with_all_ranked_words();

    let generated: Vec<_> = QuestionGenerator::new(config).take(80).collect();

    // `which` opens grammatical content questions (interrogative_opener role).
    let opener_question = generated
        .iter()
        .find(|question| question.text == "which have robots?")
        .expect("`which` should open a grammatical question via the seed role");
    assert_eq!(opener_question.grammar, QuestionGrammarClass::Grammatical);

    // `have` opens grammatical yes/no questions (auxiliary_opener role).
    let auxiliary_question = generated
        .iter()
        .find(|question| question.text == "have robots dreams?")
        .expect("`have` should open a grammatical yes/no question via the seed role");
    assert_eq!(
        auxiliary_question.grammar,
        QuestionGrammarClass::Grammatical
    );

    // A trailing function word leaves the question an open slot, never meaningful.
    let function_tail = generated
        .iter()
        .find(|question| question.text == "which have the?");
    if let Some(question) = function_tail {
        assert_ne!(
            question.class,
            GeneratedQuestionClass::GrammaticalAndMeaningful,
            "a question ending in a function word must not be logically meaningful",
        );
        assert_ne!(question.logical_meaning, LogicalMeaningClass::Meaningful);
    }
}

#[test]
fn seed_tier_curve_reproduces_halving_behavior() {
    // 20 ranked words: the top-10% tier admits 2 words for one/two-word candidates,
    // and the 5% tier admits 1 word for three-word candidates (rounded up).
    let mut words = Vec::new();
    for rank in 0..20u16 {
        words.push(formal_ai::QuestionWord::from_corpus_scores(
            // Distinct surfaces so ranking is stable; the first two are openers so
            // the stream produces grammatical candidates.
            match rank {
                0 => "what".to_string(),
                1 => "is".to_string(),
                other => format!("term{other}"),
            },
            &[1.0 - f32::from(rank) / 100.0],
        ));
    }

    let config = QuestionGenerationConfig::from_words(words)
        .with_acceptance(QuestionAcceptance::AnyQuestionLike)
        .with_minimum_ranked_words(1);

    let generated: Vec<_> = QuestionGenerator::new(config).take(60).collect();

    // The 10% tier of 20 words = 2 words, so two-word candidates only combine the
    // top two ranked words ("what", "is").
    let two_word: Vec<_> = generated
        .iter()
        .filter(|question| question.word_count == 2)
        .collect();
    assert!(
        two_word
            .iter()
            .all(|question| question.words.iter().all(|word| word == "what" || word == "is")),
        "the top-10% tier should restrict two-word candidates to the two most frequent words: {two_word:?}",
    );

    // Three-word candidates draw from the 5% tier (still 1 word after rounding up),
    // so no `term*` word ever appears — the tier halved as word count grew.
    let three_word: Vec<_> = generated
        .iter()
        .filter(|question| question.word_count == 3)
        .collect();
    assert!(
        !three_word.is_empty(),
        "the stream should reach three-word candidates",
    );
    assert!(
        three_word
            .iter()
            .all(|question| question.words.iter().all(|word| !word.starts_with("term"))),
        "the halved 5% tier should exclude lower-ranked words for three-word candidates: {three_word:?}",
    );
}

/// The generation and classification logic is language-agnostic: it never
/// branches on a specific word *or* language. Issue #527 seeds English, Russian,
/// Hindi, and Chinese vocabulary and grammar roles into the same lexicon, and the
/// same Rust glue must turn each language's frequency-ranked words into a
/// grammatical, logically meaningful question. This pins that guarantee for every
/// supported language so a one-language regression cannot land silently.
#[test]
fn seed_lexicon_generates_questions_in_every_supported_language() {
    // (language tag, native interrogative opener, English name for messages).
    let cases = [
        ("en", "what", "English"),
        ("ru", "что", "Russian"),
        ("hi", "क्या", "Hindi"),
        ("zh", "什么", "Chinese"),
    ];

    for (language, opener, name) in cases {
        let summary = question_lexicon_summary_for_language(language)
            .unwrap_or_else(|| panic!("{name} ({language}) must be seeded in the lexicon"));

        assert_eq!(
            summary.language, language,
            "{name} summary must report its own language tag",
        );
        assert!(
            summary.vocabulary.contains(&opener.to_string()),
            "{name} ({language}) vocabulary must include its interrogative opener {opener:?}: {:?}",
            summary.vocabulary,
        );
        assert!(
            summary.interrogative_openers.contains(&opener.to_string()),
            "{name} ({language}) must carry {opener:?} in its interrogative_opener role: {:?}",
            summary.interrogative_openers,
        );
        // The frequency-tier curve is shared data, so every language reads the
        // same "top 10%, then halve" policy.
        assert_eq!(summary.tier_base_basis_points, 1_000);
        assert_eq!(summary.tier_minimum_ranked_words, 4);

        // The same generator produces a grammatical, meaningful question from this
        // language's seeded words — no language-specific code path involved.
        let config = QuestionGenerationConfig::for_language(language)
            .with_acceptance(QuestionAcceptance::GrammaticalAndMeaningful);
        let meaningful = QuestionGenerator::new(config)
            .take(500)
            .find(|question| {
                question.class == GeneratedQuestionClass::GrammaticalAndMeaningful
                    && question.words.first().map(String::as_str) == Some(opener)
            })
            .unwrap_or_else(|| {
                panic!("the generator must produce a meaningful {name} ({language}) question opening with {opener:?}")
            });

        assert_eq!(meaningful.grammar, QuestionGrammarClass::Grammatical);
        assert_eq!(meaningful.logical_meaning, LogicalMeaningClass::Meaningful);
        assert!(
            meaningful.word_count >= 3,
            "a meaningful {name} question needs an opener and content: {meaningful:?}",
        );
    }
}
