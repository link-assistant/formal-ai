//! Brainstorm-seed records loaded from `data/seed/brainstorm-seeds.lino`.
//!
//! Each `brainstorm_seeds.category` entry encodes one topic the agent can
//! brainstorm about (e.g. `names` for a code-review tool, `project_ideas`).
//! Per-category `detection_keywords` decide which prompt routes to which
//! category, and the seed's `item` lines provide the canonical pool of
//! answers — so adding a new brainstorm topic (or extending an existing
//! one) does not require touching Rust code.

use super::parser::{parse_lino, split_pipe_list};
use super::BRAINSTORM_SEEDS_LINO;

/// One brainstorm category.
///
/// Carries the slug (used as the trace label), intent (used as the resolved
/// `SymbolicAnswer.intent`), detection keywords that decide which prompt
/// routes here, and the pool of canonical items the response is drawn from.
#[derive(Debug, Clone, Default)]
pub struct BrainstormCategory {
    pub slug: String,
    pub intent: String,
    pub detection_keywords: Vec<String>,
    pub items: Vec<String>,
}

/// Top-level brainstorm-seed bundle — the universal triggers that opt-in to
/// brainstorm mode plus the list of categories the matcher iterates.
#[derive(Debug, Clone, Default)]
pub struct BrainstormSeeds {
    pub triggers: Vec<String>,
    pub categories: Vec<BrainstormCategory>,
}

impl BrainstormSeeds {
    /// Return `true` when any trigger phrase appears in the normalized
    /// prompt (already lowercased by the caller).
    #[must_use]
    pub fn matches_trigger(&self, normalized: &str) -> bool {
        self.triggers
            .iter()
            .any(|trigger| !trigger.is_empty() && normalized.contains(trigger.as_str()))
    }

    /// Pick the first category whose detection keywords match the prompt.
    /// Categories with no keywords act as the default fall-through and are
    /// returned only when no keyword'd category matched first.
    #[must_use]
    pub fn pick_category(&self, normalized: &str) -> Option<&BrainstormCategory> {
        let keyworded = self.categories.iter().find(|category| {
            category
                .detection_keywords
                .iter()
                .any(|keyword| !keyword.is_empty() && normalized.contains(keyword.as_str()))
        });
        keyworded.or_else(|| {
            self.categories
                .iter()
                .find(|category| category.detection_keywords.is_empty())
        })
    }
}

#[must_use]
pub fn brainstorm_seeds() -> BrainstormSeeds {
    let tree = parse_lino(BRAINSTORM_SEEDS_LINO);
    let mut seeds = BrainstormSeeds::default();
    let Some(root) = tree.children.iter().find(|c| c.name == "brainstorm_seeds") else {
        return seeds;
    };
    seeds.triggers = split_pipe_list(root.find_child_value("trigger"))
        .into_iter()
        .map(|t| t.to_lowercase())
        .collect();
    for entry in root.children.iter().filter(|c| c.name == "category") {
        let slug = entry.id.clone();
        if slug.is_empty() {
            continue;
        }
        let detection_keywords = split_pipe_list(entry.find_child_value("detection_keywords"))
            .into_iter()
            .map(|k| k.to_lowercase())
            .collect();
        let items: Vec<String> = entry
            .children
            .iter()
            .filter(|c| c.name == "item")
            .map(|c| c.id.clone())
            .filter(|s| !s.is_empty())
            .collect();
        if items.is_empty() {
            continue;
        }
        seeds.categories.push(BrainstormCategory {
            slug,
            intent: entry.find_child_value("intent").to_string(),
            detection_keywords,
            items,
        });
    }
    seeds
}
