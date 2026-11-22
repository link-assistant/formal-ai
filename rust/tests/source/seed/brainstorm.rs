//! Request cues and topic transformations loaded from the brainstorm seed.

use super::BRAINSTORM_SEEDS_LINO;
use super::parser::{parse_lino, split_pipe_list};

/// One brainstorm category.
///
/// Carries the slug (used as the trace label), intent (used as the resolved
/// `SymbolicAnswer.intent`), detection keywords that decide which prompt
/// routes here, and any operations applied to the caller’s topic.
#[derive(Debug, Clone, Default)]
pub struct BrainstormCategory {
    pub slug: String,
    pub intent: String,
    pub detection_keywords: Vec<String>,
    pub operations: Vec<String>,
}

/// Top-level brainstorm-seed bundle — the universal triggers that opt-in to
/// brainstorm mode plus the list of categories the matcher iterates.
#[derive(Debug, Clone, Default)]
pub struct BrainstormSeeds {
    pub triggers: Vec<String>,
    pub topic_prefixes: Vec<String>,
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

    /// Read the topic after the most specific matching request cue.
    #[must_use]
    pub fn topic<'a>(&self, normalized: &'a str) -> Option<&'a str> {
        let trigger = self
            .triggers
            .iter()
            .filter(|trigger| !trigger.is_empty() && normalized.contains(trigger.as_str()))
            .max_by_key(|trigger| trigger.chars().count())?;
        let (_, suffix) = normalized.split_once(trigger.as_str())?;
        let suffix = suffix.trim();
        let suffix = suffix.split_once(' ').map_or(suffix, |(word, rest)| {
            if word.bytes().all(|byte| byte.is_ascii_digit()) {
                rest.trim_start()
            } else {
                suffix
            }
        });
        let topic = self
            .topic_prefixes
            .iter()
            .find_map(|prefix| suffix.strip_prefix(prefix.as_str()))
            .unwrap_or(suffix)
            .trim();
        topic.chars().any(char::is_alphabetic).then_some(topic)
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
        let operations: Vec<String> = entry
            .children
            .iter()
            .filter(|c| c.name == "operation")
            .map(|c| c.id.clone())
            .filter(|s| !s.is_empty())
            .collect();
        seeds.categories.push(BrainstormCategory {
            slug,
            intent: entry.find_child_value("intent").to_string(),
            detection_keywords,
            operations,
        });
    }
    seeds
}
