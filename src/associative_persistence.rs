//! Usage-weighted persistence of meta-language expressions — issue #686.
//!
//! Issue #686 asks the project to keep a **persistent** version of meta-language
//! expressions saved in **associative links networks**, so that facts are not
//! only operated on but *retained*, with a retention priority driven by how the
//! knowledge is exercised:
//!
//! * **count usages (reads) and changes (writes)** per expression;
//! * **the most frequently used or changed data persists for longer**;
//! * **usages can also be derived from the incoming and outgoing links** of an
//!   expression (its degree in the network);
//! * **keep everything as a link / link network** — not a graph of edges and
//!   vertices, but the same doublet substrate the rest of the project uses.
//!
//! The [design case study](../docs/case-studies/issue-686/README.md) audits the
//! associative stack and finds every substrate already present: the links network
//! is [`SubstitutionGraph`], a read-usage counter already exists on memory events
//! (`MemoryEvent::access_count`, issue #494), and `dreaming::usage_counts` already
//! evicts *lowest-use* records first (an LFU retention policy). What was missing —
//! and what this module supplies — are three connections the issue names
//! explicitly and the existing machinery lacks:
//!
//! 1. a **write/change** counter alongside the read counter, so "frequently
//!    changed" is protective, not just "frequently read";
//! 2. a proper **link-degree** signal that folds in *both* incoming *and* outgoing
//!    links (the existing citation count is incoming-only); and
//! 3. a first-class store that persists **meta-language expressions** themselves as
//!    an associative links network, keyed by their content-addressed id, with a
//!    single [`AssociativeMemory::retention_score`] that combines all four signals.
//!
//! Everything here is pure, deterministic symbolic arithmetic over
//! caller-supplied text and links: no clocks, no randomness, no network. Retention
//! is decided by usage, never by wall-clock time, so a replay of the same reads,
//! writes, and associations yields byte-for-byte the same ranking.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::memory::MemoryEvent;
use crate::substitution::{SubstitutionGraph, SubstitutionLink};
use crate::world_model::Context;

/// The relative importance of each retention signal.
///
/// The retention score of an expression is a weighted sum of four independent
/// signals — reads, writes, incoming links, and outgoing links. The default
/// weights every signal equally: an expression that is read once, changed once,
/// or linked once all move its score by the same amount. Callers that want, for
/// example, changes to protect an expression twice as strongly as reads can raise
/// [`RetentionWeights::write`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionWeights {
    /// Weight applied to each recorded read (usage) of the expression.
    pub read: u64,
    /// Weight applied to each recorded write (change) of the expression.
    pub write: u64,
    /// Weight applied to each incoming associative link (in-degree).
    pub incoming: u64,
    /// Weight applied to each outgoing associative link (out-degree).
    pub outgoing: u64,
}

impl RetentionWeights {
    /// Weigh every retention signal equally (all weights = 1).
    #[must_use]
    pub const fn uniform() -> Self {
        Self {
            read: 1,
            write: 1,
            incoming: 1,
            outgoing: 1,
        }
    }
}

impl Default for RetentionWeights {
    fn default() -> Self {
        Self::uniform()
    }
}

/// A single persisted meta-language expression and its usage counters.
///
/// The `id` is content-addressed over the expression text (or supplied directly
/// when ingesting an already-identified statement), so the same expression always
/// maps to the same node in the associative network. `reads` counts how many times
/// the expression has been recalled/used; `writes` counts how many times it has
/// been asserted or changed. Its associative links live in the enclosing
/// [`AssociativeMemory`]'s network, so degree is a property of the network, not of
/// the record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedExpression {
    /// Content-addressed id — the expression's node name in the links network.
    pub id: String,
    /// The meta-language expression text.
    pub text: String,
    /// How many times the expression has been read (used) so far.
    pub reads: u64,
    /// How many times the expression has been written (changed) so far.
    pub writes: u64,
    /// Context preserved during candidate extraction (time, conversation,
    /// event kind, role, tool, and other precision-bearing qualifiers).
    pub qualifiers: BTreeMap<String, String>,
    /// Ontology/alignment warnings. Misaligned candidates remain inspectable
    /// rather than being silently discarded, following Wikontic's verifier.
    pub validation_issues: Vec<String>,
}

/// One expression paired with the retention score last computed for it.
///
/// Returned by [`AssociativeMemory::retention_scores`] so a caller can render the
/// full ranking (glass-box) without recomputing each score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoredExpression {
    /// The scored expression's id.
    pub id: String,
    /// The retention score: higher means retained longer.
    pub score: u64,
}

/// A usage-weighted, associative persistence store for meta-language expressions.
///
/// The store is *itself* a links network: expressions are content-addressed nodes
/// and their associations are directed [`SubstitutionLink`]s in an embedded
/// [`SubstitutionGraph`]. Reads and writes are counted per expression; the network
/// contributes an independent degree signal. [`AssociativeMemory::retention_score`]
/// combines all four into a single priority, and the eviction helpers forget the
/// lowest-priority expressions first — so the most frequently used, most
/// frequently changed, and most connected knowledge persists for longest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssociativeMemory {
    /// The associative links network: directed expression → expression edges.
    associations: SubstitutionGraph,
    /// The persisted expressions, keyed by id for stable iteration and merge.
    expressions: BTreeMap<String, PersistedExpression>,
}

impl AssociativeMemory {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Persist an expression by its text, returning its content-addressed id.
    ///
    /// A brand-new expression starts with one write (the act of asserting it) and
    /// zero reads. Persisting the *same* text again is treated as a change, so its
    /// write counter increments — re-asserting a fact is a write, exactly as the
    /// issue frames "changes (writes)".
    pub fn persist(&mut self, text: impl Into<String>) -> String {
        let text = text.into();
        let id = stable_id("expression", &text);
        self.persist_identified(id.clone(), text);
        id
    }

    /// Persist an expression under a caller-supplied id (used when ingesting an
    /// already-identified statement, e.g. a [`Context`] statement whose id must be
    /// preserved so its dependency edges still resolve).
    pub fn persist_identified(&mut self, id: impl Into<String>, text: impl Into<String>) {
        let id = id.into();
        let text = text.into();
        self.expressions
            .entry(id.clone())
            .and_modify(|expression| {
                expression.text.clone_from(&text);
                expression.writes = expression.writes.saturating_add(1);
            })
            .or_insert(PersistedExpression {
                id,
                text,
                reads: 0,
                writes: 1,
                qualifiers: BTreeMap::new(),
                validation_issues: Vec::new(),
            });
    }

    /// Record one read (usage) of the expression. Returns `false` if unknown.
    pub fn note_read(&mut self, id: &str) -> bool {
        match self.expressions.get_mut(id) {
            Some(expression) => {
                expression.reads = expression.reads.saturating_add(1);
                true
            }
            None => false,
        }
    }

    /// Record one write (change) of the expression. Returns `false` if unknown.
    pub fn note_write(&mut self, id: &str) -> bool {
        match self.expressions.get_mut(id) {
            Some(expression) => {
                expression.writes = expression.writes.saturating_add(1);
                true
            }
            None => false,
        }
    }

    /// Associate one expression with another (a directed link `from → to`).
    ///
    /// Both endpoints must already be persisted; a self-association is rejected so
    /// degree stays a meaningful measure of *external* connectivity. Returns `true`
    /// if the link was newly added.
    pub fn associate(&mut self, from: &str, to: &str) -> bool {
        if from == to || !self.expressions.contains_key(from) || !self.expressions.contains_key(to)
        {
            return false;
        }
        self.associations.insert_link(from, to)
    }

    /// Whether an expression with this id is persisted.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.expressions.contains_key(id)
    }

    /// Look up a persisted expression by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&PersistedExpression> {
        self.expressions.get(id)
    }

    /// Read-only view of every persisted expression, keyed by id.
    #[must_use]
    pub const fn expressions(&self) -> &BTreeMap<String, PersistedExpression> {
        &self.expressions
    }

    /// The number of persisted expressions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.expressions.len()
    }

    /// Whether the store holds no expressions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.expressions.is_empty()
    }

    /// Recorded reads for an expression (0 if unknown).
    #[must_use]
    pub fn reads(&self, id: &str) -> u64 {
        self.expressions.get(id).map_or(0, |expr| expr.reads)
    }

    /// Recorded writes for an expression (0 if unknown).
    #[must_use]
    pub fn writes(&self, id: &str) -> u64 {
        self.expressions.get(id).map_or(0, |expr| expr.writes)
    }

    /// How many associative links leave this expression (out-degree).
    #[must_use]
    pub fn out_degree(&self, id: &str) -> u64 {
        self.associations
            .links()
            .iter()
            .filter(|link| link.from == id)
            .count() as u64
    }

    /// How many associative links enter this expression (in-degree).
    #[must_use]
    pub fn in_degree(&self, id: &str) -> u64 {
        self.associations
            .links()
            .iter()
            .filter(|link| link.to == id)
            .count() as u64
    }

    /// Total link degree — incoming plus outgoing.
    #[must_use]
    pub fn degree(&self, id: &str) -> u64 {
        let (mut incoming, mut outgoing) = (0_u64, 0_u64);
        for link in self.associations.links() {
            if link.to == id {
                incoming = incoming.saturating_add(1);
            }
            if link.from == id {
                outgoing = outgoing.saturating_add(1);
            }
        }
        incoming.saturating_add(outgoing)
    }

    /// Usage derived purely from the network: the total incoming + outgoing link
    /// degree of the expression. This is the issue's "calculate usages based on
    /// incoming and outgoing links" — an alternative usage signal that needs no
    /// explicit read/write instrumentation.
    #[must_use]
    pub fn link_usage(&self, id: &str) -> u64 {
        self.degree(id)
    }

    /// Recall the bounded associative neighborhood around one expression.
    ///
    /// Traversal follows both incoming and outgoing links, returns each
    /// expression once in deterministic breadth-first order, and counts every
    /// returned expression as read. `max_hops = 0` recalls only the seed. This
    /// is the links-network equivalent of Wikontic's iterative multi-hop
    /// retrieval, without introducing a separate graph representation.
    pub fn recall_related(&mut self, id: &str, max_hops: usize) -> Vec<String> {
        if !self.contains(id) {
            return Vec::new();
        }
        let mut queue = VecDeque::from([(id.to_owned(), 0_usize)]);
        let mut seen = BTreeSet::new();
        let mut recalled = Vec::new();
        while let Some((current, hops)) = queue.pop_front() {
            if !seen.insert(current.clone()) {
                continue;
            }
            self.note_read(&current);
            recalled.push(current.clone());
            if hops >= max_hops {
                continue;
            }
            let mut neighbors = self
                .associations
                .links()
                .into_iter()
                .filter_map(|link| {
                    if link.from == current {
                        Some(link.to)
                    } else if link.to == current {
                        Some(link.from)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            neighbors.sort();
            neighbors.dedup();
            queue.extend(neighbors.into_iter().map(|neighbor| (neighbor, hops + 1)));
        }
        recalled
    }

    /// The retention score under the default (uniform) weights.
    ///
    /// Higher means retained longer. Unknown ids score 0.
    #[must_use]
    pub fn retention_score(&self, id: &str) -> u64 {
        self.retention_score_with(id, RetentionWeights::uniform())
    }

    /// The retention score under caller-supplied weights.
    ///
    /// `score = read·reads + write·writes + incoming·in_degree + outgoing·out_degree`.
    /// Every term is saturating, so a pathological count can never wrap the score.
    #[must_use]
    pub fn retention_score_with(&self, id: &str, weights: RetentionWeights) -> u64 {
        let Some(expression) = self.expressions.get(id) else {
            return 0;
        };
        let (mut incoming, mut outgoing) = (0_u64, 0_u64);
        for link in self.associations.links() {
            if link.to == id {
                incoming = incoming.saturating_add(1);
            }
            if link.from == id {
                outgoing = outgoing.saturating_add(1);
            }
        }
        weights
            .read
            .saturating_mul(expression.reads)
            .saturating_add(weights.write.saturating_mul(expression.writes))
            .saturating_add(weights.incoming.saturating_mul(incoming))
            .saturating_add(weights.outgoing.saturating_mul(outgoing))
    }

    /// Every expression id, ranked most-retained first (score descending).
    ///
    /// Ties break by id ascending so the ranking is fully deterministic.
    #[must_use]
    pub fn retention_ranking(&self) -> Vec<String> {
        self.retention_scores()
            .into_iter()
            .map(|scored| scored.id)
            .collect()
    }

    /// Every expression with its retention score, most-retained first.
    #[must_use]
    pub fn retention_scores(&self) -> Vec<ScoredExpression> {
        let mut scored: Vec<ScoredExpression> = self
            .expressions
            .keys()
            .map(|id| ScoredExpression {
                id: id.clone(),
                score: self.retention_score(id),
            })
            .collect();
        // Most-retained first; deterministic id tie-break.
        scored.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.id.cmp(&right.id))
        });
        scored
    }

    /// Every expression id, ordered least-retained first — the order in which
    /// expressions would be forgotten under storage pressure.
    #[must_use]
    pub fn eviction_order(&self) -> Vec<String> {
        let mut ids = self.retention_ranking();
        ids.reverse();
        ids
    }

    /// Forget one expression: remove its record and every associative link that
    /// touches it. Returns the removed expression, or `None` if it was unknown.
    pub fn forget(&mut self, id: &str) -> Option<PersistedExpression> {
        let removed = self.expressions.remove(id)?;
        let incident: Vec<SubstitutionLink> = self
            .associations
            .links()
            .into_iter()
            .filter(|link| link.from == id || link.to == id)
            .collect();
        for link in incident {
            self.associations.remove_link(&link.from, &link.to);
        }
        Some(removed)
    }

    /// Forget the `count` lowest-scored expressions, returning them in the order
    /// they were evicted (least-retained first). Frequently used, frequently
    /// changed, and well-connected expressions are the last to go.
    pub fn evict_least_used(&mut self, count: usize) -> Vec<PersistedExpression> {
        self.eviction_order()
            .into_iter()
            .take(count)
            .filter_map(|id| self.forget(&id))
            .collect()
    }

    /// Shrink the store to at most `capacity` expressions by forgetting the
    /// lowest-scored ones. Returns the evicted expressions (empty if already within
    /// capacity). This is the retention policy the issue asks for: keep the most
    /// used or changed, drop the rest.
    pub fn retain_most_used(&mut self, capacity: usize) -> Vec<PersistedExpression> {
        let current = self.expressions.len();
        if current <= capacity {
            return Vec::new();
        }
        self.evict_least_used(current - capacity)
    }

    /// Serialize the whole store as a links network in Links Notation.
    ///
    /// Everything is rendered as a link, honoring the issue's "keep everything as a
    /// link / link network" mandate: each expression contributes its text, read,
    /// and write links, and each association contributes an `associates` link. The
    /// output is sorted, so it is byte-for-byte reproducible.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        for expression in self.expressions.values() {
            let _ = writeln!(out, "expression: ({} {})", expression.id, expression.text);
            let _ = writeln!(out, "reads: ({} {})", expression.id, expression.reads);
            let _ = writeln!(out, "writes: ({} {})", expression.id, expression.writes);
            for (name, value) in &expression.qualifiers {
                let _ = writeln!(out, "qualifier:{name}: ({} {value})", expression.id);
            }
            for issue in &expression.validation_issues {
                let _ = writeln!(out, "validation_issue: ({} {issue})", expression.id);
            }
        }
        let mut associations = self.associations.links();
        associations.sort_by(|left, right| {
            left.from
                .cmp(&right.from)
                .then_with(|| left.to.cmp(&right.to))
        });
        for link in associations {
            let _ = writeln!(out, "associates: ({} {})", link.from, link.to);
        }
        out
    }

    /// Build a store from a world-model [`Context`], preserving statement ids.
    ///
    /// Each dependent statement becomes a persisted expression (one write — the
    /// assertion), and each of its dependency edges becomes an associative link
    /// from the dependent statement to the statement it relies on. The result is a
    /// usage-weighted persistence view of the context's meta-language expressions,
    /// so the retention policy can operate over a world model directly.
    #[must_use]
    pub fn from_context(context: &Context) -> Self {
        let mut memory = Self::new();
        for statement in context.statements().values() {
            memory.persist_identified(statement.id.clone(), statement.text.clone());
        }
        for statement in context.statements().values() {
            for dependency in &statement.dependencies {
                memory.associate(&statement.id, &dependency.on);
            }
        }
        memory
    }

    /// Build the associative retention view used by the durable memory and
    /// dreaming runtime.
    ///
    /// Event ids remain the expression ids, persisted read/write counters become
    /// expression counters, and every cross-event reference becomes a directed
    /// associative link. References include explicit `evidence` and legacy ids
    /// embedded in searchable event text, so existing logs are upgraded without
    /// a migration while new logs can use first-class evidence links. Recall
    /// updates this rebuilt view; callers persist counters only by writing the
    /// corresponding durable events.
    #[must_use]
    pub fn from_memory_events(events: &[MemoryEvent]) -> Self {
        let mut memory = Self::new();
        for event in events {
            if event.id.is_empty() {
                continue;
            }
            memory.persist_identified(event.id.clone(), event_expression_text(event));
            if let Some(expression) = memory.expressions.get_mut(&event.id) {
                expression.reads = event.access_count;
                expression.writes = event.write_count.max(1);
                add_qualifier(&mut expression.qualifiers, "kind", event.kind.as_deref());
                add_qualifier(&mut expression.qualifiers, "role", event.role.as_deref());
                add_qualifier(
                    &mut expression.qualifiers,
                    "intent",
                    event.intent.as_deref(),
                );
                add_qualifier(&mut expression.qualifiers, "tool", event.tool.as_deref());
                add_qualifier(
                    &mut expression.qualifiers,
                    "sent_at",
                    event.sent_at.as_deref(),
                );
                add_qualifier(
                    &mut expression.qualifiers,
                    "conversation_id",
                    event.conversation_id.as_deref(),
                );
                add_qualifier(
                    &mut expression.qualifiers,
                    "conversation_title",
                    event.conversation_title.as_deref(),
                );
            }
        }

        for source in events.iter().filter(|event| !event.id.is_empty()) {
            let searchable = event_expression_text(source);
            for target in events.iter().filter(|event| !event.id.is_empty()) {
                if source.id == target.id {
                    continue;
                }
                let explicitly_linked = source.evidence.iter().any(|reference| {
                    reference == &target.id
                        || reference
                            .strip_suffix(&target.id)
                            .is_some_and(|prefix| prefix.ends_with(':'))
                });
                if explicitly_linked || searchable.contains(&target.id) {
                    memory.associate(&source.id, &target.id);
                }
            }
            for reference in &source.evidence {
                let resolved = events.iter().any(|target| {
                    !target.id.is_empty()
                        && (reference == &target.id
                            || reference
                                .strip_suffix(&target.id)
                                .is_some_and(|prefix| prefix.ends_with(':')))
                });
                if !resolved {
                    if let Some(expression) = memory.expressions.get_mut(&source.id) {
                        expression
                            .validation_issues
                            .push(format!("unresolved evidence link: {reference}"));
                    }
                }
            }
        }
        memory
    }
}

fn add_qualifier(qualifiers: &mut BTreeMap<String, String>, name: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        qualifiers.insert(name.to_owned(), value.to_owned());
    }
}

fn event_expression_text(event: &MemoryEvent) -> String {
    [
        event.kind.as_deref(),
        event.role.as_deref(),
        event.intent.as_deref(),
        event.tool.as_deref(),
        event.inputs.as_deref(),
        event.outputs.as_deref(),
        event.content.as_deref(),
        event.conversation_title.as_deref(),
        event.demo_label.as_deref(),
    ]
    .into_iter()
    .flatten()
    .chain(event.evidence.iter().map(String::as_str))
    .collect::<Vec<_>>()
    .join("\n")
}
