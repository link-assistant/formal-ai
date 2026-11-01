//! Bounded observed source syntax, manually paired with the JavaScript extractor.
//! Actual complete grammar/CST witnesses never establish external API behavior.
use crate::readable_source::{owned_source_links, source_from_links, source_to_links};
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
mod imports;
mod options;
pub(crate) mod view;
use self::view::{
    View, children, concrete, field, form, identifier, named, one, path_segments, trivia,
};
#[derive(Clone)]
struct Binding {
    module: String,
    imported: String,
    local: String,
    kind: String,
}
struct Facts<'a> {
    view: View<'a>,
    imports: Vec<Value>,
    calls: Vec<Value>,
    references: Vec<Value>,
    gaps: Vec<Value>,
    bindings: BTreeMap<String, Binding>,
    unsafe_names: BTreeSet<String>,
    local_modules: BTreeSet<String>,
    truncated: bool,
}
impl Facts<'_> {
    fn gap(&mut self, kind: &str, node: Option<&Value>, binding: Option<&str>) {
        if self.gaps.len() >= 64 {
            self.truncated = true;
            return;
        }
        let mut gap = node.map_or_else(|| json!({}), |node| self.view.observation(node));
        gap["kind"] = json!(kind);
        if let Some(binding) = binding {
            gap["binding"] = json!(binding);
        }
        if !self.gaps.iter().any(|old| {
            old["kind"] == gap["kind"]
                && old.get("binding") == gap.get("binding")
                && old.get("span") == gap.get("span")
        }) {
            self.gaps.push(gap);
        }
    }
}
/// Actual source is parsed by the existing installed native grammar. The wire
/// carries source syntax witnesses, never proof of runtime execution or APIs.
pub fn observed_source_usage(owner: &str, source: &str, language: &str) -> Result<Value, String> {
    if owner.starts_with('/')
        || owner.split('/').any(|part| matches!(part, "" | "." | ".."))
        || source.len() > 262144
        || !matches!(language, "JavaScript" | "TypeScript" | "Rust")
    {
        return Err("SourceUsageObservedInputRequired".to_owned());
    }
    let document = owned_source_links(owner, &source_to_links(source, language)?)?;
    if source_from_links(&document)? != source {
        return Err("SourceUsageReconstructionMismatch".to_owned());
    }
    let view = View::new(source, &document)?;
    let mut facts = Facts {
        view,
        imports: Vec::new(),
        calls: Vec::new(),
        references: Vec::new(),
        gaps: Vec::new(),
        bindings: BTreeMap::new(),
        unsafe_names: BTreeSet::new(),
        local_modules: BTreeSet::new(),
        truncated: false,
    };
    let nodes = facts
        .view
        .records
        .iter()
        .map(|record| record.node)
        .collect::<Vec<_>>();
    for node in &nodes {
        match form(node) {
            "import_declaration" | "import_statement" => facts.javascript_import(node),
            "use_declaration" => facts.rust_use(node),
            "import_expression" => {
                facts.gap("dynamic-module-loading-not-followed", Some(node), None)
            }
            "ts_import_equals_declaration" => {
                facts.gap("module-binding-grammar-not-supported", Some(node), None)
            }
            "extern_crate_declaration" => facts.gap(
                "extern-crate-binding-grammar-not-supported",
                Some(node),
                None,
            ),
            _ => {}
        }
        if matches!(form(node), "mod_item" | "module") {
            if let Some(name) = one(node, &["name"]).and_then(identifier) {
                facts.local_modules.insert(name);
            }
        }
    }
    for node in &nodes {
        let Some(record) = facts.view.record(node) else {
            continue;
        };
        if View::in_import(record) {
            continue;
        }
        let regions = match form(node) {
            "function_declaration"
            | "function_expression"
            | "arrow_function_expression"
            | "function_item"
            | "closure_expression"
            | "class_declaration"
            | "struct_item"
            | "enum_item"
            | "trait_item"
            | "mod_item"
            | "type_item"
            | "const_item"
            | "static_item" => named(node, &["name", "params", "param", "parameters"]),
            "variable_declarator"
            | "let_declaration"
            | "catch_clause"
            | "parameter"
            | "required_parameter"
            | "optional_parameter" => named(node, &["name", "pattern", "param", "parameter"]),
            "assignment_expression" | "augmented_assignment_expression" | "update_expression" => {
                named(node, &["left", "argument"])
            }
            _ => Vec::new(),
        };
        let mut potential = Vec::new();
        for region in regions {
            for nested in facts.view.descendants(region) {
                if let Some(name) = identifier(nested.node) {
                    if facts.bindings.contains_key(&name) {
                        potential.push((name, nested.node));
                    }
                }
            }
        }
        for (name, nested) in potential {
            facts.unsafe_names.insert(name.clone());
            facts.gap("possible-shadow-or-reassignment", Some(nested), Some(&name));
        }
        if matches!(form(node), "variable_declarator" | "let_declaration") {
            let initializer = one(node, &["init", "value"]);
            let name = initializer.and_then(|node| {
                identifier(node).or_else(|| {
                    matches!(form(node), "member_expression" | "field_expression")
                        .then(|| one(node, &["object", "value"]).and_then(identifier))
                        .flatten()
                })
            });
            if let Some(name) = name.filter(|name| facts.bindings.contains_key(name)) {
                facts.gap("alias-flow-not-followed", Some(node), Some(&name));
            }
        }
    }
    for node in &nodes {
        let kind = form(node);
        let Some(record) = facts.view.record(node) else {
            continue;
        };
        if !matches!(
            kind,
            "call_expression" | "optional_call_expression" | "new_expression" | "struct_expression"
        ) || View::in_import(record)
        {
            continue;
        }
        let Some(callee) = one(
            node,
            if kind == "struct_expression" {
                &["name"]
            } else {
                &["callee", "function"]
            },
        ) else {
            facts.gap("call-grammar-unavailable", Some(node), None);
            continue;
        };
        if form(callee) == "import" {
            facts.gap("dynamic-module-loading-not-followed", Some(node), None);
            continue;
        }
        let segments = path_segments(callee);
        let mut root = segments
            .as_ref()
            .and_then(|segments| segments.first())
            .cloned();
        let mut member = None;
        let mut dynamic = false;
        if matches!(
            form(callee),
            "member_expression" | "optional_member_expression" | "field_expression"
        ) {
            let object = one(callee, &["object", "value"]);
            let property = one(callee, &["property", "field"]);
            let object_name = object.and_then(identifier);
            let computed = children(callee)
                .iter()
                .any(|node| field(node, "text") == Some("["));
            if let Some(name) = object_name.filter(|name| facts.bindings.contains_key(name)) {
                member = property.and_then(identifier);
                dynamic = computed || member.is_none() || facts.bindings[&name].kind != "namespace";
                root = Some(name);
            } else if language == "Rust"
                && object.is_some_and(|object| {
                    matches!(form(object), "call_expression" | "struct_expression")
                })
            {
                let inner = object
                    .and_then(|object| one(object, &["function", "name"]))
                    .and_then(path_segments);
                if let Some(segments) = inner.filter(|segments| segments.len() > 1) {
                    let mut item = facts.view.observation(node);
                    item["root"] = json!(segments[0]);
                    item["segments"] = json!(segments);
                    item["kind"] = json!("receiver-method-call");
                    item["resolution"] = json!("receiver-dispatch-unresolved");
                    item["options"] = json!([]);
                    if facts.references.len() < 128 {
                        facts.references.push(item);
                    } else {
                        facts.truncated = true;
                    }
                    facts.gap(
                        "receiver-dispatch-unresolved",
                        Some(node),
                        segments.first().map(String::as_str),
                    );
                }
                continue;
            } else {
                continue;
            }
        }
        let Some(root) = root else {
            continue;
        };
        let bound = facts.bindings.get(&root).cloned();
        let argument_fields = named(node, &["arguments"]);
        let argument_nodes = if argument_fields.len() == 1
            && form(argument_fields[0]) == "arguments"
        {
            children(argument_fields[0])
                .into_iter()
                .filter(|node| !trivia(node) && !matches!(concrete(node).as_str(), "(" | ")" | ","))
                .collect()
        } else {
            argument_fields
        };
        let mut options = if kind == "struct_expression" {
            facts.options(one(node, &["body"]))
        } else {
            Vec::new()
        };
        if kind != "struct_expression" {
            for argument in argument_nodes {
                options.extend(facts.options(Some(argument)));
            }
        }
        if options.len() > 16 {
            options.truncate(16);
            facts.truncated = true;
        }
        let Some(bound) = bound else {
            if let Some(segments) =
                segments.filter(|segments| language == "Rust" && segments.len() > 1)
            {
                let mut item = facts.view.observation(node);
                item["root"] = json!(root);
                item["segments"] = json!(segments);
                item["kind"] = json!(if kind == "struct_expression" {
                    "struct-initializer"
                } else {
                    "qualified-call"
                });
                item["resolution"] = json!("unresolved-module-root");
                item["options"] = json!(options);
                if facts.references.len() < 128 {
                    facts.references.push(item);
                } else {
                    facts.truncated = true;
                }
                facts.gap(
                    if facts.local_modules.contains(&root) {
                        "qualified-root-has-local-declaration"
                    } else {
                        "qualified-root-resolution-required"
                    },
                    Some(node),
                    Some(&root),
                );
            }
            continue;
        };
        if facts.unsafe_names.contains(&root) || bound.kind == "type" {
            facts.gap(
                if bound.kind == "type" {
                    "type-only-binding-not-runtime-call"
                } else {
                    "import-binding-scope-unresolved"
                },
                Some(node),
                Some(&root),
            );
            continue;
        }
        if dynamic || kind == "optional_call_expression" {
            facts.gap("dynamic-dispatch-not-followed", Some(node), Some(&root));
            continue;
        }
        if language == "Rust" {
            if let Some(segments) = segments.filter(|segments| segments.len() > 1) {
                member = Some(segments[1..].join("::"));
            }
        }
        let mut item = facts.view.observation(node);
        item["module"] = json!(bound.module);
        item["binding"] = json!(root);
        item["imported"] = json!(bound.imported);
        if let Some(member) = member {
            item["member"] = json!(member);
        }
        item["kind"] = json!(if kind == "struct_expression" {
            "struct-initializer"
        } else if kind == "new_expression" {
            "constructor"
        } else {
            "call"
        });
        item["resolution"] = json!("static-import-binding-syntax");
        item["options"] = json!(options);
        if facts.calls.len() < 128 {
            facts.calls.push(item);
        } else {
            facts.truncated = true;
        }
    }
    if facts.truncated {
        facts.gap("observation-budget-exceeded", None, None);
    }
    Ok(
        json!({"schema":"observed-source-usage/v1","owner":owner,"sourceIdentity":sha256_hex(source.as_bytes()),"language":language,
        "imports":facts.imports,"calls":facts.calls,"references":facts.references,"gaps":facts.gaps,
        "complete":facts.gaps.is_empty()&&!facts.truncated,"truncated":facts.truncated,"scope":"static-source-observations-only; no execution or external API semantics",
        "limits":{"sourceBytes":262144,"cstNodes":50000,"depth":256,"observationsPerKind":128,"snippetBytes":512}}),
    )
}

/// Closed request counterpart of the installed Node operation. Frontend code
/// can never be supplied by fetched task text or source-file content.
pub fn observed_source_usage_operation(request: &Value) -> Result<Value, String> {
    let object = request
        .as_object()
        .ok_or_else(|| "SourceUsageRequestContract".to_owned())?;
    let keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if keys != ["owner", "source", "language"].into_iter().collect() {
        return Err("SourceUsageRequestContract".to_owned());
    }
    let owner = request["owner"]
        .as_str()
        .ok_or_else(|| "SourceUsageRequestContract".to_owned())?;
    let source = request["source"]
        .as_str()
        .ok_or_else(|| "SourceUsageRequestContract".to_owned())?;
    let language = request["language"]
        .as_str()
        .ok_or_else(|| "SourceUsageRequestContract".to_owned())?;
    observed_source_usage(owner, source, language)
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/observed_source_usage.rs"]
mod tests;
