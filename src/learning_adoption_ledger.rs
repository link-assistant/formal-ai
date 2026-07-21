//! Proof that learned items change answers (issue #701, requirement 2).
//!
//! A learning loop that only *emits* proposals proves nothing: the honest
//! question is whether adopting an item changes what the engine answers. This
//! module records that as a **capability delta** — one before/after pair per
//! frontier item:
//!
//! * **before** is read from the frozen frontier record
//!   (`data/meta/learning-frontier-google-trends.lino`), which captured the
//!   engine's verdict *before* the issue-#701 adoption cycle: `intent: unknown`,
//!   routed to human triage;
//! * **after** is produced live by [`FormalAiEngine::answer`] on the very same
//!   prompt, through the production solver path — no test harness, no shortcut.
//!
//! The pair is only counted as adopted when the answer both leaves the unknown
//! path *and* recovers the topic the prompt was generated from
//! ([`web_search_query_for`]); routing to some unrelated capability would be a
//! regression, not an adoption, and is recorded as one.
//!
//! Because "before" is a committed record and "after" is a pure function of the
//! committed seed, the whole ledger is reproducible offline and pins
//! byte-for-byte as `data/meta/learning-adoption-ledger.lino`.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::engine::{normalize_prompt, FormalAiEngine};
use crate::google_trends_catalog::google_trends_catalog;
use crate::learning_cycle::{recorded_google_trends_frontier, FrontierItem};
use crate::solver_handlers::web_search_query_for;

/// The intent the unknown path reports.
const UNKNOWN_INTENT: &str = "unknown";

/// One recorded capability delta: what the engine did with a prompt before the
/// learning cycle's item was adopted, and what it does now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptionPair {
    /// One-based rank of the originating trending topic.
    pub rank: usize,
    /// The topic the prompt was generated from.
    pub topic: String,
    /// Language tag of the prompt.
    pub language: String,
    /// Prompt variation (the frontier class).
    pub variation: String,
    /// The prompt itself.
    pub prompt: String,
    /// Intent recorded before adoption.
    pub before_intent: String,
    /// Intent the production path returns now.
    pub after_intent: String,
    /// The search query the answer resolved the prompt to, if any.
    pub after_query: String,
}

impl AdoptionPair {
    /// Whether the answer recovered exactly the topic the prompt was generated
    /// from — the check that separates "routed somewhere" from "understood".
    #[must_use]
    pub fn topic_recovered(&self) -> bool {
        !self.after_query.is_empty()
            && normalize_prompt(&self.after_query) == normalize_prompt(&self.topic)
    }

    /// Whether this pair is a genuine adoption: unknown before, a real
    /// capability after, and the topic recovered.
    #[must_use]
    pub fn adopted(&self) -> bool {
        self.before_intent == UNKNOWN_INTENT
            && self.after_intent != UNKNOWN_INTENT
            && self.topic_recovered()
    }

    /// The delta slug recorded in the ledger.
    #[must_use]
    pub fn capability_delta(&self) -> String {
        if self.adopted() {
            format!("{}_to_{}", self.before_intent, self.after_intent)
        } else if self.after_intent == UNKNOWN_INTENT {
            String::from("still_unknown")
        } else {
            format!("{}_without_topic_recovery", self.after_intent)
        }
    }
}

/// The full adoption ledger over a recorded frontier, plus the corpus-level
/// unknown rate the ratchet is taken from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptionLedger {
    /// Frontier slug the pairs come from.
    pub frontier: String,
    /// Every before/after pair, in recorded order.
    pub pairs: Vec<AdoptionPair>,
    /// Total prompts in the committed trends corpus.
    pub corpus_prompts: usize,
    /// Corpus prompts on the unknown path before adoption.
    pub corpus_unknown_before: usize,
    /// Corpus prompts on the unknown path now.
    pub corpus_unknown_after: usize,
}

impl AdoptionLedger {
    /// Pairs that demonstrate a real capability delta.
    #[must_use]
    pub fn adopted(&self) -> Vec<&AdoptionPair> {
        self.pairs.iter().filter(|pair| pair.adopted()).collect()
    }

    /// Pairs that did *not* adopt — kept, never dropped (R425).
    #[must_use]
    pub fn unadopted(&self) -> Vec<&AdoptionPair> {
        self.pairs.iter().filter(|pair| !pair.adopted()).collect()
    }

    /// Distinct topics covered by the adopted pairs.
    #[must_use]
    pub fn adopted_topics(&self) -> BTreeSet<&str> {
        self.adopted()
            .iter()
            .map(|pair| pair.topic.as_str())
            .collect()
    }

    /// Distinct languages covered by the adopted pairs.
    #[must_use]
    pub fn adopted_languages(&self) -> BTreeSet<&str> {
        self.adopted()
            .iter()
            .map(|pair| pair.language.as_str())
            .collect()
    }

    /// The corpus unknown rate before adoption, in basis points.
    #[must_use]
    pub fn unknown_rate_before_basis_points(&self) -> usize {
        rate_basis_points(self.corpus_unknown_before, self.corpus_prompts)
    }

    /// The corpus unknown rate after adoption, in basis points. The ratchet a
    /// regression test pins: it may fall, never rise.
    #[must_use]
    pub fn unknown_rate_after_basis_points(&self) -> usize {
        rate_basis_points(self.corpus_unknown_after, self.corpus_prompts)
    }

    /// Render the ledger as the committable Links Notation artifact. Ends
    /// trimmed of trailing whitespace.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("learning_adoption_ledger\n");
        field(&mut out, "record_type", "learning_adoption_ledger");
        field(&mut out, "issue", "701");
        field(&mut out, "frontier", &self.frontier);
        field(
            &mut out,
            "summary",
            "Every item the issue-#701 learning cycle adopted, recorded as a capability delta: \
             the intent the engine returned before adoption (read from the frozen frontier \
             record) against the intent and recovered topic it returns now through the \
             production solver path. A pair counts as adopted only when the prompt leaves the \
             unknown path and the answer recovers the topic the prompt was generated from; \
             anything else is kept here as an unadopted record rather than dropped.",
        );
        count(&mut out, "before_after_pairs", self.pairs.len());
        count(&mut out, "adopted", self.adopted().len());
        count(&mut out, "unadopted", self.unadopted().len());
        count(&mut out, "topics", self.adopted_topics().len());
        count(&mut out, "languages", self.adopted_languages().len());
        count(&mut out, "corpus_prompts", self.corpus_prompts);
        count(
            &mut out,
            "corpus_unknown_before",
            self.corpus_unknown_before,
        );
        count(&mut out, "corpus_unknown_after", self.corpus_unknown_after);
        count(
            &mut out,
            "unknown_rate_before_basis_points",
            self.unknown_rate_before_basis_points(),
        );
        count(
            &mut out,
            "unknown_rate_after_basis_points",
            self.unknown_rate_after_basis_points(),
        );
        count(
            &mut out,
            "unknown_rate_ceiling_basis_points",
            self.unknown_rate_after_basis_points(),
        );
        for pair in &self.pairs {
            out.push_str("  adoption_pair\n");
            let _ = writeln!(out, "    rank \"{}\"", pair.rank);
            nested(&mut out, "topic", &pair.topic);
            nested(&mut out, "language", &pair.language);
            nested(&mut out, "variation", &pair.variation);
            nested(&mut out, "prompt", &pair.prompt);
            nested(&mut out, "before_intent", &pair.before_intent);
            nested(&mut out, "before_routed_to", "human_triage");
            nested(&mut out, "after_intent", &pair.after_intent);
            nested(&mut out, "after_query", &pair.after_query);
            nested(
                &mut out,
                "topic_recovered",
                &pair.topic_recovered().to_string(),
            );
            nested(&mut out, "capability_delta", &pair.capability_delta());
        }
        out.trim_end().to_owned()
    }
}

/// Build the adoption ledger for the recorded Google Trends frontier.
#[must_use]
pub fn google_trends_adoption_ledger() -> AdoptionLedger {
    let recorded = recorded_google_trends_frontier();
    let pairs = recorded.iter().map(adoption_pair).collect();
    let (corpus_prompts, corpus_unknown_after) = corpus_unknown_counts();
    AdoptionLedger {
        frontier: String::from(crate::learning_cycle::GOOGLE_TRENDS_FRONTIER),
        pairs,
        corpus_prompts,
        corpus_unknown_before: recorded.len(),
        corpus_unknown_after,
    }
}

/// Re-answer one recorded frontier prompt through the production path.
fn adoption_pair(item: &FrontierItem) -> AdoptionPair {
    let answer = FormalAiEngine.answer(&item.prompt);
    AdoptionPair {
        rank: item.rank,
        topic: item.query.clone(),
        language: item.language.clone(),
        variation: item.variation.clone(),
        prompt: item.prompt.clone(),
        before_intent: item.engine_intent.clone(),
        after_intent: answer.intent.clone(),
        after_query: web_search_query_for(&item.prompt).unwrap_or_default(),
    }
}

/// The committed trends corpus size and how many of its prompts are still on the
/// unknown path.
fn corpus_unknown_counts() -> (usize, usize) {
    let catalog = google_trends_catalog();
    let mut total = 0usize;
    let mut unknown = 0usize;
    for topic in &catalog.topics {
        for prompt in &topic.prompts {
            total += 1;
            if FormalAiEngine.answer(&prompt.prompt).intent == UNKNOWN_INTENT {
                unknown += 1;
            }
        }
    }
    (total, unknown)
}

/// A rate in basis points, guarding against an empty corpus.
fn rate_basis_points(part: usize, whole: usize) -> usize {
    if whole == 0 {
        return 0;
    }
    part.saturating_mul(10_000) / whole
}

fn field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} \"{}\"", quote(value));
}

fn count(out: &mut String, key: &str, value: usize) {
    let _ = writeln!(out, "  {key} \"{value}\"");
}

fn nested(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "    {key} \"{}\"", quote(value));
}

fn quote(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\\', "\\\\")
        .replace('"', "'")
}
