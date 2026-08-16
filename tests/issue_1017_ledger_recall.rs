//! Issue #1017: answering an ordinary request must not parse a whole module.
//!
//! Two macOS core slices failed by *timing out* — the integration harness hit its
//! thirty-second `RESPONSE_TIMEOUT` waiting for a server's first HTTP response.
//! The cost was not in the socket, the handler or the memory path: every solve
//! that reaches rule recall called [`approved_lesson_for`], which built the
//! canonical learning ledger, which ran the whole self-healing pass, which
//! round-tripped the pinned module's entire CST/AST. That parse is quadratic in
//! file size (`examples/issue_1017_parse_scaling.rs` shows the per-byte cost
//! doubling every time the input doubles), so it cost over ten seconds on the
//! `dev` profile CI tests run under — inside the first request, every process.
//!
//! The pins below state the fix *structurally* rather than as a wall-clock budget,
//! because a budget is exactly what differs between a developer's machine and a
//! loaded `macos-15-intel` runner:
//!
//! * a prompt the ledger cannot answer is refused without parsing anything, and
//! * the cheap set of answerable prompts equals what the built ledger really holds,
//!   so the short-circuit can never start refusing a prompt that has a lesson.
//!
//! This lives in its own test binary on purpose: [`ast_census_runs`] is a
//! process-wide counter, so a sibling test parsing on another thread would perturb
//! it. One binary, one test, one deterministic count.

use formal_ai::agentic_coding::self_ast::ast_census_runs;
use formal_ai::{
    approved_lesson_for, canonical_failure_trace, canonical_ledger,
    canonical_ledger_failure_prompts, solve,
};

/// A request that routes to no known rule, so the solver reaches ledger recall —
/// the path that used to pay for the parse. Issue #1017 measured this exact
/// prompt at 12.08 s on a cold process and 0.16 s on a warm one.
const UNKNOWN_PROMPT: &str = "look up the latest news about renewable energy";

#[test]
fn ordinary_recall_never_parses_the_pinned_module() {
    // Ordering matters: every miss must be observed *before* anything is allowed
    // to build the ledger, because building it is what runs the parse.
    let baseline = ast_census_runs();

    // A direct miss. This is the non-vacuous check: it exercises the guard even
    // if the solver's routing later stops sending `UNKNOWN_PROMPT` to recall.
    assert!(
        approved_lesson_for(UNKNOWN_PROMPT).is_none(),
        "the canonical ledger holds no lesson for an unrelated prompt"
    );
    assert_eq!(
        ast_census_runs(),
        baseline,
        "refusing a prompt the ledger cannot answer must not parse a module"
    );

    // The whole solve, end to end — the shape the failing request actually had.
    let answer = solve(UNKNOWN_PROMPT);
    assert!(
        !answer.answer.is_empty(),
        "the solver still answers; the point is what it does not do first"
    );
    assert_eq!(
        ast_census_runs(),
        baseline,
        "an ordinary solve must not round-trip the pinned module's CST/AST"
    );

    // A hit still recalls the approved lesson: the short-circuit narrows nothing.
    let canonical_prompt = canonical_failure_trace().prompt;
    let lesson = approved_lesson_for(&canonical_prompt)
        .expect("the canonical failure prompt still recalls its approved lesson");
    assert_eq!(
        lesson.failure_prompt, canonical_prompt,
        "the recalled entry is the one promoted from the canonical trace"
    );

    // The guard is only safe while the cheap set and the real ledger agree.
    // Deriving the set from the trace is what makes it cheap; comparing it
    // against the ledger that is actually built is what keeps it honest.
    let built: Vec<String> = canonical_ledger()
        .entries()
        .iter()
        .map(|entry| entry.failure_prompt.clone())
        .collect();
    assert_eq!(
        canonical_ledger_failure_prompts(),
        built,
        "`canonical_ledger_failure_prompts` must list exactly the prompts the \
         canonical ledger answers, or the cheap miss check would refuse a prompt \
         that has an approved lesson"
    );
}
