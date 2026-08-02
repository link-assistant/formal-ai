//! Proof that the language-gap learning cycle changed what the engine answers
//! (issue #706).
//!
//! Issue #701 established the honest form of a learning claim: a loop that only
//! *emits* proposals proves nothing, so every adopted item is recorded as a
//! **capability delta** — one before/after pair per frontier item. This module
//! applies exactly that form to the language frontier:
//!
//! * **before** is read from the frozen record
//!   (`data/meta/learning-frontier-language-gap.lino`), which captured what the
//!   live engine did with every Spanish prompt *before* the cycle's proposals
//!   were adopted into `data/seed/learned-request-openers.lino`;
//! * **after** is produced live by [`FormalAiEngine::answer`] on the same
//!   prompt, through the production solver path.
//!
//! A pair counts as adopted only when the prompt leaves the unknown path *and*
//! the answer recovers the term the prompt was built around — routing a Spanish
//! request somewhere unrelated would be a regression, not an adoption, and is
//! kept here as an unadopted record rather than dropped (R425).
//!
//! Both halves are pure functions of committed data, so the ledger pins
//! byte-for-byte as `data/meta/language-adoption-ledger.lino`.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::engine::{normalize_prompt, FormalAiEngine};
use crate::learning_cycle::{parse_frontier_record, recorded_frontier, FrontierItem};
use crate::solver_handlers::web_search_query_for;

/// The intent the unknown path reports.
const UNKNOWN_INTENT: &str = "unknown";

/// The frontier this ledger is taken from.
pub const LANGUAGE_GAP_FRONTIER: &str = "language-gap";

/// One recorded capability delta for a language-frontier prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageAdoptionPair {
    /// One-based rank of the prompt inside its class.
    pub rank: usize,
    /// The term the prompt was built around.
    pub query: String,
    /// Language tag of the prompt.
    pub language: String,
    /// The frontier class the prompt belongs to.
    pub variation: String,
    /// The prompt itself.
    pub prompt: String,
    /// Intent recorded before adoption.
    pub before_intent: String,
    /// Intent the production path returns now.
    pub after_intent: String,
    /// The term the answer resolved the prompt to, if any.
    pub after_query: String,
}

impl LanguageAdoptionPair {
    /// Whether the answer recovered exactly the term the prompt was built
    /// around — the check that separates "routed somewhere" from "understood".
    #[must_use]
    pub fn term_recovered(&self) -> bool {
        !self.after_query.is_empty()
            && normalize_prompt(&self.after_query) == normalize_prompt(&self.query)
    }

    /// Whether this pair is a genuine adoption.
    #[must_use]
    pub fn adopted(&self) -> bool {
        self.before_intent == UNKNOWN_INTENT
            && self.after_intent != UNKNOWN_INTENT
            && self.term_recovered()
    }

    /// The delta slug recorded in the ledger.
    #[must_use]
    pub fn capability_delta(&self) -> String {
        if self.adopted() {
            format!("{}_to_{}", self.before_intent, self.after_intent)
        } else if self.after_intent == UNKNOWN_INTENT {
            String::from("still_unknown")
        } else {
            format!("{}_without_term_recovery", self.after_intent)
        }
    }
}

/// Every before/after pair over the frozen language frontier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageAdoptionLedger {
    /// Frontier slug the pairs come from.
    pub frontier: String,
    /// Every pair, in recorded order.
    pub pairs: Vec<LanguageAdoptionPair>,
}

impl LanguageAdoptionLedger {
    /// Pairs that demonstrate a real capability delta.
    #[must_use]
    pub fn adopted(&self) -> Vec<&LanguageAdoptionPair> {
        self.pairs.iter().filter(|pair| pair.adopted()).collect()
    }

    /// Pairs that did *not* adopt — kept, never dropped (R425).
    #[must_use]
    pub fn unadopted(&self) -> Vec<&LanguageAdoptionPair> {
        self.pairs.iter().filter(|pair| !pair.adopted()).collect()
    }

    /// Distinct languages the adopted pairs cover.
    #[must_use]
    pub fn adopted_languages(&self) -> BTreeSet<&str> {
        self.adopted()
            .into_iter()
            .map(|pair| pair.language.as_str())
            .collect()
    }

    /// Distinct frontier classes the adopted pairs cover.
    #[must_use]
    pub fn adopted_classes(&self) -> BTreeSet<&str> {
        self.adopted()
            .into_iter()
            .map(|pair| pair.variation.as_str())
            .collect()
    }

    /// Render the ledger as a Links Notation document.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut document = String::from("language_adoption_ledger\n");
        let _ = writeln!(document, "  record_type \"language_adoption_ledger\"");
        let _ = writeln!(document, "  issue \"706\"");
        let _ = writeln!(document, "  frontier \"{}\"", self.frontier);
        let _ = writeln!(
            document,
            "  summary \"Every prompt the issue-#706 language cycle adopted, recorded as a capability delta: the intent the engine returned before adoption (read from the frozen language frontier record) against the intent and recovered term it returns now through the production solver path. A pair counts as adopted only when the prompt leaves the unknown path and the answer recovers the term the prompt was built around; anything else is kept here as an unadopted record rather than dropped.\""
        );
        let _ = writeln!(document, "  before_after_pairs \"{}\"", self.pairs.len());
        let _ = writeln!(document, "  adopted \"{}\"", self.adopted().len());
        let _ = writeln!(document, "  unadopted \"{}\"", self.unadopted().len());
        let _ = writeln!(
            document,
            "  languages \"{}\"",
            self.adopted_languages().len()
        );
        let _ = writeln!(document, "  classes \"{}\"", self.adopted_classes().len());
        for pair in &self.pairs {
            let _ = writeln!(document, "  adoption_pair");
            let _ = writeln!(document, "    rank \"{}\"", pair.rank);
            let _ = writeln!(document, "    query \"{}\"", pair.query);
            let _ = writeln!(document, "    language \"{}\"", pair.language);
            let _ = writeln!(document, "    variation \"{}\"", pair.variation);
            let _ = writeln!(document, "    prompt \"{}\"", pair.prompt);
            let _ = writeln!(document, "    before_intent \"{}\"", pair.before_intent);
            let _ = writeln!(document, "    before_routed_to \"human_triage\"");
            let _ = writeln!(document, "    after_intent \"{}\"", pair.after_intent);
            let _ = writeln!(document, "    after_query \"{}\"", pair.after_query);
            let _ = writeln!(document, "    term_recovered \"{}\"", pair.term_recovered());
            let _ = writeln!(
                document,
                "    capability_delta \"{}\"",
                pair.capability_delta()
            );
        }
        document
    }
}

/// Build one pair by replaying a frozen frontier item through the live engine.
fn pair_for(item: &FrontierItem) -> LanguageAdoptionPair {
    let engine = FormalAiEngine;
    let answer = engine.answer(&item.prompt);
    LanguageAdoptionPair {
        rank: item.rank,
        query: item.query.clone(),
        language: item.language.clone(),
        variation: item.variation.clone(),
        prompt: item.prompt.clone(),
        before_intent: item.engine_intent.clone(),
        after_intent: answer.intent,
        after_query: web_search_query_for(&item.prompt).unwrap_or_default(),
    }
}

/// The adoption ledger over the frozen language-gap frontier.
#[must_use]
pub fn language_adoption_ledger() -> LanguageAdoptionLedger {
    let items = recorded_frontier(LANGUAGE_GAP_FRONTIER)
        .map(|frontier| parse_frontier_record(frontier.document))
        .unwrap_or_default();
    LanguageAdoptionLedger {
        frontier: String::from(LANGUAGE_GAP_FRONTIER),
        pairs: items.iter().map(pair_for).collect(),
    }
}
