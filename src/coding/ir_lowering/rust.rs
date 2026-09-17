//! The Rust lowering backend — the second language, which proves the IR is not
//! Python-shaped (issue #1138, plan 02 L19).

use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::ir_lowering::{LanguageLowering, LoweringGap};
use crate::coding::program_ir::{IrNode, IrType, ProgramIr};
use crate::coding::python_render::runtime_template;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RustLowering;

impl LanguageLowering for RustLowering {
    fn language(&self) -> &'static str {
        "rust"
    }

    fn lower(&self, ir: &ProgramIr, catalog: &FragmentCatalog) -> Result<String, LoweringGap> {
        let expression = lower_node(&ir.body, catalog)?;
        let parameters = ir
            .parameters
            .iter()
            .map(|(name, ty)| format!("{}: {}", rust_identifier(name), rust_type(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!(
            "pub fn {}({parameters}) -> {} {{\n    {expression}\n}}\n",
            rust_identifier(&ir.name),
            rust_type(&ir.result)
        ))
    }
}

fn lower_node(node: &IrNode, catalog: &FragmentCatalog) -> Result<String, LoweringGap> {
    match node {
        IrNode::Parameter { name, .. } => Ok(rust_identifier(name)),
        IrNode::Literal { text, ty } => Ok(rust_literal(text, ty)),
        IrNode::Apply {
            fragment,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|argument| lower_node(argument, catalog))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(rendered) = catalog.render(fragment, "rust", &arguments) {
                return Ok(rendered);
            }
            if catalog.get(fragment).is_none() {
                return Err(gap(
                    "Apply",
                    render_or_id("rust_ir_missing_fragment", &[("fragment", fragment)]),
                ));
            }
            Ok(format!(
                "{}({})",
                rust_identifier(fragment),
                arguments.join(", ")
            ))
        }
        IrNode::Each {
            item,
            items,
            body,
            predicate,
        } => {
            let item = rust_identifier(item);
            let items = lower_node(items, catalog)?;
            let predicate = predicate
                .as_deref()
                .map(|predicate| lower_node(predicate, catalog))
                .transpose()?
                .map_or_else(String::new, |predicate| {
                    render_or_id(
                        "rust_ir_filter",
                        &[("item", &item), ("predicate", &predicate)],
                    )
                });
            Ok(format!(
                "({items}).into_iter(){predicate}.map(|{item}| {}).collect::<Vec<_>>()",
                lower_node(body, catalog)?
            ))
        }
        IrNode::Fold {
            item,
            accumulator,
            items,
            initial,
            body,
        } => {
            let items = lower_node(items, catalog)?;
            let initial = lower_node(initial, catalog)?;
            let accumulator = rust_identifier(accumulator);
            let item = rust_identifier(item);
            let body = lower_node(body, catalog)?;
            render(
                "rust_ir_fold",
                &[
                    ("items", &items),
                    ("initial", &initial),
                    ("accumulator", &accumulator),
                    ("item", &item),
                    ("body", &body),
                ],
            )
        }
        IrNode::Repeat {
            counter,
            from,
            to,
            body,
        } => Ok(format!(
            "({}..{}).map(|{}| {}).collect::<Vec<_>>()",
            lower_node(from, catalog)?,
            lower_node(to, catalog)?,
            rust_identifier(counter),
            lower_node(body, catalog)?
        )),
        IrNode::Recurrence { .. } => Err(gap(
            "Recurrence",
            "a recurrence needs an explicit storage strategy before lowering",
        )),
        IrNode::RecursiveReduce { .. } => Err(gap(
            "RecursiveReduce",
            render_or_id("rust_ir_recursive_reduce_gap", &[]),
        )),
        IrNode::Condition {
            test,
            then_branch,
            else_branch,
        } => {
            let test = lower_node(test, catalog)?;
            let then_branch = lower_node(then_branch, catalog)?;
            let else_branch = lower_node(else_branch, catalog)?;
            render(
                "rust_ir_condition",
                &[
                    ("test", &test),
                    ("open", "{"),
                    ("then", &then_branch),
                    ("close", "}"),
                    ("otherwise", &else_branch),
                ],
            )
        }
        IrNode::Bind { name, value, body } => Ok(format!(
            "{{ let {} = {}; {} }}",
            rust_identifier(name),
            lower_node(value, catalog)?,
            lower_node(body, catalog)?
        )),
        IrNode::Emit { value } => {
            let value = lower_node(value, catalog)?;
            render(
                "rust_ir_emit",
                &[
                    ("open", "{"),
                    ("value", &value),
                    ("debug_format", concat!("{", ":?", "}")),
                    ("close", "}"),
                ],
            )
        }
        IrNode::Return { value } => lower_node(value, catalog),
    }
}

fn rust_type(ty: &IrType) -> String {
    match ty {
        IrType::Integer => "i64".to_owned(),
        IrType::Float => "f64".to_owned(),
        IrType::Boolean => "bool".to_owned(),
        IrType::Text => "String".to_owned(),
        // Functions render as their lambda text; a typed slot never needs a
        // Rust name for the callable itself.
        IrType::Callable => "String".to_owned(),
        IrType::Sequence(element) => format!("Vec<{}>", rust_type(element)),
        IrType::Pair(left, right) => format!("({}, {})", rust_type(left), rust_type(right)),
        IrType::Mapping(key, value) => {
            format!(
                "std::collections::BTreeMap<{}, {}>",
                rust_type(key),
                rust_type(value)
            )
        }
        IrType::Unknown(_) => "impl Clone".to_owned(),
    }
}

fn rust_literal(text: &str, ty: &IrType) -> String {
    match (text.trim(), ty) {
        ("[]", IrType::Sequence(_)) => "Vec::new()".to_owned(),
        ("{}", IrType::Mapping(_, _)) => "std::collections::BTreeMap::new()".to_owned(),
        ("None", IrType::Unknown(_)) => "()".to_owned(),
        (text, IrType::Text) if !text.starts_with('"') => format!("{text:?}.to_owned()"),
        _ => text.to_owned(),
    }
}

fn rust_identifier(value: &str) -> String {
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
        language: "rust".to_owned(),
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
