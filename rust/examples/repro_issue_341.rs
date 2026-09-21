// Reproduction for issue #341: second step of a decomposed agent plan falls to unknown.
use formal_ai::{ConversationTurn, SolverConfig, UniversalSolver};

fn main() {
    // The bug report runs in the wasm worker / manual mode, which is offline.
    let config = SolverConfig {
        offline: true,
        ..SolverConfig::default()
    };
    let solver = UniversalSolver::new(config);
    let step1 = concat!(
        "Design a simple web scraper in Python that:\n",
        "1. Fetches a webpage\n",
        "2. Extracts all headings (h1, h2, h3)\n",
        "3. Counts word frequency\n",
        "4. Generates a markdown summary"
    );
    let step2 = "test it by scraping wikipedia.org and show me the top 10 most frequent words.";

    let plan = solver.solve(step1);
    println!("=== STEP 1 intent: {} ===", plan.intent);

    let history = [
        ConversationTurn::user(step1),
        ConversationTurn::assistant(plan.answer),
    ];
    let step2_answer = solver.solve_with_history(step2, &history);
    println!("=== STEP 2 intent: {} ===", step2_answer.intent);
    println!("--- STEP 2 answer ---\n{}", step2_answer.answer);
}
