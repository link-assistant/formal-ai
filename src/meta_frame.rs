//! Issue #559 Phase 1A: an explicit, link-serializable problem frame.
//!
//! The universal solver already formalizes every prompt into an
//! [`IntentFormalization`]
//! meaning record (root requirement R157). Issue #559 generalizes the solver
//! away from one prompt → one handler intent toward one prompt → a frame of
//! *every* detected need. This module makes that frame first-class and
//! link-native, without changing routing or answers: a [`ProblemFrame`] wraps
//! the formalization, enumerates each [`Need`] found in the prompt (R7), and is
//! serialized to Links Notation via
//! `format_lino_record` (R311). It is
//! emitted as a solver loop event so the meaning record is observable, but the
//! existing dispatch still decides the answer. Later phases build the recursive
//! `WorkUnit` trace, the need-satisfaction ledger, and the method registry on
//! top of this frame.
//!
//! Vocabulary follows `docs/case-studies/issue-559/alignment.md`: `ProblemFrame`
//! = the formalized impulse made explicit; `Need` = a detected requirement or
//! question (root requirement R158); the frame is the data the recursive core
//! decomposes (see `docs/case-studies/issue-559/recursive-core.md`).

use crate::engine::stable_id;
use crate::event_log::EventLog;
use crate::intent_formalization::{formalize_intent, IntentFormalization, IntentKind};
use crate::links_format::format_lino_record;
use crate::translation::formalize_prompt;

/// Formalize a single span the same way the solver formalizes a whole prompt:
/// derive the formalization candidate first (so routes that depend on candidate
/// relevants are recognized) and then classify. This keeps a segment's kind and
/// route consistent with how the engine would route that text on its own.
#[must_use]
fn formalize_span(span: &str, language: &str) -> IntentFormalization {
    let candidate = formalize_prompt(span, language);
    formalize_intent(span, language, Some(&candidate))
}

/// Lifecycle status of a single detected [`Need`].
///
/// Phase 1A records every need as [`NeedStatus::Pending`]: the frame is a
/// trace-only projection that does not yet resolve needs. Phase 2 introduces the
/// need-satisfaction ledger (root requirement R333) that flips each status to
/// `Satisfied`, `Deferred`, `Blocked`, or `Rejected` from the answer projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedStatus {
    /// Detected but not yet resolved (the only status produced in Phase 1A).
    Pending,
    /// A work unit produced a validated result for this need.
    Satisfied,
    /// Intentionally postponed (e.g. out of scope this turn).
    Deferred,
    /// No method or evidence is available; recorded rather than hidden.
    Blocked,
    /// Deliberately not done, with a reason.
    Rejected,
}

impl NeedStatus {
    /// Stable slug used in the Links Notation trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Satisfied => "satisfied",
            Self::Deferred => "deferred",
            Self::Blocked => "blocked",
            Self::Rejected => "rejected",
        }
    }
}

/// A single question, requirement, task, or statement detected inside a prompt.
///
/// A `Need` reuses the canonical [`IntentKind`] classification rather than
/// introducing a parallel taxonomy, so a need is always one of the intent kinds
/// the engine already understands. The `source_span` is the exact substring of
/// the prompt the need was detected in, preserving provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    /// Content-addressed identifier, stable for a given frame and span.
    pub need_id: String,
    /// The substring of the prompt this need was detected in.
    pub source_span: String,
    /// The canonical intent kind classifying this need.
    pub kind: IntentKind,
    /// The routing slug the span formalizes to, when one is recognized.
    pub route: Option<String>,
    /// Lifecycle status (always [`NeedStatus::Pending`] in Phase 1A).
    pub status: NeedStatus,
}

impl Need {
    #[must_use]
    fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "problem_need".to_owned()),
            ("need_id", self.need_id.clone()),
            ("source_span", self.source_span.clone()),
            ("kind", self.kind.slug().to_owned()),
            ("status", self.status.slug().to_owned()),
        ];
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        format_lino_record(&self.need_id, &pairs)
    }
}

/// The explicit meaning record for one prompt: the formalized impulse plus every
/// detected [`Need`].
///
/// A frame is a projection of the existing [`IntentFormalization`]; it never
/// changes routing in Phase 1A. It exists so the solver loop can emit a single,
/// link-serializable object that names what the user asked for in full, which is
/// the foundation the recursive core and the need ledger build on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemFrame {
    /// Content-addressed identifier for the frame.
    pub frame_id: String,
    /// The impulse this frame formalizes.
    pub impulse_id: String,
    /// Detected language slug.
    pub language: String,
    /// The whole-prompt intent kind (the frame's primary classification).
    pub kind: IntentKind,
    /// The whole-prompt routing slug, when recognized.
    pub route: Option<String>,
    /// Every need detected in the prompt, in source order.
    pub needs: Vec<Need>,
}

impl ProblemFrame {
    /// Build a frame from an already-computed [`IntentFormalization`].
    ///
    /// This reuses the canonical formalization for the whole-prompt
    /// classification and, when the prompt contains more than one segment,
    /// re-runs the same [`formalize_intent`] classifier on each segment so a
    /// multi-part prompt yields multiple typed needs (R7). When the prompt is a
    /// single segment, the frame carries exactly one need that mirrors the
    /// frame's own classification, so single-intent prompts behave identically
    /// to today (R13).
    #[must_use]
    pub fn from_formalization(formalization: &IntentFormalization) -> Self {
        let frame_id = stable_id("problem_frame", &formalization.impulse_id);
        let segments = segment_needs(&formalization.source_text);
        let needs = if segments.len() <= 1 {
            vec![Need {
                need_id: need_id_for(&frame_id, 0, &formalization.source_text),
                source_span: formalization.source_text.clone(),
                kind: formalization.kind,
                route: formalization.route.clone(),
                status: NeedStatus::Pending,
            }]
        } else {
            segments
                .iter()
                .enumerate()
                .map(|(index, span)| {
                    let segment = formalize_span(span, &formalization.language);
                    Need {
                        need_id: need_id_for(&frame_id, index, span),
                        source_span: span.clone(),
                        kind: segment.kind,
                        route: segment.route,
                        status: NeedStatus::Pending,
                    }
                })
                .collect()
        };

        Self {
            frame_id,
            impulse_id: formalization.impulse_id.clone(),
            language: formalization.language.clone(),
            kind: formalization.kind,
            route: formalization.route.clone(),
            needs,
        }
    }

    /// Number of detected needs.
    #[must_use]
    pub const fn need_count(&self) -> usize {
        self.needs.len()
    }

    /// Render the frame and its needs as Links Notation records.
    ///
    /// The frame record references each need by id; every need is its own
    /// record, mirroring the seed's one-record-per-concept convention.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "problem_frame".to_owned()),
            ("frame_id", self.frame_id.clone()),
            ("impulse_id", self.impulse_id.clone()),
            ("language", self.language.clone()),
            ("kind", self.kind.slug().to_owned()),
            ("need_count", self.needs.len().to_string()),
        ];
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        for need in &self.needs {
            pairs.push(("need", need.need_id.clone()));
        }
        let mut out = format_lino_record(&self.frame_id, &pairs);
        for need in &self.needs {
            out.push('\n');
            out.push_str(&need.to_links_notation());
        }
        out
    }
}

#[must_use]
fn need_id_for(frame_id: &str, index: usize, span: &str) -> String {
    stable_id("problem_need", &format!("{frame_id}:{index}:{span}"))
}

/// Why a [`WorkUnit`] became a recursion leaf (or did not).
///
/// This mirrors the atomicity predicate in
/// `docs/case-studies/issue-559/recursive-core.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicityReason {
    /// The span maps to a recognized route, so a single existing method solves
    /// it directly through the registry-backed method dispatcher.
    DirectMethod,
    /// The span cannot be decomposed further and has no recognized route; it is
    /// an irreducible single need.
    SingleNeed,
    /// `depth >= max_decomposition_depth`: the unit is forced to a leaf so the
    /// recursion is always bounded (`NON-GOALS.md`).
    DepthBound,
    /// The unit was decomposed into children (it is not a leaf).
    NotAtomic,
}

impl AtomicityReason {
    /// Stable slug used in the Links Notation trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::DirectMethod => "direct_method",
            Self::SingleNeed => "single_need",
            Self::DepthBound => "depth_bound",
            Self::NotAtomic => "not_atomic",
        }
    }
}

/// A node in the recursive, bidirectional decomposition of a [`ProblemFrame`].
///
/// A `WorkUnit` is the recursively-formalized sub-impulse from
/// `docs/case-studies/issue-559/alignment.md`. Phase 1B builds the *downward*
/// (decomposition) pass only, bounded by `max_decomposition_depth` and the
/// atomicity base case, and records it as trace. It does not change routing or
/// the answer (R13); the existing dispatch still resolves every leaf. Later
/// phases add the upward (construction) pass, candidate/selection links, and the
/// validation/composition fields described in the recursive-core spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkUnit {
    /// Content-addressed identifier, stable for a given parent, depth, and span.
    pub unit_id: String,
    /// The unit or frame that produced this one (`None` for the root).
    pub parent: Option<String>,
    /// The substring of the prompt this unit covers.
    pub source_span: String,
    /// Recursion depth (0 at the root).
    pub depth: u8,
    /// Whether this unit is a recursion leaf.
    pub atomic: bool,
    /// Why the unit is (or is not) atomic.
    pub reason: AtomicityReason,
    /// The routing slug the span formalizes to, when one is recognized.
    pub route: Option<String>,
    /// Sub-units produced by the downward pass (empty when atomic).
    pub children: Vec<Self>,
}

impl WorkUnit {
    /// Build the root unit and its bounded recursive decomposition from the
    /// already-computed [`IntentFormalization`].
    #[must_use]
    pub fn from_formalization(formalization: &IntentFormalization, max_depth: u8) -> Self {
        Self::build(
            &formalization.source_text,
            None,
            &formalization.language,
            0,
            max_depth,
        )
    }

    /// Recursively build one unit, splitting only while it is non-atomic and the
    /// depth bound has not been reached.
    fn build(span: &str, parent: Option<String>, language: &str, depth: u8, max_depth: u8) -> Self {
        let unit_id = stable_id("work_unit", &format!("{parent:?}:{depth}:{span}"));
        let route = formalize_span(span, language).route;

        // Depth bound first: a forced leaf is always bounded.
        if depth >= max_depth {
            return Self::leaf(
                unit_id,
                parent,
                span,
                depth,
                AtomicityReason::DepthBound,
                route,
            );
        }

        let segments = decompose_once(span);
        if segments.len() <= 1 {
            // Cannot split further: a direct method match is the leaf reason when
            // a route is recognized, otherwise it is an irreducible single need.
            let reason = if route.is_some() {
                AtomicityReason::DirectMethod
            } else {
                AtomicityReason::SingleNeed
            };
            return Self::leaf(unit_id, parent, span, depth, reason, route);
        }

        let children = segments
            .iter()
            .map(|child_span| {
                Self::build(
                    child_span,
                    Some(unit_id.clone()),
                    language,
                    depth + 1,
                    max_depth,
                )
            })
            .collect();
        Self {
            unit_id,
            parent,
            source_span: span.to_owned(),
            depth,
            atomic: false,
            reason: AtomicityReason::NotAtomic,
            route,
            children,
        }
    }

    fn leaf(
        unit_id: String,
        parent: Option<String>,
        span: &str,
        depth: u8,
        reason: AtomicityReason,
        route: Option<String>,
    ) -> Self {
        Self {
            unit_id,
            parent,
            source_span: span.to_owned(),
            depth,
            atomic: true,
            reason,
            route,
            children: Vec::new(),
        }
    }

    /// Total number of units in this subtree (this unit plus its descendants).
    #[must_use]
    pub fn unit_count(&self) -> usize {
        1 + self.children.iter().map(Self::unit_count).sum::<usize>()
    }

    /// Number of recursion leaves (atomic units) in this subtree.
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        if self.atomic {
            1
        } else {
            self.children.iter().map(Self::leaf_count).sum()
        }
    }

    /// Collect references to every recursion leaf (atomic unit) in this subtree,
    /// in source order.
    pub fn collect_leaves<'a>(&'a self, out: &mut Vec<&'a Self>) {
        if self.atomic {
            out.push(self);
        } else {
            for child in &self.children {
                child.collect_leaves(out);
            }
        }
    }

    /// Render this unit and its descendants as Links Notation records.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "work_unit".to_owned()),
            ("unit_id", self.unit_id.clone()),
            ("source_span", self.source_span.clone()),
            ("depth", self.depth.to_string()),
            ("atomic", self.atomic.to_string()),
            ("atomicity_reason", self.reason.slug().to_owned()),
        ];
        if let Some(parent) = &self.parent {
            pairs.push(("parent", parent.clone()));
        }
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        for child in &self.children {
            pairs.push(("child", child.unit_id.clone()));
        }
        let mut out = format_lino_record(&self.unit_id, &pairs);
        for child in &self.children {
            out.push('\n');
            out.push_str(&child.to_links_notation());
        }
        out
    }

    /// Emit the `work_unit:enter` / `work_unit:exit` loop events in pre-order so
    /// the downward pass is observable in the event log (mirrors the
    /// `solve_unit` pseudo-code in the recursive-core spec).
    fn emit_events(&self, log: &mut EventLog) {
        log.append(
            "work_unit:enter",
            format!("{} {}", self.depth, self.source_span),
        );
        for child in &self.children {
            child.emit_events(log);
        }
        log.append(
            "work_unit:exit",
            format!("{} {}", self.depth, self.reason.slug()),
        );
    }
}

/// Split a span one level, preferring sentence boundaries over clause boundaries
/// so the work-unit tree is hierarchical (sentences above clauses).
///
/// Returns the single span unchanged when it cannot be split further, which is
/// the recursion base case in [`WorkUnit::build`].
#[must_use]
fn decompose_once(span: &str) -> Vec<String> {
    let sentences = split_sentences(span);
    if sentences.len() > 1 {
        return sentences;
    }
    split_clauses(span)
}

/// Split a prompt into the spans that each carry one need.
///
/// Segmentation is deliberately structural and language-neutral: it splits on
/// sentence terminators (`?`, `!`, `.`, and their CJK forms) and then on the
/// same coordinating triggers the existing decomposition step uses
/// (`record_decomposition` in `src/solver_helpers.rs`). It introduces no new
/// per-language phrase list, so it does not regress the no-hardcoded-natural-
/// language discipline. A period flanked by non-whitespace (e.g. `3.14`) is kept
/// inside its segment so decimals are never broken apart.
#[must_use]
fn segment_needs(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    for sentence in split_sentences(text) {
        for clause in split_clauses(&sentence) {
            let trimmed = clause.trim();
            if !trimmed.is_empty() {
                segments.push(trimmed.to_owned());
            }
        }
    }
    segments
}

/// Split on sentence terminators, keeping the terminator attached so a trailing
/// `?` is still visible to the question classifier.
#[must_use]
fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut current = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        current.push(ch);
        let strong_terminator = matches!(ch, '?' | '!' | '。' | '！' | '？');
        let period_boundary =
            ch == '.' && chars.get(index + 1).is_none_or(|next| next.is_whitespace());
        if strong_terminator || period_boundary {
            push_trimmed(&mut sentences, &current);
            current.clear();
        }
    }
    push_trimmed(&mut sentences, &current);
    sentences
}

/// Split a sentence on coordinating triggers, mirroring the existing shallow
/// decomposition so the frame agrees with `record_decomposition`.
#[must_use]
fn split_clauses(sentence: &str) -> Vec<String> {
    sentence
        .split([',', ';'])
        .flat_map(|chunk| chunk.split(" and "))
        .flat_map(|chunk| chunk.split(" with "))
        .map(|chunk| chunk.trim().to_owned())
        .filter(|chunk| !chunk.is_empty())
        .collect()
}

fn push_trimmed(out: &mut Vec<String>, candidate: &str) {
    let trimmed = candidate.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_owned());
    }
}

/// Build a [`ProblemFrame`] from the formalization and emit it as a loop event
/// plus its Links Notation trace.
///
/// This is trace-only: it appends a `problem_frame` event (the serialized frame)
/// and per-need `problem_frame:need` events, so the meaning record is observable
/// in the event log without altering routing or the final answer (R13).
pub(crate) fn record_problem_frame(
    log: &mut EventLog,
    formalization: &IntentFormalization,
) -> ProblemFrame {
    let frame = ProblemFrame::from_formalization(formalization);
    log.append("problem_frame", frame.to_links_notation());
    log.append("problem_frame:need_count", frame.needs.len().to_string());
    for need in &frame.needs {
        log.append(
            "problem_frame:need",
            format!("{} {}", need.kind.slug(), need.source_span),
        );
    }
    frame
}

/// Build the recursive work-unit tree for the formalized prompt and emit it as
/// loop events plus its Links Notation trace.
///
/// This is the downward (decomposition) pass of the recursive core, recorded as
/// trace only (Phase 1B): it appends a `work_unit` record (the serialized tree),
/// a `work_unit:count`, a `work_unit:leaf_count`, and the per-unit
/// `work_unit:enter` / `work_unit:exit` events. Routing and the answer are
/// unchanged — every leaf is still resolved by the existing dispatch (R13).
pub(crate) fn record_work_units(
    log: &mut EventLog,
    formalization: &IntentFormalization,
    max_depth: u8,
) -> WorkUnit {
    let root = WorkUnit::from_formalization(formalization, max_depth);
    log.append("work_unit", root.to_links_notation());
    log.append("work_unit:count", root.unit_count().to_string());
    log.append("work_unit:leaf_count", root.leaf_count().to_string());
    root.emit_events(log);
    root
}

/// One row of the need-satisfaction ledger: a detected need plus the status the
/// recursive core assigns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerRow {
    /// The need this row resolves.
    pub need_id: String,
    /// The need's source span (for provenance in the trace).
    pub source_span: String,
    /// The resolved lifecycle status.
    pub status: NeedStatus,
    /// The atomicity reason of the leaf that resolved this need, when matched.
    pub leaf_reason: Option<AtomicityReason>,
    /// The id of the work-unit leaf that resolves this need, when matched — the
    /// link that connects the ledger row back to the decomposition (R334).
    pub unit_id: Option<String>,
    /// The route the resolving leaf carries, when matched — the method the leaf
    /// is expected to dispatch to.
    pub route: Option<String>,
}

impl LedgerRow {
    #[must_use]
    fn to_links_notation(&self) -> String {
        let row_id = stable_id(
            "need_ledger_row",
            &format!("{}:{}", self.need_id, self.status.slug()),
        );
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "need_ledger_row".to_owned()),
            ("need_id", self.need_id.clone()),
            ("source_span", self.source_span.clone()),
            ("status", self.status.slug().to_owned()),
        ];
        if let Some(reason) = self.leaf_reason {
            pairs.push(("leaf_reason", reason.slug().to_owned()));
        }
        if let Some(unit_id) = &self.unit_id {
            pairs.push(("unit_id", unit_id.clone()));
        }
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        format_lino_record(&row_id, &pairs)
    }
}

/// The need-satisfaction ledger for one [`ProblemFrame`] (root requirement R333).
///
/// The ledger makes "address every detected need" (R8) structural rather than
/// prose: every need in the frame appears exactly once with an explicit status,
/// so a blocked or deferred need is recorded, never silently dropped. Phase 2
/// derives the status from the recursive work-unit tree — a need whose leaf maps
/// to a direct method is [`NeedStatus::Satisfied`] (a known method resolves it);
/// a need with no recognized method (a single-need or depth-bound leaf) is
/// [`NeedStatus::Blocked`] and reported as such. This is a behavior-preserving
/// projection: it changes neither routing nor the answer. Runtime validation
/// feedback can refine these predictions if a future validator proves a leaf
/// failed after method selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeedLedger {
    /// The frame this ledger resolves.
    pub frame_id: String,
    /// One row per detected need, in frame order.
    pub rows: Vec<LedgerRow>,
}

impl NeedLedger {
    /// Resolve the ledger for a frame against its recursive work-unit tree.
    #[must_use]
    pub fn resolve(frame: &ProblemFrame, root: &WorkUnit) -> Self {
        let mut leaves = Vec::new();
        root.collect_leaves(&mut leaves);
        let rows = frame
            .needs
            .iter()
            .map(|need| {
                let leaf = best_leaf_for(&leaves, &need.source_span);
                let status = leaf.map_or(NeedStatus::Blocked, |leaf| {
                    if leaf.route.is_some() {
                        NeedStatus::Satisfied
                    } else {
                        NeedStatus::Blocked
                    }
                });
                LedgerRow {
                    need_id: need.need_id.clone(),
                    source_span: need.source_span.clone(),
                    status,
                    leaf_reason: leaf.map(|leaf| leaf.reason),
                    unit_id: leaf.map(|leaf| leaf.unit_id.clone()),
                    route: leaf.and_then(|leaf| leaf.route.clone()),
                }
            })
            .collect();
        Self {
            frame_id: frame.frame_id.clone(),
            rows,
        }
    }

    /// Number of rows with the given status.
    #[must_use]
    pub fn count_with(&self, status: NeedStatus) -> usize {
        self.rows.iter().filter(|row| row.status == status).count()
    }

    /// Whether every detected need has an explicit, non-pending status — the
    /// structural form of "address every detected need" (R8).
    #[must_use]
    pub fn every_need_accounted_for(&self) -> bool {
        !self.rows.is_empty()
            && self
                .rows
                .iter()
                .all(|row| row.status != NeedStatus::Pending)
    }

    /// Render the ledger and its rows as Links Notation records.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let ledger_id = stable_id("need_ledger", &self.frame_id);
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "need_ledger".to_owned()),
            ("frame_id", self.frame_id.clone()),
            ("row_count", self.rows.len().to_string()),
            (
                "satisfied",
                self.count_with(NeedStatus::Satisfied).to_string(),
            ),
            ("blocked", self.count_with(NeedStatus::Blocked).to_string()),
        ];
        for row in &self.rows {
            pairs.push(("row", row.need_id.clone()));
        }
        let mut out = format_lino_record(&ledger_id, &pairs);
        for row in &self.rows {
            out.push('\n');
            out.push_str(&row.to_links_notation());
        }
        out
    }
}

/// Pick the work-unit leaf that best matches a need's span: an exact span match
/// first, then the longest leaf span contained in the need (or containing it),
/// so a need is mapped to the most specific method that covers it.
#[must_use]
fn best_leaf_for<'a>(leaves: &'a [&'a WorkUnit], span: &str) -> Option<&'a WorkUnit> {
    if let Some(exact) = leaves.iter().find(|leaf| leaf.source_span == span) {
        return Some(exact);
    }
    leaves
        .iter()
        .filter(|leaf| span.contains(leaf.source_span.as_str()) || leaf.source_span.contains(span))
        .max_by_key(|leaf| leaf.source_span.len())
        .copied()
}

/// Resolve and emit the need-satisfaction ledger as a loop event plus its Links
/// Notation trace (Phase 2). Trace-only: routing and the answer are unchanged.
pub(crate) fn record_need_ledger(
    log: &mut EventLog,
    frame: &ProblemFrame,
    root: &WorkUnit,
) -> NeedLedger {
    let ledger = NeedLedger::resolve(frame, root);
    log.append("need_ledger", ledger.to_links_notation());
    log.append(
        "need_ledger:accounted_for",
        ledger.every_need_accounted_for().to_string(),
    );
    for row in &ledger.rows {
        log.append(
            "need:status",
            format!("{} {}", row.status.slug(), row.source_span),
        );
    }
    ledger
}
