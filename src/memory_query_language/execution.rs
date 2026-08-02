use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    AggregateExpression, AggregateFunction, ComparisonOperator, CompiledMemoryQuery,
    FilterExpression, MemoryField, MemoryQueryOperation, MemoryQueryPlan, MemoryQueryValue,
    SortDirection,
};
use crate::engine::stable_id;
use crate::memory::{isoformat_now, MemoryEvent, MemoryStore};
use crate::memory_program::{
    MemoryProgramAuthorization, MemoryProgramHalt, MemoryProgramPermission,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryOutcome {
    pub rows: Vec<BTreeMap<String, MemoryQueryValue>>,
    pub matched_ids: Vec<String>,
    pub changed: usize,
    pub halt: MemoryProgramHalt,
}

#[must_use]
pub fn execute_memory_query(
    query: &CompiledMemoryQuery,
    store: &mut MemoryStore,
    authorization: MemoryProgramAuthorization,
) -> MemoryQueryOutcome {
    if let Err(gap) =
        super::lowering::validate_link_program(&query.plan, &query.link_program, query.limits)
    {
        return MemoryQueryOutcome {
            rows: Vec::new(),
            matched_ids: Vec::new(),
            changed: 0,
            halt: MemoryProgramHalt::ProgramGap {
                primitive: format!("memory_query_link_lowering:{gap}"),
            },
        };
    }
    if let Some(required) = denied_permission(query.plan.operation, authorization) {
        return MemoryQueryOutcome {
            rows: Vec::new(),
            matched_ids: Vec::new(),
            changed: 0,
            halt: MemoryProgramHalt::PermissionDenied {
                required: required.as_str().to_owned(),
            },
        };
    }
    let indices = active_event_indices(store)
        .into_iter()
        .filter(|index| {
            query
                .plan
                .filter
                .as_ref()
                .is_none_or(|filter| filter_matches(filter, &store.events()[*index]))
        })
        .collect::<Vec<_>>();
    if indices.len() > query.limits.max_matches {
        return MemoryQueryOutcome {
            rows: Vec::new(),
            matched_ids: Vec::new(),
            changed: 0,
            halt: MemoryProgramHalt::MatchLimit {
                matched: indices.len(),
                max_matches: query.limits.max_matches,
            },
        };
    }
    let matched_ids = indices
        .iter()
        .map(|index| event_identity(&store.events()[*index], *index))
        .collect::<Vec<_>>();
    match query.plan.operation {
        MemoryQueryOperation::Select => execute_select(query, store, &indices, matched_ids),
        MemoryQueryOperation::Insert => execute_insert(query, store),
        MemoryQueryOperation::Update => execute_update(query, store, &indices, matched_ids),
        MemoryQueryOperation::Delete => execute_delete(query, store, &indices, matched_ids),
    }
}

fn denied_permission(
    operation: MemoryQueryOperation,
    authorization: MemoryProgramAuthorization,
) -> Option<MemoryProgramPermission> {
    match operation {
        MemoryQueryOperation::Insert | MemoryQueryOperation::Update
            if authorization == MemoryProgramAuthorization::ReadOnly =>
        {
            Some(MemoryProgramPermission::Write)
        }
        MemoryQueryOperation::Delete
            if authorization != MemoryProgramAuthorization::DestructiveConfirmed =>
        {
            Some(MemoryProgramPermission::Destructive)
        }
        MemoryQueryOperation::Select
        | MemoryQueryOperation::Insert
        | MemoryQueryOperation::Update
        | MemoryQueryOperation::Delete => None,
    }
}

fn execute_select(
    query: &CompiledMemoryQuery,
    store: &mut MemoryStore,
    indices: &[usize],
    matched_ids: Vec<String>,
) -> MemoryQueryOutcome {
    let events = store.events();
    let mut rows = if query.plan.aggregates.is_empty() && query.plan.group_by.is_empty() {
        indices
            .iter()
            .map(|index| project_event(&events[*index], &query.plan.projection))
            .collect()
    } else {
        aggregate_rows(events, indices, &query.plan)
    };
    order_rows(&mut rows, &query.plan);
    rows = rows
        .into_iter()
        .skip(query.plan.offset)
        .take(query.plan.limit.unwrap_or(usize::MAX))
        .collect();
    let _ = store.record_access(indices);
    MemoryQueryOutcome {
        rows,
        matched_ids,
        changed: 0,
        halt: MemoryProgramHalt::Complete,
    }
}

fn execute_insert(query: &CompiledMemoryQuery, store: &mut MemoryStore) -> MemoryQueryOutcome {
    let mut event = MemoryEvent::default();
    for (field, value) in &query.plan.assignments {
        set_event_field(&mut event, *field, value);
    }
    if event.id.is_empty() {
        event.id = stable_id(
            "memory_query_event",
            &format!("{}:{}", query.id, store.len()),
        );
    }
    let id = event.id.clone();
    let row = project_event(&event, &query.plan.projection);
    store.append(event);
    MemoryQueryOutcome {
        rows: (!row.is_empty()).then_some(row).into_iter().collect(),
        matched_ids: vec![id],
        changed: 1,
        halt: MemoryProgramHalt::Complete,
    }
}

fn execute_update(
    query: &CompiledMemoryQuery,
    store: &mut MemoryStore,
    indices: &[usize],
    matched_ids: Vec<String>,
) -> MemoryQueryOutcome {
    let mut changed = 0;
    for index in indices {
        let event = &mut store.events_mut()[*index];
        let before = event.clone();
        for (field, value) in &query.plan.assignments {
            set_event_field(event, *field, value);
        }
        if *event != before {
            event.write_count = event.write_count.max(1).saturating_add(1);
            changed += 1;
        }
    }
    let rows = indices
        .iter()
        .map(|index| project_event(&store.events()[*index], &query.plan.projection))
        .filter(|row| !row.is_empty())
        .collect();
    MemoryQueryOutcome {
        rows,
        matched_ids,
        changed,
        halt: MemoryProgramHalt::Complete,
    }
}

fn execute_delete(
    query: &CompiledMemoryQuery,
    store: &mut MemoryStore,
    indices: &[usize],
    matched_ids: Vec<String>,
) -> MemoryQueryOutcome {
    let rows = indices
        .iter()
        .map(|index| project_event(&store.events()[*index], &query.plan.projection))
        .filter(|row| !row.is_empty())
        .collect();
    for id in &matched_ids {
        let retraction = MemoryEvent {
            id: stable_id("memory_retraction", &format!("{}:{id}", query.id)),
            kind: Some(String::from("memory_retraction")),
            role: Some(String::from("system")),
            inputs: Some(id.clone()),
            outputs: Some(format!("memory_query:{}", query.id)),
            content: Some(format!("memory_retraction:{id}")),
            sent_at: Some(isoformat_now()),
            evidence: vec![String::from("policy:append_only_retraction")],
            ..MemoryEvent::default()
        };
        if !store.events().iter().any(|event| event.id == retraction.id) {
            store.append(retraction);
        }
    }
    MemoryQueryOutcome {
        rows,
        matched_ids,
        changed: indices.len(),
        halt: MemoryProgramHalt::Complete,
    }
}

fn active_event_indices(store: &MemoryStore) -> Vec<usize> {
    let retracted = store
        .events()
        .iter()
        .filter(|event| event.kind.as_deref() == Some("memory_retraction"))
        .filter_map(|event| event.inputs.as_deref())
        .collect::<BTreeSet<_>>();
    store
        .events()
        .iter()
        .enumerate()
        .filter(|(index, event)| {
            event.kind.as_deref() != Some("memory_retraction")
                && !retracted.contains(event_identity(event, *index).as_str())
        })
        .map(|(index, _)| index)
        .collect()
}

fn aggregate_rows(
    events: &[MemoryEvent],
    indices: &[usize],
    plan: &MemoryQueryPlan,
) -> Vec<BTreeMap<String, MemoryQueryValue>> {
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for index in indices {
        let key = plan
            .group_by
            .iter()
            .map(|field| event_field_value(&events[*index], *field).canonical())
            .collect::<Vec<_>>()
            .join("\u{1f}");
        groups.entry(key).or_default().push(*index);
    }
    if groups.is_empty() && plan.group_by.is_empty() {
        groups.insert(String::new(), Vec::new());
    }
    groups
        .into_values()
        .map(|members| {
            let mut row = BTreeMap::new();
            if let Some(first) = members.first() {
                for field in &plan.group_by {
                    row.insert(
                        field.as_str().to_owned(),
                        event_field_value(&events[*first], *field),
                    );
                }
            }
            for aggregate in &plan.aggregates {
                row.insert(
                    aggregate.alias.clone(),
                    aggregate_value(aggregate, events, &members),
                );
            }
            row
        })
        .collect()
}

fn aggregate_value(
    aggregate: &AggregateExpression,
    events: &[MemoryEvent],
    members: &[usize],
) -> MemoryQueryValue {
    if aggregate.function == AggregateFunction::Count {
        return MemoryQueryValue::Integer(i64::try_from(members.len()).unwrap_or(i64::MAX));
    }
    let values = aggregate
        .field
        .map(|field| {
            members
                .iter()
                .filter_map(|index| numeric_value(&event_field_value(&events[*index], field)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if values.is_empty() {
        return MemoryQueryValue::Null;
    }
    let sum = values.iter().sum::<f64>();
    #[allow(clippy::cast_precision_loss)]
    let count = values.len() as f64;
    match aggregate.function {
        AggregateFunction::Count => unreachable!("count returned above"),
        AggregateFunction::Sum => integer_or_float(sum),
        AggregateFunction::Average => MemoryQueryValue::Float(sum / count),
        AggregateFunction::Minimum => {
            integer_or_float(values.iter().copied().fold(f64::INFINITY, f64::min))
        }
        AggregateFunction::Maximum => {
            integer_or_float(values.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        }
        AggregateFunction::PopulationVariance | AggregateFunction::PopulationStandardDeviation => {
            let mean = sum / count;
            let variance = values
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / count;
            MemoryQueryValue::Float(
                if aggregate.function == AggregateFunction::PopulationStandardDeviation {
                    variance.sqrt()
                } else {
                    variance
                },
            )
        }
    }
}

fn project_event(
    event: &MemoryEvent,
    fields: &[MemoryField],
) -> BTreeMap<String, MemoryQueryValue> {
    fields
        .iter()
        .map(|field| (field.as_str().to_owned(), event_field_value(event, *field)))
        .collect()
}

fn filter_matches(filter: &FilterExpression, event: &MemoryEvent) -> bool {
    match filter {
        FilterExpression::Compare {
            field,
            operator,
            value,
        } => compare(&event_field_value(event, *field), *operator, value),
        FilterExpression::And(expressions) => expressions
            .iter()
            .all(|filter| filter_matches(filter, event)),
        FilterExpression::Or(expressions) => expressions
            .iter()
            .any(|filter| filter_matches(filter, event)),
        FilterExpression::Not(expression) => !filter_matches(expression, event),
    }
}

fn compare(
    actual: &MemoryQueryValue,
    operator: ComparisonOperator,
    expected: &MemoryQueryValue,
) -> bool {
    match operator {
        ComparisonOperator::Equal => actual == expected,
        ComparisonOperator::NotEqual => actual != expected,
        ComparisonOperator::IsNull => actual == &MemoryQueryValue::Null,
        ComparisonOperator::IsNotNull => actual != &MemoryQueryValue::Null,
        ComparisonOperator::Contains => actual
            .display_text()
            .to_lowercase()
            .contains(&expected.display_text().to_lowercase()),
        ComparisonOperator::Like => like_matches(&actual.display_text(), &expected.display_text()),
        ComparisonOperator::LessThan
        | ComparisonOperator::LessThanOrEqual
        | ComparisonOperator::GreaterThan
        | ComparisonOperator::GreaterThanOrEqual => {
            let ordering = compare_values(actual, expected);
            match operator {
                ComparisonOperator::LessThan => ordering == Ordering::Less,
                ComparisonOperator::LessThanOrEqual => ordering != Ordering::Greater,
                ComparisonOperator::GreaterThan => ordering == Ordering::Greater,
                ComparisonOperator::GreaterThanOrEqual => ordering != Ordering::Less,
                _ => false,
            }
        }
    }
}

fn like_matches(actual: &str, pattern: &str) -> bool {
    let actual = actual.to_lowercase().chars().collect::<Vec<_>>();
    let pattern = pattern.to_lowercase().chars().collect::<Vec<_>>();
    let mut cache = vec![vec![None; actual.len() + 1]; pattern.len() + 1];
    wildcard_matches(&pattern, &actual, 0, 0, &mut cache)
}

fn wildcard_matches(
    pattern: &[char],
    actual: &[char],
    pattern_index: usize,
    actual_index: usize,
    cache: &mut [Vec<Option<bool>>],
) -> bool {
    if let Some(cached) = cache[pattern_index][actual_index] {
        return cached;
    }
    let matches = match pattern.get(pattern_index) {
        None => actual_index == actual.len(),
        Some('%') => {
            wildcard_matches(pattern, actual, pattern_index + 1, actual_index, cache)
                || (actual_index < actual.len()
                    && wildcard_matches(pattern, actual, pattern_index, actual_index + 1, cache))
        }
        Some('_') => {
            actual_index < actual.len()
                && wildcard_matches(pattern, actual, pattern_index + 1, actual_index + 1, cache)
        }
        Some(expected) => {
            actual.get(actual_index) == Some(expected)
                && wildcard_matches(pattern, actual, pattern_index + 1, actual_index + 1, cache)
        }
    };
    cache[pattern_index][actual_index] = Some(matches);
    matches
}

fn event_field_value(event: &MemoryEvent, field: MemoryField) -> MemoryQueryValue {
    let optional = |value: &Option<String>| {
        value
            .clone()
            .map_or(MemoryQueryValue::Null, MemoryQueryValue::Text)
    };
    match field {
        MemoryField::Id if event.id.is_empty() => MemoryQueryValue::Null,
        MemoryField::Id => MemoryQueryValue::Text(event.id.clone()),
        MemoryField::Kind => optional(&event.kind),
        MemoryField::Role => optional(&event.role),
        MemoryField::Intent => optional(&event.intent),
        MemoryField::Tool => optional(&event.tool),
        MemoryField::Inputs => optional(&event.inputs),
        MemoryField::Outputs => optional(&event.outputs),
        MemoryField::Content => optional(&event.content),
        MemoryField::SentAt => optional(&event.sent_at),
        MemoryField::DemoLabel => optional(&event.demo_label),
        MemoryField::ConversationId => optional(&event.conversation_id),
        MemoryField::ConversationTitle => optional(&event.conversation_title),
        MemoryField::Evidence => MemoryQueryValue::List(
            event
                .evidence
                .iter()
                .cloned()
                .map(MemoryQueryValue::Text)
                .collect(),
        ),
        MemoryField::AccessCount => {
            MemoryQueryValue::Integer(i64::try_from(event.access_count).unwrap_or(i64::MAX))
        }
        MemoryField::WriteCount => {
            MemoryQueryValue::Integer(i64::try_from(event.write_count).unwrap_or(i64::MAX))
        }
    }
}

fn set_event_field(event: &mut MemoryEvent, field: MemoryField, value: &MemoryQueryValue) {
    let optional = |value: &MemoryQueryValue| match value {
        MemoryQueryValue::Null => None,
        _ => Some(value.display_text()),
    };
    match field {
        MemoryField::Id => event.id = value.display_text(),
        MemoryField::Kind => event.kind = optional(value),
        MemoryField::Role => event.role = optional(value),
        MemoryField::Intent => event.intent = optional(value),
        MemoryField::Tool => event.tool = optional(value),
        MemoryField::Inputs => event.inputs = optional(value),
        MemoryField::Outputs => event.outputs = optional(value),
        MemoryField::Content => event.content = optional(value),
        MemoryField::SentAt => event.sent_at = optional(value),
        MemoryField::DemoLabel => event.demo_label = optional(value),
        MemoryField::ConversationId => event.conversation_id = optional(value),
        MemoryField::ConversationTitle => event.conversation_title = optional(value),
        MemoryField::Evidence => {
            event.evidence = match value {
                MemoryQueryValue::List(values) => {
                    values.iter().map(MemoryQueryValue::display_text).collect()
                }
                MemoryQueryValue::Null => Vec::new(),
                _ => vec![value.display_text()],
            };
        }
        MemoryField::AccessCount => event.access_count = non_negative_integer(value),
        MemoryField::WriteCount => event.write_count = non_negative_integer(value),
    }
}

fn non_negative_integer(value: &MemoryQueryValue) -> u64 {
    match value {
        MemoryQueryValue::Integer(value) => u64::try_from(*value).unwrap_or_default(),
        _ => 0,
    }
}

#[allow(clippy::cast_precision_loss)]
const fn numeric_value(value: &MemoryQueryValue) -> Option<f64> {
    match value {
        MemoryQueryValue::Integer(value) => Some(*value as f64),
        MemoryQueryValue::Float(value) => Some(*value),
        _ => None,
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn integer_or_float(value: f64) -> MemoryQueryValue {
    if value.fract().abs() < f64::EPSILON && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
        MemoryQueryValue::Integer(value as i64)
    } else {
        MemoryQueryValue::Float(value)
    }
}

fn order_rows(rows: &mut [BTreeMap<String, MemoryQueryValue>], plan: &MemoryQueryPlan) {
    rows.sort_by(|left, right| {
        for order in &plan.order_by {
            let field = order.field.as_str();
            let ordering = compare_values(
                left.get(field).unwrap_or(&MemoryQueryValue::Null),
                right.get(field).unwrap_or(&MemoryQueryValue::Null),
            );
            if ordering != Ordering::Equal {
                return if order.direction == SortDirection::Descending {
                    ordering.reverse()
                } else {
                    ordering
                };
            }
        }
        Ordering::Equal
    });
}

fn compare_values(left: &MemoryQueryValue, right: &MemoryQueryValue) -> Ordering {
    match (numeric_value(left), numeric_value(right)) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        _ => left.display_text().cmp(&right.display_text()),
    }
}

fn event_identity(event: &MemoryEvent, index: usize) -> String {
    if event.id.is_empty() {
        format!("memory_event_{index}")
    } else {
        event.id.clone()
    }
}
