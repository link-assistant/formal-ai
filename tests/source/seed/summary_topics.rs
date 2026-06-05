//! Summarization-topic records loaded from `data/seed/summary-topics.lino`.
//!
//! Each `topic "<Name>"` entry lists multilingual `detection_keywords` and
//! the canned one-paragraph body the summarization handler emits when the
//! prompt picks that topic. The seed also encodes:
//!
//! - `trigger` — universal phrases that opt prompts into "summarize a topic"
//!   mode (`summarize`, `summarise`, `резюме`, `概括`, …).
//! - `reject_substring` — phrases that route to a different handler (e.g.
//!   "summarize this *conversation*"), preventing this matcher from firing.
//! - `reject_exact` — full normalized prompts that route elsewhere (the
//!   single word "summarize" is handled by the conversation summarizer).
//! - `constraint_marker` / `constraint_label` — substrings that surface a
//!   structured constraint in the event log (e.g. "one paragraph" →
//!   `summarization:constraint=one_paragraph`).
//! - `fallback_body` — template used when no topic matched (uses `<topic>`).
//!
//! Adding a new summarization topic in any of the four supported languages
//! therefore does not require touching Rust code.

use super::parser::{parse_lino, split_pipe_list};
use super::SUMMARY_TOPICS_LINO;

/// One summarization topic — the canonical display name, multilingual
/// detection keywords, and the canned body the handler returns.
#[derive(Debug, Clone, Default)]
pub struct SummaryTopic {
    pub display_name: String,
    pub detection_keywords: Vec<String>,
    pub body: String,
}

/// Top-level summary-topic bundle.
#[derive(Debug, Clone, Default)]
pub struct SummaryTopicSeeds {
    pub triggers: Vec<String>,
    pub reject_substrings: Vec<String>,
    pub reject_exact: Vec<String>,
    pub constraint_markers: Vec<String>,
    pub constraint_label: String,
    pub fallback_body: String,
    pub topics: Vec<SummaryTopic>,
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

    /// Pick the first topic whose detection keywords match the prompt.
    #[must_use]
    pub fn pick_topic(&self, normalized: &str) -> Option<&SummaryTopic> {
        self.topics.iter().find(|topic| {
            topic
                .detection_keywords
                .iter()
                .any(|keyword| !keyword.is_empty() && normalized.contains(keyword.as_str()))
        })
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

    /// Render the fallback body when no topic matched (`<topic>` substituted).
    #[must_use]
    pub fn render_fallback(&self, topic: &str) -> String {
        if self.fallback_body.is_empty() {
            return format!("{topic} summary recorded.");
        }
        self.fallback_body.replace("<topic>", topic)
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
            "fallback_body" => seeds.fallback_body.clone_from(&child.id),
            "topic" => {
                if child.id.is_empty() {
                    continue;
                }
                let body = child.find_child_value("body").to_owned();
                if body.is_empty() {
                    continue;
                }
                seeds.topics.push(SummaryTopic {
                    display_name: child.id.clone(),
                    detection_keywords: split_pipe_list(
                        child.find_child_value("detection_keywords"),
                    )
                    .into_iter()
                    .map(|k| k.to_lowercase())
                    .collect(),
                    body,
                });
            }
            _ => {}
        }
    }
    seeds
}
