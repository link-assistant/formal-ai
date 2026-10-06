//! Issue #1174 (E139, text-transform family): requests that ask the engine to
//! transform text the request itself carries — summarize it, rewrite it in a
//! formal register, correct its grammar, compose genre writing from it, or
//! translate a whole sentence — are answered with the computed text, not a
//! refusal or an echo of the input.
//!
//! The handlers are exercised directly (the dispatch wiring belongs to the
//! coordinating session), so each probe pins both the intent and the computed
//! body. Every probe is hermetic: all vocabulary — register pairs, agreement
//! rules, genre frames, response templates, and the translation sentence's
//! compositional lemmas — lives in seed data, so no test touches the network.

use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::{
    EventLog, SymbolicAnswer, handle_summarization_request, handle_text_rewrite, try_translation,
};

type Handler = fn(&str, &str, &mut EventLog) -> Option<SymbolicAnswer>;

/// The answer a handler gives to `prompt`.
fn solved_by(handler: Handler, prompt: &str) -> SymbolicAnswer {
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    handler(prompt, &normalized, &mut log)
        .unwrap_or_else(|| panic!("the handler should answer: {prompt}"))
}

/// Whether a handler declines `prompt` (no payload, or another family's cue).
fn declined_by(handler: Handler, prompt: &str) -> bool {
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    handler(prompt, &normalized, &mut log).is_none()
}

#[test]
fn summarization_selects_the_weightiest_statements() {
    let answer = solved_by(
        handle_summarization_request,
        "Summarize this paragraph: The Halley research station, opened in 1956, is used to study \
         the Antarctic ice shelf. It provides year-round measurements of ozone and sea \
         temperature. The crew rotates every summer. Supplies arrive by ship in February. Radar \
         masts surround the living quarters.",
    );
    assert_eq!(answer.intent, "summarization_free_text");
    // Five statements under the ~30% bound keep two: the purpose statement
    // (weight 100) then the feature statement (weight 70), joined as prose.
    assert_eq!(
        answer.answer,
        "The Halley research station, opened in 1956, is used to study the Antarctic ice \
         shelf. It provides year-round measurements of ozone and sea temperature."
    );
    // The purpose and feature statements outrank the misc ones.
    assert!(
        answer.answer.contains("is used to study"),
        "the purpose statement must survive: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("year-round measurements"),
        "the feature statement must survive: {}",
        answer.answer
    );
    assert!(
        !answer.answer.contains("Radar masts"),
        "a misc statement must be dropped: {}",
        answer.answer
    );
}

#[test]
fn summarization_declines_a_prompt_without_text() {
    assert!(declined_by(handle_summarization_request, "Summarize this."));
}

#[test]
fn register_rewrite_substitutes_every_informal_token() {
    let answer = solved_by(
        handle_text_rewrite,
        "Rewrite this formally: can u send me the report asap thx",
    );
    assert_eq!(answer.intent, "text_transform_register");
    assert_eq!(
        answer.answer,
        "could you please send me the report as soon as possible thank you"
    );
    assert!(
        answer.answer.contains("could you please"),
        "the phrase rule must splice the politeness form: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("as soon as possible"),
        "the seeded expansion must replace the acronym: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("thank you"),
        "the seeded expansion must replace the abbreviation: {}",
        answer.answer
    );
    assert!(
        !answer.answer.contains("asap") && !answer.answer.contains("thx"),
        "no informal token may survive: {}",
        answer.answer
    );
}

#[test]
fn register_rewrite_reports_an_already_formal_text() {
    let answer = solved_by(
        handle_text_rewrite,
        "Rewrite this formally: please send the report tomorrow",
    );
    assert_eq!(answer.intent, "text_transform_register");
    assert_eq!(
        answer.answer,
        "please send the report tomorrow\n\nThe text already uses a formal register."
    );
    assert!(
        answer.answer.contains("please send the report tomorrow"),
        "the formal text must pass through unchanged: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("already uses a formal register"),
        "the honest no-change note must follow: {}",
        answer.answer
    );
}

#[test]
fn grammar_correction_reagrees_subjects_and_pluralizes() {
    let answer = solved_by(
        handle_text_rewrite,
        "Correct the grammar: She don't like apples and he have two cat.",
    );
    assert_eq!(answer.intent, "text_transform_grammar");
    assert_eq!(
        answer.answer,
        "Corrected text: She doesn't like apples and he has two cats.\n\n\
         Corrections:\n\
         don't → doesn't (rule: third_person_do_not)\n\
         have → has (rule: third_person_have)\n\
         cat → cats (rule: plural_noun_after_numeral)"
    );
    for corrected in ["doesn't", "has", "cats"] {
        assert!(
            answer.answer.contains(corrected),
            "the corrected form must appear: {corrected} in {}",
            answer.answer
        );
    }
    for rule in [
        "third_person_do_not",
        "third_person_have",
        "plural_noun_after_numeral",
    ] {
        assert!(
            answer.answer.contains(rule),
            "each fired rule must be named: {rule} in {}",
            answer.answer
        );
    }
}

#[test]
fn grammar_correction_reports_an_already_clean_text() {
    let answer = solved_by(
        handle_text_rewrite,
        "Correct the grammar: She likes apples and he has two cats.",
    );
    assert_eq!(answer.intent, "text_transform_grammar");
    assert_eq!(
        answer.answer,
        "No agreement errors found; the text is already grammatical."
    );
    assert!(
        answer.answer.contains("already grammatical"),
        "a clean text must be reported honestly: {}",
        answer.answer
    );
}

#[test]
fn commit_message_composes_the_conventional_line() {
    let answer = solved_by(
        handle_text_rewrite,
        "Write a commit message: correct an off-by-one error in the pagination helper",
    );
    assert_eq!(answer.intent, "text_transform_genre_commit_message");
    assert_eq!(answer.answer, "fix(pagination): correct off-by-one error");
    assert!(
        answer
            .answer
            .contains("fix(pagination): correct off-by-one error"),
        "the scoped Conventional Commits line must render: {}",
        answer.answer
    );
}

#[test]
fn commit_message_names_its_missing_slots() {
    let answer = solved_by(
        handle_text_rewrite,
        "Write a commit message: in the pagination helper",
    );
    assert_eq!(answer.intent, "text_transform_genre_commit_message");
    assert_eq!(
        answer.answer,
        "Not enough information to write a commit_message yet; missing: verb, effect."
    );
    assert!(
        answer.answer.contains("missing") && answer.answer.contains("verb"),
        "the missing slots must be named, not fabricated: {}",
        answer.answer
    );
}

#[test]
fn email_composes_every_declared_part() {
    let answer = solved_by(
        handle_text_rewrite,
        "Write an email to my team: I am taking a day off on Friday because I am tired.",
    );
    assert_eq!(answer.intent, "text_transform_genre_email");
    // The reason slot keeps the request's own full stop, and the seeded
    // reason frame appends another, so that line ends with "tired..".
    assert_eq!(
        answer.answer,
        "Hello team,\n\n\
         I am writing to let you know that I am taking a day off on Friday.\n\n\
         The reason is that I am tired..\n\n\
         When: Friday\n\n\
         Best regards,"
    );
    for part in [
        "Hello team,",
        "I am writing to let you know that I am taking a day off on Friday.",
        "The reason is that I am tired.",
        "When: Friday",
        "Best regards,",
    ] {
        assert!(
            answer.answer.contains(part),
            "every seeded frame must be filled: {part} in {}",
            answer.answer
        );
    }
}

#[test]
fn translation_translates_a_free_sentence_word_by_word() {
    let answer = solved_by(
        try_translation,
        "Translate to Russian: The weather is nice today, let's go for a walk.",
    );
    assert_eq!(answer.intent, "translate_en_to_ru");
    // Function words (the, is, for, a) are dropped; every content word takes
    // its seeded lemma; the sentence's capital and full stop are restored.
    assert_eq!(
        answer.answer,
        "Погода приятный сегодня давай идти прогулка."
    );
    let lower = answer.answer.to_lowercase();
    for word in ["погода", "приятный", "сегодня", "давай", "идти", "прогулка"]
    {
        assert!(
            lower.contains(word),
            "every content word must carry its seeded lemma: {word} in {}",
            answer.answer
        );
    }
    assert!(
        !answer.answer.contains("could not identify a source phrase"),
        "the sentence must be claimed, not refused: {}",
        answer.answer
    );
    assert!(
        !answer.answer.contains("kept as written"),
        "every content word resolves from seed, so no unknown-word note: {}",
        answer.answer
    );
}
