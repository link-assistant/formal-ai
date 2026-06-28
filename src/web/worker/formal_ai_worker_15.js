// Worker module 16 of 21. Loaded by ../formal_ai_worker.js.
const BLUEPRINT_PYTHON_HTTP_JSON_STATS = `"""Fetch JSON from a URL and report the mean and median of every number in it.

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
`;

const BLUEPRINT_JAVASCRIPT_HTTP_JSON_STATS = `// Fetch JSON from a URL and report the mean and median of every number in it.
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
    throw new Error(\`HTTP \${response.status} \${response.statusText}\`);
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
  console.log(\`count:  \${numbers.length}\`);
  console.log(\`mean:   \${mean(numbers).toFixed(4)}\`);
  console.log(\`median: \${median(numbers).toFixed(4)}\`);
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
`;

const BLUEPRINT_PYTHON_PERSONAL_BUDGET_REPORT = `from dataclasses import dataclass
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
    return f'$\{value:,.0f}'


def years(value):
    return 'not reachable' if value is None else f'\{value:.1f}'


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
        f'- Monthly income: \{money(monthly_income)}',
        f'- Needs (50%): \{money(plan["needs"])}',
        f'- Wants (30%): \{money(plan["wants"])}',
        f'- Savings (20%): \{money(plan["savings"])}',
        '',
        '## Investment Scenario',
        f'- 20% of $3000 monthly: \{money(scenario_savings)}',
        f'- Future value after 10 years at 8% annual return: \{money(scenario_future)}',
        '',
        '## City Comparison',
        '| City | Average rent | Remaining budget after expenses | Years to save $100,000 |',
        '| --- | ---: | ---: | ---: |',
    ]
    for row in comparison_rows(monthly_income):
        lines.append(
            f'| \{row["city"]} | \{money(row["rent"])} | '
            f'\{money(row["remaining"])} | \{years(row["years_to_100k"])} |'
        )
    lines.extend(['', '## Sources'])
    for cost in CITY_COSTS:
        lines.append(f'- \{cost.city}: \{cost.source}')
    lines.append('')
    lines.append('Review and update the city-cost values after checking the source pages.')
    return '\\n'.join(lines)


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
    print('\\nMarkdown report written to budget_report.md')


if __name__ == '__main__':
    main()
`;

const BLUEPRINT_PYTHON_CRYPTO_PORTFOLIO_TRACKER = `from dataclasses import dataclass
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
    return f'$\{value:,.2f}'


def render_dashboard(rows: list[dict[str, float | str]], notices: list[str]) -> str:
    total_value = sum(float(row['value_usd']) for row in rows)
    lines = [
        '# Crypto Portfolio Dashboard',
        '',
        f'**Total value:** \{money(total_value)}',
        '',
        '| Asset | Amount | Price USD | Value USD | 24h change | Portfolio weight |',
        '| --- | ---: | ---: | ---: | ---: | ---: |',
    ]
    for row in rows:
        lines.append(
            f'| \{row["symbol"]} | \{row["amount"]:,.4g} | \{money(float(row["price_usd"]))} | '
            f'\{money(float(row["value_usd"]))} | \{row["change_24h_pct"]:.2f}% | '
            f'\{row["portfolio_weight"]:.2f}% |'
        )
    lines.extend(['', '## Alerts'])
    lines.extend(notices or ['No asset dropped more than 5% in the last 24h.'])
    return '\\n'.join(lines)


def main() -> None:
    prices = fetch_prices(list(PORTFOLIO))
    rows = portfolio_rows(PORTFOLIO, prices)
    notices = notify_alerts(rows)
    formatted_log = render_dashboard(rows, notices)
    print(formatted_log)


if __name__ == '__main__':
    main()
`;

const BLUEPRINT_PYTHON_SMART_TRAVEL_PLANNER = `from dataclasses import dataclass


@dataclass(frozen=True)
class DestinationData:
    country: str
    visa_free: bool
    visa_note: str
    average_flight_cost: float
    daily_cost: float
    source: str


DEFAULT_DESTINATIONS = {
    'Japan': DestinationData(
        country='Japan',
        visa_free=False,
        visa_note='Russian citizens should confirm visa requirements before booking.',
        average_flight_cost=820.0,
        daily_cost=160.0,
        source='https://www.mofa.go.jp/',
    ),
    'UAE': DestinationData(
        country='UAE',
        visa_free=True,
        visa_note='Assume visa-free short-stay access; verify current entry rules.',
        average_flight_cost=420.0,
        daily_cost=135.0,
        source='https://u.ae/',
    ),
    'Serbia': DestinationData(
        country='Serbia',
        visa_free=True,
        visa_note='Assume visa-free short-stay access; verify current entry rules.',
        average_flight_cost=380.0,
        daily_cost=95.0,
        source='https://www.mfa.gov.rs/',
    ),
}
DEFAULT_DAYS = 7


def money(value):
    return f'$\{value:,.0f}'


class TravelPlanner:
    def __init__(self, destination_data=None):
        self.destination_data = destination_data or DEFAULT_DESTINATIONS
        self.destinations = []

    def add_destination(self, country: str, budget: float):
        lookup = {name.lower(): name for name in self.destination_data}
        key = country.strip().lower()
        if key not in lookup:
            available = ', '.join(sorted(self.destination_data))
            raise ValueError(f'Unknown destination {country!r}. Try one of: {available}')
        self.destinations.append({
            'country': lookup[key],
            'budget': float(budget),
        })

    def check_visa_requirements(self) -> bool:
        return all(
            self.destination_data[item['country']].visa_free
            for item in self._selected_destinations()
        )

    def estimate_total_cost(self) -> dict:
        return self._estimate_for_days(DEFAULT_DAYS)

    def generate_itinerary(self, days: int) -> str:
        estimates = self._estimate_for_days(days)
        ranked = sorted(
            estimates.values(),
            key=lambda row: (not row['visa_free'], row['estimated_total']),
        )
        lines = [
            '# Smart Travel Planner',
            '',
            f'Sample output for a {days}-day trip with $2,000 budget',
            '',
            '| Rank | Destination | Visa | Estimated cost | Budget status |',
            '| ---: | --- | --- | ---: | --- |',
        ]
        for rank, row in enumerate(ranked, start=1):
            visa = 'visa-free' if row['visa_free'] else 'visa check required'
            warning = row['budget_warning'] or 'Budget warning: none'
            lines.append(
                f'| {rank} | {row["country"]} | {visa} | '
                f'{money(row["estimated_total"])} | {warning} |'
            )
        lines.extend(['', '## Notes'])
        for row in ranked:
            lines.append(f'- {row["country"]}: {row["visa_note"]}')
            lines.append(f'  Source to review: {row["source"]}')
        lines.append('')
        lines.append('Review live visa rules and flight prices before purchase.')
        return '\\n'.join(lines)

    def _selected_destinations(self):
        if not self.destinations:
            raise ValueError('Add at least one destination first.')
        return self.destinations

    def _estimate_for_days(self, days):
        estimates = {}
        for item in self._selected_destinations():
            data = self.destination_data[item['country']]
            total = data.average_flight_cost + data.daily_cost * days
            budget = item['budget']
            estimates[data.country] = {
                'country': data.country,
                'visa_free': data.visa_free,
                'visa_note': data.visa_note,
                'source': data.source,
                'average_flight_cost': data.average_flight_cost,
                'daily_cost': data.daily_cost,
                'estimated_total': total,
                'budget': budget,
                'budget_warning': (
                    None
                    if budget >= total
                    else f'Budget warning: {money(budget)} is below {money(total)}'
                ),
            }
        return estimates


def build_sample_planner():
    planner = TravelPlanner()
    for country in ('Japan', 'UAE', 'Serbia'):
        planner.add_destination(country, 2000.0)
    return planner


if __name__ == '__main__':
    sample = build_sample_planner()
    print('Sample output for a 7-day trip with $2,000 budget')
    print(sample.generate_itinerary(7))
`;

const BLUEPRINT_RUST_SELF_SOURCE_METRICS = `use std::fmt::Write as _;

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
                while index < chars.len() && chars[index] != '\\n' {
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
                    output.push(if chars[index] == '\\n' { '\\n' } else { ' ' });
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
                    output.push(if current == '\\n' { '\\n' } else { ' ' });
                    index += if current == '\\\\' && index + 1 < chars.len() { 2 } else { 1 };
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
    report.push_str("{\\n");
    push_metrics(&mut report, "source_code", source, true);
    push_metrics(&mut report, "response_reasoning_text", reasoning, true);
    writeln!(report, "  \\"more_complex\\": \\"{}\\",", verdict).unwrap();
    writeln!(report, "  \\"comparison\\": {}", json_string(explanation)).unwrap();
    report.push('}');
    report
}

fn push_metrics(report: &mut String, label: &str, metrics: &Metrics, comma: bool) {
    writeln!(report, "  \\"{}\\": {{", label).unwrap();
    writeln!(report, "    \\"functions\\": {},", metrics.functions).unwrap();
    writeln!(report, "    \\"loops\\": {},", metrics.loops).unwrap();
    writeln!(report, "    \\"conditionals\\": {},", metrics.conditionals).unwrap();
    writeln!(report, "    \\"comments\\": {},", metrics.comments).unwrap();
    writeln!(report, "    \\"boolean_branches\\": {},", metrics.boolean_branches).unwrap();
    writeln!(report, "    \\"complexity_score\\": {}", metrics.complexity_score).unwrap();
    writeln!(report, "  }}{}", if comma { "," } else { "" }).unwrap();
}

fn json_string(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\\\\""),
            '\\\\' => escaped.push_str("\\\\\\\\"),
            '\\n' => escaped.push_str("\\\\n"),
            '\\r' => escaped.push_str("\\\\r"),
            '\\t' => escaped.push_str("\\\\t"),
            other => escaped.push(other),
        }
    }
    format!("\\"{}\\"", escaped)
}
`;

// Curated composite recipes. Mirrors `RECIPES` in `src/coding/blueprint.rs`.
const BLUEPRINT_RECIPES = [
  {
    slug: "http_json_stats",
    label: "fetch JSON over HTTP and report the mean and median of its numbers",
    requiredCapabilities: ["http_request", "json_parse", "statistics"],
    programs: [
      {
        languageSlug: "rust",
        libraries: ["reqwest (blocking, json)", "serde_json"],
        runCommand: "cargo run -- <url-returning-json>",
        execution: "external_libraries_and_network",
        code: BLUEPRINT_RUST_HTTP_JSON_STATS,
      },
      {
        languageSlug: "python",
        libraries: ["requests"],
        runCommand: "python stats.py <url-returning-json>",
        execution: "external_libraries_and_network",
        code: BLUEPRINT_PYTHON_HTTP_JSON_STATS,
      },
      {
        languageSlug: "javascript",
        libraries: ["Node.js 18+ (built-in global fetch; no extra packages)"],
        runCommand: "node stats.js <url-returning-json>",
        execution: "external_libraries_and_network",
        code: BLUEPRINT_JAVASCRIPT_HTTP_JSON_STATS,
      },
    ],
  },
  {
    slug: "personal_budget_report",
    label: "build a sourced 50/30/20 city budget calculator and Markdown report",
    requiredCapabilities: [
      "web_research",
      "city_costs",
      "budget_rule",
      "compound_savings",
      "markdown_report",
    ],
    programs: [
      {
        languageSlug: "python",
        libraries: ["Python 3 standard library only"],
        runCommand: "python budget_report.py",
        execution: "review_data_assumptions",
        code: BLUEPRINT_PYTHON_PERSONAL_BUDGET_REPORT,
      },
    ],
  },
  {
    slug: "smart_travel_planner",
    label:
      "build a sourced travel planner class with visa, flight-cost, budget, and itinerary logic",
    requiredCapabilities: [
      "web_research",
      "visa_requirements",
      "flight_costs",
      "travel_planner_class",
      "budget_flags",
      "sample_itinerary",
    ],
    programs: [
      {
        languageSlug: "python",
        libraries: ["Python 3 standard library only"],
        runCommand: "python travel_planner.py",
        execution: "review_data_assumptions",
        code: BLUEPRINT_PYTHON_SMART_TRAVEL_PLANNER,
      },
    ],
  },
  {
    slug: "self_source_metrics_report",
    label:
      "inspect its own Rust source, emit JSON metrics, and compare code with response prose",
    requiredCapabilities: [
      "source_text",
      "source_metrics",
      "complexity_score",
      "json_report",
      "self_response_analysis",
      "complexity_comparison",
    ],
    programs: [
      {
        languageSlug: "rust",
        libraries: ["Rust standard library only"],
        runCommand: "cargo run",
        execution: "local_source_analysis",
        code: BLUEPRINT_RUST_SELF_SOURCE_METRICS,
      },
    ],
  },
  {
    slug: "crypto_portfolio_tracker",
    label: "simulate a crypto portfolio tracker with alerts and a Markdown dashboard",
    requiredCapabilities: [
      "crypto_prices",
      "portfolio_holdings",
      "portfolio_calculations",
      "alert_logic",
      "mock_api",
      "markdown_report",
    ],
    programs: [
      {
        languageSlug: "python",
        libraries: ["Python 3 standard library only"],
        runCommand: "python crypto_portfolio.py",
        execution: "review_data_assumptions",
        code: BLUEPRINT_PYTHON_CRYPTO_PORTFOLIO_TRACKER,
      },
    ],
  },
];

// Mirror of `contains_keyword` in `src/coding/blueprint.rs`: CJK keywords match
// by substring; multi-word phrases match by substring; single words match on
// token boundaries, allowing a stem prefix when len >= 4.
function blueprintContainsKeyword(normalized, keyword) {
  if (containsCjk(keyword)) return normalized.includes(keyword);
  if (keyword.includes(" ")) return normalized.includes(keyword);
  return normalized
    .split(/[^\p{L}\p{N}\p{M}]+/u)
    .some(
      (token) =>
        token === keyword || (keyword.length >= 4 && token.startsWith(keyword)),
    );
}

// Detect which capabilities a normalized prompt requests, in catalog order so
// the decomposition plan reads top-to-bottom. Mirrors `detect_capabilities`.
function detectBlueprintCapabilities(normalized) {
  return BLUEPRINT_CAPABILITIES.filter((capability) =>
    capability.keywords.some((keyword) =>
      blueprintContainsKeyword(normalized, keyword),
    ),
  );
}

// Resolve a blueprint: the first recipe whose required capabilities are all
// present and that has a program for `languageSlug`. Mirrors `select_blueprint`.
function selectBlueprint(normalized, languageSlug) {
  const detected = detectBlueprintCapabilities(normalized);
  const detectedSlugs = detected.map((capability) => capability.slug);
  const recipe = BLUEPRINT_RECIPES.find(
    (candidate) =>
      candidate.requiredCapabilities.every((required) =>
        detectedSlugs.includes(required),
      ) &&
      candidate.programs.some((program) => program.languageSlug === languageSlug),
  );
  if (!recipe) return null;
  const program = recipe.programs.find(
    (candidate) => candidate.languageSlug === languageSlug,
  );
  if (!program) return null;
  return { recipe, program, capabilities: detected };
}

// Localized framing strings for the blueprint render. Binary-matching mirror of
// the `blueprint_intro`/`capability_label`/`libraries_heading`/
// `blueprint_execution_report`/`recipe_summary`/`how_to_run_heading` functions
// in `src/coding/blueprint.rs`. `en` is the fallback.
const BLUEPRINT_I18N = {
  en: {
    intro: (name, label) =>
      `Here is a ${name} program for the requested composite task (${label}). I decomposed your request into these sub-tasks:`,
    librariesHeading: "Required libraries:",
    howToRunHeading: "How to run it yourself:",
    executionReport: (runCommand, execution) =>
      execution === "local_source_analysis"
        ? `Execution status: not run — this source-metrics blueprint uses only the Rust standard library, but the answer renderer did not compile it in place. The code is provided for review. Run it yourself from a Cargo project: \`${runCommand}\`.`
        : execution === "review_data_assumptions"
        ? `Execution status: not run — this report blueprint was not executed in the offline sandbox, and the embedded data assumptions should be reviewed before use. The code is provided for review. Run it yourself: \`${runCommand}\`.`
        : `Execution status: not run — this program needs external libraries and network access, so the offline sandbox does not execute it. The code is provided for review. Run it yourself: \`${runCommand}\`.`,
  },
  ru: {
    intro: (name, label) =>
      `Вот программа на языке ${name}, которая решает составную задачу (${label}). Я разбил ваш запрос на следующие подзадачи:`,
    librariesHeading: "Необходимые библиотеки:",
    howToRunHeading: "Как запустить самостоятельно:",
    executionReport: (runCommand, execution) =>
      execution === "local_source_analysis"
        ? `Статус выполнения: не запускалось — этот чертёж анализа исходного кода использует только стандартную библиотеку Rust, но генератор ответа не компилировал его на месте. Код приведён для проверки. Запустить из Cargo-проекта: \`${runCommand}\`.`
        : execution === "review_data_assumptions"
        ? `Статус выполнения: не запускалось — этот отчёт не выполнялся в офлайн-песочнице, а встроенные допущения о данных нужно проверить перед использованием. Код приведён для проверки. Запустить самостоятельно: \`${runCommand}\`.`
        : `Статус выполнения: не запускалось — программе нужны внешние библиотеки и доступ к сети, поэтому офлайн-песочница её не выполняет. Код приведён для проверки. Запустить самостоятельно: \`${runCommand}\`.`,
  },
  hi: {
    intro: (name, label) =>
      `यहाँ ${name} में एक प्रोग्राम है जो इस संयुक्त कार्य को हल करता है (${label})। मैंने आपके अनुरोध को इन उप-कार्यों में विभाजित किया है:`,
    librariesHeading: "आवश्यक लाइब्रेरियाँ:",
    howToRunHeading: "इसे स्वयं कैसे चलाएँ:",
    executionReport: (runCommand, execution) =>
      execution === "local_source_analysis"
        ? `निष्पादन स्थिति: नहीं चलाया गया — यह source-metrics blueprint केवल Rust standard library का उपयोग करता है, लेकिन answer renderer ने इसे यहीं compile नहीं किया। कोड समीक्षा के लिए दिया गया है। Cargo project से स्वयं चलाएँ: \`${runCommand}\`।`
        : execution === "review_data_assumptions"
        ? `निष्पादन स्थिति: नहीं चलाया गया — यह रिपोर्ट ऑफ़लाइन सैंडबॉक्स में नहीं चली, और embedded data assumptions को उपयोग से पहले जाँचना चाहिए। कोड समीक्षा के लिए दिया गया है। स्वयं चलाएँ: \`${runCommand}\`।`
        : `निष्पादन स्थिति: नहीं चलाया गया — प्रोग्राम को बाहरी लाइब्रेरियों और नेटवर्क पहुँच की आवश्यकता है, इसलिए ऑफ़लाइन सैंडबॉक्स इसे नहीं चलाता। कोड समीक्षा के लिए दिया गया है। स्वयं चलाएँ: \`${runCommand}\`।`,
  },
  zh: {
    intro: (name, label) =>
      `这是一个解决该复合任务的 ${name} 程序（${label}）。我已将您的请求分解为以下子任务：`,
    librariesHeading: "所需的库：",
    howToRunHeading: "如何自行运行：",
    executionReport: (runCommand, execution) =>
      execution === "local_source_analysis"
        ? `执行状态：未运行 —— 该源码指标蓝图只使用 Rust 标准库，但回答渲染器没有在本地编译它。代码仅供审阅。请在 Cargo 项目中自行运行：\`${runCommand}\`。`
        : execution === "review_data_assumptions"
        ? `执行状态：未运行 —— 该报告未在离线沙箱中执行，内置数据假设应先核对再使用。代码仅供审阅。自行运行：\`${runCommand}\`。`
        : `执行状态：未运行 —— 该程序需要外部库和网络访问，因此离线沙箱不会执行它。代码仅供审阅。自行运行：\`${runCommand}\`。`,
  },
};

// Localized capability labels keyed by `${slug}:${language}`; English falls back
// to the capability's own `label`. Mirrors `capability_label`.
const BLUEPRINT_CAPABILITY_LABELS = {
  "http_request:ru": "Выполнить HTTP-запрос",
  "http_request:hi": "HTTP अनुरोध करें",
  "http_request:zh": "发起 HTTP 请求",
  "json_parse:ru": "Разобрать JSON-ответ",
  "json_parse:hi": "JSON प्रतिक्रिया पार्स करें",
  "json_parse:zh": "解析 JSON 响应",
  "statistics:ru": "Вычислить статистику (среднее, медиана)",
  "statistics:hi": "सांख्यिकी (औसत, माध्यिका) की गणना करें",
  "statistics:zh": "计算统计量（平均值、中位数）",
  "output_results:ru": "Вывести результаты",
  "output_results:hi": "परिणाम आउटपुट करें",
  "output_results:zh": "输出结果",
  "error_handling:ru": "Обработать ошибки",
  "error_handling:hi": "त्रुटियाँ संभालें",
  "error_handling:zh": "处理错误",
  "comments:ru": "Снабдить код комментариями",
  "comments:hi": "कोड में टिप्पणियाँ जोड़ें",
  "comments:zh": "为代码添加注释",
  "web_research:ru": "Найти актуальные исходные данные",
  "web_research:hi": "वर्तमान स्रोत डेटा खोजें",
  "web_research:zh": "检索当前来源数据",
  "city_costs:ru": "Сравнить стоимость жизни по городам",
  "city_costs:hi": "शहरों की जीवन-यापन लागत की तुलना करें",
  "city_costs:zh": "比较城市生活成本",
  "budget_rule:ru": "Применить правило бюджета 50/30/20",
  "budget_rule:hi": "50/30/20 बजट नियम लागू करें",
  "budget_rule:zh": "应用 50/30/20 预算规则",
  "compound_savings:ru": "Рассчитать накопления со сложным процентом",
  "compound_savings:hi": "चक्रवृद्धि बचत का अनुमान लगाएँ",
  "compound_savings:zh": "预测复利储蓄",
  "markdown_report:ru": "Экспортировать Markdown-отчёт со сравнением",
  "markdown_report:hi": "Markdown तुलना रिपोर्ट निर्यात करें",
  "markdown_report:zh": "导出 Markdown 比较报告",
};

// Localized one-line recipe summaries keyed by `${slug}:${language}`; English
// falls back to the recipe's own `label`. Mirrors `recipe_summary`.
const BLUEPRINT_RECIPE_SUMMARIES = {
  "http_json_stats:ru": "загрузить JSON по HTTP и вывести среднее и медиану его чисел",
  "http_json_stats:hi":
    "HTTP के माध्यम से JSON प्राप्त करें और उसकी संख्याओं का औसत और माध्यिका दिखाएँ",
  "http_json_stats:zh": "通过 HTTP 获取 JSON 并报告其中数字的平均值和中位数",
  "personal_budget_report:ru":
    "собрать бюджетный калькулятор 50/30/20 с городскими расходами, источниками и Markdown-отчётом",
  "personal_budget_report:hi":
    "स्रोतों सहित 50/30/20 शहर बजट कैलकुलेटर और Markdown रिपोर्ट बनाएँ",
  "personal_budget_report:zh":
    "生成带来源的 50/30/20 城市预算计算器和 Markdown 报告",
  "self_source_metrics_report:ru":
    "проанализировать собственный Rust-код, вывести JSON-метрики и сравнить код с текстом ответа",
  "self_source_metrics_report:hi":
    "अपने Rust source का निरीक्षण करें, JSON metrics निकालें, और code को response prose से तुलना करें",
  "self_source_metrics_report:zh":
    "检查自身 Rust 源码，输出 JSON 指标，并比较代码与回答文本",
  "crypto_portfolio_tracker:ru":
    "смоделировать криптопортфель с оповещениями и Markdown-панелью",
  "crypto_portfolio_tracker:hi":
    "alerts और Markdown dashboard वाला crypto portfolio tracker simulate करें",
  "crypto_portfolio_tracker:zh":
    "模拟带提醒和 Markdown 仪表盘的加密投资组合追踪器",
};

// The line-comment marker for a program language. Mirrors `comment_marker` in
// `src/coding/blueprint.rs`: hash-comment languages use `#`, everything else
// uses `//` (which also covers Rust's `//!`/`///` doc lines).
function blueprintCommentMarker(languageSlug) {
  return languageSlug === "python" || languageSlug === "ruby" ? "#" : "//";
}

// Region directive prefixes. Mirrors `REGION_OPEN`/`REGION_CLOSE` in
// `src/coding/blueprint.rs`.
const BLUEPRINT_REGION_OPEN = "region:";
const BLUEPRINT_REGION_CLOSE = "endregion:";

// Normalize a composition strategy value to the canonical "composed" (default,
// the most promising direction) or "documented". Mirrors
// `BlueprintComposition::from_value`/`default` in `src/solver.rs`.
function normalizeBlueprintComposition(value) {
  switch (String(value || "").trim().toLowerCase()) {
    case "documented":
    case "document":
    case "full":
    case "verbatim":
    case "curated":
      return "documented";
    default:
      return "composed";
  }
}

// If `trimmed` (leading whitespace already removed) is a region directive
// comment, return `{ open, slug }`; otherwise null. Mirrors `region_directive`
// in `src/coding/blueprint.rs`.
function blueprintRegionDirective(trimmed, marker) {
  if (!trimmed.startsWith(marker)) return null;
  const rest = trimmed.slice(marker.length).replace(/^\s+/, "");
  if (rest.startsWith(BLUEPRINT_REGION_CLOSE)) {
    return { open: false, slug: rest.slice(BLUEPRINT_REGION_CLOSE.length).trim() };
  }
  if (rest.startsWith(BLUEPRINT_REGION_OPEN)) {
    return { open: true, slug: rest.slice(BLUEPRINT_REGION_OPEN.length).trim() };
  }
  return null;
}

// Compose the program a blueprint emits from its annotated recipe template.
// Byte-for-byte mirror of `compose_program` in `src/coding/blueprint.rs`: drop
// every region directive line, and in the "composed" strategy drop the body of
// any region whose capability the request did not name and strip whole-line
// documentation when comments were not requested. The "documented" strategy
// keeps every region and comment (only the internal directive lines go).
function composeBlueprintProgram(blueprint, strategy) {
  const languageSlug = blueprint.program.languageSlug;
  const marker = blueprintCommentMarker(languageSlug);
  const compose = strategy !== "documented";
  const requested = (slug) =>
    blueprint.capabilities.some((capability) => capability.slug === slug);

  // Pass 1 — regions: drop every directive line, and when composing drop the
  // body of any region whose capability was not requested.
  const kept = [];
  let skipping = false;
  for (const line of blueprint.program.code.split("\n")) {
    const trimmed = line.replace(/^\s+/, "");
    const directive = blueprintRegionDirective(trimmed, marker);
    if (directive) {
      skipping = directive.open && compose && !requested(directive.slug);
      continue;
    }
    if (!skipping) kept.push(line);
  }

  // Pass 2 — comments: strip documentation when composing a request that did
  // not ask for it; otherwise keep the comments and just tidy blank runs.
  if (compose && !blueprintWantsComments(blueprint)) {
    return stripBlueprintCommentLines(kept, languageSlug);
  }
  return collapseBlueprintBlankRuns(kept);
}

// Remove the documentation from already region-filtered lines when the request
// did not ask for comments. Mirrors `strip_comment_lines` in
// `src/coding/blueprint.rs`: only whole-line comments and a leading Python
// module docstring are dropped (both non-semantic, so the stripped program
// stays compilable), and inline trailing comments are left untouched so a
// `//`/`#` inside a string literal is never sliced.
function stripBlueprintCommentLines(lines, languageSlug) {
  const marker = blueprintCommentMarker(languageSlug);
  const kept = [];
  let inDocstring = false;
  for (const line of lines) {
    const trimmed = line.replace(/^\s+/, "");
    if (languageSlug === "python") {
      if (inDocstring) {
        if (trimmed.includes('"""')) inDocstring = false;
        continue;
      }
      if (trimmed.startsWith('"""')) {
        const rest = trimmed.slice(3);
        if (!rest.includes('"""')) inDocstring = true;
        continue;
      }
    }
    if (trimmed.startsWith(marker)) continue;
    kept.push(line);
  }
  return collapseBlueprintBlankRuns(kept);
}

// Join kept lines, dropping leading blanks and collapsing runs of two or more
// blank lines (left after removing comment blocks) into one. Mirrors
// `collapse_blank_runs` in `src/coding/blueprint.rs`.
function collapseBlueprintBlankRuns(lines) {
  const out = [];
  let pendingBlank = false;
  let wroteAny = false;
  for (const line of lines) {
    if (line.trim() === "") {
      if (wroteAny) pendingBlank = true;
      continue;
    }
    if (pendingBlank) {
      out.push("");
      pendingBlank = false;
    }
    out.push(line);
    wroteAny = true;
  }
  return out.join("\n");
}

// Whether the decomposed request asked for the code to be commented. Mirrors
// `wants_comments` in `src/coding/blueprint.rs`.
function blueprintWantsComments(blueprint) {
  return blueprint.capabilities.some(
    (capability) => capability.slug === "comments",
  );
}

function blueprintCapabilityLabel(capability, language) {
  return (
    BLUEPRINT_CAPABILITY_LABELS[`${capability.slug}:${language}`] ||
    capability.label
  );
}

function blueprintRecipeSummary(recipe, language) {
  return BLUEPRINT_RECIPE_SUMMARIES[`${recipe.slug}:${language}`] || recipe.label;
}

function blueprintRecipeAddendum(recipe) {
  if (recipe.slug !== "self_source_metrics_report") return "";
  return [
    "Response self-analysis:",
    "- Reasoning text metrics: functions=0, loops=0, conditionals=0, comments=0, complexity_score=1.",
    "- Comparison: the generated Rust code is more complex than the reasoning text because it contains executable parsing, loops, conditionals, helper functions, and JSON rendering logic.",
  ].join("\n");
}

// Render the complete localized blueprint answer — decomposition plan, the
// curated program, its library prerequisites, and the honest execution report.
// Byte-for-byte mirror of `render` in `src/coding/blueprint.rs`. `composition`
// selects which projection of the recipe template to emit (default "composed").
function renderBlueprint(blueprint, language, composition) {
  const strings = BLUEPRINT_I18N[language] || BLUEPRINT_I18N.en;
  const languageInfo = WRITE_PROGRAM_LANGUAGES[blueprint.program.languageSlug];
  const name = languageInfo ? languageInfo.name : blueprint.program.languageSlug;
  const fence = languageInfo ? languageInfo.fence : blueprint.program.languageSlug;
  const summary = blueprintRecipeSummary(blueprint.recipe, language);

  let body = strings.intro(name, summary);
  body += "\n\n";
  blueprint.capabilities.forEach((capability, index) => {
    body += `${index + 1}. ${blueprintCapabilityLabel(capability, language)}\n`;
  });
  // Compose the program from the decomposition: filter optional capability
  // regions and comments according to `composition` (see
  // `composeBlueprintProgram`) so the blueprint is an honest projection of the
  // detected capabilities rather than a single frozen string.
  const programCode = composeBlueprintProgram(
    blueprint,
    normalizeBlueprintComposition(composition),
  );
  body += `\n\`\`\`${fence}\n${programCode}\n\`\`\`\n\n${strings.librariesHeading}\n`;
  for (const library of blueprint.program.libraries) {
    body += `- ${library}\n`;
  }
  body += `\n${strings.howToRunHeading}\n\n${strings.executionReport(
    blueprint.program.runCommand,
    blueprint.program.execution,
  )}`;
  const addendum = blueprintRecipeAddendum(blueprint.recipe);
  if (addendum) body += `\n\n${addendum}`;
  return body;
}

// Build the `write_program` answer for a resolved blueprint. The evidence trail
// mirrors the Rust handler's event log (`src/solver_handlers/program_blueprint.rs`):
// the recipe, the decomposed capabilities, the (language, task) parameters, and
// an honest "unavailable" execution status.
function blueprintWriteProgramAnswer(
  blueprint,
  languageSlug,
  responseLanguage,
  composition,
) {
  const mode = normalizeBlueprintComposition(composition);
  const content = renderBlueprint(blueprint, responseLanguage || "en", mode);
  return {
    intent: "write_program",
    content,
    confidence: 0.7,
    evidence: [
      `response:write_program:blueprint:${blueprint.recipe.slug}:${languageSlug}`,
      `program_parameter:language:${languageSlug}`,
      `program_parameter:task:blueprint:${blueprint.recipe.slug}`,
      `program_blueprint:recipe:${blueprint.recipe.slug}`,
      `program_blueprint:composition:${mode}`,
      ...blueprint.capabilities.map(
        (capability) => `program_blueprint:capability:${capability.slug}`,
      ),
      `execution_status:${languageSlug}:unavailable`,
    ],
  };
}

function tryProgramBlueprintFromPrompt(prompt, responseLanguage, composition) {
  const normalizedForBlueprint = normalizeProgramPrompt(prompt);
  const languageSlug = programLanguageFromPrompt(normalizedForBlueprint);
  const blueprint = languageSlug
    ? selectBlueprint(normalizedForBlueprint, languageSlug)
    : null;
  return blueprint
    ? blueprintWriteProgramAnswer(
        blueprint,
        languageSlug,
        responseLanguage,
        composition,
      )
    : null;
}
