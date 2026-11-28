//! Draft construction, bounded execution, and least-action selection.

mod verification;

use crate::coding::concept_discovery::{CandidatePart, ConceptMap};
use crate::coding::fragment_catalog::{FragmentCatalog, FragmentKind};
use crate::coding::program_ir::{IrNode, IrType, ProgramIr};
use crate::coding::python_render::render_function;
use crate::coding::python_signature::{matching_close_paren, split_top_level_commas};
use crate::coding::task_spec::{ArtifactShape, CodingTaskSpec, Example};
use crate::needs::{Need, NeedKind, NeedState};
use verification::{derived_examples, verify};

mod ir_draft;
use ir_draft::ir_draft;

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
    /// Position in the search's own ordering. Two mirror-image programs (a
    /// commutative fragment's operand orders) tie on action cost and source
    /// length, and their content-hash ids would then decide by hash luck;
    /// the search's ranking already prefers declared parameter order, so it
    /// is the canonical tie-break. Non-search drafts sit after every ranked
    /// one.
    pub(super) search_rank: usize,
    /// Distinct declared parameters the draft reads. A verified draft that
    /// skips a task input agrees with the examples only by coincidence --
    /// `abs(0 - threshold) < abs(1 - threshold)` passes a two-example check
    /// while never reading `numbers` (issue #317) -- so more inputs read
    /// outranks least action among drafts that verify.
    pub(super) referenced_params: usize,
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
    crate::diagnostic!("trace"; "coding_composition: language={} task={} needs={} programs={}", spec.language, spec.name, concepts.needs.len(), programs.len());
    compose_with_ir(spec, concepts, &catalog, programs)
}

/// Searches typed callable candidates once the runtime can verify them. Source
/// acquisition can add examples and retry this same composition step.
pub(super) fn compose_for_verification(
    spec: &CodingTaskSpec,
    concepts: &ConceptMap,
) -> CompositionOutcome {
    if spec.language == "python"
        && spec.artifact_shape == ArtifactShape::Function
        && spec.examples.is_empty()
        && !concepts.needs.iter().any(|need| {
            need.candidates
                .iter()
                .any(|candidate| !candidate.source_tests.is_empty())
        })
        && derived_examples(spec, concepts).is_empty()
    {
        return compose_with_ir(spec, concepts, &FragmentCatalog::bootstrap(), []);
    }
    compose(spec, concepts)
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
    let derived = if spec.examples.is_empty() {
        derived_examples(spec, concepts)
    } else {
        Vec::new()
    };
    let examples = if spec.examples.is_empty() {
        if derived.is_empty() {
            source_examples(concepts)
        } else {
            derived.clone()
        }
    } else {
        spec.examples.clone()
    };
    let mut drafts = Vec::new();
    let mut attempts = Vec::new();
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
    for (search_rank, program) in programs.into_iter().enumerate() {
        match ir_draft(spec, catalog, program, search_rank, &mut attempts) {
            Ok(Some(draft)) => {
                drafts.push(draft);
            }
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
    // Issue #1487: examples derived from the requirement share its reading, so
    // a negation, count, disjunction or comparison no discovered part reads
    // cannot be checked by them. It stays an unresolved need and no draft is
    // claimed; examples the request itself states, or a source documents,
    // remain an independent oracle.
    let unread = if derived.is_empty() {
        String::new()
    } else {
        crate::coding::unread_structure::unread_structure(
            &spec.requirement_sentences.join(" "),
            &concepts.structure_ids(),
        )
    };
    if !unread.is_empty() {
        let raised_by = format!("composition:{}", spec.name);
        let subject = format!("structure:{unread}");
        blocked_needs.push(Need::raised(
            NeedKind::Concept,
            &subject,
            &spec.prose_language,
            &raised_by,
        ));
        drafts.clear();
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
        right
            .referenced_params
            .cmp(&left.referenced_params)
            .then_with(|| left.action_cost.cmp(&right.action_cost))
            .then_with(|| left.source.len().cmp(&right.source.len()))
            .then_with(|| left.search_rank.cmp(&right.search_rank))
            .then_with(|| left.id.cmp(&right.id))
    });
    unverified.sort_by(|(left, _), (right, _)| {
        right
            .referenced_params
            .cmp(&left.referenced_params)
            .then_with(|| left.action_cost.cmp(&right.action_cost))
            .then_with(|| left.source.len().cmp(&right.source.len()))
            .then_with(|| left.search_rank.cmp(&right.search_rank))
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
    let IrNode::Apply {
        fragment,
        arguments,
    } = current
    else {
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

#[cfg(feature = "meta-language")]
fn syntax_is_valid(language: &str, source: &str) -> bool {
    crate::coding::validated_program_cst(language, source).is_some()
}

#[cfg(not(feature = "meta-language"))]
fn syntax_is_valid(_language: &str, _source: &str) -> bool {
    true
}

/// Echo the prompt's declared signature into a lowered artifact. The search
/// unifies types to find the composition; the surface the task declared —
/// parameter spellings, the return annotation, the import block — stays the
/// task's own words, ahead of anything the lowering hoisted.
fn echo_declared_surface(spec: &CodingTaskSpec, source: String) -> String {
    let declared_signature = spec
        .parameters
        .iter()
        .any(|parameter| parameter.annotation.is_some())
        || spec.return_annotation.is_some();
    if !declared_signature && spec.imports.is_empty() {
        return source;
    }
    let had_trailing_newline = source.ends_with('\n');
    let is_import_line = |line: &str| line.starts_with("import ") || line.starts_with("from ");
    let mut rest = source.lines().peekable();
    let mut imports: Vec<&str> = spec.imports.iter().map(String::as_str).collect();
    while let Some(line) = rest.peek().copied() {
        if !is_import_line(line) {
            break;
        }
        if !imports.contains(&line) {
            imports.push(line);
        }
        rest.next();
    }
    let mut lines: Vec<String> = imports.iter().map(|line| (*line).to_owned()).collect();
    if !lines.is_empty() {
        // Imports and the definition are separate paragraphs (PEP 8).
        if rest.peek().is_some_and(|line| line.trim().is_empty()) {
            rest.next();
        }
        lines.push(String::new());
    }
    lines.extend(rest.map(str::to_owned));
    if declared_signature
        && let Some(index) = lines.iter().position(|line| line.starts_with("def "))
        && let Some(rewritten) = echo_signature_line(spec, &lines[index])
    {
        lines[index] = rewritten;
    }
    let mut joined = lines.join("\n");
    if had_trailing_newline {
        joined.push('\n');
    }
    joined
}

/// One `def` line with the declared spellings restored: parameters keep
/// their declared annotation verbatim (open ones keep what the lowering
/// rendered), and the declared return travels before the colon.
fn echo_signature_line(spec: &CodingTaskSpec, line: &str) -> Option<String> {
    let open = line.find('(')?;
    // `matching_close_paren` returns the index one past the closing paren.
    let close = matching_close_paren(line, open)?;
    if line[close..].trim() != ":" {
        return None;
    }
    let rendered = split_top_level_commas(&line[open + 1..close - 1])
        .iter()
        .map(|raw| {
            let name = raw.split(':').next().unwrap_or(raw).trim();
            spec.parameters
                .iter()
                .find(|parameter| parameter.name == name)
                .and_then(|parameter| parameter.annotation.as_deref())
                .map_or_else(
                    || (*raw).to_owned(),
                    |annotation| format!("{}: {annotation}", name.trim_start_matches('*')),
                )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let return_annotation = spec
        .return_annotation
        .as_deref()
        .map_or_else(String::new, |annotation| format!(" -> {annotation}"));
    Some(format!("{}({rendered}){return_annotation}:", &line[..open]))
}

/// Distinct declared parameters a rendered draft's source names. Drafts
/// assembled from retrieved parts have no IR to walk, so their identifiers
/// are read from the rendered source with the same lexical scan the search
/// uses for scope.
fn source_referenced_parameters(spec: &CodingTaskSpec, source: &str) -> usize {
    let tokens = crate::coding::composition_search::python_tokens(source);
    spec.parameters
        .iter()
        .filter(|parameter| tokens.iter().any(|token| token == &parameter.name))
        .count()
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
            let source = render_function(spec, &body, imports);
            Some(Draft {
                id: format!("stdlib:{}", candidate.id),
                source: source.clone(),
                callable_name: spec.name.clone(),
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: format!("direct_stdlib({})", candidate.id),
                action_cost: 1,
                typed_ir: false,
                search_rank: usize::MAX,
                referenced_params: source_referenced_parameters(spec, &source),
            })
        }
        "wikifunctions_implementation" => {
            let Some(code) = candidate.code.as_deref() else {
                return Ok(None);
            };
            let Some(body) = adapt_implementation(code, spec) else {
                return Ok(None);
            };
            let source = render_function(spec, &body, Vec::new());
            Some(Draft {
                id: format!("part:{}", candidate.id),
                source: source.clone(),
                callable_name: spec.name.clone(),
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: format!("direct_wrap({})", candidate.id),
                action_cost: 2,
                typed_ir: false,
                search_rank: usize::MAX,
                referenced_params: source_referenced_parameters(spec, &source),
            })
        }
        "wikifunctions_recurrence" => {
            let (Some(source), Some(callable_name), Some(contract)) = (
                &candidate.code,
                &candidate.callable_name,
                &candidate.callable_contract,
            ) else {
                return Ok(None);
            };
            if contract.name != *callable_name || contract.render_python() != *source {
                return Ok(None);
            }
            let Some(bound) = contract.bound_to(spec) else {
                return Ok(None);
            };
            let Some(read_positions) = bound.read_positions() else {
                return Ok(None);
            };
            Some(Draft {
                id: format!("recurrence:{}", candidate.id),
                source: echo_declared_surface(spec, bound.render_python()),
                callable_name: bound.name,
                source_urls: vec![candidate.source_url.clone()],
                source_licenses: vec![candidate.license.clone()],
                composition: format!("source_recurrence({})", candidate.id),
                action_cost: 3,
                typed_ir: false,
                search_rank: usize::MAX,
                referenced_params: read_positions.len(),
            })
        }
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
                search_rank: usize::MAX,
                referenced_params: source_referenced_parameters(spec, source),
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
