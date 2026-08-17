//! Unknown-path rule construction for program-modification follow-ups.
//!
//! This is the runtime slice of `docs/design/rule-synthesis.md`: when seed
//! routing yields `unknown`, bind the active program artifact, decompose the
//! request through the operation vocabulary, construct a program-plan candidate,
//! verify it, and only then hand the solver a concrete rule to answer with.

use std::fmt::Write as _;
use std::path::Path;

use crate::coding::{program_spec, ProgramSpec};
use crate::engine::{
    normalize_prompt, ExecutionRecipe, ExecutionRecipeFile, SelectedRule, SymbolicAnswer,
};
use crate::event_log::EventLog;
use crate::intent_formalization::{
    active_program_context, detected_program_modifiers, ActiveProgramContext,
};
use crate::meta_algorithm_builder::{CodingSurface, MetaAlgorithmBuilder};
use crate::program_coreference::looks_like_bare_program_artifact_follow_up;
use crate::program_plan::ProgramPlan;
use crate::solver::ConversationTurn;
use crate::substitution_compiler::{
    CompiledSubstitutionFile, CompiledSubstitutionProgram, SubstitutionCompilationTarget,
};

pub struct UnknownRuleConstruction {
    pub rule: SelectedRule,
    pub coreference_trace: String,
    pub recovery_trace: String,
    pub operation_hits: String,
    pub request: String,
    pub candidate: String,
    pub verification: String,
    pub plan: String,
    pub program_plan: ProgramPlan,
}

/// Export a semantically verified program-modification rule to one requested
/// compiler target. The operation and target names come from seed data.
pub fn try_export_substitution_program(
    prompt: &str,
    history: &[ConversationTurn],
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let normalized = normalize_prompt(prompt);
    let vocabulary = crate::seed::operation_vocabulary();
    if !vocabulary.matches("export_substitution_rule", &normalized) {
        return None;
    }
    let targets = [
        ("target_rust", SubstitutionCompilationTarget::Rust),
        (
            "target_javascript",
            SubstitutionCompilationTarget::JavaScript,
        ),
        (
            "target_webassembly",
            SubstitutionCompilationTarget::WebAssembly,
        ),
    ]
    .into_iter()
    .filter_map(|(operation, target)| vocabulary.matches(operation, &normalized).then_some(target))
    .collect::<Vec<_>>();
    let [target] = targets.as_slice() else {
        return None;
    };

    log.append(
        "selected_rule",
        "initial unknown reason export_requires_verified_rule_synthesis".to_owned(),
    );
    let construction = construct_rule_from_unknown(prompt, history)?;
    let mut artifact = construction.program_plan.compile(*target).ok()?;
    artifact.supporting_files.push(CompiledSubstitutionFile {
        name: String::from("input.tsv"),
        contents: construction.program_plan.compiler_input_tsv(),
    });
    record_construction(log, &construction);
    MetaAlgorithmBuilder::for_surface(CodingSurface::RuleSynthesis).record(log);
    log.append("substitution_compilation", artifact.trace.clone());
    for file in std::iter::once(&artifact.primary_file).chain(&artifact.supporting_files) {
        log.append("substitution_export_file", file.name.clone());
    }

    let language = crate::language::detect(prompt);
    let intro = crate::seed::render_response(
        "substitution_rule_export",
        language.slug(),
        &[
            ("target", target.slug()),
            ("primary_file", &artifact.primary_file.name),
        ],
    )?;
    let commands = execution_commands(*target, &artifact.primary_file.name);
    let body = render_export(&intro, &artifact, &commands);
    let supporting_files = artifact
        .supporting_files
        .iter()
        .map(|file| ExecutionRecipeFile {
            path: file.name.clone(),
            source: file.contents.clone(),
        })
        .collect();
    let mut answer = crate::solver_handlers::finalize_simple(
        prompt,
        log,
        "substitution_rule_export",
        "response:substitution_rule_export",
        &body,
        1.0,
    );
    answer.execution_recipe = Some(Box::new(ExecutionRecipe {
        language: target.slug().to_owned(),
        source: artifact.primary_file.contents,
        path: artifact.primary_file.name,
        supporting_files,
        commands,
    }));
    Some(answer)
}

fn record_construction(log: &mut EventLog, construction: &UnknownRuleConstruction) {
    log.append(
        "write_program_coreference_rewrite",
        construction.coreference_trace.clone(),
    );
    log.append(
        "rule_synthesis_operation_vocabulary",
        construction.operation_hits.clone(),
    );
    log.append("rule_synthesis_request", construction.request.clone());
    log.append("rule_synthesis_candidate", construction.candidate.clone());
    log.append("rule_verification", construction.verification.clone());
    log.append(
        "write_program_context_recovery",
        construction.recovery_trace.clone(),
    );
    log.append("write_program_plan", construction.plan.clone());
}

fn execution_commands(target: SubstitutionCompilationTarget, primary: &str) -> Vec<String> {
    match target {
        SubstitutionCompilationTarget::Rust => vec![
            format!("rustc --edition=2021 {primary} -o substitution_program"),
            String::from("./substitution_program < input.tsv"),
        ],
        SubstitutionCompilationTarget::JavaScript => {
            let stem = primary.strip_suffix(".mjs").unwrap_or(primary);
            vec![
                String::from("rustup target add wasm32-unknown-unknown"),
                format!(
                    "rustc --edition=2021 --target wasm32-unknown-unknown --crate-type cdylib -C panic=abort {stem}_wasm.rs -o {stem}.wasm"
                ),
                format!("node {primary} {stem}.wasm < input.tsv"),
            ]
        }
        SubstitutionCompilationTarget::WebAssembly => vec![
            String::from("rustup target add wasm32-unknown-unknown"),
            format!(
                "rustc --edition=2021 --target wasm32-unknown-unknown --crate-type cdylib -C panic=abort {primary} -o substitution_program.wasm"
            ),
            String::from(
                "node run_substitution_wasm.mjs substitution_program.wasm < input.tsv",
            ),
        ],
    }
}

fn render_export(
    intro: &str,
    artifact: &CompiledSubstitutionProgram,
    commands: &[String],
) -> String {
    let mut output = String::from(intro);
    for file in std::iter::once(&artifact.primary_file).chain(&artifact.supporting_files) {
        let _ = write!(
            output,
            "\n\n`{}`\n```{}\n{}\n```",
            file.name,
            code_fence(file),
            file.contents.trim_end()
        );
    }
    output.push_str("\n\n```sh\n");
    output.push_str(&commands.join("\n"));
    output.push_str("\n```");
    output
}

fn code_fence(file: &CompiledSubstitutionFile) -> &'static str {
    let extension = Path::new(&file.name).extension();
    if extension.is_some_and(|value| value.eq_ignore_ascii_case("rs")) {
        "rust"
    } else if extension.is_some_and(|value| value.eq_ignore_ascii_case("mjs")) {
        "javascript"
    } else if extension.is_some_and(|value| value.eq_ignore_ascii_case("tsv")) {
        "text"
    } else {
        "json"
    }
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

    MetaAlgorithmBuilder::for_surface(CodingSurface::RuleSynthesis).record(log);
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

/// Resolve an unknown program follow-up from a previously approved lesson.
///
/// This is deliberately before fresh synthesis in the solver. The ledger
/// supplies the approved modifier, while the active conversation supplies the
/// current base task and language, so recall generalises across compatible
/// program artifacts instead of replaying a stale answer string.
pub fn try_recall_approved_rule(
    rule: SelectedRule,
    follow_up: &str,
    history: &[ConversationTurn],
    log: &mut EventLog,
) -> SelectedRule {
    if !matches!(rule, SelectedRule::Unknown) {
        return rule;
    }
    let Some(lesson) = crate::learning_ledger::approved_lesson_for(follow_up) else {
        return rule;
    };
    let Some(context) = active_program_context(history) else {
        return rule;
    };
    let plan = crate::program_plan::lower(&context.task, std::slice::from_ref(&lesson.modifier));
    let Some(spec) = program_spec(&plan.resolved_task, &context.language) else {
        return rule;
    };
    log.append("learning_ledger_recall.lesson", lesson.lesson_id);
    log.append("learning_ledger_recall.rule", lesson.rule_id);
    log.append("learning_ledger_recall.modifier", lesson.modifier);
    log.append("learning_ledger_recall.approved_by", lesson.reviewer);
    log.append("write_program_plan", plan.links_notation());
    SelectedRule::WriteProgram(spec)
}

#[must_use]
pub fn construct_rule_from_unknown(
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

    let plan_trace = plan.links_notation();
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
        plan: plan_trace,
        program_plan: plan,
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
        "cancel_reverse_sort" => DecompositionParts {
            // Issue #386: the inverse of reverse_sort — cancel the descending
            // order over the same program-output target.
            operation: "cancel",
            operation_modifier: Some("reverse_sort"),
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
    let cancels_sort = modifiers
        .iter()
        .any(|modifier| modifier == "cancel_reverse_sort");
    let reverses_sort = modifiers.iter().any(|modifier| modifier == "reverse_sort");
    let descending = template_has_descending_order(spec.template.code);
    // Issue #386: verify the rendered program actually matches the operation.
    // A reverse_sort must leave the output descending; its inverse,
    // cancel_reverse_sort, must leave NO descending order — otherwise the cancel
    // silently failed to remove the sort. Modifiers that touch no ordering pass.
    let render_check = if cancels_sort {
        !descending
    } else if reverses_sort {
        descending
    } else {
        true
    };
    let passed = plan_check && render_check;
    let expected_order = if reverses_sort && !cancels_sort {
        "c.txt,b.txt,a.txt"
    } else {
        "a.txt,b.txt,c.txt"
    };
    let mut out = String::from("rule_verification\n");
    push_field(&mut out, "candidate", candidate_id);
    push_field(&mut out, "fixture", "list_files_output_order");
    push_field(&mut out, "input", "a.txt,b.txt,c.txt");
    push_field(&mut out, "expected_order", expected_order);
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
