use formal_ai::{ConversationTurn, UniversalSolver};

const META_ALGORITHM: &str =
    "algorithm_construction:meta_algorithm problem_class_to_shared_ir_to_renderers_to_verification";

fn print_trace(surface: &str, links: &str) {
    let active = format!("algorithm_construction:active_surface {surface}");
    assert!(links.contains(META_ALGORITHM), "{surface}: missing builder");
    assert!(links.contains(&active), "{surface}: missing active marker");
    assert_eq!(
        links.matches("algorithm_construction:stage").count(),
        7,
        "{surface}: unexpected construction shape"
    );
    println!("{surface}: meta_algorithm=shared stages=7 active={surface}");
}

fn main() {
    let solver = UniversalSolver::default();

    let installation =
        solver.solve("Convert this README installation guide into a sh script: run `npm install`.");
    print_trace("installation_conversion", &installation.links_notation);

    let synthesis = solver.solve(
        "Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
    );
    print_trace("program_synthesis", &synthesis.links_notation);

    let catalog = solver.solve("Write hello world in Rust");
    print_trace("coding_catalog", &catalog.links_notation);

    let numeric = solver.solve(
        "I have numbers 5, 3, 8, 1, 9 — sort them in JavaScript, give me the code and the result",
    );
    print_trace("numeric_list", &numeric.links_notation);

    let first_prompt = "Write me a Rust program that lists files in the current directory";
    let first = solver.solve(first_prompt);
    let history = [
        ConversationTurn::user(first_prompt),
        ConversationTurn::assistant(first.answer),
    ];
    let rule = solver.solve_with_history("Sort the results in reverse order", &history);
    print_trace("rule_synthesis", &rule.links_notation);
}
