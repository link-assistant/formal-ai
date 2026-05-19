//! Fact-lookup records loaded from `data/seed/facts.lino`.
//!
//! Each `fact_*` entry encodes a single canned fact (e.g. "Tokyo is the
//! capital of Japan") keyed by multilingual `subject_aliases` and
//! `question_keywords`. The matcher fires when at least one alias **and**
//! at least one keyword appear in the normalized prompt, so the data file
//! alone — not Rust code — decides which surface forms route to which
//! fact, in any of the four supported languages.
//!
//! `wikidata` carries one or more `|`-separated Q-IDs that anchor the fact
//! to the structured knowledge graph; each Q-ID is appended to the event
//! log as a separate `wikidata` event so evidence links surface as
//! `wikidata:Qxxx`.
//!
//! `localized` (optional) carries per-language overrides of `summary`,
//! `source`, and `source_kind`. The solver picks the override matching the
//! user's prevailing language and falls back to the outer (English) values
//! when no override exists.

use super::parser::{parse_lino, split_pipe_list, LinoNode};
use super::FACTS_LINO;

/// A language-specific variant of a fact lookup (summary + source).
///
/// Loaded from `localized "<lang>"` blocks nested under a `fact_*` entry in
/// `data/seed/facts.lino`. Empty fields fall back to the parent record so the
/// English text remains the universal default.
#[derive(Debug, Clone, Default)]
pub struct LocalizedFact {
    pub language: String,
    pub subject_label: String,
    pub value_label: String,
    pub summary: String,
    pub source: String,
    pub source_kind: String,
}

/// A canned fact-lookup record from `data/seed/facts.lino`. See the module
/// docs for the matching contract.
///
/// The optional structured fields (`relation`, `subject_qid`, `value_qid`,
/// `subject_label`, `value_label`) anchor a fact to a Wikidata property triple
/// (e.g. `relation = "capital"`, `subject_qid = "Q159"`, `value_qid = "Q649"`
/// for "Moscow is the capital of Russia"). Records that carry these fields
/// pre-warm the fact-query reasoning cache so the browser worker can answer
/// structured questions like "столица России" instantly. Records without them
/// remain in the legacy substring-matching path for free-form facts (e.g.
/// "who painted the Mona Lisa").
#[derive(Debug, Clone)]
pub struct FactRecord {
    pub slug: String,
    pub intent: String,
    pub category: String,
    pub wikidata: Vec<String>,
    pub relation: String,
    pub subject_qid: String,
    pub value_qid: String,
    pub subject_label: String,
    pub value_label: String,
    pub subject_aliases: Vec<String>,
    pub question_keywords: Vec<String>,
    pub summary: String,
    pub source: String,
    pub source_kind: String,
    pub localized: Vec<LocalizedFact>,
}

impl FactRecord {
    /// Pick the localized variant matching `language`, falling back to the
    /// English variant or to `None` if no overrides exist for this fact.
    #[must_use]
    pub fn localized_for(&self, language: &str) -> Option<&LocalizedFact> {
        self.localized
            .iter()
            .find(|loc| loc.language == language)
            .or_else(|| self.localized.iter().find(|loc| loc.language == "en"))
    }

    /// Return the localized summary for `language` (or the default summary).
    #[must_use]
    pub fn summary_for(&self, language: &str) -> &str {
        self.localized_for(language)
            .map(|loc| loc.summary.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(self.summary.as_str())
    }

    /// Return the localized source URL for `language` (or the default source).
    #[must_use]
    pub fn source_for(&self, language: &str) -> &str {
        self.localized_for(language)
            .map(|loc| loc.source.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(self.source.as_str())
    }

    /// Return the localized subject label for `language` (or the default).
    #[must_use]
    pub fn subject_label_for(&self, language: &str) -> &str {
        self.localized_for(language)
            .map(|loc| loc.subject_label.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(self.subject_label.as_str())
    }

    /// Return the localized value label for `language` (or the default).
    #[must_use]
    pub fn value_label_for(&self, language: &str) -> &str {
        self.localized_for(language)
            .map(|loc| loc.value_label.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(self.value_label.as_str())
    }

    /// Return `true` when at least one subject alias **and** at least one
    /// question keyword appear as substrings of `normalized` (which the
    /// caller is expected to lowercase). The conjunction prevents
    /// "what is rust?" from matching the LOTR fact just because both share
    /// a question word, and the alias requirement disambiguates entities.
    #[must_use]
    pub fn matches_normalized(&self, normalized: &str) -> bool {
        let has_subject = self
            .subject_aliases
            .iter()
            .any(|alias| !alias.is_empty() && normalized.contains(alias.as_str()));
        if !has_subject {
            return false;
        }
        // No question keywords configured = match on the subject alone (rare;
        // used for bare entity prompts). Otherwise at least one keyword must
        // be present so the matcher only fires on actual questions.
        if self.question_keywords.is_empty() {
            return true;
        }
        self.question_keywords
            .iter()
            .any(|keyword| !keyword.is_empty() && normalized.contains(keyword.as_str()))
    }
}

#[must_use]
pub fn facts() -> Vec<FactRecord> {
    let tree = parse_lino(FACTS_LINO);
    let mut out = Vec::new();
    let entries: &[LinoNode] = if tree.name.is_empty() {
        tree.children.as_slice()
    } else {
        std::slice::from_ref(&tree)
    };
    for entry in entries {
        if !entry.name.starts_with("fact_") {
            continue;
        }
        let summary = entry.find_child_value("summary").to_string();
        if summary.is_empty() {
            continue;
        }
        let subject_aliases = split_pipe_list(entry.find_child_value("subject_aliases"))
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        let question_keywords = split_pipe_list(entry.find_child_value("question_keywords"))
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        let wikidata = split_pipe_list(entry.find_child_value("wikidata"));
        let mut localized = Vec::new();
        for child in entry.children.iter().filter(|c| c.name == "localized") {
            let lang = child.id.clone();
            if lang.is_empty() {
                continue;
            }
            localized.push(LocalizedFact {
                language: lang,
                subject_label: child.find_child_value("subject_label").to_string(),
                value_label: child.find_child_value("value_label").to_string(),
                summary: child.find_child_value("summary").to_string(),
                source: child.find_child_value("source").to_string(),
                source_kind: child.find_child_value("source_kind").to_string(),
            });
        }
        out.push(FactRecord {
            slug: entry.name.clone(),
            intent: entry.find_child_value("intent").to_string(),
            category: entry.find_child_value("category").to_string(),
            wikidata,
            relation: entry.find_child_value("relation").to_string(),
            subject_qid: entry.find_child_value("subject_qid").to_string(),
            value_qid: entry.find_child_value("value_qid").to_string(),
            subject_label: entry.find_child_value("subject_label").to_string(),
            value_label: entry.find_child_value("value_label").to_string(),
            subject_aliases,
            question_keywords,
            summary,
            source: entry.find_child_value("source").to_string(),
            source_kind: entry.find_child_value("source_kind").to_string(),
            localized,
        });
    }
    out
}
