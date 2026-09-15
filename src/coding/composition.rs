//! Draft construction, bounded execution, and least-action selection.

use std::fmt::Write as _;
use std::time::Duration;

use crate::agent::{AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig};
use crate::coding::concept_discovery::{CandidatePart, ConceptMap, structural_meanings};
use crate::coding::python_render::{render_function, runtime_template};
use crate::coding::task_spec::{ArtifactShape, CodingTaskSpec, Example};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftAttempt {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedDraft {
    pub id: String,
    pub source: String,
    pub assertion_count: usize,
    pub source_urls: Vec<String>,
    pub source_licenses: Vec<String>,
    pub composition: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompositionOutcome {
    pub selected: Option<VerifiedDraft>,
    pub attempts: Vec<DraftAttempt>,
    pub research_trail: String,
}

pub(super) struct Draft {
    pub(super) id: String,
    pub(super) source: String,
    pub(super) callable_name: String,
    pub(super) source_urls: Vec<String>,
    pub(super) source_licenses: Vec<String>,
    pub(super) composition: String,
    pub(super) action_cost: usize,
}

#[must_use]
pub fn compose(spec: &CodingTaskSpec, concepts: &ConceptMap) -> CompositionOutcome {
    if spec.language != "python" {
        return CompositionOutcome {
            research_trail: format!("renderer_unavailable:language={}", spec.language),
            ..CompositionOutcome::default()
        };
    }
    let examples = if spec.examples.is_empty() {
        let derived = derived_examples(spec, concepts);
        if derived.is_empty() {
            source_examples(concepts)
        } else {
            derived
        }
    } else {
        spec.examples.clone()
    };
    let mut drafts = if spec.artifact_shape == ArtifactShape::Function {
        candidate_drafts(spec, concepts)
    } else {
        Vec::new()
    };
    drafts.extend(structural_drafts(spec, concepts));
    drafts.extend(super::structural_composition::additional_drafts(
        spec, concepts,
    ));
    let mut attempts = Vec::new();
    let mut passing = Vec::new();
    for draft in drafts {
        if crate::coding::validated_program_cst("python", &draft.source).is_none() {
            attempts.push(DraftAttempt {
                id: draft.id,
                passed: false,
                detail: "candidate did not form a valid Python CST".to_owned(),
            });
            continue;
        }
        let (passed, detail) = verify(spec, &draft, &examples, concepts);
        attempts.push(DraftAttempt {
            id: draft.id.clone(),
            passed,
            detail,
        });
        if passed {
            passing.push(draft);
        }
    }
    passing.sort_by(|left, right| {
        left.action_cost
            .cmp(&right.action_cost)
            .then_with(|| left.source.len().cmp(&right.source.len()))
            .then_with(|| left.id.cmp(&right.id))
    });
    let selected = passing.into_iter().next().map(|draft| VerifiedDraft {
        id: draft.id,
        source: draft.source,
        assertion_count: if spec.artifact_shape == ArtifactShape::Program {
            1
        } else {
            examples.len()
        },
        source_urls: draft.source_urls,
        source_licenses: draft.source_licenses,
        composition: draft.composition,
    });
    let research_trail = if selected.is_some() {
        render_comparison(&attempts)
    } else {
        render_failure_trail(spec, concepts, &examples, &attempts)
    };
    CompositionOutcome {
        selected,
        attempts,
        research_trail,
    }
}

fn candidate_drafts(spec: &CodingTaskSpec, concepts: &ConceptMap) -> Vec<Draft> {
    concepts
        .needs
        .iter()
        .flat_map(|need| need.candidates.iter())
        .filter_map(|candidate| candidate_draft(spec, candidate))
        .collect()
}

fn candidate_draft(spec: &CodingTaskSpec, candidate: &CandidatePart) -> Option<Draft> {
    match candidate.kind.as_str() {
        "stdlib" => {
            let arguments = spec
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let imports = candidate
                .id
                .split_once('.')
                .map(|(module, _)| template("python_import", &[("module", module)]));
            let call = format!("{}({arguments})", candidate.id);
            let body = template("python_return", &[("expression", &call)]);
            Some(Draft {
                id: format!("stdlib:{}", candidate.id),
                source: render_function(spec, &body, imports),
                callable_name: spec.name.clone(),
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: format!("direct_stdlib({})", candidate.id),
                action_cost: 1,
            })
        }
        "wikifunctions_implementation" => {
            let body = adapt_implementation(candidate.code.as_deref()?, spec)?;
            Some(Draft {
                id: format!("part:{}", candidate.id),
                source: render_function(spec, &body, Vec::new()),
                callable_name: spec.name.clone(),
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: format!("direct_wrap({})", candidate.id),
                action_cost: 2,
            })
        }
        "wikifunctions_recurrence" => Some(Draft {
            id: format!("recurrence:{}", candidate.id),
            source: candidate.code.clone()?,
            callable_name: candidate.callable_name.clone()?,
            source_urls: vec![candidate.source_url.clone()],
            source_licenses: vec![candidate.license.clone()],
            composition: format!("source_recurrence({})", candidate.id),
            action_cost: 3,
        }),
        "source_program" => Some(Draft {
            id: format!("source:{}", candidate.id),
            source: candidate.code.clone()?,
            callable_name: candidate.callable_name.clone()?,
            source_urls: vec![candidate.source_url.clone()],
            source_licenses: vec![candidate.license.clone()],
            composition: candidate.label.clone(),
            action_cost: 4,
        }),
        _ => None,
    }
}

fn source_examples(concepts: &ConceptMap) -> Vec<Example> {
    let mut examples = concepts
        .needs
        .iter()
        .flat_map(|need| need.candidates.iter())
        .flat_map(|candidate| candidate.source_tests.iter().cloned())
        .collect::<Vec<_>>();
    examples.sort_by(|left, right| {
        left.arguments
            .cmp(&right.arguments)
            .then_with(|| left.expected.cmp(&right.expected))
    });
    examples.dedup();
    examples
}

fn structural_drafts(spec: &CodingTaskSpec, concepts: &ConceptMap) -> Vec<Draft> {
    let structures = concepts.structure_ids();
    let has = |id: &str| structures.iter().any(|structure| structure == id);
    let names = spec
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<Vec<_>>();
    let mut drafts = Vec::new();
    if spec.artifact_shape == ArtifactShape::Program {
        if has("print_stdout")
            && let Some(expected) = &spec.expected_stdout
            && let Ok(literal) = serde_json::to_string(expected)
        {
            let source = idiom("print_stdout", &[("value", &literal)]);
            drafts.push(structural_program_draft("print_stdout(literal)", source));
        }
        if has("range_inclusive") {
            let bound = first_positive_integer(&spec.requirement_sentences);
            if let Some(bound) = bound {
                let range = idiom("range_inclusive", &[("bound", &bound)]);
                let output = idiom("print_stdout", &[("value", "number")]);
                let source = template(
                    "python_for_each",
                    &[("item", "number"), ("items", &range), ("body", &output)],
                );
                drafts.push(structural_program_draft(
                    "for_each(range_inclusive,print_stdout)",
                    source,
                ));
            }
        }
        return drafts;
    }
    if has("tuple_of") && has("reduce_sum") && has("reduce_product") && names.len() == 1 {
        let sum = idiom("reduce_sum", &[("items", names[0])]);
        let product = idiom("reduce_product", &[("items", names[0])]);
        let expression = idiom("tuple_of", &[("left", &sum), ("right", &product)]);
        let body = template("python_return", &[("expression", &expression)]);
        let import = template("python_import", &[("module", "math")]);
        drafts.push(structural_draft(
            spec,
            "tuple_of(reduce_sum,reduce_product)",
            &body,
            [import],
            concepts,
        ));
    }
    if has("quantifier_any")
        && has("pairwise_distinct")
        && has("predicate_abs_diff_lt")
        && names.len() >= 2
    {
        let pairs = idiom("pairwise_distinct", &[("items", names[0])]);
        let predicate = idiom(
            "predicate_abs_diff_lt",
            &[
                ("left", "left"),
                ("right", "right"),
                ("threshold", names[1]),
            ],
        );
        let expression = idiom(
            "quantifier_any",
            &[
                ("predicate", &predicate),
                ("item", "left, right"),
                ("items", &pairs),
            ],
        );
        let body = template("python_return", &[("expression", &expression)]);
        let import = template("python_import", &[("module", "itertools")]);
        drafts.push(structural_draft(
            spec,
            "quantifier_any(pairwise_distinct(predicate_abs_diff_lt))",
            &body,
            [import],
            concepts,
        ));
    }
    if has("reduce_count") && has("vowel_character_class") && names.len() == 1 {
        let vowels = idiom("vowel_character_class", &[]);
        let predicate = idiom(
            "membership",
            &[("item", "character"), ("collection", &vowels)],
        );
        let expression = idiom(
            "reduce_count",
            &[
                ("item", "character"),
                ("items", names[0]),
                ("predicate", &predicate),
            ],
        );
        let body = template("python_return", &[("expression", &expression)]);
        drafts.push(structural_draft(
            spec,
            "reduce_count(vowel_character_class)",
            &body,
            Vec::<String>::new(),
            concepts,
        ));
    }
    if has("set_intersection") && names.len() >= 2 {
        let intersection = idiom(
            "set_intersection",
            &[("left", names[0]), ("right", names[1])],
        );
        let sorted = idiom("sort_ascending", &[("items", &intersection)]);
        let tuple = template("python_materialize_tuple", &[("items", &sorted)]);
        let body = template("python_return", &[("expression", &tuple)]);
        drafts.push(structural_draft(
            spec,
            "tuple_of(sort_ascending(set_intersection))",
            &body,
            Vec::<String>::new(),
            concepts,
        ));
    }
    if has("distinct_elements") && (has("reduce_count") || has("reduce_len")) && names.len() == 1 {
        let items = if has("case_insensitive") {
            idiom("case_insensitive", &[("text", names[0])])
        } else {
            names[0].to_owned()
        };
        let distinct = idiom("distinct_elements", &[("items", &items)]);
        let expression = idiom("reduce_len", &[("items", &distinct)]);
        let body = template("python_return", &[("expression", &expression)]);
        drafts.push(structural_draft(
            spec,
            if has("case_insensitive") {
                "reduce_len(distinct_elements(case_insensitive))"
            } else {
                "reduce_len(distinct_elements)"
            },
            &body,
            Vec::<String>::new(),
            concepts,
        ));
    }
    if has("filter_only") && has("membership") && names.len() >= 2 {
        let predicate = idiom("membership", &[("item", names[1]), ("collection", "item")]);
        let expression = idiom(
            "filter_only",
            &[
                ("item", "item"),
                ("items", names[0]),
                ("predicate", &predicate),
            ],
        );
        let body = template("python_return", &[("expression", &expression)]);
        drafts.push(structural_draft(
            spec,
            "filter_only(membership)",
            &body,
            Vec::<String>::new(),
            concepts,
        ));
    }
    if has("running_prefix") && has("reduce_max") && names.len() == 1 {
        let expression = idiom(
            "running_prefix",
            &[("items", names[0]), ("operation", "max")],
        );
        let materialized = template("python_materialize_list", &[("items", &expression)]);
        let body = template("python_return", &[("expression", &materialized)]);
        let import = template("python_import", &[("module", "itertools")]);
        drafts.push(structural_draft(
            spec,
            "running_prefix(reduce_max)",
            &body,
            [import],
            concepts,
        ));
    }
    if has("count_overlapping") && names.len() >= 2 {
        let expression = idiom(
            "count_overlapping",
            &[("text", names[0]), ("substring", names[1])],
        );
        let body = template("python_return", &[("expression", &expression)]);
        drafts.push(structural_draft(
            spec,
            "count_overlapping",
            &body,
            Vec::<String>::new(),
            concepts,
        ));
    }
    if has("range_inclusive") && names.len() <= 1 {
        let bound = names
            .first()
            .copied()
            .map(str::to_owned)
            .or_else(|| first_positive_integer(&spec.requirement_sentences));
        let Some(bound) = bound else {
            return drafts;
        };
        let range = idiom("range_inclusive", &[("bound", &bound)]);
        let list = template("python_materialize_list", &[("items", &range)]);
        let body = template("python_return", &[("expression", &list)]);
        drafts.push(structural_draft(
            spec,
            "range_inclusive",
            &body,
            Vec::<String>::new(),
            concepts,
        ));
    }
    drafts
}

fn structural_program_draft(composition: &str, source: String) -> Draft {
    Draft {
        id: format!("structure:{composition}"),
        source,
        callable_name: String::new(),
        source_urls: structural_source_urls(composition),
        source_licenses: structural_source_urls(composition)
            .iter()
            .map(|_| "PSF-2.0".to_owned())
            .collect(),
        composition: composition.to_owned(),
        action_cost: composition.matches(['(', ',']).count() + 2,
    }
}

fn structural_draft(
    spec: &CodingTaskSpec,
    composition: &str,
    body: &str,
    imports: impl IntoIterator<Item = String>,
    _concepts: &ConceptMap,
) -> Draft {
    let source_urls = structural_source_urls(composition);
    let source_licenses = source_urls.iter().map(|_| "PSF-2.0".to_owned()).collect();
    Draft {
        id: format!("structure:{composition}"),
        source: render_function(spec, body, imports),
        callable_name: spec.name.clone(),
        source_urls,
        source_licenses,
        composition: composition.to_owned(),
        action_cost: composition.matches(['(', ',']).count() + 2,
    }
}

fn structural_source_urls(composition: &str) -> Vec<String> {
    let composition_ids = composition
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .collect::<std::collections::BTreeSet<_>>();
    structural_meanings()
        .into_iter()
        .filter(|structure| {
            !structure.grounding.is_empty() && composition_ids.contains(structure.id.as_str())
        })
        .map(|structure| structure.grounding)
        .collect()
}

fn adapt_implementation(code: &str, spec: &CodingTaskSpec) -> Option<String> {
    let trimmed = code.trim();
    if trimmed.strip_prefix("def ").is_none() {
        return Some(trimmed.to_owned());
    }
    let first_line = trimmed.lines().next()?;
    let open = first_line.find('(')?;
    let close = first_line.rfind(')')?;
    let original = first_line[open + 1..close]
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>();
    if original.len() != spec.parameters.len() {
        return None;
    }
    let body = trimmed.lines().skip(1).collect::<Vec<_>>();
    let indentation = body
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut body = body
        .iter()
        .map(|line| line.get(indentation..).unwrap_or(line).to_owned())
        .collect::<Vec<_>>()
        .join("\n");
    for (from, to) in original.iter().zip(&spec.parameters) {
        body = body.replace(from, &to.name);
    }
    Some(body)
}

fn verify(
    spec: &CodingTaskSpec,
    draft: &Draft,
    examples: &[Example],
    concepts: &ConceptMap,
) -> (bool, String) {
    if spec.artifact_shape == ArtifactShape::Function && examples.is_empty() {
        return (false, "no executable examples or derived tests".to_owned());
    }
    let mut script = draft.source.clone();
    if spec.artifact_shape == ArtifactShape::Function {
        script.push_str(&template("python_test_entrypoint", &[]));
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
    workspace.create_file("solution.py", &script);
    workspace.run_command("python3 solution.py");
    let run = workspace.finish();
    let command_succeeded = run.status == AgentRunStatus::Completed
        && run
            .command_results
            .iter()
            .any(|result| result.status_code == Some(0) && !result.timed_out);
    let expected_stdout = program_stdout_expectation(spec, concepts);
    let output_matches = if spec.artifact_shape == ArtifactShape::Program {
        expected_stdout.is_some_and(|expected| {
            run.command_results
                .last()
                .is_some_and(|result| result.stdout.trim_end() == expected)
        })
    } else {
        true
    };
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

fn program_stdout_expectation(spec: &CodingTaskSpec, concepts: &ConceptMap) -> Option<String> {
    if let Some(expected) = &spec.expected_stdout {
        return Some(expected.clone());
    }
    if concepts
        .structure_ids()
        .iter()
        .any(|structure| structure == "range_inclusive")
    {
        let bound = first_positive_integer(&spec.requirement_sentences)?
            .parse::<usize>()
            .ok()?;
        return Some(
            (1..=bound)
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    None
}

fn derived_examples(spec: &CodingTaskSpec, concepts: &ConceptMap) -> Vec<Example> {
    let structures = concepts.structure_ids();
    let has = |id: &str| structures.iter().any(|structure| structure == id);
    let example = |arguments: &[&str], expected: &str| Example {
        arguments: arguments.iter().map(|value| (*value).to_owned()).collect(),
        expected: expected.to_owned(),
    };
    if has("tuple_of") && has("reduce_sum") && has("reduce_product") {
        return vec![
            example(&["[]"], "(0, 1)"),
            example(&["[1, 2, 3]"], "(6, 6)"),
        ];
    }
    if has("quantifier_any") && has("pairwise_distinct") && has("predicate_abs_diff_lt") {
        return vec![
            example(&["[1.0, 2.0, 3.0]", "0.5"], "False"),
            example(&["[1.0, 2.8, 3.0]", "0.3"], "True"),
        ];
    }
    if has("reduce_count") && has("vowel_character_class") {
        return vec![example(&["'hello'"], "2"), example(&["'sky'"], "0")];
    }
    if has("set_intersection") {
        return vec![
            example(&["(3, 4, 5, 6)", "(5, 7, 4, 10)"], "(4, 5)"),
            example(&["(1, 2)", "(3, 4)"], "()"),
        ];
    }
    if has("distinct_elements") && (has("reduce_count") || has("reduce_len")) {
        return vec![example(&["[1, 1, 2, 3]"], "3"), example(&["[]"], "0")];
    }
    if has("range_inclusive") {
        if spec.parameters.is_empty() {
            if let Some(bound) = first_positive_integer(&spec.requirement_sentences)
                && let Ok(number) = bound.parse::<usize>()
            {
                let expected = format!(
                    "[{}]",
                    (1..=number)
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return vec![Example {
                    arguments: Vec::new(),
                    expected,
                }];
            }
        } else {
            return vec![example(&["3"], "[1, 2, 3]"), example(&["1"], "[1]")];
        }
    }
    Vec::new()
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

fn render_comparison(attempts: &[DraftAttempt]) -> String {
    attempts
        .iter()
        .map(|attempt| {
            format!(
                "{}:{}",
                attempt.id,
                if attempt.passed { "passed" } else { "failed" }
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn render_failure_trail(
    spec: &CodingTaskSpec,
    concepts: &ConceptMap,
    examples: &[Example],
    attempts: &[DraftAttempt],
) -> String {
    let phrases = concepts
        .needs
        .iter()
        .map(|need| need.phrase.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    let parts = concepts.candidate_ids().join(", ");
    let failed_examples = examples
        .iter()
        .map(|example| {
            format!(
                "{}({}) == {}",
                spec.name,
                example.arguments.join(", "),
                example.expected
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "phrases={phrases};parts={parts};failed_examples={failed_examples};attempts={}",
        render_comparison(attempts)
    )
}

fn template(id: &str, values: &[(&str, &str)]) -> String {
    runtime_template(id, values).unwrap_or_else(|| panic!("missing runtime template {id}"))
}

pub(super) fn idiom(id: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = structural_meanings()
        .into_iter()
        .find(|meaning| meaning.id == id)
        .unwrap_or_else(|| panic!("missing structural meaning {id}"))
        .idiom;
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}
