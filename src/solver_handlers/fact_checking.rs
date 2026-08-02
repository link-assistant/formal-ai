//! User-facing current-dialogue fact checking (issue #845).
//!
//! The core verifier lives in [`crate::fact_checking`]. This handler connects it
//! to the conversation-maintained [`DialogueWorldModel`] without widening the
//! default boundary: only prior user statements in the current dialogue are
//! replayed, and the fact-check request itself is not added as a statement.
//! Recognition and response prose live in link data for all supported
//! languages.

use std::fmt::Write as _;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::fact_checking::{AuditScope, FactChecker, StatementVerification};
use crate::language::detect as detect_language;
use crate::seed;
use crate::solver::{ConversationTurn, SolverConfig};
use crate::solver_handlers::finalize_simple;
use crate::world_model_dialog::DialogueWorldModel;

const COUNT_PLACEHOLDER: &str = "{count}";
const FORMAL_SYSTEM_PLACEHOLDER: &str = "{formal_system}";
const SUMMARY_PLACEHOLDER: &str = "{summary}";
const STATEMENT_PLACEHOLDER: &str = "{statement}";
const PROBABILITY_PLACEHOLDER: &str = "{probability}";
const BASIS_PLACEHOLDER: &str = "{basis}";
const COUNTEREXAMPLE_PLACEHOLDER: &str = "{counterexample}";

/// Audit every declarative user statement in the current dialogue.
pub fn try_fact_checking(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    history: &[ConversationTurn],
    config: SolverConfig,
) -> Option<SymbolicAnswer> {
    if !seed::lexicon().mentions_role_raw(seed::ROLE_FACT_CHECK_CURRENT_DIALOGUE_QUERY, normalized)
    {
        return None;
    }

    let language = detect_language(prompt);
    let mut dialogue = DialogueWorldModel::from_turns(history);
    let audit = FactChecker::from_solver_config(config)
        .audit_world_model(&mut dialogue.model, AuditScope::CurrentDialogue, None)
        .ok()?;

    let summary = if audit.statements.is_empty() {
        localized_template("fact_check_no_statements", language.slug())
    } else {
        audit
            .statements
            .iter()
            .map(|statement| render_statement(statement, language.slug()))
            .collect::<Vec<_>>()
            .join("\n")
    };
    for statement in &audit.statements {
        let mut detail = String::new();
        let _ = write!(
            detail,
            "{} {} {}",
            statement.statement_id,
            statement.probability.to_decimal_string(),
            statement.probability_basis.slug()
        );
        log.append("fact_check:statement", detail);
    }
    log.append("fact_check:audit", audit.links_notation());
    log.append("fact_check:scope", "current_dialogue");
    log.append("fact_check:formal_system", audit.formal_system_id.clone());

    let body = localized_template("fact_check_current_dialogue", language.slug())
        .replace(COUNT_PLACEHOLDER, &audit.statements.len().to_string())
        .replace(FORMAL_SYSTEM_PLACEHOLDER, &audit.formal_system_name)
        .replace(SUMMARY_PLACEHOLDER, &summary);

    Some(finalize_simple(
        prompt,
        log,
        "fact_check_current_dialogue",
        "response:fact_check_current_dialogue",
        &body,
        1.0,
    ))
}

fn render_statement(statement: &StatementVerification, language: &str) -> String {
    let intent = if statement.counterexample.is_some() {
        "fact_check_statement_counterexample"
    } else {
        "fact_check_statement"
    };
    localized_template(intent, language)
        .replace(STATEMENT_PLACEHOLDER, &statement.text)
        .replace(
            PROBABILITY_PLACEHOLDER,
            &statement.probability.to_decimal_string(),
        )
        .replace(BASIS_PLACEHOLDER, statement.probability_basis.slug())
        .replace(
            COUNTEREXAMPLE_PLACEHOLDER,
            statement.counterexample.as_deref().unwrap_or_default(),
        )
}

fn localized_template(intent: &str, language: &str) -> String {
    seed::localized_response(intent, language).unwrap_or_else(|| String::from(SUMMARY_PLACEHOLDER))
}
