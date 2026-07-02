//! Canonical model id and public aliases loaded from seed data.
//!
//! The HTTP adapters and protocol serializers use this registry so the public
//! model surface can be changed by seed data instead of special-case matching
//! in request routing.

use std::sync::OnceLock;

use super::parser::{parse_lino, LinoNode};
use super::MODEL_ALIASES_LINO;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelAliasRegistry {
    pub canonical_id: String,
    pub aliases: Vec<String>,
}

impl ModelAliasRegistry {
    #[must_use]
    pub fn accepts(&self, model: &str) -> bool {
        let normalized = normalize_model_id(model);
        self.aliases
            .iter()
            .any(|alias| normalize_model_id(alias) == normalized)
    }
}

#[must_use]
pub fn model_aliases() -> &'static ModelAliasRegistry {
    static REGISTRY: OnceLock<ModelAliasRegistry> = OnceLock::new();
    REGISTRY.get_or_init(parse_model_aliases)
}

#[must_use]
pub fn canonical_model_id() -> &'static str {
    model_aliases().canonical_id.as_str()
}

#[must_use]
pub fn try_resolve_model_id(model: Option<&str>) -> Option<String> {
    let registry = model_aliases();
    let Some(model) = model.map(str::trim).filter(|model| !model.is_empty()) else {
        return Some(registry.canonical_id.clone());
    };
    if registry.accepts(model) {
        Some(registry.canonical_id.clone())
    } else {
        None
    }
}

#[must_use]
pub fn resolve_model_id(model: Option<&str>) -> String {
    try_resolve_model_id(model).unwrap_or_else(|| model_aliases().canonical_id.clone())
}

fn parse_model_aliases() -> ModelAliasRegistry {
    let tree = parse_lino(MODEL_ALIASES_LINO);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "model_aliases")
        .expect("data/seed/model-aliases.lino must declare model_aliases");
    let canonical_id = canonical_from_root(root)
        .expect("data/seed/model-aliases.lino must declare model_aliases canonical");
    let mut aliases = aliases_from_root(root);
    if !aliases
        .iter()
        .any(|alias| normalize_model_id(alias) == normalize_model_id(&canonical_id))
    {
        aliases.insert(0, canonical_id.clone());
    }
    ModelAliasRegistry {
        canonical_id,
        aliases,
    }
}

fn canonical_from_root(root: &LinoNode) -> Option<String> {
    root.children
        .iter()
        .find(|child| child.name == "canonical")
        .map(|child| child.id.clone())
}

fn aliases_from_root(root: &LinoNode) -> Vec<String> {
    root.children
        .iter()
        .filter(|child| child.name == "alias")
        .map(|child| child.id.clone())
        .collect()
}

fn normalize_model_id(model: &str) -> String {
    model.trim().to_lowercase()
}
