//! Contradiction reporting over the shared evidence-weighted requirement store.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::Language;
use crate::seed::response_for;
use crate::solver::{ConversationRole, ConversationTurn};
use crate::solver_handlers::finalize_simple;
use crate::statement_audit::{audit_corpus, AuditConfig, RepositoryCorpus, RepositoryDocument};

const RESPONSE_INTENT: &str = "requirement_contradiction";
const PRIOR_PLACEHOLDER: &str = "{prior}";
const CURRENT_PLACEHOLDER: &str = "{current}";

/// Detect a contradiction between the current prompt and retained requirements.
///
/// The conversational surface is an adapter over the same corpus, claim,
/// relative-evidence, probability-ranking, and associative-learning pipeline
/// used by repository audits.
pub fn detect_and_report(
    prompt: &str,
    language: Language,
    history: &[ConversationTurn],
    temperature: f32,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let mut documents = history
        .iter()
        .enumerate()
        .filter(|(_, turn)| turn.role == ConversationRole::User)
        .map(|(index, turn)| {
            RepositoryDocument::new(format!("conversation/{index}.md"), &turn.content)
        })
        .collect::<Vec<_>>();
    let current_path = "conversation/current.md";
    documents.push(RepositoryDocument::new(current_path, prompt));

    let audit = audit_corpus(
        &RepositoryCorpus::from_documents(documents),
        &[],
        AuditConfig {
            temperature,
            ..AuditConfig::default()
        },
    );
    let current = audit
        .statements
        .iter()
        .find(|statement| statement.location.path == current_path)?;
    let contradiction = audit
        .contradictions
        .iter()
        .find(|contradiction| contradiction.statement_ids.contains(&current.id))?;
    let prior = contradiction
        .statement_ids
        .iter()
        .filter(|id| *id != &current.id)
        .find_map(|id| {
            audit
                .statements
                .iter()
                .find(|statement| &statement.id == id)
        })?;

    log.append(
        RESPONSE_INTENT,
        [
            format!("subject={}", contradiction.subject),
            format!("statement_a={}", prior.id),
            format!("weight_a={:.6}", prior.relative_weight),
            format!("statement_b={}", current.id),
            format!("weight_b={:.6}", current.relative_weight),
        ]
        .join(" "),
    );
    log.append(
        "policy:add_only_history",
        "retraction_appends_superseding_event".to_owned(),
    );

    let template = response_for(
        RESPONSE_INTENT,
        match language {
            Language::Unknown => Language::English.slug(),
            known => known.slug(),
        },
    )?;
    let body = render_template(
        &template,
        &[
            (PRIOR_PLACEHOLDER, prior.text.as_str()),
            (CURRENT_PLACEHOLDER, current.text.as_str()),
            ("{subject}", contradiction.subject.as_str()),
            ("{prior_weight}", &format!("{:.6}", prior.relative_weight)),
            (
                "{current_weight}",
                &format!("{:.6}", current.relative_weight),
            ),
        ],
    );
    Some(finalize_simple(
        prompt,
        log,
        RESPONSE_INTENT,
        "response:requirement_contradiction",
        &body,
        0.3,
    ))
}

fn render_template(template: &str, values: &[(&str, &str)]) -> String {
    values
        .iter()
        .fold(template.to_owned(), |rendered, (key, value)| {
            rendered.replace(key, value)
        })
}
