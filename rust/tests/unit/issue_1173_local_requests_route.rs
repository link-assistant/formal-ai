//! Issue #1173 R1173-3: a request that needs no outside knowledge is
//! answered by its class handler, never by the fallback. Two held-out
//! routing probes reached the browser fallback until this change: a
//! compound-interest problem compounded "yearly" (the `year` inside "yearly"
//! hid the stated term from the years scan, in both runtimes) and a CSV table
//! asked for as JSON (no reader existed). Both now reach their handler; the
//! browser twin is `rust/tests/web/issue-1173-local-requests-route.test.mjs`.

use formal_ai::{SolverConfig, UniversalSolver};

fn solved(prompt: &str) -> (String, String) {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    let answer = solver.solve(prompt);
    (answer.intent, answer.answer)
}

#[test]
fn a_yearly_compounding_problem_reaches_the_interest_handler() {
    let (intent, answer) = solved("invest $2500 at 6% annual interest compounded yearly for 10 years");
    assert_eq!(intent, "calculation");
    assert_eq!(answer, "Compound interest calculation\n\nFormula: A = P(1 + r/n)^(n*t)\nP = 2500 USD\nr = 0.06 (6% annual)\nn = 1 (annually)\nt = 10 years\n\nStep 1: periodic rate = r/n = 0.06/1 = 0.06\nStep 2: number of periods = n*t = 1*10 = 10\nStep 3: A = 2500 * (1 + 0.06)^10\nFinal amount: 4477.12 USD");
}

#[test]
fn a_csv_table_asked_for_as_json_reaches_the_format_converter() {
    let (intent, answer) = solved("Convert this CSV to JSON: name,age\nAnn,30");
    assert_eq!(intent, "format_conversion");
    assert_eq!(answer, "Converted CSV to JSON. Data rows: 1; header columns: name, age. CSV carries no types, so every value stays a JSON string rather than a guessed number or boolean.\n\n```json\n[\n  {\n    \"name\": \"Ann\",\n    \"age\": \"30\"\n  }\n]\n```\nRound-trip check: parsing the emitted JSON back gave every row the same value under every header column as the CSV it came from.");
}
