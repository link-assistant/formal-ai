//! Reusable associative packages, handlers, triggers, and permissions.
//!
//! Packages are the local, doublet-backed adaptation of the Deep.Foundation
//! package idea from R65: a package is reviewable Links Notation that can be
//! installed, dependency-checked, exported/imported, replayed, and asked
//! whether it grants a tool or action capability.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use lino_objects_codec::format::parse_indented;

use crate::engine::{normalize_prompt, stable_id, GraphEdge, GraphNode, KNOWLEDGE_SCHEMA_VERSION};
use crate::link_store::{DoubletLink, LinkRecord};
use crate::links_format::push_lino_node;
use crate::seed::parser::{parse_lino, LinoNode};
use crate::skill_compiler::CompiledSkillPackage;

/// A package dependency by package id and optional exact version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDependency {
    pub package_id: String,
    pub version: String,
}

/// A package-provided executable handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageHandler {
    pub id: String,
    pub kind: String,
    pub capability: String,
    pub response: String,
}

/// Trigger-style computation over package links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageTrigger {
    pub id: String,
    pub kind: String,
    pub match_prompt: String,
    pub normalized_match: String,
    pub handler_id: String,
}

/// Explicit package permission grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagePermission {
    pub id: String,
    pub capability: String,
    pub effect: String,
    pub description: String,
}

/// A reusable associative package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociativePackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub dependencies: Vec<PackageDependency>,
    pub handlers: Vec<PackageHandler>,
    pub triggers: Vec<PackageTrigger>,
    pub permissions: Vec<PackagePermission>,
}

/// Deterministic replay result from an installed package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageReplay {
    pub package_id: String,
    pub trigger_id: String,
    pub handler_id: String,
    pub answer: String,
    pub cache_hit: String,
}

/// Permission decision for package-gated tools/actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackagePermissionDecision {
    Allowed {
        package_id: String,
        permission_id: String,
        capability: String,
    },
    Denied {
        capability: String,
        reason: String,
    },
}

/// Installed package registry with deterministic dependency validation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PackageStore {
    packages: BTreeMap<String, AssociativePackage>,
}

/// Package installation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageInstallError {
    MissingDependency {
        package_id: String,
        dependency_id: String,
    },
    VersionMismatch {
        package_id: String,
        dependency_id: String,
        required: String,
        installed: String,
    },
}

/// Package import failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageImportError {
    EmptyDocument,
    IllFormedLinksNotation(String),
    NotAssociativePackage(String),
    MissingField(&'static str),
}

impl fmt::Display for PackageInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDependency {
                package_id,
                dependency_id,
            } => write!(
                formatter,
                "package {package_id} requires missing dependency {dependency_id}"
            ),
            Self::VersionMismatch {
                package_id,
                dependency_id,
                required,
                installed,
            } => write!(
                formatter,
                "package {package_id} requires {dependency_id}@{required}, installed {installed}"
            ),
        }
    }
}

impl Error for PackageInstallError {}

impl fmt::Display for PackageImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDocument => write!(formatter, "package document is empty"),
            Self::IllFormedLinksNotation(message) => {
                write!(formatter, "ill-formed package Links Notation: {message}")
            }
            Self::NotAssociativePackage(id) => {
                write!(formatter, "{id} is not an associative package")
            }
            Self::MissingField(field) => write!(formatter, "package is missing {field}"),
        }
    }
}

impl Error for PackageImportError {}

impl PackageHandler {
    #[must_use]
    pub fn new(id: &str, kind: &str, capability: &str) -> Self {
        Self {
            id: id.to_owned(),
            kind: kind.to_owned(),
            capability: capability.to_owned(),
            response: String::new(),
        }
    }

    #[must_use]
    pub fn with_response(mut self, response: &str) -> Self {
        response.clone_into(&mut self.response);
        self
    }
}

impl PackageTrigger {
    #[must_use]
    pub fn new(id: &str, kind: &str, match_prompt: &str, handler_id: &str) -> Self {
        Self {
            id: id.to_owned(),
            kind: kind.to_owned(),
            match_prompt: match_prompt.to_owned(),
            normalized_match: normalize_prompt(match_prompt),
            handler_id: handler_id.to_owned(),
        }
    }
}

impl AssociativePackage {
    #[must_use]
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
            dependencies: Vec::new(),
            handlers: Vec::new(),
            triggers: Vec::new(),
            permissions: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_compiled_skill(
        id: &str,
        name: &str,
        version: &str,
        skill: &CompiledSkillPackage,
    ) -> Self {
        let mut package = Self::new(id, name, version)
            .with_handler(
                PackageHandler::new(
                    &skill.handler_id,
                    "deterministic_response",
                    &format!("handler:{}", skill.handler_id),
                )
                .with_response(&skill.response),
            )
            .with_trigger(PackageTrigger {
                id: skill.rule_id.clone(),
                kind: String::from("exact_normalized_prompt"),
                match_prompt: skill.trigger.clone(),
                normalized_match: skill.normalized_trigger.clone(),
                handler_id: skill.handler_id.clone(),
            });
        for test in &skill.expected_tests {
            if test.normalized_input == skill.normalized_trigger
                && test.expected_output == skill.response
            {
                continue;
            }
            package = package
                .with_handler(
                    PackageHandler::new(
                        &test.handler_id,
                        "deterministic_response",
                        &format!("handler:{}", test.handler_id),
                    )
                    .with_response(&test.expected_output),
                )
                .with_trigger(PackageTrigger {
                    id: test.trigger_id.clone(),
                    kind: String::from("exact_normalized_prompt"),
                    match_prompt: test.input.clone(),
                    normalized_match: test.normalized_input.clone(),
                    handler_id: test.handler_id.clone(),
                });
        }
        for permission in &skill.required_permissions {
            package = package.with_permission(&permission.capability, &permission.description);
        }
        package
    }

    #[must_use]
    pub fn with_dependency(mut self, package_id: &str, version: &str) -> Self {
        self.dependencies.push(PackageDependency {
            package_id: package_id.to_owned(),
            version: version.to_owned(),
        });
        self
    }

    #[must_use]
    pub fn with_handler(mut self, handler: PackageHandler) -> Self {
        self.handlers.push(handler);
        self
    }

    #[must_use]
    pub fn with_trigger(mut self, trigger: PackageTrigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    #[must_use]
    pub fn with_permission(mut self, capability: &str, description: &str) -> Self {
        let id = stable_id(
            "package_permission",
            &format!("{}:{capability}:{description}", self.id),
        );
        self.permissions.push(PackagePermission {
            id,
            capability: capability.to_owned(),
            effect: String::from("allow"),
            description: description.to_owned(),
        });
        self
    }

    #[must_use]
    pub fn replay(&self, prompt: &str) -> Option<PackageReplay> {
        let normalized = normalize_prompt(prompt);
        let trigger = self.triggers.iter().find(|trigger| {
            trigger.kind == "exact_normalized_prompt" && trigger.normalized_match == normalized
        })?;
        let handler = self
            .handlers
            .iter()
            .find(|handler| handler.id == trigger.handler_id)?;
        if handler.kind != "deterministic_response" || handler.response.is_empty() {
            return None;
        }
        Some(PackageReplay {
            package_id: self.id.clone(),
            trigger_id: trigger.id.clone(),
            handler_id: handler.id.clone(),
            answer: handler.response.clone(),
            cache_hit: self.id.clone(),
        })
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, &self.id, None);
        push_lino_node(&mut out, 2, "type", Some("associative_package"));
        push_lino_node(
            &mut out,
            2,
            "schema_version",
            Some(KNOWLEDGE_SCHEMA_VERSION),
        );
        push_lino_node(&mut out, 2, "name", Some(&self.name));
        push_lino_node(&mut out, 2, "version", Some(&self.version));
        push_lino_node(
            &mut out,
            2,
            "source",
            Some("R65 Deep.Foundation-inspired local package model"),
        );
        for dependency in &self.dependencies {
            push_lino_node(&mut out, 2, "dependency", Some(&dependency.package_id));
            push_lino_node(&mut out, 4, "version", Some(&dependency.version));
        }
        for handler in &self.handlers {
            push_lino_node(&mut out, 2, "handler", Some(&handler.id));
            push_lino_node(&mut out, 4, "kind", Some(&handler.kind));
            push_lino_node(&mut out, 4, "capability", Some(&handler.capability));
            push_lino_node(&mut out, 4, "response", Some(&handler.response));
        }
        for trigger in &self.triggers {
            push_lino_node(&mut out, 2, "trigger", Some(&trigger.id));
            push_lino_node(&mut out, 4, "kind", Some(&trigger.kind));
            push_lino_node(&mut out, 4, "match_prompt", Some(&trigger.match_prompt));
            push_lino_node(
                &mut out,
                4,
                "normalized_match",
                Some(&trigger.normalized_match),
            );
            push_lino_node(&mut out, 4, "handler", Some(&trigger.handler_id));
        }
        for permission in &self.permissions {
            push_lino_node(&mut out, 2, "permission", Some(&permission.id));
            push_lino_node(&mut out, 4, "effect", Some(&permission.effect));
            push_lino_node(&mut out, 4, "capability", Some(&permission.capability));
            push_lino_node(&mut out, 4, "description", Some(&permission.description));
        }
        out.trim_end().to_owned()
    }

    pub fn from_links_notation(text: &str) -> Result<Self, PackageImportError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(PackageImportError::EmptyDocument);
        }
        parse_indented(trimmed)
            .map_err(|error| PackageImportError::IllFormedLinksNotation(format!("{error:?}")))?;
        let tree = parse_lino(trimmed);
        let root = tree
            .children
            .first()
            .ok_or(PackageImportError::EmptyDocument)?;
        if root.find_child_value("type") != "associative_package" {
            return Err(PackageImportError::NotAssociativePackage(root.name.clone()));
        }
        let name = required_child(root, "name")?;
        let version = required_child(root, "version")?;
        let mut package = Self::new(&root.name, name, version);
        for child in &root.children {
            match child.name.as_str() {
                "dependency" => {
                    package.dependencies.push(PackageDependency {
                        package_id: child.id.clone(),
                        version: child.find_child_value("version").to_owned(),
                    });
                }
                "handler" => {
                    package.handlers.push(PackageHandler {
                        id: child.id.clone(),
                        kind: child.find_child_value("kind").to_owned(),
                        capability: child.find_child_value("capability").to_owned(),
                        response: child.find_child_value("response").to_owned(),
                    });
                }
                "trigger" => {
                    package.triggers.push(PackageTrigger {
                        id: child.id.clone(),
                        kind: child.find_child_value("kind").to_owned(),
                        match_prompt: child.find_child_value("match_prompt").to_owned(),
                        normalized_match: child.find_child_value("normalized_match").to_owned(),
                        handler_id: child.find_child_value("handler").to_owned(),
                    });
                }
                "permission" => {
                    package.permissions.push(PackagePermission {
                        id: child.id.clone(),
                        effect: child.find_child_value("effect").to_owned(),
                        capability: child.find_child_value("capability").to_owned(),
                        description: child.find_child_value("description").to_owned(),
                    });
                }
                _ => {}
            }
        }
        Ok(package)
    }

    #[must_use]
    pub fn grants_capability(&self, capability: &str) -> Option<&PackagePermission> {
        self.permissions
            .iter()
            .find(|permission| permission.effect == "allow" && permission.capability == capability)
    }

    #[must_use]
    pub fn link_records(&self) -> Vec<LinkRecord> {
        let mut records = vec![link_record(
            &self.id,
            "AssociativePackage",
            "associative_package",
            "R65",
            &[
                ("name", self.name.as_str()),
                ("version", self.version.as_str()),
                ("source", "Deep.Foundation-inspired local package model"),
            ],
        )];
        for dependency in &self.dependencies {
            records.push(link_record(
                &stable_id(
                    "package_dependency",
                    &format!("{}:{}", self.id, dependency.package_id),
                ),
                "PackageDependency",
                "dependency_link",
                &self.id,
                &[
                    ("package_id", dependency.package_id.as_str()),
                    ("version", dependency.version.as_str()),
                ],
            ));
        }
        for handler in &self.handlers {
            records.push(link_record(
                &handler.id,
                "PackageHandler",
                "compiled_handler",
                &self.id,
                &[
                    ("kind", handler.kind.as_str()),
                    ("capability", handler.capability.as_str()),
                ],
            ));
        }
        for trigger in &self.triggers {
            records.push(link_record(
                &trigger.id,
                "PackageTrigger",
                "trigger_rule",
                &self.id,
                &[
                    ("kind", trigger.kind.as_str()),
                    ("match_prompt", trigger.match_prompt.as_str()),
                    ("handler", trigger.handler_id.as_str()),
                ],
            ));
        }
        for permission in &self.permissions {
            records.push(link_record(
                &permission.id,
                "PackagePermission",
                "permission_grant",
                &self.id,
                &[
                    ("effect", permission.effect.as_str()),
                    ("capability", permission.capability.as_str()),
                    ("description", permission.description.as_str()),
                ],
            ));
        }
        records
    }
}

impl PackageStore {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            packages: BTreeMap::new(),
        }
    }

    pub fn install(&mut self, package: AssociativePackage) -> Result<(), PackageInstallError> {
        for dependency in &package.dependencies {
            let Some(installed) = self.packages.get(&dependency.package_id) else {
                return Err(PackageInstallError::MissingDependency {
                    package_id: package.id.clone(),
                    dependency_id: dependency.package_id.clone(),
                });
            };
            if !dependency.version.is_empty() && installed.version != dependency.version {
                return Err(PackageInstallError::VersionMismatch {
                    package_id: package.id.clone(),
                    dependency_id: dependency.package_id.clone(),
                    required: dependency.version.clone(),
                    installed: installed.version.clone(),
                });
            }
        }
        self.packages.insert(package.id.clone(), package);
        Ok(())
    }

    #[must_use]
    pub fn replay(&self, prompt: &str) -> Option<PackageReplay> {
        self.packages
            .values()
            .find_map(|package| package.replay(prompt))
    }

    #[must_use]
    pub fn permission_for_capability(&self, capability: &str) -> PackagePermissionDecision {
        for package in self.packages.values() {
            if let Some(permission) = package.grants_capability(capability) {
                return PackagePermissionDecision::Allowed {
                    package_id: package.id.clone(),
                    permission_id: permission.id.clone(),
                    capability: capability.to_owned(),
                };
            }
        }
        PackagePermissionDecision::Denied {
            capability: capability.to_owned(),
            reason: format!("no installed associative package grants {capability}"),
        }
    }

    #[must_use]
    pub fn permission_for_tool(&self, tool_name: &str) -> PackagePermissionDecision {
        self.permission_for_capability(&format!("tool:{tool_name}"))
    }

    #[must_use]
    pub fn packages(&self) -> Vec<&AssociativePackage> {
        self.packages.values().collect()
    }
}

#[must_use]
pub fn default_associative_packages() -> Vec<AssociativePackage> {
    vec![
        AssociativePackage::new(
            "pkg_formal_ai_core",
            "formal-ai core package",
            KNOWLEDGE_SCHEMA_VERSION,
        )
        .with_handler(PackageHandler::new(
            "handler_calculator",
            "rust_handler",
            "tool:calculator",
        ))
        .with_trigger(PackageTrigger::new(
            "trigger_calculator_tool_call",
            "tool_invocation",
            "calculator",
            "handler_calculator",
        ))
        .with_handler(PackageHandler::new(
            "handler_web_search",
            "rust_handler",
            "tool:web_search",
        ))
        .with_trigger(PackageTrigger::new(
            "trigger_web_search_tool_call",
            "tool_invocation",
            "web_search",
            "handler_web_search",
        ))
        .with_handler(PackageHandler::new(
            "handler_javascript_execution",
            "rust_handler",
            "tool:javascript_execution",
        ))
        .with_trigger(PackageTrigger::new(
            "trigger_javascript_execution_tool_call",
            "tool_invocation",
            "javascript_execution",
            "handler_javascript_execution",
        ))
        .with_permission("tool:calculator", "local deterministic calculator tool")
        .with_permission("tool:web_search", "browser-backed web search API")
        .with_permission(
            "tool:javascript_execution",
            "bounded deterministic JavaScript expression execution",
        )
        .with_permission("tool:concept_lookup", "seed-backed concept lookup")
        .with_permission(
            "tool:write_program",
            "seed-backed program template renderer",
        ),
        // Agentic-coding capability grants for issue #468. These tools are run by
        // the *client* (an external agentic CLI, or the in-repo offline driver), not
        // by an in-server handler, so they are permission-only. The chat tool gate
        // still refuses every tool unless `agent_mode` is explicitly opted in, so
        // granting them by default enables no hidden autonomous action — it only lets
        // an agent that already opted in drive the full search → fetch → write → run
        // loop the issue asks for. `tool:web_search` is granted by the core package.
        AssociativePackage::new(
            "pkg_agentic_coding",
            "agentic-coding capability package",
            KNOWLEDGE_SCHEMA_VERSION,
        )
        .with_permission(
            "tool:web_fetch",
            "agentic-coding HTTP source fetch (client-executed)",
        )
        .with_permission(
            "tool:write_file",
            "agentic-coding workspace file write (client-executed)",
        )
        .with_permission(
            "tool:run_command",
            "agentic-coding sandboxed command runner (client-executed)",
        )
        // Capability-*class* grants: an agentic client executes tools in its own
        // isolated sandbox, so what the server authorises is a *kind* of action,
        // not a specific tool name. Any CLI's naming for a class — `web_search`
        // / `websearch`, `read` / `read_file`, `write` / `write_file`, `bash` /
        // `run_command` — maps to the same grant (see
        // `planner::Capability::permission_key`), so the gate admits every real
        // agentic CLI's toolset without a per-name allowlist. The `agent_mode`
        // opt-in remains the real guard on whether tools run at all; these grants
        // only say *which kinds* an opted-in client may drive.
        .with_permission(
            "tool:capability:search",
            "agentic-coding web search (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:fetch",
            "agentic-coding source fetch (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:read",
            "agentic-coding file read (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:write",
            "agentic-coding file write (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:edit",
            "agentic-coding file edit (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:run",
            "agentic-coding command runner (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:grep",
            "agentic-coding code search (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:glob",
            "agentic-coding file glob (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:list_dir",
            "agentic-coding directory listing (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:todo",
            "agentic-coding planning scratchpad (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:subagent",
            "agentic-coding delegation (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:read_many",
            "agentic-coding multi-file read (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:multi_edit",
            "agentic-coding multi-file edit (any CLI naming, client-executed)",
        )
        .with_permission(
            "tool:capability:ask_user",
            "agentic-coding structured user confirmation (any CLI naming)",
        ),
    ]
}

#[must_use]
pub fn default_package_store() -> PackageStore {
    let mut store = PackageStore::new();
    for package in default_associative_packages() {
        store
            .install(package)
            .expect("default packages have no invalid dependencies");
    }
    store
}

#[must_use]
pub(crate) fn default_package_graph_projection() -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for package in default_associative_packages() {
        nodes.push(GraphNode {
            id: package.id.clone(),
            label: package.name.clone(),
            links_notation: package.links_notation(),
        });
        edges.push(GraphEdge {
            from: String::from("formal_ai_knowledge"),
            to: package.id.clone(),
            role: String::from("package"),
        });
        for handler in &package.handlers {
            nodes.push(GraphNode {
                id: handler.id.clone(),
                label: format!("Package handler: {}", handler.capability),
                links_notation: format!("{} kind={}", handler.id, handler.kind),
            });
            edges.push(GraphEdge {
                from: package.id.clone(),
                to: handler.id.clone(),
                role: String::from("package_handler"),
            });
        }
        for trigger in &package.triggers {
            nodes.push(GraphNode {
                id: trigger.id.clone(),
                label: format!("Package trigger: {}", trigger.kind),
                links_notation: format!("{} handler={}", trigger.id, trigger.handler_id),
            });
            edges.push(GraphEdge {
                from: package.id.clone(),
                to: trigger.id.clone(),
                role: String::from("package_trigger"),
            });
            edges.push(GraphEdge {
                from: trigger.id.clone(),
                to: trigger.handler_id.clone(),
                role: String::from("trigger_handler"),
            });
        }
        for permission in &package.permissions {
            nodes.push(GraphNode {
                id: permission.id.clone(),
                label: format!("Package permission: {}", permission.capability),
                links_notation: format!("{} effect={}", permission.id, permission.effect),
            });
            edges.push(GraphEdge {
                from: package.id.clone(),
                to: permission.id.clone(),
                role: String::from("package_permission"),
            });
        }
    }
    (nodes, edges)
}

fn required_child<'a>(
    node: &'a LinoNode,
    name: &'static str,
) -> Result<&'a str, PackageImportError> {
    let value = node.find_child_value(name);
    if value.is_empty() {
        Err(PackageImportError::MissingField(name))
    } else {
        Ok(value)
    }
}

fn link_record(
    record_id: &str,
    record_type: &str,
    subtype: &str,
    source_id: &str,
    fields: &[(&str, &str)],
) -> LinkRecord {
    let mut links = Vec::new();
    push_doublet(&mut links, record_id, "Type");
    push_doublet(&mut links, "Type", record_type);
    push_doublet(&mut links, record_type, "SubType");
    push_doublet(&mut links, "SubType", subtype);
    push_doublet(&mut links, subtype, "Value");
    push_doublet(&mut links, record_id, source_id);
    push_field(
        &mut links,
        record_id,
        "schema_version",
        KNOWLEDGE_SCHEMA_VERSION,
    );
    for (key, value) in fields {
        push_field(&mut links, record_id, key, value);
    }
    LinkRecord {
        stable_id: record_id.to_owned(),
        schema_version: KNOWLEDGE_SCHEMA_VERSION.to_owned(),
        record_type: record_type.to_owned(),
        source_id: source_id.to_owned(),
        links,
    }
}

fn push_field(links: &mut Vec<DoubletLink>, record_id: &str, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    let field = format!("field:{key}");
    let field_value = format!("value:{value}");
    push_doublet(links, record_id, &field);
    push_doublet(links, &field, &field_value);
}

fn push_doublet(links: &mut Vec<DoubletLink>, from: &str, to: &str) {
    links.push(DoubletLink {
        index: stable_id("doublet", &format!("{from}->{to}")),
        from: from.to_owned(),
        to: to.to_owned(),
    });
}
