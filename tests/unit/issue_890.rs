//! Issue #890: a formal proof must be reusable independently of its prose
//! presentation and translated through the general program-translation path.

use std::collections::BTreeSet;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::proof_program::FormalProof;
use formal_ai::{FormalAiEngine, SymbolicAnswer};

static NEXT_ARTIFACT: AtomicUsize = AtomicUsize::new(0);

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn inline_proof_statement(answer: &str) -> &str {
    answer
        .split('`')
        .nth(1)
        .unwrap_or_else(|| panic!("expected an inline formal statement, got: {answer}"))
}

fn fenced_program<'a>(answer: &'a str, language: &str) -> &'a str {
    let marker = format!("```{language}\n");
    answer
        .split_once(&marker)
        .and_then(|(_, tail)| tail.split_once("\n```").map(|(program, _)| program))
        .unwrap_or_else(|| panic!("expected a {language} program, got: {answer}"))
}

fn meaning_link(response: &SymbolicAnswer) -> &str {
    response
        .evidence_links
        .iter()
        .find(|link| link.starts_with("meaning:"))
        .unwrap_or_else(|| panic!("expected a proof meaning link, got: {response:?}"))
}

fn execute_rust(program: &str) -> String {
    let stem = artifact_stem();
    let source = std::env::temp_dir().join(format!("{stem}.rs"));
    let binary = std::env::temp_dir().join(stem);
    fs::write(&source, program).expect("write generated Rust proof");
    let compiled = Command::new("rustc")
        .args(["--edition=2021", "-o"])
        .arg(&binary)
        .arg(&source)
        .output()
        .expect("run rustc");
    assert!(
        compiled.status.success(),
        "generated Rust proof must compile: {}\n{program}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = Command::new(&binary)
        .output()
        .expect("run generated Rust proof");
    let _ = fs::remove_file(source);
    let _ = fs::remove_file(binary);
    assert!(executed.status.success(), "generated Rust proof must run");
    String::from_utf8(executed.stdout).expect("Rust output is UTF-8")
}

fn execute_python(program: &str) -> String {
    let source = std::env::temp_dir().join(format!("{}.py", artifact_stem()));
    fs::write(&source, program).expect("write generated Python proof");
    let executed = Command::new("python3")
        .arg(&source)
        .output()
        .expect("run generated Python proof");
    let _ = fs::remove_file(source);
    assert!(
        executed.status.success(),
        "generated Python proof must run: {}\n{program}",
        String::from_utf8_lossy(&executed.stderr)
    );
    String::from_utf8(executed.stdout).expect("Python output is UTF-8")
}

fn artifact_stem() -> String {
    format!(
        "formal-ai-issue-890-{}-{}",
        std::process::id(),
        NEXT_ARTIFACT.fetch_add(1, Ordering::Relaxed)
    )
}

fn translate_solved_interval() -> (SymbolicAnswer, SymbolicAnswer) {
    let solved = answer("Я загадал число больше 1 но меньше 3. что это за число?");
    assert_eq!(solved.intent, "number_constraint_reasoning");
    let statement = inline_proof_statement(&solved.answer);
    (
        answer(&format!("Translate `{statement}` to Rust")),
        answer(&format!("Translate `{statement}` to Python")),
    )
}

#[test]
fn proof_meaning_is_independent_from_its_programming_language_presentations() {
    let proof =
        FormalProof::integer_interval("x", 1, false, 3, false).expect("valid interval proof");
    let canonical = proof.statement();
    assert_eq!(FormalProof::from_statement(&canonical), Some(proof.clone()));
    assert!(proof.render_program("rust").is_some());
    assert!(proof.render_program("python").is_some());
    assert_eq!(
        proof.slug(),
        FormalProof::from_statement(&canonical).unwrap().slug()
    );
}

#[test]
fn same_solved_proof_uses_general_translation_path_for_two_targets() {
    let (rust, python) = translate_solved_interval();
    assert_eq!(rust.intent, "translate_proof_to_rust", "{}", rust.answer);
    assert_eq!(
        python.intent, "translate_proof_to_python",
        "{}",
        python.answer
    );
    assert_eq!(
        meaning_link(&rust),
        meaning_link(&python),
        "surface languages must share one language-neutral proof meaning"
    );
    assert!(rust.answer.contains("fn main()"));
    assert!(python.answer.contains("assert x > 1 and x < 3"));
}

#[test]
fn generated_proof_programs_compile_and_execute() {
    let proof =
        FormalProof::integer_interval("x", 1, false, 3, false).expect("valid interval proof");
    let rust = proof.render_program("rust").expect("Rust proof renderer");
    let python = proof
        .render_program("python")
        .expect("Python proof renderer");
    assert_eq!(execute_rust(&rust), "2\n");
    assert_eq!(execute_python(&python), "2\n");

    let edge = FormalProof::integer_interval("x", i64::MAX, false, i64::MAX, true)
        .expect("represent an interval beyond the largest integer");
    assert!(!edge.is_satisfiable());
    assert_eq!(
        FormalProof::from_statement(&edge.statement()),
        Some(edge.clone())
    );
    assert_eq!(
        execute_rust(&edge.render_program("rust").expect("Rust edge proof")),
        "unsatisfiable\n"
    );
    assert_eq!(
        execute_python(&edge.render_program("python").expect("Python edge proof")),
        "unsatisfiable\n"
    );
}

#[test]
fn solved_interval_preserves_exclusive_bound_at_i64_max() {
    let response = answer(
        "I chose a number greater than 9223372036854775807 and less than or equal to \
         9223372036854775807. What is the number?",
    );
    assert_eq!(response.intent, "number_constraint_reasoning");
    assert!(
        response.answer.contains("there is no solution"),
        "exclusive i64::MAX must not saturate into a witness: {}",
        response.answer
    );
    assert!(
        response.answer.contains(
            "x > 9223372036854775807 and x <= 9223372036854775807 is unsatisfiable over integers"
        ),
        "the answer must expose the same unsatisfiable proof meaning: {}",
        response.answer
    );
}

#[test]
fn whole_issue_890_workflow_solves_translates_and_executes() {
    let (rust, python) = translate_solved_interval();
    assert_eq!(meaning_link(&rust), meaning_link(&python));
    assert_eq!(execute_rust(fenced_program(&rust.answer, "rust")), "2\n");
    assert_eq!(
        execute_python(fenced_program(&python.answer, "python")),
        "2\n"
    );
}

#[test]
fn every_registered_natural_language_can_request_proof_translation() {
    let statement = "x > 1 and x < 3 is satisfiable";
    let prompts = [
        ("en", format!("Translate `{statement}` to Rust")),
        ("ru", format!("Переведи `{statement}` на Раст")),
        ("hi", format!("`{statement}` का रस्ट में अनुवाद करो")),
        ("zh", format!("把`{statement}`翻译成Rust")),
    ];
    let covered = prompts
        .iter()
        .map(|(language, _)| (*language).to_owned())
        .collect::<BTreeSet<_>>();
    let supported = formal_ai::supported_languages()
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        covered, supported,
        "fixture must grow with the language registry"
    );

    let mut shared_meaning = None;
    for (language, prompt) in prompts {
        let response = answer(&prompt);
        assert_eq!(
            response.intent, "translate_proof_to_rust",
            "{language} request did not use proof translation: {}",
            response.answer
        );
        assert!(
            response.answer.contains("fn main()"),
            "{language} request did not return executable Rust: {}",
            response.answer
        );
        let meaning = meaning_link(&response).to_owned();
        if let Some(expected) = &shared_meaning {
            assert_eq!(&meaning, expected, "{language} changed the proof meaning");
        } else {
            shared_meaning = Some(meaning);
        }
    }
}
