//! Semantic meta-language identity.
//!
//! In Wikidata, a meaning is identified by either:
//!
//! - a Q-item (e.g. `Q2095` "food"), which is language-neutral and holds
//!   sitelinks to every language's Wikipedia article describing that
//!   concept, **or**
//! - a Lexeme sense `L<id>-S<id>` (e.g. `L8485-S1` "the English noun hello,
//!   greeting"), which represents one specific meaning of one specific
//!   word in one specific language. Senses are linked across languages
//!   via P5972 (translation) and P5137 (item for this sense).
//!
//! `MeaningId` is the canonical projection used by the formalize step.
//! Equality is on the wikidata fields, not on the surface form that
//! produced it — that way `hello (en)` and `привет (ru)` collapse to the
//! same id when Wikidata records them as senses of the same item.

use std::fmt::{Display, Formatter};

/// Identity of a concept in the Wikidata semantic meta-language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeaningId {
    /// Wikidata Q-item (e.g. `Q42`). Preferred — it is language-neutral.
    pub wikidata_item: Option<String>,
    /// Wikidata Lexeme sense (e.g. `L8485-S1`).
    pub wikidata_sense: Option<String>,
    /// Wiktionary page title used as a fallback id when Wikidata has no
    /// entry. Always namespaced with the source language code, e.g.
    /// `en:hello`, `ru:как дела`.
    pub wiktionary_page: Option<String>,
}

impl MeaningId {
    /// Identity carrying only a Wikidata Q-item.
    #[must_use]
    #[allow(dead_code)]
    pub fn from_item(item: impl Into<String>) -> Self {
        Self {
            wikidata_item: Some(item.into()),
            wikidata_sense: None,
            wiktionary_page: None,
        }
    }

    /// Identity carrying only a Wikidata lexeme sense.
    #[must_use]
    pub fn from_sense(sense: impl Into<String>) -> Self {
        Self {
            wikidata_item: None,
            wikidata_sense: Some(sense.into()),
            wiktionary_page: None,
        }
    }

    /// Fallback identity used when no Wikidata link is available. The
    /// page key is `<lang>:<title>` so the same Wiktionary page across
    /// language editions hashes to different ids.
    #[must_use]
    pub fn from_wiktionary_page(lang: &str, title: &str) -> Self {
        Self {
            wikidata_item: None,
            wikidata_sense: None,
            wiktionary_page: Some(format!("{lang}:{title}")),
        }
    }

    /// Return the most specific id available, preferring Wikidata's
    /// language-neutral Q-item, falling back to a lexeme sense, and
    /// finally to the Wiktionary page id.
    #[must_use]
    pub fn slug(&self) -> String {
        if let Some(item) = self.wikidata_item.as_deref() {
            return format!("wikidata:{item}");
        }
        if let Some(sense) = self.wikidata_sense.as_deref() {
            return format!("wikidata-sense:{sense}");
        }
        if let Some(page) = self.wiktionary_page.as_deref() {
            return format!("wiktionary:{page}");
        }
        // An empty id is a programming error; callers that fail to find
        // anything should not construct a MeaningId at all.
        String::from("meaning:unknown")
    }

    /// True when the id carries at least one Wikidata pointer.
    #[must_use]
    #[allow(dead_code)]
    pub const fn is_wikidata_backed(&self) -> bool {
        self.wikidata_item.is_some() || self.wikidata_sense.is_some()
    }
}

impl Display for MeaningId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.slug())
    }
}
