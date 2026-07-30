//! Statement-level deduplication over a set of sourced observations.
//!
//! Issue #844 asks for a merge that is *conservative*, *explainable*, and
//! *reversible*:
//!
//! - **Conservative.** Two statements merge only when they reduce to the same
//!   [`StatementSignature`] — the same polarity over the same set of content
//!   terms. Terms are lowercased whole tokens with the seed's function words
//!   dropped; nothing is stemmed, and no synonym table is consulted. "The
//!   parser is fast" merges with "Parser is fast" and with "the fast parser",
//!   but never with "the parser is fast enough" (an extra term) nor with "all
//!   tests pass" vs "some tests pass" (quantifiers are content, not function
//!   words).
//! - **Explainable.** Every absorbed statement produces a [`MergeLink`]
//!   naming the representative it folded into and the signature they share, so
//!   "asserted by 9 of 11 sources" can always be traced back to the exact
//!   sentences and sources behind it.
//! - **Reversible.** [`DedupReport::split`] undoes one merge: the merged node
//!   is replaced by one node per absorbed variant and its merge links are
//!   dropped, which is the "a merge that conflates two facts can be split when
//!   new evidence arrives" requirement of `VISION.md:176`.
//!
//! Negation is folded into the signature's [`Polarity`] rather than left in the
//! terms, so "the parser is fast" and "the parser is not fast" land on the same
//! term set with opposite signs — which is exactly how the pair is recognized
//! as a [`Contradiction`] instead of being reported as two unrelated facts.

use super::vocabulary;
use crate::engine::stable_id;
use crate::relative_meta_logic::SourceTier;
use crate::summarization::Statement;

/// Which side of a term set a statement takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Polarity {
    /// The statement affirms its terms ("the parser is fast").
    #[default]
    Asserted,
    /// The statement denies its terms ("the parser is not fast").
    Denied,
}

impl Polarity {
    /// Stable slug used in signature keys and trace payloads.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Asserted => "asserted",
            Self::Denied => "denied",
        }
    }

    /// The opposite polarity — the sign a contradicting twin carries.
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Asserted => Self::Denied,
            Self::Denied => Self::Asserted,
        }
    }
}

/// The canonical form two statements must share to be merged: a polarity plus
/// the sorted, de-duplicated set of content terms.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatementSignature {
    /// Whether the terms are affirmed or denied.
    pub polarity: Polarity,
    /// Sorted, de-duplicated lowercased content terms.
    pub terms: Vec<String>,
}

impl StatementSignature {
    /// Reduce `text` to its signature.
    ///
    /// Tokenize → drop function words → extract negation cues (setting the
    /// polarity) → sort and de-duplicate. Sorting is what makes "the fast
    /// parser" and "the parser is fast" the same fact; it also means word order
    /// carries no meaning here, which is deliberate — this signature answers
    /// "same claim?", not "same sentence?".
    #[must_use]
    pub fn of(text: &str) -> Self {
        let tokens = vocabulary::tokenize(text);
        let content = vocabulary::strip_words(&tokens, vocabulary::function_words());
        let polarity = if vocabulary::mentions_word(&content, vocabulary::negation_cues()) {
            Polarity::Denied
        } else {
            Polarity::Asserted
        };
        let mut terms = vocabulary::strip_words(&content, vocabulary::negation_cues());
        terms.sort_unstable();
        terms.dedup();
        Self { polarity, terms }
    }

    /// The signature of this statement's denial — same terms, opposite sign.
    #[must_use]
    pub fn negated(&self) -> Self {
        Self {
            polarity: self.polarity.flipped(),
            terms: self.terms.clone(),
        }
    }

    /// A stable textual key: `"asserted:fast is parser"`. Two statements share a
    /// key if and only if they share a signature.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}:{}", self.polarity.slug(), self.terms.join(" "))
    }

    /// Content-addressed identifier derived from [`Self::key`]. The same fact
    /// always gets the same id, in this run and in any later run.
    #[must_use]
    pub fn id(&self) -> String {
        stable_id("statement", &self.key())
    }

    /// `true` when the signature carries no content term at all (a sentence made
    /// only of function words). Such statements are never merged with each
    /// other — there is no evidence they say the same thing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

/// One statement as observed from one source.
#[derive(Debug, Clone)]
pub struct SourcedStatement {
    /// The statement itself, already formalized by [`super::formalize`].
    pub statement: Statement,
    /// Stable label of the source that asserted it (URL, host, handle).
    pub source: String,
    /// The source's trust tier, used by [`super::importance`] to weigh it.
    pub tier: SourceTier,
}

impl SourcedStatement {
    /// Build an observation from its parts.
    #[must_use]
    pub fn new(statement: Statement, source: impl Into<String>, tier: SourceTier) -> Self {
        Self {
            statement,
            source: source.into(),
            tier,
        }
    }

    /// Build an observation by formalizing a raw sentence, inferring its kind
    /// and static weight the same way the rest of the pipeline does.
    #[must_use]
    pub fn from_sentence(text: &str, source: impl Into<String>, tier: SourceTier) -> Self {
        let kind = super::classify_sentence(text);
        Self::new(
            Statement::new(text.trim(), kind, super::weight_for_kind(kind)),
            source,
            tier,
        )
    }
}

/// One sentence that folded into a merged fact, with the source that said it.
#[derive(Debug, Clone)]
pub struct StatementVariant {
    /// The sentence, verbatim.
    pub text: String,
    /// The source that asserted it.
    pub source: String,
    /// That source's trust tier.
    pub tier: SourceTier,
}

/// One fact, asserted by one or more sources.
#[derive(Debug, Clone)]
pub struct MergedStatement {
    /// Content-addressed id, equal to [`StatementSignature::id`] for a merged
    /// node and suffixed with the variant index for a node produced by
    /// [`DedupReport::split`].
    pub id: String,
    /// The shared signature.
    pub signature: StatementSignature,
    /// The first-observed sentence, kept verbatim for display.
    pub representative: Statement,
    /// Every (sentence, source) pair that folded into this node, in first-seen
    /// order, starting with the representative's own.
    pub variants: Vec<StatementVariant>,
}

impl MergedStatement {
    /// Distinct sources asserting this fact, in first-seen order. A source that
    /// repeats itself is counted once, so "asserted by 9 of 11 sources" counts
    /// sources and not sentences.
    #[must_use]
    pub fn sources(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for variant in &self.variants {
            if !out.contains(&variant.source.as_str()) {
                out.push(&variant.source);
            }
        }
        out
    }

    /// Distinct (source, tier) pairs, in first-seen order. When a source is seen
    /// at two tiers the first one wins, matching `sources()`.
    #[must_use]
    pub fn evidence(&self) -> Vec<(&str, SourceTier)> {
        let mut out: Vec<(&str, SourceTier)> = Vec::new();
        for variant in &self.variants {
            if !out.iter().any(|(source, _)| *source == variant.source) {
                out.push((&variant.source, variant.tier));
            }
        }
        out
    }

    /// How many distinct sources assert this fact.
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.sources().len()
    }

    /// The static kind prior of the representative sentence.
    #[must_use]
    pub const fn prior(&self) -> u8 {
        self.representative.weight
    }
}

/// The record of one absorption: why two sentences became one fact.
#[derive(Debug, Clone)]
pub struct MergeLink {
    /// Id of the node the sentence folded into.
    pub representative: String,
    /// The absorbed sentence, verbatim.
    pub absorbed: String,
    /// The source that contributed the absorbed sentence.
    pub source: String,
    /// The shared signature key that justified the merge.
    pub justification: String,
}

/// An affirmed fact and its denial, both present in the evidence.
#[derive(Debug, Clone)]
pub struct Contradiction {
    /// Id of the affirmed node.
    pub asserted: String,
    /// Id of the denied node.
    pub denied: String,
    /// The term set both sides share.
    pub terms: Vec<String>,
}

/// The result of a deduplication pass.
#[derive(Debug, Clone, Default)]
pub struct DedupReport {
    /// One node per (signature, polarity), in first-seen order.
    pub statements: Vec<MergedStatement>,
    /// One link per absorbed sentence.
    pub links: Vec<MergeLink>,
    /// Affirmed/denied twins found among the nodes.
    pub contradictions: Vec<Contradiction>,
    /// Distinct sources seen across all observations, in first-seen order.
    pub sources: Vec<String>,
}

impl DedupReport {
    /// The node with `id`, if present.
    #[must_use]
    pub fn statement(&self, id: &str) -> Option<&MergedStatement> {
        self.statements.iter().find(|node| node.id == id)
    }

    /// The merge links that justify node `id` — the explainable trail from the
    /// merged fact back to every sentence it absorbed.
    #[must_use]
    pub fn justification(&self, id: &str) -> Vec<&MergeLink> {
        self.links
            .iter()
            .filter(|link| link.representative == id)
            .collect()
    }

    /// Is this node one side of a contradiction?
    #[must_use]
    pub fn is_contradicted(&self, id: &str) -> bool {
        self.contradictions
            .iter()
            .any(|pair| pair.asserted == id || pair.denied == id)
    }

    /// Undo the merge of node `id`: replace it with one node per absorbed
    /// variant and drop its merge links. Returns `false` when `id` is unknown or
    /// carries a single variant (nothing was merged, so nothing can be split).
    ///
    /// This is the reversibility requirement: when new evidence shows a merge
    /// conflated two facts, the conflation can be undone without recomputing
    /// the whole report.
    pub fn split(&mut self, id: &str) -> bool {
        let Some(position) = self.statements.iter().position(|node| node.id == id) else {
            return false;
        };
        if self.statements[position].variants.len() < 2 {
            return false;
        }
        let node = self.statements.remove(position);
        let mut replacements = Vec::with_capacity(node.variants.len());
        for (index, variant) in node.variants.iter().enumerate() {
            let mut representative = node.representative.clone();
            representative.text.clone_from(&variant.text);
            replacements.push(MergedStatement {
                id: format!("{}_{index}", node.id),
                signature: node.signature.clone(),
                representative,
                variants: vec![variant.clone()],
            });
        }
        for (offset, replacement) in replacements.into_iter().enumerate() {
            self.statements.insert(position + offset, replacement);
        }
        self.links.retain(|link| link.representative != id);
        self.contradictions
            .retain(|pair| pair.asserted != id && pair.denied != id);
        true
    }
}

/// Merge `observations` into one node per distinct fact.
///
/// N sources asserting the same fact yield exactly one node whose `sources`
/// lists all N and whose merge links justify every absorption. A source that
/// repeats itself contributes its sentence as a variant but is counted once.
#[must_use]
pub fn deduplicate(observations: &[SourcedStatement]) -> DedupReport {
    let mut report = DedupReport::default();
    for observation in observations {
        let text = observation.statement.text.trim().to_string();
        if text.is_empty() {
            continue;
        }
        if !report.sources.contains(&observation.source) {
            report.sources.push(observation.source.clone());
        }
        let signature = StatementSignature::of(&text);
        let id = signature.id();
        let existing = report
            .statements
            .iter_mut()
            .find(|node| !node.signature.is_empty() && node.signature == signature);
        let variant = StatementVariant {
            text: text.clone(),
            source: observation.source.clone(),
            tier: observation.tier,
        };
        if let Some(node) = existing {
            if node
                .variants
                .iter()
                .any(|seen| seen.text == variant.text && seen.source == variant.source)
            {
                // The same source repeating the same sentence adds nothing.
                continue;
            }
            node.variants.push(variant);
            report.links.push(MergeLink {
                representative: node.id.clone(),
                absorbed: text,
                source: observation.source.clone(),
                justification: signature.key(),
            });
        } else {
            let mut representative = observation.statement.clone();
            representative.text.clone_from(&text);
            report.statements.push(MergedStatement {
                // A signature with no content term cannot address itself
                // (every such sentence would collide), so it falls back to
                // addressing its own text.
                id: if signature.is_empty() {
                    stable_id("statement", &text)
                } else {
                    id
                },
                signature,
                representative,
                variants: vec![variant],
            });
        }
    }
    report.contradictions = find_contradictions(&report.statements);
    report
}

fn find_contradictions(statements: &[MergedStatement]) -> Vec<Contradiction> {
    let mut out = Vec::new();
    for node in statements {
        if node.signature.polarity != Polarity::Asserted || node.signature.is_empty() {
            continue;
        }
        let denial = node.signature.negated();
        if let Some(twin) = statements.iter().find(|other| other.signature == denial) {
            out.push(Contradiction {
                asserted: node.id.clone(),
                denied: twin.id.clone(),
                terms: node.signature.terms.clone(),
            });
        }
    }
    out
}
