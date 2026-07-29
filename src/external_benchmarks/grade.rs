//! Grading upstream benchmark cases (issue #698).
//!
//! Grading is deliberately the upstream check, not a repository-local proxy:
//! Python suites execute the upstream unit tests, math suites compare the final
//! answer, editing suites compare the produced text with the gold edit. A case
//! passes only when the upstream criterion is met, so a 0% score stays 0%.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::cases::{BenchmarkCase, Expectation};
use super::manifest::Grading;
use super::vocabulary;

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
        Expectation::SweBench { .. } => {
            return failure(
                case,
                "SWE-bench instances must be graded together by the official harness",
            );
        }
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
        Grading::PythonUnitTest
        | Grading::PythonAsserts
        | Grading::SweBenchTests
        | Grading::NotApplicable => false,
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

/// Grade candidate patches with the pinned official SWE-bench evaluator.
///
/// Empty/non-diff replies are rejected using the evaluator's own empty-patch
/// criterion without starting Docker. If any real candidate patch exists, the
/// official harness applies it to the upstream image and runs the instance
/// tests; a harness or Docker failure is returned as infrastructure
/// unavailability rather than counted as a model failure.
pub fn grade_swebench(
    cases: &[BenchmarkCase],
    answers: &[String],
    workspace: &Path,
) -> Result<Vec<CaseOutcome>, String> {
    if cases.len() != answers.len() {
        return Err("SWE-bench cases and answers have different lengths".to_string());
    }
    let patches: Vec<Option<String>> = answers.iter().map(|answer| extract_diff(answer)).collect();
    if patches.iter().all(Option::is_none) {
        return Ok(cases
            .iter()
            .map(|case| {
                failure(
                    case,
                    "empty patch rejected by the official SWE-bench criterion",
                )
            })
            .collect());
    }

    ensure_swebench_runtime()?;
    fs::create_dir_all(workspace)
        .map_err(|error| format!("failed to create {}: {error}", workspace.display()))?;
    let dataset_path = workspace.join("dataset.json");
    let dataset: Result<Vec<serde_json::Value>, String> = cases
        .iter()
        .map(|case| match &case.expectation {
            Expectation::SweBench { record } => serde_json::from_str(record).map_err(|error| {
                vocabulary::render(
                    "external_benchmark_swe_record_error",
                    &[("case", &case.id), ("error", &error.to_string())],
                )
            }),
            _ => Err(vocabulary::render(
                "external_benchmark_not_swe_case",
                &[("case", &case.id)],
            )),
        })
        .collect();
    let dataset_bytes = serde_json::to_vec(&dataset?).map_err(|error| {
        vocabulary::render(
            "external_benchmark_swe_encode_error",
            &[("error", &error.to_string())],
        )
    })?;
    fs::write(&dataset_path, &dataset_bytes).map_err(|error| {
        vocabulary::render(
            "external_benchmark_swe_slice_write_error",
            &[
                ("path", &dataset_path.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })?;
    let predictions_path = workspace.join("predictions.jsonl");
    let mut predictions = String::new();
    for (case, patch) in cases.iter().zip(&patches) {
        let row = serde_json::json!({
            "instance_id": case.id,
            "model_name_or_path": "formal-ai",
            "model_patch": patch.as_deref().unwrap_or(""),
        });
        predictions.push_str(&row.to_string());
        predictions.push('\n');
    }
    fs::write(&predictions_path, &predictions).map_err(|error| {
        vocabulary::render(
            "external_benchmark_swe_predictions_write_error",
            &[
                ("path", &predictions_path.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })?;

    let run_id = format!(
        "formal_ai_{:016x}",
        fnv1a64(dataset_bytes.iter().copied().chain(predictions.bytes()))
    );
    let prior_logs = workspace.join("logs").join("run_evaluation").join(&run_id);
    if prior_logs.exists() {
        fs::remove_dir_all(&prior_logs).map_err(|error| {
            vocabulary::render(
                "external_benchmark_swe_clear_logs_error",
                &[
                    ("path", &prior_logs.display().to_string()),
                    ("error", &error.to_string()),
                ],
            )
        })?;
    }
    let report_path = workspace.join(format!("formal-ai.{run_id}.json"));
    if report_path.exists() {
        fs::remove_file(&report_path).map_err(|error| {
            vocabulary::render(
                "external_benchmark_swe_remove_report_error",
                &[
                    ("path", &report_path.display().to_string()),
                    ("error", &error.to_string()),
                ],
            )
        })?;
    }
    let mut command = Command::new("python3");
    command
        .args(["-m", "swebench.harness.run_evaluation", "--dataset_name"])
        .arg(&dataset_path)
        .arg("--predictions_path")
        .arg(&predictions_path)
        .args([
            "--max_workers",
            "1",
            "--run_id",
            &run_id,
            "--timeout",
            "900",
            "--cache_level",
            "none",
            "--instance_ids",
        ]);
    for case in cases {
        command.arg(&case.id);
    }
    let output = command.current_dir(workspace).output().map_err(|error| {
        vocabulary::render(
            "external_benchmark_swe_start_error",
            &[("error", &error.to_string())],
        )
    })?;
    if !output.status.success() {
        return Err(vocabulary::render(
            "external_benchmark_swe_exit_error",
            &[
                ("status", &output.status.to_string()),
                ("error", &truncate(&String::from_utf8_lossy(&output.stderr))),
            ],
        ));
    }

    let report = fs::read_to_string(&report_path).map_err(|error| {
        vocabulary::render(
            "external_benchmark_swe_missing_report",
            &[
                ("path", &report_path.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })?;
    outcomes_from_swebench_report(cases, &report)
}

fn fnv1a64(bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

fn ensure_swebench_runtime() -> Result<(), String> {
    let module = Command::new("python3")
        .args(["-c", "import swebench.harness.run_evaluation"])
        .output()
        .map_err(|error| {
            vocabulary::render(
                "external_benchmark_swe_inspect_error",
                &[("error", &error.to_string())],
            )
        })?;
    if !module.status.success() {
        return Err("the pinned official `swebench` Python harness is not installed".to_string());
    }
    let docker = Command::new("docker")
        .arg("info")
        .output()
        .map_err(|error| {
            vocabulary::render(
                "external_benchmark_swe_docker_error",
                &[("error", &error.to_string())],
            )
        })?;
    if !docker.status.success() {
        return Err(vocabulary::render(
            "external_benchmark_swe_docker_error",
            &[("error", &truncate(&String::from_utf8_lossy(&docker.stderr)))],
        ));
    }
    Ok(())
}

fn outcomes_from_swebench_report(
    cases: &[BenchmarkCase],
    report: &str,
) -> Result<Vec<CaseOutcome>, String> {
    let report: serde_json::Value = serde_json::from_str(report).map_err(|error| {
        vocabulary::render(
            "external_benchmark_swe_report_error",
            &[("error", &error.to_string())],
        )
    })?;
    let ids = |field: &str| -> BTreeSet<String> {
        report
            .get(field)
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .map(ToString::to_string)
            .collect()
    };
    let errors = ids("error_ids");
    if !errors.is_empty() {
        return Err(vocabulary::render(
            "external_benchmark_swe_infrastructure_error",
            &[("cases", &errors.into_iter().collect::<Vec<_>>().join(", "))],
        ));
    }
    let resolved = ids("resolved_ids");
    let empty = ids("empty_patch_ids");
    Ok(cases
        .iter()
        .map(|case| {
            if resolved.contains(&case.id) {
                CaseOutcome {
                    id: case.id.clone(),
                    passed: true,
                    detail: String::new(),
                }
            } else if empty.contains(&case.id) {
                failure(
                    case,
                    "empty patch rejected by the official SWE-bench criterion",
                )
            } else {
                failure(
                    case,
                    "candidate patch did not resolve the official SWE-bench tests",
                )
            }
        })
        .collect())
}

/// Extract a candidate unified diff without ever falling back to prose or a
/// language code block.
#[must_use]
pub fn extract_diff(answer: &str) -> Option<String> {
    let candidate = fenced_block(answer).unwrap_or_else(|| answer.trim().to_string());
    let start = candidate.find("diff --git")?;
    let patch = candidate[start..].trim();
    (!patch.is_empty()).then(|| patch.to_string())
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
