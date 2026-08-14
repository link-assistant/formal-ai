//! Mine reusable method proposals from real recursive-core executions.

use formal_ai::intent_formalization::{formalize_intent, IntentFormalization};
use formal_ai::meta_construction::RecursionMode;
use formal_ai::method_learning::learn_methods_from_event_logs;
use formal_ai::promotion::render_promotion_proposals;
use formal_ai::recipe_interpreter::RecipeProgram;
use formal_ai::selection::SelectionMode;
use formal_ai::skill_ledger::SkillMode;
use formal_ai::translation::formalize_prompt;
use formal_ai::EventLog;

fn formalize(prompt: &str) -> IntentFormalization {
    let candidate = formalize_prompt(prompt, "en");
    formalize_intent(prompt, "en", Some(&candidate))
}

fn solve_trace(prompt: &str) -> EventLog {
    RecipeProgram::from_repo()
        .execute(
            &formalize(prompt),
            4,
            RecursionMode::Both,
            SelectionMode::Record,
            SkillMode::Accumulate,
        )
        .expect("the production recursive recipe should execute")
        .log
}

fn main() {
    let observations = [
        ("solve-translation", "translate apple to Russian"),
        (
            "solve-composed",
            "translate apple to Russian and write a hello world program in Python",
        ),
        ("solve-unknown", "zzqqx unfathomable gibberish token"),
    ]
    .map(|(id, prompt)| (id, solve_trace(prompt)));
    let borrowed = observations
        .iter()
        .map(|(id, log)| (*id, log))
        .collect::<Vec<_>>();
    let learning = learn_methods_from_event_logs(&borrowed);
    let proposal = learning
        .promotion_proposals()
        .into_iter()
        .next()
        .expect("three matching traces should produce a validated proposal");

    print!("{}", render_promotion_proposals(&[proposal]));
}
