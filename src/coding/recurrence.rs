//! Source-backed formalization and rendering of recursive definitions.
//!
//! Wikifunctions identifiers are intentionally opaque here. Their fetched
//! labels are resolved through seed word forms whose `action` fields name the
//! language-neutral operations. This lets the same algorithm formalize new
//! recurrence families without adding source-object IDs or benchmark prompts.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::coding::function_catalog::wikifunctions::{
    AbstractImplementation, FunctionPart, FunctionTest, ZExpression,
};
use crate::links_format::push_lino_node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Conditional,
    LessEqual,
    Equal,
    Add,
    Multiply,
    Subtract,
    SubtractOne,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Parameter(String),
    Literal(i64),
    Recur(Box<Expression>),
    Apply(Operation, Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recurrence {
    pub source_function_zid: String,
    pub source_implementation_zid: String,
    pub source_label: String,
    pub parameter: String,
    pub expression: Expression,
    pub predecessor_offsets: Vec<u64>,
    pub source_url: String,
    pub source_sha256: String,
    pub fetched_at: String,
    pub license: String,
    pub operator_sources: Vec<String>,
}

impl Recurrence {
    /// Derive a valid concise callable name from the fetched source label.
    #[must_use]
    pub fn suggested_identifier(&self) -> String {
        identifier_from_source_label(&self.source_label)
    }

    #[must_use]
    pub fn render_python(&self, function_name: &str) -> String {
        let mut out = format!("def {function_name}({}):\n", self.parameter);
        let expression = render_expression(&self.expression, function_name);
        let _ = writeln!(out, "    return {expression}");
        out.trim_end().to_owned()
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(
            &mut out,
            0,
            "source_observation",
            Some(&self.source_function_zid),
        );
        push_lino_node(&mut out, 2, "label", Some(&self.source_label));
        push_lino_node(
            &mut out,
            2,
            "abstract_implementation",
            Some(&self.source_implementation_zid),
        );
        push_lino_node(&mut out, 2, "license", Some(&self.license));
        push_lino_node(&mut out, 2, "source_url", Some(&self.source_url));
        push_lino_node(&mut out, 2, "sha256", Some(&self.source_sha256));
        push_lino_node(&mut out, 2, "fetched_at", Some(&self.fetched_at));
        for source in &self.operator_sources {
            push_lino_node(&mut out, 2, "operator_source", Some(source));
        }
        push_lino_node(
            &mut out,
            0,
            "derived_formalization",
            Some(&self.source_function_zid),
        );
        push_lino_node(&mut out, 2, "parameter", Some(&self.parameter));
        push_lino_node(
            &mut out,
            2,
            "expression",
            Some(&serialize_expression(&self.expression)),
        );
        push_lino_node(&mut out, 2, "termination_measure", Some(&self.parameter));
        for offset in &self.predecessor_offsets {
            push_lino_node(&mut out, 2, "predecessor_offset", Some(&offset.to_string()));
        }
        out.trim_end().to_owned()
    }
}

#[must_use]
pub fn identifier_from_source_label(label: &str) -> String {
    let title_word = label
        .split(|character: char| !character.is_alphanumeric())
        .find(|word| word.chars().next().is_some_and(char::is_uppercase));
    let source = title_word.unwrap_or(label);
    let rendered = source
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    rendered.trim_matches('_').to_owned()
}

/// Resolve a fetched abstract implementation into a recurrence and prove that
/// every self call decreases the single natural-number parameter.
pub fn formalize(
    definition: &FunctionPart,
    implementation: &AbstractImplementation,
    operator_descriptors: &[FunctionPart],
) -> Result<Recurrence, String> {
    if implementation.function_zid != definition.zid {
        return Err("abstract implementation targets a different function".to_owned());
    }
    if definition.argument_keys.len() != 1 {
        return Err("only single-measure recurrences are currently bounded".to_owned());
    }
    let source_key = &definition.argument_keys[0];
    let parameter = "n";
    let descriptors = operator_descriptors
        .iter()
        .map(|descriptor| (descriptor.zid.as_str(), descriptor))
        .collect::<BTreeMap<_, _>>();
    let expression = resolve_expression(
        &implementation.expression,
        &definition.zid,
        source_key,
        parameter,
        &descriptors,
    )?;
    let Expression::Apply(Operation::Conditional, branches) = &expression else {
        return Err("recurrence root is not a conditional".to_owned());
    };
    if branches.len() != 3 {
        return Err("conditional does not have condition/then/else branches".to_owned());
    }
    if contains_recur(&branches[1]) {
        return Err("boundary result contains a recursive call".to_owned());
    }
    validate_boundary(&branches[0], parameter)?;
    let mut offsets = Vec::new();
    collect_offsets(&branches[2], parameter, &mut offsets)?;
    if offsets.is_empty() {
        return Err("transition has no recursive call".to_owned());
    }
    offsets.sort_unstable();
    offsets.dedup();
    let mut operator_sources = operator_descriptors
        .iter()
        .map(|descriptor| descriptor.source_url.clone())
        .collect::<Vec<_>>();
    operator_sources.sort();
    operator_sources.dedup();
    Ok(Recurrence {
        source_function_zid: definition.zid.clone(),
        source_implementation_zid: implementation.zid.clone(),
        source_label: definition.labels.get("en").cloned().unwrap_or_default(),
        parameter: parameter.to_owned(),
        expression,
        predecessor_offsets: offsets,
        source_url: implementation.source_url.clone(),
        source_sha256: implementation.sha256.clone(),
        fetched_at: implementation.fetched_at.clone(),
        license: implementation.license.clone(),
        operator_sources,
    })
}

fn resolve_expression(
    source: &ZExpression,
    self_zid: &str,
    source_parameter: &str,
    parameter: &str,
    descriptors: &BTreeMap<&str, &FunctionPart>,
) -> Result<Expression, String> {
    match source {
        ZExpression::Reference(key) if key == source_parameter => {
            Ok(Expression::Parameter(parameter.to_owned()))
        }
        ZExpression::Reference(key) => Err(format!("unknown source reference {key}")),
        ZExpression::Literal(value) => value
            .parse::<i64>()
            .map(Expression::Literal)
            .map_err(|_| format!("unsupported non-integer literal {value}")),
        ZExpression::Call {
            function_zid,
            arguments,
        } if function_zid == self_zid => {
            let argument = arguments
                .values()
                .next()
                .ok_or_else(|| "recursive call has no argument".to_owned())?;
            Ok(Expression::Recur(Box::new(resolve_expression(
                argument,
                self_zid,
                source_parameter,
                parameter,
                descriptors,
            )?)))
        }
        ZExpression::Call {
            function_zid,
            arguments,
        } => {
            let descriptor = descriptors
                .get(function_zid.as_str())
                .ok_or_else(|| format!("missing descriptor for source function {function_zid}"))?;
            let operation = operation_for_descriptor(descriptor)?;
            let ordered = descriptor
                .argument_keys
                .iter()
                .map(|key| {
                    arguments
                        .get(key)
                        .ok_or_else(|| format!("source call is missing argument {key}"))
                        .and_then(|argument| {
                            resolve_expression(
                                argument,
                                self_zid,
                                source_parameter,
                                parameter,
                                descriptors,
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Expression::Apply(operation, ordered))
        }
    }
}

fn operation_for_descriptor(descriptor: &FunctionPart) -> Result<Operation, String> {
    let normalized_labels = descriptor
        .labels
        .values()
        .map(|label| crate::engine::normalize_prompt(label))
        .collect::<Vec<_>>();
    for meaning in
        crate::seed::lexicon().meanings_with_role(crate::seed::ROLE_CODING_RECURRENCE_OPERATOR)
    {
        if !normalized_labels
            .iter()
            .any(|label| meaning.evidenced_in(label))
        {
            continue;
        }
        for form in meaning.word_forms() {
            let operation = match form.action.as_str() {
                "conditional" => Some(Operation::Conditional),
                "less_equal" => Some(Operation::LessEqual),
                "equal" => Some(Operation::Equal),
                "add" => Some(Operation::Add),
                "multiply" => Some(Operation::Multiply),
                "subtract" => Some(Operation::Subtract),
                "subtract_one" => Some(Operation::SubtractOne),
                _ => None,
            };
            if let Some(operation) = operation {
                return Ok(operation);
            }
        }
    }
    Err(format!(
        "no seeded operation matches fetched label for {}",
        descriptor.zid
    ))
}

fn validate_boundary(expression: &Expression, parameter: &str) -> Result<(), String> {
    let Expression::Apply(operation @ (Operation::LessEqual | Operation::Equal), operands) =
        expression
    else {
        return Err("boundary condition is not an equality or ordered bound".to_owned());
    };
    if operands.len() != 2
        || !matches!(&operands[0], Expression::Parameter(name) if name == parameter)
        || !matches!(operands[1], Expression::Literal(value) if value >= 0)
    {
        return Err(format!("invalid {operation:?} boundary operands"));
    }
    Ok(())
}

fn contains_recur(expression: &Expression) -> bool {
    match expression {
        Expression::Recur(_) => true,
        Expression::Apply(_, operands) => operands.iter().any(contains_recur),
        Expression::Parameter(_) | Expression::Literal(_) => false,
    }
}

fn collect_offsets(
    expression: &Expression,
    parameter: &str,
    offsets: &mut Vec<u64>,
) -> Result<(), String> {
    match expression {
        Expression::Recur(argument) => {
            offsets.push(predecessor_offset(argument, parameter)?);
            Ok(())
        }
        Expression::Apply(_, operands) => {
            for operand in operands {
                collect_offsets(operand, parameter, offsets)?;
            }
            Ok(())
        }
        Expression::Parameter(_) | Expression::Literal(_) => Ok(()),
    }
}

fn predecessor_offset(expression: &Expression, parameter: &str) -> Result<u64, String> {
    match expression {
        Expression::Apply(Operation::SubtractOne, operands) if matches!(operands.as_slice(), [Expression::Parameter(name)] if name == parameter) => {
            Ok(1)
        }
        Expression::Apply(Operation::Subtract, operands) if matches!(operands.first(), Some(Expression::Parameter(name)) if name == parameter) =>
        {
            let Some(Expression::Literal(offset)) = operands.get(1) else {
                return Err("recursive subtraction has a non-literal offset".to_owned());
            };
            u64::try_from(*offset)
                .ok()
                .filter(|offset| *offset > 0)
                .ok_or_else(|| "recursive argument does not strictly decrease".to_owned())
        }
        _ => Err("recursive argument is not a proven predecessor".to_owned()),
    }
}

fn render_expression(expression: &Expression, function_name: &str) -> String {
    match expression {
        Expression::Parameter(name) => name.clone(),
        Expression::Literal(value) => value.to_string(),
        Expression::Recur(argument) => {
            let rendered = render_expression(argument, function_name);
            let compact = rendered
                .strip_prefix('(')
                .and_then(|value| value.strip_suffix(')'))
                .unwrap_or(&rendered);
            format!("{function_name}({compact})")
        }
        Expression::Apply(operation, operands) => {
            let rendered = operands
                .iter()
                .map(|operand| render_expression(operand, function_name))
                .collect::<Vec<_>>();
            match operation {
                Operation::Conditional if rendered.len() == 3 => {
                    format!("{} if {} else {}", rendered[1], rendered[0], rendered[2])
                }
                Operation::LessEqual => binary(&rendered, "<="),
                Operation::Equal => binary(&rendered, "=="),
                Operation::Add => binary(&rendered, "+"),
                Operation::Multiply => binary(&rendered, "*"),
                Operation::Subtract => binary(&rendered, "-"),
                Operation::SubtractOne if rendered.len() == 1 => format!("{} - 1", rendered[0]),
                _ => String::new(),
            }
        }
    }
}

fn binary(operands: &[String], symbol: &str) -> String {
    if operands.len() == 2 {
        format!("({} {symbol} {})", operands[0], operands[1])
    } else {
        String::new()
    }
}

fn serialize_expression(expression: &Expression) -> String {
    match expression {
        Expression::Parameter(name) => format!("parameter({name})"),
        Expression::Literal(value) => format!("literal({value})"),
        Expression::Recur(argument) => format!("recur({})", serialize_expression(argument)),
        Expression::Apply(operation, operands) => {
            let name = match operation {
                Operation::Conditional => "conditional",
                Operation::LessEqual => "less_equal",
                Operation::Equal => "equal",
                Operation::Add => "add",
                Operation::Multiply => "multiply",
                Operation::Subtract => "subtract",
                Operation::SubtractOne => "subtract_one",
            };
            format!(
                "{name}({})",
                operands
                    .iter()
                    .map(serialize_expression)
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}

#[must_use]
pub fn python_assertions(
    recurrence: &Recurrence,
    function_name: &str,
    tests: &[FunctionTest],
) -> String {
    let mut script = recurrence.render_python(function_name);
    for test in tests
        .iter()
        .filter(|test| test.function_zid == recurrence.source_function_zid)
    {
        if test.arguments.len() == 1 {
            let _ = writeln!(
                script,
                "\nassert {function_name}({}) == {}",
                test.arguments[0], test.expected
            );
        }
    }
    script.trim_end().to_owned()
}
