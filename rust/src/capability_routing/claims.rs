//! Claim routing (issue #1175 R3): which structural evidence a handler's claim
//! needs, consulted before the handler runs.
//!
//! The `claim` rows of `data/seed/capability-routing.lino` name a handler (the
//! native dispatch name, and the browser worker's function as
//! `browser_handler`) and the evidence kinds any one of which admits it. The
//! router reads the prompt's structure through the evidence predicate each
//! kind names — the verb's object phrase, a parse as command arguments, a
//! repository subject — and a handler whose row admits on none of them is not
//! offered the prompt at all, so it cannot claim on a surface word. A handler
//! with no row is admitted as before; the rows are where the issue #1175
//! classes of misroute live, and new rows are added there, not in code.
//!
//! A row may also name a `refusal_event`: without its evidence the handler is
//! then admitted to the refusal lane only, where its answer stands only when
//! it recorded that event — a cued request without its operand keeps its
//! named refusal, and no answer is claimed on the cue alone.
//!
//! The dialogue kinds read the `prior_turn:*` events of the dialogue log
//! ([`claim_admission_in_dialogue`]); the composer, lookup and dialogue kinds
//! live in `claim_evidence.rs`.
//!
//! Mirrored by `claimRouteAdmits` in `js/worker/formal_ai_worker_dispatch.js`.

use std::sync::OnceLock;

use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;

/// One `claim` row: a handler and the evidence kinds that admit it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRow {
    pub handler: String,
    pub browser_handler: String,
    pub admits_on: Vec<String>,
    /// The event a handler records when it refuses by name. A row naming one
    /// admits the handler without its evidence to refuse only: the answer is
    /// kept when the handler recorded this event, and dropped otherwise.
    pub refusal_events: Vec<String>,
    pub because: String,
}

/// The evidence kinds the router can read, in the order the code knows them.
///
/// A row naming a kind outside this list is a seed error the unit suite
/// reports, never a silent admission.
pub const CLAIM_EVIDENCE_KINDS: &[&str] = &[
    "object_phrase_artifact",
    "approval_of_a_proposal",
    "shell_command_shape",
    "semantic_shell_task",
    "repository_subject",
    "supplied_page",
    "javascript_program",
    "incompatible_unit_pair",
    "fetch_url",
    "navigation_url",
    "calendar_date_signal",
    "code_artifact",
    "supplied_text",
    "stated_number",
    "calculation_expression",
    "currency_rate_basis",
    "investment_terms",
    "conversion_target_currency",
    "interval_bounds",
    "measured_quantity",
    "calendar_anchor",
    "function_under_test",
    "structured_document",
    // Issue #1175 R3 classes (b), (c) and (e): read by `claim_evidence.rs`.
    "dialogue_turn",
    "prior_user_request",
    "prior_reply",
    "prior_software_project",
    "prior_research_request",
    "prior_procedure",
    "coreference_antecedent",
    "prior_program",
    "prior_numeric_list",
    "list_items",
    "text_operation",
    "shell_command_operand",
    "composition_topic",
    "advice_topic",
    "cached_destination",
    "pattern_constraints",
    "table_reference",
    "filesystem_object",
    "function_spec",
    "script_language",
    "program_task",
    "document_format",
    "install_steps",
    "call_expression",
    "concept_subject",
    "definition_merge_term",
    "mechanism_subject",
    "procedure_task",
    "search_focus",
    "conversation_topic_subject",
    "marketplace_scope",
    "verifiable_spec",
    "legality_assessment",
    "assistant_addressee",
    "punctuation_only",
    "unbalanced_brackets",
    "memory_program_reading",
    "learnable_source",
    // Follow-up round of issue #1175 R3.
    "backticked_term",
    "name_assignment",
    "recall_query_term",
    "supplied_payload",
    "summary_topic",
    "brainstorm_category",
    "persona_or_topic",
    "document_operand",
    "translation_text",
    "triz_precedent",
    "algorithm_operation",
    "source_reference",
    "stated_claim",
    "assistant_subject",
    "memory_query_statement",
    "fact_subject",
];

/// Parse the `claim` rows of a capability-routing document.
#[must_use]
pub fn claim_rows_from(text: &str) -> Vec<ClaimRow> {
    let tree = parse_lino(text);
    let mut rows = Vec::new();
    for document in &tree.children {
        for record in &document.children {
            if record.name != "claim" {
                continue;
            }
            rows.push(ClaimRow {
                handler: record.find_child_value("handler").to_owned(),
                browser_handler: record.find_child_value("browser_handler").to_owned(),
                admits_on: record
                    .children
                    .iter()
                    .filter(|child| child.name == "admits_on" && !child.id.is_empty())
                    .map(|child| child.id.clone())
                    .collect(),
                refusal_events: record
                    .children
                    .iter()
                    .filter(|child| child.name == "refusal_event" && !child.id.is_empty())
                    .map(|child| child.id.clone())
                    .collect(),
                because: record.find_child_value("because").to_owned(),
            });
        }
    }
    rows
}

/// The shipped `claim` rows.
#[must_use]
pub fn claim_rows() -> &'static [ClaimRow] {
    static ROWS: OnceLock<Vec<ClaimRow>> = OnceLock::new();
    ROWS.get_or_init(|| claim_rows_from(super::CAPABILITY_ROUTING_LINO))
}

/// Whether the prompt carries evidence of `kind`; `None` for an unknown kind.
///
/// The dialogue kinds read an empty dialogue here; the dispatcher asks
/// [`claim_evidence_holds_in_dialogue`] with the turns before the prompt.
#[must_use]
pub fn claim_evidence_holds(kind: &str, prompt: &str, normalized: &str) -> Option<bool> {
    claim_evidence_holds_in_dialogue(kind, prompt, normalized, &EventLog::new())
}

/// Whether the prompt, or the dialogue before it, carries evidence of `kind`.
///
/// `dialogue` is the event log whose `prior_turn:*` events are the earlier
/// turns; `None` for an unknown kind.
#[must_use]
pub fn claim_evidence_holds_in_dialogue(
    kind: &str,
    prompt: &str,
    normalized: &str,
    dialogue: &EventLog,
) -> Option<bool> {
    prompt_evidence_holds(kind, prompt, normalized)
        .or_else(|| super::claim_evidence::class_evidence_holds(kind, prompt, normalized, dialogue))
}

/// The structural evidence kinds read from the prompt alone.
fn prompt_evidence_holds(kind: &str, prompt: &str, normalized: &str) -> Option<bool> {
    let canonical = crate::engine::normalize_prompt(prompt);
    let canonical = if canonical.is_empty() {
        normalized
    } else {
        canonical.as_str()
    };
    let lowered = prompt.to_lowercase();
    let holds = match kind {
        "object_phrase_artifact" => crate::solver_handlers::software_project_claims(canonical),
        "approval_of_a_proposal" => {
            crate::solver_handlers::software_project_approval_claims(canonical)
        }
        "shell_command_shape" => crate::solver_terminal::names_terminal_command(prompt),
        "semantic_shell_task" => {
            crate::agentic_coding::semantic_shell_command_for_task(prompt).is_some()
        }
        "repository_subject" => {
            let rules = crate::history_context::HistoryRules::load(None);
            !crate::history_context::lineage_subjects(prompt, &rules).is_empty()
                || crate::history_context::status_subject(prompt, &rules).is_some()
                || !crate::history_context::definition_subjects(prompt, &rules).is_empty()
        }
        "supplied_page" => crate::web_formalize::split_supplied_page(prompt).is_some(),
        "javascript_program" => crate::solver_helpers::extract_javascript_program(prompt).is_some(),
        "incompatible_unit_pair" => {
            crate::solver_handler_units::names_incompatible_unit_pair(normalized)
        }
        "fetch_url" => crate::solver_handlers::http_fetch_claims(prompt, normalized),
        "navigation_url" => crate::solver_handlers::url_navigation_claims(prompt, normalized),
        "calendar_date_signal" => crate::solver_handlers::calendar_claims(normalized),
        "code_artifact" => crate::solver_handlers::code_debugging::code_block(prompt).is_some(),
        "supplied_text" => {
            crate::solver_handlers::text_rewrite::free_text_payload(prompt).is_some()
        }
        // Issue #1175 R3, numeric group: each kind is the operand the handler's
        // own reader parses before it answers, read by that same reader.
        "stated_number" => {
            !crate::solver_handlers::numeric_list::parse_numbers(&lowered).is_empty()
        }
        "calculation_expression" => {
            !crate::calculation::calculation_expression_candidates(prompt).is_empty()
        }
        "currency_rate_basis" => {
            crate::solver_handlers::calculator_rate::asks_for_usd_rate_basis(canonical)
        }
        "investment_terms" => {
            crate::solver_handlers::compound_interest::parse_compound_interest_request(
                prompt, normalized,
            )
            .is_some()
        }
        "conversion_target_currency" => {
            crate::solver_handlers::compound_interest::target_currency(normalized, None).is_some()
        }
        "interval_bounds" => crate::number_constraints::extract_interval_bounds(
            &crate::engine::normalize_prompt(normalized),
            &prompt
                .chars()
                .flat_map(char::to_lowercase)
                .collect::<String>(),
        )
        .is_some(),
        "measured_quantity" => {
            !crate::solver_handlers::numeric_list::parse_numbers(&lowered).is_empty()
                && crate::solver_handlers::unit_conversion::unit_mentions(&lowered).len() == 2
        }
        "calendar_anchor" => {
            use crate::solver_handlers::calendar;
            calendar::detect_weekday(normalized).is_some()
                || calendar::month::detect_month(normalized).is_some()
                || calendar::date_weekday::stated_date(prompt, normalized).is_some()
                || calendar::mentions_current_day_question(normalized)
        }
        // Issue #1175 R3, refusal group: the operand each handler's own
        // reader extracts before it composes anything.
        "function_under_test" => crate::solver_handlers::names_function_under_test(prompt),
        "structured_document" => crate::solver_handlers::carries_structured_document(prompt),
        _ => return None,
    };
    Some(holds)
}

/// How the router offers a prompt to a handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimAdmission {
    /// The handler's evidence holds (or it has no row): any answer is kept.
    Full,
    /// The evidence is absent but the row names a refusal event: the handler
    /// may only refuse, so its answer is kept only when it recorded that
    /// event.
    RefusalOnly,
    /// The handler is not offered the prompt.
    Denied,
}

/// How the router offers `prompt` to `handler`, with no earlier turn.
#[must_use]
pub fn claim_admission(handler: &str, prompt: &str, normalized: &str) -> ClaimAdmission {
    claim_admission_in_dialogue(handler, prompt, normalized, &EventLog::new())
}

/// How the router offers `prompt` to `handler`, given the dialogue before it.
///
/// `dialogue` is the event log whose `prior_turn:*` events the dialogue
/// evidence kinds read.
#[must_use]
pub fn claim_admission_in_dialogue(
    handler: &str,
    prompt: &str,
    normalized: &str,
    dialogue: &EventLog,
) -> ClaimAdmission {
    let Some(row) = claim_rows().iter().find(|row| row.handler == handler) else {
        return ClaimAdmission::Full;
    };
    if row.admits_on.iter().any(|kind| {
        claim_evidence_holds_in_dialogue(kind, prompt, normalized, dialogue) == Some(true)
    }) {
        ClaimAdmission::Full
    } else if row.refusal_events.is_empty() {
        ClaimAdmission::Denied
    } else {
        ClaimAdmission::RefusalOnly
    }
}

/// Whether the router offers `prompt` to `handler` at all (fully or to the
/// refusal lane).
///
/// A handler with no `claim` row is admitted; a handler with one is admitted
/// when any evidence kind its row lists holds, or to refuse only when its row
/// names a refusal event.
#[must_use]
pub fn claim_admitted(handler: &str, prompt: &str, normalized: &str) -> bool {
    claim_admission(handler, prompt, normalized) != ClaimAdmission::Denied
}

/// Whether a refusal-lane answer may stand: the handler recorded one of its
/// row's refusal events among `events` (the events it appended).
#[must_use]
pub fn refusal_recorded(handler: &str, events: &[crate::event_log::Event]) -> bool {
    claim_rows()
        .iter()
        .find(|row| row.handler == handler)
        .is_some_and(|row| {
            events
                .iter()
                .any(|event| row.refusal_events.iter().any(|kind| *kind == event.kind))
        })
}
