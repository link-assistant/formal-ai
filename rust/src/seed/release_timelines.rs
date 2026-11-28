//! Publisher query, selector, and wording for release timelines.
//!
//! A timeline record in the installed registry carries no release entries.
//! Explicit test registries can also include captured entries for offline replay.

use std::sync::OnceLock;

use super::RELEASE_TIMELINES_LINO;
use super::parser::{LinoNode, parse_lino};

/// Answer wording for one language, with `{placeholder}` slots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReleaseTimelinePhrasing {
    pub language: String,
    pub released_heading: String,
    pub released_item: String,
    pub announced_heading: String,
    pub announced_item: String,
    pub undated_item: String,
    pub item_separator: String,
    pub section_end: String,
    pub provenance_note: String,
    pub stale_note: String,
}

/// One work in a timeline: its grounding id, localized titles, and release date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseTimelineEntry {
    /// Wikidata entity identifier of the work.
    pub qid: String,
    /// Earliest publication date as `YYYY-MM-DD`, empty when none is announced.
    pub release_date: String,
    /// Localized titles as `(language, title)` pairs, in declaration order.
    pub titles: Vec<(String, String)>,
}

impl ReleaseTimelineEntry {
    /// Title for `language`, falling back to the English one, then to the id.
    #[must_use]
    pub fn title_for(&self, language: &str) -> &str {
        self.titles
            .iter()
            .find(|(candidate, _)| candidate == language)
            .or_else(|| self.titles.iter().find(|(candidate, _)| candidate == "en"))
            .map_or(self.qid.as_str(), |(_, title)| title.as_str())
    }

    /// Four-digit year of the release date, empty when the entry is undated.
    #[must_use]
    pub fn release_year(&self) -> &str {
        self.release_date.split('-').next().unwrap_or_default()
    }
}

/// A dated list of works for one subject, captured from one source snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseTimeline {
    /// Stable slug referenced by `release_timeline <slug>` in `facts.lino`.
    pub slug: String,
    /// Wikidata entity id of the subject the timeline covers.
    pub subject_qid: String,
    /// Human-readable label of the source the snapshot came from.
    pub source_label: String,
    /// Endpoint the snapshot was requested from.
    pub source_url: String,
    /// Repository path of the query that produced the snapshot.
    pub query_file: String,
    /// Repository path of the checked-in raw snapshot.
    pub cache_file: String,
    /// Date the snapshot was taken, as `YYYY-MM-DD`.
    pub retrieved_at: String,
    /// How many days the snapshot is considered current for.
    pub fresh_for_days: i64,
    /// SHA-256 of the checked-in snapshot bytes.
    pub sha256: String,
    /// Localized subject phrases as `(language, phrase)` pairs.
    pub subjects: Vec<(String, String)>,
    /// Works in the timeline, in snapshot order.
    pub entries: Vec<ReleaseTimelineEntry>,
}

impl ReleaseTimeline {
    /// Subject phrase for `language`, falling back to the English one.
    #[must_use]
    pub fn subject_for(&self, language: &str) -> &str {
        self.subjects
            .iter()
            .find(|(candidate, _)| candidate == language)
            .or_else(|| {
                self.subjects
                    .iter()
                    .find(|(candidate, _)| candidate == "en")
            })
            .map_or("", |(_, phrase)| phrase.as_str())
    }
}

/// Request frames and source wording for release timelines in one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSeriesRequest {
    /// Language code shared by the frames and the presentation wording.
    pub language: String,
    /// Normalized question shapes with exactly one `{subject}` slot.
    pub frames: Vec<String>,
    /// Publisher search string with a slot for the named subject.
    pub search: String,
    /// Localized heading text with a slot for the publisher's subject label.
    pub subject: String,
}

/// A publisher query for timelines with an unregistered subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenReleaseSeries {
    /// Endpoint and rendering settings used after the subject is discovered.
    pub timeline: ReleaseTimeline,
    /// Localized request and search grammar.
    pub requests: Vec<ReleaseSeriesRequest>,
}

/// The parsed registry: wording per language plus every declared timeline.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReleaseTimelines {
    /// Publisher query with `{subject}` and `{language}` placeholders.
    pub query: String,
    /// Grammar for identifying a series from a new request.
    pub open_series: Option<OpenReleaseSeries>,
    pub phrasings: Vec<ReleaseTimelinePhrasing>,
    pub timelines: Vec<ReleaseTimeline>,
}

impl ReleaseTimelines {
    /// Look up a timeline by its slug.
    #[must_use]
    pub fn timeline(&self, slug: &str) -> Option<&ReleaseTimeline> {
        self.timelines.iter().find(|timeline| timeline.slug == slug)
    }

    /// Wording for `language`, falling back to the English block.
    #[must_use]
    pub fn phrasing_for(&self, language: &str) -> Option<&ReleaseTimelinePhrasing> {
        self.phrasings
            .iter()
            .find(|phrasing| phrasing.language == language)
            .or_else(|| {
                self.phrasings
                    .iter()
                    .find(|phrasing| phrasing.language == "en")
            })
    }
}

/// The release-timeline registry, parsed once from seed data.
#[must_use]
pub fn release_timelines() -> &'static ReleaseTimelines {
    static REGISTRY: OnceLock<ReleaseTimelines> = OnceLock::new();
    REGISTRY.get_or_init(|| parse_release_timelines(RELEASE_TIMELINES_LINO))
}

/// Parse a registry from Links Notation text (exposed for tests and tools).
#[must_use]
pub fn parse_release_timelines(text: &str) -> ReleaseTimelines {
    let tree = parse_lino(text);
    let Some(root) = tree
        .children
        .iter()
        .find(|node| node.name == "release_timelines")
    else {
        return ReleaseTimelines::default();
    };
    ReleaseTimelines {
        query: root.find_child_value("query").to_owned(),
        open_series: root
            .children
            .iter()
            .find(|node| node.name == "open-series")
            .map(|node| OpenReleaseSeries {
                timeline: parse_timeline(node),
                requests: node
                    .children
                    .iter()
                    .filter(|child| child.name == "request")
                    .map(|request| ReleaseSeriesRequest {
                        language: request.id.clone(),
                        frames: request
                            .children
                            .iter()
                            .filter(|field| field.name == "frame")
                            .map(|field| field.id.clone())
                            .collect(),
                        search: request.find_child_value("search").to_owned(),
                        subject: request.find_child_value("subject").to_owned(),
                    })
                    .collect(),
            }),
        phrasings: root
            .children
            .iter()
            .filter(|node| node.name == "phrasing")
            .map(parse_phrasing)
            .collect(),
        timelines: root
            .children
            .iter()
            .filter(|node| node.name == "timeline")
            .map(parse_timeline)
            .collect(),
    }
}

fn parse_phrasing(node: &LinoNode) -> ReleaseTimelinePhrasing {
    ReleaseTimelinePhrasing {
        language: node.id.clone(),
        released_heading: node.find_child_value("released-heading").to_owned(),
        released_item: node.find_child_value("released-item").to_owned(),
        announced_heading: node.find_child_value("announced-heading").to_owned(),
        announced_item: node.find_child_value("announced-item").to_owned(),
        undated_item: node.find_child_value("undated-item").to_owned(),
        item_separator: node.find_child_value("item-separator").to_owned(),
        section_end: node.find_child_value("section-end").to_owned(),
        provenance_note: node.find_child_value("provenance-note").to_owned(),
        stale_note: node.find_child_value("stale-note").to_owned(),
    }
}

fn parse_timeline(node: &LinoNode) -> ReleaseTimeline {
    ReleaseTimeline {
        slug: if node.id.is_empty() {
            node.name.clone()
        } else {
            node.id.clone()
        },
        subject_qid: node.find_child_value("subject-qid").to_owned(),
        source_label: node.find_child_value("source-label").to_owned(),
        source_url: node.find_child_value("source-url").to_owned(),
        query_file: node.find_child_value("query-file").to_owned(),
        cache_file: node.find_child_value("cache-file").to_owned(),
        retrieved_at: node.find_child_value("retrieved-at").to_owned(),
        fresh_for_days: node
            .find_child_value("fresh-for-days")
            .trim()
            .parse()
            .unwrap_or_default(),
        sha256: node.find_child_value("sha256").to_owned(),
        subjects: localized_pairs(node, "subject"),
        entries: node
            .children
            .iter()
            .filter(|child| child.name == "entry")
            .map(parse_entry)
            .collect(),
    }
}

fn parse_entry(node: &LinoNode) -> ReleaseTimelineEntry {
    ReleaseTimelineEntry {
        qid: node.id.clone(),
        release_date: node.find_child_value("release-date").to_owned(),
        titles: localized_pairs(node, "title"),
    }
}

/// Collect the `<field>` of every `localized <language>` block as
/// `(language, text)` pairs, in declaration order.
fn localized_pairs(node: &LinoNode, field: &str) -> Vec<(String, String)> {
    node.children
        .iter()
        .filter(|child| child.name == "localized")
        .filter_map(|child| {
            let language = child.id.clone();
            let text = child.find_child_value(field).to_owned();
            (!language.is_empty() && !text.is_empty()).then_some((language, text))
        })
        .collect()
}
