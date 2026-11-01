use super::view::{
    children, concrete, field, form, identifier, literal, one, path_segments, role, trivia,
};
use super::{Binding, Facts};
use serde_json::{Value, json};
impl Facts<'_> {
    fn append_import(&mut self, module: &str, bindings: Vec<Binding>, node: &Value, kind: &str) {
        if self.imports.len() >= 128 {
            self.truncated = true;
            return;
        }
        let mut row = self.view.observation(node);
        row["module"] = json!(module);
        row["kind"] = json!(kind);
        row["bindings"]=json!(bindings.iter().map(|binding|json!({"imported":binding.imported,"local":binding.local,"kind":binding.kind})).collect::<Vec<_>>());
        self.imports.push(row);
        for binding in bindings {
            if binding.local == "_" {
                self.gap(
                    "anonymous-import-binding-unresolved",
                    Some(node),
                    Some(&binding.local),
                );
                continue;
            }
            if self.bindings.contains_key(&binding.local) {
                self.unsafe_names.insert(binding.local.clone());
                self.gap("duplicate-import-binding", Some(node), Some(&binding.local));
            } else {
                self.bindings.insert(binding.local.clone(), binding);
            }
        }
    }
    pub(super) fn javascript_import(&mut self, node: &Value) {
        let Some(module) = one(node, &["source"]).and_then(literal) else {
            self.gap(
                "module-literal-escape-or-grammar-unsupported",
                Some(node),
                None,
            );
            return;
        };
        let type_only = children(node)
            .iter()
            .any(|child| field(child, "text") == Some("type"));
        let mut bindings = Vec::new();
        for child in children(node) {
            if role(child, "specifiers") || form(child) == "import_clause" {
                self.javascript_specifier(
                    child,
                    &module,
                    type_only,
                    form(node) == "import_statement",
                    &mut bindings,
                );
            }
        }
        self.append_import(&module, bindings, node, "static-import");
    }
    fn javascript_specifier(
        &mut self,
        node: &Value,
        module: &str,
        type_only: bool,
        native: bool,
        output: &mut Vec<Binding>,
    ) {
        let kind = form(node);
        if kind == "import_specifier" {
            let imported = one(node, &["imported", "name"]);
            let local = one(node, &["local", "alias"]).or(imported);
            if let (Some(imported), Some(local)) = (
                imported.and_then(|node| identifier(node).or_else(|| literal(node))),
                local.and_then(identifier),
            ) {
                let is_type = type_only
                    || children(node)
                        .iter()
                        .any(|child| field(child, "text") == Some("type"));
                output.push(Binding {
                    module: module.to_owned(),
                    imported,
                    local,
                    kind: if is_type { "type" } else { "named" }.to_owned(),
                });
            } else {
                self.gap("import-binding-grammar-unsupported", Some(node), None);
            }
            return;
        }
        if matches!(
            kind,
            "import_default_specifier" | "import_namespace_specifier" | "namespace_import"
        ) {
            let local = one(node, &["local", "alias"])
                .or_else(|| {
                    children(node)
                        .into_iter()
                        .find(|child| identifier(child).is_some())
                })
                .and_then(identifier);
            if let Some(local) = local {
                output.push(Binding {
                    module: module.to_owned(),
                    imported: if kind == "import_default_specifier" {
                        "default"
                    } else {
                        "*"
                    }
                    .to_owned(),
                    local,
                    kind: if type_only {
                        "type"
                    } else if kind == "import_default_specifier" {
                        "default"
                    } else {
                        "namespace"
                    }
                    .to_owned(),
                });
            } else {
                self.gap("import-binding-grammar-unsupported", Some(node), None);
            }
            return;
        }
        if kind == "identifier" && native {
            if let Some(local) = identifier(node) {
                output.push(Binding {
                    module: module.to_owned(),
                    imported: "default".to_owned(),
                    local,
                    kind: if type_only { "type" } else { "default" }.to_owned(),
                });
            }
            return;
        }
        if matches!(kind, "import_clause" | "named_imports") {
            for child in children(node) {
                self.javascript_specifier(child, module, type_only, native, output);
            }
        }
    }
    pub(super) fn rust_use(&mut self, node: &Value) {
        let Some(argument) = one(node, &["argument"]) else {
            self.gap("rust-use-grammar-unsupported", Some(node), None);
            return;
        };
        let mut groups: Vec<(String, Vec<Binding>)> = Vec::new();
        self.rust_use_tree(argument, &[], node, &mut groups);
        for (module, bindings) in groups {
            self.append_import(&module, bindings, node, "rust-use");
        }
    }
    fn rust_binding(
        &mut self,
        segments: &[String],
        alias: Option<String>,
        kind: &str,
        statement: &Value,
        groups: &mut Vec<(String, Vec<Binding>)>,
    ) {
        let Some(module) = segments.first() else {
            self.gap("rust-use-path-unresolved", Some(statement), None);
            return;
        };
        let binding = Binding {
            module: module.clone(),
            imported: if segments.len() == 1 {
                "self".to_owned()
            } else {
                segments[1..].join("::")
            },
            local: alias.unwrap_or_else(|| segments.last().cloned().unwrap_or_default()),
            kind: if segments.len() == 1 {
                "namespace"
            } else {
                kind
            }
            .to_owned(),
        };
        if let Some((_, bindings)) = groups.iter_mut().find(|(name, _)| name == module) {
            bindings.push(binding);
        } else {
            groups.push((module.clone(), vec![binding]));
        }
    }
    fn rust_use_tree(
        &mut self,
        node: &Value,
        prefix: &[String],
        statement: &Value,
        groups: &mut Vec<(String, Vec<Binding>)>,
    ) {
        match form(node) {
            "use_as_clause" => {
                let path = one(node, &["path"]).and_then(path_segments);
                let alias = one(node, &["alias"]).and_then(identifier);
                let (Some(path), Some(alias)) = (path, alias) else {
                    self.gap("rust-use-alias-grammar-unsupported", Some(node), None);
                    return;
                };
                let is_self = path.as_slice() == ["self"];
                let segments = if is_self && !prefix.is_empty() {
                    prefix.to_vec()
                } else {
                    [prefix, path.as_slice()].concat()
                };
                self.rust_binding(
                    &segments,
                    Some(alias),
                    if is_self { "namespace" } else { "named" },
                    statement,
                    groups,
                );
            }
            "scoped_use_list" => {
                let path = one(node, &["path"]).map_or_else(|| Some(Vec::new()), path_segments);
                let list = one(node, &["list"]);
                let (Some(path), Some(list)) = (path, list) else {
                    self.gap("rust-use-list-grammar-unsupported", Some(node), None);
                    return;
                };
                self.rust_use_tree(list, &[prefix, path.as_slice()].concat(), statement, groups);
            }
            "use_list" => {
                for child in children(node) {
                    if !trivia(child) && !matches!(concrete(child).as_str(), "{" | "}" | ",") {
                        self.rust_use_tree(child, prefix, statement, groups);
                    }
                }
            }
            "use_wildcard" => self.gap("glob-import-resolution-unavailable", Some(node), None),
            _ => {
                let Some(path) = path_segments(node) else {
                    self.gap("rust-use-path-grammar-unsupported", Some(node), None);
                    return;
                };
                let is_self = path.as_slice() == ["self"];
                let segments = if is_self && !prefix.is_empty() {
                    prefix.to_vec()
                } else {
                    [prefix, path.as_slice()].concat()
                };
                self.rust_binding(
                    &segments,
                    None,
                    if is_self { "namespace" } else { "named" },
                    statement,
                    groups,
                );
            }
        }
    }
}
