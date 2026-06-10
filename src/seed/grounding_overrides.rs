//! Grounding override layer for cached external-source records (issue #398).
//!
//! Cached Wikidata / Wiktionary records live under `data/cache/<source>/...`.
//! When upstream is missing or wrong, a human-authored *override* under
//! `data/overrides/<source>/...` — at the *same per-id path* — supplies the
//! corrected value together with a recorded `reason`. The record a consumer
//! finally sees is resolved as `(cache or live API) then overrides`: the
//! override decorates the cache, never replacing the whole record.
//!
//! An override fact that merely repeats a value the cache already carries is
//! *redundant*. `tests/unit/overrides.rs` walks the whole `data/overrides`
//! tree and fails the build until every redundant fact is removed, so the
//! layer can never silently drift away from upstream: once the cache (or a
//! live API refresh) catches up, the override must go.

use super::parser::{parse_lino, LinoNode};

/// One field-level fact carried by an override, e.g. section `labels`, key
/// `hi`, value `KISS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideFact {
    pub section: String,
    pub key: String,
    pub value: String,
}

/// Parse an override or cache `.lino` document and return its single top-level
/// id node (e.g. `Q131560`). Returns `None` for an empty document.
#[must_use]
pub fn parse_record(text: &str) -> Option<LinoNode> {
    parse_lino(text).children.into_iter().next()
}

/// The recorded justification for an override, or `None` when absent or blank.
#[must_use]
pub fn override_reason(record: &LinoNode) -> Option<String> {
    record
        .children
        .iter()
        .find(|child| child.name == "reason")
        .map(|child| child.id.clone())
        .filter(|reason| !reason.trim().is_empty())
}

/// Collect the `section / key value` facts an override asserts, ignoring the
/// top-level `reason` line. `record` is the override's root id node.
#[must_use]
pub fn override_facts(record: &LinoNode) -> Vec<OverrideFact> {
    let mut facts = Vec::new();
    for section in &record.children {
        if section.name == "reason" {
            continue;
        }
        for entry in &section.children {
            facts.push(OverrideFact {
                section: section.name.clone(),
                key: entry.name.clone(),
                value: entry.id.clone(),
            });
        }
    }
    facts
}

/// True when `cache` already carries `fact` (same section, key and value),
/// which makes the override redundant and forces its removal.
#[must_use]
pub fn cache_contains(cache: &LinoNode, fact: &OverrideFact) -> bool {
    cache.children.iter().any(|section| {
        section.name == fact.section
            && section
                .children
                .iter()
                .any(|entry| entry.name == fact.key && entry.id == fact.value)
    })
}

/// Resolve `(cache) then overrides`: return the cache record decorated with the
/// override's facts. Overrides win on conflict; previously absent keys and
/// sections are appended.
#[must_use]
pub fn resolve(cache: &LinoNode, over: &LinoNode) -> LinoNode {
    let mut merged = cache.clone();
    for fact in override_facts(over) {
        let section_index = merged
            .children
            .iter()
            .position(|section| section.name == fact.section)
            .unwrap_or_else(|| {
                merged.children.push(LinoNode {
                    name: fact.section.clone(),
                    id: String::new(),
                    children: Vec::new(),
                });
                merged.children.len() - 1
            });
        let section = &mut merged.children[section_index];
        if let Some(entry) = section
            .children
            .iter_mut()
            .find(|entry| entry.name == fact.key)
        {
            entry.id = fact.value;
        } else {
            section.children.push(LinoNode {
                name: fact.key,
                id: fact.value,
                children: Vec::new(),
            });
        }
    }
    merged
}
