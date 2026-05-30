//! Coreference recovery for program-generation follow-ups.

use formal_ai::{ConversationTurn, SymbolicAnswer, UniversalSolver};

struct ProgramFollowUpCase {
    name: &'static str,
    initial: &'static str,
    path_argument_follow_up: &'static str,
    bare_results_follow_up: &'static str,
}

fn solve_bare_results_follow_up(
    solver: &UniversalSolver,
    case: &ProgramFollowUpCase,
) -> SymbolicAnswer {
    let initial = solver.solve(case.initial);
    assert_eq!(
        initial.intent, "write_program",
        "{} setup should create a program, got: {}",
        case.name, initial.intent
    );

    let path_argument_history = [
        ConversationTurn::user(case.initial),
        ConversationTurn::assistant(initial.answer.clone()),
    ];
    let path_argument =
        solver.solve_with_history(case.path_argument_follow_up, &path_argument_history);
    assert_eq!(
        path_argument.intent, "write_program",
        "{} setup should recover the path-argument modification, got: {}",
        case.name, path_argument.intent
    );
    assert!(
        path_argument
            .links_notation
            .contains("program_parameter:task list_files_arg"),
        "{} setup should resolve to list_files_arg, got: {}",
        case.name,
        path_argument.links_notation
    );

    let full_history = [
        ConversationTurn::user(case.initial),
        ConversationTurn::assistant(initial.answer),
        ConversationTurn::user(case.path_argument_follow_up),
        ConversationTurn::assistant(path_argument.answer),
    ];
    solver.solve_with_history(case.bare_results_follow_up, &full_history)
}

#[test]
fn bare_imperative_results_follow_ups_bind_active_program_in_all_languages() {
    let solver = UniversalSolver::default();
    let cases = [
        ProgramFollowUpCase {
            name: "English",
            initial: "Write me a Rust program that lists the files in the current directory",
            path_argument_follow_up: "Make the program accept a path as an argument",
            bare_results_follow_up: "Sort the results in reverse order",
        },
        ProgramFollowUpCase {
            name: "Russian",
            initial:
                "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории",
            path_argument_follow_up: "Сделай так, чтобы программа принимала путь как аргумент",
            bare_results_follow_up: "Сделай сортировку результатов в обратном порядке",
        },
        ProgramFollowUpCase {
            name: "Hindi",
            initial: "Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो",
            path_argument_follow_up: "इसे ऐसा बनाओ कि प्रोग्राम पथ को तर्क के रूप में स्वीकार करे",
            bare_results_follow_up: "परिणामों को उल्टे क्रम में क्रमबद्ध करो",
        },
        ProgramFollowUpCase {
            name: "Chinese",
            initial: "用 Rust 编写一个列出当前目录中文件的程序",
            path_argument_follow_up: "制作程序，使其接受路径作为参数",
            bare_results_follow_up: "把结果按相反顺序排序",
        },
    ];

    for case in cases {
        let response = solve_bare_results_follow_up(&solver, &case);
        assert_eq!(
            response.intent, "write_program",
            "{} bare follow-up should route back to write_program, got: {}",
            case.name, response.intent
        );
        assert!(
            response
                .links_notation
                .contains("program_parameter:task list_files_arg"),
            "{} bare follow-up should bind the active program task, got: {}",
            case.name,
            response.links_notation
        );
        assert!(
            response
                .links_notation
                .contains("program_parameter:language rust"),
            "{} bare follow-up should reuse the active Rust program, got: {}",
            case.name,
            response.links_notation
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("write_program_coreference_rewrite:")),
            "{} bare follow-up should expose the coreference rewrite, got: {:?}",
            case.name,
            response.evidence_links
        );
    }
}

#[test]
fn bare_results_follow_up_without_active_program_stays_out_of_write_program() {
    let response = UniversalSolver::default().solve("Sort the results in reverse order");

    assert_ne!(
        response.intent, "write_program",
        "without an active program artifact, bare results should not invent a write_program target"
    );
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("write_program_coreference_rewrite:")),
        "standalone prompt should not log a program coreference rewrite, got: {:?}",
        response.evidence_links
    );
}
