//! Issue #559 (R342): skill accumulation — candidate skills and a curriculum.
//!
//! The meta core already proves, per request, which detected needs it resolved and
//! which it could not (the solution evidence, [`crate::solution_evidence`]). A
//! system that "improves itself" must turn that outcome into *learning* the next
//! request can reuse — the deterministic analog of an agent that keeps a skill
//! library and a curriculum of what it still cannot do (R21). This module records
//! that accumulation as link data, from the evidence the loop already produced:
//!
//! * every need that was **satisfied** by a catalogued method becomes a
//!   [`CandidateSkill`] — a reusable, named capability the solver demonstrably has,
//!   captured with the span that demonstrated it; and
//! * every need that was **blocked** (no method resolved it, or its chain never
//!   connected) becomes a [`CurriculumItem`] — a recorded gap to close, never a
//!   silently dropped failure.
//!
//! Accumulation is **proposal-only and gated**, exactly like the meta
//! self-improvement loop (R340). A candidate skill is born [`SkillStatus::Proposed`]
//! and cannot become [`SkillStatus::Stable`] until its [`PromotionGate`] is
//! satisfied — that is, until tests *and* a benchmark delta vouch for it. At trace
//! time neither exists, so nothing is ever auto-promoted: there is no unreviewed
//! self-modification (C3). The default [`SkillMode::Off`] records nothing, so the
//! trace and the answer are exactly what shipped before this stage existed (R13);
//! [`SkillMode::Accumulate`] emits the ledger as a trace-only `skill_ledger` event.

use crate::engine::stable_id;
use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::meta_frame::NeedStatus;
use crate::solution_evidence::SolutionEvidence;

/// Whether the meta core records the skill-accumulation ledger.
///
/// The ledger is trace-only and proposal-only regardless of the mode; the mode only
/// gates whether it is emitted at all, so the default leaves behaviour untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkillMode {
    /// Default: record nothing, so the trace is exactly the pre-stage trace (R13).
    #[default]
    Off,
    /// Record the candidate skills and curriculum items as a trace-only ledger.
    Accumulate,
}

impl SkillMode {
    /// Whether the ledger should be emitted at all.
    #[must_use]
    pub const fn emits_ledger(self) -> bool {
        matches!(self, Self::Accumulate)
    }

    /// The stable slug used in traces and config parsing.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Accumulate => "accumulate",
        }
    }

    /// Parse a slug back into a mode, accepting the canonical spellings.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.trim().to_ascii_lowercase().as_str() {
            "off" => Some(Self::Off),
            "accumulate" => Some(Self::Accumulate),
            _ => None,
        }
    }
}

/// Lifecycle status of a [`CandidateSkill`].
///
/// A skill begins [`Self::Proposed`] and may only advance to [`Self::Stable`] once
/// its [`PromotionGate`] is satisfied; [`Self::Deprecated`] and [`Self::Retired`]
/// are the reverse transitions for a skill a later policy supersedes. Only
/// `Proposed` is produced at trace time, since the gate is never satisfied without
/// the tests and benchmark deltas that human review supplies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillStatus {
    /// Demonstrated once but not yet vouched for by tests and a benchmark delta.
    Proposed,
    /// Promoted: tests and a benchmark delta confirm the skill is reusable.
    Stable,
    /// Superseded by a better skill; kept for provenance, not selected.
    Deprecated,
    /// Withdrawn entirely; recorded so the history stays auditable.
    Retired,
}

impl SkillStatus {
    /// Stable slug used in the Links Notation trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Stable => "stable",
            Self::Deprecated => "deprecated",
            Self::Retired => "retired",
        }
    }
}

/// The evidence a [`CandidateSkill`] needs before it may be promoted to stable.
///
/// Promotion is deliberately conservative: a skill demonstrated once is not yet
/// reusable. It must be backed by a regression test *and* a benchmark delta proving
/// it helps, mirroring the promotion rule in the Phase 7 plan ("promotion rules
/// require tests + benchmark deltas"). Both flags are `false` at trace time, so
/// [`Self::satisfied`] is always `false` and the candidate stays proposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PromotionGate {
    /// A regression test covers the skill.
    pub has_tests: bool,
    /// A benchmark delta shows the skill helps rather than regresses.
    pub has_benchmark_delta: bool,
}

impl PromotionGate {
    /// Whether the skill may be promoted: it needs both tests and a benchmark delta.
    #[must_use]
    pub const fn satisfied(self) -> bool {
        self.has_tests && self.has_benchmark_delta
    }

    /// The status this gate implies: [`SkillStatus::Stable`] only once satisfied,
    /// otherwise [`SkillStatus::Proposed`].
    #[must_use]
    const fn status(self) -> SkillStatus {
        if self.satisfied() {
            SkillStatus::Stable
        } else {
            SkillStatus::Proposed
        }
    }
}

/// A reusable capability the solver demonstrably has, distilled from a satisfied
/// need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSkill {
    /// Content-addressed identifier, stable for a given method and demonstrating span.
    pub skill_id: String,
    /// The catalogued method this skill wraps — the reusable handler.
    pub method: String,
    /// The route the demonstrating leaf carried, when one was recognized.
    pub route: Option<String>,
    /// The prompt span that demonstrated this skill (provenance).
    pub source_span: String,
    /// The work-unit leaf that demonstrated it, linking back to the decomposition.
    pub work_unit_id: Option<String>,
    /// The evidence required before this skill may be promoted (never met at trace
    /// time, so the skill stays proposed).
    pub gate: PromotionGate,
    /// The lifecycle status implied by the gate.
    pub status: SkillStatus,
}

impl CandidateSkill {
    /// Whether this skill may be promoted to stable (always `false` at trace time).
    #[must_use]
    pub const fn promotable(&self) -> bool {
        self.gate.satisfied()
    }

    #[must_use]
    fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "candidate_skill".to_owned()),
            ("skill_id", self.skill_id.clone()),
            ("method", self.method.clone()),
            ("source_span", self.source_span.clone()),
            ("status", self.status.slug().to_owned()),
            ("has_tests", self.gate.has_tests.to_string()),
            (
                "has_benchmark_delta",
                self.gate.has_benchmark_delta.to_string(),
            ),
            ("promotable", self.promotable().to_string()),
        ];
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        if let Some(unit_id) = &self.work_unit_id {
            pairs.push(("work_unit", unit_id.clone()));
        }
        format_lino_record(&self.skill_id, &pairs)
    }
}

/// A recorded gap: a need the solver could not resolve, kept as a thing to learn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurriculumItem {
    /// Content-addressed identifier, stable for a given need.
    pub item_id: String,
    /// The need this gap was detected for.
    pub need_id: String,
    /// The prompt span the unresolved need was detected in (provenance).
    pub source_span: String,
    /// The need's resolved status (e.g. blocked) — why it is a curriculum item.
    pub status: NeedStatus,
    /// A human-readable reason the need was not satisfied.
    pub reason: String,
}

impl CurriculumItem {
    #[must_use]
    fn to_links_notation(&self) -> String {
        format_lino_record(
            &self.item_id,
            &[
                ("record_type", "curriculum_item".to_owned()),
                ("item_id", self.item_id.clone()),
                ("need_id", self.need_id.clone()),
                ("source_span", self.source_span.clone()),
                ("status", self.status.slug().to_owned()),
                ("reason", self.reason.clone()),
            ],
        )
    }
}

/// The skill-accumulation ledger for one request: the skills it demonstrated and
/// the curriculum of gaps it could not yet close.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillLedger {
    /// The frame this ledger accumulates from.
    pub frame_id: String,
    /// One candidate skill per satisfied, method-resolved need, in frame order.
    pub skills: Vec<CandidateSkill>,
    /// One curriculum item per unresolved need, in frame order.
    pub curriculum: Vec<CurriculumItem>,
}

impl SkillLedger {
    /// Accumulate the ledger from the request's solution evidence.
    ///
    /// A trail that is connected end to end, satisfied, and resolved to a catalogued
    /// method becomes a proposed [`CandidateSkill`]; every other trail becomes a
    /// [`CurriculumItem`] recording the gap. This is a pure projection of the
    /// evidence — it changes neither routing nor the answer (R13).
    #[must_use]
    pub fn from_evidence(evidence: &SolutionEvidence) -> Self {
        let mut skills = Vec::new();
        let mut curriculum = Vec::new();
        for trail in &evidence.trails {
            let demonstrated = trail.connected && trail.status == NeedStatus::Satisfied;
            if let (true, Some(method)) = (demonstrated, &trail.method) {
                let gate = PromotionGate::default();
                skills.push(CandidateSkill {
                    skill_id: stable_id(
                        "candidate_skill",
                        &format!("{method}:{}", trail.source_span),
                    ),
                    method: method.clone(),
                    route: trail.route.clone(),
                    source_span: trail.source_span.clone(),
                    work_unit_id: trail.work_unit_id.clone(),
                    gate,
                    status: gate.status(),
                });
            } else {
                let reason = curriculum_reason(trail.method.is_some(), trail.connected);
                curriculum.push(CurriculumItem {
                    item_id: stable_id("curriculum_item", &trail.need_id),
                    need_id: trail.need_id.clone(),
                    source_span: trail.source_span.clone(),
                    status: trail.status,
                    reason,
                });
            }
        }
        Self {
            frame_id: evidence.frame_id.clone(),
            skills,
            curriculum,
        }
    }

    /// Number of proposed candidate skills.
    #[must_use]
    pub fn proposed_count(&self) -> usize {
        self.count_with(SkillStatus::Proposed)
    }

    /// Number of candidate skills already promoted to stable (always 0 at trace
    /// time, since the promotion gate is never satisfied without review).
    #[must_use]
    pub fn stable_count(&self) -> usize {
        self.count_with(SkillStatus::Stable)
    }

    /// Number of recorded curriculum items (unresolved needs).
    #[must_use]
    pub const fn curriculum_count(&self) -> usize {
        self.curriculum.len()
    }

    /// Number of candidate skills that may currently be promoted (always 0 at trace
    /// time): the structural proof that nothing is auto-promoted without review.
    #[must_use]
    pub fn promotable_count(&self) -> usize {
        self.skills
            .iter()
            .filter(|skill| skill.promotable())
            .count()
    }

    fn count_with(&self, status: SkillStatus) -> usize {
        self.skills
            .iter()
            .filter(|skill| skill.status == status)
            .count()
    }

    /// Render the ledger as a `skill_ledger` header plus one record per skill and
    /// curriculum item.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let ledger_id = stable_id("skill_ledger", &self.frame_id);
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "skill_ledger".to_owned()),
            ("frame_id", self.frame_id.clone()),
            ("skill_count", self.skills.len().to_string()),
            ("proposed", self.proposed_count().to_string()),
            ("stable", self.stable_count().to_string()),
            ("promotable", self.promotable_count().to_string()),
            ("curriculum_count", self.curriculum_count().to_string()),
        ];
        for skill in &self.skills {
            pairs.push(("skill", skill.skill_id.clone()));
        }
        for item in &self.curriculum {
            pairs.push(("curriculum", item.item_id.clone()));
        }
        let mut out = format_lino_record(&ledger_id, &pairs);
        for skill in &self.skills {
            out.push('\n');
            out.push_str(&skill.to_links_notation());
        }
        for item in &self.curriculum {
            out.push('\n');
            out.push_str(&item.to_links_notation());
        }
        out
    }
}

/// Explain why a trail became a curriculum item rather than a skill.
fn curriculum_reason(has_method: bool, connected: bool) -> String {
    match (has_method, connected) {
        (false, _) => {
            "No catalogued method resolves this need; recorded as a gap to close.".to_owned()
        }
        (true, false) => "A method exists but the need's chain did not connect end to end; \
             recorded as a gap to close."
            .to_owned(),
        (true, true) => "The need resolved to a method but was not satisfied; recorded as a gap \
             to close."
            .to_owned(),
    }
}

/// Accumulate and emit the skill-accumulation ledger as a trace-only event, gated
/// by `mode`.
///
/// Returns `None` when `mode` is [`SkillMode::Off`] (the default), so the trace is
/// exactly what shipped before this stage existed (R13). When emitted it appends one
/// `skill_ledger` event (the serialized ledger) and a compact
/// `skill_ledger:promotable` count, which is always `0` — the auditable proof that
/// no skill is auto-promoted without review (C3).
pub(crate) fn record_skill_ledger(
    log: &mut EventLog,
    evidence: &SolutionEvidence,
    mode: SkillMode,
) -> Option<SkillLedger> {
    if !mode.emits_ledger() {
        return None;
    }
    let ledger = SkillLedger::from_evidence(evidence);
    log.append("skill_ledger", ledger.to_links_notation());
    log.append(
        "skill_ledger:promotable",
        ledger.promotable_count().to_string(),
    );
    Some(ledger)
}
