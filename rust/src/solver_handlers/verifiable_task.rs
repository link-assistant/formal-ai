//! The generic recognise → derive → execute → verify route (#1138, plan 08).
//!
//! Task vocabulary and operation cues live in the meaning lexicon. This module
//! only interprets those roles, builds the shared [`ProgramIr`], executes the
//! lowered program in the bounded workspace, and reports observed checks.

use std::time::Duration;

use crate::agent::{AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig};
use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::ir_lowering::lowering_for;
use crate::coding::program_ir::{IrNode, IrType, ProgramIr, ReuseMode};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};
use crate::verifiable_task::{
    TaskExpectation, VerifiableTask, normalized_category, recognise_verifiable,
};

use super::{finalize_simple, try_pattern_inference};

const FLOW_INCREASE: &str = "verifiable_quantity_flow_increase";
const FLOW_DECREASE: &str = "verifiable_quantity_flow_decrease";
const QUANTITY_DETERMINER: &str = "verifiable_quantity_determiner";
const UNIT_CONVERSION: &str = "verifiable_unit_conversion";

/// The composed program's observed answer and its proof-bearing metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedAnswer {
    pub value: String,
    pub source: String,
    pub derivation_id: String,
    pub fragments: Vec<String>,
    pub source_urls: Vec<String>,
    pub source_licenses: Vec<String>,
    pub checks: Vec<Evidence>,
}

/// Try the generic verifiable-task route with the process's live-fetch policy.
pub fn try_verifiable_task(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_verifiable_task_with_online(
        prompt,
        normalized,
        log,
        crate::coding::synthesis_runtime::live_fetch_enabled(),
    )
}

/// Try the generic route, allowing callers and tests to pin retrieval offline.
pub fn try_verifiable_task_with_online(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    online: bool,
) -> Option<SymbolicAnswer> {
    // Pattern inference keeps its established result and intent, but its only
    // registry entrance is now this generic interpreter. Its surface gate is a
    // seed role, not the retired Rust marker table.
    if let Some(answer) = try_pattern_inference(prompt, normalized, log) {
        return Some(answer);
    }

    let task = recognise_verifiable(prompt)?;
    if matches!(
        task.expectation,
        TaskExpectation::Callable { .. } | TaskExpectation::Stdout { .. }
    ) {
        return None;
    }
    log.append("verifiable_task:recognised", task.to_links_notation());
    let candidates = derive_candidates(&task, log, online);
    log.append("verifiable_task:candidates", candidates.len().to_string());
    let mut answers = execute_candidates(&task, &candidates, log);

    match classify_agreement(&answers) {
        AnswerAgreement::Independent => {
            for answer in &mut answers {
                answer.checks.push(symbolic_check(
                    &answer.derivation_id,
                    "agreement",
                    answer.value.as_bytes(),
                ));
            }
        }
        AnswerAgreement::Disagreement => {
            log.append(
                "verifiable_task:disagreement",
                answers
                    .iter()
                    .map(|answer| answer.value.trim())
                    .collect::<Vec<_>>()
                    .join("|"),
            );
            answers.clear();
        }
        AnswerAgreement::Insufficient | AnswerAgreement::Correlated => {}
    }

    for answer in &mut answers {
        append_round_trip_checks(&task, answer);
        append_unit_check(&task, answer);
    }

    let Some(answer) = answers
        .into_iter()
        .min_by_key(|answer| answer.derivation_id.clone())
    else {
        return Some(gap_answer(prompt, &task, log));
    };
    let corroborated = answer.checks.iter().any(|check| {
        check.produced_by.ends_with("agreement") || check.produced_by.ends_with("round_trip")
    });
    let reasoning_intent = if corroborated {
        "verifiable_task_verified"
    } else {
        "verifiable_task_uncorroborated"
    };
    let reasoning = crate::seed::localized_response(reasoning_intent, &task.prose_language)
        .or_else(|| crate::seed::localized_response(reasoning_intent, "en"))
        .unwrap_or_default();
    for check in &answer.checks {
        let slug = evidence_check_slug(check);
        log.append("verifiable_task:check", slug.to_owned());
    }
    log.append(
        "verifiable_task:executed",
        format!("{}:{}", answer.derivation_id, answer.value.trim()),
    );
    if !corroborated {
        log.append(
            "verifiable_task:uncorroborated",
            answer.derivation_id.clone(),
        );
    }
    let body = task.render(answer.value.trim(), &reasoning);
    let mut symbolic = finalize_simple(
        prompt,
        log,
        "verifiable_task",
        "response:verifiable_task",
        &body,
        if corroborated { 1.0 } else { 0.7 },
    );
    symbolic.evidence_links.push(format!(
        "verifiable_task:executed:{}:{}",
        answer.derivation_id,
        answer.value.trim()
    ));
    for check in &answer.checks {
        let slug = evidence_check_slug(check);
        symbolic
            .evidence_links
            .push(format!("verifiable_task:check:{slug}"));
    }
    if !corroborated {
        symbolic.evidence_links.push(format!(
            "verifiable_task:uncorroborated:{}",
            answer.derivation_id
        ));
    }
    Some(symbolic)
}

fn gap_answer(prompt: &str, task: &VerifiableTask, log: &mut EventLog) -> SymbolicAnswer {
    log.append("verifiable_task:gap", task.identity());
    log.append(
        "verifiable_task:searched",
        format!(
            "expectation={};language={}",
            task.expectation.slug(),
            task.prose_language
        ),
    );
    let body = crate::seed::localized_response("verifiable_task_gap", &task.prose_language)
        .or_else(|| crate::seed::localized_response("verifiable_task_gap", "en"))
        .unwrap_or_default();
    let mut answer = finalize_simple(
        prompt,
        log,
        "verifiable_task",
        "response:verifiable_task_gap",
        &body,
        0.0,
    );
    answer
        .evidence_links
        .push(format!("verifiable_task:gap:{}", task.identity()));
    answer.evidence_links.push(format!(
        "verifiable_task:searched:{}:{}",
        task.expectation.slug(),
        task.prose_language
    ));
    answer
}

fn derive_candidates(task: &VerifiableTask, log: &mut EventLog, online: bool) -> Vec<ProgramIr> {
    match &task.expectation {
        TaskExpectation::Unknown { name } => derive_unknown(task, name),
        TaskExpectation::Numeric { unit } => derive_numeric(task, unit.as_deref()),
        TaskExpectation::Count { subject } => derive_count(task, subject, log, online),
        TaskExpectation::EditedText { .. }
        | TaskExpectation::Callable { .. }
        | TaskExpectation::Stdout { .. }
        | TaskExpectation::Boolean => Vec::new(),
    }
}

fn derive_unknown(task: &VerifiableTask, name: &str) -> Vec<ProgramIr> {
    if name.is_empty() || task.quantities.len() < 2 || !task.prompt.contains('=') {
        return Vec::new();
    }
    let coefficient = task.quantities[0].value.parse::<i128>().ok();
    let target = task.quantities[1].value.parse::<i128>().ok();
    let (Some(coefficient), Some(target)) = (coefficient, target) else {
        return Vec::new();
    };
    if coefficient == 0 {
        return Vec::new();
    }
    let exact = target % coefficient == 0;
    let target = target.to_string();
    let coefficient = coefficient.to_string();
    let values = &[
        ("target", target.as_str()),
        ("coefficient", coefficient.as_str()),
    ];
    let expression = crate::coding::python_render::runtime_template(
        if exact {
            "verifiable_exact_division"
        } else {
            "verifiable_division"
        },
        values,
    )
    .unwrap_or_else(|| String::from("verifiable_division"));
    let alternate =
        crate::coding::python_render::runtime_template("verifiable_parenthesized_division", values)
            .unwrap_or_else(|| String::from("verifiable_parenthesized_division"));
    vec![
        expression_ir(
            expression,
            "linear_equation_inverse",
            source_for(FLOW_INCREASE),
        ),
        expression_ir(
            alternate,
            "linear_equation_round_trip",
            source_for(FLOW_DECREASE),
        ),
    ]
}

fn derive_numeric(task: &VerifiableTask, target_unit: Option<&str>) -> Vec<ProgramIr> {
    let determiners = role_offsets(&task.prompt, &task.prose_language, QUANTITY_DETERMINER);
    let quantities = task
        .quantities
        .iter()
        .filter(|quantity| {
            target_unit.is_none_or(|_| quantity.unit.is_some())
                && !determiners.contains(&quantity.offset)
        })
        .collect::<Vec<_>>();
    let Some(signs) = flow_signs(task, &quantities) else {
        return Vec::new();
    };
    let mut terms = Vec::new();
    for (quantity, sign) in quantities.into_iter().zip(signs) {
        let Some(scale) = conversion_scale(quantity.unit.as_deref(), target_unit) else {
            return Vec::new();
        };
        let Ok(value) = quantity.value.parse::<i128>() else {
            return Vec::new();
        };
        let Some(scaled) = value.checked_mul(scale.numerator) else {
            return Vec::new();
        };
        let Some(signed) = sign.checked_mul(scaled) else {
            return Vec::new();
        };
        if scale.denominator == 1 || signed % scale.denominator == 0 {
            terms.push((signed / scale.denominator).to_string());
        } else {
            terms.push(format!("({signed} / {})", scale.denominator));
        }
    }
    if terms.is_empty() {
        return Vec::new();
    }
    let direct = signed_expression(&terms);
    let folded = format!("sum([{}])", terms.join(", "));
    let sources = [source_for(FLOW_INCREASE), source_for(FLOW_DECREASE)]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    vec![
        expression_ir(direct, "quantity_flow_direct", sources.first().cloned()),
        expression_ir(folded, "quantity_flow_fold", sources.get(1).cloned()),
    ]
}

fn derive_count(
    task: &VerifiableTask,
    subject: &str,
    log: &mut EventLog,
    online: bool,
) -> Vec<ProgramIr> {
    let cache_root = crate::coding::synthesis_runtime::source_cache_root();
    let client = CachedSourceClient::new(&cache_root, CurlSourceTransport).with_online(online);
    let preferences = crate::how_to_guide::ServicePreferences::default();
    let mut availability =
        crate::service_accessibility::ServiceAccessibilityCache::load(&cache_root);
    let bounds = crate::source_walk::LookupBounds::default();
    let now = crate::service_accessibility::unix_now();
    let category = normalized_category(subject);
    let mut count = 0_i128;
    let mut urls = Vec::new();
    for entity in &task.entities {
        let outcome = crate::concept_lookup::lookup_surface(
            &entity.surface,
            &task.prose_language,
            &client,
            &preferences,
            &bounds,
            &mut availability,
            now,
        );
        for row in &outcome.outcomes {
            log.append(
                "verifiable_task:membership_source",
                format!("{}:{}:{}", entity.surface, row.source_id, row.status),
            );
        }
        let retrieved_match = outcome.items.iter().any(|sense| {
            let gloss = normalized_category(&sense.gloss);
            gloss.contains(&category)
                || category
                    .split_whitespace()
                    .all(|token| token.len() < 3 || gloss.contains(token))
        });
        let seeded_sources = (!retrieved_match)
            .then(|| {
                crate::verifiable_task::seeded_membership_sources(
                    &entity.surface,
                    subject,
                    &task.prose_language,
                )
            })
            .flatten();
        if retrieved_match || seeded_sources.is_some() {
            count += entity.multiplicity.parse::<i128>().unwrap_or(1);
            urls.extend(outcome.items.into_iter().map(|sense| sense.source_url));
            if let Some(sources) = seeded_sources {
                for source in &sources {
                    log.append(
                        "verifiable_task:membership_source",
                        format!("{}:seed:{source}", entity.surface),
                    );
                }
                urls.extend(sources);
            }
        }
    }
    urls.sort();
    urls.dedup();
    if count == 0 || urls.is_empty() {
        return Vec::new();
    }
    // Same trace contract as the gsm8k/math composition paths: the counted
    // category and the derived count are recorded as evidence so a membership
    // count is auditable as a composition step, not only as a membership
    // source list (issue #314: the object-counting trace prefixes).
    log.append("composition:category", category);
    log.append("composition:count", count.to_string());
    vec![expression_ir(
        count.to_string(),
        "retrieved_category_membership",
        urls.first().cloned(),
    )]
}

fn flow_signs(
    task: &VerifiableTask,
    quantities: &[&crate::verifiable_task::quantities::Quantity],
) -> Option<Vec<i128>> {
    let mut cues = flow_cues(&task.prompt, &task.prose_language, FLOW_INCREASE, 1);
    cues.extend(flow_cues(
        &task.prompt,
        &task.prose_language,
        FLOW_DECREASE,
        -1,
    ));
    cues.sort_unstable_by_key(|cue| cue.offset);
    if cues.is_empty() {
        return None;
    }
    quantities
        .iter()
        .map(|quantity| {
            cues.iter()
                .filter(|cue| match cue.argument_position.as_str() {
                    "quantity_before" => cue.offset <= quantity.offset,
                    "quantity_after" => cue.offset >= quantity.offset,
                    _ => true,
                })
                .min_by_key(|cue| cue.offset.abs_diff(quantity.offset))
                .map(|cue| cue.sign)
        })
        .collect()
}

struct FlowCue {
    offset: usize,
    sign: i128,
    argument_position: String,
}

fn flow_cues(prompt: &str, language: &str, role: &str, sign: i128) -> Vec<FlowCue> {
    let lowered = prompt.to_lowercase();
    crate::seed::lexicon()
        .meanings_with_role(role)
        .flat_map(|meaning| &meaning.lexemes)
        .filter(|lexeme| lexeme.language == language)
        .flat_map(|lexeme| &lexeme.words)
        .flat_map(|word| {
            let surface = word.text.to_lowercase();
            lowered
                .match_indices(&surface)
                .map(|(offset, _)| FlowCue {
                    offset,
                    sign,
                    argument_position: word.action.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn role_offsets(prompt: &str, language: &str, role: &str) -> Vec<usize> {
    let lowered = prompt.to_lowercase();
    let mut offsets = crate::seed::lexicon()
        .meanings_with_role(role)
        .flat_map(|meaning| &meaning.lexemes)
        .filter(|lexeme| lexeme.language == language)
        .flat_map(|lexeme| &lexeme.words)
        .flat_map(|word| {
            let surface = word.text.to_lowercase();
            lowered
                .match_indices(&surface)
                .map(|(offset, _)| offset)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

#[derive(Clone, Copy)]
struct UnitScale {
    numerator: i128,
    denominator: i128,
}

fn conversion_scale(source: Option<&str>, target: Option<&str>) -> Option<UnitScale> {
    let (Some(source), Some(target)) = (source, target) else {
        return Some(UnitScale {
            numerator: 1,
            denominator: 1,
        });
    };
    let source = unit_slug(source)?;
    let target = unit_slug(target)?;
    if source == target {
        return Some(UnitScale {
            numerator: 1,
            denominator: 1,
        });
    }
    crate::seed::lexicon()
        .meanings_with_role(UNIT_CONVERSION)
        .find_map(|conversion| {
            let units = conversion
                .defined_by
                .iter()
                .filter(|slug| {
                    crate::seed::lexicon()
                        .meaning(slug)
                        .is_some_and(|meaning| meaning.has_role(crate::seed::ROLE_MEASUREMENT_UNIT))
                })
                .collect::<Vec<_>>();
            let factor = conversion
                .words()
                .find_map(|word| word.parse::<i128>().ok())?;
            match units.as_slice() {
                [from, to] if ***from == source && ***to == target => Some(UnitScale {
                    numerator: factor,
                    denominator: 1,
                }),
                [from, to] if ***from == target && ***to == source && factor != 0 => {
                    Some(UnitScale {
                        numerator: 1,
                        denominator: factor,
                    })
                }
                _ => None,
            }
        })
}

fn unit_slug(surface: &str) -> Option<String> {
    crate::seed::lexicon()
        .meanings_with_role(crate::seed::ROLE_MEASUREMENT_UNIT)
        .find(|meaning| {
            meaning
                .words()
                .any(|word| word.eq_ignore_ascii_case(surface))
        })
        .map(|meaning| meaning.slug.clone())
}

fn source_for(role: &str) -> Option<String> {
    crate::seed::lexicon()
        .meanings_with_role(role)
        .find_map(|meaning| (!meaning.wikidata.is_empty()).then(|| meaning.wikidata.clone()))
}

fn signed_expression(terms: &[String]) -> String {
    terms.join(" + ")
}

fn expression_ir(expression: String, fragment: &str, source_url: Option<String>) -> ProgramIr {
    ProgramIr {
        name: String::from("main"),
        parameters: Vec::new(),
        result: IrType::Integer,
        body: IrNode::Emit {
            value: Box::new(IrNode::Literal {
                text: expression,
                ty: IrType::Integer,
            }),
        },
        fragments: vec![fragment.to_owned()],
        source_urls: source_url.into_iter().collect(),
        source_licenses: Vec::new(),
        reuse: ReuseMode::ShapeOnly,
    }
}

/// Lower and run each IR candidate. Stdout, not a separately calculated claim,
/// becomes the answer value.
pub fn execute_candidates(
    task: &VerifiableTask,
    candidates: &[ProgramIr],
    log: &mut EventLog,
) -> Vec<VerifiedAnswer> {
    let catalog = FragmentCatalog::default();
    let Some(lowering) = lowering_for("python") else {
        return Vec::new();
    };
    let mut ordered = candidates.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|candidate| (candidate.action_cost(), candidate.content_id()));
    let mut answers = Vec::new();
    for candidate in ordered {
        if candidate.type_check(&catalog).is_err() {
            continue;
        }
        let Ok(mut source) = lowering.lower(candidate, &catalog) else {
            continue;
        };
        source.push_str("\nmain()\n");
        let config = AgentWorkspaceConfig {
            time_budget: Duration::from_secs(5),
            ..AgentWorkspaceConfig::default()
        };
        let Ok(mut workspace) = AgentWorkspace::for_prompt(
            &format!(
                "verifiable-task:{}:{}",
                task.identity(),
                candidate.content_id()
            ),
            &config,
        ) else {
            continue;
        };
        workspace.create_file("solution.py", &source);
        workspace.run_command("python3 solution.py");
        let run = workspace.finish();
        let Some(result) = run.command_results.last() else {
            continue;
        };
        if run.status != AgentRunStatus::Completed
            || result.status_code != Some(0)
            || result.timed_out
        {
            continue;
        }
        let value = canonical_answer_value(task, result.stdout.trim());
        if !output_has_shape(task, &value) {
            continue;
        }
        let derivation_id = candidate.content_id();
        let mut ran = Evidence::observed(
            "python3 solution.py",
            vec![String::from("python3"), String::from("solution.py")],
            Some(0),
            result.stdout.as_bytes(),
            ObservationKind::CommandExit,
            EvidenceSource::LocalProcess,
        );
        ran.for_need = task.identity();
        ran.produced_by = String::from("verifiable_task:ran");
        ran.source_ids.clone_from(&candidate.source_urls);
        let shape = symbolic_check(&derivation_id, "shape", value.as_bytes());
        log.append("verifiable_task:run", ran.to_links_notation());
        answers.push(VerifiedAnswer {
            value,
            source,
            derivation_id,
            fragments: candidate.fragments.clone(),
            source_urls: candidate.source_urls.clone(),
            source_licenses: candidate.source_licenses.clone(),
            checks: vec![ran, shape],
        });
    }
    answers
}

fn canonical_answer_value(task: &VerifiableTask, observed: &str) -> String {
    if !matches!(
        task.expectation,
        TaskExpectation::Numeric { .. }
            | TaskExpectation::Count { .. }
            | TaskExpectation::Unknown { .. }
    ) {
        return observed.to_owned();
    }
    let Some((whole, fractional)) = observed.split_once('.') else {
        return observed.to_owned();
    };
    if !whole.is_empty()
        && fractional.chars().all(|character| character == '0')
        && whole.chars().enumerate().all(|(index, character)| {
            character.is_ascii_digit() || (index == 0 && character == '-')
        })
    {
        whole.to_owned()
    } else {
        observed.to_owned()
    }
}

fn output_has_shape(task: &VerifiableTask, value: &str) -> bool {
    match task.expectation {
        TaskExpectation::Numeric { .. } | TaskExpectation::Count { .. } => {
            value.parse::<f64>().is_ok()
        }
        TaskExpectation::Unknown { .. } => value.parse::<f64>().is_ok(),
        TaskExpectation::EditedText { .. }
        | TaskExpectation::Callable { .. }
        | TaskExpectation::Stdout { .. } => !value.is_empty(),
        TaskExpectation::Boolean => matches!(value, "true" | "false"),
    }
}

fn append_round_trip_checks(task: &VerifiableTask, answer: &mut VerifiedAnswer) {
    let passed = match &task.expectation {
        TaskExpectation::Unknown { .. } if task.quantities.len() >= 2 => {
            let coefficient = task.quantities[0].value.parse::<f64>().ok();
            let target = task.quantities[1].value.parse::<f64>().ok();
            let value = answer.value.parse::<f64>().ok();
            matches!((coefficient, target, value), (Some(a), Some(b), Some(x)) if a.mul_add(x, -b).abs() < 1e-9)
        }
        TaskExpectation::Count { .. } => answer.value.parse::<usize>().is_ok_and(|count| {
            count
                <= task
                    .entities
                    .iter()
                    .filter_map(|entity| entity.multiplicity.parse::<usize>().ok())
                    .sum()
        }),
        TaskExpectation::EditedText { source } => {
            answer.value != *source && !answer.value.is_empty()
        }
        _ => false,
    };
    if passed {
        answer.checks.push(symbolic_check(
            &answer.derivation_id,
            "round_trip",
            answer.value.as_bytes(),
        ));
    }
}

fn append_unit_check(task: &VerifiableTask, answer: &mut VerifiedAnswer) {
    let TaskExpectation::Numeric { unit: Some(target) } = &task.expectation else {
        return;
    };
    if task
        .quantities
        .iter()
        .all(|quantity| conversion_scale(quantity.unit.as_deref(), Some(target)).is_some())
    {
        answer.checks.push(symbolic_check(
            &answer.derivation_id,
            "unit_consistency",
            target.as_bytes(),
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerAgreement {
    Insufficient,
    Correlated,
    Independent,
    Disagreement,
}

fn same_answer(left: &VerifiedAnswer, right: &VerifiedAnswer) -> bool {
    match (
        left.value.trim().parse::<f64>(),
        right.value.trim().parse::<f64>(),
    ) {
        (Ok(left), Ok(right)) => (left - right).abs() < 1e-9,
        _ => left.value.trim() == right.value.trim(),
    }
}

fn derivations_are_disjoint(left: &VerifiedAnswer, right: &VerifiedAnswer) -> bool {
    left.derivation_id != right.derivation_id
        && left
            .fragments
            .iter()
            .all(|fragment| !right.fragments.contains(fragment))
}

pub fn classify_agreement(answers: &[VerifiedAnswer]) -> AnswerAgreement {
    if answers.len() < 2 {
        return AnswerAgreement::Insufficient;
    }
    let first = &answers[0];
    if !answers
        .iter()
        .skip(1)
        .all(|answer| same_answer(first, answer))
    {
        return AnswerAgreement::Disagreement;
    }
    if answers.iter().enumerate().any(|(index, left)| {
        answers
            .iter()
            .skip(index + 1)
            .any(|right| derivations_are_disjoint(left, right))
    }) {
        AnswerAgreement::Independent
    } else {
        AnswerAgreement::Correlated
    }
}

fn evidence_check_slug(evidence: &Evidence) -> &str {
    if evidence.kind == ObservationKind::SymbolicCheck {
        return evidence
            .command
            .rsplit(':')
            .next()
            .unwrap_or("symbolic_check");
    }
    evidence
        .produced_by
        .strip_prefix("verifiable_task:")
        .unwrap_or(&evidence.produced_by)
}

fn symbolic_check(derivation_id: &str, slug: &str, observed: &[u8]) -> Evidence {
    let mut evidence = Evidence::observed(
        format!("{derivation_id}:{slug}"),
        Vec::new(),
        None,
        observed,
        ObservationKind::SymbolicCheck,
        EvidenceSource::Engine,
    );
    derivation_id.clone_into(&mut evidence.for_need);
    evidence.produced_by = format!("verifiable_task:{slug}");
    evidence
}

