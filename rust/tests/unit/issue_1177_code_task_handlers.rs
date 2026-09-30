//! Issue #1177: the code-task family — nine solver handlers for debugging,
//! regex synthesis, SQL synthesis, shell-command composition, code
//! explanation, code review, test generation, code refactoring, and
//! JSON/YAML format conversion.
//!
//! Every handler is structural: no code, regex, query, command, or generated
//! test is ever executed, and every answer template states that honestly.
//! The tests pin the contract from two angles:
//! - engine level: `UniversalSolver` answers each cued request with the
//!   derived artifact (these run once the dispatch chain wires the handlers),
//! - handler level: each `handle_*` function directly — the composition, the
//!   honesty marker, and the refusal to claim unrelated prompts.
//!
//! The solver runs offline so no test touches the network; prompts are
//! normalized with `web_engine_core::normalize_prompt`, the public twin of
//! the engine's normalizer.

use formal_ai::event_log::EventLog;
use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::{SolverConfig, UniversalSolver};

/// The answer a hermetic (offline) solver gives to `prompt`.
fn solved(prompt: &str) -> String {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    solver.solve(prompt).answer
}

/// The fenced block with the given tag, extracted from an answer body.
fn fenced(answer: &str, tag: &str) -> String {
    let open = ["```", tag, "\n"].concat();
    let start = answer
        .find(open.as_str())
        .unwrap_or_else(|| panic!("answer should carry a `{tag}` fence"))
        + open.len();
    let end = start
        + answer[start..]
            .find("```")
            .unwrap_or_else(|| panic!("the `{tag}` fence should be closed"));
    answer[start..end].to_owned()
}

// ---------------------------------------------------------------------------
// Engine level: the cued requests are answered with the derived artifacts.
// ---------------------------------------------------------------------------

#[test]
fn engine_answers_code_debugging_request() {
    let answer = solved("What's wrong with `def average(xs): return sum(xs) / len(xs) - 1`?");
    assert!(answer.contains("- 1"), "the shift must be named: {answer}");
    assert!(
        answer.contains("average"),
        "the function must be named: {answer}"
    );
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn engine_answers_regex_synthesis_request() {
    let answer = solved(
        "Write a regular expression that matches five digits optionally followed by a hyphen and four digits",
    );
    assert!(
        answer.contains("^\\d{5}(-\\d{4})?$"),
        "the composed pattern must be anchored: {answer}"
    );
    assert!(answer.contains("Verified structurally"), "{answer}");
}

#[test]
fn engine_answers_sql_synthesis_request() {
    let answer = solved("Write a SQL query that selects all users older than 30");
    assert!(answer.contains("SELECT"), "{answer}");
    assert!(answer.contains("FROM users"), "{answer}");
    assert!(answer.contains("age > 30"), "{answer}");
    assert!(answer.contains("nothing was run"), "{answer}");
}

#[test]
fn engine_answers_shell_compose_request() {
    let answer = solved("find .log files larger than 10 MB under /var");
    assert!(answer.contains("find /var"), "{answer}");
    assert!(answer.contains("-name '*.log'"), "{answer}");
    assert!(answer.contains("-size +10M"), "{answer}");
    assert!(answer.contains("Not executed"), "{answer}");
}

#[test]
fn engine_answers_code_explanation_request() {
    let answer = solved("Explain this code: `def average(xs): return sum(xs) / len(xs)`");
    assert!(answer.contains("function"), "{answer}");
    assert!(answer.contains("average"), "{answer}");
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn engine_answers_code_review_request() {
    let prompt = "Review this code:\n```python\ndef load(path):\n    try:\n        f = open(path)\n    except:\n        pass\n```";
    let answer = solved(prompt);
    assert!(
        answer.contains("except:"),
        "the offending line is quoted: {answer}"
    );
    assert!(answer.contains("docs.python.org"), "{answer}");
}

#[test]
fn engine_answers_test_generation_request() {
    let answer =
        solved("Write tests for `is_palindrome(s)` ignoring case, spaces, and punctuation");
    assert!(answer.contains("def test_"), "{answer}");
    assert!(answer.contains("RaceCar"), "{answer}");
    assert!(answer.contains("is_palindrome(normalize("), "{answer}");
    assert!(answer.contains("NOT executed"), "{answer}");
}

#[test]
fn engine_answers_code_refactoring_request() {
    let prompt = "Refactor this promise chain with async/await:\n```javascript\nfetch(url)\n  .then(r => r.json())\n  .then(d => render(d))\n  .catch(e => log(e));\n```";
    let answer = solved(prompt);
    let code = fenced(&answer, "javascript");
    assert!(code.contains("async function run()"), "{answer}");
    assert!(code.contains("await fetch(url)"), "{answer}");
    assert!(code.contains("catch (e)"), "{answer}");
}

#[test]
fn engine_answers_format_conversion_request() {
    let json_text = "{\"name\": \"formal-ai\", \"tags\": [\"rust\", \"lino\"], \"counts\": {\"lines\": 42, \"passed\": 41}}";
    let to_yaml = ["Convert this JSON to YAML:\n```json\n", json_text, "\n```"].concat();
    let yaml_answer = solved(&to_yaml);
    let yaml_text = fenced(&yaml_answer, "yaml");
    assert!(yaml_text.contains("name: formal-ai"), "{yaml_answer}");
    assert!(yaml_text.contains("lines: 42"), "{yaml_answer}");

    // Feeding the emitted YAML back must reproduce the same values as JSON.
    let to_json = ["Convert this YAML to JSON:\n```yaml\n", &yaml_text, "\n```"].concat();
    let json_answer = solved(&to_json);
    let emitted = fenced(&json_answer, "json");
    let expected: serde_json::Value = serde_json::from_str(json_text).expect("probe json parses");
    let actual: serde_json::Value =
        serde_json::from_str(emitted.as_str()).expect("emitted json parses");
    assert_eq!(actual, expected, "{json_answer}");
}

// ---------------------------------------------------------------------------
// Handler level: each `handle_*` function directly.
// ---------------------------------------------------------------------------

/// Run one handler over a prompt and return its answer body.
macro_rules! answer_of {
    ($handler:path, $prompt:expr) => {{
        let prompt: &str = $prompt;
        let normalized = normalize_prompt(prompt);
        let mut log = EventLog::new();
        let answer = $handler(prompt, &normalized, &mut log)
            .unwrap_or_else(|| panic!("{} should answer this request", stringify!($handler)));
        answer.answer
    }};
}

#[test]
fn handler_code_debugging_reports_shifted_quotient() {
    let answer = answer_of!(
        formal_ai::handle_code_debugging,
        "What's wrong with `def average(xs): return sum(xs) / len(xs) - 1`?"
    );
    assert!(answer.contains("- 1"), "the shift must be named: {answer}");
    assert!(
        answer.contains("sum(xs)"),
        "the fixed form must be shown: {answer}"
    );
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn handler_regex_synthesis_composes_anchored_pattern() {
    let answer = answer_of!(
        formal_ai::handle_regex_synthesis,
        "Write a regular expression that matches five digits optionally followed by a hyphen and four digits"
    );
    assert!(
        answer.contains("^\\d{5}(-\\d{4})?$"),
        "the composed pattern must be anchored: {answer}"
    );
    assert!(answer.contains("Verified structurally"), "{answer}");
    assert!(answer.contains("No match was run"), "{answer}");
}

#[test]
fn handler_regex_synthesis_composes_at_least_repetition() {
    let answer = answer_of!(
        formal_ai::handle_regex_synthesis,
        "Write a regular expression for three or more digits"
    );
    assert!(
        answer.contains("\\d{3,}"),
        "the at-least repetition must render: {answer}"
    );
}

#[test]
fn handler_sql_synthesis_composes_filtered_select() {
    let answer = answer_of!(
        formal_ai::handle_sql_synthesis,
        "Write a SQL query that selects all users older than 30"
    );
    assert!(answer.contains("SELECT"), "{answer}");
    assert!(answer.contains("FROM users"), "{answer}");
    assert!(answer.contains("age > 30"), "{answer}");
    assert!(answer.contains("nothing was run"), "{answer}");
}

#[test]
fn handler_shell_compose_builds_find_command() {
    let answer = answer_of!(
        formal_ai::handle_shell_command_compose,
        "find .log files larger than 10 MB under /var"
    );
    assert!(answer.contains("find /var"), "{answer}");
    assert!(answer.contains("-name '*.log'"), "{answer}");
    assert!(answer.contains("-size +10M"), "{answer}");
    assert!(answer.contains("Not executed"), "{answer}");
}

#[test]
fn handler_code_explanation_names_function_and_promise() {
    let answer = answer_of!(
        formal_ai::handle_code_explanation,
        "Explain this code: `def average(xs): return sum(xs) / len(xs)`"
    );
    assert!(answer.contains("function"), "{answer}");
    assert!(answer.contains("average"), "{answer}");
    assert!(answer.contains("sum(xs) / len(xs)"), "{answer}");
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn handler_code_review_reports_bare_except_with_source() {
    let prompt = "Review this code:\n```python\ndef load(path):\n    try:\n        f = open(path)\n    except:\n        pass\n```";
    let answer = answer_of!(formal_ai::handle_code_review, prompt);
    assert!(
        answer.contains("except:"),
        "the offending line is quoted: {answer}"
    );
    assert!(answer.contains("docs.python.org"), "{answer}");
    assert!(answer.contains("no linter was run"), "{answer}");
}

#[test]
fn handler_test_generation_builds_palindrome_suite() {
    let answer = answer_of!(
        formal_ai::handle_test_generation,
        "Write tests for `is_palindrome(s)` ignoring case, spaces, and punctuation"
    );
    assert!(answer.contains("def test_"), "{answer}");
    assert!(answer.contains("RaceCar"), "{answer}");
    assert!(answer.contains("never odd or even"), "{answer}");
    assert!(answer.contains("def normalize(value):"), "{answer}");
    assert!(answer.contains("is_palindrome(normalize("), "{answer}");
    assert!(answer.contains("NOT executed"), "{answer}");
}

#[test]
fn handler_code_refactoring_rewrites_promise_chain() {
    let prompt = "Refactor this promise chain with async/await:\n```javascript\nfetch(url)\n  .then(r => r.json())\n  .then(d => render(d))\n  .catch(e => log(e));\n```";
    let answer = answer_of!(formal_ai::handle_code_refactoring, prompt);
    let code = fenced(&answer, "javascript");
    assert!(code.contains("async function run()"), "{answer}");
    assert!(code.contains("await fetch(url)"), "{answer}");
    assert!(code.contains("await r.json()"), "{answer}");
    assert!(code.contains("await render(d)"), "{answer}");
    assert!(code.contains("catch (e)"), "{answer}");
    assert!(answer.contains("No code was executed"), "{answer}");
}

#[test]
fn handler_format_conversion_round_trips_json_and_yaml() {
    let json_text = "{\"name\": \"formal-ai\", \"tags\": [\"rust\", \"lino\"], \"counts\": {\"lines\": 42, \"passed\": 41}}";
    let to_yaml = ["Convert this JSON to YAML:\n```json\n", json_text, "\n```"].concat();
    let yaml_answer = answer_of!(formal_ai::handle_format_conversion, &to_yaml);
    let yaml_text = fenced(&yaml_answer, "yaml");
    assert!(yaml_text.contains("name: formal-ai"), "{yaml_answer}");
    assert!(yaml_text.contains("- rust"), "{yaml_answer}");
    assert!(yaml_text.contains("lines: 42"), "{yaml_answer}");

    let to_json = ["Convert this YAML to JSON:\n```yaml\n", &yaml_text, "\n```"].concat();
    let json_answer = answer_of!(formal_ai::handle_format_conversion, &to_json);
    let emitted = fenced(&json_answer, "json");
    let expected: serde_json::Value = serde_json::from_str(json_text).expect("probe json parses");
    let actual: serde_json::Value =
        serde_json::from_str(emitted.as_str()).expect("emitted json parses");
    assert_eq!(actual, expected, "{json_answer}");
}

#[test]
fn unrelated_prompts_are_not_claimed_by_any_code_task_handler() {
    let unrelated = [
        "Hello, how are you today?",
        "What is the capital of France?",
    ];
    let handlers: [(
        fn(&str, &str, &mut EventLog) -> Option<formal_ai::engine::SymbolicAnswer>,
        &str,
    ); 9] = [
        (formal_ai::handle_code_debugging, "debugging"),
        (formal_ai::handle_regex_synthesis, "regex"),
        (formal_ai::handle_sql_synthesis, "sql"),
        (formal_ai::handle_shell_command_compose, "shell"),
        (formal_ai::handle_code_explanation, "explanation"),
        (formal_ai::handle_code_review, "review"),
        (formal_ai::handle_test_generation, "test generation"),
        (formal_ai::handle_code_refactoring, "refactoring"),
        (formal_ai::handle_format_conversion, "conversion"),
    ];
    for prompt in unrelated {
        let normalized = normalize_prompt(prompt);
        for (handler, name) in handlers {
            let mut log = EventLog::new();
            assert!(
                handler(prompt, &normalized, &mut log).is_none(),
                "{name} must not claim: {prompt}"
            );
        }
    }
}
