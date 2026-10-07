//! The JavaScript lowering backend (PR #1188 dogfooding, T1): an ES module
//! function whose body is one composed expression.
//!
//! Only what a fragment's seeded `javascript` realization spells can lower;
//! every other node is a named [`LoweringGap`], never a Python surface passed
//! off as JavaScript. The function shape is the seeded
//! `javascript_ir_function` runtime template, which the browser composer
//! (`js/worker/formal_ai_worker_program_ir.js`) reads too.

use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::ir_lowering::{LanguageLowering, LoweringGap};
use crate::coding::program_ir::{IrNode, ProgramIr};
use crate::coding::python_render::runtime_template;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JavaScriptLowering;

impl LanguageLowering for JavaScriptLowering {
    fn language(&self) -> &'static str {
        "javascript"
    }

    fn lower(&self, ir: &ProgramIr, catalog: &FragmentCatalog) -> Result<String, LoweringGap> {
        let expression = lower_node(&ir.body, catalog)?;
        if ir.name.is_empty() {
            return Ok(expression);
        }
        let parameters = ir
            .parameters
            .iter()
            .map(|(name, _)| identifier(name))
            .collect::<Vec<_>>()
            .join(", ");
        runtime_template(
            "javascript_ir_function",
            &[
                ("name", &identifier(&ir.name)),
                ("parameters", &parameters),
                ("expression", &expression),
            ],
        )
        .ok_or_else(|| gap("Template", "javascript_ir_function"))
    }
}

fn lower_node(node: &IrNode, catalog: &FragmentCatalog) -> Result<String, LoweringGap> {
    match node {
        IrNode::Parameter { name, .. } => Ok(identifier(name)),
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
                .render(fragment, "javascript", &arguments)
                .ok_or_else(|| gap("Apply", fragment))
        }
        IrNode::Return { value } => lower_node(value, catalog),
        IrNode::Each { .. } => Err(gap("Each", "Each")),
        IrNode::Fold { .. } => Err(gap("Fold", "Fold")),
        IrNode::Repeat { .. } => Err(gap("Repeat", "Repeat")),
        IrNode::Recurrence { .. } => Err(gap("Recurrence", "Recurrence")),
        IrNode::RecursiveReduce { .. } => Err(gap("RecursiveReduce", "RecursiveReduce")),
        IrNode::Condition { .. } => Err(gap("Condition", "Condition")),
        IrNode::Bind { .. } => Err(gap("Bind", "Bind")),
        IrNode::Emit { .. } => Err(gap("Emit", "Emit")),
    }
}

/// A JavaScript identifier for `value`: every other character becomes `_`.
fn identifier(value: &str) -> String {
    let mut identifier = value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '_' || character == '$' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if identifier.is_empty() || identifier.starts_with(char::is_numeric) {
        identifier.insert(0, '_');
    }
    identifier
}

/// The gap for `node`, its detail the seeded `javascript_ir_gap` sentence
/// about `subject` (the node kind, or the fragment with no realization).
fn gap(node: &str, subject: impl AsRef<str>) -> LoweringGap {
    let subject = subject.as_ref();
    LoweringGap {
        language: "javascript".to_owned(),
        node: node.to_owned(),
        detail: runtime_template("javascript_ir_gap", &[("subject", subject)])
            .unwrap_or_else(|| subject.to_owned()),
    }
}
