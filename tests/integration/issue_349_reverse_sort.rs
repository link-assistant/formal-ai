use formal_ai::{ConversationTurn, UniversalSolver};

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
#[ignore = "tracks #349/#355 until #358/#359 implement reverse-sort program modifications"]
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
