use super::*;

pub(super) fn the_algorithm_rule_set_answers_unchanged_in_every_prompt_language() {
    let english = FormalAiEngine.answer("Write me a sorting algorithm in Rust");
    assert_eq!(english.intent, "algorithm_sort_rust");
    assert_eq!(english.answer, SORT_RUST);
    for (prompt, intent, expected) in [
        (
            "write a sorting algorithm in python with tests",
            "algorithm_sort_python",
            SORT_PYTHON_TESTS,
        ),
        (
            "Напиши сортировку списка на go с тестом",
            "algorithm_sort_go",
            SORT_GO,
        ),
        (
            "Rust में सॉर्टिंग एल्गोरिदम लिखो",
            "algorithm_sort_rust",
            SORT_RUST_HI,
        ),
        (
            "写一个 typescript 排序 algorithm 带 test",
            "algorithm_sort_typescript",
            SORT_TYPESCRIPT_TESTS,
        ),
    ] {
        let mut log = EventLog::default();
        let answer = run_handler("algorithm", prompt, &prompt.to_lowercase(), &mut log)
            .unwrap_or_else(|| panic!("{prompt} is answered by the algorithm rules"));
        assert_eq!(answer.intent, intent, "{prompt}");
        assert_eq!(answer.answer, expected, "{prompt}");
        assert!(
            log.events()
                .iter()
                .any(|event| event.kind == "execution_status" && event.payload == "unavailable"),
            "{prompt} records the execution status"
        );
        assert!(
            !log.events()
                .iter()
                .any(|event| event.kind == "algorithm:refusal"),
            "{prompt} names the sort operation"
        );
    }
    for prompt in ["explain the algorithm", "write an algorithm with tests"] {
        let mut log = EventLog::default();
        let unnamed = run_handler("algorithm", prompt, prompt, &mut log)
            .expect("an algorithm request without an operation asks for the operation");
        assert_eq!(unnamed.intent, "algorithm_operation_required");
        assert_eq!(
            unnamed.answer,
            "What should the algorithm do? Include its expected inputs and outputs."
        );
        assert!(log.events().iter().any(|event| {
            event.kind == "algorithm:refusal" && event.payload == "no operation named"
        }));
    }
    let mut log = EventLog::default();
    assert!(
        run_handler(
            "algorithm",
            "What is the capital of France?",
            "what is the capital of france?",
            &mut log
        )
        .is_none()
    );
}
