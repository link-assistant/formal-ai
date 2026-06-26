//! Curated program bodies used by composite `write_program` blueprints.
//!
//! Keeping longer bodies outside `blueprint.rs` preserves the repository's
//! file-size limit while leaving recipe selection and rendering in one place.

pub(super) const RUST_HTTP_JSON_STATS: &str = r#"//! Fetch JSON from a URL and report the mean and median of every number in it.
//!
//! Cargo.toml dependencies:
//!   reqwest = { version = "0.12", features = ["blocking", "json"] }
//!   serde_json = "1"

use std::env;
use std::error::Error;

use serde_json::Value;

/// Recursively collect every numeric value out of a decoded JSON document,
/// regardless of how deeply it is nested inside arrays or objects.
fn collect_numbers(value: &Value, numbers: &mut Vec<f64>) {
    match value {
        Value::Number(number) => {
            if let Some(as_float) = number.as_f64() {
                numbers.push(as_float);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_numbers(item, numbers)),
        Value::Object(map) => map.values().for_each(|item| collect_numbers(item, numbers)),
        _ => {}
    }
}

/// Arithmetic mean of the samples (the caller guarantees a non-empty slice).
fn mean(samples: &[f64]) -> f64 {
    samples.iter().sum::<f64>() / samples.len() as f64
}

/// Median of the samples; averages the two middle values when the count is even.
fn median(samples: &mut [f64]) -> f64 {
    samples.sort_by(|left, right| left.partial_cmp(right).expect("no NaN in input"));
    let middle = samples.len() / 2;
    if samples.len() % 2 == 0 {
        (samples[middle - 1] + samples[middle]) / 2.0
    } else {
        samples[middle]
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Read the target URL from the first command-line argument.
    let url = env::args()
        .nth(1)
        .ok_or("usage: stats <url-returning-json>")?;

    // 2. Make the HTTP GET request and parse the JSON body. Both steps can fail,
    //    so `?` propagates any network or decoding error up to `main`.
    let document: Value = reqwest::blocking::get(&url)?.json()?;

    // 3. Gather every number from the decoded document.
    let mut numbers = Vec::new();
    collect_numbers(&document, &mut numbers);
    // region:error_handling
    // Guard against an empty data set before computing statistics.
    if numbers.is_empty() {
        return Err("the JSON response contained no numbers".into());
    }
    // endregion:error_handling

    // 4. Compute and print the statistics.
    println!("count:  {}", numbers.len());
    println!("mean:   {:.4}", mean(&numbers));
    println!("median: {:.4}", median(&mut numbers));
    Ok(())
}
"#;

pub(super) const PYTHON_HTTP_JSON_STATS: &str = r#""""Fetch JSON from a URL and report the mean and median of every number in it.

Dependencies:  pip install requests
"""

import statistics
import sys

import requests


def collect_numbers(value):
    """Recursively collect every int/float out of a decoded JSON value."""
    # bool subclasses int, so skip it explicitly
    if isinstance(value, bool):
        return []
    if isinstance(value, (int, float)):
        return [float(value)]
    if isinstance(value, list):
        return [number for item in value for number in collect_numbers(item)]
    if isinstance(value, dict):
        return [number for item in value.values() for number in collect_numbers(item)]
    return []


def main():
    # 1. Read the target URL from the first command-line argument.
    if len(sys.argv) < 2:
        raise SystemExit("usage: stats.py <url-returning-json>")
    url = sys.argv[1]

    # 2. Make the HTTP GET request and parse the JSON body.
    response = requests.get(url, timeout=30)
    # region:error_handling
    # Turn any non-2xx HTTP status into an exception before decoding.
    response.raise_for_status()
    # endregion:error_handling
    document = response.json()

    # 3. Gather every number from the decoded JSON.
    numbers = collect_numbers(document)
    # region:error_handling
    if not numbers:
        raise SystemExit("the JSON response contained no numbers")
    # endregion:error_handling

    # 4. Compute and print the statistics.
    print(f"count:  {len(numbers)}")
    print(f"mean:   {statistics.mean(numbers):.4f}")
    print(f"median: {statistics.median(numbers):.4f}")


if __name__ == "__main__":
    main()
"#;

pub(super) const JAVASCRIPT_HTTP_JSON_STATS: &str = r#"// Fetch JSON from a URL and report the mean and median of every number in it.
//
// Requirements: Node.js 18+ (built-in global fetch; no extra packages).

// Recursively collect every finite number out of a decoded JSON value.
function collectNumbers(value) {
  if (typeof value === "number" && Number.isFinite(value)) return [value];
  if (Array.isArray(value)) return value.flatMap(collectNumbers);
  if (value && typeof value === "object") {
    return Object.values(value).flatMap(collectNumbers);
  }
  return [];
}

// Arithmetic mean of the samples (the caller guarantees a non-empty array).
function mean(samples) {
  return samples.reduce((total, sample) => total + sample, 0) / samples.length;
}

// Median of the samples; averages the two middle values for an even count.
function median(samples) {
  const sorted = [...samples].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0
    ? (sorted[middle - 1] + sorted[middle]) / 2
    : sorted[middle];
}

async function main() {
  // 1. Read the target URL from the first command-line argument.
  const url = process.argv[2];
  if (!url) throw new Error("usage: node stats.js <url-returning-json>");

  // 2. Make the HTTP GET request and parse the JSON body.
  const response = await fetch(url);
  // region:error_handling
  // Fail fast on a non-2xx status before we try to decode the body.
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} ${response.statusText}`);
  }
  // endregion:error_handling
  const document = await response.json();

  // 3. Gather every number from the decoded JSON.
  const numbers = collectNumbers(document);
  // region:error_handling
  if (numbers.length === 0) {
    throw new Error("the JSON response contained no numbers");
  }
  // endregion:error_handling

  // 4. Compute and print the statistics.
  console.log(`count:  ${numbers.length}`);
  console.log(`mean:   ${mean(numbers).toFixed(4)}`);
  console.log(`median: ${median(numbers).toFixed(4)}`);
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
"#;

pub(super) const PYTHON_PERSONAL_BUDGET_REPORT: &str = r#"from dataclasses import dataclass
from math import log
from pathlib import Path


@dataclass(frozen=True)
class CityCost:
    city: str
    average_rent: float
    living_cost_ex_rent: float
    source: str


CITY_COSTS = [
    CityCost('Moscow', 950.0, 850.0, 'https://www.numbeo.com/cost-of-living/in/Moscow'),
    CityCost('Berlin', 1550.0, 1250.0, 'https://www.numbeo.com/cost-of-living/in/Berlin'),
    CityCost('New York', 3600.0, 1850.0, 'https://www.numbeo.com/cost-of-living/in/New-York'),
]
ANNUAL_RETURN = 0.08
GOAL = 100_000.0


def budget_50_30_20(monthly_income):
    return {
        'needs': monthly_income * 0.50,
        'wants': monthly_income * 0.30,
        'savings': monthly_income * 0.20,
    }


def future_value(monthly_contribution, annual_return, years):
    monthly_rate = annual_return / 12
    months = years * 12
    return monthly_contribution * (((1 + monthly_rate) ** months - 1) / monthly_rate)


def years_to_goal(monthly_savings, goal=GOAL, annual_return=ANNUAL_RETURN):
    if monthly_savings <= 0:
        return None
    monthly_rate = annual_return / 12
    months = log(1 + goal * monthly_rate / monthly_savings) / log(1 + monthly_rate)
    return months / 12


def money(value):
    return f'${value:,.0f}'


def years(value):
    return 'not reachable' if value is None else f'{value:.1f}'


def comparison_rows(monthly_income):
    plan = budget_50_30_20(monthly_income)
    rows = []
    for cost in CITY_COSTS:
        expenses = cost.average_rent + cost.living_cost_ex_rent
        remaining = monthly_income - expenses
        monthly_savings = max(0.0, min(plan['savings'], remaining))
        rows.append({
            'city': cost.city,
            'rent': cost.average_rent,
            'remaining': remaining,
            'monthly_savings': monthly_savings,
            'years_to_100k': years_to_goal(monthly_savings),
            'source': cost.source,
        })
    return rows


def render_markdown(monthly_income):
    plan = budget_50_30_20(monthly_income)
    scenario_savings = 3000.0 * 0.20
    scenario_future = future_value(scenario_savings, ANNUAL_RETURN, 10)
    lines = [
        '# Budget Calculator Report',
        '',
        '## 50/30/20 Budget',
        f'- Monthly income: {money(monthly_income)}',
        f'- Needs (50%): {money(plan["needs"])}',
        f'- Wants (30%): {money(plan["wants"])}',
        f'- Savings (20%): {money(plan["savings"])}',
        '',
        '## Investment Scenario',
        f'- 20% of $3000 monthly: {money(scenario_savings)}',
        f'- Future value after 10 years at 8% annual return: {money(scenario_future)}',
        '',
        '## City Comparison',
        '| City | Average rent | Remaining budget after expenses | Years to save $100,000 |',
        '| --- | ---: | ---: | ---: |',
    ]
    for row in comparison_rows(monthly_income):
        lines.append(
            f'| {row["city"]} | {money(row["rent"])} | '
            f'{money(row["remaining"])} | {years(row["years_to_100k"])} |'
        )
    lines.extend(['', '## Sources'])
    for cost in CITY_COSTS:
        lines.append(f'- {cost.city}: {cost.source}')
    lines.append('')
    lines.append('Review and update the city-cost values after checking the source pages.')
    return '\n'.join(lines)


def read_income():
    try:
        raw = input('Monthly income in USD [3000]: ').strip()
    except EOFError:
        raw = ''
    return float(raw or '3000')


def main():
    report = render_markdown(read_income())
    Path('budget_report.md').write_text(report, encoding='utf-8')
    print(report)
    print('\nMarkdown report written to budget_report.md')


if __name__ == '__main__':
    main()
"#;

pub(super) const PYTHON_CRYPTO_PORTFOLIO_TRACKER: &str = r#"from dataclasses import dataclass
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
    """Mock a public price API response for deterministic offline execution."""
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
    main()
"#;

pub(super) const RUST_SELF_SOURCE_METRICS: &str = r#"use std::fmt::Write as _;

const SOURCE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", file!()));
const RESPONSE_REASONING: &str = "The response decomposes the request, provides a source-metrics program, and compares the code with this prose.";

#[derive(Default)]
struct Metrics {
    functions: usize,
    loops: usize,
    conditionals: usize,
    comments: usize,
    boolean_branches: usize,
    complexity_score: usize,
}

fn main() {
    let source_metrics = analyze_rust_text(SOURCE);
    let reasoning_metrics = analyze_rust_text(RESPONSE_REASONING);
    println!("{}", render_report(&source_metrics, &reasoning_metrics));
}

fn analyze_rust_text(text: &str) -> Metrics {
    let (sanitized, comments) = sanitize_rust_text(text);
    let tokens = rust_tokens(&sanitized);
    let loops = count_any(&tokens, &["for", "while", "loop"]);
    let conditionals = count_any(&tokens, &["if", "match"]);
    let boolean_branches = sanitized.matches("&&").count() + sanitized.matches("||").count();
    Metrics {
        functions: count_any(&tokens, &["fn"]),
        loops,
        conditionals,
        comments,
        boolean_branches,
        complexity_score: 1 + loops + conditionals + boolean_branches,
    }
}

fn sanitize_rust_text(text: &str) -> (String, usize) {
    let mut output = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut comments = 0;
    let mut index = 0;
    while index < chars.len() {
        match (chars[index], chars.get(index + 1)) {
            ('/', Some('/')) => {
                comments += 1;
                output.push(' ');
                output.push(' ');
                index += 2;
                while index < chars.len() && chars[index] != '\n' {
                    output.push(' ');
                    index += 1;
                }
            }
            ('/', Some('*')) => {
                comments += 1;
                output.push(' ');
                output.push(' ');
                index += 2;
                while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                    output.push(if chars[index] == '\n' { '\n' } else { ' ' });
                    index += 1;
                }
                if index + 1 < chars.len() {
                    output.push(' ');
                    output.push(' ');
                    index += 2;
                }
            }
            ('"', _) => {
                output.push(' ');
                index += 1;
                while index < chars.len() {
                    let current = chars[index];
                    output.push(if current == '\n' { '\n' } else { ' ' });
                    index += if current == '\\' && index + 1 < chars.len() { 2 } else { 1 };
                    if current == '"' {
                        break;
                    }
                }
            }
            _ => {
                output.push(chars[index]);
                index += 1;
            }
        }
    }
    (output, comments)
}

fn rust_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        if character == '_' || character.is_ascii_alphanumeric() {
            current.push(character);
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn count_any(tokens: &[String], needles: &[&str]) -> usize {
    tokens
        .iter()
        .filter(|token| needles.iter().any(|needle| token.as_str() == *needle))
        .count()
}

fn render_report(source: &Metrics, reasoning: &Metrics) -> String {
    let verdict = if source.complexity_score > reasoning.complexity_score {
        "generated_code"
    } else if source.complexity_score < reasoning.complexity_score {
        "reasoning_text"
    } else {
        "tie"
    };
    let explanation = if verdict == "generated_code" {
        "The generated Rust code is more complex because it contains functions, loops, conditionals, and comment syntax; the reasoning text is plain prose."
    } else if verdict == "reasoning_text" {
        "The reasoning text is more complex under this scanner."
    } else {
        "Both texts have the same computed complexity score."
    };
    let mut report = String::new();
    report.push_str("{\n");
    push_metrics(&mut report, "source_code", source, true);
    push_metrics(&mut report, "response_reasoning_text", reasoning, true);
    writeln!(report, "  \"more_complex\": \"{}\",", verdict).unwrap();
    writeln!(report, "  \"comparison\": {}", json_string(explanation)).unwrap();
    report.push('}');
    report
}

fn push_metrics(report: &mut String, label: &str, metrics: &Metrics, comma: bool) {
    writeln!(report, "  \"{}\": {{", label).unwrap();
    writeln!(report, "    \"functions\": {},", metrics.functions).unwrap();
    writeln!(report, "    \"loops\": {},", metrics.loops).unwrap();
    writeln!(report, "    \"conditionals\": {},", metrics.conditionals).unwrap();
    writeln!(report, "    \"comments\": {},", metrics.comments).unwrap();
    writeln!(report, "    \"boolean_branches\": {},", metrics.boolean_branches).unwrap();
    writeln!(report, "    \"complexity_score\": {}", metrics.complexity_score).unwrap();
    writeln!(report, "  }}{}", if comma { "," } else { "" }).unwrap();
}

fn json_string(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    format!("\"{}\"", escaped)
}
"#;
