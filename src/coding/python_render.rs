//! Python rendering for discovered composition trees.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::coding::task_spec::CodingTaskSpec;

const RUNTIME: &str = include_str!("../../data/seed/coding-discovery-runtime.lino");

/// Render a named Python/runtime template from the coding-discovery seed.
#[must_use]
pub fn runtime_template(id: &str, values: &[(&str, &str)]) -> Option<String> {
    let root = crate::seed::parser::parse_lino(RUNTIME);
    let mut rendered = root
        .children
        .iter()
        .find(|node| node.name == "coding_discovery_runtime")?
        .children
        .iter()
        .find(|node| node.name == "template" && node.id == id)?
        .find_child_value("text")
        .to_owned();
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    Some(rendered)
}

#[must_use]
pub fn render_function(
    spec: &CodingTaskSpec,
    body: &str,
    extra_imports: impl IntoIterator<Item = String>,
) -> String {
    let mut imports = spec.imports.iter().cloned().collect::<BTreeSet<_>>();
    imports.extend(extra_imports);
    let mut out = String::new();
    if !imports.is_empty() {
        out.push_str(&imports.into_iter().collect::<Vec<_>>().join("\n"));
        out.push_str("\n\n");
    }
    let parameters = spec
        .parameters
        .iter()
        .map(|parameter| {
            parameter.annotation.as_ref().map_or_else(
                || parameter.name.clone(),
                |annotation| format!("{}: {annotation}", parameter.name),
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let return_annotation = spec
        .return_annotation
        .as_ref()
        .map_or(String::new(), |annotation| format!(" -> {annotation}"));
    let _ = writeln!(out, "def {}({parameters}){return_annotation}:", spec.name);
    for line in body.lines() {
        if line.trim().is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, "    {line}");
        }
    }
    out
}
