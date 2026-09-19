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

const PYTHON_PROGRAM: &str = r###"from dataclasses import dataclass
from typing import Mapping

@dataclass(frozen=True)
class AssetPrice:
    symbol: str
    price_usd: float
    change_24h_pct: float

PORTFOLIO = {
    'BTC': 2.5,
    'ETH': 15.0,
    'TON': 1000.0,
    'USDT': 5000.0,
}

MOCK_API_RESPONSE = {
    'BTC': {'price_usd': 64120.25, 'change_24h_pct': -2.4},
    'ETH': {'price_usd': 3350.80, 'change_24h_pct': 1.2},
    'TON': {'price_usd': 6.85, 'change_24h_pct': -5.7},
    'USDT': {'price_usd': 1.00, 'change_24h_pct': 0.0},
}

def fetch_prices(symbols: list[str]) -> dict[str, AssetPrice]:
    prices = {}
    for symbol in symbols:
        payload = MOCK_API_RESPONSE[symbol]
        prices[symbol] = AssetPrice(
            symbol=symbol,
            price_usd=payload['price_usd'],
            change_24h_pct=payload['change_24h_pct'],
        )
    return prices

def portfolio_rows(
    holdings: Mapping[str, float],
    prices: Mapping[str, AssetPrice],
) -> list[dict[str, float | str]]:
    total_value = sum(amount * prices[symbol].price_usd for symbol, amount in holdings.items())
    rows = []
    for symbol, amount in holdings.items():
        price = prices[symbol]
        value = amount * price.price_usd
        portfolio_weight = value / total_value * 100
        rows.append({
            'symbol': symbol,
            'amount': amount,
            'price_usd': price.price_usd,
            'value_usd': value,
            'change_24h_pct': price.change_24h_pct,
            'portfolio_weight': portfolio_weight,
        })
    return rows

def notify_alerts(rows: list[dict[str, float | str]]) -> list[str]:
    return [
        f'Notify: {row["symbol"]} dropped {row["change_24h_pct"]:.2f}% in 24h'
        for row in rows
        if float(row['change_24h_pct']) < -5.0
    ]

def money(value: float) -> str:
    return f'${value:,.2f}'

def render_dashboard(rows: list[dict[str, float | str]], notices: list[str]) -> str:
    total_value = sum(float(row['value_usd']) for row in rows)
    lines = [
        '# Crypto Portfolio Dashboard',
        '',
        f'**Total value:** {money(total_value)}',
        '',
        '| Asset | Amount | Price USD | Value USD | 24h change | Portfolio weight |',
        '| --- | ---: | ---: | ---: | ---: | ---: |',
    ]
    for row in rows:
        lines.append(
            f'| {row["symbol"]} | {row["amount"]:,.4g} | {money(float(row["price_usd"]))} | '
            f'{money(float(row["value_usd"]))} | {row["change_24h_pct"]:.2f}% | '
            f'{row["portfolio_weight"]:.2f}% |'
        )
    lines.extend(['', '## Alerts'])
    lines.extend(notices or ['No asset dropped more than 5% in the last 24h.'])
    return '\n'.join(lines)

def main() -> None:
    prices = fetch_prices(list(PORTFOLIO))
    rows = portfolio_rows(PORTFOLIO, prices)
    notices = notify_alerts(rows)
    formatted_log = render_dashboard(rows, notices)
    print(formatted_log)

if __name__ == '__main__':
    main()"###;

fn expected_answer(
    intro: &str,
    decomposition: &str,
    libraries: &str,
    run_heading: &str,
    execution: &str,
) -> String {
    format!(
        "{intro} {decomposition}\n\n```python\n{PYTHON_PROGRAM}\n```\n\n{libraries}\n- Python 3 standard library only\n\n{run_heading}\n\n{execution}"
    )
}

const ENGLISH_DECOMPOSITION_WITH_HTTP: &str = "I decomposed your request into these sub-tasks:\n\n1. Make an HTTP request\n2. Output the results\n3. Research current source data\n4. Export a Markdown comparison report\n5. Fetch current crypto prices\n6. Model portfolio holdings\n7. Calculate total value, 24h changes, and weights\n8. Notify when an asset drops more than 5%\n9. Mock the public API endpoint";
const ENGLISH_DECOMPOSITION: &str = "I decomposed your request into these sub-tasks:\n\n1. Output the results\n2. Research current source data\n3. Export a Markdown comparison report\n4. Fetch current crypto prices\n5. Model portfolio holdings\n6. Calculate total value, 24h changes, and weights\n7. Notify when an asset drops more than 5%\n8. Mock the public API endpoint";
const EXECUTION_EN: &str = "Execution status: not run — this report blueprint was not executed in the offline sandbox, and the embedded data assumptions should be reviewed before use. The code is provided for review. Run it yourself: `python crypto_portfolio.py`.";

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
    assert_eq!(
        response.answer,
        expected_answer(
            "Here is a Python program for the requested composite task (simulate a crypto portfolio tracker with alerts and a Markdown dashboard).",
            ENGLISH_DECOMPOSITION_WITH_HTTP,
            "Required libraries:",
            "How to run it yourself:",
            EXECUTION_EN,
        )
    );
}

#[test]
fn issue_458_crypto_portfolio_tracker_covers_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
        localized_intro: &'static str,
        expected_answer: String,
    }

    let solver = UniversalSolver::default();
    for case in [
        Case {
            language: "en",
            prompt: "English: Simulate a crypto portfolio tracker: current prices for BTC ETH TON USDT, portfolio holdings, total value, 24h change, weight distribution. Write a Python script with a public API mock endpoint, alert notify if any asset drops more than 5%, and a Markdown report.",
            localized_intro: "Here is a Python program",
            expected_answer: expected_answer(
                "Here is a Python program for the requested composite task (simulate a crypto portfolio tracker with alerts and a Markdown dashboard).",
                ENGLISH_DECOMPOSITION,
                "Required libraries:",
                "How to run it yourself:",
                EXECUTION_EN,
            ),
        },
        Case {
            language: "ru",
            prompt: "Смоделируй crypto portfolio tracker: current prices for BTC ETH TON USDT, portfolio holdings, total value, 24h change, weight distribution. Write a Python script with a public API mock endpoint, alert notify if any asset drops more than 5%, and a Markdown report.",
            localized_intro: "Вот программа на языке Python",
            expected_answer: expected_answer(
                "Вот программа на языке Python, которая решает составную задачу (смоделировать криптопортфель с оповещениями и Markdown-панелью).",
                "Я разбил ваш запрос на следующие подзадачи:\n\n1. Вывести результаты\n2. Найти актуальные исходные данные\n3. Экспортировать Markdown-отчёт со сравнением\n4. Fetch current crypto prices\n5. Model portfolio holdings\n6. Calculate total value, 24h changes, and weights\n7. Notify when an asset drops more than 5%\n8. Mock the public API endpoint",
                "Необходимые библиотеки:",
                "Как запустить самостоятельно:",
                "Статус выполнения: не запускалось — этот отчёт не выполнялся в офлайн-песочнице, а встроенные допущения о данных нужно проверить перед использованием. Код приведён для проверки. Запустить самостоятельно: `python crypto_portfolio.py`.",
            ),
        },
        Case {
            language: "hi",
            prompt: "कृपया crypto portfolio tracker simulate करें: current prices for BTC ETH TON USDT, portfolio holdings, total value, 24h change, weight distribution. Write a Python script with a public API mock endpoint, alert notify if any asset drops more than 5%, and a Markdown report.",
            localized_intro: "यहाँ Python में एक प्रोग्राम है",
            expected_answer: expected_answer(
                "यहाँ Python में एक प्रोग्राम है जो इस संयुक्त कार्य को हल करता है (alerts और Markdown dashboard वाला crypto portfolio tracker simulate करें)।",
                "मैंने आपके अनुरोध को इन उप-कार्यों में विभाजित किया है:\n\n1. परिणाम आउटपुट करें\n2. वर्तमान स्रोत डेटा खोजें\n3. Markdown तुलना रिपोर्ट निर्यात करें\n4. Fetch current crypto prices\n5. Model portfolio holdings\n6. Calculate total value, 24h changes, and weights\n7. Notify when an asset drops more than 5%\n8. Mock the public API endpoint",
                "आवश्यक लाइब्रेरियाँ:",
                "इसे स्वयं कैसे चलाएँ:",
                "निष्पादन स्थिति: नहीं चलाया गया — यह रिपोर्ट ऑफ़लाइन सैंडबॉक्स में नहीं चली, और embedded data assumptions को उपयोग से पहले जाँचना चाहिए। कोड समीक्षा के लिए दिया गया है। स्वयं चलाएँ: `python crypto_portfolio.py`।",
            ),
        },
        Case {
            language: "zh",
            prompt: "请 simulate crypto portfolio tracker: current prices for BTC ETH TON USDT, portfolio holdings, total value, 24h change, weight distribution. Write a Python script with a public API mock endpoint, alert notify if any asset drops more than 5%, and a Markdown report.",
            localized_intro: "这是一个解决该复合任务的 Python 程序",
            expected_answer: expected_answer(
                "这是一个解决该复合任务的 Python 程序（模拟带提醒和 Markdown 仪表盘的加密投资组合追踪器）。",
                "我已将您的请求分解为以下子任务：\n\n1. 输出结果\n2. 检索当前来源数据\n3. 导出 Markdown 比较报告\n4. Fetch current crypto prices\n5. Model portfolio holdings\n6. Calculate total value, 24h changes, and weights\n7. Notify when an asset drops more than 5%\n8. Mock the public API endpoint",
                "所需的库：",
                "如何自行运行：",
                "执行状态：未运行 —— 该报告未在离线沙箱中执行，内置数据假设应先核对再使用。代码仅供审阅。自行运行：`python crypto_portfolio.py`。",
            ),
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
        assert_eq!(response.answer, case.expected_answer);
    }
}
