//! Grading upstream benchmark cases (issue #698).
//!
//! Grading is deliberately the upstream check, not a repository-local proxy:
//! Python suites execute the upstream unit tests, math suites compare the final
//! answer, editing suites compare the produced text with the gold edit. A case
//! passes only when the upstream criterion is met, so a 0% score stays 0%.

use std::fs;
use std::path::Path;
use std::process::Command;

use super::cases::{BenchmarkCase, Expectation};
use super::manifest::Grading;

/// Wall-clock ceiling for one upstream Python test run.
const PYTHON_TIMEOUT_SECONDS: &str = "20";

/// The outcome of grading one case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseOutcome {
    pub id: String,
    pub passed: bool,
    /// Why the case failed (empty when it passed), truncated for the report.
    pub detail: String,
}

/// Grade `answer` (the solver's reply) against the upstream expectation.
#[must_use]
pub fn grade_case(
    case: &BenchmarkCase,
    grading: Grading,
    answer: &str,
    workspace: &Path,
) -> CaseOutcome {
    let (passed, detail) = match &case.expectation {
        Expectation::PythonUnitTest {
            test_code,
            entry_point,
        } => {
            let Some(code) = extract_python(answer) else {
                return failure(case, "answer contains no Python code");
            };
            let program = format!("{code}\n\n{test_code}\n\ncheck({entry_point})\n");
            run_python(&program, workspace, &case.id)
        }
        Expectation::PythonAsserts { setup, asserts } => {
            let Some(code) = extract_python(answer) else {
                return failure(case, "answer contains no Python code");
            };
            let program = format!("{code}\n\n{setup}\n\n{}\n", asserts.join("\n"));
            run_python(&program, workspace, &case.id)
        }
        Expectation::Value { expected } => grade_value(grading, expected, answer),
    };
    CaseOutcome {
        id: case.id.clone(),
        passed,
        detail: if passed {
            String::new()
        } else {
            truncate(&detail)
        },
    }
}

fn grade_value(grading: Grading, expected: &str, answer: &str) -> (bool, String) {
    let matched = match grading {
        Grading::NumericAnswer => {
            let gold = normalize_number(&expected.replace(',', ""));
            gold.is_some_and(|gold| final_number(answer).is_some_and(|found| found == gold))
        }
        Grading::BoxedAnswer => normalize(&final_answer(answer)) == normalize(expected),
        Grading::ExactText => {
            let gold = normalize(expected);
            normalize(answer) == gold || normalize(&final_answer(answer)) == gold
        }
        Grading::UnifiedDiff => {
            let produced = extract_python(answer).unwrap_or_else(|| answer.to_string());
            normalize_diff(&produced) == normalize_diff(expected)
        }
        Grading::PythonUnitTest | Grading::PythonAsserts | Grading::NotApplicable => false,
    };
    (
        matched,
        format!(
            "expected `{}`, answer `{}`",
            truncate(expected),
            truncate(answer)
        ),
    )
}

/// A unified diff compares equal when its non-empty content lines do, so
/// hunk-header line numbers and trailing whitespace do not decide the score.
fn normalize_diff(patch: &str) -> Vec<String> {
    patch
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with("@@") && !line.starts_with("index "))
        .map(ToString::to_string)
        .collect()
}

fn failure(case: &BenchmarkCase, detail: &str) -> CaseOutcome {
    CaseOutcome {
        id: case.id.clone(),
        passed: false,
        detail: detail.to_string(),
    }
}

/// Whether a `python3` interpreter is callable in this environment.
#[must_use]
pub fn python_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn run_python(program: &str, workspace: &Path, case_id: &str) -> (bool, String) {
    let file_name = format!("{}.py", case_id.replace(['/', ' ', '.'], "_"));
    let path = workspace.join(file_name);
    if let Err(error) = fs::create_dir_all(workspace) {
        return (
            false,
            format!("failed to create {}: {error}", workspace.display()),
        );
    }
    if let Err(error) = fs::write(&path, program) {
        return (
            false,
            format!("failed to write {}: {error}", path.display()),
        );
    }
    let output = Command::new("timeout")
        .arg(PYTHON_TIMEOUT_SECONDS)
        .arg("python3")
        .arg(&path)
        .current_dir(workspace)
        .output();
    match output {
        Ok(output) if output.status.success() => (true, String::new()),
        Ok(output) => (
            false,
            format!(
                "python exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim(),
            ),
        ),
        Err(error) => (false, format!("failed to run python3: {error}")),
    }
}

/// Pull the first fenced code block out of an answer, falling back to the whole
/// answer when it already looks like Python source.
#[must_use]
pub fn extract_python(answer: &str) -> Option<String> {
    if let Some(block) = fenced_block(answer) {
        return Some(block);
    }
    (answer.contains("def ") || answer.contains("import ")).then(|| answer.to_string())
}

fn fenced_block(answer: &str) -> Option<String> {
    let start = answer.find("```")?;
    let after_fence = &answer[start + 3..];
    let body_start = after_fence.find('\n')? + 1;
    let body = &after_fence[body_start..];
    let end = body.find("```").unwrap_or(body.len());
    let block = body[..end].trim_end();
    (!block.trim().is_empty()).then(|| block.to_string())
}

/// The last standalone number in an answer, normalised the way GSM8K grades
/// (thousands separators and a trailing period removed).
#[must_use]
pub fn final_number(answer: &str) -> Option<String> {
    let cleaned = answer.replace(',', "");
    let mut found = None;
    let mut current = String::new();
    for character in cleaned.chars().chain(std::iter::once(' ')) {
        if character.is_ascii_digit()
            || (character == '.' && !current.is_empty())
            || (character == '-' && current.is_empty())
        {
            current.push(character);
        } else {
            if let Some(number) = normalize_number(&current) {
                found = Some(number);
            }
            current.clear();
        }
    }
    found
}

fn normalize_number(candidate: &str) -> Option<String> {
    let trimmed = candidate.trim_end_matches('.');
    if trimmed.is_empty() || !trimmed.chars().any(|character| character.is_ascii_digit()) {
        return None;
    }
    let normalized = trimmed.parse::<f64>().ok().map_or_else(
        || trimmed.to_string(),
        |number| {
            if number.fract().abs() < f64::EPSILON {
                // `{:.0}` renders the integral value without an f64 -> i64 cast.
                format!("{number:.0}")
            } else {
                trimmed.to_string()
            }
        },
    );
    Some(normalized)
}

/// The final answer expression: the last `\boxed{...}` when present, otherwise
/// the last non-empty line.
#[must_use]
pub fn final_answer(answer: &str) -> String {
    if let Some(boxed) = last_boxed(answer) {
        return boxed;
    }
    answer
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn last_boxed(answer: &str) -> Option<String> {
    let start = answer.rfind("\\boxed{")? + "\\boxed{".len();
    let mut depth = 1_usize;
    let mut body = String::new();
    for character in answer[start..].chars() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(body);
                }
            }
            _ => {}
        }
        body.push(character);
    }
    None
}

/// Whitespace- and markup-insensitive comparison for text and math answers.
#[must_use]
pub fn normalize(value: &str) -> String {
    value
        .trim()
        .trim_matches('$')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches('.')
        .to_lowercase()
}

fn truncate(value: &str) -> String {
    const LIMIT: usize = 240;
    let single_line = value.replace('\n', "\\n");
    if single_line.chars().count() <= LIMIT {
        return single_line;
    }
    let cut: String = single_line.chars().take(LIMIT).collect();
    format!("{cut}…")
}
