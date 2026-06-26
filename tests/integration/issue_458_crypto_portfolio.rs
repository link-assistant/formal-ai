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

#[test]
fn issue_458_crypto_portfolio_tracker_covers_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
        localized_intro: &'static str,
    }

    let solver = UniversalSolver::default();
    for case in [
        Case {
            language: "en",
            prompt: "English: Simulate a crypto portfolio tracker: current prices for BTC ETH TON USDT, portfolio holdings, total value, 24h change, weight distribution. Write a Python script with a public API mock endpoint, alert notify if any asset drops more than 5%, and a Markdown report.",
            localized_intro: "Here is a Python program",
        },
        Case {
            language: "ru",
            prompt: "Смоделируй crypto portfolio tracker: current prices for BTC ETH TON USDT, portfolio holdings, total value, 24h change, weight distribution. Write a Python script with a public API mock endpoint, alert notify if any asset drops more than 5%, and a Markdown report.",
            localized_intro: "Вот программа на языке Python",
        },
        Case {
            language: "hi",
            prompt: "कृपया crypto portfolio tracker simulate करें: current prices for BTC ETH TON USDT, portfolio holdings, total value, 24h change, weight distribution. Write a Python script with a public API mock endpoint, alert notify if any asset drops more than 5%, and a Markdown report.",
            localized_intro: "यहाँ Python में एक प्रोग्राम है",
        },
        Case {
            language: "zh",
            prompt: "请 simulate crypto portfolio tracker: current prices for BTC ETH TON USDT, portfolio holdings, total value, 24h change, weight distribution. Write a Python script with a public API mock endpoint, alert notify if any asset drops more than 5%, and a Markdown report.",
            localized_intro: "这是一个解决该复合任务的 Python 程序",
        },
    ] {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "write_program",
            "{} prompt should route to write_program, got: {} / {}",
            case.language, response.intent, response.answer
        );
        assert!(
            response
                .links_notation
                .contains("program_blueprint:recipe crypto_portfolio_tracker"),
            "{} prompt should use the crypto portfolio blueprint, got: {}",
            case.language,
            response.links_notation
        );
        assert!(
            response
                .links_notation
                .contains(&format!("language:{}", case.language)),
            "{} prompt should preserve detected language in the trace, got: {}",
            case.language,
            response.links_notation
        );
        assert!(
            response.answer.contains(case.localized_intro),
            "{} prompt should render the localized blueprint intro, got: {}",
            case.language,
            response.answer
        );
        assert!(
            response.answer.contains("```python") && response.answer.contains("notify_alerts"),
            "{} prompt should still return executable Python alert code, got: {}",
            case.language,
            response.answer
        );
    }
}
