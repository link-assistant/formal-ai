use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::syntax::{Parser, Token};
use super::{
    AggregateExpression, AggregateFunction, ComparisonOperator, FilterExpression, MemoryField,
    MemoryQueryError, MemoryQueryOperation, MemoryQueryPlan, MemoryQueryValue, SortDirection,
    SortExpression,
};

#[derive(Debug, Clone)]
enum GraphValue {
    Scalar(MemoryQueryValue),
    Enum(String),
    Object(BTreeMap<String, Self>),
    List(Vec<Self>),
}

#[derive(Debug, Clone)]
struct Selection {
    alias: Option<String>,
    name: String,
    arguments: BTreeMap<String, GraphValue>,
    children: Vec<Self>,
}

pub(super) fn parse_graphql(source: &str) -> Result<MemoryQueryPlan, MemoryQueryError> {
    let mut parser = Parser::new(source)?;
    let mut declared_mutation = false;
    if parser.eat_word("query") || parser.eat_word("subscription") {
        consume_operation_header(&mut parser)?;
    } else if parser.eat_word("mutation") {
        declared_mutation = true;
        consume_operation_header(&mut parser)?;
    }
    parser.expect_symbol('{')?;
    let root = parse_selection(&mut parser)?;
    parser.expect_symbol('}')?;
    parser.finish()?;

    match root.name.to_ascii_lowercase().as_str() {
        "memory" | "memoryevents" => parse_memory_read(root),
        "memoryaggregate" | "memoryaggregates" => parse_memory_aggregate(root),
        "creatememory" | "creatememoryevent" if declared_mutation => parse_create(&root),
        "updatememory" | "updatememoryevent" if declared_mutation => parse_update(&root),
        "deletememory" | "deletememoryevent" if declared_mutation => parse_delete(&root),
        name => Err(MemoryQueryError::new(format!(
            "graphql_root_not_allowed:{name}"
        ))),
    }
}

fn consume_operation_header(parser: &mut Parser) -> Result<(), MemoryQueryError> {
    if matches!(parser.peek(), Some(Token::Word(_))) {
        let _ = parser.word()?;
    }
    if parser.eat_symbol('(') {
        let mut depth = 1_usize;
        while depth > 0 {
            match parser.next() {
                Some(Token::Symbol('(')) => depth += 1,
                Some(Token::Symbol(')')) => depth -= 1,
                Some(_) => {}
                None => {
                    return Err(MemoryQueryError::new("graphql_variables_unterminated"));
                }
            }
        }
    }
    Ok(())
}

fn parse_selection(parser: &mut Parser) -> Result<Selection, MemoryQueryError> {
    let first = parser.word()?;
    let (alias, name) = if parser.eat_symbol(':') {
        (Some(first), parser.word()?)
    } else {
        (None, first)
    };
    let arguments = if parser.eat_symbol('(') {
        let values = parse_named_values(parser, ')')?;
        parser.expect_symbol(')')?;
        values
    } else {
        BTreeMap::new()
    };
    let children = if parser.eat_symbol('{') {
        let mut children = Vec::new();
        while !parser.eat_symbol('}') {
            children.push(parse_selection(parser)?);
        }
        children
    } else {
        Vec::new()
    };
    Ok(Selection {
        alias,
        name,
        arguments,
        children,
    })
}

fn parse_named_values(
    parser: &mut Parser,
    closing: char,
) -> Result<BTreeMap<String, GraphValue>, MemoryQueryError> {
    let mut values = BTreeMap::new();
    while parser.peek() != Some(&Token::Symbol(closing)) {
        let name = parser.word()?;
        parser.expect_symbol(':')?;
        values.insert(name, parse_graph_value(parser)?);
        if !parser.eat_symbol(',') && parser.peek() != Some(&Token::Symbol(closing)) {
            // GraphQL commas are optional, so the next identifier is enough.
        }
    }
    Ok(values)
}

fn parse_graph_value(parser: &mut Parser) -> Result<GraphValue, MemoryQueryError> {
    if parser.eat_symbol('{') {
        let object = parse_named_values(parser, '}')?;
        parser.expect_symbol('}')?;
        return Ok(GraphValue::Object(object));
    }
    if parser.eat_symbol('[') {
        let mut values = Vec::new();
        while !parser.eat_symbol(']') {
            values.push(parse_graph_value(parser)?);
            let _ = parser.eat_symbol(',');
        }
        return Ok(GraphValue::List(values));
    }
    match parser.peek() {
        Some(Token::Quoted(_) | Token::Number(_)) => parser.value().map(GraphValue::Scalar),
        Some(Token::Word(value))
            if value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("false")
                || value.eq_ignore_ascii_case("null") =>
        {
            parser.value().map(GraphValue::Scalar)
        }
        Some(Token::Word(_)) => parser.word().map(GraphValue::Enum),
        Some(Token::Symbol('$')) => {
            let _ = parser.next();
            Err(MemoryQueryError::new("graphql_variable_unbound"))
        }
        found => Err(MemoryQueryError::new(format!(
            "graphql_value_expected:{found:?}"
        ))),
    }
}

fn parse_memory_read(root: Selection) -> Result<MemoryQueryPlan, MemoryQueryError> {
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Select);
    apply_common_arguments(&root.arguments, &mut plan)?;
    for selection in root.children {
        if selection.name != "__typename" {
            plan.projection.push(MemoryField::parse(&selection.name)?);
        }
    }
    if plan.projection.is_empty() {
        return Err(MemoryQueryError::new("graphql_selection_required"));
    }
    Ok(plan)
}

fn parse_memory_aggregate(root: Selection) -> Result<MemoryQueryPlan, MemoryQueryError> {
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Select);
    apply_common_arguments(&root.arguments, &mut plan)?;
    if let Some(value) = find_argument(&root.arguments, "groupBy") {
        plan.group_by = graph_fields(value)?;
        plan.projection.clone_from(&plan.group_by);
    }
    for selection in root.children {
        let function = aggregate_function(&selection.name)?;
        let field = selection
            .arguments
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("field"))
            .map(|(_, value)| graph_field(value))
            .transpose()?;
        let alias = selection
            .alias
            .clone()
            .or_else(|| graph_optional_text(&selection.arguments, "as"))
            .unwrap_or_else(|| selection.name.clone());
        plan.aggregates.push(AggregateExpression {
            function,
            field,
            alias,
        });
    }
    Ok(plan)
}

fn parse_create(root: &Selection) -> Result<MemoryQueryPlan, MemoryQueryError> {
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Insert);
    let input = required_object(&root.arguments, "input")?;
    plan.assignments = graph_assignments(input)?;
    plan.projection = selection_projection(&root.children)?;
    Ok(plan)
}

fn parse_update(root: &Selection) -> Result<MemoryQueryPlan, MemoryQueryError> {
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Update);
    let assignments = find_argument(&root.arguments, "set")
        .or_else(|| find_argument(&root.arguments, "input"))
        .ok_or_else(|| MemoryQueryError::new("graphql_update_set_required"))?;
    let GraphValue::Object(assignments) = assignments else {
        return Err(MemoryQueryError::new("graphql_update_set_object_required"));
    };
    plan.assignments = graph_assignments(assignments)?;
    if let Some(filter) = find_argument(&root.arguments, "where") {
        plan.filter = Some(graph_filter(filter)?);
    }
    plan.projection = selection_projection(&root.children)?;
    Ok(plan)
}

fn parse_delete(root: &Selection) -> Result<MemoryQueryPlan, MemoryQueryError> {
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Delete);
    if let Some(filter) = find_argument(&root.arguments, "where") {
        plan.filter = Some(graph_filter(filter)?);
    }
    plan.projection = selection_projection(&root.children)?;
    Ok(plan)
}

fn apply_common_arguments(
    arguments: &BTreeMap<String, GraphValue>,
    plan: &mut MemoryQueryPlan,
) -> Result<(), MemoryQueryError> {
    if let Some(filter) = find_argument(arguments, "where") {
        plan.filter = Some(graph_filter(filter)?);
    }
    if let Some(order) = find_argument(arguments, "orderBy") {
        plan.order_by = graph_order(order)?;
    }
    if let Some(limit) =
        find_argument(arguments, "first").or_else(|| find_argument(arguments, "limit"))
    {
        plan.limit = Some(graph_usize(limit, "first")?);
    }
    if let Some(offset) =
        find_argument(arguments, "offset").or_else(|| find_argument(arguments, "skip"))
    {
        plan.offset = graph_usize(offset, "offset")?;
    }
    Ok(())
}

fn graph_filter(value: &GraphValue) -> Result<FilterExpression, MemoryQueryError> {
    let GraphValue::Object(fields) = value else {
        return Err(MemoryQueryError::new("graphql_where_object_required"));
    };
    let mut expressions = Vec::new();
    for (name, value) in fields {
        if name.eq_ignore_ascii_case("and") || name.eq_ignore_ascii_case("or") {
            let GraphValue::List(items) = value else {
                return Err(MemoryQueryError::new(format!(
                    "graphql_list_required:{name}"
                )));
            };
            let nested = items
                .iter()
                .map(graph_filter)
                .collect::<Result<Vec<_>, _>>()?;
            expressions.push(if name.eq_ignore_ascii_case("and") {
                FilterExpression::And(nested)
            } else {
                FilterExpression::Or(nested)
            });
            continue;
        }
        if name.eq_ignore_ascii_case("not") {
            expressions.push(FilterExpression::Not(Box::new(graph_filter(value)?)));
            continue;
        }
        let field = MemoryField::parse(name)?;
        let GraphValue::Object(comparisons) = value else {
            return Err(MemoryQueryError::new(format!(
                "graphql_filter_operator_required:{name}"
            )));
        };
        for (operator, value) in comparisons {
            expressions.push(FilterExpression::Compare {
                field,
                operator: graph_operator(operator)?,
                value: graph_scalar(value)?,
            });
        }
    }
    Ok(if expressions.len() == 1 {
        expressions.remove(0)
    } else {
        FilterExpression::And(expressions)
    })
}

fn graph_operator(value: &str) -> Result<ComparisonOperator, MemoryQueryError> {
    match value.to_ascii_lowercase().as_str() {
        "eq" => Ok(ComparisonOperator::Equal),
        "ne" => Ok(ComparisonOperator::NotEqual),
        "lt" => Ok(ComparisonOperator::LessThan),
        "le" | "lte" => Ok(ComparisonOperator::LessThanOrEqual),
        "gt" => Ok(ComparisonOperator::GreaterThan),
        "ge" | "gte" => Ok(ComparisonOperator::GreaterThanOrEqual),
        "contains" => Ok(ComparisonOperator::Contains),
        "like" => Ok(ComparisonOperator::Like),
        "isnull" => Ok(ComparisonOperator::IsNull),
        "isnotnull" => Ok(ComparisonOperator::IsNotNull),
        _ => Err(MemoryQueryError::new(format!(
            "graphql_filter_operator_unknown:{value}"
        ))),
    }
}

fn graph_order(value: &GraphValue) -> Result<Vec<SortExpression>, MemoryQueryError> {
    let GraphValue::Object(fields) = value else {
        return Err(MemoryQueryError::new("graphql_order_object_required"));
    };
    fields
        .iter()
        .map(|(field, direction)| {
            let direction = match graph_text(direction)?.to_ascii_lowercase().as_str() {
                "asc" | "ascending" => SortDirection::Ascending,
                "desc" | "descending" => SortDirection::Descending,
                other => {
                    return Err(MemoryQueryError::new(format!(
                        "graphql_sort_direction_unknown:{other}"
                    )));
                }
            };
            Ok(SortExpression {
                field: MemoryField::parse(field)?,
                direction,
            })
        })
        .collect()
}

fn graph_assignments(
    values: &BTreeMap<String, GraphValue>,
) -> Result<BTreeMap<MemoryField, MemoryQueryValue>, MemoryQueryError> {
    values
        .iter()
        .map(|(name, value)| Ok((MemoryField::parse(name)?, graph_scalar(value)?)))
        .collect()
}

fn selection_projection(children: &[Selection]) -> Result<Vec<MemoryField>, MemoryQueryError> {
    children
        .iter()
        .filter(|selection| selection.name != "__typename")
        .map(|selection| MemoryField::parse(&selection.name))
        .collect()
}

fn graph_fields(value: &GraphValue) -> Result<Vec<MemoryField>, MemoryQueryError> {
    match value {
        GraphValue::List(values) => values.iter().map(graph_field).collect(),
        _ => Ok(vec![graph_field(value)?]),
    }
}

fn graph_field(value: &GraphValue) -> Result<MemoryField, MemoryQueryError> {
    MemoryField::parse(&graph_text(value)?)
}

fn graph_scalar(value: &GraphValue) -> Result<MemoryQueryValue, MemoryQueryError> {
    match value {
        GraphValue::Scalar(value) => Ok(value.clone()),
        GraphValue::Enum(value) => Ok(MemoryQueryValue::Text(value.clone())),
        GraphValue::List(values) => values
            .iter()
            .map(graph_scalar)
            .collect::<Result<Vec<_>, _>>()
            .map(MemoryQueryValue::List),
        GraphValue::Object(_) => Err(MemoryQueryError::new("graphql_scalar_expected")),
    }
}

fn graph_text(value: &GraphValue) -> Result<String, MemoryQueryError> {
    match graph_scalar(value)? {
        MemoryQueryValue::Text(value) => Ok(value),
        other => Ok(other.display_text()),
    }
}

fn graph_usize(value: &GraphValue, name: &str) -> Result<usize, MemoryQueryError> {
    match graph_scalar(value)? {
        MemoryQueryValue::Integer(value) => usize::try_from(value)
            .map_err(|_| MemoryQueryError::new(format!("graphql_non_negative:{name}"))),
        _ => Err(MemoryQueryError::new(format!(
            "graphql_integer_required:{name}"
        ))),
    }
}

fn aggregate_function(name: &str) -> Result<AggregateFunction, MemoryQueryError> {
    match name.to_ascii_lowercase().as_str() {
        "count" => Ok(AggregateFunction::Count),
        "sum" => Ok(AggregateFunction::Sum),
        "avg" | "average" => Ok(AggregateFunction::Average),
        "min" | "minimum" => Ok(AggregateFunction::Minimum),
        "max" | "maximum" => Ok(AggregateFunction::Maximum),
        "variance" | "populationvariance" | "varpop" => Ok(AggregateFunction::PopulationVariance),
        "standarddeviation" | "populationstandarddeviation" | "stddevpop" => {
            Ok(AggregateFunction::PopulationStandardDeviation)
        }
        _ => Err(MemoryQueryError::new(format!(
            "graphql_aggregate_unknown:{name}"
        ))),
    }
}

fn required_object<'a>(
    arguments: &'a BTreeMap<String, GraphValue>,
    name: &str,
) -> Result<&'a BTreeMap<String, GraphValue>, MemoryQueryError> {
    match find_argument(arguments, name) {
        Some(GraphValue::Object(value)) => Ok(value),
        Some(_) => Err(MemoryQueryError::new(format!(
            "graphql_object_required:{name}"
        ))),
        None => Err(MemoryQueryError::new(format!(
            "graphql_argument_missing:{name}"
        ))),
    }
}

fn find_argument<'a>(
    arguments: &'a BTreeMap<String, GraphValue>,
    expected: &str,
) -> Option<&'a GraphValue> {
    arguments
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(expected))
        .map(|(_, value)| value)
}

fn graph_optional_text(arguments: &BTreeMap<String, GraphValue>, expected: &str) -> Option<String> {
    find_argument(arguments, expected).and_then(|value| graph_text(value).ok())
}
