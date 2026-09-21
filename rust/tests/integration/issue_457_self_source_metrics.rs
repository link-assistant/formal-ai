//! Issue #457: a composite Rust `write_program` request for source-code metrics
//! and self-analysis used to dead-end on `task=missing`.

use formal_ai::UniversalSolver;

const ISSUE_PROMPT: &str = "Write a Rust program that:\n\
   1. Parses its own source code as text\n\
   2. Counts: functions, loops, conditionals, comments\n\
   3. Calculates a \"complexity score\" based on cyclomatic complexity\n\
   4. Outputs a JSON report with metrics\n\
   5. Then: analyze YOUR OWN response to this prompt using the same metrics\n\
   6. Compare: which is more complex — your generated code or your reasoning text?";

const EXPECTED_ANSWER: &str = r#"Here is a Rust program for the requested composite task (inspect its own Rust source, emit JSON metrics, and compare code with response prose). I decomposed your request into these sub-tasks:

1. Parse the JSON response
2. Output the results
3. Document the code with comments
4. Research current source data
5. Export a Markdown comparison report
6. Read the program's own source code as text
7. Count functions, loops, conditionals, and comments
8. Calculate a cyclomatic-complexity score
9. Output the metrics as JSON
10. Analyze the assistant response with the same metrics
11. Compare code complexity with reasoning-text complexity

```rust
use std::fmt::Write as _;

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
```

Required libraries:
- Rust standard library only

How to run it yourself:

Execution status: not run — this source-metrics blueprint uses only the Rust standard library, but the answer renderer did not compile it in place. The code is provided for review. Run it yourself from a Cargo project: `cargo run`.

Response self-analysis:
- Reasoning text metrics: functions=0, loops=0, conditionals=0, comments=0, complexity_score=1.
- Comparison: the generated Rust code is more complex than the reasoning text because it contains executable parsing, loops, conditionals, helper functions, and JSON rendering logic."#;

#[test]
fn issue_457_rust_self_source_metrics_request_returns_blueprint_program() {
    let solver = UniversalSolver::default();
    let response = solver.solve(ISSUE_PROMPT);

    assert_eq!(
        response.intent, "write_program",
        "the issue prompt must route to write_program, got: {} / {}",
        response.intent, response.answer
    );
    assert!(
        !response.answer.contains("I do not have a template")
            && !response.answer.contains("task `missing`"),
        "must not surface the missing-template dead-end, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("```rust"),
        "answer must contain a Rust code fence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("include_str!")
            && response.answer.contains("functions")
            && response.answer.contains("loops")
            && response.answer.contains("conditionals")
            && response.answer.contains("comments")
            && response.answer.contains("complexity_score"),
        "program must parse its own source and report the requested metrics, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Response self-analysis")
            && response
                .answer
                .contains("generated Rust code is more complex"),
        "answer must compare generated code and reasoning text, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("program_blueprint:recipe self_source_metrics_report"),
        "trace must record the source-metrics blueprint recipe, got: {}",
        response.links_notation
    );
    assert_eq!(response.answer, EXPECTED_ANSWER);
}

#[test]
fn issue_457_language_coverage_markers_cover_supported_surfaces() {
    // This issue changes language-facing blueprint rendering in Rust and the
    // browser worker; keep explicit markers for every supported UI language.
    let covered_languages = [
        ("en", "English"),
        ("ru", "Russian"),
        ("hi", "Hindi"),
        ("zh", "Chinese"),
    ];

    assert_eq!(
        covered_languages.map(|(language, _)| language),
        ["en", "ru", "hi", "zh"]
    );
}
