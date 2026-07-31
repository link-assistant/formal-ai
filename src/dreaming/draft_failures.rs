//! Mining the durable `draft_failure` records a candidate-solution portfolio
//! leaves behind (issue #704, requirement 6).
//!
//! A losing draft is not waste. [`crate::draft_portfolio`] records one
//! structured `draft_failure` per failing slot — which strategy was spent, how
//! far it got against the generated tests, and how much of its bounded retry
//! budget it consumed. [`EventLog::append_to_link_store`] persists those events
//! as memory links, so the dreaming loop reads them back here with the same
//! projection it uses for every other kind of memory.
//!
//! What the loop derives is a *lesson*, expressed as a language-neutral slug so
//! the conclusion stays data (no English sentence is minted in Rust):
//!
//! - `deprioritize_strategy` — the strategy passed nothing and burned its whole
//!   retry budget: spending a slot on it for this problem shape is wasted work.
//! - `extend_strategy` — the strategy passed some tests but not all: it is close,
//!   and the gap is a concrete thing to learn.
//! - `raise_draft_count` — the strategy stopped early: more slots, not a
//!   different strategy, is the cheaper fix.
//!
//! [`EventLog::append_to_link_store`]: crate::event_log::EventLog::append_to_link_store

use std::collections::BTreeMap;

use crate::memory::MemoryEvent;

/// What one draft strategy's repeated failures suggest the system should learn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftFailureLesson {
    /// The strategy the failing slots were spent on.
    pub strategy: String,
    /// How many recorded failures this lesson generalizes.
    pub occurrences: usize,
    /// Generated tests the best attempt passed, summed across occurrences.
    pub passed_tests: usize,
    /// Generated tests that existed, summed across occurrences.
    pub total_tests: usize,
    /// Retry attempts spent, summed across occurrences.
    pub attempts: u32,
    /// Did every occurrence exhaust its bounded retry budget?
    pub exhausted_retry_budget: bool,
    /// Language-neutral conclusion slug (see the module docs).
    pub lesson: String,
    /// Memory events this lesson was derived from.
    pub source_event_ids: Vec<String>,
}

/// Read one field out of a `draft_failure` Links Notation record.
fn field<'a>(record: &'a str, key: &str) -> Option<&'a str> {
    record.lines().find_map(|line| {
        line.trim()
            .strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(' '))
            .map(|value| value.trim().trim_matches('"'))
    })
}

/// Is this memory event a persisted portfolio draft failure?
fn is_draft_failure(event: &MemoryEvent) -> Option<&str> {
    let content = event.content.as_deref()?;
    let tagged = field(content, "record_type") == Some("draft_failure");
    let by_kind = event.kind.as_deref() == Some("draft_failure");
    (tagged || by_kind).then_some(content)
}

/// Aggregate the durable draft failures in `events` into one lesson per
/// strategy, most-failing first.
///
/// Aggregation is what makes this learning rather than logging: a single failed
/// draft is noise, the same strategy failing the same way repeatedly is a
/// property of the problem shape that the portfolio's next round can act on.
#[must_use]
pub fn draft_failure_lessons(events: &[MemoryEvent]) -> Vec<DraftFailureLesson> {
    let mut by_strategy: BTreeMap<String, DraftFailureLesson> = BTreeMap::new();
    for event in events {
        let Some(record) = is_draft_failure(event) else {
            continue;
        };
        let strategy = field(record, "strategy").unwrap_or("unknown").to_owned();
        let passed = field(record, "passed_tests")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        let total = field(record, "total_tests")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        let attempt = field(record, "attempt")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        let max_attempts = field(record, "max_attempts")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(crate::draft_portfolio::MAX_ATTEMPTS);
        let entry = by_strategy
            .entry(strategy.clone())
            .or_insert_with(|| DraftFailureLesson {
                strategy,
                occurrences: 0,
                passed_tests: 0,
                total_tests: 0,
                attempts: 0,
                exhausted_retry_budget: true,
                lesson: String::new(),
                source_event_ids: Vec::new(),
            });
        entry.occurrences += 1;
        entry.passed_tests += passed;
        entry.total_tests += total;
        entry.attempts = entry.attempts.saturating_add(attempt);
        entry.exhausted_retry_budget &= attempt >= max_attempts;
        if !event.id.is_empty() {
            entry.source_event_ids.push(event.id.clone());
        }
    }
    let mut lessons = by_strategy.into_values().collect::<Vec<_>>();
    for lesson in &mut lessons {
        lesson.lesson = conclusion(lesson).to_owned();
    }
    lessons.sort_by(|left, right| {
        right
            .occurrences
            .cmp(&left.occurrences)
            .then_with(|| left.strategy.cmp(&right.strategy))
    });
    lessons
}

const fn conclusion(lesson: &DraftFailureLesson) -> &'static str {
    if lesson.passed_tests > 0 && lesson.passed_tests < lesson.total_tests {
        "extend_strategy"
    } else if lesson.exhausted_retry_budget {
        "deprioritize_strategy"
    } else {
        "raise_draft_count"
    }
}
