//! Python rendering for discovered composition trees.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use crate::coding::task_spec::CodingTaskSpec;

/// Render a named Python/runtime template from the coding-discovery seed.
#[must_use]
pub fn runtime_template(id: &str, values: &[(&str, &str)]) -> Option<String> {
    let text =
        crate::coding::fragment_catalog::bootstrap_seed_text("coding-discovery-runtime.lino")?;
    render_runtime_template(&text, id, values)
}

/// Explicit-root variant used by the bootstrap-deletability proof.
#[must_use]
pub fn runtime_template_from(
    seed_directory: &Path,
    id: &str,
    values: &[(&str, &str)],
) -> Option<String> {
    let seed_directory = if seed_directory.join("data/seed").is_dir() {
        seed_directory.join("data/seed")
    } else {
        seed_directory.to_path_buf()
    };
    let text =
        std::fs::read_to_string(seed_directory.join("coding-discovery-runtime.lino")).ok()?;
    render_runtime_template(&text, id, values)
}

fn render_runtime_template(text: &str, id: &str, values: &[(&str, &str)]) -> Option<String> {
    let root = crate::seed::parser::parse_lino(text);
    let template = root
        .children
        .iter()
        .filter(|node| {
            matches!(
                node.name.as_str(),
                "coding_discovery_runtime" | "coding_runtime_templates"
            )
        })
        .flat_map(|node| node.children.iter())
        .find(|node| node.name == "template" && node.id == id)?;
    // Templates whose verbatim syntax contains the seed's reserved multi-value
    // separator (`|` in a Rust closure) are declared under `code`, the one
    // field the seed guard exempts for verbatim listings.
    let shape = template.find_child_value("text");
    let shape = if shape.is_empty() {
        template.find_child_value("code")
    } else {
        shape
    };
    let mut rendered = shape.to_owned();
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
