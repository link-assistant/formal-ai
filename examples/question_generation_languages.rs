//! Issue #527: the question generator is language-agnostic.
//!
//! The same enumeration, frequency-tiering, and grammar/logic classification runs
//! over whichever language the caller selects. This example prints the first
//! grammatical, logically meaningful questions the generator produces for every
//! supported language (English, Russian, Hindi, Chinese) straight from the seed
//! lexicon — no language-specific code path is involved.
//!
//! Run with: `cargo run --example question_generation_languages`

use formal_ai::{
    question_lexicon_summary_for_language, GeneratedQuestionClass, QuestionAcceptance,
    QuestionGenerationConfig, QuestionGenerator,
};

fn main() {
    for (language, name) in [
        ("en", "English"),
        ("ru", "Russian"),
        ("hi", "Hindi"),
        ("zh", "Chinese"),
    ] {
        let Some(summary) = question_lexicon_summary_for_language(language) else {
            continue;
        };
        println!("== {name} ({language}) ==");
        println!("  vocabulary: {:?}", summary.vocabulary);

        let config = QuestionGenerationConfig::for_language(language)
            .with_acceptance(QuestionAcceptance::GrammaticalAndMeaningful);
        let meaningful: Vec<String> = QuestionGenerator::new(config)
            .take(500)
            .filter(|question| question.class == GeneratedQuestionClass::GrammaticalAndMeaningful)
            .take(3)
            .map(|question| question.text)
            .collect();
        println!("  meaningful questions: {meaningful:?}\n");
    }
}
