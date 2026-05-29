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
