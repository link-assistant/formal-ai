//! Program-modifier regressions for `write_program`.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn assert_write_program_parameters(response: &SymbolicAnswer, language: &str, task: &str) {
    assert_eq!(response.intent, "write_program");
    assert!(
        response
            .links_notation
            .contains(&format!("program_parameter:language {language}")),
        "Links Notation trace should include language={language}, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains(&format!("program_parameter:task {task}")),
        "Links Notation trace should include task={task}, got: {}",
        response.links_notation
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| { link == &format!("response:write_program:{task}:{language}") }),
        "evidence links should include the parameterized response link, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn explicit_list_files_reverse_sort_modifiers_compose() {
    let response = answer(
        "Write me a Rust program that lists files with a path argument sorted in reverse order",
    );

    assert_write_program_parameters(&response, "rust", "list_files_arg_reverse_sort");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("env::args"));
    assert!(
        response.answer.contains("names.sort_by(|a, b| b.cmp(a))"),
        "reverse-sort template should sort descending, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("Output:\n```text\nmain.rs\nREADME.md\nCargo.toml"),
        "reverse-sort template should publish descending sample output, got: {}",
        response.answer
    );
}
