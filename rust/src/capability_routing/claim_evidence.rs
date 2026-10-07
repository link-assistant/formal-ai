//! Claim evidence for the dialogue, composer and lookup classes (issue #1175 R3).
//!
//! Every kind here is the reader the handler itself runs before it answers,
//! so a `claim` row admits exactly the prompts the handler could answer from
//! structure: (b) the earlier turn a follow-up continues, read from the
//! `prior_turn:*` events of the dialogue log, (c) the specification a composer
//! builds from (a composer without one is admitted to its refusal lane only),
//! and (e) the subject a lookup resolves or the shape a policy answers.
//!
//! Mirrored by `CLASS_CLAIM_EVIDENCE` in
//! `js/worker/formal_ai_worker_claim_evidence.js`.

use crate::event_log::EventLog;
use crate::solver::ConversationTurn;
use crate::solver_handlers as handlers;
use crate::solver_helpers::{last_assistant_turn, last_user_turn};

/// Whether the prompt, or the dialogue before it, carries evidence of `kind`.
///
/// `None` for a kind this module does not read. `dialogue` is the event log
/// whose `prior_turn:*` events are the earlier turns.
#[must_use]
pub fn class_evidence_holds(
    kind: &str,
    prompt: &str,
    normalized: &str,
    dialogue: &EventLog,
) -> Option<bool> {
    let holds = match kind {
        // (b) The earlier turn the follow-up continues.
        "dialogue_turn" => !dialogue_turns(dialogue).is_empty(),
        "prior_user_request" => last_user_turn(dialogue).is_some(),
        "prior_reply" => last_assistant_turn(dialogue).is_some(),
        "prior_software_project" => handlers::continues_software_project(dialogue),
        "prior_research_request" => handlers::follows_research_request(dialogue),
        "prior_procedure" => continues_procedure(dialogue),
        "coreference_antecedent" => handlers::names_coreference_antecedent(dialogue),
        "prior_numeric_list" => {
            handlers::numeric_list::continues_numeric_list(&dialogue_turns(dialogue))
        }
        "list_items" => handlers::numeric_list::names_list_items(prompt),
        "text_operation" => {
            handlers::parses_text_operation(prompt, normalized, &dialogue_turns(dialogue))
        }
        "shell_command_operand" => handlers::names_shell_command(prompt),
        // (c) The specification a composer builds from.
        "composition_topic" => {
            !handlers::topic_words(normalized, crate::language::detect(prompt).slug()).is_empty()
        }
        "advice_topic" => handlers::names_advice_topic(normalized),
        "cached_destination" => handlers::names_cached_destination(normalized),
        "pattern_constraints" => handlers::names_pattern_constraints(prompt),
        "table_reference" => handlers::names_query_table(prompt),
        "filesystem_object" => handlers::names_filesystem_object(prompt, normalized),
        "function_spec" => handlers::looks_like_python_function_request(prompt, normalized),
        "script_language" => crate::engine::hello_world_program_by_alias(normalized).is_some(),
        "document_format" => handlers::names_document_format(normalized),
        "install_steps" => handlers::carries_install_steps(prompt, normalized),
        "call_expression" => names_call_expression(prompt),
        // (e) The subject a lookup resolves, or the shape a policy answers.
        "concept_subject" => {
            crate::concepts::extract_concept_query(prompt).is_some()
                || handlers::definition_term(prompt).is_some()
        }
        "definition_merge_term" => {
            crate::definition_merge::names_definition_merge_term(prompt, normalized)
        }
        "mechanism_subject" => crate::solver_handler_how::names_mechanism_subject(prompt),
        "procedure_task" => crate::solver_handler_how::procedural_how_to_task(normalized).is_some(),
        "search_focus" => handlers::detect_web_search_query(prompt).is_some(),
        "conversation_topic_subject" => handlers::names_conversation_topic(prompt, normalized),
        "learnable_source" => crate::seed::learning_sources()
            .match_directive(normalized)
            .is_some(),
        "marketplace_scope" => handlers::names_marketplace(prompt, normalized),
        "verifiable_spec" => names_verifiable_spec(prompt, normalized),
        "legality_assessment" => crate::legality_warning::assess(prompt, normalized).is_some(),
        "assistant_addressee" => addresses_assistant(normalized),
        "punctuation_only" => carries_no_word(prompt),
        "unbalanced_brackets" => unbalanced_brackets(prompt),
        // The program-writing and memory-program rows are browser-only: the
        // native solver never offers those handlers a prompt.
        "prior_program" | "program_task" | "memory_program_reading" => false,
        _ => return None,
    };
    Some(holds)
}

/// The earlier turns the dialogue log recorded, oldest first.
fn dialogue_turns(dialogue: &EventLog) -> Vec<ConversationTurn> {
    dialogue
        .events()
        .iter()
        .filter_map(|event| match event.kind.as_str() {
            "prior_turn:user" => Some(ConversationTurn::user(event.payload.clone())),
            "prior_turn:assistant" => Some(ConversationTurn::assistant(event.payload.clone())),
            _ => None,
        })
        .collect()
}

/// Whether the earlier exchange was an answered how-to request whose task re-parses.
fn continues_procedure(dialogue: &EventLog) -> bool {
    last_assistant_turn(dialogue).is_some()
        && last_user_turn(dialogue).is_some_and(|user| {
            crate::solver_handler_how::procedural_how_to_task(&crate::engine::normalize_prompt(
                user,
            ))
            .is_some()
        })
}

/// Whether the prompt carries a call expression: an identifier followed by `(`.
fn names_call_expression(prompt: &str) -> bool {
    let chars: Vec<char> = prompt.chars().collect();
    chars.iter().enumerate().any(|(index, ch)| {
        *ch == '('
            && chars[..index]
                .iter()
                .rev()
                .find(|before| !before.is_whitespace())
                .is_some_and(|before| before.is_alphanumeric() || *before == '_')
    })
}

/// Whether the verifiable-task reader formalizes the request, or its pattern arm reads a run.
fn names_verifiable_spec(prompt: &str, normalized: &str) -> bool {
    crate::verifiable_task::recognise_verifiable(prompt).is_some()
        || handlers::try_pattern_inference(prompt, normalized, &mut EventLog::new()).is_some()
}

/// The words of `text`: whitespace-separated, each trimmed of edge punctuation.
fn words(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .map(|word| word.trim_matches(|ch: char| !ch.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect()
}

/// Whether the prompt addresses the assistant.
///
/// A word is an `assistant_self_reference` surface, or that surface plus an
/// inflectional ending of at most three letters (`твоё`, `तुमने`); unspaced
/// scripts and multi-word surfaces match as substrings.
fn addresses_assistant(normalized: &str) -> bool {
    let text = normalized.to_lowercase();
    let words = words(&text);
    crate::seed::lexicon()
        .words_for_role(crate::seed::ROLE_ASSISTANT_SELF_REFERENCE)
        .iter()
        .map(|surface| surface.to_lowercase())
        .any(|surface| {
            if surface.contains(char::is_whitespace) || surface.chars().any(is_unspaced_script) {
                text.contains(&surface)
            } else {
                words.iter().any(|word| {
                    word.strip_prefix(surface.as_str())
                        .is_some_and(|ending| ending.chars().count() <= 3)
                })
            }
        })
}

/// Whether `ch` belongs to a script written without spaces between words.
const fn is_unspaced_script(ch: char) -> bool {
    matches!(ch, '\u{3400}'..='\u{9fff}')
}

/// Whether the prompt is non-empty and carries no letter or digit at all.
fn carries_no_word(prompt: &str) -> bool {
    let trimmed = prompt.trim();
    !trimmed.is_empty() && !trimmed.chars().any(char::is_alphanumeric)
}

/// Whether the round brackets of the prompt fail to balance.
fn unbalanced_brackets(prompt: &str) -> bool {
    let mut depth = 0_i64;
    for ch in prompt.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return true;
        }
    }
    depth != 0
}

/// The question without a trailing sentence that only shapes the answer (issue #1173 R3).
///
/// "What is Rust? Explain briefly." reads as "What is Rust?": the last of
/// several sentences is a shaping directive when it has at most three words,
/// one of them a `rule_brief_request` manner surface, and so names no operand
/// of its own. The concept reader reads through it; twin of
/// `withoutAnswerShapeDirective` in the browser worker.
#[must_use]
pub fn without_answer_shape_directive(prompt: &str) -> &str {
    let text = prompt.trim();
    let is_end = |ch: char| matches!(ch, '.' | '!' | '?' | '。' | '！' | '？');
    let body = text.trim_end_matches(is_end);
    let Some(boundary) = body.rfind(is_end) else {
        return text;
    };
    let head = &text[..boundary + body[boundary..].chars().next().map_or(1, char::len_utf8)];
    let last = text[head.len()..].trim().to_lowercase();
    let last_words = words(&last);
    let shaping = !last_words.is_empty()
        && last_words.len() <= 3
        && crate::seed::lexicon()
            .words_for_role(crate::seed::ROLE_RULE_BRIEF_REQUEST)
            .iter()
            .map(|surface| surface.to_lowercase())
            .any(|surface| {
                if surface.contains(char::is_whitespace) || surface.chars().any(is_unspaced_script)
                {
                    last.contains(&surface)
                } else {
                    last_words.contains(&surface.as_str())
                }
            });
    if shaping {
        // Keep the run of sentence terminators that closes the question.
        let mut end = head.len();
        for ch in text[end..].chars() {
            if !is_end(ch) {
                break;
            }
            end += ch.len_utf8();
        }
        text[..end].trim()
    } else {
        text
    }
}
