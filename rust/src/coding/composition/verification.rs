//! Bounded execution and independently computed verification examples.

use std::fmt::Write as _;
use std::time::Duration;

use super::{Draft, template};
use crate::agent::{AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig};
use crate::coding::concept_discovery::ConceptMap;
use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::task_spec::{ArtifactShape, CodingTaskSpec, Example};

/// The address space one verification run of a Python draft may map. The
/// examples a draft is checked against are a handful of small literals; no
/// plausible solution needs more than a fraction of this to answer them.
const VERIFICATION_ADDRESS_SPACE_BYTES: u64 = 512 * 1024 * 1024;

pub(super) fn verify(
    spec: &CodingTaskSpec,
    draft: &Draft,
    examples: &[Example],
    concepts: &ConceptMap,
    catalog: &FragmentCatalog,
) -> (bool, String) {
    if spec.language == "python"
        && spec.artifact_shape == ArtifactShape::Function
        && examples.is_empty()
    {
        return (false, "no executable examples or derived tests".to_owned());
    }
    let expected_stdout = if spec.artifact_shape == ArtifactShape::Program {
        let Some(expected) = program_stdout_expectation(spec, concepts) else {
            return (false, "no bounded expected stdout".to_owned());
        };
        Some(expected)
    } else {
        None
    };
    let mut script = draft.source.clone();
    if spec.language == "python" && spec.artifact_shape == ArtifactShape::Function {
        let Ok(entrypoint) = template(catalog, "python_test_entrypoint", &[]) else {
            return (false, "missing fragment:python_test_entrypoint".to_owned());
        };
        script.push_str(&entrypoint);
        // Issue #1639: the address-space cap is set before any example calls
        // the draft, so a candidate that materializes a combinatorial
        // explosion fails with `MemoryError` instead of exhausting the host.
        // Where the platform has no such limit the guard is a no-op and the
        // search-level bound on self-amplifying fragments still applies.
        let limit = VERIFICATION_ADDRESS_SPACE_BYTES.to_string();
        if let Ok(guard) = template(
            catalog,
            "python_memory_guard",
            &[("indent", "    "), ("bytes", &limit)],
        ) {
            script.push_str(&guard);
        }
        for example in examples {
            let _ = writeln!(
                script,
                "    assert {}({}) == {}",
                draft.callable_name,
                example.arguments.join(", "),
                example.expected
            );
        }
    }
    let config = AgentWorkspaceConfig {
        time_budget: Duration::from_secs(5),
        ..AgentWorkspaceConfig::default()
    };
    let Ok(mut workspace) = AgentWorkspace::for_prompt(
        &format!("coding-discovery:{}:{}", spec.name, draft.id),
        &config,
    ) else {
        return (false, "could not create bounded workspace".to_owned());
    };
    let (file, command) = match spec.language.as_str() {
        "python" => ("solution.py", "python3 solution.py"),
        "rust" if examples.is_empty() => ("solution.rs", "rustc --crate-type lib solution.rs"),
        "rust" => {
            return (
                false,
                "verification_unavailable:rust_example_adapter".to_owned(),
            );
        }
        language => {
            return (
                false,
                format!("verification_unavailable:language={language}"),
            );
        }
    };
    workspace.create_file(file, &script);
    workspace.run_command(command);
    let run = workspace.finish();
    let command_succeeded = run.status == AgentRunStatus::Completed
        && run
            .command_results
            .iter()
            .any(|result| result.status_code == Some(0) && !result.timed_out);
    let output_matches = expected_stdout.is_none_or(|expected| {
        run.command_results
            .last()
            .is_some_and(|result| result.stdout.trim_end() == expected)
    });
    let passed = command_succeeded && output_matches;
    let detail = run.command_results.last().map_or_else(
        || "no command result".to_owned(),
        |result| {
            format!(
                "exit={:?};timed_out={};stderr={}",
                result.status_code,
                result.timed_out,
                result.stderr.trim()
            )
        },
    );
    (passed, detail)
}

const MAX_DERIVED_RANGE: usize = 1_000;

fn verification_range(spec: &CodingTaskSpec) -> Option<std::ops::RangeInclusive<usize>> {
    let bound = first_positive_integer(&spec.requirement_sentences)?
        .parse::<usize>()
        .ok()?;
    (bound <= MAX_DERIVED_RANGE).then_some(1..=bound)
}

fn program_stdout_expectation(spec: &CodingTaskSpec, concepts: &ConceptMap) -> Option<String> {
    if let Some(expected) = &spec.expected_stdout {
        return Some(expected.clone());
    }
    if concepts
        .structure_ids()
        .iter()
        .any(|structure| structure == "range_inclusive")
    {
        return Some(
            verification_range(spec)?
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    None
}

pub(super) fn derived_examples(spec: &CodingTaskSpec, concepts: &ConceptMap) -> Vec<Example> {
    let structures = concepts.structure_ids();
    let has = |id: &str| structures.iter().any(|structure| structure == id);
    // Exercise the empty and nonempty domains without borrowing any task's
    // example inputs. Compute the expected result independently of the draft.
    let sequence = |length| {
        (-2_i32..)
            .filter(|value| *value != 0)
            .take(length)
            .collect::<Vec<_>>()
    };
    let lengths = [0, 5];
    if has("tuple_of") && has("reduce_sum") && has("reduce_product") {
        return lengths
            .map(|length| {
                let values = sequence(length);
                let sum = values.iter().sum::<i32>();
                let product = values.iter().product::<i32>();
                Example {
                    arguments: vec![format!("{values:?}")],
                    expected: format!("({sum}, {product})"),
                }
            })
            .into();
    }
    if has("quantifier_any") && has("pairwise_distinct") && has("predicate_abs_diff_lt") {
        return [(0.0, 1.0), (1.0, 1.0), (1.0, 1.5), (2.0, 1.5)]
            .map(|(gap, threshold): (f64, f64)| {
                let values: Vec<_> = (0..4).map(|position| f64::from(position) * gap).collect();
                Example {
                    arguments: vec![format!("{values:?}"), threshold.to_string()],
                    expected: if gap < threshold { "True" } else { "False" }.to_owned(),
                }
            })
            .into();
    }
    if has("reduce_count") && has("vowel_character_class") {
        let vowels = |letter: u8| b"aeiou".contains(&letter.to_ascii_lowercase());
        return [false, true]
            .map(|include_vowels| {
                let input: String = (b'a'..=b'z')
                    .filter(|letter| include_vowels || !vowels(*letter))
                    .map(char::from)
                    .collect();
                Example {
                    arguments: vec![format!("{input:?}")],
                    expected: input
                        .bytes()
                        .filter(|letter| vowels(*letter))
                        .count()
                        .to_string(),
                }
            })
            .into();
    }
    if has("set_intersection") {
        let left = sequence(4);
        return [2, 5]
            .map(|offset| {
                let right: Vec<_> = left.iter().map(|value| value + offset).collect();
                let shared: Vec<_> = left
                    .iter()
                    .copied()
                    .filter(|value| right.contains(value))
                    .collect();
                Example {
                    arguments: vec![python_int_tuple(&left), python_int_tuple(&right)],
                    expected: python_int_tuple(&shared),
                }
            })
            .into();
    }
    if has("distinct_elements") && (has("reduce_count") || has("reduce_len")) {
        return lengths
            .map(|length| {
                let values: Vec<_> = sequence(length)
                    .into_iter()
                    .flat_map(|value| [value; 2])
                    .collect();
                Example {
                    arguments: vec![format!("{values:?}")],
                    expected: values
                        .into_iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        .to_string(),
                }
            })
            .into();
    }
    if has("range_inclusive") {
        let ranges = if spec.parameters.is_empty() {
            verification_range(spec).into_iter().collect()
        } else {
            vec![1..=2, 1..=5]
        };
        return ranges
            .into_iter()
            .map(|range| Example {
                arguments: if spec.parameters.is_empty() {
                    Vec::new()
                } else {
                    vec![range.end().to_string()]
                },
                expected: format!("{:?}", range.collect::<Vec<_>>()),
            })
            .collect();
    }
    Vec::new()
}

fn python_int_tuple(values: &[i32]) -> String {
    let rendered = values
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let trailing = if values.len() == 1 { "," } else { "" };
    format!("({rendered}{trailing})")
}

fn first_positive_integer(sentences: &[String]) -> Option<String> {
    let numeric = sentences
        .iter()
        .flat_map(|sentence| sentence.split(|character: char| !character.is_ascii_digit()))
        .find(|token| token.parse::<usize>().is_ok_and(|number| number > 0))
        .map(str::to_owned);
    if numeric.is_some() {
        return numeric;
    }
    let normalized = crate::engine::normalize_prompt(&sentences.join(" "));
    crate::seed::lexicon()
        .arithmetic_normalization_tables()
        .0
        .into_iter()
        .find(|(surface, value)| {
            value.parse::<usize>().is_ok_and(|number| number > 0)
                && normalized.split_whitespace().any(|word| word == surface)
        })
        .map(|(_, value)| value)
}
