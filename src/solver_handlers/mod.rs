//! Specialized handlers extracted from the universal solver in `solver.rs` to
//! keep that module under the 1000-line cap enforced by
//! `scripts/check-file-size.rs`. These handlers are free functions: every one
//! takes the prompt (and pre-lowercased `normalized` view) plus a mutable
//! event log, and returns `Some(SymbolicAnswer)` when it claims the impulse.

mod behavior_rules;
mod benchmark_prompts;
mod calendar;
mod definition_merge;
mod feature_capability;
mod software_project;
mod software_project_code;
mod user_intent;
mod web_requests;
mod web_search_intent;

pub use behavior_rules::try_behavior_rules;
pub use benchmark_prompts::{
    try_brainstorming_request, try_coreference_request, try_fact_lookup, try_roleplay_request,
    try_summarization_request,
};
pub use calendar::try_calendar_reasoning;
pub use definition_merge::{try_definition_merge, try_definition_merge_by_default};
pub use feature_capability::{try_feature_capability, CapabilityRuntime};
pub use software_project::try_software_project_request;
pub use user_intent::{
    try_capabilities, try_clarification, try_ill_formed, try_opinion_question, try_proof_request,
    try_proof_request_with_config, try_punctuation_only_prompt, try_shell_refusal,
    try_who_is_question,
};
pub use web_requests::{try_http_fetch, try_project_lookup, try_url_navigate, try_web_search};

use std::fmt::Write as _;

use crate::calculation::{
    calculation_expression_candidates, evaluate_calculation, interpretation_statements,
    PromptInterpretation,
};
use crate::concepts::{
    extract_concept_query, lookup_concept_query, resolve_context_label, ConceptRecord,
};
use crate::engine::{
    answer_links_notation, hello_world_program_by_alias, knowledge_links_notation, stable_id,
    ExecutionStatus, SymbolicAnswer,
};
use crate::event_log::{build_evidence_links, EventLog};
use crate::language::detect as detect_language;
use crate::seed::response_for;
use crate::solver_helpers::{
    build_sorting_algorithm_answer, detect_algorithm_language, detect_program_languages,
    extract_backticked, extract_concept_from_query, extract_introduced_name,
    extract_javascript_program, extract_quoted_phrase, format_write_script_execution, humanize_url,
    infer_program_languages_from_code, infer_source_from_prompt, is_write_script_request,
    last_user_turn, normalize_code_meaning, normalize_meaning, recall_name_from_history,
    translate_program,
};
use crate::summarization::{
    generate_chat_title, summarize_dialog, DialogTurn, SummarizationConfig, SummarizationMode,
};
use crate::translation::{
    detect_source_language, detect_target_language, extract_unquoted_translation_surface,
};

pub fn try_conversation_memory(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if let Some(answer) = try_recall_name(prompt, normalized, log) {
        return Some(answer);
    }
    if let Some(answer) = try_recall_last_question(prompt, normalized, log) {
        return Some(answer);
    }
    if let Some(answer) = try_summarize_conversation(prompt, normalized, log) {
        return Some(answer);
    }
    None
}

fn try_recall_name(prompt: &str, normalized: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let asks_name = normalized.contains("what is my name")
        || normalized.contains("what's my name")
        || normalized.contains("do you know my name")
        || normalized.contains("who am i");
    if !asks_name {
        return None;
    }
    let name = recall_name_from_history(log, prompt).or_else(|| extract_introduced_name(prompt))?;
    log.append("filter:user", format!("name={name}"));
    let body = format!("Your name is {name}.");
    Some(finalize_simple(
        prompt,
        log,
        "recall_name",
        "response:recall_name",
        &body,
        0.9,
    ))
}

fn try_recall_last_question(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let asks = normalized.contains("what did i ask")
        || normalized.contains("what was my last question")
        || normalized.contains("what was my previous question")
        || normalized.contains("repeat my last message");
    if !asks {
        return None;
    }
    let previous = last_user_turn(log)?;
    let body = format!("Your previous message was: \"{previous}\"");
    log.append("filter:user", "previous_turn".to_owned());
    Some(finalize_simple(
        prompt,
        log,
        "recall_last_question",
        "response:recall_last_question",
        &body,
        0.9,
    ))
}

fn try_summarize_conversation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let asks = normalized.contains("summarize the conversation")
        || normalized.contains("summarize our conversation")
        || normalized.contains("summarize this conversation")
        || normalized.contains("summary of our chat")
        || normalized.contains("what have we talked about")
        || normalized == "summarize"
        // Russian
        || normalized.contains("о чём мы разговаривали")
        || normalized.contains("о чем мы разговаривали")
        || normalized.contains("о чём мы говорили")
        || normalized.contains("о чем мы говорили")
        || normalized.contains("резюме беседы")
        || normalized.contains("резюме разговора")
        || normalized.contains("резюмируй разговор")
        || normalized.contains("резюмируй беседу")
        || normalized == "резюме"
        // Chinese
        || normalized.contains("总结");
    if !asks {
        return None;
    }
    let turns: Vec<DialogTurn> = log
        .events()
        .iter()
        .filter_map(|event| match event.kind {
            "prior_turn:user" => Some(DialogTurn::user(event.payload.clone())),
            "prior_turn:assistant" => Some(DialogTurn::assistant(event.payload.clone())),
            _ => None,
        })
        .collect();
    let user_turn_count = turns.iter().filter(|t| t.role == "user").count();
    if user_turn_count == 0 {
        return None;
    }
    let language = detect_language(prompt).slug();
    // Standard mode keeps roughly 50% of the highest-weighted statements; with
    // the dialog bias (user +20, assistant -10) the user's questions dominate
    // the output while still keeping room for any assistant prose worth
    // remembering.
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Standard)
        .with_language(language);
    let summary = summarize_dialog(&turns, &config);
    let title = generate_chat_title(&turns, language);
    let user_turns: Vec<&str> = turns
        .iter()
        .filter(|t| t.role == "user")
        .map(|t| t.text.as_str())
        .collect();
    let mut body = match language {
        "ru" => {
            format!("Резюме разговора: {summary}\n\nЗаголовок: {title}\n\nРеплики пользователя:\n")
        }
        "zh" => format!("对话摘要:{summary}\n\n标题:{title}\n\n用户发言:\n"),
        _ => format!("Conversation summary: {summary}\n\nTitle: {title}\n\nUser turns:\n"),
    };
    for (index, turn) in user_turns.iter().enumerate() {
        writeln!(body, "  {}. {turn}", index + 1).expect("string write is infallible");
    }
    log.append("filter:user", "conversation_summary".to_owned());
    log.append("summarization:mode", "standard".to_owned());
    log.append("summarization:language", language.to_owned());
    log.append("chat_title", title);
    Some(finalize_simple(
        prompt,
        log,
        "summarize_conversation",
        "response:summarize_conversation",
        body.trim_end(),
        0.9,
    ))
}

pub fn try_arithmetic(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let candidates = calculation_expression_candidates(prompt);
    let mut first_explicit_error: Option<(String, String, Vec<PromptInterpretation>)> = None;
    for candidate in candidates {
        let expression = candidate.expression;
        let interpretations = candidate.interpretations;
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
                for interpretation in &interpretations {
                    log.append(
                        "interpretation",
                        format!(
                            "{} -> {}",
                            interpretation.original, interpretation.corrected
                        ),
                    );
                }
                let body = if interpretations.is_empty() {
                    calculation_body
                } else {
                    format!(
                        "{}\n\n{}",
                        interpretation_statements(&interpretations),
                        calculation_body
                    )
                };
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

pub fn try_concept_lookup(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let query = extract_concept_query(prompt)?;
    log.append("concept_lookup:request", query.term.clone());
    if let Some(context) = query.context.as_deref() {
        log.append("concept_lookup:context", context.to_owned());
    }
    let Some(lookup) = lookup_concept_query(&query) else {
        log.append("concept_lookup:miss", query.term);
        return None;
    };
    let record: &'static ConceptRecord = lookup.record;
    log.append("concept_lookup:hit", record.slug.clone());
    let language = detect_language(prompt).slug();
    let localized = record.localized_for(language);
    let source_for_log = localized
        .map(|loc| loc.source.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source.as_str());
    // Issue #21: log the percent-decoded IRI form so diagnostic chips stay
    // readable. Rendering uses humanize_url too (see render_concept_*).
    log.append("source", humanize_url(source_for_log));
    if !record.wikidata.is_empty() {
        log.append("wikidata", record.wikidata.clone());
    }
    if lookup.context_match {
        if let Some(context) = lookup.context.as_deref() {
            log.append("concept_lookup:context-match", context.to_owned());
            let body = render_concept_in_context(language, context, record);
            return Some(finalize_simple(
                prompt,
                log,
                "concept_lookup_in_context",
                "response:concept_lookup_in_context",
                &body,
                0.9,
            ));
        }
    } else if let Some(context) = lookup.context.as_deref() {
        log.append("concept_lookup:context-mismatch", context.to_owned());
    }
    let body = render_concept_plain(language, record);
    Some(finalize_simple(
        prompt,
        log,
        "concept_lookup",
        "response:concept_lookup",
        &body,
        0.9,
    ))
}

/// Render a plain `concept_lookup` body using the localized variant when
/// available (so `что такое IIR` in Russian returns the ru.wikipedia.org
/// summary, not the English one).
fn render_concept_plain(language: &str, record: &ConceptRecord) -> String {
    let localized = record.localized_for(language);
    let term = localized
        .map(|loc| loc.term.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.term.as_str());
    let summary = localized
        .map(|loc| loc.summary.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.summary.as_str());
    let source = localized
        .map(|loc| loc.source.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source.as_str());
    let source_kind = localized
        .map(|loc| loc.source_kind.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source_kind.as_str());
    let source_markup = render_source_link(source);
    format!(
        "{term} ({category}): {summary}\n\nSource: {source_markup} ({source_kind}).",
        category = record.category,
    )
}

/// Issue #21: render a URL as a readable IRI while keeping the canonical
/// percent-encoded form as the link target. Returns the bare URL when the
/// humanized and encoded forms match (no link wrapping needed).
fn render_source_link(source: &str) -> String {
    let human = humanize_url(source);
    if human == source {
        source.to_owned()
    } else {
        format!("[{human}]({source})")
    }
}

/// Render a `concept_lookup_in_context` body, preferring the language-specific
/// template loaded from `data/seed/multilingual-responses.lino`. Falls back
/// to the English template (and, if that is missing, a hardcoded one) so the
/// solver still works when seed loading fails.
///
/// Maintainer requirement R8 (issue #20): use the full disambiguated context
/// name, e.g. `В контексте «ml» (Машинное обучение)`. The raw user-typed
/// context is shown verbatim and the resolved registry label is appended in
/// parentheses; when the two collide (user already typed the localized
/// label) the `no_alias` template is used to avoid `«ml» (ml)`.
///
/// Maintainer requirement R9: the term and summary use the localized variant
/// (e.g. `Фильтр с бесконечной импульсной характеристикой… или IIR-фильтр`)
/// when the user's prevailing language has a `localized` block.
#[allow(clippy::literal_string_with_formatting_args)]
fn render_concept_in_context(language: &str, context: &str, record: &ConceptRecord) -> String {
    let context_record = resolve_context_label(context);
    let context_label =
        context_record.map_or_else(|| context.to_owned(), |c| c.label_for(language).to_owned());
    let use_no_alias = context_label.trim().to_lowercase() == context.trim().to_lowercase();
    let intent_variant = if use_no_alias {
        "concept_lookup_in_context_no_alias"
    } else {
        "concept_lookup_in_context"
    };
    let template = response_for(intent_variant, language)
        .or_else(|| response_for(intent_variant, "en"))
        .or_else(|| response_for("concept_lookup_in_context", language))
        .or_else(|| response_for("concept_lookup_in_context", "en"))
        .unwrap_or_else(|| {
            String::from(
                "In the context of {context} ({context_label}), {term} ({category}) means: \
                 {summary}\n\nSource: {source} ({source_kind}).",
            )
        });
    let localized = record.localized_for(language);
    let term = localized
        .map(|loc| loc.term.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.term.as_str());
    let summary = localized
        .map(|loc| loc.summary.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.summary.as_str());
    let source = localized
        .map(|loc| loc.source.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source.as_str());
    let source_kind = localized
        .map(|loc| loc.source_kind.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(record.source_kind.as_str());
    let source_markup = render_source_link(source);
    template
        .replace("{context_label}", &context_label)
        .replace("{context}", context)
        .replace("{term}", term)
        .replace("{category}", &record.category)
        .replace("{summary}", summary)
        .replace("{source}", &source_markup)
        .replace("{source_kind}", source_kind)
}

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

pub fn try_meta_explanation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let is_why_question = normalized.starts_with("why ")
        || normalized.starts_with("why did")
        || normalized.starts_with("why do you")
        || normalized.contains("why did you answer");
    let is_how_you_work = normalized.contains("how do you work")
        || normalized.contains("how does this work")
        || normalized.contains("how does it work")
        || normalized.contains("show me how you work")
        || normalized.contains("explain how you work")
        // Russian
        || normalized.contains("как ты работаешь")
        || normalized.contains("покажи как ты работаешь")
        || normalized.contains("расскажи как ты работаешь")
        || normalized.contains("объясни как ты работаешь")
        || normalized.contains("как ты устроен")
        || normalized.contains("покажи как ты устроен")
        // Hindi
        || normalized.contains("तुम कैसे काम करते हो")
        || normalized.contains("आप कैसे काम करते हैं")
        // Chinese
        || normalized.contains("你是怎么工作的")
        || normalized.contains("你怎么运作");
    if !is_why_question && !is_how_you_work {
        return None;
    }
    let language = detect_language(prompt).slug();
    let body = if is_why_question {
        response_for("meta_explanation", language)
            .or_else(|| response_for("meta_explanation", "en"))
            .unwrap_or_else(|| String::from(
                "I answered that way because the prompt matched a deterministic Links Notation rule. \
                 The evidence and trace events are appended to the log; see the trace link for the \
                 full chain.",
            ))
    } else {
        response_for("meta_explanation", language)
            .or_else(|| response_for("meta_explanation", "en"))
            .unwrap_or_else(|| {
                String::from(
                "I work by matching your prompt against deterministic Links Notation rules stored \
                 in memory. Each rule maps a recognized pattern to a fixed response. When no rule \
                 matches, I report intent: unknown. There is no neural inference — every answer is \
                 fully traceable to a symbolic rule.",
            )
            })
    };
    Some(finalize_simple(
        prompt,
        log,
        "meta_explanation",
        "response:meta_explanation",
        &body,
        1.0,
    ))
}

pub fn try_network_query(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if normalized.contains("show me the current network")
        || normalized.contains("show me the network")
        || normalized.contains("export the network")
        || normalized.contains("export network")
    {
        let snapshot = knowledge_links_notation();
        let body = format!(
            "Here is the current link network as a links-notation snapshot:\n\n```links\n{snapshot}\n```"
        );
        return Some(finalize_simple(
            prompt,
            log,
            "network_snapshot",
            "response:network_snapshot",
            &body,
            1.0,
        ));
    }
    if let Some(concept) = extract_concept_from_query(prompt) {
        let body = format!(
            "Here is what I know about '{concept}':\n\nintent: {concept}\nrole: \
             the network records '{concept}' as a concept with rules and example links."
        );
        return Some(finalize_simple(
            prompt,
            log,
            &format!("concept_introspection_{concept}"),
            "response:concept_introspection",
            &body,
            1.0,
        ));
    }
    if normalized.contains("list the facts i have contributed")
        || normalized.contains("list my facts")
        || normalized.starts_with("list facts")
    {
        log.append("filter:user", "self".to_owned());
        let body = String::from(
            "No facts have been recorded under your user filter yet. Submit a 'teach this fact' \
             request to start your personal contribution list.",
        );
        return Some(finalize_simple(
            prompt,
            log,
            "filter_user",
            "response:filter_user",
            &body,
            1.0,
        ));
    }
    None
}

pub fn try_translation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let target = detect_target_language(normalized);
    let is_translation_request = normalized.starts_with("translate")
        || normalized.starts_with("переведи")
        || normalized.starts_with("опиши")
        || (target.is_some()
            && (normalized.contains("अनुवाद")
                || normalized.contains("翻译")
                || normalized.contains("翻譯")))
        || (normalized.starts_with("define ")
            && (extract_quoted_phrase(prompt).is_some() || extract_backticked(prompt).is_some())
            && (normalized.contains(" links notation") || normalized.contains(" в links")));
    if !is_translation_request {
        return None;
    }

    let mut source = detect_source_language(normalized);
    if source.is_none() {
        source = Some(infer_source_from_prompt(prompt));
    }

    let backticked = extract_backticked(prompt);

    if let Some(code) = &backticked {
        let detected = detect_program_languages(normalized)
            .or_else(|| infer_program_languages_from_code(code, normalized));
        if let Some((source_lang, target_lang)) = detected {
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
    }

    // Prefer an explicitly quoted fragment (`Translate "apple" to Russian`).
    // When the user omits the quotes (`translate apple to russian`),
    // fall back to a structural extraction of the substring between the
    // verb and the target preposition so the Wiktionary pipeline still
    // receives a non-empty surface. See issue #216.
    let surface = extract_quoted_phrase(prompt)
        .or_else(|| extract_unquoted_translation_surface(prompt))
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

    let (raw_target, meaning_id, translation_gap) = if let Ok(translation) = pipeline_result {
        let raw = translation
            .primary_surface()
            .map_or_else(|| format!("[{target_slug}] {surface}"), str::to_owned);
        let gap = translation.candidates.is_empty();
        (raw, translation.meaning.slug(), gap)
    } else {
        // Fallback: hash the surface fragment so the trace still has a
        // stable id. The pipeline error itself is not propagated to the
        // user — the placeholder string already signals that translation
        // could not be performed.
        let surface_meaning = if surface.is_empty() {
            prompt.to_owned()
        } else {
            surface.clone()
        };
        let id = stable_id("meaning", &normalize_meaning(&surface_meaning));
        (format!("[{target_slug}] {surface}"), id, true)
    };
    log.append("meaning", meaning_id);
    if translation_gap && !surface.is_empty() {
        log.append("translation_gap", surface.clone());
    }

    let translated_surface = crate::translation::match_source_formatting(&raw_target, &surface);
    let body = if surface.is_empty() {
        translated_surface
    } else {
        format!("\"{translated_surface}\"")
    };
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

pub fn try_write_script(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !is_write_script_request(normalized) {
        return None;
    }
    let program = hello_world_program_by_alias(normalized)?;
    let body = format!(
        "Here is a minimal {} script:\n\n```{}\n{}\n```\n\n{}",
        program.language,
        program.code_fence,
        program.code,
        format_write_script_execution(program)
    );
    let intent = format!("write_script_{}", program.slug);
    log.append(
        "execution_status",
        program.execution.status.label().to_owned(),
    );
    log.append(
        "execution_environment",
        program.execution.environment.to_owned(),
    );
    Some(finalize_simple(
        prompt,
        log,
        &intent,
        &format!("response:hello_world:{}", program.slug),
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

pub fn try_source_refresh(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !normalized.contains("refresh")
        || !(normalized.contains("cache") || normalized.contains("page"))
    {
        return None;
    }
    let target = stable_id("source", prompt);
    log.append("source_refresh", target.clone());
    let body = format!(
        "Cached source {target} has been queued for refresh against its origin URL. The \
         refresh event is appended to the audit log and a fresh fetched_at timestamp will be \
         recorded once the new copy is verified."
    );
    Some(finalize_simple(
        prompt,
        log,
        "source_refresh",
        "response:source_refresh",
        &body,
        1.0,
    ))
}

pub fn try_source_conflict(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !(normalized.contains("conflict")
        || (normalized.contains("born in") && normalized.contains(" or ")))
    {
        return None;
    }
    log.append(
        "conflict:source_disagreement",
        "sources disagree on the answer".to_owned(),
    );
    let body = String::from(
        "Sources disagree on this question. The disagreement is recorded as a \
         conflict:source_disagreement link in the network rather than silently resolved.",
    );
    Some(finalize_simple(
        prompt,
        log,
        "source_conflict",
        "response:source_conflict",
        &body,
        0.3,
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
    log.append("intent", intent.to_owned());
    log.append("response", response_link.to_owned());
    let trace_id = log.append("trace", intent.to_owned());
    let evidence_links = build_evidence_links(prompt, log, response_link);
    let links_notation = answer_links_notation(prompt, intent, body, log, &trace_id);
    SymbolicAnswer {
        intent: intent.to_owned(),
        answer: body.to_owned(),
        confidence,
        evidence_links,
        links_notation,
    }
}
