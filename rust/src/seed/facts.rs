//! Fact-lookup records loaded from `data/seed/facts.lino`.
//!
//! Each `fact_*` entry encodes a single canned fact (e.g. "Tokyo is the
//! capital of Japan") keyed by multilingual `subject_aliases` and
//! `question_keywords`. The matcher fires when at least one alias **and**
//! at least one keyword appear in the normalized prompt — each matched at
//! word boundaries, never as a raw substring inside a longer word (issue
//! #1172) — so the data file alone — not Rust code — decides which surface
//! forms route to which fact, in any of the four supported languages.
//!
//! `wikidata` carries one or more Q-IDs as a parenthesized link list that
//! anchors the fact to the structured knowledge graph; each Q-ID is appended
//! to the event log as a separate `wikidata` event so evidence links surface
//! as `wikidata:Qxxx`.
//!
//! The written records are the facts no committed capture can reproduce. A
//! fact the captures state (a relation grounded in a Wikidata property, both
//! items captured) is not written here at all: `fact_derivation` derives it
//! from `data/seed/fact-captures.lino` and `data/seed/fact-realization.lino`
//! and [`facts`] appends it after the written ones (issue #1172 R9).
//!
//! `localized` (optional) carries per-language overrides of `summary`,
//! `source`, and `source_kind`. The solver picks the override matching the
//! user's prevailing language and falls back to the outer (English) values
//! when no override exists.

use super::FACTS_LINO;
use super::parser::{LinoNode, parse_lino, split_pipe_list};

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
    /// Slug of a `data/seed/release-timelines.lino` timeline that answers this
    /// fact. When set, the answer is *computed* from the checked-in snapshot at
    /// question time instead of being read from `summary`, so a list of dated
    /// works stays ordered and keeps announced titles apart (issue #892).
    pub release_timeline: String,
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
    /// question keyword appear in `normalized` — each matched at word
    /// boundaries, never as a raw substring inside a longer word (issue
    /// #1172: the United States alias "us" used to match inside
    /// "a**us**tralia", so "What is the capital of Australia?" answered with
    /// Washington, D.C. even though no Australia fact exists). `normalized`
    /// is the caller's `engine::normalize_prompt` output. The conjunction
    /// prevents "what is rust?" from matching the LOTR fact just because
    /// both share a question word, and the alias requirement disambiguates
    /// entities.
    #[must_use]
    pub fn matches_normalized(&self, normalized: &str) -> bool {
        let has_subject = self
            .subject_aliases
            .iter()
            .any(|alias| Self::contains_word_sequence(normalized, alias));
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
            .any(|keyword| Self::contains_word_sequence(normalized, keyword))
    }

    /// Does the surface word or phrase `phrase` appear in `normalized` as a
    /// whole word (or whole multi-word phrase)?
    ///
    /// Issue #1172 (R1): a subject alias or question keyword may never match
    /// as a raw substring inside a longer word — "us" must not match inside
    /// "australia", and "capital" must not match inside "capitalism".
    /// Space-delimited scripts therefore match on token boundaries: the
    /// phrase is tokenized with `engine::normalize_prompt` and must appear
    /// as a consecutive token run of `normalized`. Both sides are pushed
    /// through the same normalizer, because callers do not agree on how far
    /// they normalized the prompt: `meta_method_dispatch::try_dispatch`
    /// passes a merely lowercased prompt (so "spider-man" arrives as one
    /// token while the alias "spider man" tokenizes as two), while
    /// `benchmark_prompts::fact_store_resolves` passes full
    /// `normalize_prompt` output. Re-normalizing is idempotent for input
    /// that is already normalized, so one comparison serves both callers
    /// and a hyphenated alias ("человек-паук фильмы") matches a hyphenated
    /// prompt exactly as its unhyphenated twin matches the normalized one.
    /// Scripts written without inter-word spaces (CJK, per
    /// `coding::contains_cjk`) have no token boundaries to honor, so those
    /// phrases keep substring matching — the same contract as
    /// `seed::meanings::surface_present` (issue #386), which this mirrors for
    /// the fact store's own matching path.
    ///
    /// An associated function (rather than a free one) so it ships with the
    /// re-exported [`FactRecord`] type: the seed module keeps `facts` private
    /// and re-exports selected items, and the solver-side subject-alias
    /// reporter in `solver_handlers::benchmark_prompts` needs this exact
    /// comparison to agree with [`Self::matches_normalized`].
    #[must_use]
    pub fn contains_word_sequence(normalized: &str, phrase: &str) -> bool {
        if phrase.is_empty() {
            return false;
        }
        if crate::coding::contains_cjk(phrase) {
            return normalized.contains(phrase);
        }
        let phrase_normalized = crate::engine::normalize_prompt(phrase);
        let phrase_tokens: Vec<&str> = phrase_normalized.split_whitespace().collect();
        if phrase_tokens.is_empty() {
            // A phrase that normalizes to nothing (bare punctuation) is not a
            // surface any prompt can carry; the old substring matcher would
            // have accepted it inside any prompt containing that punctuation.
            return false;
        }
        let prompt_normalized = crate::engine::normalize_prompt(normalized);
        let tokens: Vec<&str> = prompt_normalized.split_whitespace().collect();
        tokens
            .windows(phrase_tokens.len())
            .any(|window| window == phrase_tokens.as_slice())
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
        let release_timeline = entry.find_child_value("release_timeline").to_string();
        // A record needs *an* answer: either a written summary or a timeline to
        // compute one from. Anything else is an incomplete entry.
        if summary.is_empty() && release_timeline.is_empty() {
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
            release_timeline,
            localized,
        });
    }
    // Issue #1172 R9: the records the committed Wikidata captures derive
    // (`fact_derivation`) follow the written ones.
    out.extend(super::fact_derivation::derived_facts().iter().cloned());
    out
}
