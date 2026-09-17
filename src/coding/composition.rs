//! Draft construction, bounded execution, and least-action selection.

use std::fmt::Write as _;
use std::time::Duration;

use crate::agent::{AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig};
use crate::coding::concept_discovery::{CandidatePart, ConceptMap};
use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::program_ir::{IrNode, IrType, ProgramIr};
use crate::coding::python_render::render_function;
use crate::coding::task_spec::{ArtifactShape, CodingTaskSpec, Example};
use crate::needs::{Need, NeedKind, NeedState};

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

/// A typed, lowered candidate for which no executable semantic oracle was
/// available. This must never be rendered as "tests passed".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnverifiedDraft {
    pub id: String,
    pub source: String,
    pub source_urls: Vec<String>,
    pub source_licenses: Vec<String>,
    pub composition: String,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompositionOutcome {
    pub selected: Option<VerifiedDraft>,
    pub unverified: Vec<UnverifiedDraft>,
    pub attempts: Vec<DraftAttempt>,
    /// Source-backed pieces which were requested but unavailable. A blocked
    /// part is data for rediscovery, never an empty-string program fragment.
    pub blocked_needs: Vec<Need>,
    pub research_trail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MissingFragment {
    id: String,
}

impl MissingFragment {
    fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }
}

/// Catalog-bound rendering without a closure-captured argument lifetime.
///
/// Each method call borrows its replacement slice independently. This matters
/// for composition, where most calls pass temporary arrays containing local
/// strings; a local closure would infer one shared lifetime for all calls and
/// incorrectly keep the first temporary borrowed until the closure's last use.
#[derive(Clone, Copy)]
pub(super) struct FragmentRenderer<'a> {
    catalog: &'a FragmentCatalog,
}

impl<'a> FragmentRenderer<'a> {
    pub(super) const fn new(catalog: &'a FragmentCatalog) -> Self {
        Self { catalog }
    }

    pub(super) fn template(
        self,
        id: &str,
        values: &[(&str, &str)],
    ) -> Result<String, MissingFragment> {
        template(self.catalog, id, values)
    }
}

pub(super) struct Draft {
    pub(super) id: String,
    pub(super) source: String,
    pub(super) callable_name: String,
    pub(super) source_urls: Vec<String>,
    pub(super) source_licenses: Vec<String>,
    pub(super) composition: String,
    pub(super) action_cost: usize,
    pub(super) typed_ir: bool,
}

#[must_use]
pub fn compose(spec: &CodingTaskSpec, concepts: &ConceptMap) -> CompositionOutcome {
    let catalog = FragmentCatalog::bootstrap();
    let programs = crate::coding::composition_search::search_with_structures(
        spec,
        &catalog,
        crate::coding::composition_search::SearchBounds::default(),
        &concepts.structure_ids(),
    );
    compose_with_ir(spec, concepts, &catalog, programs)
}

/// Compose ordinary retrieved parts together with already-elaborated IR.
///
/// Procedure retrieval calls this entry point so its ordered steps enter the
/// same verification and least-action selection path as every other source.
#[must_use]
pub fn compose_with_ir(
    spec: &CodingTaskSpec,
    concepts: &ConceptMap,
    catalog: &FragmentCatalog,
    programs: impl IntoIterator<Item = ProgramIr>,
) -> CompositionOutcome {
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
    let mut drafts = Vec::new();
    let mut blocked_needs = Vec::new();
    for structure_id in concepts.structure_ids() {
        let available = catalog.get(&structure_id).is_some()
            || catalog.fragments().iter().any(|fragment| {
                fragment
                    .supports
                    .iter()
                    .any(|supported| supported == &structure_id)
            });
        if !available {
            blocked_needs.push(blocked_fragment_need(
                spec,
                &MissingFragment::new(&structure_id),
            ));
        }
    }
    for program in programs {
        match ir_draft(spec, catalog, program) {
            Ok(Some(draft)) => drafts.push(draft),
            Ok(None) => {}
            Err(missing) => blocked_needs.push(blocked_fragment_need(spec, &missing)),
        }
    }
    if spec.language == "python" && spec.artifact_shape == ArtifactShape::Function {
        match candidate_drafts(spec, concepts, catalog) {
            Ok(found) => drafts.extend(found),
            Err(missing) => blocked_needs.push(blocked_fragment_need(spec, &missing)),
        }
    }
    if spec.language == "python"
        && spec.artifact_shape == ArtifactShape::Function
        && !examples.is_empty()
        && !drafts.is_empty()
        && let Err(missing) = template(catalog, "python_test_entrypoint", &[])
    {
        blocked_needs.push(blocked_fragment_need(spec, &missing));
        drafts.clear();
    }
    blocked_needs.sort_by(|left, right| left.need_id.cmp(&right.need_id));
    blocked_needs.dedup_by(|left, right| left.need_id == right.need_id);
    let mut attempts = Vec::new();
    let mut passing = Vec::new();
    let mut unverified = Vec::new();
    for draft in drafts {
        if !syntax_is_valid(&spec.language, &draft.source) {
            attempts.push(DraftAttempt {
                id: draft.id,
                passed: false,
                detail: crate::coding::python_render::runtime_template(
                    "coding_invalid_cst",
                    &[("language", &spec.language)],
                )
                .unwrap_or_else(|| String::from("coding_invalid_cst")),
            });
            continue;
        }
        if draft.typed_ir && spec.artifact_shape == ArtifactShape::Function && examples.is_empty() {
            let detail = "unverified:no_executable_oracle;typed_and_lowered".to_owned();
            attempts.push(DraftAttempt {
                id: draft.id.clone(),
                passed: false,
                detail: detail.clone(),
            });
            unverified.push((draft, detail));
            continue;
        }
        let (passed, detail) = verify(spec, &draft, &examples, concepts, catalog);
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
    unverified.sort_by(|(left, _), (right, _)| {
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
        render_failure_trail(spec, concepts, &examples, &attempts, &blocked_needs)
    };
    CompositionOutcome {
        selected,
        unverified: unverified
            .into_iter()
            .map(|(draft, detail)| UnverifiedDraft {
                id: draft.id,
                source: draft.source,
                source_urls: draft.source_urls,
                source_licenses: draft.source_licenses,
                composition: draft.composition,
                detail,
            })
            .collect(),
        attempts,
        blocked_needs,
        research_trail,
    }
}

/// Whether the top-level fragment's driving iteration is bounded by a
/// constant: its sequence-typed slot is filled without reading any of the
/// program's parameters. Returns 1 in that case, 0 otherwise.
fn constant_bounded_domain(program: &ProgramIr, catalog: &FragmentCatalog) -> usize {
    let mut current = &program.body;
    while let IrNode::Return { value } | IrNode::Emit { value } = current {
        current = value;
    }
    let IrNode::Apply { fragment, arguments } = current else {
        return 0;
    };
    let Some(definition) = catalog.get(fragment) else {
        return 0;
    };
    let Some(slot) = definition
        .signature
        .iter()
        .position(|ty| matches!(ty, IrType::Sequence(_)))
    else {
        return 0;
    };
    let Some(domain) = arguments.get(slot) else {
        return 0;
    };
    let input_names = program
        .parameters
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    let reads_input = |node: &IrNode| -> bool {
        let mut stack = vec![node];
        while let Some(current) = stack.pop() {
            match current {
                IrNode::Parameter { name, .. } => {
                    if input_names.contains(&name.as_str()) {
                        return true;
                    }
                }
                IrNode::Apply { arguments, .. } => stack.extend(arguments),
                IrNode::Literal { .. } => {}
                _ => return false,
            }
        }
        false
    };
    usize::from(!reads_input(domain))
}

fn ir_draft(
    spec: &CodingTaskSpec,
    catalog: &FragmentCatalog,
    program: ProgramIr,
) -> Result<Option<Draft>, MissingFragment> {
    if let Some(id) = program
        .fragments
        .iter()
        .find(|fragment| catalog.get(fragment).is_none())
    {
        return Err(MissingFragment::new(id));
    }
    if program.type_check(catalog).is_err() {
        return Ok(None);
    }
    let Some(lowering) = crate::coding::ir_lowering::lowering_for(&spec.language) else {
        return Ok(None);
    };
    let Ok(source) = lowering.lower(&program, catalog) else {
        return Ok(None);
    };
    let content_id = program.content_id();
    let mut action_cost = program.action_cost();
    // A draft whose driving iteration runs over a constant performs
    // input-independent action: its work is bounded by the constant, not by
    // the task's input, so agreement with the examples is coincidence. Such
    // a draft owes the unexamined input as extra action.
    action_cost += 4 * constant_bounded_domain(&program, catalog);
    Ok(Some(Draft {
        id: content_id.clone(),
        source,
        callable_name: program.name,
        source_urls: program.source_urls,
        source_licenses: program.source_licenses,
        composition: format!("typed_search({content_id})"),
        action_cost,
        typed_ir: true,
    }))
}

#[cfg(feature = "meta-language")]
fn syntax_is_valid(language: &str, source: &str) -> bool {
    crate::coding::validated_program_cst(language, source).is_some()
}

#[cfg(not(feature = "meta-language"))]
fn syntax_is_valid(_language: &str, _source: &str) -> bool {
    true
}

fn candidate_drafts(
    spec: &CodingTaskSpec,
    concepts: &ConceptMap,
    catalog: &FragmentCatalog,
) -> Result<Vec<Draft>, MissingFragment> {
    concepts
        .needs
        .iter()
        .flat_map(|need| need.candidates.iter())
        .map(|candidate| candidate_draft(spec, candidate, catalog))
        .filter_map(Result::transpose)
        .collect()
}

fn candidate_draft(
    spec: &CodingTaskSpec,
    candidate: &CandidatePart,
    catalog: &FragmentCatalog,
) -> Result<Option<Draft>, MissingFragment> {
    let renderer = FragmentRenderer::new(catalog);
    Ok(match candidate.kind.as_str() {
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
                .map(|(module, _)| renderer.template("python_import", &[("module", module)]))
                .transpose()?;
            let call = format!("{}({arguments})", candidate.id);
            let body = renderer.template("python_return", &[("expression", &call)])?;
            Some(Draft {
                id: format!("stdlib:{}", candidate.id),
                source: render_function(spec, &body, imports),
                callable_name: spec.name.clone(),
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: format!("direct_stdlib({})", candidate.id),
                action_cost: 1,
                typed_ir: false,
            })
        }
        "wikifunctions_implementation" => {
            let Some(code) = candidate.code.as_deref() else {
                return Ok(None);
            };
            let Some(body) = adapt_implementation(code, spec) else {
                return Ok(None);
            };
            Some(Draft {
                id: format!("part:{}", candidate.id),
                source: render_function(spec, &body, Vec::new()),
                callable_name: spec.name.clone(),
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: format!("direct_wrap({})", candidate.id),
                action_cost: 2,
                typed_ir: false,
            })
        }
        "wikifunctions_recurrence" => match (&candidate.code, &candidate.callable_name) {
            (Some(source), Some(callable_name)) => Some(Draft {
                id: format!("recurrence:{}", candidate.id),
                source: source.clone(),
                callable_name: callable_name.clone(),
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: format!("source_recurrence({})", candidate.id),
                action_cost: 3,
                typed_ir: false,
            }),
            _ => None,
        },
        "source_program" => match (&candidate.code, &candidate.callable_name) {
            (Some(source), Some(callable_name)) => Some(Draft {
                id: format!("source:{}", candidate.id),
                source: source.clone(),
                callable_name: callable_name.clone(),
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: candidate.label.clone(),
                action_cost: 4,
                typed_ir: false,
            }),
            _ => None,
        },
        _ => None,
    })
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
    catalog: &FragmentCatalog,
) -> (bool, String) {
    if spec.language == "python"
        && spec.artifact_shape == ArtifactShape::Function
        && examples.is_empty()
    {
        return (false, "no executable examples or derived tests".to_owned());
    }
    let mut script = draft.source.clone();
    if spec.language == "python" && spec.artifact_shape == ArtifactShape::Function {
        let Ok(entrypoint) = template(catalog, "python_test_entrypoint", &[]) else {
            return (false, "missing fragment:python_test_entrypoint".to_owned());
        };
        script.push_str(&entrypoint);
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
    blocked_needs: &[Need],
) -> String {
    let phrases = concepts
        .needs
        .iter()
        .map(super::concept_discovery::ConceptRequirement::phrase)
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
    let blocked = blocked_needs
        .iter()
        .map(|need| need.subject.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "phrases={phrases};parts={parts};failed_examples={failed_examples};blocked_needs={blocked};attempts={}",
        render_comparison(attempts)
    )
}

fn blocked_fragment_need(spec: &CodingTaskSpec, missing: &MissingFragment) -> Need {
    let subject = format!("fragment:{}", missing.id);
    let raised_by = format!("composition:{}", spec.name);
    let mut need = Need::raised(NeedKind::Part, &subject, &spec.prose_language, &raised_by);
    need.state = NeedState::Unsatisfiable;
    need
}

pub(super) fn template(
    catalog: &FragmentCatalog,
    id: &str,
    values: &[(&str, &str)],
) -> Result<String, MissingFragment> {
    let from_catalog = catalog
        .render_named(id, "python", values)
        .filter(|rendered| !rendered.is_empty());
    // Renderer scaffolds (`python_import`, `python_test_entrypoint`, …) are
    // deliberately not catalogued as operations; they render from the runtime
    // seed, so a catalog miss falls back there before a need is raised.
    let rendered = from_catalog
        .or_else(|| crate::coding::python_render::runtime_template(id, values))
        .filter(|rendered| !rendered.is_empty());
    rendered.ok_or_else(|| MissingFragment::new(id))
}
