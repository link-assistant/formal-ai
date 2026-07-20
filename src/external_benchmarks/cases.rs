//! Upstream record -> executable benchmark case (issue #698).
//!
//! Each suite keeps its own field names, so this module normalises them into a
//! single `BenchmarkCase` the runner can drive through the solver. Prompts are
//! the upstream task text; nothing is rewritten to make a case easier.

use super::manifest::{Grading, SuiteManifest};

/// What a produced answer is checked against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expectation {
    /// Run `test_code`, then call `check(entry_point)` on the produced code.
    PythonUnitTest {
        test_code: String,
        entry_point: String,
    },
    /// Run `setup` and each assertion against the produced code.
    PythonAsserts { setup: String, asserts: Vec<String> },
    /// The gold answer as an upstream string (number, boxed expression, text).
    Value { expected: String },
}

/// One upstream case, ready to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCase {
    pub id: String,
    pub prompt: String,
    pub expectation: Expectation,
}

/// Turn `slice` upstream records into executable cases, preserving upstream
/// order so a rerun scores the same case set.
pub fn parse_cases(
    manifest: &SuiteManifest,
    records: &[String],
    slice: usize,
) -> Result<Vec<BenchmarkCase>, String> {
    let mut cases = Vec::new();
    for (index, record) in records.iter().enumerate() {
        if cases.len() == slice {
            break;
        }
        let value: serde_json::Value = serde_json::from_str(record).map_err(|error| {
            format!("{}: record {index} is not valid JSON: {error}", manifest.id)
        })?;
        cases.push(parse_case(manifest, &value, index)?);
    }
    Ok(cases)
}

fn parse_case(
    manifest: &SuiteManifest,
    value: &serde_json::Value,
    index: usize,
) -> Result<BenchmarkCase, String> {
    match manifest.id {
        "humaneval" => {
            let task_id = string_field(value, "task_id", manifest, index)?;
            let prompt = string_field(value, "prompt", manifest, index)?;
            Ok(BenchmarkCase {
                id: task_id,
                prompt: format!(
                    "Complete this Python function. Reply with the full implementation in a ```python code block.\n\n{prompt}"
                ),
                expectation: Expectation::PythonUnitTest {
                    test_code: string_field(value, "test", manifest, index)?,
                    entry_point: string_field(value, "entry_point", manifest, index)?,
                },
            })
        }
        "mbpp" => {
            let task_id = number_field(value, "task_id", manifest, index)?;
            let text = string_field(value, "text", manifest, index)?;
            let asserts = string_array_field(value, "test_list", manifest, index)?;
            Ok(BenchmarkCase {
                id: format!("MBPP/{task_id}"),
                prompt: format!(
                    "{text}\nReply with the Python code in a ```python code block. It must pass these tests:\n{}",
                    asserts.join("\n"),
                ),
                expectation: Expectation::PythonAsserts {
                    setup: value
                        .get("test_setup_code")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    asserts,
                },
            })
        }
        "gsm8k" => {
            let question = string_field(value, "question", manifest, index)?;
            let answer = string_field(value, "answer", manifest, index)?;
            let expected = answer
                .rsplit("####")
                .next()
                .unwrap_or_default()
                .trim()
                .replace(',', "");
            Ok(BenchmarkCase {
                id: format!("GSM8K/{index}"),
                prompt: question,
                expectation: Expectation::Value { expected },
            })
        }
        "math" => {
            let problem = string_field(value, "problem", manifest, index)?;
            let unique_id = value
                .get("unique_id")
                .and_then(serde_json::Value::as_str)
                .map_or_else(|| format!("MATH/{index}"), ToString::to_string);
            Ok(BenchmarkCase {
                id: unique_id,
                prompt: problem,
                expectation: Expectation::Value {
                    expected: string_field(value, "answer", manifest, index)?,
                },
            })
        }
        "object_counting" => Ok(BenchmarkCase {
            id: format!("object_counting/{index}"),
            prompt: string_field(value, "input", manifest, index)?,
            expectation: Expectation::Value {
                expected: target_field(value, manifest, index)?,
            },
        }),
        "coedit" => {
            let id = value
                .get("_id")
                .map_or_else(|| index.to_string(), ToString::to_string)
                .trim_matches('"')
                .to_string();
            Ok(BenchmarkCase {
                id: format!("CoEdIT/{id}"),
                prompt: string_field(value, "src", manifest, index)?,
                expectation: Expectation::Value {
                    expected: string_field(value, "tgt", manifest, index)?,
                },
            })
        }
        "swebench_lite" => {
            let instance = string_field(value, "instance_id", manifest, index)?;
            let statement = string_field(value, "problem_statement", manifest, index)?;
            let repository = string_field(value, "repo", manifest, index)?;
            Ok(BenchmarkCase {
                id: instance,
                prompt: format!(
                    "Repository {repository}. Resolve this issue and reply with the fix as a unified diff patch.\n\n{statement}"
                ),
                expectation: Expectation::Value {
                    expected: string_field(value, "patch", manifest, index)?,
                },
            })
        }
        other => Err(format!("no case parser for suite `{other}`")),
    }
}

fn string_field(
    value: &serde_json::Value,
    field: &str,
    manifest: &SuiteManifest,
    index: usize,
) -> Result<String, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| {
            format!(
                "{}: record {index} is missing the string field `{field}`",
                manifest.id
            )
        })
}

fn number_field(
    value: &serde_json::Value,
    field: &str,
    manifest: &SuiteManifest,
    index: usize,
) -> Result<i64, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            format!(
                "{}: record {index} is missing the numeric field `{field}`",
                manifest.id
            )
        })
}

fn string_array_field(
    value: &serde_json::Value,
    field: &str,
    manifest: &SuiteManifest,
    index: usize,
) -> Result<Vec<String>, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.as_str().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .filter(|entries| !entries.is_empty())
        .ok_or_else(|| {
            format!(
                "{}: record {index} is missing the string array `{field}`",
                manifest.id
            )
        })
}

/// BIG-bench targets are either a string or a single-entry list of strings.
fn target_field(
    value: &serde_json::Value,
    manifest: &SuiteManifest,
    index: usize,
) -> Result<String, String> {
    if let Some(target) = value.get("target").and_then(serde_json::Value::as_str) {
        return Ok(target.to_string());
    }
    value
        .get("target")
        .and_then(serde_json::Value::as_array)
        .and_then(|entries| entries.first())
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("{}: record {index} is missing a `target`", manifest.id))
}

/// The grading mode a suite's expectation implies, used by the runner to pick
/// a checker and by the ledger to record how a score was produced.
#[must_use]
pub const fn grading_for(manifest: &SuiteManifest) -> Grading {
    manifest.grading
}
