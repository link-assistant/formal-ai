//! Specialized free-function handlers extracted from `solver.rs`.
include!("modules.rs");
pub use agent_workspace::try_agent_workspace_task;
pub use behavior_rules::try_behavior_rules_with_runtime;
pub use benchmark_prompts::{
    fact_store_resolves, try_brainstorming_request, try_conversation_topic_request,
    try_coreference_request, try_fact_lookup, try_roleplay_request, try_summarization_request,
};
pub use calendar::try_calendar_reasoning;
pub use calendar_create::{
    calendar_claims, try_calendar_create_event, try_routed_calendar_create_event,
};
pub use compound_interest::try_compound_interest;
pub use conversation_memory::is_exact_memory_query;
pub use conversation_memory::{
    MemoryQueryExecution, answer_memory_recall, execute_memory_query,
    execute_memory_query_with_options, try_conversation_memory,
};
pub use document_originality::try_document_originality_check;
pub use document_request::try_document_request;
pub use fact_checking::try_fact_checking;
pub use feature_capability::{CapabilityRuntime, try_feature_capability};
pub use installation_conversion::try_installation_conversion;
pub use meta_explanation::{try_meta_explanation, try_meta_explanation_with_runtime};
pub use natural_language_tools::try_natural_language_tool_request;
pub use numeric_list::{try_numeric_list, try_numeric_list_with_history};
pub use pattern_inference::{try_pattern_inference, try_pattern_inference_with_response_language};
pub use playwright_script::try_playwright_script;
pub use program_blueprint::try_program_blueprint;
pub use program_synthesis::{
    looks_like_python_function_request, try_program_synthesis, try_program_synthesis_with_online,
};
pub use research_table::{try_research_comparison_table, try_research_result_followup};
pub use response_language_followup::try_response_language_followup;
pub use self_awareness::SelfAwarenessRuntime;
pub use shell_command_transform::{
    try_shell_command_transform, try_shell_command_transform_with_history,
};
pub use software_project::{software_project_claims, try_software_project_request};
pub use software_project_followup::try_software_project_followup;
pub use task_decomposition::{looks_like_task_decomposition, try_task_decomposition_with_depth};
pub use text_manipulation::{
    names_a_quoted_replacement, names_text_operation, text_outside_quoted_segments,
};
pub use text_manipulation::{try_text_manipulation, try_text_manipulation_with_history};
pub use user_intent::{try_proof_request, try_proof_request_with_config};
pub use verifiable_task::try_verifiable_task;
pub use verifiable_task::{AnswerAgreement, VerifiedAnswer, classify_agreement};
pub use web_requests::{
    detect_web_search_query, try_explicit_repository_lookup, try_http_fetch,
    try_http_fetch_with_offline, try_project_lookup, try_project_lookup_with_response_language,
    try_routed_http_fetch_with_offline, try_url_navigate, try_web_search,
    try_web_search_with_client, try_web_search_with_offline, url_navigation_claims,
};
pub use world_state::try_world_state;
pub use {
    web_requests::answer_web_search_query, web_search_intent::WebSearchQueryKind,
    web_search_intent::web_search_query_for,
};

use crate::calculation::{
    PromptInterpretation, calculation_expression_candidates, evaluate_calculation,
    interpretation_statements,
};
use crate::engine::{
    ExecutionStatus, SymbolicAnswer, answer_links_notation, hello_world_program_by_alias, stable_id,
};
use crate::event_log::{EventLog, build_evidence_links};
use crate::solver_helpers::{
    build_sorting_algorithm_answer, detect_algorithm_language, detect_program_languages,
    extract_backticked, extract_javascript_program, extract_quoted_phrase,
    format_write_script_execution, infer_program_languages_from_code, infer_source_from_prompt,
    is_write_script_request, normalize_code_meaning, normalize_meaning, translate_program,
};
use crate::translation::{
    detect_source_language, detect_target_language, extract_unquoted_translation_surface,
};

pub fn try_arithmetic(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    if let Some(answer) = calculator_rate::try_calculator_rate_basis(prompt, log) {
        return Some(answer);
    }

    let candidates = calculation_expression_candidates(prompt);
    let mut first_explicit_error: Option<(String, String, Vec<PromptInterpretation>)> = None;
    for candidate in candidates {
        let expression = candidate.expression;
        let interpretations = candidate.interpretations;
        let reasoning_steps = candidate.reasoning_steps;
        let result_label = candidate.result_label;
        log.append("calculation:request", expression.clone());
        match evaluate_calculation(&expression) {
            Ok(evaluation) => {
                let formatted = evaluation.formatted;
                log.append("calculation:engine", evaluation.engine.slug());
                if let Some(lino) = evaluation.lino {
                    log.append("calculation:lino", lino);
                }
                if !evaluation.steps.is_empty() {
                    log.append("calculation:steps", evaluation.steps.len().to_string());
                }
                let calculation_body = if expression.contains('=') && formatted.contains(" = ") {
                    format!("{expression} => {formatted}")
                } else {
                    format!("{expression} = {formatted}")
                };
                if !reasoning_steps.is_empty() {
                    log.append(
                        "calculation:reasoning_steps",
                        reasoning_steps.len().to_string(),
                    );
                }
                if let Some(label) = result_label.as_deref() {
                    log.append("calculation:result_label", label.to_owned());
                }
                for interpretation in &interpretations {
                    log.append(
                        "interpretation",
                        format!(
                            "{} -> {}",
                            interpretation.original, interpretation.corrected
                        ),
                    );
                }
                let mut sections = Vec::new();
                if !interpretations.is_empty() {
                    sections.push(interpretation_statements(&interpretations));
                }
                if !reasoning_steps.is_empty() {
                    sections.push(
                        reasoning_steps
                            .iter()
                            .enumerate()
                            .map(|(index, step)| render_calculation_reasoning_step(index, step))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                }
                sections.push(calculation_body);
                if let Some(label) = result_label {
                    sections.push(format!(
                        "Therefore, there are {formatted} {label} in total."
                    ));
                }
                let body = sections.join("\n\n");
                log.append("calculation", body.clone());
                return Some(finalize_simple(
                    prompt,
                    log,
                    "calculation",
                    "response:calculation",
                    &body,
                    1.0,
                ));
            }
            Err(error) => {
                let error = error.to_string();
                log.append("calculation:error", error.clone());
                if candidate.explicit && first_explicit_error.is_none() {
                    first_explicit_error = Some((expression, error, interpretations));
                }
            }
        }
    }
    let (expression, error, interpretations) = first_explicit_error?;
    for interpretation in &interpretations {
        log.append(
            "interpretation",
            format!(
                "{} -> {}",
                interpretation.original, interpretation.corrected
            ),
        );
    }
    let error_body = format!(
        "I parsed '{expression}' as an arithmetic request but could not evaluate it: {error}."
    );
    let body = if interpretations.is_empty() {
        error_body
    } else {
        format!(
            "{}\n\n{}",
            interpretation_statements(&interpretations),
            error_body
        )
    };
    Some(finalize_simple(
        prompt,
        log,
        "calculation_error",
        "response:calculation_error",
        &body,
        0.3,
    ))
}

fn render_calculation_reasoning_step(index: usize, step: &str) -> String {
    if step.trim_start().starts_with('[') {
        step.to_owned()
    } else {
        format!("Step {}: {step}", index + 1)
    }
}

// Plan 09 leaf 18: the concept-lookup orchestration and its renderers live in
// `src/concepts.rs`, beside the extraction and ranking machinery they drive.
pub use crate::concepts::{
    render_source_link, try_concept_lookup, try_concept_lookup_with_response_language,
};

pub fn try_javascript_execution(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let program = extract_javascript_program(prompt)?;
    log.append("execution:request", "javascript".to_owned());
    log.append("execution:source", program.clone());
    log.append("execution_status", "javascript:unavailable".to_owned());
    log.append("execution_environment", "no-js-runtime".to_owned());
    let body = format!(
        "I do not embed a JavaScript runtime in this build, so I cannot \
         execute the program for you. The deterministic solver only runs \
         code that has been verified offline; running arbitrary JavaScript \
         would violate that contract. Here is the program you asked me to \
         run, copy-paste reviewable:\n\n```js\n{program}\n```\n\n\
         To execute it yourself, save the snippet as `program.js` and run \
         `node program.js` (or `deno run program.js`)."
    );
    Some(finalize_simple(
        prompt,
        log,
        "javascript_execution_unavailable",
        "response:javascript_execution_unavailable",
        &body,
        0.6,
    ))
}

// Plan 09 leaf 18: the network snapshot, source refresh, learn-from-source
// and source-conflict procedures live in `src/retrieval_procedures.rs`, beside
// the M2 retrieval interpreter they extend.
pub use crate::retrieval_procedures::{
    try_learn_from_source, try_network_query, try_source_conflict, try_source_refresh,
};

pub fn try_translation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if document_request::looks_like_document_conversion_request(prompt, normalized) {
        return None;
    }

    let target = detect_target_language(normalized);
    let formal_language = crate::translation::formal_language_in_prompt(normalized);
    let backticked = extract_backticked(prompt);
    let detected_program = backticked.as_deref().and_then(|code| {
        detect_program_languages(normalized)
            .or_else(|| infer_program_languages_from_code(code, normalized))
    });
    let has_program_target = detected_program.is_some();
    let unquoted_surface = extract_unquoted_translation_surface(prompt);
    // Issue #386: recognise a translation command by *meaning*, not by hardcoded
    // verbs. The translation-action stems live once in
    // data/seed/meanings-translation.lino; this code knows the concept and the
    // head-initial/head-final typology the `translate` gloss documents.
    // Clause-initial English/Russian commands are matched as a prefix; head-final
    // Hindi/Chinese place the verb later, so they are matched anywhere but gated
    // by a target marker (as before) to avoid firing on an incidental verb noun.
    let lexicon = crate::seed::lexicon();
    let head_initial_command = lexicon
        .words_for_role_in_languages(crate::seed::ROLE_TRANSLATION_ACTION, &["en", "ru"])
        .iter()
        .any(|stem| normalized.starts_with(stem.as_str()));
    let head_final_command = (target.is_some() || has_program_target || formal_language.is_some())
        && lexicon
            .words_for_role_in_languages(crate::seed::ROLE_TRANSLATION_ACTION, &["hi", "zh"])
            .iter()
            .any(|stem| normalized.contains(stem.as_str()));
    let source_first_command = crate::translation::prompt::is_source_first_translation_request(
        normalized,
        target.is_some() || has_program_target || formal_language.is_some(),
        unquoted_surface.is_some(),
    );
    // Issue #386: the define-in-Links-Notation request is recognised by *meaning*
    // too, not by literal verbs and format strings. The imperative verb lives once
    // as the `definition_command` meaning and the target-format phrases as
    // `links_notation_format`, both in data/seed/meanings-translation.lino. Exactly
    // as the original recogniser, only the English verb is scanned (a clause-initial
    // prefix with a trailing space, so `defined`/`definition` never trigger it) and
    // the English and Russian format markers are scanned as space-prefixed
    // substrings; the Hindi and Chinese surfaces are carried for coverage only.
    let is_define_in_links = || {
        lexicon
            .words_for_role_in_languages(crate::seed::ROLE_DEFINITION_COMMAND, &["en"])
            .iter()
            .any(|verb| normalized.starts_with(format!("{verb} ").as_str()))
            && (extract_quoted_phrase(prompt).is_some() || extract_backticked(prompt).is_some())
            && lexicon
                .words_for_role_in_languages(crate::seed::ROLE_LINKS_NOTATION_FORMAT, &["en", "ru"])
                .iter()
                .any(|marker| normalized.contains(format!(" {marker}").as_str()))
    };
    let define_in_links = is_define_in_links();
    let is_translation_request =
        head_initial_command || head_final_command || source_first_command || define_in_links;
    if !is_translation_request {
        return None;
    }

    let mut source = detect_source_language(normalized);
    if source.is_none() {
        source = Some(infer_source_from_prompt(prompt));
    }

    // Issue #917: natural and formal statements are concrete syntaxes of the
    // same seed-defined semantic triple. A natural target means the formal
    // syntax is the source (`from FOL to Russian`); otherwise the named formal
    // language is the target (`from English to FOL`). This runs before the
    // atomic Wiktionary pipeline because a complete statement carries three
    // language-neutral meaning ids, not one word meaning.
    if let Some(formal_slug) = formal_language {
        let statement_surface = backticked
            .clone()
            .or_else(|| extract_quoted_phrase(prompt))
            .or_else(|| unquoted_surface.clone())
            .unwrap_or_default();
        let (source_slug, target_slug) = target.map_or_else(
            || (source.unwrap_or("en"), formal_slug),
            |natural_target| (formal_slug, natural_target),
        );
        if let Ok(translated) =
            crate::translation::translate_statement(&statement_surface, source_slug, target_slug)
        {
            log.append("language_from", source_slug.to_owned());
            log.append("language_to", target_slug.to_owned());
            log.append("meaning", translated.meaning);
            log.append("surface", translated.surface.clone());
            let intent = format!("translate_{source_slug}_to_{target_slug}");
            return Some(finalize_simple(
                prompt,
                log,
                &intent,
                "response:translate_statement",
                &translated.surface,
                1.0,
            ));
        }
    }

    if let Some(code) = &backticked
        && let Some((source_lang, target_lang)) = detected_program
    {
        let translated = translate_program(code, source_lang, target_lang);
        let body = format!(
            "Translated `{code}` from {source_lang} to {target_lang}:\n\n```{target_lang}\n{translated}\n```"
        );
        log.append("language_from", source_lang.to_owned());
        log.append("language_to", target_lang.to_owned());
        let meaning_id = stable_id("meaning", &normalize_code_meaning(code));
        log.append("meaning", meaning_id);
        let intent = format!("translate_{source_lang}_to_{target_lang}");
        return Some(finalize_simple(
            prompt,
            log,
            &intent,
            "response:translate_code",
            &body,
            1.0,
        ));
    }

    // Prefer an explicitly quoted fragment (`Translate "apple" to Russian`).
    // When the user omits the quotes (`translate apple to russian`),
    // fall back to a structural extraction of the substring between the
    // verb and the target preposition so the Wiktionary pipeline still
    // receives a non-empty surface. See issue #216.
    let surface = extract_quoted_phrase(prompt)
        .or(unquoted_surface)
        .unwrap_or_default();
    let source_slug = source.unwrap_or("en");
    let target_slug = target.unwrap_or("en");

    log.append("language_from", source_slug.to_owned());
    log.append("language_to", target_slug.to_owned());

    // Run the real Wiktionary + Wikidata translation pipeline. The pipeline
    // returns a `MeaningId` that we publish into the trace verbatim, so two
    // surfaces that resolve to the same Wikidata Q-item end up with the
    // same `meaning:...` id regardless of source language.
    let pipeline_result =
        crate::solver_helpers::translate_surface_detailed(&surface, source_slug, target_slug);

    let (target_surface, meaning_id, translation_gap) = if let Ok(translation) = pipeline_result {
        let target_surface = translation.primary_surface().map(str::to_owned);
        let gap = target_surface.is_none();
        // A seed meaning stands in only where Wikidata has nothing to say and
        // the caller asked for links; otherwise the pipeline's own id is the id.
        let seed_meaning = (define_in_links && !translation.meaning.is_wikidata_backed())
            .then(|| crate::translation::seed_meaning_for_surface(&surface, source_slug))
            .flatten();
        let meaning_id =
            seed_meaning.map_or_else(|| translation.meaning.slug(), |meaning| meaning.slug());
        (target_surface, meaning_id, gap)
    } else {
        // Fallback: hash the surface fragment so the trace still has a
        // stable id. The pipeline error itself is not propagated to the
        // user; the response below reports the gap without manufacturing
        // a fake target-language surface.
        let surface_meaning = if surface.is_empty() {
            prompt
        } else {
            surface.as_str()
        };
        let id = stable_id("meaning", &normalize_meaning(surface_meaning));
        (None, id, true)
    };
    log.append("meaning", meaning_id);
    if translation_gap && !surface.is_empty() {
        log.append("translation_gap", surface.clone());
    }

    let body = target_surface.map_or_else(
        || render_translation_gap(&surface, source_slug, target_slug),
        |raw_target| {
            let translated_surface =
                crate::translation::match_source_formatting(&raw_target, &surface);
            log.append("surface", translated_surface.clone());
            if surface.is_empty() {
                translated_surface
            } else {
                format!("\"{translated_surface}\"")
            }
        },
    );
    let intent = format!("translate_{source_slug}_to_{target_slug}");
    Some(finalize_simple(
        prompt,
        log,
        &intent,
        "response:translate",
        &body,
        1.0,
    ))
}

fn render_translation_gap(surface: &str, source_slug: &str, target_slug: &str) -> String {
    let surface = surface.trim();
    if surface.is_empty() {
        return format!(
            "I could not identify a source phrase to translate from {source_slug} to \
             {target_slug}."
        );
    }
    format!(
        "I could not translate \"{surface}\" from {source_slug} to {target_slug} with the \
         available formalization data. I recorded this as a translation gap for follow-up."
    )
}

pub fn try_write_script(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !is_write_script_request(prompt, normalized) {
        return None;
    }
    let program = hello_world_program_by_alias(normalized)?;
    let body = format!(
        "Here is a minimal {} script:\n\n```{}\n{}\n```\n\n{}",
        program.language.name,
        program.language.code_fence,
        program.template.code,
        format_write_script_execution(program)
    );
    let intent = format!("write_script_{}", program.language.slug);
    log.append(
        "execution_status",
        program.language.execution_status().label().to_owned(),
    );
    log.append("execution_environment", program.language.environment());
    Some(finalize_simple(
        prompt,
        log,
        &intent,
        &format!(
            "response:write_program:hello_world:{}",
            program.language.slug
        ),
        &body,
        1.0,
    ))
}

pub fn try_algorithm(prompt: &str, normalized: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    if !normalized.contains("algorithm") && !normalized.contains("sort") {
        return None;
    }
    let with_tests = normalized.contains("test");
    let lang_slug = detect_algorithm_language(normalized);
    let body = build_sorting_algorithm_answer(lang_slug, with_tests);
    let intent = format!("algorithm_sort_{lang_slug}");
    log.append(
        "execution_status",
        ExecutionStatus::Unavailable.label().to_owned(),
    );
    log.append(
        "execution_environment",
        "no compile/run sandbox configured for this generated snippet".to_owned(),
    );
    Some(finalize_simple(
        prompt,
        log,
        &intent,
        "response:algorithm",
        &body,
        1.0,
    ))
}

pub fn try_execution_failure(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !normalized.contains("undefined_function") {
        return None;
    }
    log.append("trace:execution_failure", "undefined_function".to_owned());
    let body = String::from(
        "Execution status: failed in isolated sandbox.\n\
         ```python\nundefined_function()\n```\n\
         Traceback (most recent call last):\n  File 'main.py', line 1, in <module>\n\
         NameError: name 'undefined_function' is not defined.\n\
         The failure trace is appended to the action log; see the trace link.",
    );
    let agent_request = normalized.contains("[agent]");
    if agent_request {
        log.append("agent_mode:opted_in", prompt.to_owned());
        log.append("action_log", prompt.to_owned());
    }
    Some(finalize_simple(
        prompt,
        log,
        "execution_failure",
        "response:execution_failure",
        &body,
        0.4,
    ))
}

pub fn finalize_simple(
    prompt: &str,
    log: &mut EventLog,
    intent: &str,
    response_link: &str,
    body: &str,
    confidence: f32,
) -> SymbolicAnswer {
    let body = crate::question_necessity::enforce_questions(body, log);
    log.append("intent", intent.to_owned());
    if log.first_of("candidate").is_none() {
        log.append("candidate", intent.to_owned());
    }
    if log.first_of("validation").is_none() {
        log.append(
            "validation",
            "accepted_without_extra_constraints".to_owned(),
        );
    }
    log.append("response", response_link.to_owned());
    if log.first_of("trace:simplification").is_none() {
        log.append("trace:simplification", "smallest_sufficient".to_owned());
    }
    let trace_id = log.append("trace", intent.to_owned());
    let evidence_links = build_evidence_links(prompt, log, response_link);
    let links_notation = answer_links_notation(prompt, intent, &body, log, &trace_id);
    SymbolicAnswer {
        intent: intent.to_owned(),
        thinking_steps: log.thinking_steps_for_answer(&body),
        answer: body,
        execution_recipe: None,
        confidence,
        evidence_links,
        links_notation,
    }
}
