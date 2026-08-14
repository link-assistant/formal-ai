//! The handler half of an intent's `relevants` list: which handlers a prompt
//! promotes ahead of the seed-declared precedence table.
//!
//! Split out of `intent_formalization.rs` (issue #932, following the earlier
//! `write_program_request` split) so the promotion table can carry the reason
//! each entry exists without pushing the parent module past its reviewed size.

use crate::cue_lexicon;
use crate::seed;

use super::{
    contains_term_for_relevants, looks_arithmetic, looks_like_latest_news_search,
    looks_like_program_synthesis, looks_like_records_information_search,
    looks_like_single_concept_lookup, looks_like_text_manipulation, push_unique,
    requested_write_program_parameters,
};

/// Append a `handler:<name>` relevant for every handler the prompt promotes.
///
/// [`crate::method_registry::MethodRegistry::ordered_method_names_for_relevants`]
/// hoists a promoted handler ahead of the *whole* `handler-precedence.lino`
/// table, so a gate here outranks the declared order. Entries are therefore
/// listed in the order the seed declares: promoting one reading of a prompt
/// without promoting the higher-ranked reading it competes with would silently
/// invert the table.
pub(super) fn append_prompt_relevants(prompt: &str, normalized: &str, relevants: &mut Vec<String>) {
    let lower_prompt = prompt.to_ascii_lowercase();
    let operation_view = seed::operation_vocabulary().canonicalized_prompt(normalized);
    let handlers = [
        (
            "handler:conversation_control",
            crate::conversation_control::is_conversation_control_prompt(prompt),
        ),
        (
            "handler:execution_failure",
            cue_lexicon::matches("execution_failure_prompt", &lower_prompt)
                || cue_lexicon::matches("execution_failure_normalized", normalized),
        ),
        ("handler:arithmetic", looks_arithmetic(prompt, normalized)),
        (
            "handler:web_search",
            cue_lexicon::matches("web_search", normalized)
                || looks_like_latest_news_search(normalized)
                || looks_like_records_information_search(normalized),
        ),
        // Issue #847: a question *about* a task — split it, is it atomic, what
        // is the first step — must reach decomposition rather than the how-to
        // or text-manipulation readings of the same verbs, so it is promoted
        // ahead of both.
        (
            "handler:task_decomposition",
            crate::solver_handlers::looks_like_task_decomposition(normalized),
        ),
        (
            "handler:procedural_how_to",
            cue_lexicon::matches("procedural_how_to", normalized)
                || crate::solver_handler_how::looks_like_procedural_how_to(normalized),
        ),
        (
            "handler:proof_request",
            cue_lexicon::matches("proof_request", normalized),
        ),
        (
            "handler:fact_checking",
            crate::seed::lexicon().mentions_role_raw(
                crate::seed::ROLE_FACT_CHECK_CURRENT_DIALOGUE_QUERY,
                normalized,
            ),
        ),
        (
            "handler:world_state",
            cue_lexicon::matches(crate::world_model_atoms::QUERY_CUES, normalized),
        ),
        // Issue #932: a guide conversion ("convert this README.md installation
        // guide into a sh script") names its own steps — "create the project",
        // "build the project" — and quotes the commands they run, so the same
        // words also fire the write-script and software-project cues below.
        // Promoting only those would hoist them ahead of the whole precedence
        // table and defeat `handler-precedence.lino`, which ranks
        // `installation_conversion` above both (issue #423). The conversion
        // handler still declines when no install steps are recoverable, and
        // dispatch then continues down the table as before.
        (
            "handler:installation_conversion",
            crate::solver_handlers::is_install_conversion_request(normalized),
        ),
        (
            "handler:write_script",
            cue_lexicon::matches("write_script", normalized),
        ),
        (
            "handler:write_program",
            requested_write_program_parameters(prompt, normalized).is_some(),
        ),
        (
            "handler:program_synthesis",
            looks_like_program_synthesis(&operation_view),
        ),
        (
            "handler:text_manipulation",
            looks_like_text_manipulation(&operation_view),
        ),
        (
            "handler:software_project",
            cue_lexicon::matches("software_project", normalized),
        ),
        (
            "handler:meta_explanation",
            seed::lexicon().mentions_role_raw(seed::ROLE_ASSISTANT_MECHANISM_INQUIRY, normalized),
        ),
        // Issue #531: a concrete "what is the pattern in <grid/sequence>" request
        // carries both pattern-inference intent and a parseable run of atoms. It
        // must rank ahead of `concept_lookup` (which the shared "what is …" cue
        // would otherwise claim) so the data is analysed structurally instead of
        // answered as a dictionary definition. The gate mirrors the handler, so a
        // bare "what is a pattern?" stays with the concept lookup.
        (
            "handler:pattern_inference",
            crate::solver_handlers::looks_like_pattern_inference(prompt),
        ),
        (
            "handler:concept_lookup",
            cue_lexicon::matches("concept_lookup", normalized)
                || looks_like_single_concept_lookup(prompt),
        ),
        ("handler:calendar_create_event", {
            let lex = seed::lexicon();
            let has_day_ref = lex
                .words_for_role(seed::ROLE_CALENDAR_DAY_REFERENCE)
                .iter()
                .any(|w| contains_term_for_relevants(normalized, w));
            let has_digit = normalized.chars().any(|c| c.is_ascii_digit());
            let has_date_signal = has_day_ref || has_digit;
            let has_schedule = lex
                .words_for_role(seed::ROLE_CALENDAR_SCHEDULE_ACTION)
                .iter()
                .any(|w| contains_term_for_relevants(normalized, w))
                || lex
                    .words_for_role(seed::ROLE_CALENDAR_EVENT)
                    .iter()
                    .any(|w| contains_term_for_relevants(normalized, w));
            // Fallback cues for classic phrasing (RU from the bug report + EN "schedule ... 18th").
            // The verb cues use token-boundary matching (cue-lexicon `match "token"`) so that
            // unrelated text such as "free-programming-books" (the word "book" embedded in
            // "books") cannot masquerade as a schedule verb. The "число"/"в "/":" glue stays in
            // code as a structural composite: a date marker conjoined with a preposition or a
            // time separator.
            let fallback_cue = cue_lexicon::matches("calendar_fallback_verbs", normalized)
                || (cue_lexicon::matches("calendar_ru_date_marker", normalized)
                    && (normalized.contains("в ") || normalized.contains(':')))
                || (has_digit && cue_lexicon::matches("calendar_digit_actions", normalized));
            has_date_signal && (has_schedule || fallback_cue)
        }),
    ];
    for (handler, matches) in handlers {
        if matches {
            push_unique(relevants, String::from(handler));
        }
    }
}
