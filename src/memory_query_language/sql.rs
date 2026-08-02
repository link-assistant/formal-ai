use alloc::borrow::ToOwned;
use alloc::boxed::Box;
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

pub(super) fn parse_sql(source: &str) -> Result<MemoryQueryPlan, MemoryQueryError> {
    let mut parser = Parser::new(source)?;
    let plan = if parser.is_word("select") {
        parse_select(&mut parser)?
    } else if parser.is_word("insert") {
        parse_insert(&mut parser)?
    } else if parser.is_word("update") {
        parse_update(&mut parser)?
    } else if parser.is_word("delete") {
        parse_delete(&mut parser)?
    } else {
        return Err(MemoryQueryError::new("sql_operation_expected"));
    };
    parser.finish()?;
    Ok(plan)
}

fn parse_select(parser: &mut Parser) -> Result<MemoryQueryPlan, MemoryQueryError> {
    parser.expect_word("select")?;
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Select);
    loop {
        parse_select_item(parser, &mut plan)?;
        if !parser.eat_symbol(',') {
            break;
        }
    }
    parser.expect_word("from")?;
    expect_memory_table(parser)?;
    parse_select_clauses(parser, &mut plan)?;
    Ok(plan)
}

fn parse_select_item(
    parser: &mut Parser,
    plan: &mut MemoryQueryPlan,
) -> Result<(), MemoryQueryError> {
    if parser.eat_symbol('*') {
        plan.projection.extend(MemoryField::ALL);
        return Ok(());
    }
    let name = parser.word()?;
    if !parser.eat_symbol('(') {
        plan.projection.push(MemoryField::parse(&name)?);
        let _ = optional_alias(parser)?;
        return Ok(());
    }
    let function = parse_aggregate_function(&name)?;
    let field = if parser.eat_symbol('*') {
        None
    } else {
        Some(MemoryField::parse(&parser.word()?)?)
    };
    parser.expect_symbol(')')?;
    if function != AggregateFunction::Count && field.is_some_and(|selected| !selected.is_numeric())
    {
        return Err(MemoryQueryError::new(format!(
            "aggregate_numeric_field_required:{}",
            function.as_str()
        )));
    }
    let alias = optional_alias(parser)?.unwrap_or_else(|| default_alias(function, field));
    plan.aggregates.push(AggregateExpression {
        function,
        field,
        alias,
    });
    Ok(())
}

fn parse_select_clauses(
    parser: &mut Parser,
    plan: &mut MemoryQueryPlan,
) -> Result<(), MemoryQueryError> {
    loop {
        if parser.eat_word("where") {
            if plan.filter.is_some() {
                return Err(MemoryQueryError::new("duplicate_where_clause"));
            }
            plan.filter = Some(parse_or_expression(parser)?);
        } else if parser.eat_word("group") {
            parser.expect_word("by")?;
            plan.group_by = parse_field_list(parser)?;
        } else if parser.eat_word("order") {
            parser.expect_word("by")?;
            plan.order_by = parse_order_list(parser)?;
        } else if parser.eat_word("limit") {
            plan.limit = Some(parse_usize(parser, "LIMIT")?);
        } else if parser.eat_word("offset") {
            plan.offset = parse_usize(parser, "OFFSET")?;
        } else {
            break;
        }
    }
    if plan.projection.is_empty() && plan.aggregates.is_empty() {
        return Err(MemoryQueryError::new("select_projection_required"));
    }
    Ok(())
}

fn parse_insert(parser: &mut Parser) -> Result<MemoryQueryPlan, MemoryQueryError> {
    parser.expect_word("insert")?;
    let _ = parser.eat_word("into");
    expect_memory_table(parser)?;
    parser.expect_symbol('(')?;
    let fields = parse_field_list(parser)?;
    parser.expect_symbol(')')?;
    parser.expect_word("values")?;
    parser.expect_symbol('(')?;
    let values = parse_value_list(parser)?;
    parser.expect_symbol(')')?;
    if fields.len() != values.len() {
        return Err(MemoryQueryError::new(format!(
            "insert_arity_mismatch:{}:{}",
            fields.len(),
            values.len()
        )));
    }
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Insert);
    plan.assignments = fields.into_iter().zip(values).collect();
    parse_returning(parser, &mut plan)?;
    Ok(plan)
}

fn parse_update(parser: &mut Parser) -> Result<MemoryQueryPlan, MemoryQueryError> {
    parser.expect_word("update")?;
    expect_memory_table(parser)?;
    parser.expect_word("set")?;
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Update);
    loop {
        let field = MemoryField::parse(&parser.word()?)?;
        if parser.eat_operator().as_deref() != Some("=") {
            return Err(MemoryQueryError::new("update_assignment_operator_expected"));
        }
        plan.assignments.insert(field, parser.value()?);
        if !parser.eat_symbol(',') {
            break;
        }
    }
    if parser.eat_word("where") {
        plan.filter = Some(parse_or_expression(parser)?);
    }
    parse_returning(parser, &mut plan)?;
    Ok(plan)
}

fn parse_delete(parser: &mut Parser) -> Result<MemoryQueryPlan, MemoryQueryError> {
    parser.expect_word("delete")?;
    let _ = parser.eat_word("from");
    expect_memory_table(parser)?;
    let mut plan = MemoryQueryPlan::empty(MemoryQueryOperation::Delete);
    if parser.eat_word("where") {
        plan.filter = Some(parse_or_expression(parser)?);
    }
    parse_returning(parser, &mut plan)?;
    Ok(plan)
}

fn parse_returning(
    parser: &mut Parser,
    plan: &mut MemoryQueryPlan,
) -> Result<(), MemoryQueryError> {
    if parser.eat_word("returning") {
        if parser.eat_symbol('*') {
            plan.projection.extend(MemoryField::ALL);
        } else {
            plan.projection = parse_field_list(parser)?;
        }
    }
    Ok(())
}

fn parse_or_expression(parser: &mut Parser) -> Result<FilterExpression, MemoryQueryError> {
    let mut expressions = vec![parse_and_expression(parser)?];
    while parser.eat_word("or") {
        expressions.push(parse_and_expression(parser)?);
    }
    Ok(if expressions.len() == 1 {
        expressions.remove(0)
    } else {
        FilterExpression::Or(expressions)
    })
}

fn parse_and_expression(parser: &mut Parser) -> Result<FilterExpression, MemoryQueryError> {
    let mut expressions = vec![parse_not_expression(parser)?];
    while parser.eat_word("and") {
        expressions.push(parse_not_expression(parser)?);
    }
    Ok(if expressions.len() == 1 {
        expressions.remove(0)
    } else {
        FilterExpression::And(expressions)
    })
}

fn parse_not_expression(parser: &mut Parser) -> Result<FilterExpression, MemoryQueryError> {
    if parser.eat_word("not") {
        return Ok(FilterExpression::Not(Box::new(parse_not_expression(
            parser,
        )?)));
    }
    if parser.eat_symbol('(') {
        let expression = parse_or_expression(parser)?;
        parser.expect_symbol(')')?;
        return Ok(expression);
    }
    parse_comparison(parser)
}

fn parse_comparison(parser: &mut Parser) -> Result<FilterExpression, MemoryQueryError> {
    let field = MemoryField::parse(&parser.word()?)?;
    if parser.eat_word("is") {
        let not = parser.eat_word("not");
        parser.expect_word("null")?;
        return Ok(FilterExpression::Compare {
            field,
            operator: if not {
                ComparisonOperator::IsNotNull
            } else {
                ComparisonOperator::IsNull
            },
            value: MemoryQueryValue::Null,
        });
    }
    let operator = if parser.eat_word("like") {
        ComparisonOperator::Like
    } else if parser.eat_word("contains") {
        ComparisonOperator::Contains
    } else {
        match parser.eat_operator().as_deref() {
            Some("=") => ComparisonOperator::Equal,
            Some("!=" | "<>") => ComparisonOperator::NotEqual,
            Some("<") => ComparisonOperator::LessThan,
            Some("<=") => ComparisonOperator::LessThanOrEqual,
            Some(">") => ComparisonOperator::GreaterThan,
            Some(">=") => ComparisonOperator::GreaterThanOrEqual,
            _ => {
                return Err(MemoryQueryError::new("comparison_operator_expected"));
            }
        }
    };
    Ok(FilterExpression::Compare {
        field,
        operator,
        value: parser.value()?,
    })
}

fn parse_field_list(parser: &mut Parser) -> Result<Vec<MemoryField>, MemoryQueryError> {
    let mut fields = vec![MemoryField::parse(&parser.word()?)?];
    while parser.eat_symbol(',') {
        fields.push(MemoryField::parse(&parser.word()?)?);
    }
    Ok(fields)
}

fn parse_value_list(parser: &mut Parser) -> Result<Vec<MemoryQueryValue>, MemoryQueryError> {
    let mut values = vec![parser.value()?];
    while parser.eat_symbol(',') {
        values.push(parser.value()?);
    }
    Ok(values)
}

fn parse_order_list(parser: &mut Parser) -> Result<Vec<SortExpression>, MemoryQueryError> {
    let mut orders = Vec::new();
    loop {
        let field = MemoryField::parse(&parser.word()?)?;
        let direction = if parser.eat_word("desc") {
            SortDirection::Descending
        } else {
            let _ = parser.eat_word("asc");
            SortDirection::Ascending
        };
        orders.push(SortExpression { field, direction });
        if !parser.eat_symbol(',') {
            break;
        }
    }
    Ok(orders)
}

fn parse_usize(parser: &mut Parser, clause: &str) -> Result<usize, MemoryQueryError> {
    match parser.next() {
        Some(Token::Number(value)) => value
            .parse()
            .map_err(|_| MemoryQueryError::new(format!("non_negative_integer:{clause}"))),
        _ => Err(MemoryQueryError::new(format!(
            "non_negative_integer:{clause}"
        ))),
    }
}

fn optional_alias(parser: &mut Parser) -> Result<Option<String>, MemoryQueryError> {
    if parser.eat_word("as") {
        return parser.word().map(Some);
    }
    Ok(None)
}

fn parse_aggregate_function(name: &str) -> Result<AggregateFunction, MemoryQueryError> {
    match name.to_ascii_lowercase().as_str() {
        "count" => Ok(AggregateFunction::Count),
        "sum" => Ok(AggregateFunction::Sum),
        "avg" | "average" => Ok(AggregateFunction::Average),
        "min" | "minimum" => Ok(AggregateFunction::Minimum),
        "max" | "maximum" => Ok(AggregateFunction::Maximum),
        "var_pop" | "variance" => Ok(AggregateFunction::PopulationVariance),
        "stddev_pop" | "stddev" => Ok(AggregateFunction::PopulationStandardDeviation),
        _ => Err(MemoryQueryError::new(format!(
            "unsupported_memory_aggregate:{name}"
        ))),
    }
}

fn default_alias(function: AggregateFunction, field: Option<MemoryField>) -> String {
    field.map_or_else(
        || function.as_str().to_owned(),
        |selected| format!("{}_{}", function.as_str(), selected.as_str()),
    )
}

fn expect_memory_table(parser: &mut Parser) -> Result<(), MemoryQueryError> {
    let table = parser.word()?;
    if matches!(
        table.to_ascii_lowercase().as_str(),
        "memory" | "memoryevent" | "memoryevents" | "memory_event" | "memory_events"
    ) {
        Ok(())
    } else {
        Err(MemoryQueryError::new(format!(
            "memory_table_not_allowed:{table}"
        )))
    }
}
