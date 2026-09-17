//! The runtime obligation tree an answer must discharge before it may finish (#1138 B5).
//!
//! Plan 05 owns this module. `ObligationOutcome::Satisfied` carries an
//! [`Evidence`] and nothing else, so the type system — not a convention —
//! forbids a satisfied obligation without an observation, and
//! [`need_status_with_observation`] is the single constructor of
//! `NeedStatus::Satisfied`; [`need_ledger_with_execution`] reaches it only from
//! an evidence-bearing satisfied obligation.
//!
//! The tree is built from the clauses a request enumerates, an expectation is
//! derived per clause from the rules of
//! `data/meta/obligation-evidence-contract.lino`, and a clause no expectation
//! can be read out of is split rather than dropped — R710-R9's "unknown clauses
//! cannot be silently discarded", as a type rather than as a review note.

use crate::engine::stable_id;
use crate::event_log::EventLog;
use crate::execution_evidence::Evidence;
use crate::links_format::format_lino_record;
use crate::meta_frame::{NeedLedger, NeedStatus, ProblemFrame};
use crate::protocol::ChatMessage;

/// What must be observed before this node may be called satisfied.
///
/// This is *not* plan 08's `TaskExpectation`, which declares the shape a task's
/// **answer** must take; the two are bridged by
/// `TaskExpectation::to_obligation_expectation` (plan 00 §9 R15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationExpectation {
    /// The named path must exist and its bytes must hash to `sha256` when given.
    FileBytes {
        /// Path the clause names.
        path: String,
        /// Expected digest of the file's bytes, when the clause states one.
        sha256: Option<String>,
    },
    /// The named command must run and exit with `expected_exit`.
    CommandExit {
        /// The command line the clause names.
        command: String,
        /// The exit status the clause requires.
        expected_exit: i64,
    },
    /// The named command's observed output must hash to `sha256`.
    OutputHash {
        /// The command line the clause names.
        command: String,
        /// Expected digest of the observed bytes.
        sha256: String,
    },
    /// One of the generated checks of loop step 6 must pass.
    ///
    /// `check_id` is `"<VerifiedAnswer::derivation_id>:<check slug>"`, owned by
    /// plan 08 (plan 00 §9 R15).
    SymbolicCheck {
        /// The check this node waits on.
        check_id: String,
    },
    /// No expectation could be derived from this clause. Never discarded: such a
    /// node is split, and when it cannot be split it is reported as a gap.
    Underivable {
        /// Why no expectation could be derived.
        reason: String,
    },
}

impl ObligationExpectation {
    /// The slug naming this expectation's shape — the same vocabulary the rule
    /// rows of `data/meta/obligation-evidence-contract.lino` use.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::FileBytes { .. } => "file_bytes",
            Self::CommandExit { .. } => "command_exit",
            Self::OutputHash { .. } => "output_hash",
            Self::SymbolicCheck { .. } => "symbolic_check",
            Self::Underivable { .. } => "underivable",
        }
    }

    /// Whether an observation could ever answer this expectation.
    ///
    /// `Underivable` is the one shape nothing can answer, which is why such a
    /// node is split rather than waited on.
    #[must_use]
    pub const fn is_observable(&self) -> bool {
        !matches!(self, Self::Underivable { .. })
    }

    /// Links Notation projection of one expectation.
    ///
    /// Every variant projects, the underivable one included: a gap that does not
    /// serialize cannot be reported, and reporting it is exactly why the clause
    /// is kept as a node (R710-R9).
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from("obligation_expectation")),
            ("shape", self.slug().to_owned()),
        ];
        match self {
            Self::FileBytes { path, sha256 } => {
                pairs.push(("path", path.clone()));
                if let Some(digest) = sha256 {
                    pairs.push(("sha256", digest.clone()));
                }
            }
            Self::CommandExit {
                command,
                expected_exit,
            } => {
                pairs.push(("command", command.clone()));
                pairs.push(("expected_exit", expected_exit.to_string()));
            }
            Self::OutputHash { command, sha256 } => {
                pairs.push(("command", command.clone()));
                pairs.push(("sha256", sha256.clone()));
            }
            Self::SymbolicCheck { check_id } => pairs.push(("check_id", check_id.clone())),
            Self::Underivable { reason } => pairs.push(("reason", reason.clone())),
        }
        format_lino_record(
            &stable_id("obligation_expectation", &format!("{self:?}")),
            &pairs,
        )
    }
}

/// The outcome of one obligation node. `Satisfied` cannot exist without a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationOutcome {
    /// No observation has been made yet.
    Unattempted,
    /// An observation was made and did not meet the expectation.
    Refuted {
        /// The observation that refuted the expectation.
        record: Evidence,
        /// What did not match.
        mismatch: String,
    },
    /// An observation was made and met the expectation.
    Satisfied {
        /// The observation that discharged the expectation.
        record: Evidence,
    },
    /// No observation is reachable and no split helped; the reason is named.
    Unsatisfiable {
        /// Why nothing can be observed.
        reason: String,
    },
}

impl ObligationOutcome {
    /// The slug naming this outcome; the ledger counts by it.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Unattempted => "unattempted",
            Self::Refuted { .. } => "refuted",
            Self::Satisfied { .. } => "satisfied",
            Self::Unsatisfiable { .. } => "unsatisfiable",
        }
    }

    /// The observation this outcome was reached by, when one reached it.
    ///
    /// `Satisfied` has exactly one field and it is the record, so no path
    /// reaches a satisfied outcome carrying no observation.
    #[must_use]
    pub const fn record(&self) -> Option<&Evidence> {
        match self {
            Self::Refuted { record, .. } | Self::Satisfied { record } => Some(record),
            Self::Unattempted | Self::Unsatisfiable { .. } => None,
        }
    }

    /// Links Notation projection of one outcome, with the record it carries.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from("obligation_outcome")),
            ("outcome", self.slug().to_owned()),
        ];
        match self {
            Self::Unattempted => {}
            Self::Refuted { record, mismatch } => {
                pairs.push(("mismatch", mismatch.clone()));
                pairs.push(("evidence_id", record.evidence_id.clone()));
            }
            Self::Satisfied { record } => {
                pairs.push(("evidence_id", record.evidence_id.clone()));
                pairs.push(("command", record.command.clone()));
                pairs.push((
                    "observed_output_sha256",
                    record.observed_output_sha256.clone(),
                ));
            }
            Self::Unsatisfiable { reason } => pairs.push(("reason", reason.clone())),
        }
        format_lino_record(
            &stable_id("obligation_outcome", &format!("{self:?}")),
            &pairs,
        )
    }
}

/// One node of the obligation tree: the runtime counterpart of a `WorkUnit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationNode {
    /// `stable_id("obligation", &format!("{parent:?}:{depth}:{clause}"))`.
    pub node_id: String,
    /// The parent node id, when this node came from a split.
    pub parent: Option<String>,
    /// The clause exactly as the user wrote it.
    pub clause: String,
    /// UTF-8 byte span of `clause` in the original request (R710-R9).
    pub span: (usize, usize),
    /// The `Need::need_id` this node discharges, when it maps to one.
    pub need_id: Option<String>,
    /// Split depth of this node.
    pub depth: u8,
    /// What must be observed here.
    pub expectation: ObligationExpectation,
    /// What has been observed here.
    pub outcome: ObligationOutcome,
    /// Nodes this clause split into.
    pub children: Vec<Self>,
}

impl ObligationNode {
    /// Build the obligation tree for a request: one node per clause the request
    /// enumerates, each carrying the expectation its clause derives, and — for a
    /// clause no expectation can be read out of — the split
    /// `task_decomposition::split_once_checkable` accepts, bounded by
    /// `max_split_depth`.
    ///
    /// Nothing is ever dropped. A clause that derives no expectation and admits
    /// no split stays in the tree as an `Underivable` leaf with its byte span, so
    /// the answer can report the gap instead of a completion sentence (R710-R9).
    #[must_use]
    pub fn build(request: &str, max_split_depth: u8) -> Self {
        let span = (0, request.len());
        let mut root = Self::leaf(None, request, span, 0, derive_expectation(request));
        let clauses = clauses_with_spans(request);
        if clauses.len() < 2 {
            // A request with one clause is its own obligation; the root node is
            // the node, exactly as the single-target path already behaves.
            root.expand(max_split_depth);
            return root;
        }
        root.expectation = ObligationExpectation::Underivable {
            reason: String::from("request_enumerates_several_clauses"),
        };
        root.children = clauses
            .into_iter()
            .map(|(clause, span)| {
                let mut child = Self::leaf(
                    Some(root.node_id.clone()),
                    &clause,
                    span,
                    1,
                    derive_expectation(&clause),
                );
                child.expand(max_split_depth);
                child
            })
            .collect();
        root
    }

    /// One childless node with its derived expectation and no observation yet.
    fn leaf(
        parent: Option<String>,
        clause: &str,
        span: (usize, usize),
        depth: u8,
        expectation: ObligationExpectation,
    ) -> Self {
        let node_id = stable_id(
            "obligation",
            &format!("{parent:?}:{depth}:{clause}", clause = clause.trim()),
        );
        Self {
            node_id,
            parent,
            clause: clause.trim().to_owned(),
            span,
            need_id: None,
            depth,
            expectation,
            outcome: ObligationOutcome::Unattempted,
            children: Vec::new(),
        }
    }

    /// Post-order: a leaf is discharged when it is `Satisfied` or
    /// `Unsatisfiable`; a node with children is discharged when all of them are.
    ///
    /// **Deviation from plan 05 as written, recorded here:** the plan's sentence
    /// also asks an *interior* node's own outcome to be satisfied or
    /// unsatisfiable. An interior node carries no expectation an observation can
    /// answer — its children carry them — so requiring one would make every tree
    /// with a split permanently undischargeable, which is the opposite of the
    /// gate this type exists to enforce.
    #[must_use]
    pub fn discharged(&self) -> bool {
        if self.children.is_empty() {
            return matches!(
                self.outcome,
                ObligationOutcome::Satisfied { .. } | ObligationOutcome::Unsatisfiable { .. }
            );
        }
        self.children.iter().all(Self::discharged)
    }

    /// The first leaf, in source order, that is not discharged.
    #[must_use]
    pub fn next_open(&self) -> Option<&Self> {
        if self.children.is_empty() {
            return (!self.discharged()).then_some(self);
        }
        self.children.iter().find_map(Self::next_open)
    }

    /// Collect every leaf of the tree, in source order.
    pub fn collect_leaves<'a>(&'a self, out: &mut Vec<&'a Self>) {
        if self.children.is_empty() {
            out.push(self);
            return;
        }
        for child in &self.children {
            child.collect_leaves(out);
        }
    }

    /// Split an underivable clause into the pieces a reader can check, and
    /// recurse into each of them.
    ///
    /// The splitter is the one that already exists —
    /// `task_decomposition::split_once_checkable`, which accepts a split only
    /// when it yields at least two pieces each of which someone can tell is done
    /// — and the bound is the one that already exists,
    /// `recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND`, passed in as
    /// `max_split_depth`. No second splitter and no second bound is declared
    /// here (plan 05 leaf 7).
    fn expand(&mut self, max_split_depth: u8) {
        if self.expectation.is_observable() || self.depth >= max_split_depth {
            return;
        }
        let pieces = crate::task_decomposition::split_once_checkable(&self.clause);
        if pieces.len() < 2 {
            return;
        }
        let mut cursor = 0_usize;
        let mut children = Vec::with_capacity(pieces.len());
        for piece in pieces {
            let trimmed = piece.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Spans stay relative to the original request: a split piece is
            // located where it actually occurs in the parent clause, and a piece
            // the splitter reworded keeps its parent's span rather than a
            // fabricated one.
            let span = self.clause[cursor..]
                .find(trimmed)
                .map_or(self.span, |offset| {
                    let start = self.span.0 + cursor + offset;
                    cursor += offset + trimmed.len();
                    (start, start + trimmed.len())
                });
            let mut child = Self::leaf(
                Some(self.node_id.clone()),
                trimmed,
                span,
                self.depth.saturating_add(1),
                derive_expectation(trimmed),
            );
            child.expand(max_split_depth);
            children.push(child);
        }
        if children.len() >= 2 {
            self.children = children;
        }
    }

    /// Every leaf of the tree, mutably, in source order.
    fn leaves_mut<'a>(&'a mut self, out: &mut Vec<&'a mut Self>) {
        if self.children.is_empty() {
            out.push(self);
            return;
        }
        for child in &mut self.children {
            child.leaves_mut(out);
        }
    }

    /// Whether `record` answers this node's expectation, and what it says.
    ///
    /// An observation discharges only the node whose expectation names its
    /// command or its path (R710-R4); anything else leaves every node alone.
    fn judge(&self, record: &Evidence) -> Option<ObligationOutcome> {
        match &self.expectation {
            ObligationExpectation::FileBytes { path, sha256 } => {
                if !record.names(path) {
                    return None;
                }
                Some(match sha256 {
                    Some(expected) if &record.observed_output_sha256 == expected => {
                        ObligationOutcome::Satisfied {
                            record: record.clone(),
                        }
                    }
                    Some(expected) => ObligationOutcome::Refuted {
                        record: record.clone(),
                        mismatch: self.mismatch_sentence(
                            "file_bytes_digest",
                            &[
                                (concat!("{", "path", "}"), path),
                                (
                                    concat!("{", "observed", "}"),
                                    &record.observed_output_sha256,
                                ),
                                (concat!("{", "expected", "}"), expected),
                            ],
                        ),
                    },
                    None if record.reports_success() => ObligationOutcome::Satisfied {
                        record: record.clone(),
                    },
                    None => ObligationOutcome::Refuted {
                        record: record.clone(),
                        mismatch: self.mismatch_sentence(
                            "file_bytes_no_success",
                            &[(concat!("{", "path", "}"), path)],
                        ),
                    },
                })
            }
            ObligationExpectation::CommandExit {
                command,
                expected_exit,
            } => {
                if !record.names(command) {
                    return None;
                }
                Some(if record.exit_code == Some(*expected_exit) {
                    ObligationOutcome::Satisfied {
                        record: record.clone(),
                    }
                } else {
                    ObligationOutcome::Refuted {
                        record: record.clone(),
                        mismatch: self.mismatch_sentence(
                            "command_exit",
                            &[
                                (concat!("{", "command", "}"), command),
                                (
                                    concat!("{", "observed", "}"),
                                    &record.exit_code.map_or_else(
                                        || String::from("none"),
                                        |code| code.to_string(),
                                    ),
                                ),
                                (concat!("{", "expected", "}"), &expected_exit.to_string()),
                            ],
                        ),
                    }
                })
            }
            ObligationExpectation::OutputHash { command, sha256 } => {
                if !record.names(command) {
                    return None;
                }
                Some(if &record.observed_output_sha256 == sha256 {
                    ObligationOutcome::Satisfied {
                        record: record.clone(),
                    }
                } else {
                    ObligationOutcome::Refuted {
                        record: record.clone(),
                        mismatch: self.mismatch_sentence(
                            "output_hash",
                            &[
                                (concat!("{", "command", "}"), command),
                                (
                                    concat!("{", "observed", "}"),
                                    &record.observed_output_sha256,
                                ),
                                (concat!("{", "expected", "}"), sha256),
                            ],
                        ),
                    }
                })
            }
            ObligationExpectation::SymbolicCheck { check_id } => {
                if !record.names(check_id) {
                    return None;
                }
                Some(if record.reports_success() {
                    ObligationOutcome::Satisfied {
                        record: record.clone(),
                    }
                } else {
                    ObligationOutcome::Refuted {
                        record: record.clone(),
                        mismatch: self.mismatch_sentence(
                            "symbolic_check",
                            &[(concat!("{", "check", "}"), check_id)],
                        ),
                    }
                })
            }
            // Nothing answers a clause no expectation could be read out of. It
            // is split, and failing that it is reported.
            ObligationExpectation::Underivable { .. } => None,
        }
    }

    /// The seeded sentence that names what did not match, in the language the
    /// clause was written in.
    ///
    /// The sentences are rows of `data/seed/obligation-mismatch.lino` in five
    /// languages, never string literals here: a refusal a user cannot read is
    /// as opaque as no refusal at all (R379).
    fn mismatch_sentence(&self, id: &str, bindings: &[(&str, &str)]) -> String {
        let language = crate::language::detect(&self.clause).slug();
        let mut sentence = mismatch_template(id, language);
        for (placeholder, value) in bindings {
            sentence = sentence.replace(placeholder, value);
        }
        sentence
    }

    /// Links Notation projection of the node and, beneath it, its children.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from("obligation_node")),
            ("node_id", self.node_id.clone()),
        ];
        if let Some(parent) = &self.parent {
            pairs.push(("parent", parent.clone()));
        }
        pairs.push(("clause", self.clause.clone()));
        pairs.push(("span_start", self.span.0.to_string()));
        pairs.push(("span_end", self.span.1.to_string()));
        if let Some(need_id) = &self.need_id {
            pairs.push(("need_id", need_id.clone()));
        }
        pairs.push(("depth", self.depth.to_string()));
        pairs.push(("expectation", self.expectation.slug().to_owned()));
        pairs.push(("outcome", self.outcome.slug().to_owned()));
        if let ObligationOutcome::Unsatisfiable { reason } = &self.outcome {
            pairs.push(("reason", reason.clone()));
        }
        if let ObligationExpectation::Underivable { reason } = &self.expectation {
            pairs.push(("underivable_reason", reason.clone()));
        }
        if let Some(record) = self.outcome.record() {
            pairs.push(("evidence_id", record.evidence_id.clone()));
        }
        let mut out = format_lino_record(&self.node_id, &pairs);
        out.push('\n');
        out.push_str(&self.expectation.to_links_notation());
        out.push('\n');
        out.push_str(&self.outcome.to_links_notation());
        for child in &self.children {
            out.push('\n');
            out.push_str(&child.to_links_notation());
        }
        out
    }
}

/// The runtime counterpart of `NeedLedger`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationLedger {
    /// The frame this ledger belongs to.
    pub frame_id: String,
    /// The obligation tree for the frame's request.
    pub root: ObligationNode,
}

impl ObligationLedger {
    /// Build the ledger for one frame's request.
    ///
    /// Each leaf is attributed to the frame need whose source span it covers, so
    /// the join back to the planning ledger is by need rather than by position.
    #[must_use]
    pub fn for_frame(frame: &ProblemFrame, request: &str, max_split_depth: u8) -> Self {
        let mut root = ObligationNode::build(request, max_split_depth);
        let mut leaves = Vec::new();
        root.leaves_mut(&mut leaves);
        for leaf in leaves {
            leaf.need_id = need_for_clause(frame, &leaf.clause);
        }
        Self {
            frame_id: frame.frame_id.clone(),
            root,
        }
    }

    /// Apply one observation to the node whose expectation it answers. Returns the
    /// id of the node the record answered, or `None` when no node expected it —
    /// an unrelated result can never clear a step (R710-R4).
    ///
    /// A record that contradicts the expectation refutes the node rather than
    /// finishing it: a refuted node is still open, and the session may not
    /// finalize while it is.
    pub fn observe(&mut self, record: &Evidence) -> Option<String> {
        let mut leaves = Vec::new();
        self.root.leaves_mut(&mut leaves);
        for leaf in leaves {
            if matches!(leaf.outcome, ObligationOutcome::Satisfied { .. }) {
                continue;
            }
            if let Some(outcome) = leaf.judge(record) {
                leaf.outcome = outcome;
                return Some(leaf.node_id.clone());
            }
        }
        None
    }

    /// Every obligation is `Satisfied` or `Unsatisfiable` with a named reason.
    #[must_use]
    pub fn every_obligation_discharged(&self) -> bool {
        self.root.discharged()
    }

    /// Number of leaves whose outcome is `Satisfied`.
    #[must_use]
    pub fn satisfied_count(&self) -> usize {
        self.count_of("satisfied")
    }

    /// Number of leaves whose outcome is `Unsatisfiable`.
    #[must_use]
    pub fn unsatisfiable_count(&self) -> usize {
        self.count_of("unsatisfiable")
    }

    /// Number of leaves whose outcome is `Unattempted`.
    #[must_use]
    pub fn unattempted_count(&self) -> usize {
        self.count_of("unattempted")
    }

    /// Number of leaves whose outcome is `Refuted`.
    #[must_use]
    pub fn refuted_count(&self) -> usize {
        self.count_of("refuted")
    }

    fn count_of(&self, slug: &str) -> usize {
        let mut leaves = Vec::new();
        self.root.collect_leaves(&mut leaves);
        leaves
            .iter()
            .filter(|leaf| leaf.outcome.slug() == slug)
            .count()
    }

    /// The node ids, with their needs, that an observation discharged.
    #[must_use]
    pub fn satisfied_need_ids(&self) -> Vec<String> {
        let mut leaves = Vec::new();
        self.root.collect_leaves(&mut leaves);
        let mut needs: Vec<String> = leaves
            .iter()
            .filter(|leaf| matches!(leaf.outcome, ObligationOutcome::Satisfied { .. }))
            .filter_map(|leaf| leaf.need_id.clone())
            .collect();
        needs.sort_unstable();
        needs.dedup();
        needs
    }

    /// Links Notation projection of the whole ledger: the counts first, then the
    /// tree, so every clause and its byte span is reportable.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut leaves = Vec::new();
        self.root.collect_leaves(&mut leaves);
        let pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from("obligation_ledger")),
            ("frame_id", self.frame_id.clone()),
            ("node_count", leaves.len().to_string()),
            ("satisfied", self.satisfied_count().to_string()),
            ("refuted", self.refuted_count().to_string()),
            ("unsatisfiable", self.unsatisfiable_count().to_string()),
            ("unattempted", self.unattempted_count().to_string()),
            ("discharged", self.every_obligation_discharged().to_string()),
        ];
        let mut out = format_lino_record(&stable_id("obligation_ledger", &self.frame_id), &pairs);
        out.push('\n');
        out.push_str(&self.root.to_links_notation());
        out
    }
}

/// Emit the obligation ledger and the executed need ledger as append-only
/// events, and return the executed ledger (plan 05 leaf 10).
///
/// The planning ledger stays in the log beside it, untouched: a reader sees both
/// what was planned and what was observed, and the difference between them is
/// the whole of bottleneck B5.
pub(crate) fn record_obligation_ledger(
    log: &mut EventLog,
    frame: &ProblemFrame,
    planned: &NeedLedger,
    obligations: &ObligationLedger,
) -> NeedLedger {
    let executed = need_ledger_with_execution(planned, obligations);
    log.append("obligation_ledger", obligations.to_links_notation());
    log.append(
        "obligation_ledger:discharged",
        obligations.every_obligation_discharged().to_string(),
    );
    log.append("need_ledger:executed", executed.to_links_notation());
    log.append(
        "need_ledger:executed_satisfied",
        executed.count_with(NeedStatus::Satisfied).to_string(),
    );
    debug_assert_eq!(
        frame.frame_id, obligations.frame_id,
        "the obligation ledger and the frame it discharges must be the same frame"
    );
    executed
}

/// The five-language sentences that say what an observation did not match.
const MISMATCH_LINO: &str = include_str!("../data/seed/obligation-mismatch.lino");

/// Record type of one localized mismatch sentence.
const RECORD_MISMATCH: &str = "obligation_mismatch";

/// One localized template from `data/seed/obligation-mismatch.lino`.
///
/// An absent row renders as the empty string rather than as an invented
/// sentence: a visible gap is honest, a fabricated explanation is not.
#[must_use]
pub fn mismatch_template(id: &str, language: &str) -> String {
    let root = crate::seed::parser::parse_lino(MISMATCH_LINO);
    let Some(record) = root.children.iter().find(|node| {
        node.find_child_value("record_type") == RECORD_MISMATCH && node.find_child_value("id") == id
    }) else {
        return String::new();
    };
    let localized = record.find_child_value(language);
    if localized.trim().is_empty() {
        record.find_child_value("en").to_owned()
    } else {
        localized.to_owned()
    }
}

/// The frame need whose source span this clause covers, when one does.
fn need_for_clause(frame: &ProblemFrame, clause: &str) -> Option<String> {
    frame
        .needs
        .iter()
        .filter(|need| {
            clause.contains(need.source_span.as_str()) || need.source_span.contains(clause)
        })
        .max_by_key(|need| need.source_span.len())
        .map(|need| need.need_id.clone())
}

/// What the session must do next about its obligations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationStep {
    /// Make the observation this node expects.
    Observe(ObligationNode),
    /// The node's expectation is `Underivable` and it can still be split.
    Decompose(ObligationNode),
    /// Nothing is left to split and nothing can be observed; report the gap.
    ReportGap {
        /// The node that cannot be discharged.
        node_id: String,
        /// The clause exactly as the user wrote it.
        clause: String,
        /// Its byte span in the original request.
        span: (usize, usize),
        /// Why nothing can be observed.
        reason: String,
    },
}

/// Project a positive observation into the need ledger's terminal status.
///
/// This is the single constructor for [`NeedStatus::Satisfied`] in `src/`.
/// Callers retain their domain-specific evidence (an execution record, a
/// successful prerequisite re-probe, or a formalization observation) and pass
/// whether that observation matched. A missing or contradictory observation
/// preserves `otherwise`; it can never manufacture satisfaction.
#[must_use]
pub const fn need_status_with_observation(
    observation_matches: bool,
    otherwise: NeedStatus,
) -> NeedStatus {
    if observation_matches {
        NeedStatus::Satisfied
    } else {
        otherwise
    }
}

/// The join: a *new* need ledger whose rows are upgraded from `Planned` to
/// `Satisfied` exactly where an obligation carrying an [`Evidence`] discharged
/// the same need. Never mutates its input.
///
/// This is the only execution-ledger path to satisfaction (plan 05 leaf 9); the
/// shared constructor also projects domain observations such as a successful
/// prerequisite re-probe without duplicating the terminal variant.
#[must_use]
pub fn need_ledger_with_execution(
    planned: &NeedLedger,
    obligations: &ObligationLedger,
) -> NeedLedger {
    let discharged = obligations.satisfied_need_ids();
    let mut executed = planned.clone();
    for row in &mut executed.rows {
        // This execution match is reachable only from an
        // `ObligationOutcome::Satisfied`, which cannot be constructed without
        // an `Evidence` (plan 05 leaf 9).
        let observation_matches = discharged.iter().any(|need_id| need_id == &row.need_id);
        row.status = need_status_with_observation(observation_matches, row.status);
    }
    executed
}

/// What the session must do next about `request`, given what the transcript has
/// already observed. Replaces `task_obligations::outstanding`.
///
/// The order is the one plan 05 states and the tests pin: observe what can be
/// observed; failing that, decompose what can still be split; and only when
/// neither is available report the gap, with the clause, its byte span and the
/// reason. A gap is never reported while a split is still on the table, and a
/// completion sentence is never reported in place of a gap (R710-R9).
#[must_use]
pub fn next_step(request: &str, messages: &[ChatMessage]) -> Option<ObligationStep> {
    let mut ledger = ObligationLedger {
        frame_id: stable_id("obligation_ledger", request),
        root: ObligationNode::build(
            request,
            crate::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND,
        ),
    };
    for record in crate::agentic_coding::transcript_evidence::records(messages) {
        ledger.observe(&record);
    }
    let mut leaves = Vec::new();
    ledger.root.collect_leaves(&mut leaves);
    let open: Vec<&ObligationNode> = leaves
        .into_iter()
        .filter(|leaf| !leaf.discharged())
        .collect();
    if open.is_empty() {
        return None;
    }
    if let Some(node) = open
        .iter()
        .find(|leaf| leaf.expectation.is_observable())
        .copied()
    {
        return Some(ObligationStep::Observe(node.clone()));
    }
    if let Some(node) = open
        .iter()
        .find(|leaf| {
            leaf.depth < crate::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND
                && crate::task_decomposition::split_once_checkable(&leaf.clause).len() >= 2
        })
        .copied()
    {
        return Some(ObligationStep::Decompose(node.clone()));
    }
    let node = open[0];
    let reason = match &node.expectation {
        ObligationExpectation::Underivable { reason } => reason.clone(),
        other => other.slug().to_owned(),
    };
    Some(ObligationStep::ReportGap {
        node_id: node.node_id.clone(),
        clause: node.clause.clone(),
        span: node.span,
        reason,
    })
}

mod derivation;

pub use derivation::{
    ExpectationRule, ExpectationRules, clauses_with_spans, derive_expectation, expected_digest,
};
