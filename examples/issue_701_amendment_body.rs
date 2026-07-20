//! Does a retained dreaming amendment change the *body* of an answer, or only
//! decorate it with a trailing "Learned standing requirement" line?
//!
//! Run as an example (`cp` into `examples/` or run with `--manifest-path`), or
//! read it as the record of the issue-#701 decoration audit.
//!
//! ```sh
//! cargo run --release --example issue_701_amendment_body
//! ```

use formal_ai::dreaming_application::amendment_line;
use formal_ai::{solve_with_amendment_records, RetainedAmendment, UniversalSolver};

fn main() {
    let solver = UniversalSolver::default();
    let amendment = RetainedAmendment {
        id: String::from("amendment-1"),
        topic: String::from("latex"),
        rule: String::from("Always include a LaTeX verification step in proof solutions."),
    };
    for prompt in [
        "latex: solve a new recurrence proof",
        "latex: как доказать рекуррентное соотношение",
    ] {
        let plain = solver.solve(prompt);
        let amended = solve_with_amendment_records(&solver, prompt, &[], &[amendment.clone()]);
        let line = amendment_line(&amendment);
        let body = amended.answer.replace(&line, "");
        println!("--- {prompt}");
        println!("plain intent   = {}", plain.intent);
        println!("amended intent = {}", amended.intent);
        println!(
            "body differs beyond the appended line = {}",
            body.trim() != plain.answer.trim()
        );
        println!("plain body   : {}", plain.answer.replace('\n', " | "));
        println!("amended body : {}", body.replace('\n', " | "));
    }
}
