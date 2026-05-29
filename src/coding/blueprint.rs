//! Issue #340: composite-program *blueprints* for `write_program` requests that
//! the verified template [`catalog`](crate::coding::catalog) cannot resolve to a
//! single, sandbox-runnable task.
//!
//! The catalog is the deterministic, honest baseline: every catalog template is
//! a self-contained program that was actually compiled and run, so the engine
//! can claim "compiled and ran". That invariant is exactly why a request like
//!
//! > Write a Rust program that makes an HTTP GET request, parses the JSON
//! > response, calculates the mean and median, and outputs the results.
//!
//! cannot live in the catalog: it needs external libraries (`reqwest`,
//! `serde_json`) and *network access*, neither of which the offline WASM sandbox
//! can verify. Before this module such a request fell through to
//! `write_program_unsupported` ("I do not have a template for language `rust`
//! and task `missing`") — a dead end that ignored the four explicit
//! sub-requirements the user spelled out.
//!
//! A *blueprint* closes that gap while staying honest. It walks the same
//! "decompose the problem into tasks" path as the universal solver:
//!
//! 1. **Decompose** — [`detect_capabilities`] scans the prompt for recognized
//!    programming capabilities (`http_request`, `json_parse`, `statistics`,
//!    `error_handling`, `comments`, …), in every supported prompt language.
//! 2. **Match a recipe** — [`select_blueprint`] finds a [`BlueprintRecipe`]
//!    whose required capabilities are all present and that has a curated,
//!    review-quality program for the requested language.
//! 3. **Render** — the engine returns the full program together with the
//!    decomposition plan, the libraries it depends on, and an *honest* execution
//!    report ("not run — needs external libraries / network; review before
//!    running"). Nothing claims to have executed.
//!
//! Adding coverage is data: a new capability keyword set or a new recipe/
//! language program extends the tables below without touching the solver.

use std::fmt::Write as _;

use crate::language::Language;

/// A recognized programming capability — one "sub-task" a composite request can
/// decompose into. Detection is keyword-based and script-aware (the same
/// token/substring rules the catalog uses), so prompts in en/ru/hi/zh resolve.
#[derive(Clone, Copy)]
pub struct Capability {
    pub slug: &'static str,
    /// Short, English label used in the decomposition plan heading map below.
    pub label: &'static str,
    /// Surface phrases (lowercased) that signal this capability, across every
    /// supported prompt language. CJK phrases are matched by substring.
    pub keywords: &'static [&'static str],
}

/// A curated program for one language inside a [`BlueprintRecipe`].
#[derive(Clone, Copy)]
pub struct RecipeProgram {
    pub language_slug: &'static str,
    /// External libraries / runtime prerequisites the program depends on, in the
    /// canonical form a user would install (e.g. `reqwest`, `serde_json`).
    pub libraries: &'static [&'static str],
    /// The command a user runs to execute the program once dependencies are
    /// installed. Canonical (not localized) — it is a literal shell command.
    pub run_command: &'static str,
    pub code: &'static str,
}

/// A composite program recipe: a recognizable multi-step task that needs
/// libraries or I/O the sandbox cannot verify, realized as curated programs.
#[derive(Clone, Copy)]
pub struct BlueprintRecipe {
    pub slug: &'static str,
    /// English label describing the whole program (used as a fallback title).
    pub label: &'static str,
    /// Capabilities that must all be present for this recipe to match.
    pub required_capabilities: &'static [&'static str],
    pub programs: &'static [RecipeProgram],
}

impl BlueprintRecipe {
    fn program_for(&self, language_slug: &str) -> Option<&'static RecipeProgram> {
        self.programs
            .iter()
            .find(|program| program.language_slug == language_slug)
    }
}

/// A resolved blueprint: the recipe, the chosen language program, and the
/// capabilities that were detected in the request (the decomposition).
pub struct Blueprint {
    pub recipe: &'static BlueprintRecipe,
    pub program: &'static RecipeProgram,
    pub capabilities: Vec<&'static Capability>,
}

/// The capability catalog. Each entry maps a class of sub-task onto the surface
/// phrases that request it. Latin/Cyrillic keywords match on whitespace token
/// boundaries; CJK keywords match by substring (see [`contains_keyword`]).
pub const CAPABILITIES: &[Capability] = &[
    Capability {
        slug: "http_request",
        label: "Make an HTTP request",
        keywords: &[
            "http",
            "https",
            "url",
            "get request",
            "http get",
            "fetch",
            "download",
            "запрос",
            "ссылк",
            "загруз",
            "स्थानांतरण",
            "अनुरोध",
            "请求",
            "下载",
            "网址",
        ],
    },
    Capability {
        slug: "json_parse",
        label: "Parse the JSON response",
        keywords: &[
            "json",
            "parse",
            "parses",
            "parsing",
            "deserialize",
            "разбор",
            "разобрать",
            "парсинг",
            "जेसन",
            "पार्स",
            "解析",
        ],
    },
    Capability {
        slug: "statistics",
        label: "Calculate statistics (mean, median)",
        keywords: &[
            "statistics",
            "statistic",
            "mean",
            "average",
            "median",
            "статистик",
            "среднее",
            "медиан",
            "औसत",
            "माध्यिका",
            "सांख्यिकी",
            "统计",
            "平均",
            "中位数",
        ],
    },
    Capability {
        slug: "output_results",
        label: "Output the results",
        keywords: &[
            "output",
            "print",
            "outputs",
            "display",
            "report",
            "вывод",
            "вывести",
            "печат",
            "आउटपुट",
            "छाप",
            "输出",
            "打印",
            "显示",
        ],
    },
    Capability {
        slug: "error_handling",
        label: "Handle errors",
        keywords: &[
            "error handling",
            "error-handling",
            "errors",
            "error",
            "exception",
            "ошибк",
            "обработк",
            "त्रुटि",
            "错误",
            "异常",
        ],
    },
    Capability {
        slug: "comments",
        label: "Document the code with comments",
        keywords: &[
            "comments",
            "comment",
            "commented",
            "documented",
            "комментар",
            "टिप्पणि",
            "注释",
            "评论",
        ],
    },
];

/// Curated composite recipes. Programs are hand-written, idiomatic, and reviewed
/// (not sandbox-executed, by design — they need network access / external
/// libraries). Each was checked for syntax with its toolchain offline; see
/// `experiments/issue-340-blueprint`.
pub const RECIPES: &[BlueprintRecipe] = &[BlueprintRecipe {
    slug: "http_json_stats",
    label: "fetch JSON over HTTP and report the mean and median of its numbers",
    required_capabilities: &["http_request", "json_parse", "statistics"],
    programs: &[
        RecipeProgram {
            language_slug: "rust",
            libraries: &["reqwest (blocking, json)", "serde_json"],
            run_command: "cargo run -- <url-returning-json>",
            code: RUST_HTTP_JSON_STATS,
        },
        RecipeProgram {
            language_slug: "python",
            libraries: &["requests"],
            run_command: "python stats.py <url-returning-json>",
            code: PYTHON_HTTP_JSON_STATS,
        },
        RecipeProgram {
            language_slug: "javascript",
            libraries: &["Node.js 18+ (built-in global fetch; no extra packages)"],
            run_command: "node stats.js <url-returning-json>",
            code: JAVASCRIPT_HTTP_JSON_STATS,
        },
    ],
}];

const RUST_HTTP_JSON_STATS: &str = r#"//! Fetch JSON from a URL and report the mean and median of every number in it.
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

    // 3. Gather every number, then guard against an empty data set.
    let mut numbers = Vec::new();
    collect_numbers(&document, &mut numbers);
    if numbers.is_empty() {
        return Err("the JSON response contained no numbers".into());
    }

    // 4. Compute and print the statistics.
    println!("count:  {}", numbers.len());
    println!("mean:   {:.4}", mean(&numbers));
    println!("median: {:.4}", median(&mut numbers));
    Ok(())
}
"#;

const PYTHON_HTTP_JSON_STATS: &str = r#""""Fetch JSON from a URL and report the mean and median of every number in it.

Dependencies:  pip install requests
"""

import statistics
import sys

import requests


def collect_numbers(value):
    """Recursively collect every int/float out of a decoded JSON value."""
    if isinstance(value, bool):  # bool subclasses int, so skip it explicitly
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

    # 2. Make the HTTP GET request and parse the JSON body, turning any HTTP
    #    error status into an exception before we try to decode it.
    response = requests.get(url, timeout=30)
    response.raise_for_status()
    document = response.json()

    # 3. Gather every number, then guard against an empty data set.
    numbers = collect_numbers(document)
    if not numbers:
        raise SystemExit("the JSON response contained no numbers")

    # 4. Compute and print the statistics.
    print(f"count:  {len(numbers)}")
    print(f"mean:   {statistics.mean(numbers):.4f}")
    print(f"median: {statistics.median(numbers):.4f}")


if __name__ == "__main__":
    main()
"#;

const JAVASCRIPT_HTTP_JSON_STATS: &str = r#"// Fetch JSON from a URL and report the mean and median of every number in it.
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

  // 2. Make the HTTP GET request and parse the JSON body, failing fast on a
  //    non-2xx status before we try to decode it.
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} ${response.statusText}`);
  }
  const document = await response.json();

  // 3. Gather every number, then guard against an empty data set.
  const numbers = collectNumbers(document);
  if (numbers.length === 0) {
    throw new Error("the JSON response contained no numbers");
  }

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

/// Detect which capabilities a normalized (lowercased) prompt requests, in
/// catalog order so the decomposition plan reads top-to-bottom.
#[must_use]
pub fn detect_capabilities(normalized: &str) -> Vec<&'static Capability> {
    CAPABILITIES
        .iter()
        .filter(|capability| {
            capability
                .keywords
                .iter()
                .any(|keyword| contains_keyword(normalized, keyword))
        })
        .collect()
}

/// Resolve a blueprint for a request: the first recipe whose required
/// capabilities are all present in `normalized` and that has a program for
/// `language_slug`. Returns `None` when nothing matches (the caller then keeps
/// the honest `write_program_unsupported` fallback).
#[must_use]
pub fn select_blueprint(normalized: &str, language_slug: &str) -> Option<Blueprint> {
    let detected = detect_capabilities(normalized);
    let detected_slugs: Vec<&str> = detected.iter().map(|capability| capability.slug).collect();
    let recipe = RECIPES.iter().find(|recipe| {
        recipe
            .required_capabilities
            .iter()
            .all(|required| detected_slugs.contains(required))
            && recipe.program_for(language_slug).is_some()
    })?;
    let program = recipe.program_for(language_slug)?;
    Some(Blueprint {
        recipe,
        program,
        capabilities: detected,
    })
}

fn contains_keyword(normalized: &str, keyword: &str) -> bool {
    if crate::coding::contains_cjk(keyword) {
        return normalized.contains(keyword);
    }
    if keyword.contains(' ') {
        return normalized.contains(keyword);
    }
    // Token-boundary match for single Latin/Cyrillic words, but allow a stem
    // (e.g. "запрос" matching "запросы") by also accepting a prefix match on a
    // token. This keeps short tokens from matching inside unrelated words while
    // tolerating inflection in Russian/Hindi.
    normalized
        .split(|character: char| !character.is_alphanumeric())
        .any(|token| token == keyword || (keyword.len() >= 4 && token.starts_with(keyword)))
}

/// Localized heading that introduces the generated blueprint program.
#[must_use]
pub fn blueprint_intro(language_name: &str, recipe_label: &str, language: Language) -> String {
    match language {
        Language::Russian => format!(
            "Вот программа на языке {language_name}, которая решает составную задачу \
             ({recipe_label}). Я разбил ваш запрос на следующие подзадачи:"
        ),
        Language::Hindi => format!(
            "यहाँ {language_name} में एक प्रोग्राम है जो इस संयुक्त कार्य को हल करता है \
             ({recipe_label})। मैंने आपके अनुरोध को इन उप-कार्यों में विभाजित किया है:"
        ),
        Language::Chinese => format!(
            "这是一个解决该复合任务的 {language_name} 程序（{recipe_label}）。\
             我已将您的请求分解为以下子任务："
        ),
        _ => format!(
            "Here is a {language_name} program for the requested composite task \
             ({recipe_label}). I decomposed your request into these sub-tasks:"
        ),
    }
}

/// Localized label for a capability in the decomposition plan.
#[must_use]
pub fn capability_label(capability: &Capability, language: Language) -> &'static str {
    match (capability.slug, language) {
        ("http_request", Language::Russian) => "Выполнить HTTP-запрос",
        ("http_request", Language::Hindi) => "HTTP अनुरोध करें",
        ("http_request", Language::Chinese) => "发起 HTTP 请求",
        ("json_parse", Language::Russian) => "Разобрать JSON-ответ",
        ("json_parse", Language::Hindi) => "JSON प्रतिक्रिया पार्स करें",
        ("json_parse", Language::Chinese) => "解析 JSON 响应",
        ("statistics", Language::Russian) => "Вычислить статистику (среднее, медиана)",
        ("statistics", Language::Hindi) => "सांख्यिकी (औसत, माध्यिका) की गणना करें",
        ("statistics", Language::Chinese) => "计算统计量（平均值、中位数）",
        ("output_results", Language::Russian) => "Вывести результаты",
        ("output_results", Language::Hindi) => "परिणाम आउटपुट करें",
        ("output_results", Language::Chinese) => "输出结果",
        ("error_handling", Language::Russian) => "Обработать ошибки",
        ("error_handling", Language::Hindi) => "त्रुटियाँ संभालें",
        ("error_handling", Language::Chinese) => "处理错误",
        ("comments", Language::Russian) => "Снабдить код комментариями",
        ("comments", Language::Hindi) => "कोड में टिप्पणियाँ जोड़ें",
        ("comments", Language::Chinese) => "为代码添加注释",
        _ => capability.label,
    }
}

/// Localized libraries heading.
#[must_use]
pub const fn libraries_heading(language: Language) -> &'static str {
    match language {
        Language::Russian => "Необходимые библиотеки:",
        Language::Hindi => "आवश्यक लाइब्रेरियाँ:",
        Language::Chinese => "所需的库：",
        _ => "Required libraries:",
    }
}

/// Localized, honest execution report: the blueprint is never executed because
/// it depends on external libraries and/or network access the offline sandbox
/// cannot provide. The run command stays canonical.
#[must_use]
pub fn blueprint_execution_report(run_command: &str, language: Language) -> String {
    match language {
        Language::Russian => format!(
            "Статус выполнения: не запускалось — программе нужны внешние библиотеки и \
             доступ к сети, поэтому офлайн-песочница её не выполняет. Код приведён для \
             проверки. Запустить самостоятельно: `{run_command}`."
        ),
        Language::Hindi => format!(
            "निष्पादन स्थिति: नहीं चलाया गया — प्रोग्राम को बाहरी लाइब्रेरियों और नेटवर्क पहुँच \
             की आवश्यकता है, इसलिए ऑफ़लाइन सैंडबॉक्स इसे नहीं चलाता। कोड समीक्षा के लिए दिया \
             गया है। स्वयं चलाएँ: `{run_command}`।"
        ),
        Language::Chinese => format!(
            "执行状态：未运行 —— 该程序需要外部库和网络访问，因此离线沙箱不会执行它。\
             代码仅供审阅。自行运行：`{run_command}`。"
        ),
        _ => format!(
            "Execution status: not run — this program needs external libraries and network \
             access, so the offline sandbox does not execute it. The code is provided for \
             review. Run it yourself: `{run_command}`."
        ),
    }
}

/// Localized one-line summary of what a recipe's program does, used inside the
/// intro heading so the framing prose matches the response language.
#[must_use]
pub fn recipe_summary(recipe: &BlueprintRecipe, language: Language) -> &'static str {
    match (recipe.slug, language) {
        ("http_json_stats", Language::Russian) => {
            "загрузить JSON по HTTP и вывести среднее и медиану его чисел"
        }
        ("http_json_stats", Language::Hindi) => {
            "HTTP के माध्यम से JSON प्राप्त करें और उसकी संख्याओं का औसत और माध्यिका दिखाएँ"
        }
        ("http_json_stats", Language::Chinese) => {
            "通过 HTTP 获取 JSON 并报告其中数字的平均值和中位数"
        }
        _ => recipe.label,
    }
}

/// Localized "Run it yourself" heading shown above the execution report.
#[must_use]
const fn how_to_run_heading(language: Language) -> &'static str {
    match language {
        Language::Russian => "Как запустить самостоятельно:",
        Language::Hindi => "इसे स्वयं कैसे चलाएँ:",
        Language::Chinese => "如何自行运行：",
        _ => "How to run it yourself:",
    }
}

/// Render the complete localized blueprint answer: decomposition plan, the
/// curated program, its library prerequisites, and the honest execution report.
///
/// The code fence and language name come from the verified catalog so the
/// rendering matches every other `write_program` answer; if the language is
/// somehow absent from the catalog we fall back to the recipe's own slug.
#[must_use]
pub fn render(blueprint: &Blueprint, language: Language) -> String {
    let catalog_language = crate::coding::program_language_by_slug(blueprint.program.language_slug);
    let language_name =
        catalog_language.map_or(blueprint.program.language_slug, |language| language.name);
    let code_fence = catalog_language.map_or(blueprint.program.language_slug, |l| l.code_fence);

    let summary = recipe_summary(blueprint.recipe, language);
    let mut body = blueprint_intro(language_name, summary, language);

    body.push_str("\n\n");
    for (index, capability) in blueprint.capabilities.iter().enumerate() {
        writeln!(
            body,
            "{}. {}",
            index + 1,
            capability_label(capability, language)
        )
        .expect("string write is infallible");
    }

    write!(
        body,
        "\n```{code_fence}\n{}\n```\n\n{}\n",
        blueprint.program.code,
        libraries_heading(language)
    )
    .expect("string write is infallible");
    for library in blueprint.program.libraries {
        writeln!(body, "- {library}").expect("string write is infallible");
    }

    write!(
        body,
        "\n{}\n\n{}",
        how_to_run_heading(language),
        blueprint_execution_report(blueprint.program.run_command, language)
    )
    .expect("string write is infallible");

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_http_json_statistics_capabilities() {
        let prompt = "write a rust program that makes an http get request to a url, parses the \
                      json response, calculates statistics mean median and outputs the results \
                      with error handling and comments";
        let detected: Vec<&str> = detect_capabilities(prompt)
            .iter()
            .map(|capability| capability.slug)
            .collect();
        for expected in [
            "http_request",
            "json_parse",
            "statistics",
            "output_results",
            "error_handling",
            "comments",
        ] {
            assert!(
                detected.contains(&expected),
                "missing {expected} in {detected:?}"
            );
        }
    }

    #[test]
    fn selects_rust_http_json_stats_blueprint() {
        let prompt = "makes an http get request to a url parses the json response calculates \
                      statistics mean median";
        let blueprint = select_blueprint(prompt, "rust").expect("blueprint resolves for rust");
        assert_eq!(blueprint.recipe.slug, "http_json_stats");
        assert_eq!(blueprint.program.language_slug, "rust");
        assert!(blueprint.program.code.contains("fn main()"));
        assert!(blueprint.program.code.contains("reqwest::blocking::get"));
    }

    #[test]
    fn missing_statistics_capability_does_not_match_recipe() {
        // http + json but no statistics -> the recipe's required capabilities are
        // not all present, so no blueprint (honest unsupported fallback kept).
        let prompt = "write a program that makes an http get request and parses the json";
        assert!(select_blueprint(prompt, "rust").is_none());
    }

    #[test]
    fn unsupported_language_for_recipe_yields_none() {
        let prompt = "http get request parse json calculate mean median statistics";
        // The recipe only ships rust/python/javascript programs.
        assert!(select_blueprint(prompt, "go").is_none());
    }

    #[test]
    fn every_recipe_program_language_is_a_catalog_language() {
        for recipe in RECIPES {
            for program in recipe.programs {
                assert!(
                    crate::coding::program_language_by_slug(program.language_slug).is_some(),
                    "recipe {} references unknown language {}",
                    recipe.slug,
                    program.language_slug
                );
            }
        }
    }

    #[test]
    fn russian_keywords_detect_capabilities() {
        let prompt = "напиши программу которая делает http запрос разбирает json и считает \
                      среднее и медиану";
        let detected: Vec<&str> = detect_capabilities(prompt)
            .iter()
            .map(|capability| capability.slug)
            .collect();
        assert!(detected.contains(&"http_request"), "{detected:?}");
        assert!(detected.contains(&"json_parse"), "{detected:?}");
        assert!(detected.contains(&"statistics"), "{detected:?}");
    }

    #[test]
    fn render_contains_plan_code_libraries_and_honest_execution() {
        let prompt = "http get request parse json calculate mean median statistics";
        let blueprint = select_blueprint(prompt, "rust").expect("blueprint resolves");
        let rendered = render(&blueprint, Language::English);
        // Decomposition plan is numbered.
        assert!(rendered.contains("1. Make an HTTP request"), "{rendered}");
        // The real program is embedded in a fenced block.
        assert!(rendered.contains("```rust"), "{rendered}");
        assert!(rendered.contains("reqwest::blocking::get"), "{rendered}");
        // Library prerequisites are listed.
        assert!(rendered.contains("Required libraries:"), "{rendered}");
        assert!(rendered.contains("serde_json"), "{rendered}");
        // The execution report is honest: it never claims the program ran.
        assert!(rendered.contains("not run"), "{rendered}");
        assert!(
            !rendered.to_lowercase().contains("compiled and ran"),
            "{rendered}"
        );
    }

    #[test]
    fn render_localizes_framing_into_russian() {
        let prompt = "http get request parse json calculate mean median statistics";
        let blueprint = select_blueprint(prompt, "python").expect("blueprint resolves");
        let rendered = render(&blueprint, Language::Russian);
        assert!(rendered.contains("Статус выполнения"), "{rendered}");
        assert!(rendered.contains("```python"), "{rendered}");
    }
}
