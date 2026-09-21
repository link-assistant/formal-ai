//! Browser adapter for the shared exact-memory query plan.
//!
//! The SQL and GraphQL parsers are included from `src/memory_query_language`;
//! this file only adapts browser event records to that typed Rust plan and
//! serializes the bounded outcome for the worker bridge.

use alloc::borrow::ToOwned;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::memory_query_language::{
    parse_memory_query_plan, AggregateExpression, AggregateFunction, ComparisonOperator,
    FilterExpression, MemoryField, MemoryQueryOperation, MemoryQueryPlan, MemoryQueryValue,
    QueryDialect, SortDirection,
};
use crate::web_engine_core::stable_id;
use crate::{decode_uri_component, push_json_string};

const MAX_MATCHES: usize = 128;
const MAX_ITERATIONS: usize = 4;

#[derive(Debug, Clone, PartialEq)]
struct BrowserMemoryRecord {
    values: BTreeMap<MemoryField, MemoryQueryValue>,
}

impl BrowserMemoryRecord {
    fn empty() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    fn value(&self, field: MemoryField) -> MemoryQueryValue {
        self.values
            .get(&field)
            .cloned()
            .unwrap_or(MemoryQueryValue::Null)
    }

    fn set(&mut self, field: MemoryField, value: MemoryQueryValue) {
        self.values.insert(field, value);
    }
}

struct BrowserQueryOutcome {
    rows: Vec<BTreeMap<String, MemoryQueryValue>>,
    matched_ids: Vec<String>,
    changed: usize,
    halt: &'static str,
    events: Vec<BrowserMemoryRecord>,
}

pub(super) fn answer(payload: &str) -> String {
    let Some((source, source_events)) = decode_payload(payload) else {
        return String::new();
    };
    let Some(dialect) = detect_dialect(&source) else {
        return String::new();
    };
    let plan = match parse_memory_query_plan(&source, dialect) {
        Ok(plan) => plan,
        Err(error) => return rejected_answer(dialect, &error.message),
    };
    let canonical = plan.canonical(MAX_MATCHES, MAX_ITERATIONS);
    let query_id = stable_id("memory_query", &canonical);
    let outcome = execute(&plan, &query_id, source_events.clone());
    serialize_answer(dialect, &plan, &query_id, &source_events, &outcome)
}

fn detect_dialect(source: &str) -> Option<QueryDialect> {
    let normalized = source.trim().to_ascii_lowercase();
    if normalized.starts_with("select ")
        || normalized.starts_with("insert into ")
        || normalized.starts_with("update ")
        || normalized.starts_with("delete from ")
    {
        return Some(QueryDialect::SqlAnsi);
    }
    if normalized.contains('{')
        && (normalized.starts_with("query")
            || normalized.starts_with("mutation")
            || normalized.starts_with('{'))
        && normalized.contains("memory")
    {
        return Some(QueryDialect::GraphQl);
    }
    None
}

fn decode_payload(payload: &str) -> Option<(String, Vec<BrowserMemoryRecord>)> {
    let mut lines = payload.lines();
    let query = lines.next()?.strip_prefix("q\t")?;
    let source = decode_uri_component(query);
    let mut events = Vec::new();
    for line in lines {
        let Some(fields) = line.strip_prefix("e\t") else {
            continue;
        };
        let values = fields.split('\t').collect::<Vec<_>>();
        if values.len() != MemoryField::ALL.len() {
            continue;
        }
        let mut event = BrowserMemoryRecord::empty();
        for (field, encoded) in MemoryField::ALL.into_iter().zip(values) {
            event.set(field, decode_value(encoded));
        }
        events.push(event);
    }
    Some((source, events))
}

fn decode_value(encoded: &str) -> MemoryQueryValue {
    let Some((tag, value)) = encoded.split_at_checked(1) else {
        return MemoryQueryValue::Null;
    };
    match tag {
        "s" => MemoryQueryValue::Text(decode_uri_component(value)),
        "i" => value
            .parse::<i64>()
            .map_or(MemoryQueryValue::Null, MemoryQueryValue::Integer),
        "f" => value
            .parse::<f64>()
            .map_or(MemoryQueryValue::Null, MemoryQueryValue::Float),
        "b" => MemoryQueryValue::Boolean(value == "1"),
        "l" => MemoryQueryValue::List(
            value
                .split(',')
                .filter(|item| !item.is_empty())
                .map(|item| MemoryQueryValue::Text(decode_uri_component(item)))
                .collect(),
        ),
        _ => MemoryQueryValue::Null,
    }
}

fn execute(
    plan: &MemoryQueryPlan,
    query_id: &str,
    mut events: Vec<BrowserMemoryRecord>,
) -> BrowserQueryOutcome {
    if plan.operation == MemoryQueryOperation::Delete {
        return BrowserQueryOutcome {
            rows: Vec::new(),
            matched_ids: Vec::new(),
            changed: 0,
            halt: "permission_denied",
            events,
        };
    }
    let indices = active_event_indices(&events)
        .into_iter()
        .filter(|index| {
            plan.filter
                .as_ref()
                .is_none_or(|filter| filter_matches(filter, &events[*index]))
        })
        .collect::<Vec<_>>();
    if indices.len() > MAX_MATCHES {
        return BrowserQueryOutcome {
            rows: Vec::new(),
            matched_ids: Vec::new(),
            changed: 0,
            halt: "match_limit",
            events,
        };
    }
    let mut matched_ids = indices
        .iter()
        .map(|index| event_identity(&events[*index], *index))
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    let changed = match plan.operation {
        MemoryQueryOperation::Select => {
            rows = select_rows(plan, &events, &indices);
            0
        }
        MemoryQueryOperation::Insert => {
            let mut event = BrowserMemoryRecord::empty();
            for (field, value) in &plan.assignments {
                event.set(*field, value.clone());
            }
            if event.value(MemoryField::Id) == MemoryQueryValue::Null {
                event.set(
                    MemoryField::Id,
                    MemoryQueryValue::Text(stable_id(
                        "memory_query_event",
                        &format!("{query_id}:{}", events.len()),
                    )),
                );
            }
            if event.value(MemoryField::WriteCount) == MemoryQueryValue::Null {
                event.set(MemoryField::WriteCount, MemoryQueryValue::Integer(1));
            }
            matched_ids = vec![event.value(MemoryField::Id).display_text()];
            if !plan.projection.is_empty() {
                rows.push(project(&event, &plan.projection));
            }
            events.push(event);
            1
        }
        MemoryQueryOperation::Update => {
            let mut changed = 0;
            for index in &indices {
                let before = events[*index].clone();
                for (field, value) in &plan.assignments {
                    events[*index].set(*field, value.clone());
                }
                if events[*index] != before {
                    let writes = non_negative_integer(&before.value(MemoryField::WriteCount));
                    events[*index].set(
                        MemoryField::WriteCount,
                        MemoryQueryValue::Integer(writes.max(1).saturating_add(1)),
                    );
                    changed += 1;
                }
            }
            if !plan.projection.is_empty() {
                rows = indices
                    .iter()
                    .map(|index| project(&events[*index], &plan.projection))
                    .collect();
            }
            changed
        }
        MemoryQueryOperation::Delete => 0,
    };
    order_and_page(&mut rows, plan);
    BrowserQueryOutcome {
        rows,
        matched_ids,
        changed,
        halt: "complete",
        events,
    }
}

fn select_rows(
    plan: &MemoryQueryPlan,
    events: &[BrowserMemoryRecord],
    indices: &[usize],
) -> Vec<BTreeMap<String, MemoryQueryValue>> {
    if plan.aggregates.is_empty() && plan.group_by.is_empty() {
        return indices
            .iter()
            .map(|index| project(&events[*index], &plan.projection))
            .collect();
    }
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for index in indices {
        let key = plan
            .group_by
            .iter()
            .map(|field| events[*index].value(*field).canonical())
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
                    row.insert(field.as_str().to_owned(), events[*first].value(*field));
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
    events: &[BrowserMemoryRecord],
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
                .filter_map(|index| numeric_value(&events[*index].value(field)))
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
        AggregateFunction::Count => unreachable!(),
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
                .map(|value| {
                    let distance = value - mean;
                    distance * distance
                })
                .sum::<f64>()
                / count;
            MemoryQueryValue::Float(
                if aggregate.function == AggregateFunction::PopulationStandardDeviation {
                    square_root(variance)
                } else {
                    variance
                },
            )
        }
    }
}

fn project(
    event: &BrowserMemoryRecord,
    fields: &[MemoryField],
) -> BTreeMap<String, MemoryQueryValue> {
    fields
        .iter()
        .map(|field| (field.as_str().to_owned(), event.value(*field)))
        .collect()
}

fn filter_matches(filter: &FilterExpression, event: &BrowserMemoryRecord) -> bool {
    match filter {
        FilterExpression::Compare {
            field,
            operator,
            value,
        } => compare(&event.value(*field), *operator, value),
        FilterExpression::And(expressions) => expressions
            .iter()
            .all(|expression| filter_matches(expression, event)),
        FilterExpression::Or(expressions) => expressions
            .iter()
            .any(|expression| filter_matches(expression, event)),
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
        ComparisonOperator::LessThan => compare_values(actual, expected) == Ordering::Less,
        ComparisonOperator::LessThanOrEqual => {
            compare_values(actual, expected) != Ordering::Greater
        }
        ComparisonOperator::GreaterThan => compare_values(actual, expected) == Ordering::Greater,
        ComparisonOperator::GreaterThanOrEqual => {
            compare_values(actual, expected) != Ordering::Less
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
    let result = match pattern.get(pattern_index) {
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
    cache[pattern_index][actual_index] = Some(result);
    result
}

fn order_and_page(rows: &mut Vec<BTreeMap<String, MemoryQueryValue>>, plan: &MemoryQueryPlan) {
    rows.sort_by(|left, right| {
        for order in &plan.order_by {
            let field = order.field.as_str();
            let comparison = compare_values(
                left.get(field).unwrap_or(&MemoryQueryValue::Null),
                right.get(field).unwrap_or(&MemoryQueryValue::Null),
            );
            if comparison != Ordering::Equal {
                return if order.direction == SortDirection::Descending {
                    comparison.reverse()
                } else {
                    comparison
                };
            }
        }
        Ordering::Equal
    });
    *rows = rows
        .drain(..)
        .skip(plan.offset)
        .take(plan.limit.unwrap_or(usize::MAX))
        .collect();
}

fn compare_values(left: &MemoryQueryValue, right: &MemoryQueryValue) -> Ordering {
    match (numeric_value(left), numeric_value(right)) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        _ => left.display_text().cmp(&right.display_text()),
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

fn non_negative_integer(value: &MemoryQueryValue) -> i64 {
    match value {
        MemoryQueryValue::Integer(value) => (*value).max(0),
        _ => 0,
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn integer_or_float(value: f64) -> MemoryQueryValue {
    if value >= i64::MIN as f64 && value <= i64::MAX as f64 {
        let integer = value as i64;
        if integer as f64 == value {
            return MemoryQueryValue::Integer(integer);
        }
    }
    MemoryQueryValue::Float(value)
}

fn square_root(value: f64) -> f64 {
    if value <= 0.0 {
        return 0.0;
    }
    let mut estimate = if value < 1.0 { 1.0 } else { value };
    for _ in 0..24 {
        estimate = (estimate + value / estimate) / 2.0;
    }
    estimate
}

fn event_identity(event: &BrowserMemoryRecord, index: usize) -> String {
    match event.value(MemoryField::Id) {
        MemoryQueryValue::Null => format!("memory_event_{index}"),
        value => value.display_text(),
    }
}

fn active_event_indices(events: &[BrowserMemoryRecord]) -> Vec<usize> {
    let retracted = events
        .iter()
        .filter(|event| {
            event.value(MemoryField::Kind).display_text() == "memory_retraction"
        })
        .map(|event| event.value(MemoryField::Inputs).display_text())
        .collect::<BTreeSet<_>>();
    events
        .iter()
        .enumerate()
        .filter(|(index, event)| {
            event.value(MemoryField::Kind).display_text() != "memory_retraction"
                && !retracted.contains(&event_identity(event, *index))
        })
        .map(|(index, _)| index)
        .collect()
}

fn rejected_answer(dialect: QueryDialect, message: &str) -> String {
    let mut content = String::from("memory_query_error\n  dialect ");
    push_lino_value(&mut content, dialect.as_str());
    content.push_str("\n  message ");
    push_lino_value(&mut content, message);
    let mut output = String::from("{\"intent\":\"memory_exact_query_rejected\",\"content\":");
    push_json_string(&mut output, &content);
    output.push_str(",\"confidence\":1,\"evidence\":[");
    push_json_string(&mut output, &format!("memory_exact_query_rejected:{message}"));
    output.push_str(",\"response:memory_exact_query_rejected\"]}");
    output
}

fn serialize_answer(
    dialect: QueryDialect,
    plan: &MemoryQueryPlan,
    query_id: &str,
    source_events: &[BrowserMemoryRecord],
    outcome: &BrowserQueryOutcome,
) -> String {
    let compiled = compiled_trace(dialect, plan, query_id);
    let result = result_trace(dialect, query_id, outcome);
    let intent = if outcome.halt == "permission_denied" {
        "memory_exact_query_refused"
    } else {
        "memory_exact_query"
    };
    let mut output = String::from("{\"intent\":");
    push_json_string(&mut output, intent);
    output.push_str(",\"content\":");
    push_json_string(&mut output, &result);
    output.push_str(",\"confidence\":1,\"evidence\":[");
    push_json_string(
        &mut output,
        &format!("memory_exact_query_compiled:{compiled}"),
    );
    output.push(',');
    push_json_string(
        &mut output,
        &format!("memory_exact_query_execution:{result}"),
    );
    output.push_str(",\"response:memory_exact_query\"]");
    if outcome.changed > 0 {
        output.push_str(",\"memoryOperation\":");
        serialize_memory_operation(&mut output, source_events, &outcome.events);
    }
    output.push('}');
    output
}

fn compiled_trace(dialect: QueryDialect, plan: &MemoryQueryPlan, query_id: &str) -> String {
    let mut output = String::from("memory_query");
    push_lino_key(&mut output, 2, "id");
    push_lino_value(&mut output, query_id);
    push_lino_key(&mut output, 2, "dialect");
    push_lino_value(&mut output, dialect.as_str());
    push_lino_key(&mut output, 2, "parser_engine");
    push_lino_value(&mut output, "rust_shared_exact_parser");
    push_lino_key(&mut output, 2, "grammar");
    push_lino_value(
        &mut output,
        if dialect == QueryDialect::GraphQl {
            "GraphQL"
        } else {
            "sql-ansi"
        },
    );
    push_lino_key(&mut output, 2, "operation");
    push_lino_value(&mut output, plan.operation.as_str());
    push_lino_key(&mut output, 2, "link_cli_substitution");
    push_lino_value(&mut output, "typed_plan_to_links");
    push_lino_key(&mut output, 2, "effect");
    output.push_str(match plan.operation {
        MemoryQueryOperation::Select => "read",
        MemoryQueryOperation::Insert => "create",
        MemoryQueryOperation::Update => "update",
        MemoryQueryOperation::Delete => "delete",
    });
    output
}

fn result_trace(
    dialect: QueryDialect,
    query_id: &str,
    outcome: &BrowserQueryOutcome,
) -> String {
    let mut output = String::from("memory_query_result");
    push_lino_key(&mut output, 2, "query");
    push_lino_value(&mut output, query_id);
    push_lino_key(&mut output, 2, "dialect");
    push_lino_value(&mut output, dialect.as_str());
    push_lino_key(&mut output, 2, "matched");
    output.push_str(&outcome.matched_ids.len().to_string());
    push_lino_key(&mut output, 2, "changed");
    output.push_str(&outcome.changed.to_string());
    push_lino_key(&mut output, 2, "halt");
    push_lino_value(&mut output, outcome.halt);
    for id in &outcome.matched_ids {
        push_lino_key(&mut output, 2, "matched_id");
        push_lino_value(&mut output, id);
    }
    for row in &outcome.rows {
        output.push('\n');
        output.push_str("  row");
        for (field, value) in row {
            push_lino_key(&mut output, 4, field);
            push_lino_value(&mut output, &value.canonical());
        }
    }
    output
}

fn push_lino_value(output: &mut String, value: &str) {
    push_json_string(output, value);
}

fn push_lino_key(output: &mut String, indent: usize, key: &str) {
    output.push('\n');
    for _ in 0..indent {
        output.push(' ');
    }
    output.push_str(key);
    output.push(' ');
}

fn serialize_memory_operation(
    output: &mut String,
    source: &[BrowserMemoryRecord],
    result: &[BrowserMemoryRecord],
) {
    output.push_str("{\"action\":\"program\",\"updates\":[");
    let mut first_update = true;
    for (before, after) in source.iter().zip(result) {
        let differences = MemoryField::ALL
            .into_iter()
            .filter(|field| before.value(*field) != after.value(*field))
            .collect::<Vec<_>>();
        if differences.is_empty() {
            continue;
        }
        if !first_update {
            output.push(',');
        }
        first_update = false;
        output.push_str("{\"id\":");
        push_json_value(output, &before.value(MemoryField::Id));
        output.push_str(",\"fields\":{");
        for (index, field) in differences.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_json_string(output, field.as_str());
            output.push(':');
            push_json_value(output, &after.value(*field));
        }
        output.push_str("}}");
    }
    output.push_str("],\"appends\":[");
    for (index, event) in result.iter().skip(source.len()).enumerate() {
        if index > 0 {
            output.push(',');
        }
        serialize_record(output, event);
    }
    output.push_str("]}");
}

fn serialize_record(output: &mut String, event: &BrowserMemoryRecord) {
    output.push('{');
    let mut first = true;
    for field in MemoryField::ALL {
        let value = event.value(field);
        if value == MemoryQueryValue::Null {
            continue;
        }
        if !first {
            output.push(',');
        }
        first = false;
        push_json_string(output, field.as_str());
        output.push(':');
        push_json_value(output, &value);
    }
    output.push('}');
}

fn push_json_value(output: &mut String, value: &MemoryQueryValue) {
    match value {
        MemoryQueryValue::Null => output.push_str("null"),
        MemoryQueryValue::Text(value) => push_json_string(output, value),
        MemoryQueryValue::Integer(value) => output.push_str(&value.to_string()),
        MemoryQueryValue::Float(value) => output.push_str(&value.to_string()),
        MemoryQueryValue::Boolean(value) => output.push_str(if *value { "true" } else { "false" }),
        MemoryQueryValue::List(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                push_json_value(output, value);
            }
            output.push(']');
        }
    }
}
