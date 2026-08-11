use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::link_store::memory_events_to_link_records;
use crate::memory::{MemoryEvent, MemoryStore};
use crate::memory_program::{MemoryProgramAuthorization, MemoryProgramLimits};
use crate::seed;
use crate::solver::ConversationTurn;
use crate::solver_handlers::{
    answer_memory_recall, execute_memory_query_with_options, finalize_simple, is_exact_memory_query,
};
use std::collections::BTreeMap;

const REQUEST_HISTORY_CONVERSATION_ID: &str = "request_history";

pub fn answer_from_memory_if_requested(
    prompt: &str,
    history: &[ConversationTurn],
    memory_events: &[MemoryEvent],
) -> Option<SymbolicAnswer> {
    if let Some(answer) = answer_memory_inspection(prompt, history, memory_events) {
        return Some(answer);
    }
    let events = memory_events_with_request_history(memory_events, history);
    if is_exact_memory_query(prompt) {
        // Protocol functions receive an immutable event slice. Execute exact
        // reads over an isolated projection and default mutations to read-only;
        // native/browser callers with explicit authorization retain CRUD.
        let mut store = MemoryStore::from_events(events);
        return execute_memory_query_with_options(
            prompt,
            &mut store,
            Some(REQUEST_HISTORY_CONVERSATION_ID),
            MemoryProgramLimits::default(),
            MemoryProgramAuthorization::ReadOnly,
        )
        .map(|execution| execution.answer);
    }
    if memory_events.is_empty() {
        return None;
    }
    answer_memory_recall(prompt, &events, Some(REQUEST_HISTORY_CONVERSATION_ID))
}

fn answer_memory_inspection(
    prompt: &str,
    history: &[ConversationTurn],
    memory_events: &[MemoryEvent],
) -> Option<SymbolicAnswer> {
    let normalized = crate::engine::normalize_prompt(prompt);
    let previous_root_query = history.iter().rev().any(|turn| {
        turn.role == crate::solver::ConversationRole::User
            && is_root_links_query(&crate::engine::normalize_prompt(&turn.content))
    });
    let lexicon = seed::lexicon();
    let is_associative_correction = lexicon
        .mentions_role(seed::ROLE_MEMORY_RETRIEVAL_CORRECTION, &normalized)
        && previous_root_query;
    let records = memory_events_to_link_records(memory_events);
    let language = detect_language(prompt).slug();
    let (intent, body) = if lexicon.mentions_role(seed::ROLE_MEMORY_LINK_COUNT_QUERY, &normalized) {
        let link_count: usize = records.iter().map(|record| record.links.len()).sum();
        (
            "memory_link_count",
            seed::render_response(
                "memory_link_count",
                language,
                &[
                    ("records", &records.len().to_string()),
                    ("links", &link_count.to_string()),
                ],
            )?,
        )
    } else if lexicon.mentions_role(seed::ROLE_MEMORY_INVENTORY_QUERY, &normalized) {
        let mut kinds = BTreeMap::<String, usize>::new();
        let mut conversations = BTreeMap::<String, usize>::new();
        for event in memory_events {
            *kinds
                .entry(
                    event
                        .kind
                        .clone()
                        .unwrap_or_else(|| String::from("memory_event")),
                )
                .or_default() += 1;
            if let Some(conversation_id) = &event.conversation_id {
                *conversations.entry(conversation_id.clone()).or_default() += 1;
            }
        }
        let kinds = render_counts(&kinds, language)?;
        let conversations = render_counts(&conversations, language)?;
        (
            "memory_inventory",
            seed::render_response(
                "memory_inventory",
                language,
                &[
                    ("records", &memory_events.len().to_string()),
                    ("kinds", &kinds),
                    ("conversations", &conversations),
                ],
            )?,
        )
    } else if is_root_links_query(&normalized) || is_associative_correction {
        let rendered = records
            .iter()
            .map(|record| {
                format!(
                    "- (({}: {} {}))",
                    record.stable_id, record.stable_id, record.source_id
                )
            })
            .collect::<Vec<_>>();
        let listing = if rendered.is_empty() {
            seed::localized_response("memory_root_links_empty", language)?
        } else {
            rendered.join("\n")
        };
        (
            "memory_root_links",
            seed::render_response("memory_root_links", language, &[("listing", &listing)])?,
        )
    } else {
        return None;
    };
    let mut log = EventLog::new();
    log.append("impulse", prompt.to_owned());
    log.append("memory:inspect", intent.to_owned());
    Some(finalize_simple(
        prompt,
        &mut log,
        intent,
        &format!("response:{intent}"),
        &body,
        1.0,
    ))
}

fn is_root_links_query(normalized: &str) -> bool {
    seed::lexicon().mentions_role(seed::ROLE_MEMORY_ROOT_LINKS_QUERY, normalized)
}

fn render_counts(counts: &BTreeMap<String, usize>, language: &str) -> Option<String> {
    if counts.is_empty() {
        return seed::localized_response("memory_inventory_empty", language);
    }
    counts
        .iter()
        .map(|(name, count)| {
            let count = count.to_string();
            seed::render_response(
                "memory_inventory_item",
                language,
                &[("name", name), ("count", &count)],
            )
        })
        .collect::<Option<Vec<_>>>()
        .map(|items| items.join(", "))
}

fn memory_events_with_request_history(
    memory_events: &[MemoryEvent],
    history: &[ConversationTurn],
) -> Vec<MemoryEvent> {
    let mut events = Vec::with_capacity(memory_events.len() + history.len());
    events.extend_from_slice(memory_events);
    for (index, turn) in history.iter().enumerate() {
        events.push(MemoryEvent {
            id: format!("request-history-{index}"),
            kind: Some(String::from("message")),
            role: Some(turn.role.slug().to_owned()),
            content: Some(turn.content.clone()),
            conversation_id: Some(String::from(REQUEST_HISTORY_CONVERSATION_ID)),
            conversation_title: Some(String::from("Current request")),
            ..MemoryEvent::default()
        });
    }
    events
}
