//! Canonical proper-name registry loaded from seed data.
//!
//! Issue #699 batch 2 removes the fixed misspelling enumeration that used to
//! live in `src/solver_handlers/user_intent.rs`. That table hardcoded eight
//! people *and* three hand-written typos each, so it could only ever "correct"
//! spellings someone had already anticipated.
//!
//! This registry stores the opposite kind of information: correctly spelled
//! surfaces per language, grounded in Wikidata, with **no** misspellings at
//! all. Approximate matching against them is a language-neutral primitive in
//! `crate::entity_resolution`, so an unanticipated typo of any name the system
//! remembers resolves without touching Rust.

use std::sync::OnceLock;

use super::ENTITY_NAMES_LINO;
use super::parser::{LinoNode, parse_lino};

/// One named entity with its correctly spelled surfaces per language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityName {
    /// Stable slug used in trace logs, e.g. `elon_musk`.
    pub slug: String,
    /// Wikidata entity id grounding the name, e.g. `Q317521`.
    pub grounded_in: String,
    /// Correctly spelled surfaces, in declaration order, across every
    /// supported language. The first entry is the canonical English form.
    pub surfaces: Vec<String>,
}

/// The canonical proper-name registry, parsed once from seed data.
#[must_use]
pub fn entity_names() -> &'static [EntityName] {
    static REGISTRY: OnceLock<Vec<EntityName>> = OnceLock::new();
    REGISTRY.get_or_init(parse_entity_names).as_slice()
}

fn parse_entity_names() -> Vec<EntityName> {
    let tree = parse_lino(ENTITY_NAMES_LINO);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "entity_names")
        .expect("data/seed/entity-names.lino must declare entity_names");
    root.children
        .iter()
        .filter(|node| node.name == "entity")
        .map(parse_entity)
        .collect()
}

fn parse_entity(node: &LinoNode) -> EntityName {
    EntityName {
        slug: node.id.clone(),
        grounded_in: node.find_child_value("grounded-in").to_owned(),
        surfaces: parse_surfaces(node),
    }
}

/// Collect every surface across all `lexeme <lang>` blocks in declaration
/// order. Resolution matches on spelling shape rather than on language, so the
/// surfaces share one flat list; the per-language grouping in the seed file
/// keeps multilingual coverage auditable.
fn parse_surfaces(node: &LinoNode) -> Vec<String> {
    let mut surfaces = Vec::new();
    for lexeme in node.children.iter().filter(|child| child.name == "lexeme") {
        for surface in lexeme
            .children
            .iter()
            .filter(|child| child.name == "surface")
        {
            let text = surface.find_child_value("text");
            if !text.is_empty() && !surfaces.iter().any(|existing| existing == text) {
                surfaces.push(text.to_owned());
            }
        }
    }
    surfaces
}
