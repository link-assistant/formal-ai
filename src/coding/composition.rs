//! Draft construction, bounded execution, and least-action selection.

use std::fmt::Write as _;
use std::time::Duration;

use crate::agent::{AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig};
use crate::coding::concept_discovery::{CandidatePart, ConceptMap, structural_meanings};
use crate::coding::python_render::{render_function, runtime_template};
use crate::coding::task_spec::{CodingTaskSpec, Example};

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
    pub composition: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompositionOutcome {
    pub selected: Option<VerifiedDraft>,
    pub attempts: Vec<DraftAttempt>,
    pub research_trail: String,
}

struct Draft {
    id: String,
    source: String,
    source_urls: Vec<String>,
    composition: String,
    action_cost: usize,
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
        derived_examples(spec, concepts)
    } else {
        spec.examples.clone()
    };
    let mut drafts = candidate_drafts(spec, concepts);
    drafts.extend(structural_drafts(spec, concepts));
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
        let (passed, detail) = verify(spec, &draft, &examples);
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
        assertion_count: examples.len(),
        source_urls: draft.source_urls,
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
                source_urls: vec![candidate.source_url.clone()],
                composition: format!("direct_stdlib({})", candidate.id),
                action_cost: 1,
            })
        }
        "wikifunctions_implementation" => {
            let body = adapt_implementation(candidate.code.as_deref()?, spec)?;
            Some(Draft {
                id: format!("part:{}", candidate.id),
                source: render_function(spec, &body, Vec::new()),
                source_urls: vec![candidate.source_url.clone()],
                composition: format!("direct_wrap({})", candidate.id),
                action_cost: 2,
            })
        }
        _ => None,
    }
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
        let distinct = idiom("distinct_elements", &[("items", names[0])]);
        let expression = idiom("reduce_len", &[("items", &distinct)]);
        let body = template("python_return", &[("expression", &expression)]);
        drafts.push(structural_draft(
            spec,
            "reduce_len(distinct_elements)",
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

fn structural_draft(
    spec: &CodingTaskSpec,
    composition: &str,
    body: &str,
    imports: impl IntoIterator<Item = String>,
    _concepts: &ConceptMap,
) -> Draft {
    let composition_ids = composition
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .collect::<std::collections::BTreeSet<_>>();
    let source_urls = structural_meanings()
        .into_iter()
        .filter(|structure| {
            !structure.grounding.is_empty() && composition_ids.contains(structure.id.as_str())
        })
        .map(|structure| structure.grounding)
        .collect();
    Draft {
        id: format!("structure:{composition}"),
        source: render_function(spec, body, imports),
        source_urls,
        composition: composition.to_owned(),
        action_cost: composition.matches(['(', ',']).count() + 2,
    }
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

fn verify(spec: &CodingTaskSpec, draft: &Draft, examples: &[Example]) -> (bool, String) {
    if examples.is_empty() {
        return (false, "no executable examples or derived tests".to_owned());
    }
    let mut script = draft.source.clone();
    script.push_str(&template("python_test_entrypoint", &[]));
    for example in examples {
        let _ = writeln!(
            script,
            "    assert {}({}) == {}",
            spec.name,
            example.arguments.join(", "),
            example.expected
        );
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
    let passed = run.status == AgentRunStatus::Completed
        && run
            .command_results
            .iter()
            .any(|result| result.status_code == Some(0) && !result.timed_out);
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
    sentences
        .iter()
        .flat_map(|sentence| sentence.split(|character: char| !character.is_ascii_digit()))
        .find(|token| token.parse::<usize>().is_ok_and(|number| number > 0))
        .map(str::to_owned)
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

fn idiom(id: &str, values: &[(&str, &str)]) -> String {
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
