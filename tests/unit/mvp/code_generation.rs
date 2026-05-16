//! Code-generation tests covering the top programming languages and the
//! execution-evidence requirements from issue #8.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: prototype hello-world seeds.
// ---------------------------------------------------------------------------

#[test]
fn rust_hello_world_seed_compiles_and_runs() {
    let response = answer("Write me hello world program in Rust");
    assert_eq!(response.intent, "hello_world_rust");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("fn main()"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
    assert!(response.answer.contains("Check command: `rustc"));
    assert!(response.answer.contains("Run command: `./main`"));
    assert!(response.answer.contains("Hello, world!"));
}

#[test]
fn python_hello_world_seed_runs() {
    let response = answer("Write hello world in Python");
    assert_eq!(response.intent, "hello_world_python");
    assert!(response.answer.contains("```python"));
    assert!(response.answer.contains("print(\"Hello, world!\")"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
    assert!(response.answer.contains("python3"));
}

#[test]
fn javascript_hello_world_seed_runs() {
    let response = answer("Show me hello world in JavaScript");
    assert_eq!(response.intent, "hello_world_javascript");
    assert!(response.answer.contains("```javascript"));
    assert!(response.answer.contains("console.log(\"Hello, world!\");"));
    assert!(response.answer.contains("node"));
}

#[test]
fn go_hello_world_seed_runs() {
    let response = answer("hello world in Go");
    assert_eq!(response.intent, "hello_world_go");
    assert!(response.answer.contains("```go"));
    assert!(response.answer.contains("fmt.Println"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
}

#[test]
fn c_hello_world_seed_compiles_and_runs() {
    let response = answer("hello world in C");
    assert_eq!(response.intent, "hello_world_c");
    assert!(response.answer.contains("```c"));
    assert!(response.answer.contains("#include <stdio.h>"));
    assert!(response
        .answer
        .contains("Execution status: compiled and ran"));
}

#[test]
fn typescript_hello_world_seed_reports_unavailable_execution() {
    let response = answer("hello world in TypeScript");
    assert_eq!(response.intent, "hello_world_typescript");
    assert!(response.answer.contains("```typescript"));
    assert!(response
        .answer
        .contains("Execution status: not compiled or run"));
    assert!(response
        .answer
        .contains("Expected output after verification"));
}

#[test]
fn rust_alias_rs_is_supported() {
    let response = answer("hello world in rs");
    assert_eq!(response.intent, "hello_world_rust");
}

#[test]
fn javascript_alias_node_is_supported() {
    let response = answer("hello world in node");
    assert_eq!(response.intent, "hello_world_javascript");
}

#[test]
fn python_alias_py_is_supported() {
    let response = answer("hello world in py");
    assert_eq!(response.intent, "hello_world_python");
}

#[test]
fn go_alias_golang_is_supported() {
    let response = answer("hello world in golang");
    assert_eq!(response.intent, "hello_world_go");
}

#[test]
fn hello_world_without_recognized_language_falls_back_to_unknown() {
    let response = answer("hello world in elvish");
    assert_eq!(response.intent, "unknown");
}

// ---------------------------------------------------------------------------
// Issue #53: Russian transliteration of "hello world" must be recognized.
// The reporter sent "Напиши хелло ворлд на питоне" and received "unknown".
// ---------------------------------------------------------------------------

#[test]
fn russian_transliteration_хелло_ворлд_питоне_returns_python() {
    // The exact prompt from the reported issue.
    let response = answer("Напиши хелло ворлд на питоне");
    assert_eq!(
        response.intent, "hello_world_python",
        "Russian-transliterated hello world in Python should resolve, got: {}",
        response.intent
    );
    assert!(
        response.answer.contains("```python"),
        "answer should include a Python code block, got: {}",
        response.answer
    );
}

#[test]
fn russian_transliteration_хелло_ворлд_на_джаваскрипт_returns_javascript() {
    let response = answer("хелло ворлд на джаваскрипт");
    assert_eq!(
        response.intent, "hello_world_javascript",
        "Russian-transliterated hello world in JavaScript should resolve, got: {}",
        response.intent
    );
}

#[test]
fn russian_transliteration_хелло_ворлд_на_расте_returns_rust() {
    let response = answer("хелло ворлд на расте");
    assert_eq!(
        response.intent, "hello_world_rust",
        "Russian-transliterated hello world in Rust should resolve, got: {}",
        response.intent
    );
}

// ---------------------------------------------------------------------------
// MVP expectations: top programming languages and richer code-generation.
// ---------------------------------------------------------------------------

const POPULAR_LANGUAGES: &[(&str, &str)] = &[
    ("Rust", "rust"),
    ("Python", "python"),
    ("JavaScript", "javascript"),
    ("TypeScript", "typescript"),
    ("Go", "go"),
    ("C", "c"),
    ("C++", "cpp"),
    ("Java", "java"),
    ("C#", "csharp"),
    ("Ruby", "ruby"),
];

#[test]
#[ignore = "MVP-target: hello-world should be available in the top 10 popular languages"]
fn top_ten_popular_languages_each_have_a_hello_world_seed() {
    for (language, slug) in POPULAR_LANGUAGES {
        let response = answer(&format!("Write me hello world in {language}"));
        let expected_intent = format!("hello_world_{slug}");
        assert_eq!(
            response.intent, expected_intent,
            "missing hello-world seed for {language}"
        );
    }
}

#[test]
#[ignore = "MVP-target: code answers should include execution metadata in Links Notation form"]
fn code_answers_include_execution_links_in_notation() {
    let response = answer("Write me hello world program in Rust");
    assert!(
        response
            .links_notation
            .contains("execution_status compiled and ran"),
        "Links Notation trace should describe execution status, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("execution_environment"),
        "Links Notation trace should describe execution environment"
    );
}

#[test]
#[ignore = "MVP-target: code answers should declare an isolation level for execution evidence"]
fn code_answers_declare_isolation_level() {
    let response = answer("Write me hello world program in Rust");
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("docker")
            || lower.contains("sandbox")
            || lower.contains("webvm")
            || lower.contains("isolated"),
        "execution evidence should declare an isolation boundary, got: {}",
        response.answer
    );
}

#[test]
#[ignore = "MVP-target: requesting `sorting algorithm` should produce reviewable code and tests"]
fn sorting_algorithm_request_returns_code_and_tests() {
    let response = answer("Write me a sorting algorithm in Python with tests");
    assert!(response.intent.starts_with("algorithm_"));
    assert!(response.answer.contains("```python"));
    assert!(response.answer.contains("def test_") || response.answer.contains("assert "));
}

#[test]
#[ignore = "MVP-target: translation between programming languages should keep semantics"]
fn translating_a_program_between_languages_keeps_semantics() {
    let rust = answer("Translate `fn add(a: i32, b: i32) -> i32 { a + b }` to Python");
    assert!(rust.intent.starts_with("translate_"));
    assert!(rust.answer.contains("def add"));
    assert!(rust.answer.contains("return a + b"));
}

#[test]
#[ignore = "MVP-target: failing generated code must be re-emitted with the failure trace, not silently"]
fn execution_failures_are_reported_with_full_trace() {
    let response = answer("Write a Python script that calls undefined_function()");
    assert!(
        response.answer.contains("Execution status: failed"),
        "failures must be honest, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("trace:execution_failure")),
        "failed runs must expose a trace link"
    );
}
