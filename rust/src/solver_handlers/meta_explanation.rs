use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, Slot, localized_response};
use crate::skill_procedure::{CompiledProcedure, extract_compiled_procedure_artifact};

use super::finalize_simple;
use super::self_awareness::{
    SelfAwarenessRuntime, surface_label, surface_memory, surface_runtime, surface_web_search,
};

pub fn try_meta_explanation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_meta_explanation_with_runtime(prompt, normalized, log, SelfAwarenessRuntime::default())
}

pub fn try_meta_explanation_with_runtime(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: SelfAwarenessRuntime,
) -> Option<SymbolicAnswer> {
    let is_why_question = is_why_question(normalized);
    let is_how_you_work = is_how_you_work(normalized);
    let is_architecture_question = is_architecture_question(normalized);
    if !is_why_question && !is_how_you_work && !is_architecture_question {
        return None;
    }
    let language = meta_language(prompt, normalized);
    let mut intent = "meta_explanation";
    let body = if is_why_question {
        draft_comparison_from_history(log).map_or_else(
            || {
                let mut body = why_explanation_body(language);
                if let Some(procedure) = compiled_procedure_from_history(log) {
                    log.append("skill_compile:procedure", procedure.id.clone());
                    body.push_str(&cited_procedure_steps(&procedure, language));
                }
                body
            },
            |comparison| {
                intent = "draft_comparison_explanation";
                draft_comparison_explanation(&comparison, language)
            },
        )
    } else if is_architecture_question {
        architecture_explanation_body(language, runtime)
    } else {
        localized_response("meta_explanation", language).unwrap_or_default()
    };
    Some(finalize_simple(
        prompt,
        log,
        intent,
        if intent == "draft_comparison_explanation" {
            "response:draft_comparison_explanation"
        } else {
            "response:meta_explanation"
        },
        &body,
        1.0,
    ))
}

struct DraftComparison {
    winner_index: String,
    strategy: String,
    passed_tests: String,
    total_tests: String,
    smaller_percent: String,
}

fn draft_comparison_from_history(log: &EventLog) -> Option<DraftComparison> {
    let artifact = log
        .events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:assistant")
        .find_map(|event| {
            event
                .payload
                .contains("draft_comparison_artifact")
                .then_some(event.payload.as_str())
        })?;
    Some(DraftComparison {
        winner_index: artifact_field(artifact, "winner_index")?,
        strategy: artifact_field(artifact, "winner_strategy")?,
        passed_tests: artifact_field(artifact, "passed_tests")?,
        total_tests: artifact_field(artifact, "total_tests")?,
        smaller_percent: artifact_field(artifact, "smaller_percent")?,
    })
}

fn artifact_field(artifact: &str, field: &str) -> Option<String> {
    artifact.lines().find_map(|line| {
        let line = line.trim();
        let value = line.strip_prefix(field)?.trim();
        (!value.is_empty()).then(|| value.trim_matches('"').to_owned())
    })
}

fn draft_comparison_explanation(comparison: &DraftComparison, language: &str) -> String {
    let template = localized_response("draft_comparison_explanation", language).unwrap_or_default();
    [
        ("{winner_index}", comparison.winner_index.as_str()),
        ("{strategy}", comparison.strategy.as_str()),
        ("{passed_tests}", comparison.passed_tests.as_str()),
        ("{total_tests}", comparison.total_tests.as_str()),
        ("{smaller_percent}", comparison.smaller_percent.as_str()),
    ]
    .into_iter()
    .fold(template, |body, (placeholder, value)| {
        body.replace(placeholder, value)
    })
}

fn why_explanation_body(language: &str) -> String {
    localized_response("meta_explanation_why", language).unwrap_or_default()
}

/// The languages a `policy meta_explanation` entry of
/// `data/seed/handler-rules.lino` names (issue #918): which lexemes of the
/// rationale lead read as fronted leads, which languages need the
/// compositional cause-plus-prior-answer reading, and which carry the
/// compositional operating-principle phrasing.
fn policy_languages(key: &str) -> Vec<String> {
    crate::rule_interpreter::handler_policy("meta_explanation", key)
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

/// Recover the most recent persisted artifact from an earlier assistant turn.
///
/// The original user prose is deliberately not recompiled: the explanation cites
/// the exact integrity-checked program that was published and could be executed.
fn compiled_procedure_from_history(log: &EventLog) -> Option<CompiledProcedure> {
    log.events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:assistant")
        .find_map(|event| extract_compiled_procedure_artifact(&event.payload).ok())
}

/// Cite the compiled steps and the source sentence spans they were read from.
fn cited_procedure_steps(procedure: &CompiledProcedure, language: &str) -> String {
    let template =
        localized_response("compiled_procedure_explanation", language).unwrap_or_default();
    format!(
        "\n\n{}",
        template.replace("{steps}", &procedure.restate_steps())
    )
}

/// True when the prompt asks the assistant to justify its previous answer.
///
/// The English and Russian why-questions front the interrogative, so each
/// [`answer_rationale_lead`](seed::ROLE_ANSWER_RATIONALE_LEAD) surface is matched
/// directly — a [`Slot::Prefix`] form against the start of the prompt and a bare
/// form anywhere in it. The Hindi and Chinese why-questions are head-final, so
/// they are detected instead as a same-language pair of a
/// [`causal_interrogative`](seed::ROLE_CAUSAL_INTERROGATIVE) and a
/// [`prior_answer_reference`](seed::ROLE_PRIOR_ANSWER_REFERENCE); the Hindi and
/// Chinese rationale surfaces are inert completeness forms, skipped here by the
/// language filter. No question word is hardcoded in this function.
fn is_why_question(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    let lead_languages = policy_languages("rationale-lead-languages");
    for meaning in lexicon.meanings_with_role(seed::ROLE_ANSWER_RATIONALE_LEAD) {
        for lexeme in &meaning.lexemes {
            if !lead_languages.contains(&lexeme.language) {
                continue;
            }
            for form in &lexeme.words {
                let matched = match form.slot() {
                    Slot::Prefix => normalized
                        .strip_prefix(form.before_slot())
                        .is_some_and(addresses_assistant),
                    _ => normalized.contains(form.text.as_str()),
                };
                if matched {
                    return true;
                }
            }
        }
    }
    policy_languages("compositional-why-languages")
        .iter()
        .any(|language| {
            let names_cause = lexicon
                .words_for_role_in_languages(seed::ROLE_CAUSAL_INTERROGATIVE, &[language.as_str()])
                .iter()
                .any(|word| normalized.contains(word.as_str()));
            let names_prior_answer = lexicon
                .words_for_role_in_languages(
                    seed::ROLE_PRIOR_ANSWER_REFERENCE,
                    &[language.as_str()],
                )
                .iter()
                .any(|word| normalized.contains(word.as_str()));
            names_cause && names_prior_answer
        })
}

/// True when the rest of a fronted why-question addresses the assistant.
///
/// A prefix rationale lead ("why …", "почему …") only opens a question; "Why
/// does this fail: def f(x): …" asks about the user's code, not about the
/// assistant's answer. The question is about the assistant itself only when its
/// remainder carries an
/// [`assistant_self_reference`](seed::ROLE_ASSISTANT_SELF_REFERENCE) surface
/// ("you", "ты", "вы", …) as a whole word — "why did you …", "почему ты …".
/// The remainder is cut into letter/digit tokens first, so punctuation glued to
/// a word ("you?") does not hide it.
fn addresses_assistant(rest: &str) -> bool {
    let tokens = rest
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    seed::lexicon().mentions_role(seed::ROLE_ASSISTANT_SELF_REFERENCE, &tokens)
}

/// True when the prompt asks the assistant to explain how it works.
///
/// Most phrasings are complete clauses carried by
/// [`assistant_mechanism_inquiry`](seed::ROLE_ASSISTANT_MECHANISM_INQUIRY) and
/// matched as raw substrings. The Russian principle-of-operation phrasing
/// (принцип работы … тебя) is compositional, so it is recognised by requiring an
/// [`operating_principle`](seed::ROLE_OPERATING_PRINCIPLE) surface together with
/// an [`assistant_self_reference`](seed::ROLE_ASSISTANT_SELF_REFERENCE) surface,
/// both read in Russian only.
fn is_how_you_work(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    if lexicon.mentions_role_raw(seed::ROLE_ASSISTANT_MECHANISM_INQUIRY, normalized) {
        return true;
    }
    let languages = policy_languages("operating-principle-languages");
    let languages: Vec<&str> = languages.iter().map(String::as_str).collect();
    let names_principle = lexicon
        .words_for_role_in_languages(seed::ROLE_OPERATING_PRINCIPLE, &languages)
        .iter()
        .any(|word| normalized.contains(word.as_str()));
    let addresses_assistant = lexicon
        .words_for_role_in_languages(seed::ROLE_ASSISTANT_SELF_REFERENCE, &languages)
        .iter()
        .any(|word| normalized.contains(word.as_str()));
    names_principle && addresses_assistant
}

/// True when the prompt asks how the assistant itself is built rather than
/// requesting a task.
///
/// Decomposes exactly like the original two-list screen: the prompt must address
/// the assistant — carry an
/// [`assistant_self_reference`](seed::ROLE_ASSISTANT_SELF_REFERENCE) surface —
/// *and* name an [`architecture_concept`](seed::ROLE_ARCHITECTURE_CONCEPT) such
/// as a language model, neural network, or the project's local rules. Both are
/// matched as raw substrings across all four languages.
fn is_architecture_question(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role_raw(seed::ROLE_ASSISTANT_SELF_REFERENCE, normalized)
        && lexicon.mentions_role_raw(seed::ROLE_ARCHITECTURE_CONCEPT, normalized)
}

fn meta_language(prompt: &str, normalized: &str) -> &'static str {
    let lower = format!("{} {}", prompt.to_lowercase(), normalized);
    if meta_has_char_in_range(&lower, '\u{0400}', '\u{04ff}') {
        return "ru";
    }
    if meta_has_char_in_range(&lower, '\u{0900}', '\u{097f}') {
        return "hi";
    }
    if meta_has_char_in_range(&lower, '\u{4e00}', '\u{9fff}') {
        return "zh";
    }
    detect_language(prompt).slug()
}

fn meta_has_char_in_range(text: &str, start: char, end: char) -> bool {
    text.chars().any(|ch| (start..=end).contains(&ch))
}

fn architecture_explanation_body(language: &str, runtime: SelfAwarenessRuntime) -> String {
    seed::render_localized_once(
        "meta_explanation_architecture",
        language,
        &[
            ("surface_label", surface_label(runtime)),
            ("surface", runtime.surface.slug()),
            ("runtime", surface_runtime(runtime)),
            ("memory", surface_memory(runtime)),
            ("web_search", surface_web_search(runtime)),
        ],
    )
}
