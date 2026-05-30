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

#[test]
fn explicit_reverse_sort_path_argument_programs_work_in_every_supported_prompt_language() {
    struct Case {
        name: &'static str,
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        Case {
            name: "English",
            language: "en",
            prompt:
                "Write me a Rust program that lists files with a path argument sorted in reverse order",
        },
        Case {
            name: "Russian",
            language: "ru",
            prompt: "Напиши программу на Rust которая выводит список файлов, принимает путь как аргумент и сортирует результаты в обратном порядке",
        },
        Case {
            name: "Hindi",
            language: "hi",
            prompt: "Rust में ऐसा प्रोग्राम लिखो जो फ़ाइलों की सूची दिखाए, पथ को तर्क के रूप में ले और उल्टे क्रम में क्रमबद्ध करे",
        },
        Case {
            name: "Chinese",
            language: "zh",
            prompt: "用 Rust 编写一个列出文件的程序，接受路径作为参数，并按相反顺序排序结果",
        },
    ];

    for case in cases {
        let response = answer(case.prompt);
        assert_write_program_parameters(&response, "rust", "list_files_arg_reverse_sort");
        assert!(
            response.answer.contains("env::args"),
            "{} ({}) should use a path argument, got: {}",
            case.name,
            case.language,
            response.answer
        );
        assert!(
            response.answer.contains("names.sort_by(|a, b| b.cmp(a))"),
            "{} ({}) should sort file names in reverse order, got: {}",
            case.name,
            case.language,
            response.answer
        );
    }
}
