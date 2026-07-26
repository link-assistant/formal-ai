//! The human-gated promotion ledger for the self-healing loop (issue #558).
//!
//! Issue #558 asks for a self-healing algorithm that *"promotes improvements when
//! tests and the user accept them"* and writes the accepted result *"to mainline
//! history as an approved learning record"*. [`crate::self_healing`] closes the
//! reasoning loop up to a reviewable [`RepairCase`]; this module supplies the
//! **terminal promotion step** the issue requires: a durable, append-only ledger
//! of lessons that were *both* benchmark-green *and* approved by a human.
//!
//! The gate is deliberately strict and models the issue's two acceptance
//! conditions as one operation. A [`RepairCase`] can only be promoted when:
//!
//! * its source ↔ links round-trip is faithful (so an accepted edit could in
//!   principle be recompiled — the "recompile itself" guardrail), and
//! * the benchmark gate passed and a lesson is adoptable
//!   ([`RepairOutcome::AwaitingReview`]) — *"when tests … accept"*, and
//! * a human explicitly approves ([`HumanApproval::is_granted`]) — *"and the
//!   user accept\[s\]"*.
//!
//! Nothing is promoted automatically: [`LearningLedger::promote`] takes an explicit
//! [`HumanApproval`] and refuses every case that is not green *and* approved. Once
//! promoted, [`LearningLedger::lesson_for`] lets the system recognise a *repeated*
//! failure and recall the already-approved lesson instead of re-deriving it — the
//! concrete payoff of "auto learning": a failure seen once and approved is answered
//! from the ledger the next time. Every field is a deterministic function of the
//! repair case and the approval, so the ledger and its content id are reproducible.

use std::{fmt::Write as _, sync::OnceLock};

use crate::engine::stable_id;
use crate::self_healing::{RepairCase, RepairOutcome};

/// An explicit, auditable human decision on a reviewed [`RepairCase`].
///
/// Promotion is *"when tests and the user accept"* — the benchmark gate covers
/// "tests"; this value carries the "user" half. It records who reviewed and whether
/// they granted adoption, so the ledger entry is attributable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanApproval {
    reviewer: String,
    granted: bool,
}

impl HumanApproval {
    /// A granted approval from `reviewer` — the human accepts the lesson.
    #[must_use]
    pub fn granted(reviewer: impl Into<String>) -> Self {
        Self {
            reviewer: reviewer.into(),
            granted: true,
        }
    }

    /// A withheld approval from `reviewer` — the human declines the lesson.
    #[must_use]
    pub fn declined(reviewer: impl Into<String>) -> Self {
        Self {
            reviewer: reviewer.into(),
            granted: false,
        }
    }

    /// Whether the human accepted the lesson.
    #[must_use]
    pub const fn is_granted(&self) -> bool {
        self.granted
    }

    /// Who made the decision.
    #[must_use]
    pub fn reviewer(&self) -> &str {
        &self.reviewer
    }
}

/// Why a [`RepairCase`] could not be promoted into the ledger.
///
/// Every variant is a guardrail: the loop stays proposal-only until *both* the
/// tests and the human accept, and never records a lesson it could not recompile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionRejected {
    /// No lesson could be synthesised from the failure ([`RepairOutcome::NoCandidate`]).
    NoReviewableProposal,
    /// A lesson exists but the benchmark gate did not pass
    /// ([`RepairOutcome::BlockedByBenchmark`]) — "tests" did not accept.
    TestsNotGreen,
    /// The mapped source did not round-trip byte-for-byte, so an accepted edit
    /// could not be recompiled faithfully; adoption is blocked.
    SourceNotFaithful,
    /// The human withheld approval — "the user" did not accept.
    HumanDeclined,
    /// A lesson for this exact failure is already in the ledger; promotion is
    /// idempotent and refuses to record a duplicate.
    AlreadyPromoted,
}

impl PromotionRejected {
    /// A stable, human-readable slug for the rejection reason.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::NoReviewableProposal => "no_reviewable_proposal",
            Self::TestsNotGreen => "tests_not_green",
            Self::SourceNotFaithful => "source_not_faithful",
            Self::HumanDeclined => "human_declined",
            Self::AlreadyPromoted => "already_promoted",
        }
    }
}

/// One promoted lesson — the "approved learning record" issue #558 calls for.
///
/// It flattens the parts of a green, approved [`RepairCase`] a future lookup needs:
/// the failure it answers, the source it maps onto, the adopted rule, and the human
/// who approved it. Deterministic: every field comes from the case and approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    /// Stable content-addressed id of the promoted lesson.
    pub lesson_id: String,
    /// The originating repair-case id (provenance back to the reasoning loop).
    pub case_id: String,
    /// The original input the system could not answer — the lookup key.
    pub failure_prompt: String,
    /// The source module the failure maps onto.
    pub module_path: String,
    /// The adopted learned rule id.
    pub rule_id: String,
    /// The program-plan task the learned rule resolves the failure to.
    pub resolved_task: String,
    /// The modifier that triggers the learned rule.
    pub modifier: String,
    /// The benchmark suite that gated adoption.
    pub benchmark_suite: String,
    /// Passing case count from the gate run that green-lit adoption.
    pub benchmark_passed: usize,
    /// The human who approved promotion.
    pub reviewer: String,
}

impl LedgerEntry {
    /// A one-line human-readable summary of the promoted lesson.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Approved lesson `{}` (rule `{}`) for failure `{}` → `{}`, mapped onto {}, gated by {} passing case(s), approved by {}.",
            self.lesson_id,
            self.rule_id,
            self.failure_prompt,
            self.resolved_task,
            self.module_path,
            self.benchmark_passed,
            self.reviewer,
        )
    }
}

/// The durable, append-only ledger of human-approved lessons.
///
/// Built by promoting green, approved [`RepairCase`]s. Entries are kept in
/// promotion order and the whole ledger serialises to Links Notation with a stable
/// content id, so it can be committed as an auditable artifact and re-derived.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LearningLedger {
    entries: Vec<LedgerEntry>,
}

impl LearningLedger {
    /// An empty ledger.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// The promoted lessons, in promotion order.
    #[must_use]
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// How many lessons have been promoted.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the ledger holds no lessons yet.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Promote `case` into the ledger — the terminal, human-gated step of the loop.
    ///
    /// Succeeds only when the case is benchmark-green with an adoptable lesson, its
    /// source round-trips faithfully, and `approval` is granted. Returns the newly
    /// recorded [`LedgerEntry`], or a [`PromotionRejected`] explaining which gate
    /// stopped it. Idempotent per failure prompt: a second promotion of the same
    /// failure is refused as [`PromotionRejected::AlreadyPromoted`].
    pub fn promote(
        &mut self,
        case: &RepairCase,
        approval: &HumanApproval,
    ) -> Result<&LedgerEntry, PromotionRejected> {
        // Tests-accept gate: only a benchmark-green, adoptable case is reviewable.
        match case.outcome {
            RepairOutcome::AwaitingReview => {}
            RepairOutcome::BlockedByBenchmark => return Err(PromotionRejected::TestsNotGreen),
            RepairOutcome::NoCandidate => return Err(PromotionRejected::NoReviewableProposal),
        }
        // Recompile guardrail: never record a lesson whose source cannot be
        // reconstructed byte-for-byte.
        if !case.source_round_trip.faithful {
            return Err(PromotionRejected::SourceNotFaithful);
        }
        // User-accept gate.
        if !approval.is_granted() {
            return Err(PromotionRejected::HumanDeclined);
        }
        // Idempotency: one approved lesson per distinct failure.
        if self.knows(&case.failure_prompt) {
            return Err(PromotionRejected::AlreadyPromoted);
        }

        // `AwaitingReview` guarantees at least one adoptable rule; take the first.
        let rule = case
            .learning
            .adoptable_rules()
            .into_iter()
            .next()
            .ok_or(PromotionRejected::NoReviewableProposal)?;
        let lesson_id = stable_id(
            "promoted_lesson",
            &format!("{}:{}:{}", case.id, rule.rule_id, approval.reviewer()),
        );
        self.entries.push(LedgerEntry {
            lesson_id,
            case_id: case.id.clone(),
            failure_prompt: case.failure_prompt.clone(),
            module_path: case.source_round_trip.module_path.clone(),
            rule_id: rule.rule_id.clone(),
            resolved_task: rule.resolved_task.clone(),
            modifier: rule.modifier.clone(),
            benchmark_suite: case.learning.gate.suite_id.clone(),
            benchmark_passed: case.learning.gate.passed,
            reviewer: approval.reviewer().to_owned(),
        });
        Ok(self.entries.last().expect("just pushed"))
    }

    /// The approved lesson for a *repeated* failure, if one was promoted.
    ///
    /// This is what makes the loop "auto learning": a failure the system once could
    /// not answer, then learned and had approved, is now recognised and answered
    /// from the ledger without re-deriving it. Matching is on the normalised prompt
    /// (trimmed, case-insensitive) so trivial rephrasings of whitespace/case hit.
    #[must_use]
    pub fn lesson_for(&self, prompt: &str) -> Option<&LedgerEntry> {
        let needle = normalise(prompt);
        self.entries
            .iter()
            .find(|entry| normalise(&entry.failure_prompt) == needle)
    }

    /// Whether the ledger already holds an approved lesson for `prompt`.
    #[must_use]
    pub fn knows(&self, prompt: &str) -> bool {
        self.lesson_for(prompt).is_some()
    }

    /// Render the whole ledger as Links Notation — the committable, auditable
    /// "mainline history" of approved learning records. Ends trimmed.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("learning_ledger\n");
        field(&mut out, "engine", "meta_language");
        field(&mut out, "human_gated", "true");
        let _ = writeln!(out, "  lesson_count \"{}\"", self.entries.len());
        for entry in &self.entries {
            out.push_str("  lesson\n");
            nested(&mut out, "lesson_id", &entry.lesson_id);
            nested(&mut out, "case_id", &entry.case_id);
            nested(&mut out, "failure_prompt", &entry.failure_prompt);
            nested(&mut out, "module_path", &entry.module_path);
            nested(&mut out, "rule_id", &entry.rule_id);
            nested(&mut out, "modifier", &entry.modifier);
            nested(&mut out, "resolved_task", &entry.resolved_task);
            nested(&mut out, "benchmark_suite", &entry.benchmark_suite);
            let _ = writeln!(out, "    benchmark_passed \"{}\"", entry.benchmark_passed);
            nested(&mut out, "reviewer", &entry.reviewer);
        }
        out.trim_end().to_owned()
    }

    /// A stable content id over the ledger's Links Notation — a fingerprint of the
    /// whole approved-lesson history.
    #[must_use]
    pub fn content_id(&self) -> String {
        stable_id("learning_ledger", &self.links_notation())
    }
}

/// Build the canonical, fully-worked ledger: the canonical self-healing case
/// promoted with a granted approval.
///
/// Deterministic and self-contained — used by the committed
/// `data/meta/learning-ledger.lino` artifact, the example, and the tests. Panics
/// only if the canonical case is not promotable, which would itself be a bug.
#[must_use]
pub fn canonical_ledger() -> LearningLedger {
    let mut ledger = LearningLedger::new();
    ledger
        .promote(
            &crate::self_healing::canonical_case(),
            &HumanApproval::granted("maintainer"),
        )
        .expect("the canonical self-healing case is green and approvable");
    ledger
}

/// Recall an approved, committed lesson for the live solver path.
///
/// Returning an owned entry keeps the runtime caller independent of the
/// ledger's storage lifetime. Only the canonical human-approved ledger is
/// consulted; proposed or merely uploaded lessons never reach this path.
#[must_use]
pub fn approved_lesson_for(prompt: &str) -> Option<LedgerEntry> {
    static APPROVED_LEDGER: OnceLock<LearningLedger> = OnceLock::new();
    APPROVED_LEDGER
        .get_or_init(canonical_ledger)
        .lesson_for(prompt)
        .cloned()
}

fn normalise(prompt: &str) -> String {
    prompt.trim().to_lowercase()
}

fn field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} \"{}\"", quote(value));
}

fn nested(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "    {key} \"{}\"", quote(value));
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
