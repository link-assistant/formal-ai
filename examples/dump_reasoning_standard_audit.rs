//! Audit reasoning episodes against the issue #1073 standard and print the ledger.
//!
//! Run with `cargo run --example dump_reasoning_standard_audit`. It prints two
//! ledgers, because the requirement is a *floor*, not a hard-case escalation:
//!
//!   1. the reference dialog the standard was derived from, which satisfies every
//!      gate it triggers and reaches `confirmed`;
//!   2. the most trivial request the pipeline can formalize, which triggers no
//!      gate at all — and still has every declared gate enumerated with the
//!      trigger that was false, and a `not_confirmed_not_refuted` verdict naming
//!      what blocked the check.
//!
//! Both are produced without a model in the loop.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::reasoning_standard::{audit, open_episode, reference_episode, standard};
use formal_ai::translation::formalize_prompt;

const TRIVIAL: &str = "hi";

fn main() {
    let standard = standard().expect("the standard should load");

    println!("# the reference dialog");
    print!(
        "{}",
        audit(&standard, &reference_episode()).to_links_notation()
    );

    let candidate = formalize_prompt(TRIVIAL, "en");
    let formalization = formalize_intent(TRIVIAL, "en", Some(&candidate));
    let trivial = audit(&standard, &open_episode(&formalization));
    println!();
    println!("# the trivial request {TRIVIAL:?}");
    print!("{}", trivial.to_links_notation());
    println!();
    println!(
        "# every declared gate is reported for {TRIVIAL:?}: {} of {}",
        trivial.outcomes.len(),
        standard.gates.len()
    );
}
