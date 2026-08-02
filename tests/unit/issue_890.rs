//! Issue #890: a formal proof must be reusable independently of its prose
//! presentation and translated through the general program-translation path.

use std::collections::BTreeSet;
use std::fs;
use std::process::Command;

use formal_ai::{FormalAiEngine, SymbolicAnswer};

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
    let stem = format!("formal-ai-issue-890-{}", std::process::id());
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
    let source = std::env::temp_dir().join(format!(
        "formal-ai-issue-890-{}.py",
        std::process::id()
    ));
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

#[test]
fn solved_interval_proof_translates_to_two_executable_programs() {
    let solved = answer("Я загадал число больше 1 но меньше 3. что это за число?");
    assert_eq!(solved.intent, "number_constraint_reasoning");
    let statement = inline_proof_statement(&solved.answer);

    let rust = answer(&format!("Translate `{statement}` to Rust"));
    let python = answer(&format!("Translate `{statement}` to Python"));

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
    assert_eq!(covered, supported, "fixture must grow with the language registry");

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
