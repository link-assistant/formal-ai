//! Shareable, reviewable package artifacts (issue #668).
//!
//! Export uses the existing package codec so permission gates survive transfer.
//! Import never infers availability or approval from the untrusted manifest:
//! callers supply their local handler catalog and the user's review decision.

use std::collections::{BTreeMap, BTreeSet};
use crate::associative_package::{AssociativePackage, PackageStore};
use crate::links_format::push_lino_node;
use crate::memory::{MemoryEvent, MemoryStore};
use crate::seed::parser::parse_lino;

/// A scoped package plus its portable knowledge and provenance. The contained
/// Links Notation is data; importing this artifact does not execute it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedPackage {
    pub package: AssociativePackage,
    pub contained_links: String,
    pub provenance: Vec<String>,
}

/// Local handler metadata, independent of claims made by a downloaded package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableHandler {
    pub kind: String,
    pub capability: String,
    pub agent_tagged: bool,
}

/// A review is bound to the exact exported artifact, preventing approvals from
/// being reused after the manifest or permission list changes.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PermissionReview {
    pub artifact: String,
    pub acknowledged: bool,
    pub approved_agent_capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharingError {
    InvalidArtifact(String),
    ReviewRequired,
    AgentApprovalRequired(String),
    UnavailableHandler(String),
    InvalidTrigger(String),
    DuplicateIdentifier(String),
    UnsupportedPermission(String),
    Install(String),
}

impl SharedPackage {
    /// Export one package, rather than the user's full memory or credentials.
    #[must_use]
    pub fn export(&self) -> String {
        let mut out = self.package.links_notation();
        out.push('\n');
        push_lino_node(&mut out, 2, "contained_links", Some(&self.contained_links));
        for source in &self.provenance {
            push_lino_node(&mut out, 2, "provenance", Some(source));
        }
        out
    }

    /// Decode a single manifest and retain its opaque knowledge links.
    pub fn parse(text: &str) -> Result<Self, SharingError> {
        let package = AssociativePackage::from_links_notation(text)
            .map_err(|error| SharingError::InvalidArtifact(error.to_string()))?;
        let tree = parse_lino(text);
        if tree.children.len() != 1 {
            return Err(SharingError::InvalidArtifact(String::from("multiple_roots")));
        }
        let root = &tree.children[0];
        Ok(Self {
            package,
            contained_links: root.find_child_value("contained_links").to_owned(),
            provenance: root.children.iter().filter(|node| node.name == "provenance")
                .map(|node| node.id.clone()).collect(),
        })
    }

    /// Permission rows to display before asking the person to confirm import.
    #[must_use]
    pub fn declared_permissions(&self) -> Vec<(String, String, String)> {
        self.package.permissions.iter().map(|permission| (
            permission.capability.clone(), permission.effect.clone(),
            permission.description.clone(),
        )).collect()
    }
}

/// Import atomically with respect to validation: any rejection leaves both the
/// package registry and memory unchanged. Availability checks also cover
/// untriggered handlers, so dormant references cannot bypass the review.
pub fn import_package(
    artifact: &str,
    store: &mut PackageStore,
    memory: &mut MemoryStore,
    available: &BTreeMap<String, AvailableHandler>,
    review: &PermissionReview,
) -> Result<SharedPackage, SharingError> {
    let shared = SharedPackage::parse(artifact)?;
    if !review.acknowledged || review.artifact != artifact {
        return Err(SharingError::ReviewRequired);
    }
    let mut handlers = BTreeSet::new();
    let mut agent_capabilities = BTreeSet::new();
    for handler in &shared.package.handlers {
        if !handlers.insert(handler.id.clone()) {
            return Err(SharingError::DuplicateIdentifier(handler.id.clone()));
        }
        let local = available.get(&handler.id)
            .filter(|local| local.kind == handler.kind && local.capability == handler.capability)
            .ok_or_else(|| SharingError::UnavailableHandler(handler.id.clone()))?;
        if local.agent_tagged { agent_capabilities.insert(local.capability.clone()); }
    }
    let mut triggers = BTreeSet::new();
    for trigger in &shared.package.triggers {
        if !triggers.insert(trigger.id.clone()) {
            return Err(SharingError::DuplicateIdentifier(trigger.id.clone()));
        }
        if !handlers.contains(&trigger.handler_id) {
            return Err(SharingError::InvalidTrigger(trigger.id.clone()));
        }
    }
    for permission in &shared.package.permissions {
        if permission.effect != "allow" && permission.effect != "deny" {
            return Err(SharingError::UnsupportedPermission(permission.effect.clone()));
        }
        if permission.effect == "allow" && !available.values().any(|handler|
            handler.capability == permission.capability) {
            return Err(SharingError::UnsupportedPermission(permission.capability.clone()));
        }
        // A locally agent-tagged capability requires consent even if its handler
        // is not included in this package: permission grants may be used later.
        let is_agent = agent_capabilities.contains(&permission.capability)
            || available.values().any(|handler| handler.agent_tagged
                && handler.capability == permission.capability);
        if permission.effect == "allow" && is_agent
            && !review.approved_agent_capabilities.contains(&permission.capability) {
            return Err(SharingError::AgentApprovalRequired(permission.capability.clone()));
        }
    }
    store.install(shared.package.clone()).map_err(|error| SharingError::Install(error.to_string()))?;
    memory.append(MemoryEvent {
        id: crate::engine::stable_id("package_imported", artifact),
        kind: Some(String::from("package_imported")),
        content: Some(artifact.to_owned()),
        evidence: shared.provenance.clone(),
        ..MemoryEvent::default()
    });
    Ok(shared)
}
