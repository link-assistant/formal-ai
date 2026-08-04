//! Issue #906: the language router took the word after "in" as the target
//! programming language.
//!
//! The reporter's four cases, verbatim:
//!
//! 1. "Create a file named hello.txt **in the current directory** …" routed as
//!    a request in language `the`.
//! 2. A request that named no language routed as language `missing` — the
//!    internal sentinel — and the sentinel reached the reply ("Teach me the
//!    missing idiom for `missing`").
//! 3. "Fix the failing CI job **in Rust**." came back as an encyclopedia
//!    definition of Rust: the modifier replaced the task instead of modifying
//!    it.
//! 4. "Create a file named hello.txt containing Hello World, in JavaScript."
//!    came back as `console.log('Hello, world!')` — the path and the content
//!    were discarded.
//!
//! The issue asks for a table-driven corpus: "~20 prompts × (expected
//! language, expected task), including the negative cases `the`, `the current
//! directory`, and no-language-at-all". That corpus is [`CORPUS`] below. It is
//! read at the level the defect lives at — what may fill the "in <language>"
//! position — so it stays fast enough to run on every change; the end-to-end
//! consequences are asserted separately underneath it.

use formal_ai::implementation_language::{
    is_known, requested_in, without_modifier, without_trailing_known_modifier,
};
use formal_ai::{FormalAiEngine, SymbolicAnswer};

/// A prompt and the implementation language it does — or does not — name.
struct Case {
    prompt: &'static str,
    language: Option<&'static str>,
}

const fn case(prompt: &'static str, language: Option<&'static str>) -> Case {
    Case { prompt, language }
}

/// The regression corpus requested by issue #906.
///
/// Positive rows pin the languages that must keep resolving (known names,
/// aliases, transliterations, and an *unknown* name, which must still be read
/// so the engine can report what it was asked for). Negative rows pin the
/// defect: a closed-class word never names a language.
const CORPUS: &[Case] = &[
    // --- known languages, in every request language -------------------------
    case("Write me hello world program in Rust", Some("rust")),
    case("hello world in python", Some("python")),
    case("write a hello world program in JavaScript", Some("javascript")),
    case("hello world in py", Some("python")),
    case("hello world in golang", Some("go")),
    case("hello world in node", Some("javascript")),
    case("напиши программу hello world на python", Some("python")),
    case("Напиши хелло ворлд на питоне", Some("python")),
    case("count to three in rust", Some("rust")),
    // --- languages we do not catalogue, but must still read ------------------
    case("hello world in elvish", Some("elvish")),
    case("hello world in the elvish language", Some("elvish")),
    case("write a hello world program in language elvish", Some("elvish")),
    // --- negatives: `the`, `the current directory`, no language at all -------
    case(
        "Create a file named hello.txt in the current directory whose entire content is the single line: Hello World.",
        None,
    ),
    case("Fix the failing CI job in the current directory.", None),
    case("run the tests in the background", None),
    case("Write a program that prints hello world.", None),
    case("write a program", None),
    case("hello world", None),
    // --- negatives: the position is filled by something that names no language
    case("hello world in 3 steps", None),
    case("print the numbers in reverse order", None),
];

#[test]
fn corpus_resolves_exactly_the_languages_the_prompts_name() {
    let mut wrong = Vec::new();
    for Case { prompt, language } in CORPUS {
        let resolved = requested_in(prompt);
        if resolved.as_deref() != *language {
            wrong.push(format!(
                "{prompt:?}: expected {language:?}, got {resolved:?}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "language router regressions:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn a_closed_class_word_is_never_read_as_a_language() {
    // The reported defect, stated directly: "in the current directory" names a
    // place, not a programming language.
    assert_eq!(requested_in("in the current directory"), None);
    assert_eq!(requested_in("in the background"), None);
    assert!(!is_known("the"));
}

#[test]
fn an_unknown_name_is_still_read_so_the_request_can_be_reported() {
    // Refusing every name outside the catalogue would make the engine unable to
    // say what it was asked for; the fix validates the *class* of word, not
    // membership in the catalogue.
    assert_eq!(
        requested_in("hello world in elvish").as_deref(),
        Some("elvish")
    );
    assert!(!is_known("elvish"));
}

#[test]
fn the_modifier_modifies_the_request_instead_of_replacing_it() {
    // Issue #906 case 3: the topic of "Fix the failing CI job in Rust." is the
    // CI job. Stripping the modifier is what stops the unknown-prompt reasoner
    // from answering with a definition of the language.
    assert_eq!(
        without_modifier("Fix the failing CI job in Rust.").as_deref(),
        Some("Fix the failing CI job")
    );
    // No modifier, nothing removed.
    assert_eq!(without_modifier("Fix the failing CI job"), None);
}

#[test]
fn only_a_trailing_known_language_is_stripped_from_recovered_content() {
    // Issue #906 case 4: "…containing Hello World, in JavaScript." names the
    // bytes *and* the language; only the bytes belong in the file.
    assert_eq!(
        without_trailing_known_modifier("Hello World, in JavaScript").as_deref(),
        Some("Hello World")
    );
    // A payload that merely mentions a place keeps every word.
    assert_eq!(without_trailing_known_modifier("Meet me in Paris"), None);
    // A modifier that is not at the end is not content punctuation either.
    assert_eq!(
        without_trailing_known_modifier("in Rust, fix the failing CI job"),
        None
    );
}

// ---------------------------------------------------------------------------
// End-to-end consequences: what the requester actually reads.
// ---------------------------------------------------------------------------

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn a_request_without_a_language_asks_which_language_instead_of_naming_a_gap() {
    // Issue #906 case 2: an unfilled parameter is not a skill gap, and the
    // `missing` sentinel must never reach the requester.
    let response = answer("Write a program that prints hello world.");
    assert_eq!(response.intent, "write_program_language_unspecified");
    assert!(
        !response.answer.contains("missing"),
        "the internal sentinel reached the reply: {}",
        response.answer
    );
    assert!(response.answer.contains("hello_world"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "response:write_program:language_unspecified"));
}

#[test]
fn a_request_naming_neither_task_nor_language_says_so() {
    let response = answer("write a program");
    assert_eq!(response.intent, "write_program_request_unspecified");
    assert!(!response.answer.contains("missing"), "{}", response.answer);
}

#[test]
fn a_named_language_with_no_derivable_program_still_names_the_skill_gap() {
    // The pre-existing honest-failure path (issue #699) is unchanged: when both
    // parameters are known, the miss is a genuine gap in what we can synthesize.
    let response = answer("hello world in elvish");
    assert_eq!(response.intent, "write_program_skill_gap");
    assert!(response.answer.contains("elvish"));
    assert!(response.answer.contains("hello_world"));
    assert!(!response.answer.contains("missing idiom for `missing`"));
}

#[test]
fn a_file_request_is_not_answered_with_a_program_that_was_never_asked_for() {
    // Issue #906 case 4: the task alias table matched only because the file's
    // *content* reads "hello world". Answering with `console.log(...)` discards
    // the path and the content and reports a program nobody requested.
    let response = answer("Create a file named hello.txt containing Hello World, in JavaScript.");
    assert_ne!(response.intent, "write_program");
    assert!(
        !response.answer.contains("console.log"),
        "a file request was answered with a fabricated program: {}",
        response.answer
    );
}
