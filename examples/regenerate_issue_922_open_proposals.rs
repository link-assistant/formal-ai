//! Regenerate the committed issue #922 promotion-proposal document.
//!
//! ```bash
//! cargo run --example regenerate_issue_922_open_proposals
//! ```
//!
//! `examples/issue-922-method-learning/open-proposals.lino` is the reviewed
//! promotion input, and `tests/unit/issue_922_method_learning.rs` compares it
//! field-for-field with what the learner proposes from three real recursive-core
//! traces. The document is therefore generator output pinned in git: whenever the
//! recursive core emits a different trace tail, the longest recurring operation
//! sequence changes and the candidate's content-addressed id moves with it. This
//! example re-derives the document exactly the way that test derives its
//! expectation, so the checked-in evidence keeps describing the live pipeline
//! instead of a trace shape the code no longer produces.

use std::fs;
use std::path::Path;

use formal_ai::EventLog;
use formal_ai::intent_formalization::{IntentFormalization, formalize_intent};
use formal_ai::meta_construction::RecursionMode;
use formal_ai::method_learning::learn_methods_from_event_logs;
use formal_ai::promotion::render_promotion_proposals;
use formal_ai::recipe_interpreter::RecipeProgram;
use formal_ai::selection::SelectionMode;
use formal_ai::skill_ledger::SkillMode;
use formal_ai::translation::formalize_prompt;

const DOCUMENT: &str = "examples/issue-922-method-learning/open-proposals.lino";

/// The three prompts the regression observes, in its order.
const OBSERVED: &[(&str, &str)] = &[
    ("solve-translation", "translate apple to Russian"),
    (
        "solve-composed",
        "translate apple to Russian and write a hello world program in Python",
    ),
    ("solve-unknown", "zzqqx unfathomable gibberish token"),
];

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
        .expect("the production recursive recipe should produce an event log")
        .log
}

/// The reviewer's own one-sentence summary, read back out of the committed
/// document so regenerating the machine-derived fields never overwrites it.
fn reviewed_summary(document: &str) -> Option<String> {
    let line = document
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("summary \""))?;
    let body = line.strip_prefix("summary \"")?.strip_suffix('"')?;
    Some(body.replace("\\\"", "\""))
}

fn main() {
    let observations: Vec<(String, EventLog)> = OBSERVED
        .iter()
        .map(|(id, prompt)| ((*id).to_owned(), solve_trace(prompt)))
        .collect();
    let borrowed: Vec<(&str, &EventLog)> = observations
        .iter()
        .map(|(id, log)| (id.as_str(), log))
        .collect();

    let learning = learn_methods_from_event_logs(&borrowed);
    let mut promotions = learning.promotion_proposals();
    assert!(
        !promotions.is_empty(),
        "the real traces must still produce at least one promotion proposal"
    );
    // The document is a *reviewed* input, not the learner's whole output: a
    // human reviewed the strongest candidate, so it carries the first proposal
    // only and states its purpose in a sentence rather than in the renderer's
    // `adopt_learned_method:…` slug. Both of those are review decisions, so the
    // regeneration keeps them and refreshes only the machine-derived fields.
    promotions.truncate(1);
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(DOCUMENT);
    let current = fs::read_to_string(&path).unwrap_or_default();
    if let Some(reviewed) = reviewed_summary(&current) {
        promotions[0].summary = reviewed;
    }
    let rendered = format!("{}\n", render_promotion_proposals(&promotions));

    if current == rendered {
        println!(
            "{DOCUMENT} already matches the live learner ({} bytes)",
            rendered.len()
        );
        return;
    }
    fs::write(&path, &rendered).expect("write the promotion-proposal document");
    println!(
        "rewrote {DOCUMENT}: {} bytes -> {} bytes",
        current.len(),
        rendered.len()
    );
    println!("source: {}", promotions[0].source);
}
