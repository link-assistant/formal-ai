//! Test-generation handler (issue #1177, E142 code-task family).
//!
//! Recognizes "write tests for …" requests (English and Russian; cue
//! phrases live in `data/seed/code-task-cues.lino`), identifies the function
//! under test (a backtick span like `is_palindrome(s)`, else the word after
//! "for"), matches it to a problem shape from the seed `test_cases` records
//! (palindrome / average / max; trigger words are seed data), applies the
//! requested normalizations (case, spaces, punctuation — word map in the
//! same seed file), and renders a pytest suite.
//!
//! The suite is NOT executed: executing generated tests is issue #1185's
//! scope, and the answer (a template from
//! `data/seed/multilingual-responses.lino`) says so. When the function
//! matches no known shape, a smoke-test skeleton is emitted and the
//! derivation says the shape was not recognized.

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::{LinoNode, parse_lino};

const CUES_PATH: &str = "data/seed/code-task-cues.lino";
const INTENT: &str = "test_generation";

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// Trigger phrases for one intent (optionally one named role) from
/// `data/seed/code-task-cues.lino`.
fn cue_phrases(intent: &str, role: &str) -> Vec<String> {
    let Some(text) = seed_text(CUES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "cues") {
        if record.find_child_value("intent") != intent {
            continue;
        }
        if record
            .children
            .iter()
            .find(|child| child.name == "intent")
            .map_or("", |child| child.find_child_value("role"))
            != role
        {
            continue;
        }
        for phrase in record
            .children
            .iter()
            .flat_map(|child| child.children.iter())
            .filter(|phrase| phrase.name == "phrase")
        {
            if !phrase.id.is_empty() {
                out.push(phrase.id.clone());
            }
        }
    }
    out
}

/// The `entry` records of one word map from `data/seed/code-task-cues.lino`.
fn word_entries(map: &str) -> Vec<LinoNode> {
    let Some(text) = seed_text(CUES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "map") {
        if record.find_child_value("name") != map {
            continue;
        }
        out.extend(
            record
                .children
                .iter()
                .flat_map(|child| child.children.iter())
                .filter(|entry| entry.name == "entry")
                .cloned(),
        );
    }
    out
}

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// Render the request→emission mapping rows through the shared seed
/// template.
fn mapping_rows(rows: &[(String, String)]) -> String {
    rows.iter()
        .map(|(request, emission)| {
            template(
                "mapping_line",
                &[("request", request), ("emission", emission)],
            )
        })
        .collect()
}

/// One sample case of a problem shape.
struct TestCase {
    input: String,
    /// "true"/"false" for predicate shapes.
    outcome: Option<String>,
    /// expected value for value-returning shapes.
    output: Option<String>,
}

/// One `test_cases` record: a problem shape with its trigger words.
struct Shape {
    name: String,
    triggers: Vec<String>,
    cases: Vec<TestCase>,
}

fn shapes() -> Vec<Shape> {
    let Some(text) = seed_text(CUES_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree
        .children
        .iter()
        .filter(|child| child.name == "test_cases")
    {
        let name = record.find_child_value("shape").to_string();
        if name.is_empty() {
            continue;
        }
        let mut triggers = Vec::new();
        let mut cases = Vec::new();
        // A shape's triggers and cases sit under its `shape` child.
        let Some(body) = record
            .children
            .first()
            .filter(|child| child.name == "shape")
        else {
            continue;
        };
        for child in &body.children {
            match child.name.as_str() {
                "trigger" if !child.id.is_empty() => triggers.push(child.id.clone()),
                "case" => cases.push(TestCase {
                    input: child.find_child_value("input").to_string(),
                    outcome: match child.find_child_value("outcome") {
                        "" => None,
                        value => Some(value.to_owned()),
                    },
                    output: match child.find_child_value("output") {
                        "" => None,
                        value => Some(value.to_owned()),
                    },
                }),
                _ => {}
            }
        }
        out.push(Shape {
            name,
            triggers,
            cases,
        });
    }
    out
}

/// The run of identifier characters starting at `text`.
fn identifier_at(text: &str) -> &str {
    let end = text
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(text.len());
    &text[..end]
}

/// The balanced-parenthesis span starting at the '(' at `open_at`.
fn inside_parens(text: &str, open_at: usize) -> &str {
    let mut depth = 0usize;
    for (offset, ch) in text[open_at..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open_at + 1..open_at + offset];
                }
            }
            _ => {}
        }
    }
    ""
}

/// The function under test: a backtick span like `is_palindrome(s)`, else
/// the word after "for". Returns (name, parameter list).
fn function_spec(prompt: &str) -> Option<(String, String)> {
    if let Some(start) = prompt.find('`')
        && let Some(end) = prompt[start + 1..].find('`')
    {
        let span = &prompt[start + 1..start + 1 + end];
        let name = identifier_at(span);
        if !name.is_empty() {
            let args = span
                .find('(')
                .map_or("s", |open| inside_parens(span, open));
            return Some((name.to_owned(), args.to_owned()));
        }
    }
    let words: Vec<&str> = prompt.split_whitespace().collect();
    for (index, word) in words.iter().enumerate() {
        if (*word == "for" || *word == "для")
            && let Some(next) = words.get(index + 1)
        {
            let name = identifier_at(next);
            if !name.is_empty() {
                return Some((name.to_owned(), "s".to_owned()));
            }
        }
    }
    None
}

/// The Python literal for a sample input: bracketed inputs pass through,
/// everything else becomes an escaped quoted string.
fn python_literal(input: &str) -> String {
    if input.starts_with('[') {
        return input.to_owned();
    }
    let escaped = input.replace('\\', "\\\\").replace('"', "\\\"");
    ["\"", &escaped, "\""].concat()
}

/// The test-function identifier for one sample input.
fn case_id(shape: &str, input: &str) -> String {
    let cleaned: String = input
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    if cleaned.is_empty() {
        return [shape, "_", "empty"].concat();
    }
    [shape, "_", &cleaned].concat()
}

/// Emit the suite lines for one shape's cases.
fn case_lines(
    shape: &Shape,
    function: &str,
    normalize: &str,
) -> (Vec<String>, Vec<(String, String)>) {
    let mut lines = Vec::new();
    let mut rows: Vec<(String, String)> = Vec::new();
    for case in &shape.cases {
        lines.push(["def ", "test_", &case_id(&shape.name, &case.input), "():"].concat());
        // normalize() wraps string inputs only; a list input passes through.
        let argument = if normalize.is_empty() || case.input.starts_with('[') {
            python_literal(&case.input)
        } else {
            [normalize, "(", &python_literal(&case.input), ")"].concat()
        };
        let call = [function, "(", &argument, ")"].concat();
        let assertion = match (&case.outcome, &case.output) {
            (Some(outcome), _) => {
                let expected = if outcome == "true" { "True" } else { "False" };
                ["    assert ", &call, " is ", expected].concat()
            }
            (None, Some(output)) => ["    assert ", &call, " == ", output].concat(),
            (None, None) => ["    assert ", &call].concat(),
        };
        lines.push(assertion.clone());
        rows.push((case.input.clone(), assertion));
    }
    (lines, rows)
}

/// The normalize helper for the requested properties, if any were
/// requested. Returns (helper lines, call prefix).
fn normalize_helper(properties: &[String]) -> (Vec<String>, String) {
    let lower = properties.iter().any(|p| p == "case");
    let filters = properties
        .iter()
        .any(|p| p == "spaces" || p == "punctuation");
    if lower && filters {
        (
            vec![
                ["def ", "normalize(value):"].concat(),
                [
                    "    return ",
                    "\"\".join(ch for ch in value.lower() if ch.isalnum())",
                ]
                .concat(),
            ],
            "normalize".to_owned(),
        )
    } else if filters {
        (
            vec![
                ["def ", "normalize(value):"].concat(),
                [
                    "    return ",
                    "\"\".join(ch for ch in value if ch.isalnum())",
                ]
                .concat(),
            ],
            "normalize".to_owned(),
        )
    } else if lower {
        (
            vec![
                ["def ", "normalize(value):"].concat(),
                ["    return ", "value.lower()"].concat(),
            ],
            "normalize".to_owned(),
        )
    } else {
        (Vec::new(), String::new())
    }
}

/// Try to recognize a test-writing request and generate a pytest suite for
/// the function under test.
///
/// Returns `None` when the prompt is not a
/// test-generation request; refuses by name when no function is identifiable.
pub fn handle_test_generation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let lower = prompt.to_lowercase();
    // An agent opt-in is the agent flow's to serve: it acts on the workspace,
    // where this chat composer only prints a suite.
    if crate::solver_helpers::is_agent_opt_in(&lower) {
        return None;
    }
    let cued = cue_phrases(INTENT, "request")
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()));
    if !cued {
        return None;
    }
    log.append("test_generation:request", "cued".to_owned());

    let Some((function, _params)) = function_spec(prompt) else {
        log.append("test_generation:refusal", "function=none".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            INTENT,
            "response:test_generation",
            &template("test_generation_refusal", &[]),
            0.4,
        ));
    };
    log.append("test_generation:function", function.clone());

    // Requested normalizations (word map; the matched words are echoed).
    let mut properties: Vec<String> = Vec::new();
    let mut property_words: Vec<String> = Vec::new();
    for entry in word_entries("test_property") {
        let word = entry.find_child_value("word");
        if normalized.contains(word) || lower.contains(word) {
            properties.push(entry.find_child_value("value").to_owned());
            property_words.push(word.to_owned());
        }
    }

    // The shape whose trigger words name the function under test.
    let table = shapes();
    let shape = table.iter().find(|shape| {
        shape
            .triggers
            .iter()
            .any(|trigger| function.contains(trigger.as_str()))
    });

    let mut normalize_applied = false;
    let (mut lines, mut rows): (Vec<String>, Vec<(String, String)>) = if let Some(shape) = shape {
        log.append("test_generation:shape", shape.name.clone());
        let (helper, mut call_prefix) = normalize_helper(&properties);
        // When every sample input is a list, normalize() would never be
        // called: drop it so the suite carries no dead helper.
        if !shape.cases.iter().any(|case| !case.input.starts_with('[')) {
            call_prefix.clear();
        }
        normalize_applied = !call_prefix.is_empty();
        let mut lines = Vec::new();
        if normalize_applied {
            lines.extend(helper);
            lines.push(String::new());
        }
        let (case_lines, case_rows) = case_lines(shape, &function, &call_prefix);
        lines.extend(case_lines);
        (lines, case_rows)
    } else {
        log.append("test_generation:shape", "none".to_owned());
        (
            vec![
                ["def ", "test_smoke():"].concat(),
                ["    assert ", &function, " is not None"].concat(),
            ],
            Vec::new(),
        )
    };
    if normalize_applied {
        for word in &property_words {
            rows.push((word.clone(), "normalize()".to_owned()));
        }
    }

    lines.push(String::new());
    let suite = lines.join("\n");
    log.append("test_generation:suite", format!("case={}", rows.len()));

    let body = template(
        "test_generation_suite",
        &[("suite", &suite), ("derivation", &mapping_rows(&rows))],
    );

    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:test_generation",
        &body,
        0.7,
    ))
}
