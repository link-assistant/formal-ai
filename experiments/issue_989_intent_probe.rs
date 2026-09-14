//! Inspect the seeded dialog-control route from issue #989 without a server.

use formal_ai::language::detect;
use formal_ai::seed;
use formal_ai::UniversalSolver;

fn main() {
    let prompt = "`quick` is subjective opinion, please don't use these anymore.";
    let normalized = "quick is subjective opinion please don t use these anymore".to_owned();
    println!("normalized={normalized}");
    println!("language={}", detect(prompt).slug());
    println!(
        "role={}",
        seed::lexicon().mentions_role(seed::ROLE_CONVERSATION_PREFERENCE_AVOID, &normalized)
    );
    println!(
        "response={:?}",
        seed::render_response(
            "conversation_preference",
            detect(prompt).slug(),
            &[("term", "quick")],
        )
    );
    let answer = UniversalSolver::default().solve(prompt);
    println!("intent={}", answer.intent);
    println!("answer={}", answer.answer);
}
