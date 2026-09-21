//! Issue #386: after building a reverse-sorted, path-argument file lister, the
//! follow-up "Отмени сортировку" ("cancel the sorting") returned
//! `intent: unknown` instead of removing the sort. The fix makes "cancel X" the
//! data-derived inverse of "X": `cancel_reverse_sort` declares
//! `inverse "reverse_sort"` in the seed, and the subtractive substitution rules
//! are derived at runtime. These tests pin the behavior — the four-turn Russian
//! reproduction, the full reasoning chain, and every supported language.

use formal_ai::{ConversationTurn, SolverConfig, UniversalSolver};

const FIRST_PROMPT: &str =
    "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории";
const PATH_ARGUMENT_PROMPT: &str = "Сделай так, чтобы программа принимала путь как аргумент";
const REVERSE_SORT_PROMPT: &str = "Сделай сортировку результатов в обратном порядке";
const CANCEL_SORT_PROMPT: &str = "Отмени сортировку";

/// Replay the conversation up to (but not including) the cancel follow-up:
/// `list_files` → `+path_argument` → `+reverse_sort`. Each turn's rendered
/// answer is threaded back as the assistant turn, exactly how the demo
/// accumulates program state across a dialog.
fn issue_386_history(solver: &UniversalSolver) -> Vec<ConversationTurn> {
    let first = solver.solve(FIRST_PROMPT);
    assert_eq!(
        first.intent, "write_program",
        "issue #386 setup should start with a Rust list-files program, got: {}",
        first.intent
    );

    let after_first = [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer.clone()),
    ];
    let path_argument = solver.solve_with_history(PATH_ARGUMENT_PROMPT, &after_first);
    assert_eq!(
        path_argument.intent, "write_program",
        "issue #386 setup should keep the path-argument follow-up working, got: {}",
        path_argument.intent
    );

    let after_path = [
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer.clone()),
        ConversationTurn::user(PATH_ARGUMENT_PROMPT),
        ConversationTurn::assistant(path_argument.answer.clone()),
    ];
    let reverse_sort = solver.solve_with_history(REVERSE_SORT_PROMPT, &after_path);
    assert_eq!(
        reverse_sort.intent, "write_program",
        "issue #386 setup should reach the reverse-sorted program, got: {}",
        reverse_sort.intent
    );
    assert!(
        answer_reverses_sort(&reverse_sort.answer),
        "issue #386 setup turn 5 must reverse the sort order, got: {}",
        reverse_sort.answer
    );

    vec![
        ConversationTurn::user(FIRST_PROMPT),
        ConversationTurn::assistant(first.answer),
        ConversationTurn::user(PATH_ARGUMENT_PROMPT),
        ConversationTurn::assistant(path_argument.answer),
        ConversationTurn::user(REVERSE_SORT_PROMPT),
        ConversationTurn::assistant(reverse_sort.answer),
    ]
}

#[test]
fn issue_386_cancel_sort_follow_up_must_not_be_unknown() {
    let solver = UniversalSolver::default();
    let history = issue_386_history(&solver);
    let response = solver.solve_with_history(CANCEL_SORT_PROMPT, &history);

    assert_ne!(
        response.intent, "unknown",
        "turn 7 must route to a real program-modification answer, got: {}",
        response.answer
    );
    assert_eq!(
        response.intent, "write_program",
        "cancelling the sort must re-emit the program, got: {}",
        response.intent
    );
    assert!(
        !answer_reverses_sort(&response.answer),
        "turn 7 must REMOVE the reverse sort (ascending again), got: {}",
        response.answer
    );
    assert!(
        answer_keeps_path_argument(&response.answer),
        "cancelling the sort must keep the earlier path-argument modification, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Report issue"),
        "resolvable modifications should answer with the modification, not a report prompt: {}",
        response.answer
    );
}

#[test]
fn issue_386_diagnostic_mode_emits_full_cancel_reasoning_chain() {
    let setup_solver = UniversalSolver::default();
    let history = issue_386_history(&setup_solver);
    let diagnostic_solver = UniversalSolver::new(SolverConfig {
        diagnostic_mode: true,
        ..SolverConfig::default()
    });
    let response = diagnostic_solver.solve_with_history(CANCEL_SORT_PROMPT, &history);

    assert_eq!(response.intent, "write_program");
    assert!(
        response.answer.contains("[diagnostic]"),
        "diagnostic mode should append an inspectable trace block, got: {}",
        response.answer
    );
    for expected in [
        // The seed router yields `unknown`; rule synthesis takes over.
        "selected_rule initial unknown reason no_seed_route next try_rule_synthesis",
        // The active program artifact is the accumulated reverse-sorted variant.
        "write_program_coreference_rewrite",
        "referent=active_program_artifact task=list_files_arg_reverse_sort language=rust",
        // The cancel verb is decomposed as the inverse of reverse_sort.
        "rule_synthesis_operation_vocabulary",
        "cancel_reverse_sort",
        "rule_synthesis_request",
        "operation cancel",
        "operation_modifier reverse_sort",
        // The derived subtractive rule is selected and verified.
        "rule_synthesis_candidate",
        "candidate cancel_reverse_sort__reverse_sort_list_files_arg",
        "rule_verification",
        "lowering_check passed",
        "render_check passed",
        "status passed",
        // The plan lowers back to the unsorted path variant.
        "write_program_plan",
        "resolved_task list_files_arg",
        "program_parameter:task list_files_arg",
    ] {
        assert!(
            response.answer.contains(expected),
            "diagnostic trace should include `{expected}`, got: {}",
            response.answer
        );
    }
    // The cancel must not leave a descending-order claim behind.
    assert!(
        !response
            .answer
            .contains("program_parameter:task list_files_arg_reverse_sort"),
        "the cancelled plan must not resolve to the reverse-sorted task, got: {}",
        response.answer
    );
}

#[test]
fn issue_386_cancel_sort_is_undone_in_every_supported_language() {
    struct Case {
        language: &'static str,
        first: &'static str,
        path_argument: &'static str,
        reverse_sort: &'static str,
        cancel_sort: &'static str,
    }

    // Every conversation targets a Rust program; only the natural language of
    // the requests varies. The cancel verb in each language must undo the sort.
    for case in [
        Case {
            language: "en",
            first: "Write me a Rust program that lists the files in the current directory",
            path_argument: "Make the program accept a path as an argument",
            reverse_sort: "Sort the results in reverse order",
            cancel_sort: "Cancel the sorting",
        },
        Case {
            language: "ru",
            first: FIRST_PROMPT,
            path_argument: PATH_ARGUMENT_PROMPT,
            reverse_sort: REVERSE_SORT_PROMPT,
            cancel_sort: CANCEL_SORT_PROMPT,
        },
        Case {
            language: "hi",
            first: "Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो",
            path_argument: "इसे ऐसा बनाओ कि प्रोग्राम पथ को तर्क के रूप में स्वीकार करे",
            reverse_sort: "परिणामों को उल्टे क्रम में क्रमबद्ध करो",
            cancel_sort: "सॉर्ट हटाओ",
        },
        Case {
            language: "zh",
            first: "用 Rust 编写一个列出当前目录中文件的程序",
            path_argument: "制作程序，使其接受路径作为参数",
            reverse_sort: "对结果倒序排序",
            cancel_sort: "取消排序",
        },
    ] {
        let solver = UniversalSolver::default();

        let first = solver.solve(case.first);
        assert_eq!(
            first.intent, "write_program",
            "[{}] first turn should write a program, got: {}",
            case.language, first.intent
        );

        let after_first = [
            ConversationTurn::user(case.first),
            ConversationTurn::assistant(first.answer.clone()),
        ];
        let path = solver.solve_with_history(case.path_argument, &after_first);
        assert_eq!(
            path.intent, "write_program",
            "[{}] path-argument follow-up should write a program, got: {}",
            case.language, path.intent
        );

        let after_path = [
            ConversationTurn::user(case.first),
            ConversationTurn::assistant(first.answer.clone()),
            ConversationTurn::user(case.path_argument),
            ConversationTurn::assistant(path.answer.clone()),
        ];
        let reverse = solver.solve_with_history(case.reverse_sort, &after_path);
        assert_eq!(
            reverse.intent, "write_program",
            "[{}] reverse-sort follow-up should write a program, got: {}",
            case.language, reverse.intent
        );
        assert!(
            answer_reverses_sort(&reverse.answer),
            "[{}] reverse-sort follow-up must reverse the order, got: {}",
            case.language,
            reverse.answer
        );

        let after_reverse = [
            ConversationTurn::user(case.first),
            ConversationTurn::assistant(first.answer),
            ConversationTurn::user(case.path_argument),
            ConversationTurn::assistant(path.answer),
            ConversationTurn::user(case.reverse_sort),
            ConversationTurn::assistant(reverse.answer),
        ];
        let cancel = solver.solve_with_history(case.cancel_sort, &after_reverse);
        assert_eq!(
            cancel.intent, "write_program",
            "[{}] cancel follow-up must route to write_program (issue #386), got: {}",
            case.language, cancel.intent
        );
        assert!(
            !answer_reverses_sort(&cancel.answer),
            "[{}] cancel follow-up must remove the reverse sort, got: {}",
            case.language,
            cancel.answer
        );
        assert!(
            answer_keeps_path_argument(&cancel.answer),
            "[{}] cancel follow-up must keep the path argument, got: {}",
            case.language,
            cancel.answer
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

fn answer_keeps_path_argument(answer: &str) -> bool {
    let compact = answer
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<String>();

    compact.contains("env::args") || compact.contains("args().nth(")
}
