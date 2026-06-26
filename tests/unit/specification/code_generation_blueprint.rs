//! Issue #340: composite `write_program` blueprint tests.
//!
//! A composite request the verified template catalog cannot resolve to a single
//! template (HTTP GET -> parse JSON -> compute mean/median -> output) must no
//! longer dead-end on `write_program_unsupported`. The blueprint synthesizer
//! decomposes the request into capabilities and returns a real, idiomatic
//! program with an honest "not run" execution report. These tests live beside
//! `code_generation` to keep each specification file within the repository
//! line-count limit.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn rust_http_json_statistics_request_returns_blueprint_program() {
    let response = answer(
        "Write a Rust program that:\n\
         1. Makes an HTTP GET request to a URL\n\
         2. Parses the JSON response\n\
         3. Calculates statistics (mean, median) from the data\n\
         4. Outputs the results\n\n\
         Include error handling and comments.",
    );
    // The dead-end is gone: the request is now answered as a write_program.
    assert_eq!(
        response.intent, "write_program",
        "composite request should be answered, not dead-ended, got: {}",
        response.intent
    );
    assert!(
        !response.answer.contains("I do not have a template"),
        "composite request must not surface the unsupported dead-end, got: {}",
        response.answer
    );
    // A real, idiomatic Rust program covering all four sub-requirements.
    assert!(
        response.answer.contains("```rust"),
        "got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("fn main()"),
        "got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("reqwest::blocking::get"),
        "should make the HTTP GET, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("serde_json") || response.answer.contains("Value"),
        "should parse JSON, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("median") && response.answer.contains("mean"),
        "should compute mean and median, got: {}",
        response.answer
    );
    // Honest execution status: the program needs network + libraries, so it is
    // explicitly NOT claimed to have run.
    assert!(
        response.answer.contains("not run"),
        "execution status must be the honest not-run report, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("compiled and ran"),
        "blueprint must never claim it compiled and ran, got: {}",
        response.answer
    );
    // The decomposition plan and evidence trail are recorded.
    assert!(
        response
            .links_notation
            .contains("program_blueprint:recipe http_json_stats"),
        "trace should record the resolved recipe, got: {}",
        response.links_notation
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "response:write_program:blueprint:http_json_stats:rust"),
        "evidence links should include the blueprint response link, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn python_http_json_statistics_request_returns_blueprint_program() {
    let response = answer(
        "Write a Python program that makes an HTTP GET request to a URL, parses the JSON \
         response, calculates the mean and median statistics, and outputs the results with \
         error handling.",
    );
    assert_eq!(response.intent, "write_program", "got: {}", response.intent);
    assert!(
        response.answer.contains("```python"),
        "got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("import requests"),
        "should use requests, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("statistics.mean")
            && response.answer.contains("statistics.median"),
        "should compute mean and median, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("not run"),
        "got: {}",
        response.answer
    );
}

#[test]
fn budget_calculator_request_returns_python_blueprint_program() {
    let response = answer(
        "I want to build a budget calculator. Here's what I need:\n\
         \n\
         1. Search for average living costs in Moscow, Berlin, and New York\n\
         2. Write a Python program that:\n\
            - Takes monthly income as input\n\
            - Calculates 50/30/20 budget rule (needs/wants/savings)\n\
            - Shows how much can be saved in each city\n\
         3. Calculate: If I save 20% of $3000 monthly at 8% annual return for 10 years, \
            how much will I have?\n\
         4. Create a comparison table showing:\n\
            - City name\n\
            - Average rent\n\
            - Remaining budget after expenses\n\
            - Years to save $100,000\n\
         5. Export all this as a formatted markdown report with sources.",
    );

    assert_eq!(
        response.intent, "write_program",
        "budget calculator request should be answered, not dead-ended, got: {}",
        response.intent
    );
    assert!(
        !response.answer.contains("I do not have a template"),
        "budget request must not surface the unsupported dead-end, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("```python"),
        "answer should include a Python code block, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("50/30/20") || response.answer.contains("0.50"),
        "program should calculate the 50/30/20 rule, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("future_value")
            && response.answer.contains("0.08")
            && response.answer.contains("10"),
        "program should calculate the requested 10-year future value, got: {}",
        response.answer
    );
    for city in ["Moscow", "Berlin", "New York"] {
        assert!(
            response.answer.contains(city),
            "program should compare {city}, got: {}",
            response.answer
        );
    }
    assert!(
        response.answer.contains("markdown") || response.answer.contains(".md"),
        "program should export a formatted markdown report, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Source") || response.answer.contains("sources"),
        "program should include source metadata, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("program_blueprint:recipe personal_budget_report"),
        "trace should record the resolved budget recipe, got: {}",
        response.links_notation
    );
    assert!(
        response.evidence_links.iter().any(|link| {
            link == "response:write_program:blueprint:personal_budget_report:python"
        }),
        "evidence links should include the budget blueprint response link, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn travel_planner_request_returns_python_blueprint_program() {
    let response = answer(
        "Build a \"Smart Travel Planner\" prototype:\n\
         \n\
         1. Search: visa requirements for Russian citizens visiting Japan, UAE, Serbia\n\
         2. Search: average flight costs from Moscow to these destinations (next 3 months)\n\
         3. Write a Python class `TravelPlanner` with methods:\n\
            - `add_destination(country: str, budget: float)`\n\
            - `check_visa_requirements()` -> returns bool\n\
            - `estimate_total_cost()` -> returns dict\n\
            - `generate_itinerary(days: int)` -> returns markdown\n\
         4. Add business logic:\n\
            - Prioritize destinations with visa-free access\n\
            - Flag if budget < estimated cost\n\
         5. Generate sample output for: 7-day trip, $2000 budget\n\
         6. Output: class code + usage example + sample itinerary",
    );

    assert_eq!(
        response.intent, "write_program",
        "travel planner request should be answered, not dead-ended, got: {}",
        response.intent
    );
    assert!(
        !response.answer.contains("I do not have a template")
            && !response.answer.contains("task `missing`"),
        "travel planner request must not surface the unsupported dead-end, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("```python"),
        "answer should include a Python code block, got: {}",
        response.answer
    );
    for required in [
        "class TravelPlanner",
        "add_destination",
        "check_visa_requirements",
        "estimate_total_cost",
        "generate_itinerary",
        "visa-free",
        "Budget warning",
        "Sample output",
        "7-day",
        "$2,000",
    ] {
        assert!(
            response.answer.contains(required),
            "answer should include {required:?}, got: {}",
            response.answer
        );
    }
    assert!(
        response
            .links_notation
            .contains("program_blueprint:recipe smart_travel_planner"),
        "trace should record the resolved travel planner recipe, got: {}",
        response.links_notation
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "response:write_program:blueprint:smart_travel_planner:python"),
        "evidence links should include the travel planner blueprint response link, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn budget_calculator_blueprint_covers_supported_prompt_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
        localized_status: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            prompt: "Search sources for average living costs and rent in Moscow, Berlin, and New York. \
                     Write a Python program that takes monthly income, applies the 50/30/20 budget rule, \
                     calculates saving 20% of $3000 monthly at 8% annual return for 10 years, creates a \
                     comparison table, and exports a markdown report with sources.",
            localized_status: "Execution status",
        },
        Case {
            language: "ru",
            prompt: "Найди источники по средней стоимости жизни и аренде в Москве, Берлине и Нью-Йорке. \
                     Напиши программу на Python, которая принимает месячный доход, применяет правило \
                     бюджета 50/30/20, считает накопить 20% от $3000, создает таблицу сравнения и \
                     экспортирует markdown отчёт с источниками.",
            localized_status: "Статус выполнения",
        },
        Case {
            language: "hi",
            prompt: "मास्को, बर्लिन और न्यूयॉर्क में औसत जीवन यापन लागत और किराया के स्रोत खोजो। \
                     Python प्रोग्राम लिखो जो मासिक आय ले, 50/30/20 बजट नियम लगाए, $3000 का 20% \
                     8% वार्षिक रिटर्न पर 10 साल बचत की गणना करे, तुलना तालिका बनाए और स्रोतों के \
                     साथ मार्कडाउन रिपोर्ट निर्यात करे।",
            localized_status: "निष्पादन स्थिति",
        },
        Case {
            language: "zh",
            prompt: "搜索莫斯科、柏林和纽约的平均生活成本和租金来源。编写 Python 程序，输入月收入，应用 \
                     50/30/20 预算规则，计算每月存 $3000 的 20% 按 8% 年收益 10 年，创建比较表格，\
                     并导出带来源的 Markdown 报告。",
            localized_status: "执行状态",
        },
    ];

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "write_program",
            "language {} should route to budget blueprint, got: {}",
            case.language, response.intent
        );
        assert!(
            response.answer.contains(case.localized_status),
            "language {} should receive localized execution status, got: {}",
            case.language,
            response.answer
        );
        assert!(
            response
                .links_notation
                .contains("program_blueprint:recipe personal_budget_report"),
            "language {} should record budget recipe, got: {}",
            case.language,
            response.links_notation
        );
    }
}

#[test]
fn javascript_http_json_statistics_request_returns_blueprint_program() {
    let response = answer(
        "Write a JavaScript program that fetches JSON from a URL via an HTTP GET request, \
         parses it, computes the mean and median, and prints the results.",
    );
    assert_eq!(response.intent, "write_program", "got: {}", response.intent);
    assert!(
        response.answer.contains("```javascript") || response.answer.contains("```js"),
        "got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("fetch("),
        "should use fetch, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("not run"),
        "got: {}",
        response.answer
    );
}

#[test]
fn russian_http_json_statistics_request_returns_blueprint_in_russian() {
    let response = answer(
        "Напиши программу на Rust, которая делает HTTP GET запрос к URL, разбирает JSON ответ, \
         вычисляет среднее и медиану и выводит результаты.",
    );
    assert_eq!(response.intent, "write_program", "got: {}", response.intent);
    assert!(
        response.answer.contains("```rust"),
        "got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Статус выполнения"),
        "execution report should be localized to Russian, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("missing"),
        "must not surface the missing-template error, got: {}",
        response.answer
    );
}

#[test]
fn hindi_http_json_statistics_request_returns_blueprint_in_hindi() {
    // Hindi (हिंदी) composite request for a Python program. The capability
    // keywords are matched in Devanagari, and the execution report is localized.
    let response = answer(
        "Python में एक प्रोग्राम लिखो जो http अनुरोध करे, json पार्स करे और औसत और माध्यिका की गणना करे।",
    );
    assert_eq!(response.intent, "write_program", "got: {}", response.intent);
    assert!(
        response.answer.contains("```python"),
        "got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("निष्पादन स्थिति"),
        "execution report should be localized to Hindi, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("I do not have a template"),
        "must not surface the unsupported dead-end, got: {}",
        response.answer
    );
}

#[test]
fn chinese_http_json_statistics_request_returns_blueprint_in_chinese() {
    // Chinese (中文) composite request for a JavaScript program. CJK keywords
    // are matched by substring and the execution report is localized.
    let response =
        answer("用 JavaScript 编写一个程序，发起 http 请求，解析 json，并计算平均值和中位数。");
    assert_eq!(response.intent, "write_program", "got: {}", response.intent);
    assert!(
        response.answer.contains("```javascript") || response.answer.contains("```js"),
        "got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("执行状态"),
        "execution report should be localized to Chinese, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("I do not have a template"),
        "must not surface the unsupported dead-end, got: {}",
        response.answer
    );
}

#[test]
fn english_http_json_statistics_request_records_blueprint_capabilities() {
    // English (language: "en") composite request: assert the full capability
    // decomposition is recorded in the trace so the blueprint is auditable.
    let response = answer(
        "Write a Rust program that makes an HTTP GET request, parses the JSON, computes the \
         mean and median, outputs the results, with error handling and comments.",
    );
    assert_eq!(response.intent, "write_program", "got: {}", response.intent);
    for capability in [
        "http_request",
        "json_parse",
        "statistics",
        "output_results",
        "error_handling",
        "comments",
    ] {
        assert!(
            response
                .links_notation
                .contains(&format!("program_blueprint:capability {capability}")),
            "trace should record the {capability} capability, got: {}",
            response.links_notation
        );
    }
}

/// Pull the body out of the first fenced code block of a rendered answer.
fn fenced_code(answer: &str) -> &str {
    let after_open = answer
        .split_once("```")
        .map(|(_, rest)| rest)
        .expect("answer has an opening fence");
    let body = after_open
        .split_once('\n')
        .map(|(_, rest)| rest)
        .expect("fence has a body");
    body.split("```")
        .next()
        .expect("answer has a closing fence")
}

#[test]
fn comments_capability_composes_the_emitted_program_end_to_end() {
    // The same `http_json_stats` recipe must emit two *different* programs
    // depending on whether the decomposition contains the `comments` capability,
    // proving the program is assembled from the request (composition) rather than
    // recalled verbatim from a table (NON-GOALS.md forbids a memoized answer
    // cache). This exercises the full public engine path, not just the module.
    let documented = answer(
        "Write a Rust program that makes an HTTP GET request to a URL, parses the JSON response, \
         calculates the mean and median, outputs the results, with error handling and comments.",
    );
    let stripped = answer(
        "Write a Rust program that makes an HTTP GET request to a URL, parses the JSON response, \
         calculates the mean and median, and outputs the results.",
    );
    assert_eq!(documented.intent, "write_program");
    assert_eq!(stripped.intent, "write_program");

    let documented_code = fenced_code(&documented.answer);
    let stripped_code = fenced_code(&stripped.answer);

    // With comments requested, the documentation is present.
    assert!(
        documented_code
            .lines()
            .any(|line| line.trim_start().starts_with("//")),
        "documented program should keep whole-line comments, got: {documented_code}"
    );
    // Without comments, every whole-line comment is gone...
    assert!(
        !stripped_code
            .lines()
            .any(|line| line.trim_start().starts_with("//")),
        "stripped program must drop whole-line comments, got: {stripped_code}"
    );
    // ...but the core logic (a different, still-valid program) is preserved...
    assert!(
        stripped_code.contains("reqwest::blocking::get")
            && stripped_code.contains("fn median(")
            && stripped_code.contains("fn mean("),
        "stripped program must keep the core logic, got: {stripped_code}"
    );
    // ...with no leftover blank-line runs from the removed comment blocks...
    assert!(
        !stripped_code.contains("\n\n\n"),
        "stripped program must not leave blank-line runs, got: {stripped_code}"
    );
    // ...and the stripped program is genuinely smaller than the documented one.
    assert!(
        stripped_code.len() < documented_code.len(),
        "stripped program should be smaller ({} vs {} bytes)",
        stripped_code.len(),
        documented_code.len()
    );
}

#[test]
fn python_comments_omitted_drops_docstring_and_hash_comments_end_to_end() {
    // The Python projection drops the module docstring and `#` comment lines when
    // comments are not requested, while keeping the import and computation.
    let stripped = answer(
        "Write a Python program that makes an HTTP GET request to a URL, parses the JSON \
         response, and calculates the mean and median.",
    );
    assert_eq!(stripped.intent, "write_program");
    let code = fenced_code(&stripped.answer);
    assert!(
        !code.contains("\"\"\""),
        "stripped python must drop the docstring, got: {code}"
    );
    assert!(
        !code.lines().any(|line| line.trim_start().starts_with('#')),
        "stripped python must drop whole-line # comments, got: {code}"
    );
    assert!(
        code.contains("requests.get") && code.contains("statistics.median"),
        "stripped python must keep the core logic, got: {code}"
    );
}

#[test]
fn partial_composite_request_without_statistics_stays_unsupported() {
    // http + json but NO statistics -> no recipe matches, so the honest
    // unsupported answer is preserved (we do not fabricate a program).
    let response = answer(
        "Write a Rust program that makes an HTTP GET request to a URL and parses the JSON \
         response.",
    );
    assert_eq!(
        response.intent, "write_program_unsupported",
        "an unmatched composite request keeps the honest unsupported answer, got: {}",
        response.intent
    );
}
