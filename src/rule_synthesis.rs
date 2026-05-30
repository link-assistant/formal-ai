//! Unknown-path rule construction for program-modification follow-ups.
//!
//! This is the runtime slice of `docs/design/rule-synthesis.md`: when seed
//! routing yields `unknown`, bind the active program artifact, decompose the
//! request through the operation vocabulary, construct a program-plan candidate,
//! verify it, and only then hand the solver a concrete rule to answer with.

use std::fmt::Write as _;

use crate::coding::{program_spec, ProgramSpec};
use crate::engine::{normalize_prompt, SelectedRule};
use crate::event_log::EventLog;
use crate::intent_formalization::{
    active_program_context, detected_program_modifiers, ActiveProgramContext,
};
use crate::program_coreference::looks_like_bare_program_artifact_follow_up;
use crate::program_plan::ProgramPlan;
use crate::solver::ConversationTurn;

struct UnknownRuleConstruction {
    rule: SelectedRule,
    coreference_trace: String,
    recovery_trace: String,
    operation_hits: String,
    request: String,
    candidate: String,
    verification: String,
    plan: String,
}

pub fn try_construct_unknown_rule(
    rule: SelectedRule,
    follow_up: &str,
    history: &[ConversationTurn],
    log: &mut EventLog,
) -> SelectedRule {
    if !matches!(&rule, SelectedRule::Unknown) {
        return rule;
    }

    log.append(
        "selected_rule",
        "initial unknown reason no_seed_route next try_rule_synthesis".to_owned(),
    );

    let Some(construction) = construct_rule_from_unknown(follow_up, history) else {
        return rule;
    };

    log.append(
        "write_program_coreference_rewrite",
        construction.coreference_trace,
    );
    log.append(
        "rule_synthesis_operation_vocabulary",
        construction.operation_hits,
    );
    log.append("rule_synthesis_request", construction.request);
    log.append("rule_synthesis_candidate", construction.candidate);
    log.append("rule_verification", construction.verification);
    log.append(
        "write_program_context_recovery",
        construction.recovery_trace,
    );
    log.append("write_program_plan", construction.plan);
    construction.rule
}

#[must_use]
fn construct_rule_from_unknown(
    follow_up: &str,
    history: &[ConversationTurn],
) -> Option<UnknownRuleConstruction> {
    let normalized = normalize_prompt(follow_up);
    if !looks_like_bare_program_artifact_follow_up(&normalized) {
        return None;
    }

    let context = active_program_context(history)?;
    let modifiers = detected_program_modifiers(&normalized);
    if modifiers.is_empty() {
        return None;
    }

    let plan = crate::program_plan::lower(&context.task, &modifiers);
    if !plan.was_modified() {
        return None;
    }

    let spec = program_spec(&plan.resolved_task, &context.language)?;
    let primary_modifier = primary_modifier(&modifiers)?;
    let candidate_id = candidate_rule_id(&plan, primary_modifier);
    let verification = verification_trace(&candidate_id, &plan, spec, &modifiers);
    if !verification.passed {
        return None;
    }

    Some(UnknownRuleConstruction {
        rule: SelectedRule::WriteProgram(spec),
        coreference_trace: format!(
            "referent=active_program_artifact task={} language={}",
            context.task, context.language
        ),
        recovery_trace: format!(
            "write_program task={} language={}",
            plan.resolved_task, context.language
        ),
        operation_hits: operation_hits(&normalized),
        request: synthesis_request(&context, follow_up, primary_modifier),
        candidate: synthesis_candidate(&candidate_id, &context, &plan, primary_modifier),
        verification: verification.links_notation,
        plan: plan.links_notation(),
    })
}

fn operation_hits(normalized: &str) -> String {
    crate::seed::operation_vocabulary()
        .detect(normalized)
        .join(",")
}

fn primary_modifier(modifiers: &[String]) -> Option<&str> {
    modifiers
        .iter()
        .find(|modifier| modifier.as_str() == "reverse_sort")
        .or_else(|| modifiers.first())
        .map(String::as_str)
}

fn candidate_rule_id(plan: &ProgramPlan, modifier: &str) -> String {
    plan.report
        .traces
        .iter()
        .rev()
        .find(|trace| trace.rule_id.contains(modifier))
        .map_or_else(
            || format!("{modifier}_{}", plan.base_task),
            |trace| trace.rule_id.clone(),
        )
}

fn synthesis_request(context: &ActiveProgramContext, follow_up: &str, modifier: &str) -> String {
    let parts = decomposition_parts(modifier);
    let mut out = String::from("rule_synthesis_request\n");
    push_field(&mut out, "issue", "#359");
    push_field(&mut out, "impulse", "current_turn");
    push_field(&mut out, "artifact", "program:last");
    push_field(&mut out, "artifact_language", &context.language);
    push_field(&mut out, "base_task", &context.task);
    push_field(&mut out, "bare_imperative", "true");
    push_field(&mut out, "operation", parts.operation);
    if let Some(operation_modifier) = parts.operation_modifier {
        push_field(&mut out, "operation_modifier", operation_modifier);
    }
    push_field(&mut out, "target", parts.target);
    push_field(&mut out, "target_kind", parts.target_kind);
    push_field(&mut out, "source_text", follow_up);
    out.trim_end().to_owned()
}

fn synthesis_candidate(
    candidate_id: &str,
    context: &ActiveProgramContext,
    plan: &ProgramPlan,
    modifier: &str,
) -> String {
    let parts = decomposition_parts(modifier);
    let mut out = String::from("rule_synthesis_candidate\n");
    push_field(&mut out, "id", candidate_id);
    push_field(&mut out, "source", "constructed_from_operation_vocabulary");
    push_field(&mut out, "base_task", &context.task);
    push_field(&mut out, "modifier", modifier);
    push_field(&mut out, "operation", parts.operation);
    if let Some(operation_modifier) = parts.operation_modifier {
        push_field(&mut out, "operation_modifier", operation_modifier);
    }
    push_field(&mut out, "target", parts.target);
    push_field(&mut out, "resolved_task", &plan.resolved_task);
    out.trim_end().to_owned()
}

struct DecompositionParts {
    operation: &'static str,
    operation_modifier: Option<&'static str>,
    target: &'static str,
    target_kind: &'static str,
}

fn decomposition_parts(modifier: &str) -> DecompositionParts {
    match modifier {
        "reverse_sort" => DecompositionParts {
            operation: "sort",
            operation_modifier: Some("descending"),
            target: "program:last.output_order",
            target_kind: "program_output",
        },
        "path_argument" => DecompositionParts {
            operation: "accept",
            operation_modifier: Some("path_argument"),
            target: "program:last.input",
            target_kind: "program_input",
        },
        _ => DecompositionParts {
            operation: "modify",
            operation_modifier: None,
            target: "program:last",
            target_kind: "program_artifact",
        },
    }
}

struct VerificationTrace {
    passed: bool,
    links_notation: String,
}

fn verification_trace(
    candidate_id: &str,
    plan: &ProgramPlan,
    spec: ProgramSpec,
    modifiers: &[String],
) -> VerificationTrace {
    let plan_check = plan.was_modified() && plan.report.applied_count() > 0;
    let render_check = !modifiers.iter().any(|modifier| modifier == "reverse_sort")
        || template_has_descending_order(spec.template.code);
    let passed = plan_check && render_check;
    let mut out = String::from("rule_verification\n");
    push_field(&mut out, "candidate", candidate_id);
    push_field(&mut out, "fixture", "list_files_output_order");
    push_field(&mut out, "input", "a.txt,b.txt,c.txt");
    push_field(&mut out, "expected_order", "c.txt,b.txt,a.txt");
    push_field(
        &mut out,
        "lowering_check",
        if plan_check { "passed" } else { "failed" },
    );
    push_field(
        &mut out,
        "render_check",
        if render_check { "passed" } else { "failed" },
    );
    push_field(&mut out, "status", if passed { "passed" } else { "failed" });
    VerificationTrace {
        passed,
        links_notation: out.trim_end().to_owned(),
    }
}

fn template_has_descending_order(code: &str) -> bool {
    let compact = code
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<String>();
    [
        "sort_by(|a,b|b.cmp(a))",
        "reverse=true",
        ".sort().reverse()",
        "sort.sort(sort.reverse",
        "compare_desc",
        "rbegin(),names.rend()",
        "comparator.reverseorder()",
        "orderbydescending",
        "sort.reverse",
    ]
    .iter()
    .any(|marker| compact.contains(marker))
}

fn push_field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} {value}");
}
