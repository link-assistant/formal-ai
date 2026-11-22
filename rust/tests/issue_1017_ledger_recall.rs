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
use formal_ai::{LearningLedger, approved_lesson_for, solve};

/// A request that routes to no known rule, so the solver reaches ledger recall —
/// the path that used to pay for the parse. Issue #1017 measured this exact
/// prompt at 12.08 s on a cold process and 0.16 s on a warm one.
const UNKNOWN_PROMPT: &str = "look up the latest news about renewable energy";

/// What the solver answers offline, exactly — recorded here so the test doubles
/// as documentation (R234-2) and so a change in the answer is a decision rather
/// than a silent drift. Live fetching is off by default, so the request is
/// reported as an unexecuted plan naming the sources it would have used.
const EXPECTED_ANSWER: &str = "No captured provider response is available for `renewable energy`. \
Live fetching is off by default; enable it explicitly to populate the replayable source cache. \
The unexecuted plan includes DuckDuckGo, Internet Archive, Wikipedia, Wikidata, Wiktionary, and \
Wikinews.";

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
    assert_eq!(
        answer.answer, EXPECTED_ANSWER,
        "the solver still answers, and answers the same thing; the point is what \
         it does not do first"
    );
    assert_eq!(
        ast_census_runs(),
        baseline,
        "an ordinary solve must not round-trip the pinned module's CST/AST"
    );

    let fixture = LearningLedger::from_approved_lessons_seed(include_str!(
        "fixtures/issue-558/approved-lessons.lino"
    ))
    .unwrap();
    let prompt = &fixture.entries()[0].failure_prompt;
    assert!(
        approved_lesson_for(prompt).is_none(),
        "a fresh runtime has no shipped lesson"
    );
    assert!(
        fixture.lesson_for(prompt).is_some(),
        "the explicit reviewed record remains usable"
    );
    assert_eq!(ast_census_runs(), baseline);
}
