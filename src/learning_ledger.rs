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
use crate::link_store::LinkStore;
use crate::memory::{MemoryEvent, MemoryStore};
use crate::self_healing::{RepairCase, RepairOutcome};

const APPROVED_LESSONS_LINO: &str = include_str!("../data/seed/approved-lessons.lino");

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

    /// Load approved lessons from their seed through the canonical link-store
    /// projection.
    ///
    /// The seed is data, not a constructor for the one historical case: adding
    /// a second reviewed lesson is therefore a seed append. Each row first
    /// becomes a [`MemoryEvent`] and is appended through [`LinkStore`]; the
    /// resulting ledger is then projected from the stored events. This keeps
    /// the portable Links Notation and the associative store on one read path.
    ///
    /// # Errors
    ///
    /// Returns a stable error slug when the document has the wrong root, a
    /// lesson is incomplete, a count is not numeric, or the link projection
    /// loses a row.
    pub fn from_approved_lessons_seed(seed: &str) -> Result<Self, String> {
        let document = crate::seed::parser::parse_lino(seed);
        let root = document
            .children
            .iter()
            .find(|node| node.name == "approved_lessons")
            .ok_or_else(|| String::from("approved_lessons_root_missing"))?;
        let mut store = MemoryStore::new();
        for lesson in root.children.iter().filter(|node| node.name == "lesson") {
            let value = |name: &str| -> Result<String, String> {
                let value = lesson.find_child_value(name).trim();
                if value.is_empty() {
                    Err(format!("approved_lesson_{name}_missing:{}", lesson.id))
                } else {
                    Ok(value.to_owned())
                }
            };
            let benchmark_passed = value("benchmark_passed")?;
            benchmark_passed
                .parse::<usize>()
                .map_err(|_| format!("approved_lesson_benchmark_passed_invalid:{}", lesson.id))?;
            let event = MemoryEvent {
                id: lesson.id.clone(),
                kind: Some(String::from("approved_lesson")),
                role: Some(String::from("system")),
                intent: Some(String::from("repair_lesson")),
                tool: Some(value("module_path")?),
                inputs: Some(value("failure_prompt")?),
                outputs: Some(value("resolved_task")?),
                content: Some(value("rule_id")?),
                demo_label: Some(value("modifier")?),
                conversation_id: Some(value("case_id")?),
                conversation_title: Some(value("benchmark_suite")?),
                evidence: vec![
                    format!("benchmark_passed={benchmark_passed}"),
                    format!("reviewer={}", value("reviewer")?),
                ],
                ..MemoryEvent::default()
            };
            store
                .append_memory_event(event)
                .map_err(|error| format!("approved_lesson_link_store:{error}"))?;
        }
        let records = store.records();
        if records.len() != store.events().len() {
            return Err(String::from("approved_lesson_projection_incomplete"));
        }
        let entries = store
            .events()
            .iter()
            .map(ledger_entry_from_event)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { entries })
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

/// Load the canonical, fully-worked ledger from the approved-lessons seed.
///
/// The seed contains the exact entry the former hard-coded constructor derived,
/// so the committed `data/meta/learning-ledger.lino` remains byte-identical.
/// New reviewed lessons are data edits and pass through the same [`LinkStore`]
/// projection instead of requiring a new Rust branch.
#[must_use]
pub fn canonical_ledger() -> LearningLedger {
    LearningLedger::from_approved_lessons_seed(APPROVED_LESSONS_LINO)
        .expect("the embedded approved-lessons seed must be valid")
}

/// The failure prompts [`canonical_ledger`] can possibly answer, read straight
/// off the canonical failure trace the ledger is promoted from.
///
/// Issue #1017: building the ledger runs the whole self-healing pass, and that
/// pass round-trips the pinned module's entire CST/AST — a one-time cost of over
/// ten seconds on a `dev` build, which is what CI runs. That work is only ever
/// *useful* for a prompt the ledger actually holds a lesson for, and which
/// prompts those are is decided by the trace, not by the round-trip. Deriving
/// the answerable set from the same trace lets [`approved_lesson_for`] prove a
/// miss before paying for the parse, without weakening a single promotion gate:
/// the ledger is still built, still gated, and still consulted for a hit.
///
/// `issue_1017_ledger_recall.rs` asserts this set equals the failure prompts of
/// the ledger that is actually built, so the two cannot drift apart.
#[must_use]
pub fn canonical_ledger_failure_prompts() -> Vec<String> {
    canonical_ledger()
        .entries()
        .iter()
        .map(|entry| entry.failure_prompt.clone())
        .collect()
}

/// Recall an approved, committed lesson for the live solver path.
///
/// Returning an owned entry keeps the runtime caller independent of the
/// ledger's storage lifetime. Only the canonical human-approved ledger is
/// consulted; proposed or merely uploaded lessons never reach this path.
#[must_use]
pub fn approved_lesson_for(prompt: &str) -> Option<LedgerEntry> {
    static APPROVED_LEDGER: OnceLock<LearningLedger> = OnceLock::new();

    // Cheap miss check first — see `canonical_ledger_failure_prompts`. Matching
    // is on the same normalised form `lesson_for` uses, so the guard admits
    // exactly the prompts the ledger would answer.
    let needle = normalise(prompt);
    if !canonical_ledger_failure_prompts()
        .iter()
        .any(|failure_prompt| normalise(failure_prompt) == needle)
    {
        return None;
    }
    APPROVED_LEDGER
        .get_or_init(canonical_ledger)
        .lesson_for(prompt)
        .cloned()
}

fn normalise(prompt: &str) -> String {
    prompt.trim().to_lowercase()
}

fn ledger_entry_from_event(event: &MemoryEvent) -> Result<LedgerEntry, String> {
    let required = |name: &str, value: Option<&str>| -> Result<String, String> {
        value
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| format!("approved_lesson_{name}_missing:{}", event.id))
    };
    let benchmark_passed = evidence_value(event, "benchmark_passed")?
        .parse::<usize>()
        .map_err(|_| format!("approved_lesson_benchmark_passed_invalid:{}", event.id))?;
    Ok(LedgerEntry {
        lesson_id: required("lesson_id", Some(event.id.as_str()))?,
        case_id: required("case_id", event.conversation_id.as_deref())?,
        failure_prompt: required("failure_prompt", event.inputs.as_deref())?,
        module_path: required("module_path", event.tool.as_deref())?,
        rule_id: required("rule_id", event.content.as_deref())?,
        resolved_task: required("resolved_task", event.outputs.as_deref())?,
        modifier: required("modifier", event.demo_label.as_deref())?,
        benchmark_suite: required("benchmark_suite", event.conversation_title.as_deref())?,
        benchmark_passed,
        reviewer: evidence_value(event, "reviewer")?,
    })
}

fn evidence_value(event: &MemoryEvent, key: &str) -> Result<String, String> {
    let prefix = format!("{key}=");
    event
        .evidence
        .iter()
        .find_map(|value| value.strip_prefix(&prefix))
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("approved_lesson_{key}_missing:{}", event.id))
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
