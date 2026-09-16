//! Issue #1138 B8 (plan 08, L2, L5): recognition is seed data, in five languages.
//!
//! Every expectation kind must be recognised from the cues the seed declares,
//! in all five languages, and an open-ended request must *not* be recognised —
//! the new route is strictly additive and leaves every existing route untouched.
//! No English literal may appear in the recognizer itself.

use std::fs;

use formal_ai::verifiable_task::{AnswerShape, TaskExpectation, recognise_verifiable};
use walkdir::WalkDir;

use super::{LANGUAGES, case, repo_root};

fn expected_kind(slug: &str, task: &TaskExpectation) -> bool {
    match slug {
        "numeric" | "numeric_with_unit" => matches!(task, TaskExpectation::Numeric { .. }),
        "count" => matches!(task, TaskExpectation::Count { .. }),
        "edited_text" => matches!(task, TaskExpectation::EditedText { .. }),
        "unknown" => matches!(task, TaskExpectation::Unknown { .. }),
        other => panic!("the corpus declares an expectation this test does not know: {other}"),
    }
}

fn expected_shape(slug: &str) -> AnswerShape {
    match slug {
        "trailing_number" => AnswerShape::TrailingNumber,
        "delimited_value" => AnswerShape::DelimitedValue,
        "whole_body" => AnswerShape::WholeBody,
        other => panic!("the corpus declares a shape this test does not know: {other}"),
    }
}

/// Each of the four expectation kinds is recognised in each of the five
/// languages, with the answer shape the corpus declares.
#[test]
fn each_expectation_kind_is_recognised_in_five_languages() {
    for family in [
        "arithmetic_narrative",
        "counted_category",
        "instructed_edit",
        "named_unknown",
        "unit_conversion",
    ] {
        for language in LANGUAGES {
            let paraphrase = case(family, language);
            let task = recognise_verifiable(&paraphrase.prompt).unwrap_or_else(|| {
                panic!("{family}/{language}: {:?} must be recognised", paraphrase.prompt)
            });
            assert!(
                expected_kind(&paraphrase.expectation, &task.expectation),
                "{family}/{language}: expected {}, recognised {:?}",
                paraphrase.expectation,
                task.expectation
            );
            assert_eq!(
                task.shape,
                expected_shape(&paraphrase.shape),
                "{family}/{language}: the answer shape must be the one the corpus declares"
            );
            assert_eq!(
                task.prose_language, *language,
                "{family}: the detected prose language must be the corpus language"
            );
            assert_eq!(
                task.prompt, paraphrase.prompt,
                "{family}/{language}: the prompt is carried unmodified"
            );
        }
    }
}

/// An open-ended request carries no checkable expectation. Recognition returns
/// nothing and every existing route is untouched.
#[test]
fn an_open_ended_request_is_not_a_verifiable_task() {
    for prompt in [
        "Tell me about the history of the Silk Road.",
        "Расскажи о том, как работает твоя память.",
        "अपने बारे में कुछ बताओ।",
        "聊聊你对音乐的看法。",
        "Cuéntame algo interesante sobre el océano.",
    ] {
        assert!(
            recognise_verifiable(prompt).is_none(),
            "an open-ended request is not a verifiable task: {prompt:?}"
        );
    }
}

/// The recognizer's cues live in seed. A quoted natural-language sentence inside
/// `src/verifiable_task*` would be a memorized surface, not a recognizer.
#[test]
fn recognition_names_no_english_literal() {
    let mut offenders: Vec<String> = Vec::new();
    for entry in WalkDir::new(repo_root().join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let relative = path
            .strip_prefix(repo_root())
            .unwrap_or(path)
            .display()
            .to_string()
            .replace('\\', "/");
        if !relative.starts_with("src/verifiable_task") {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or(line);
            // A wave-T `todo!("plan 08 leaf …")` marker is a leaf reference, not
            // a user-facing cue; it disappears with the body it stands in for.
            if code.contains("todo!") {
                continue;
            }
            let Some(start) = code.find('"') else {
                continue;
            };
            let Some(end) = code[start + 1..].find('"') else {
                continue;
            };
            let literal = &code[start + 1..start + 1 + end];
            let words = literal
                .split_whitespace()
                .filter(|word| word.chars().all(|c| c.is_ascii_alphabetic()) && word.len() >= 2)
                .count();
            if words >= 2 {
                offenders.push(format!("{relative}:{}: {literal}", index + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "the recognizer's cues belong in seed, not in Rust: {offenders:?}"
    );
}
