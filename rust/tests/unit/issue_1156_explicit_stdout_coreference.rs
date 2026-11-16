//! Issue #1156: a Hello World request names one stdout line several times, and
//! the delivered program printed it twice. The requirement, not the
//! wording, decides how many lines the program writes:
//!
//! * references to one literal are one output obligation (coreference),
//! * distinct literals become lines in request order,
//! * only an explicit count ("twice", "three times", "дважды") or a
//!   trailing "again" repeats a line (issue #1162 acceptance item 5),
//! * a file name is never taken as the output (`Main.java` printing
//!   "Main.kt"),
//! * the verify script compares against the expected text derived from the
//!   request, and Kotlin, Scala and Rust each print the line once.

use formal_ai::EventLog;
use formal_ai::intent_formalization::bound_output_literals;
use formal_ai::program_contract::{answer, explicit_stdout};

const KOTLIN_ISSUE: &str = include_str!("../fixtures/issue-1162/hello-world-kotlin-issue.md");

/// The delivered recipe, flattened: the execution recipe type is reachable
/// only through `SymbolicAnswer`, so the test reads its public fields.
struct Recipe {
    language: String,
    path: String,
    source: String,
    supporting_files: Vec<(String, String)>,
}

fn recipe(prompt: &str) -> Recipe {
    let mut log = EventLog::new();
    let recipe = answer(prompt, &mut log)
        .and_then(|answer| answer.execution_recipe)
        .unwrap_or_else(|| panic!("no execution recipe for {prompt:?}"));
    Recipe {
        language: recipe.language.clone(),
        path: recipe.path.clone(),
        source: recipe.source.clone(),
        supporting_files: recipe
            .supporting_files
            .iter()
            .map(|file| (file.path.clone(), file.source.clone()))
            .collect(),
    }
}

#[test]
fn the_issue_body_names_one_line_three_times_and_yields_one_line() {
    assert_eq!(bound_output_literals(KOTLIN_ISSUE), vec!["Hello, World!"]);
    assert_eq!(
        explicit_stdout(KOTLIN_ISSUE).as_deref(),
        Some("Hello, World!")
    );
}

#[test]
fn repeated_references_and_check_again_wording_stay_one_line() {
    for prompt in [
        "Write a Kotlin program that prints `Hello, World!`. The program must print \"Hello, World!\".",
        "Write a Kotlin program that prints `Hello, World!`. Check again that it prints `Hello, World!`.",
        "Write a Kotlin program that prints `Hello, World!` and test it against the expected output.",
    ] {
        assert_eq!(
            bound_output_literals(prompt),
            vec!["Hello, World!"],
            "{prompt}"
        );
    }
}

#[test]
fn only_sequenced_distinct_literals_form_several_lines() {
    assert_eq!(
        explicit_stdout("Write a Kotlin program that first prints `A` and then prints `B`.")
            .as_deref(),
        Some("A\nB")
    );
}

#[test]
fn an_explicit_count_or_again_repeats_the_line() {
    for (prompt, lines) in [
        (
            "Write a Kotlin program that prints `Hello, World!` twice.",
            2,
        ),
        (
            "Write a Kotlin program that prints `Hello, World!` three times.",
            3,
        ),
        (
            "Write a Kotlin program that first prints `Hello, World!` and then prints `Hello, World!` again.",
            2,
        ),
        (
            "Напиши программу на Kotlin, которая выводит `Привет` дважды.",
            2,
        ),
    ] {
        let outputs = bound_output_literals(prompt);
        assert_eq!(outputs.len(), lines, "{prompt}");
        assert!(
            outputs.windows(2).all(|pair| pair[0] == pair[1]),
            "{prompt}"
        );
    }
}

#[test]
fn a_paraphrase_with_outputs_is_an_output_obligation() {
    let prompt = "Write a Kotlin program that outputs exactly `Hello, World!` and add a GitHub Actions workflow that runs it.";
    assert_eq!(bound_output_literals(prompt), vec!["Hello, World!"]);
    assert!(recipe(prompt).source.contains("println(\"Hello, World!\")"));
}

#[test]
fn a_file_name_is_never_the_output() {
    for prompt in [
        "Create `Main.kt` with a Kotlin program that prints `Hello, World!`.",
        "Write a Kotlin program in `Main.kt` that prints exactly: `Hello, World!`",
    ] {
        assert_eq!(
            bound_output_literals(prompt),
            vec!["Hello, World!"],
            "{prompt}"
        );
        let recipe = recipe(prompt);
        assert_eq!(recipe.language, "kotlin", "{prompt}");
        assert!(!recipe.source.contains("println(\"Main.kt\")"), "{prompt}");
        assert!(!recipe.path.ends_with(".java"), "{prompt}");
    }
}

#[test]
fn kotlin_scala_and_rust_print_once_and_verify_the_derived_line() {
    for (language, path, line) in [
        ("Kotlin", "Main.kt", "println(\"Hello, World!\")"),
        ("Scala", "Main.scala", "println(\"Hello, World!\")"),
        ("Rust", "main.rs", "println!(\"{}\", \"Hello, World!\");"),
    ] {
        let recipe = recipe(&KOTLIN_ISSUE.replace("Kotlin", language));
        assert_eq!(recipe.path, path, "{language}");
        assert_eq!(
            recipe.source.matches(line).count(),
            1,
            "{language}: {}",
            recipe.source
        );
        let verify = recipe
            .supporting_files
            .iter()
            .find(|(path, _)| path == "tests/verify-output.sh")
            .map(|(_, source)| source)
            .unwrap_or_else(|| panic!("{language}: no verify script"));
        assert!(
            verify.contains("printf '%s\\n' 'Hello, World!' > \"$verification_dir/expected\""),
            "{language}: {verify}"
        );
    }
}
