use formal_ai::{ConversationTurn, SolverConfig, UniversalSolver};

const FIRST_PROMPT: &str =
    "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
const PATH_ARGUMENT_PROMPT: &str = "Сделай так, чтобы программа принимала путь как аргумент";
const REVERSE_SORT_PROMPT: &str = "Сделай сортировку результатов в обратном порядке";

fn issue_349_history(solver: &UniversalSolver) -> [ConversationTurn; 4] {
    let first = solver.solve(FIRST_PROMPT);
    assert_eq!(
        first.intent, "write_program",
        "issue #349 setup should start with a Rust list-files program, got: {}",
        first.intent
    );

    let first_history = [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer.clone()),
    ];
    let path_argument = solver.solve_with_history(PATH_ARGUMENT_PROMPT, &first_history);
    assert_eq!(
        path_argument.intent, "write_program",
        "issue #349 setup should keep the path-argument follow-up working, got: {}",
        path_argument.intent
    );

    [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer),
        ConversationTurn::user(PATH_ARGUMENT_PROMPT),
        ConversationTurn::assistant(path_argument.answer),
    ]
}

#[test]
fn issue_349_reverse_sort_follow_up_must_not_be_unknown() {
    let solver = UniversalSolver::default();
    let history = issue_349_history(&solver);
    let response = solver.solve_with_history(REVERSE_SORT_PROMPT, &history);

    assert_ne!(
        response.intent, "unknown",
        "turn 5 must route to a real program-modification answer, got: {}",
        response.answer
    );
    assert!(
        answer_reverses_sort(&response.answer),
        "turn 5 answer must reverse the file-name sort order, got: {}",
        response.answer
    );
}

#[test]
fn issue_349_diagnostic_mode_emits_full_turn_5_reasoning_chain() {
    let setup_solver = UniversalSolver::default();
    let history = issue_349_history(&setup_solver);
    let diagnostic_solver = UniversalSolver::new(SolverConfig {
        diagnostic_mode: true,
        ..SolverConfig::default()
    });
    let response = diagnostic_solver.solve_with_history(REVERSE_SORT_PROMPT, &history);

    assert_eq!(response.intent, "write_program");
    assert!(
        response.answer.contains("[diagnostic]"),
        "diagnostic mode should append an inspectable trace block, got: {}",
        response.answer
    );
    for expected in [
        "selected_rule initial unknown reason no_seed_route next try_rule_synthesis",
        "write_program_coreference_rewrite",
        "referent=active_program_artifact task=list_files_arg language=rust",
        "rule_synthesis_operation_vocabulary",
        "reverse_sort",
        "rule_synthesis_request",
        "rule_synthesis_candidate",
        "rule_verification",
        "status passed",
        "write_program_plan",
        "program_parameter:task list_files_arg_reverse_sort",
    ] {
        assert!(
            response.answer.contains(expected),
            "diagnostic trace should include `{expected}`, got: {}",
            response.answer
        );
    }
}

#[test]
fn issue_360_diagnostic_mode_marks_program_traces_for_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let solver = UniversalSolver::new(SolverConfig {
        diagnostic_mode: true,
        ..SolverConfig::default()
    });
    for case in [
        Case {
            language: "en",
            prompt:
                "Write me a Rust program that lists files with a path argument sorted in reverse order",
        },
        Case {
            language: "ru",
            prompt: "Напиши программу на Rust которая выводит список файлов, принимает путь как аргумент и сортирует результаты в обратном порядке",
        },
        Case {
            language: "hi",
            prompt: "Rust में ऐसा प्रोग्राम लिखो जो फ़ाइलों की सूची दिखाए, पथ को तर्क के रूप में ले और उल्टे क्रम में क्रमबद्ध करे",
        },
        Case {
            language: "zh",
            prompt: "用 Rust 编写一个列出文件的程序，接受路径作为参数，并按相反顺序排序结果",
        },
    ] {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "write_program",
            "{} diagnostic prompt should route to write_program, got: {}",
            case.language, response.intent
        );
        assert!(
            response.answer.contains("[diagnostic]"),
            "{} diagnostic program answer should expose a trace block, got: {}",
            case.language,
            response.answer
        );
    }
}

fn answer_reverses_sort(answer: &str) -> bool {
    let compact = answer
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<String>();

    compact.contains("names.reverse()")
        || compact.contains("names.sort_by(")
        || compact.contains(".rev()")
        || compact.contains("reverse(")
}
