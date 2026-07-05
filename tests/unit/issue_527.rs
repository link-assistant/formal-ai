use formal_ai::{
    generated_question_answers, GeneratedQuestionClass, LogicalMeaningClass, QuestionAcceptance,
    QuestionGenerationConfig, QuestionGenerator, QuestionGrammarClass, QuestionWord,
};

#[test]
fn generated_questions_are_lazy_ordered_by_word_count_and_classified() {
    let config = QuestionGenerationConfig::from_words([
        QuestionWord::from_corpus_scores("what", &[0.99, 0.98]),
        QuestionWord::from_corpus_scores("is", &[0.97, 0.96]),
        QuestionWord::from_corpus_scores("formal", &[0.60, 0.62]),
        QuestionWord::from_corpus_scores("ai", &[0.58, 0.59]),
    ])
    .with_acceptance(QuestionAcceptance::AnyQuestionLike)
    .with_all_ranked_words();

    let generated: Vec<_> = QuestionGenerator::new(config).take(25).collect();

    assert_eq!(
        generated.first().map(|question| question.text.as_str()),
        Some("what?")
    );
    assert!(
        generated
            .windows(2)
            .all(|pair| pair[0].word_count <= pair[1].word_count),
        "questions should be generated one-word first, then two-word, and so on: {generated:?}",
    );
    assert!(
        generated
            .iter()
            .any(|question| question.word_count == 3 && question.text == "what is formal?"),
        "the stream should continue into three-word questions without eager materialization: {generated:?}",
    );

    let one_word = generated
        .iter()
        .find(|question| question.text == "what?")
        .expect("single-word interrogative should be generated");
    assert_eq!(one_word.grammar, QuestionGrammarClass::Fragment);
    assert_eq!(one_word.logical_meaning, LogicalMeaningClass::OpenSlot);

    let meaningful = generated
        .iter()
        .find(|question| question.text == "what is formal?")
        .expect("meaningful three-word question should be generated");
    assert_eq!(meaningful.grammar, QuestionGrammarClass::Grammatical);
    assert_eq!(meaningful.logical_meaning, LogicalMeaningClass::Meaningful);
    assert_eq!(
        meaningful.class,
        GeneratedQuestionClass::GrammaticalAndMeaningful
    );
}

#[test]
fn generator_can_filter_to_grammatical_meaningful_questions() {
    let config = QuestionGenerationConfig::from_words([
        QuestionWord::from_corpus_scores("what", &[1.0, 1.0]),
        QuestionWord::from_corpus_scores("is", &[0.9, 0.9]),
        QuestionWord::from_corpus_scores("formal", &[0.8, 0.8]),
        QuestionWord::from_corpus_scores("ai", &[0.7, 0.7]),
    ])
    .with_acceptance(QuestionAcceptance::GrammaticalAndMeaningful)
    .with_all_ranked_words();

    let generated: Vec<_> = QuestionGenerator::new(config).take(4).collect();

    assert_eq!(
        generated
            .iter()
            .map(|question| question.text.as_str())
            .collect::<Vec<_>>(),
        vec![
            "what is formal?",
            "what is ai?",
            "what formal ai?",
            "is formal ai?",
        ],
    );
    assert!(
        generated
            .iter()
            .all(|question| question.class == GeneratedQuestionClass::GrammaticalAndMeaningful),
        "strict acceptance should remove fragments and logically empty questions: {generated:?}",
    );
}

#[test]
fn default_frequency_policy_reduces_ranked_vocabulary_after_two_words() {
    let mut words = vec![
        QuestionWord::from_corpus_scores("what", &[1.0, 1.0]),
        QuestionWord::from_corpus_scores("is", &[0.99, 0.99]),
        QuestionWord::from_corpus_scores("formal", &[0.98, 0.98]),
        QuestionWord::from_corpus_scores("ai", &[0.97, 0.97]),
    ];
    for offset in 0_u16..36 {
        words.push(QuestionWord::from_corpus_scores(
            format!("term{offset}"),
            &[0.5 - f32::from(offset) / 100.0],
        ));
    }

    let config = QuestionGenerationConfig::from_words(words)
        .with_acceptance(QuestionAcceptance::AnyQuestionLike)
        .with_minimum_ranked_words(1);

    let generated: Vec<_> = QuestionGenerator::new(config).take(40).collect();
    assert!(
        generated.iter().any(|question| question.word_count == 2
            && question.words.iter().any(|word| word == "formal")),
        "the 10% tier should include the third-ranked word for two-word candidates: {generated:?}",
    );

    let three_word_questions: Vec<_> = generated
        .iter()
        .filter(|question| question.word_count == 3)
        .collect();
    assert!(
        !three_word_questions.is_empty(),
        "the stream should reach three-word candidates: {generated:?}",
    );
    assert!(
        three_word_questions
            .iter()
            .all(|question| question.words.iter().all(|word| word != "formal")),
        "the 5% tier should exclude the third-ranked word for three-word candidates: {three_word_questions:?}",
    );
}

#[test]
fn strict_generator_stops_when_no_grammatical_candidate_can_exist() {
    let config = QuestionGenerationConfig::from_words([
        QuestionWord::from_corpus_scores("what", &[1.0]),
        QuestionWord::from_corpus_scores("is", &[0.9]),
    ])
    .with_acceptance(QuestionAcceptance::GrammaticalAndMeaningful)
    .with_all_ranked_words();

    assert_eq!(QuestionGenerator::new(config).next(), None);
}

#[test]
fn generated_question_answer_stream_answers_with_existing_engine() {
    let config = QuestionGenerationConfig::from_words([
        QuestionWord::from_corpus_scores("what", &[1.0, 1.0]),
        QuestionWord::from_corpus_scores("is", &[0.9, 0.9]),
        QuestionWord::from_corpus_scores("formal", &[0.8, 0.8]),
        QuestionWord::from_corpus_scores("ai", &[0.7, 0.7]),
    ])
    .with_acceptance(QuestionAcceptance::GrammaticalAndMeaningful)
    .with_all_ranked_words();

    let answered = generated_question_answers(config)
        .next()
        .expect("answer stream should yield at least one question-answer pair");

    assert_eq!(answered.question.text, "what is formal?");
    assert!(!answered.answer.answer.trim().is_empty());
    assert!(
        answered
            .answer
            .evidence_links
            .iter()
            .any(|link| link.starts_with("trace:")),
        "answers should keep the standard symbolic trace evidence: {:?}",
        answered.answer.evidence_links,
    );
}
