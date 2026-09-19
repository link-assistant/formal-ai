//! The Python lowering backend (issue #1138, plan 02 L6).
//!
//! `python_render::render_function` moves behind [`LanguageLowering`]; the
//! per-language surface forms stay data, read from the fragment catalog.

use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::ir_lowering::{LanguageLowering, LoweringGap};
use crate::coding::program_ir::{IrNode, IrType, ProgramIr};
use crate::coding::python_render::runtime_template;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PythonLowering;

impl LanguageLowering for PythonLowering {
    fn language(&self) -> &'static str {
        "python"
    }

    fn lower(&self, ir: &ProgramIr, catalog: &FragmentCatalog) -> Result<String, LoweringGap> {
        let (expression, modules) = hoist_inline_imports(&lower_node(&ir.body, catalog)?);
        let imports = modules
            .iter()
            .map(|module| render("python_import", &[("module", module)]))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        let prepend = |program: String| {
            if imports.is_empty() {
                program
            } else {
                format!("{imports}\n{program}")
            }
        };
        if ir.name.is_empty() {
            return Ok(prepend(expression));
        }
        let parameters = ir
            .parameters
            .iter()
            .map(|(name, ty)| match python_annotation(ty) {
                Some(annotation) => format!("{}: {}", python_identifier(name), annotation),
                None => python_identifier(name),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let function = render(
            "python_ir_function",
            &[
                ("name", &python_identifier(&ir.name)),
                ("parameters", &parameters),
                ("expression", &expression),
            ],
        )?;
        // A hoisted import block and a top-level definition are separate
        // paragraphs (PEP 8); a bare expression stays glued to its imports.
        Ok(if imports.is_empty() {
            function
        } else {
            format!("{imports}\n\n{function}")
        })
    }
}

/// The target-language spelling of a parameter's type, re-derived from the
/// typed signature the search unified — never from the prompt's prose. An
/// untyped parameter lowers without an annotation.
fn python_annotation(ty: &IrType) -> Option<String> {
    match ty {
        IrType::Integer => Some(String::from("int")),
        IrType::Float => Some(String::from("float")),
        IrType::Boolean => Some(String::from("bool")),
        IrType::Text => Some(String::from("str")),
        IrType::Sequence(element) | IrType::OrderedSequence(element) => {
            Some(match python_annotation(element) {
                Some(inner) => format!("list[{inner}]"),
                None => String::from("list"),
            })
        }
        IrType::Pair(left, right) => {
            Some(match (python_annotation(left), python_annotation(right)) {
                (Some(left), Some(right)) => format!("tuple[{left}, {right}]"),
                _ => String::from("tuple"),
            })
        }
        IrType::Mapping(key, value) => {
            Some(match (python_annotation(key), python_annotation(value)) {
                (Some(key), Some(value)) => format!("dict[{key}, {value}]"),
                _ => String::from("dict"),
            })
        }
        IrType::Callable | IrType::Unknown(_) => None,
    }
}

/// Seed surfaces keep `__import__('module')` inline so each stays a single
/// expression; the Python backend is the place that decides the idiomatic
/// module form (PEP 8 imports at the top), by rewriting attribute access and
/// collecting one import per module.
fn hoist_inline_imports(expression: &str) -> (String, Vec<String>) {
    const NEEDLE: &str = "__import__('";
    let mut modules = std::collections::BTreeSet::new();
    let mut lowered = String::with_capacity(expression.len());
    let mut rest = expression;
    while let Some(start) = rest.find(NEEDLE) {
        lowered.push_str(&rest[..start]);
        let after = &rest[start + NEEDLE.len()..];
        let Some(end) = after.find('\'') else {
            break;
        };
        let module = &after[..end];
        match after[end + 1..].strip_prefix(").") {
            Some(attribute) => {
                modules.insert(module.to_owned());
                lowered.push_str(module);
                lowered.push('.');
                rest = attribute;
            }
            None => {
                lowered.push_str(&rest[start..start + NEEDLE.len() + end + 1]);
                rest = &after[end + 1..];
            }
        }
    }
    lowered.push_str(rest);
    (lowered, modules.into_iter().collect())
}

fn lower_node(node: &IrNode, catalog: &FragmentCatalog) -> Result<String, LoweringGap> {
    match node {
        IrNode::Parameter { name, .. } => Ok(python_identifier(name)),
        IrNode::Literal { text, .. } => Ok(text.clone()),
        IrNode::Apply {
            fragment,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|argument| lower_node(argument, catalog))
                .collect::<Result<Vec<_>, _>>()?;
            catalog
                .render(fragment, "python", &arguments)
                .ok_or_else(|| {
                    gap(
                        "Apply",
                        render_or_id("python_ir_missing_fragment", &[("fragment", fragment)]),
                    )
                })
        }
        IrNode::Each {
            item,
            items,
            body,
            predicate,
        } => {
            let item = python_identifier(item);
            let items = lower_node(items, catalog)?;
            let body = lower_node(body, catalog)?;
            let predicate = predicate.as_deref().map_or_else(
                || Ok(String::new()),
                |predicate| {
                    let predicate = lower_node(predicate, catalog)?;
                    render("python_ir_predicate", &[("predicate", &predicate)])
                },
            )?;
            render(
                "python_ir_each",
                &[
                    ("body", &body),
                    ("item", &item),
                    ("items", &items),
                    ("predicate", &predicate),
                ],
            )
        }
        IrNode::Fold {
            item,
            accumulator,
            items,
            initial,
            body,
        } => {
            let accumulator = python_identifier(accumulator);
            let item = python_identifier(item);
            let body = lower_node(body, catalog)?;
            let items = lower_node(items, catalog)?;
            let initial = lower_node(initial, catalog)?;
            render(
                "python_ir_fold",
                &[
                    ("accumulator", &accumulator),
                    ("item", &item),
                    ("body", &body),
                    ("items", &items),
                    ("initial", &initial),
                ],
            )
        }
        IrNode::Repeat {
            counter,
            from,
            to,
            body,
        } => {
            let body = lower_node(body, catalog)?;
            let counter = python_identifier(counter);
            let from = lower_node(from, catalog)?;
            let to = lower_node(to, catalog)?;
            render(
                "python_ir_repeat",
                &[
                    ("body", &body),
                    ("counter", &counter),
                    ("from", &from),
                    ("to", &to),
                ],
            )
        }
        IrNode::Recurrence {
            state,
            base,
            transition,
            index,
        } => lower_recurrence(state, base, transition, index, catalog),
        IrNode::RecursiveReduce {
            state,
            target,
            item,
            items,
            next,
            admissible,
            base_test,
            base,
            local,
            reducer,
            combine,
        } => lower_recursive_reduce(
            state,
            target,
            item,
            items,
            next,
            admissible.as_deref(),
            base_test,
            base,
            local,
            reducer,
            combine,
            catalog,
        ),
        IrNode::Condition {
            test,
            then_branch,
            else_branch,
        } => {
            let then_branch = lower_node(then_branch, catalog)?;
            let test = lower_node(test, catalog)?;
            let else_branch = lower_node(else_branch, catalog)?;
            render(
                "python_ir_condition",
                &[
                    ("then", &then_branch),
                    ("test", &test),
                    ("otherwise", &else_branch),
                ],
            )
        }
        IrNode::Bind { name, value, body } => Ok(format!(
            "(lambda {}: {})({})",
            python_identifier(name),
            lower_node(body, catalog)?,
            lower_node(value, catalog)?
        )),
        IrNode::Emit { value } => {
            let value = lower_node(value, catalog)?;
            render("python_ir_emit", &[("value", &value)])
        }
        IrNode::Return { value } => lower_node(value, catalog),
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_recursive_reduce(
    state: &[String],
    target: &[IrNode],
    item: &[String],
    items: &IrNode,
    next: &[IrNode],
    admissible: Option<&IrNode>,
    base_test: &IrNode,
    base: &IrNode,
    local: &IrNode,
    reducer: &str,
    combine: &str,
    catalog: &FragmentCatalog,
) -> Result<String, LoweringGap> {
    if state.is_empty()
        || state.len() != target.len()
        || state.len() != next.len()
        || item.is_empty()
    {
        return Err(gap(
            "RecursiveReduce",
            render_or_id("python_ir_recursive_shape", &[]),
        ));
    }
    let targets = target
        .iter()
        .map(|node| lower_node(node, catalog))
        .collect::<Result<Vec<_>, _>>()?;
    let next = next
        .iter()
        .map(|node| lower_node(node, catalog))
        .collect::<Result<Vec<_>, _>>()?;
    let items = lower_node(items, catalog)?;
    let predicate = admissible.map_or_else(
        || Ok(String::new()),
        |node| Ok(format!(" if {}", lower_node(node, catalog)?)),
    )?;
    let item = item
        .iter()
        .map(|name| python_identifier(name))
        .collect::<Vec<_>>()
        .join(", ");
    let next = next.join(", ");
    let recursive_values = render(
        "python_ir_recursive_values",
        &[
            ("next", &next),
            ("item", &item),
            ("items", &items),
            ("predicate", &predicate),
        ],
    )?;
    let reduced = catalog
        .render(reducer, "python", &[recursive_values])
        .ok_or_else(|| {
            gap(
                "RecursiveReduce",
                render_or_id("python_ir_recursive_reducer", &[("fragment", reducer)]),
            )
        })?;
    let local = lower_node(local, catalog)?;
    let combined = catalog
        .render(combine, "python", &[local, reduced])
        .ok_or_else(|| {
            gap(
                "RecursiveReduce",
                render_or_id("python_ir_recursive_combine", &[("fragment", combine)]),
            )
        })?;
    let target = targets.join(", ");
    let state = state
        .iter()
        .map(|name| python_identifier(name))
        .collect::<Vec<_>>()
        .join(", ");
    let base = lower_node(base, catalog)?;
    let base_test = lower_node(base_test, catalog)?;
    render(
        "python_ir_recursive_reduce",
        &[
            ("target", &target),
            ("state", &state),
            ("base", &base),
            ("base_test", &base_test),
            ("combined", &combined),
        ],
    )
}

fn lower_recurrence(
    state: &[String],
    base: &[IrNode],
    transition: &IrNode,
    index: &IrNode,
    catalog: &FragmentCatalog,
) -> Result<String, LoweringGap> {
    if base.is_empty() || state.is_empty() {
        return Err(gap("Recurrence", "base cases and state names are required"));
    }
    let base = base
        .iter()
        .map(|node| lower_node(node, catalog))
        .collect::<Result<Vec<_>, _>>()?;
    let mut transition = lower_node(transition, catalog)?;
    for (offset, name) in state.iter().enumerate() {
        transition = replace_identifier(
            &transition,
            &python_identifier(name),
            &format!("values[-{}]", offset + 1),
        );
    }
    let index = lower_node(index, catalog)?;
    let base_count = base.len();
    let base = base.join(", ");
    let base_count = base_count.to_string();
    render(
        "python_ir_recurrence",
        &[
            ("base", &base),
            ("base_count", &base_count),
            ("transition", &transition),
            ("index", &index),
        ],
    )
}

fn replace_identifier(source: &str, sought: &str, replacement: &str) -> String {
    let mut output = String::new();
    let mut token = String::new();
    let flush = |output: &mut String, token: &mut String| {
        if token == sought {
            output.push_str(replacement);
        } else {
            output.push_str(token);
        }
        token.clear();
    };
    for character in source.chars() {
        if character.is_alphanumeric() || character == '_' {
            token.push(character);
        } else {
            flush(&mut output, &mut token);
            output.push(character);
        }
    }
    flush(&mut output, &mut token);
    output
}

fn python_identifier(value: &str) -> String {
    let mut identifier = value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if identifier.is_empty() {
        identifier.push('_');
    }
    if identifier.starts_with(char::is_numeric) {
        identifier.insert(0, '_');
    }
    identifier
}

fn gap(node: &str, detail: impl Into<String>) -> LoweringGap {
    LoweringGap {
        language: "python".to_owned(),
        node: node.to_owned(),
        detail: detail.into(),
    }
}

fn render(id: &str, values: &[(&str, &str)]) -> Result<String, LoweringGap> {
    runtime_template(id, values).ok_or_else(|| gap("Template", id))
}

fn render_or_id(id: &str, values: &[(&str, &str)]) -> String {
    runtime_template(id, values).unwrap_or_else(|| id.to_owned())
}
