//! Native structural adapter draft. Semantic role selection comes exclusively
//! from canonical JS -> checked IR -> generated Rust. This CST adapter is
//! manually paired and qualified with shared behavior vectors. No source regex.
//! Intended owner: rust/src/agentic_coding/comparison_boundary.rs.
use super::source_lexical_policy::source_lexical_rule;
use crate::readable_source::{owned_source_links, source_from_links, source_owner};
use serde_json::{Value, json};

#[derive(Clone)]
pub struct BoundaryDefinition {
    pub declaration: String,
    pub bound: String,
    pub inclusive: bool,
}
#[derive(Clone)]
struct Operand {
    name: String,
    number: Option<f64>,
}
#[derive(Clone)]
struct Comparison {
    left: Operand,
    right: Operand,
    operator: String,
    path: Vec<usize>,
}
struct Selection {
    parameters: Vec<String>,
    pair: Vec<Comparison>,
    roles: Vec<String>,
    exported: bool,
}
fn rows(value: &Value) -> Result<&[Value], String> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| "BoundaryNodeRequired".to_owned())
}
fn field<'a>(node: &'a Value, key: &str) -> Result<Option<&'a Value>, String> {
    let found = rows(node)?
        .iter()
        .filter(|value| {
            value
                .as_array()
                .and_then(|row| row.first())
                .and_then(Value::as_str)
                == Some(key)
        })
        .collect::<Vec<_>>();
    if found.len() > 1 {
        return Err(format!("BoundaryFieldAmbiguous:{key}"));
    }
    match found.first() {
        Some(value) => Ok(Some(
            rows(value)?
                .get(1)
                .ok_or_else(|| "BoundaryFieldMalformed".to_owned())?,
        )),
        None => Ok(None),
    }
}
fn text_field(node: &Value, key: &str) -> Result<String, String> {
    Ok(field(node, key)?
        .and_then(|value| value.get("quoted"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned())
}
fn kind(node: &Value) -> Result<String, String> {
    text_field(node, "form")
}
fn role(node: &Value) -> Result<String, String> {
    text_field(node, "role")
}
fn children(node: &Value) -> Result<Vec<(usize, &Value)>, String> {
    Ok(rows(node)?
        .iter()
        .enumerate()
        .filter(|(_, value)| {
            value
                .as_array()
                .and_then(|row| row.first())
                .and_then(Value::as_str)
                .is_some_and(|key| {
                    !matches!(
                        key,
                        "form" | "role" | "text" | "language" | "path" | "meaning" | "semantic-gap"
                    )
                })
        })
        .collect())
}
fn concrete(node: &Value) -> Result<String, String> {
    if field(node, "text")?.is_some() {
        return text_field(node, "text");
    }
    children(node)?
        .into_iter()
        .map(|(_, child)| concrete(child))
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.concat())
}
fn at<'a>(node: &'a Value, path: &[usize]) -> Result<&'a Value, String> {
    path.iter().try_fold(node, |current, index| {
        rows(current)?
            .get(*index)
            .ok_or_else(|| "BoundaryPathInvalid".to_owned())
    })
}
fn replace_field(node: &mut Value, key: &str, text: &str) -> Result<(), String> {
    let found = node
        .as_array_mut()
        .ok_or_else(|| "BoundaryNodeRequired".to_owned())?
        .iter_mut()
        .find(|row| {
            row.as_array()
                .and_then(|items| items.first())
                .and_then(Value::as_str)
                == Some(key)
        })
        .ok_or_else(|| "BoundaryLeafFieldRequired".to_owned())?;
    found
        .as_array_mut()
        .ok_or_else(|| "BoundaryFieldRequired".to_owned())?[1] = json!({"quoted":text});
    Ok(())
}
fn at_mut<'a>(node: &'a mut Value, path: &[usize]) -> Result<&'a mut Value, String> {
    if let Some((head, tail)) = path.split_first() {
        let nested = node
            .as_array_mut()
            .and_then(|items| items.get_mut(*head))
            .ok_or_else(|| "BoundaryPathInvalid".to_owned())?;
        at_mut(nested, tail)
    } else {
        Ok(node)
    }
}
fn records(node: &Value, path: &[usize], output: &mut Vec<Vec<usize>>) -> Result<(), String> {
    if path.len() > 512 {
        return Err("BoundaryDepthLimit".to_owned());
    }
    output.push(path.to_vec());
    for (index, child) in children(node)? {
        let mut nested = path.to_vec();
        nested.push(index);
        records(child, &nested, output)?;
    }
    Ok(())
}
fn semantic<'a>(node: &'a Value, name: &str) -> Result<&'a Value, String> {
    let found = children(node)?
        .into_iter()
        .filter_map(|(_, child)| (role(child).ok().as_deref() == Some(name)).then_some(child))
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err(format!("BoundaryRoleRequired:{name}"));
    }
    Ok(found[0])
}
fn identifier(node: &Value) -> Result<String, String> {
    let name = concrete(node)?;
    if kind(node)? != "identifier"
        || field(node, "text")?.is_none()
        || name.is_empty()
        || !name.chars().enumerate().all(|(index, character)| {
            character.is_ascii_alphabetic()
                || character == '_'
                || character == '$'
                || index > 0 && character.is_ascii_digit()
        })
    {
        return Err("BoundarySimpleIdentifierRequired".to_owned());
    }
    Ok(name)
}
fn operand(node: &Value, parameters: &[String]) -> Result<Operand, String> {
    if kind(node)? == "identifier" {
        let name = identifier(node)?;
        if !parameters.contains(&name) {
            return Err("BoundaryFreeReference".to_owned());
        }
        return Ok(Operand { name, number: None });
    }
    if matches!(kind(node)?.as_str(), "numeric_literal" | "number") {
        // Same declared decimal literal subset in both roots; no expression evaluation.
        let raw = concrete(node)?;
        static DECIMAL: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let decimal = DECIMAL.get_or_init(|| {
            regex::Regex::new(
                r"^(?:[0-9][0-9_]*(?:\.[0-9_]*)?|\.[0-9][0-9_]*)(?:[eE][+-]?[0-9][0-9_]*)?$",
            )
            .expect("StaticBoundaryDecimalPattern")
        });
        if !decimal.is_match(&raw) {
            return Err("BoundaryDecimalLiteralRequired".to_owned());
        }
        let number = raw
            .replace('_', "")
            .parse::<f64>()
            .map_err(|_| "BoundaryFiniteLiteralRequired".to_owned())?;
        if !number.is_finite() {
            return Err("BoundaryFiniteLiteralRequired".to_owned());
        }
        return Ok(Operand {
            name: String::new(),
            number: Some(number),
        });
    }
    Err("BoundaryOperandUnsupported".to_owned())
}
fn insignificant(node: &Value) -> bool {
    kind(node).is_ok_and(|value| {
        matches!(
            value.as_str(),
            "whitespace" | "comment" | "line_comment" | "block_comment"
        )
    })
}
fn select(document: &Value, definition: &BoundaryDefinition) -> Result<Selection, String> {
    source_from_links(document)?;
    if source_owner(document)?.is_none() {
        return Err("BoundaryOwnedSourceRequired".to_owned());
    }
    let content = rows(document)?
        .get(2)
        .ok_or_else(|| "BoundaryOwnedSourceRequired".to_owned())?;
    if text_field(content, "language")? != "JavaScript" {
        return Err("BoundaryLanguageUnsupported".to_owned());
    }
    if !matches!(definition.bound.as_str(), "upper" | "lower")
        || definition.declaration.is_empty()
        || !definition
            .declaration
            .chars()
            .enumerate()
            .all(|(index, character)| {
                character.is_ascii_alphabetic()
                    || character == '_'
                    || character == '$'
                    || index > 0 && character.is_ascii_digit()
            })
    {
        return Err("BoundaryDefinitionInvalid".to_owned());
    }
    let mut all = Vec::new();
    records(document, &[], &mut all)?;
    let functions = all
        .iter()
        .filter_map(|path| {
            let node = at(document, path).ok()?;
            (kind(node).ok().as_deref() == Some("function_declaration")
                && children(node).ok()?.iter().any(|(_, child)| {
                    role(child).ok().as_deref() == Some("name")
                        && concrete(child).ok().as_deref() == Some(definition.declaration.as_str())
                }))
            .then_some(path)
        })
        .collect::<Vec<_>>();
    if functions.len() != 1 {
        return Err("BoundaryDeclarationAmbiguous".to_owned());
    }
    let function = at(document, functions[0])?;
    let mut nested = Vec::new();
    records(function, &[], &mut nested)?;
    for path in nested {
        let node = at(function, &path)?;
        if field(node, "meaning")?.is_some() || field(node, "semantic-gap")?.is_some() {
            return Err("BoundaryUnresolvedMeaning".to_owned());
        }
    }
    if children(function)?
        .iter()
        .any(|(_, child)| matches!(concrete(child).unwrap_or_default().as_str(), "async" | "*"))
    {
        return Err("BoundaryAsyncOrGeneratorUnsupported".to_owned());
    }
    let mut parameter_nodes = children(function)?
        .into_iter()
        .filter_map(|(_, child)| (role(child).ok().as_deref() == Some("params")).then_some(child))
        .collect::<Vec<_>>();
    if parameter_nodes.is_empty() {
        let group = semantic(function, "parameters")?;
        if kind(group)? != "formal_parameters" {
            return Err("BoundaryParametersUnsupported".to_owned());
        }
        parameter_nodes = children(group)?
            .into_iter()
            .map(|(_, child)| child)
            .filter(|child| {
                !insignificant(child)
                    && !matches!(
                        concrete(child).unwrap_or_default().as_str(),
                        "(" | ")" | ","
                    )
            })
            .collect();
    }
    let parameters = parameter_nodes
        .into_iter()
        .map(identifier)
        .collect::<Result<Vec<_>, _>>()?;
    if parameters.is_empty()
        || parameters.len() > 3
        || parameters
            .iter()
            .enumerate()
            .any(|(index, name)| parameters[..index].contains(name))
    {
        return Err("BoundaryParametersUnsupported".to_owned());
    }
    let body = semantic(function, "body")?;
    let statements = children(body)?
        .into_iter()
        .map(|(_, child)| child)
        .filter(|child| {
            !insignificant(child)
                && !matches!(concrete(child).unwrap_or_default().as_str(), "{" | "}")
        })
        .collect::<Vec<_>>();
    if !matches!(kind(body)?.as_str(), "block_statement" | "statement_block")
        || statements.len() != 1
        || kind(statements[0])? != "return_statement"
    {
        return Err("BoundarySingleReturnRequired".to_owned());
    }
    let mut expressions = children(statements[0])?
        .into_iter()
        .filter_map(|(_, child)| (role(child).ok().as_deref() == Some("argument")).then_some(child))
        .collect::<Vec<_>>();
    if expressions.is_empty() {
        expressions = children(statements[0])?
            .into_iter()
            .map(|(_, child)| child)
            .filter(|child| {
                !insignificant(child)
                    && !matches!(concrete(child).unwrap_or_default().as_str(), "return" | ";")
            })
            .collect();
    }
    if expressions.len() != 1 {
        return Err("BoundaryReturnExpressionAmbiguous".to_owned());
    }
    let expression = expressions[0];
    let conjunction = children(expression)?
        .into_iter()
        .map(|(_, child)| child)
        .filter(|child| text_field(child, "text").ok().as_deref() == Some("&&"))
        .collect::<Vec<_>>();
    if !matches!(
        kind(expression)?.as_str(),
        "logical_expression" | "binary_expression"
    ) || conjunction.len() != 1
    {
        return Err("BoundaryConjunctionRequired".to_owned());
    }
    let expression_left = semantic(expression, "left")?;
    let expression_right = semantic(expression, "right")?;
    if children(expression)?.iter().any(|(_, child)| {
        !insignificant(child)
            && !std::ptr::eq(*child, expression_left)
            && !std::ptr::eq(*child, expression_right)
            && !std::ptr::eq(*child, conjunction[0])
    }) {
        return Err("BoundaryConjunctionRequired".to_owned());
    }
    let mut pair = Vec::new();
    for side in ["left", "right"] {
        let comparison = semantic(expression, side)?;
        if kind(comparison)? != "binary_expression" {
            return Err("BoundaryComparisonRequired".to_owned());
        }
        let operators = children(comparison)?
            .into_iter()
            .map(|(_, child)| child)
            .filter(|child| {
                text_field(child, "text")
                    .is_ok_and(|value| matches!(value.as_str(), "<" | "<=" | ">" | ">="))
            })
            .collect::<Vec<_>>();
        if operators.len() != 1 {
            return Err("BoundaryOperatorAmbiguous".to_owned());
        }
        let comparison_left = semantic(comparison, "left")?;
        let comparison_right = semantic(comparison, "right")?;
        if children(comparison)?.iter().any(|(_, child)| {
            !insignificant(child)
                && !std::ptr::eq(*child, comparison_left)
                && !std::ptr::eq(*child, comparison_right)
                && !std::ptr::eq(*child, operators[0])
        }) {
            return Err("BoundaryComparisonResidual".to_owned());
        }
        let operator_path = all
            .iter()
            .find(|path| at(document, path).is_ok_and(|node| std::ptr::eq(node, operators[0])))
            .ok_or_else(|| "BoundaryOperatorPathRequired".to_owned())?
            .clone();
        pair.push(Comparison {
            left: operand(semantic(comparison, "left")?, &parameters)?,
            right: operand(semantic(comparison, "right")?, &parameters)?,
            operator: concrete(operators[0])?,
            path: operator_path,
        });
    }
    let roles =
        crate::agentic_coding::comparison_boundary_generated::comparison_boundary_role_kernel(
            pair[0].left.name.clone(),
            pair[0].right.name.clone(),
            pair[1].left.name.clone(),
            pair[1].right.name.clone(),
            pair[0].operator.clone(),
            pair[1].operator.clone(),
            definition.bound.clone(),
            definition.inclusive,
        );
    if roles.first().map(String::as_str) != Some("selected") {
        return Err(roles
            .get(1)
            .cloned()
            .unwrap_or_else(|| "BoundaryRolesUnavailable".to_owned()));
    }
    let exported = functions[0].split_last().is_some_and(|(_, path)| {
        at(document, path).is_ok_and(|parent| {
            kind(parent).is_ok_and(|value| {
                matches!(
                    value.as_str(),
                    "export_named_declaration" | "export_statement"
                )
            }) && children(parent).is_ok_and(|items| {
                !items
                    .iter()
                    .any(|(_, child)| concrete(child).ok().as_deref() == Some("default"))
            })
        })
    });
    Ok(Selection {
        parameters,
        pair,
        roles,
        exported,
    })
}

/// The parser callback is supplied by the registered grammar host. It receives
/// source bytes and returns the same owned recursive source model; no authority
/// is inferred from caller strings or a solver-provided serialization.
pub fn derive_comparison_boundary_candidate(
    document: &Value,
    definition: &BoundaryDefinition,
    parse_owned: impl Fn(&str, &str) -> Result<Value, String>,
) -> Result<Value, String> {
    let owner = source_owner(document)?.ok_or_else(|| "BoundaryOwnedSourceRequired".to_owned())?;
    if parse_owned(&source_from_links(document)?, owner)? != *document {
        return Err("BoundaryPreimageReparseMismatch".to_owned());
    }
    let observed = select(document, definition)?;
    let selected = if observed.roles[1] == "0" { 0 } else { 1 };
    let path = &observed.pair[selected].path;
    let before = &observed.roles[3];
    let after = &observed.roles[4];
    let before_form = kind(at(document, path)?)?;
    let mut changed = document.clone();
    replace_field(at_mut(&mut changed, path)?, "text", after)?;
    if before_form == *before {
        replace_field(at_mut(&mut changed, path)?, "form", after)?;
    }
    let source = source_from_links(&changed)?;
    if parse_owned(&source, owner)? != changed {
        return Err("BoundaryIndependentReparseMismatch".to_owned());
    }
    let mut restored = changed.clone();
    replace_field(at_mut(&mut restored, path)?, "text", before)?;
    replace_field(at_mut(&mut restored, path)?, "form", &before_form)?;
    if restored != *document {
        return Err("BoundaryConcreteResidualMismatch".to_owned());
    }
    let mut operator_path = vec![json!("operator-path")];
    operator_path.extend(path.iter().map(|index| json!({"quoted":index.to_string()})));
    Ok(
        json!({"document":changed,"source":source,"changed":before!=after,
        "rule":["comparison-boundary-transform",["owner",{"quoted":owner}],
            ["declaration",{"quoted":definition.declaration}],["bound",{"quoted":definition.bound}],
            ["inclusive",{"quoted":definition.inclusive.to_string()}],["preimage",document],
            operator_path,
            ["before",{"quoted":before}],["after",{"quoted":after}]],
        "verification":{"schema":"comparison-boundary-proof/v1","completeConcreteResidual":true,
            "independentGrammarReparse":true,"exactInverse":true,"sharedSubject":observed.roles[2],
            "bound":definition.bound,"inclusive":definition.inclusive,"scope":"finite-number-interval-only",
            "arbitrarySemanticEquivalence":false,"compilerValidity":"not-checked","authorshipCredit":0}}),
    )
}

/// Host convenience retains ownership and actual source in the native model.
pub fn native_comparison_boundary_candidate(
    source: &str,
    owner: &str,
    definition: &BoundaryDefinition,
) -> Result<Value, String> {
    let parse = |text: &str, path: &str| {
        owned_source_links(
            path,
            &crate::readable_source::source_to_links(text, "JavaScript")?,
        )
    };
    derive_comparison_boundary_candidate(&parse(source, owner)?, definition, parse)
}

pub fn select_comparison_boundary_definition(
    document: &Value,
    bound: &str,
    inclusive: bool,
) -> Result<BoundaryDefinition, String> {
    let mut all = Vec::new();
    records(document, &[], &mut all)?;
    let mut candidates = Vec::new();
    for path in all {
        let node = at(document, &path)?;
        if kind(node)? != "function_declaration" {
            continue;
        }
        let Ok(name) = semantic(node, "name").and_then(identifier) else {
            continue;
        };
        let definition = BoundaryDefinition {
            declaration: name,
            bound: bound.to_owned(),
            inclusive,
        };
        if select(document, &definition).is_ok() {
            candidates.push(definition);
        }
    }
    if candidates.len() != 1 {
        return Err("BoundaryUniqueSuitableDeclarationRequired".to_owned());
    }
    candidates
        .pop()
        .ok_or_else(|| "BoundaryUniqueSuitableDeclarationRequired".to_owned())
}

/// Contract-derived test construction runs before candidate selection. The
/// caller still owns an independent acceptance checker outside this artifact.
pub fn derive_comparison_boundary_regression(
    document: &Value,
    definition: &BoundaryDefinition,
    module_specifier: &str,
    parse_owned: impl Fn(&str, &str) -> Result<Value, String>,
) -> Result<Value, String> {
    let owner = source_owner(document)?.ok_or_else(|| "BoundaryOwnedSourceRequired".to_owned())?;
    if parse_owned(&source_from_links(document)?, owner)? != *document {
        return Err("BoundaryPreimageReparseMismatch".to_owned());
    }
    if source_lexical_rule(module_specifier, "relative-module-prefix") != Some(true)
        || !(module_specifier.ends_with(".mjs") || module_specifier.ends_with(".js"))
        || !module_specifier
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_./-".contains(character))
    {
        return Err("BoundaryRegressionModuleUnsupported".to_owned());
    }
    let observed = select(document, definition)?;
    if !observed.exported {
        return Err("BoundaryRegressionNamedExportRequired".to_owned());
    }
    let subject = &observed.roles[2];
    let lower_index = if observed.roles[5] == "0" { 0 } else { 1 };
    let upper_index = if observed.roles[6] == "0" { 0 } else { 1 };
    let bound_operand = |index: usize| {
        if observed.pair[index].left.name == *subject {
            observed.pair[index].right.clone()
        } else {
            observed.pair[index].left.clone()
        }
    };
    let lower = bound_operand(lower_index);
    let upper = bound_operand(upper_index);
    let low = lower.number.unwrap_or_else(|| {
        upper
            .number
            .map_or(0.0, |high| high - 1.0_f64.max(high.abs() / 2.0))
    });
    let high = upper
        .number
        .unwrap_or_else(|| low + 2.0_f64.max(low.abs() / 2.0));
    let width = high - low;
    let middle = low + width / 2.0;
    if ![low, high, width, middle]
        .iter()
        .all(|value| value.is_finite())
        || !(low < middle && middle < high)
    {
        return Err("BoundaryFiniteWitnessUnavailable".to_owned());
    }
    let lower_inclusive = if definition.bound == "lower" {
        definition.inclusive
    } else {
        observed.roles[7] == ">="
    };
    let upper_inclusive = if definition.bound == "upper" {
        definition.inclusive
    } else {
        observed.roles[8] == "<="
    };
    let forms: Value = serde_json::from_str(include_str!(
        "../../embedded/data/meta/comparison-boundary-regression.json"
    ))
    .map_err(|_| "BoundaryRegressionFormsRequired".to_owned())?;
    let render = |key: &str, bindings: &[(&str, String)]| -> Result<String, String> {
        let template = forms[key]
            .as_str()
            .ok_or_else(|| "BoundaryRegressionFormsRequired".to_owned())?;
        Ok(bindings
            .iter()
            .fold(template.to_owned(), |text, (name, value)| {
                text.replace(&format!("{{{name}}}"), value)
            }))
    };
    let witnesses = [low - width, low, middle, high, high + width];
    let mut samples = Vec::new();
    let mut assertions = Vec::new();
    for value in witnesses {
        let expected = crate::agentic_coding::comparison_boundary_generated::comparison_boundary_witness_kernel(
            value,
            low,
            high,
            lower_inclusive,
            upper_inclusive,
        );
        let argument_values = observed
            .parameters
            .iter()
            .map(|parameter| {
                if parameter == subject {
                    Ok(value)
                } else if parameter == &lower.name {
                    Ok(low)
                } else if parameter == &upper.name {
                    Ok(high)
                } else {
                    Err("BoundaryUnusedParameterUnsupported".to_owned())
                }
            })
            .collect::<Result<Vec<f64>, String>>()?;
        if !argument_values.iter().all(|value| value.is_finite()) {
            return Err("BoundaryFiniteWitnessUnavailable".to_owned());
        }
        assertions.push(render(
            "assertion",
            &[
                (
                    "arguments",
                    argument_values
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                ("expected", expected.to_string()),
            ],
        )?);
        samples.push(json!({"args":argument_values,"expected":expected}));
    }
    let polarity = forms["polarities"][if definition.inclusive {
        "inclusive"
    } else {
        "exclusive"
    }]
    .as_str()
    .ok_or_else(|| "BoundaryRegressionFormsRequired".to_owned())?;
    let name = render(
        "title",
        &[
            ("bound", definition.bound.clone()),
            ("polarity", polarity.to_owned()),
        ],
    )?;
    let imports = forms["imports"]
        .as_array()
        .ok_or_else(|| "BoundaryRegressionFormsRequired".to_owned())?
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| "BoundaryRegressionFormsRequired".to_owned())?
        .join("\n");
    let binding = render(
        "binding",
        &[
            ("declaration", definition.declaration.clone()),
            (
                "module",
                serde_json::to_string(module_specifier).map_err(|error| error.to_string())?,
            ),
        ],
    )?;
    let opening = render(
        "open",
        &[(
            "name",
            serde_json::to_string(&name).map_err(|error| error.to_string())?,
        )],
    )?;
    let closing = forms["close"]
        .as_str()
        .ok_or_else(|| "BoundaryRegressionFormsRequired".to_owned())?;
    let source = [
        imports,
        binding,
        String::new(),
        opening,
        assertions.join("\n"),
        closing.to_owned(),
        String::new(),
    ]
    .join("\n");
    parse_owned(&source, owner)?;
    Ok(
        json!({"source":source,"samples":samples,"contract":{"schema":"comparison-boundary-regression/v1",
        "declaration":definition.declaration,"subject":subject,"bound":definition.bound,
        "inclusive":definition.inclusive,"scope":"finite-number-interval-only",
        "independentAcceptance":false,"authorshipCredit":0}}),
    )
}

pub fn native_comparison_boundary_regression(
    source: &str,
    owner: &str,
    definition: &BoundaryDefinition,
    module_specifier: &str,
) -> Result<Value, String> {
    let parse = |text: &str, path: &str| {
        owned_source_links(
            path,
            &crate::readable_source::source_to_links(text, "JavaScript")?,
        )
    };
    derive_comparison_boundary_regression(
        &parse(source, owner)?,
        definition,
        module_specifier,
        parse,
    )
}

fn relative_boundary_owner(path: &str) -> bool {
    !path.is_empty()
        && (path.ends_with(".js") || path.ends_with(".mjs"))
        && path
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_./-".contains(character))
        && path
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}
/// Native counterpart of the installed JS host operation, with identical wire
/// fields and current-source provenance. No installed Node capability required.
pub fn observed_comparison_boundary_operation(request: &Value) -> Result<Value, String> {
    let source = request["source"]
        .as_str()
        .ok_or_else(|| "BoundaryObservedRequestRequired".to_owned())?;
    let owner = request["owner"]
        .as_str()
        .filter(|value| relative_boundary_owner(value))
        .ok_or_else(|| "BoundaryObservedRequestRequired".to_owned())?;
    let mode = request["mode"]
        .as_str()
        .filter(|value| matches!(*value, "repair" | "regression"))
        .ok_or_else(|| "BoundaryObservedRequestRequired".to_owned())?;
    if source.len() > 131072 {
        return Err("BoundaryObservedRequestRequired".to_owned());
    }
    let bound = request["bound"]
        .as_str()
        .ok_or_else(|| "BoundaryDefinitionInvalid".to_owned())?;
    let inclusive = request["inclusive"]
        .as_bool()
        .ok_or_else(|| "BoundaryDefinitionInvalid".to_owned())?;
    let parse = |text: &str, path: &str| {
        owned_source_links(
            path,
            &crate::readable_source::source_to_links(text, "JavaScript")?,
        )
    };
    let document = parse(source, owner)?;
    let definition = if let Some(declaration) = request["declaration"].as_str() {
        BoundaryDefinition {
            declaration: declaration.to_owned(),
            bound: bound.to_owned(),
            inclusive,
        }
    } else {
        select_comparison_boundary_definition(&document, bound, inclusive)?
    };
    let digest = |text: &str| crate::source_fetch::sha256_hex(text.as_bytes());
    if mode == "repair" {
        let result = derive_comparison_boundary_candidate(&document, &definition, parse)?;
        let after = result["source"]
            .as_str()
            .ok_or_else(|| "BoundaryCandidateSourceRequired".to_owned())?;
        return Ok(
            json!({"schema":"observed-comparison-boundary-operation/v1","mode":"repair","owner":owner,
            "candidate":{"path":owner,"source":after,"beforeSha256":digest(source),"afterSha256":digest(after),"changed":result["changed"]},
            "rule":result["rule"],"sourceModel":result["document"],"verification":result["verification"],"authored":false,"independentAcceptance":false}),
        );
    }
    let test_owner = request["testOwner"]
        .as_str()
        .filter(|value| relative_boundary_owner(value) && *value != owner)
        .ok_or_else(|| "BoundaryRegressionTargetRequired".to_owned())?;
    let mut source_parts = owner.split('/').collect::<Vec<_>>();
    let mut target_parts = test_owner.split('/').collect::<Vec<_>>();
    target_parts.pop();
    while !source_parts.is_empty() && !target_parts.is_empty() && source_parts[0] == target_parts[0]
    {
        source_parts.remove(0);
        target_parts.remove(0);
    }
    let mut module_specifier = "../".repeat(target_parts.len()) + &source_parts.join("/");
    if !module_specifier.starts_with('.') {
        module_specifier = "./".to_owned() + &module_specifier;
    }
    let result =
        derive_comparison_boundary_regression(&document, &definition, &module_specifier, parse)?;
    let after = result["source"]
        .as_str()
        .ok_or_else(|| "BoundaryCandidateSourceRequired".to_owned())?;
    Ok(
        json!({"schema":"observed-comparison-boundary-operation/v1","mode":"regression","owner":owner,
        "candidate":{"path":test_owner,"source":after,"sourceOwnerSha256":digest(source),"afterSha256":digest(after)},
        "sourceModel":document,"samples":result["samples"],"verification":result["contract"],"authored":false,"independentAcceptance":false}),
    )
}
