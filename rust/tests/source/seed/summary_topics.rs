//! Routing cues and topic boundaries for summarization requests.

use super::SUMMARY_TOPICS_LINO;
use super::parser::{parse_lino, split_pipe_list};

/// Top-level summary-topic bundle.
#[derive(Debug, Clone, Default)]
pub struct SummaryTopicSeeds {
    pub triggers: Vec<String>,
    pub reject_substrings: Vec<String>,
    pub reject_exact: Vec<String>,
    pub constraint_markers: Vec<String>,
    pub constraint_label: String,
    pub fallback_response: String,
    pub topic_leads: Vec<String>,
    pub topic_tails: Vec<String>,
}

impl SummaryTopicSeeds {
    /// Return `true` when any trigger appears in the normalized prompt and
    /// none of the rejecters fire. Callers should treat this as the
    /// guard at the top of the handler — keeps the routing decision data-
    /// driven rather than embedded in Rust if-chains.
    #[must_use]
    pub fn matches_trigger(&self, normalized: &str) -> bool {
        let hit = self
            .triggers
            .iter()
            .any(|t| !t.is_empty() && normalized.contains(t.as_str()));
        if !hit {
            return false;
        }
        if self
            .reject_exact
            .iter()
            .any(|r| !r.is_empty() && normalized == r.as_str())
        {
            return false;
        }
        if self
            .reject_substrings
            .iter()
            .any(|r| !r.is_empty() && normalized.contains(r.as_str()))
        {
            return false;
        }
        true
    }

    /// Extract the named subject without using any registry record as a cue.
    #[must_use]
    pub fn subject<'a>(&self, lower: &'a str) -> Option<&'a str> {
        let (position, end) = self
            .triggers
            .iter()
            .filter_map(|cue| {
                lower
                    .find(cue)
                    .map(|position| (position, position + cue.len()))
            })
            .max_by_key(|(_, end)| *end)?;
        let trim = |ch: char| {
            ch.is_whitespace() || matches!(ch, '?' | '!' | '.' | ':' | '？' | '！' | '。' | '।')
        };
        let after = lower[end..].trim_matches(trim);
        let subject = if after.is_empty() {
            lower[..position].trim_matches(trim)
        } else {
            after
        };
        let start = self
            .topic_leads
            .iter()
            .filter_map(|lead| {
                subject
                    .rfind(lead.as_str())
                    .map(|position| position + lead.len())
            })
            .max()
            .unwrap_or(0);
        let subject = subject[start..].trim_matches(trim);
        let end = self
            .topic_tails
            .iter()
            .filter_map(|tail| subject.strip_suffix(tail.as_str()).map(str::len))
            .min()
            .unwrap_or(subject.len());
        let subject = subject[..end].trim_matches(trim);
        (!subject.is_empty()).then_some(subject)
    }

    /// Return `Some(label)` when any constraint marker fires in the prompt.
    #[must_use]
    pub fn constraint_for(&self, normalized: &str) -> Option<&str> {
        let hit = self
            .constraint_markers
            .iter()
            .any(|m| !m.is_empty() && normalized.contains(m.as_str()));
        hit.then_some(self.constraint_label.as_str())
    }
}

#[must_use]
pub fn summary_topic_seeds() -> SummaryTopicSeeds {
    let tree = parse_lino(SUMMARY_TOPICS_LINO);
    let mut seeds = SummaryTopicSeeds::default();
    let Some(root) = tree.children.iter().find(|c| c.name == "summary_topics") else {
        return seeds;
    };
    seeds.triggers = split_pipe_list(root.find_child_value("trigger"))
        .into_iter()
        .map(|t| t.to_lowercase())
        .collect();
    for child in &root.children {
        match child.name.as_str() {
            "reject_substring" => {
                for value in split_pipe_list(&child.id) {
                    seeds.reject_substrings.push(value.to_lowercase());
                }
            }
            "reject_exact" => seeds.reject_exact.push(child.id.to_lowercase()),
            "constraint_marker" => seeds.constraint_markers.push(child.id.to_lowercase()),
            "constraint_label" => seeds.constraint_label.clone_from(&child.id),
            "fallback_response" => seeds.fallback_response.clone_from(&child.id),
            "topic_lead" => seeds.topic_leads.extend(split_pipe_list(&child.id)),
            "topic_tail" => seeds.topic_tails.extend(split_pipe_list(&child.id)),
            _ => {}
        }
    }
    seeds
}
