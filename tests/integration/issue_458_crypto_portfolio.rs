//! Issue #458: a composite Python crypto-portfolio request used to either
//! dead-end on `task=missing` or get swallowed by generic web-search routing.

use formal_ai::UniversalSolver;

const ISSUE_PROMPT: &str = "Simulate a crypto portfolio tracker:\n\
   1. Search current prices for: BTC, ETH, TON, USDT\n\
   2. Assume portfolio: 2.5 BTC, 15 ETH, 1000 TON, 5000 USDT\n\
   3. Calculate:\n\
      - Total value in USD\n\
      - 24h change % for each asset\n\
      - Portfolio weight distribution\n\
   4. Write a Python script that:\n\
      - Fetches prices from a public API (mock the endpoint)\n\
      - Implements alert logic: \"Notify if any asset drops >5%\"\n\
      - Logs results to a formatted string\n\
   5. Output: dashboard-style markdown + executable code";

#[test]
fn issue_458_crypto_portfolio_tracker_returns_python_blueprint() {
    let solver = UniversalSolver::default();
    let response = solver.solve(ISSUE_PROMPT);

    assert_eq!(
        response.intent, "write_program",
        "the issue prompt must route to write_program, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        !response.answer.contains("I do not have a template")
            && !response.answer.contains("task `missing`")
            && !response.answer.contains("Web search requested"),
        "must not surface a missing-template or generic-search dead end, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("```python"),
        "answer must contain a Python code fence, got: {}",
        response.answer
    );
    for expected in [
        "BTC",
        "ETH",
        "TON",
        "USDT",
        "portfolio_weight",
        "notify",
        "# Crypto Portfolio Dashboard",
    ] {
        assert!(
            response.answer.contains(expected),
            "answer must include {expected:?}, got: {}",
            response.answer
        );
    }
    assert!(
        response
            .links_notation
            .contains("program_blueprint:recipe crypto_portfolio_tracker"),
        "trace must record the crypto portfolio blueprint recipe, got: {}",
        response.links_notation
    );
}
