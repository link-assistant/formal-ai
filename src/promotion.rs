//! Issue #656 (E37): the benchmark-gated promotion protocol.
//!
//! Every self-improvement loop in the codebase is proposal-only by design:
//! [`crate::self_improvement`] proposes seed rules but never writes `data/seed/`,
//! [`crate::meta_self_improvement`] defaults to `Off`, `src/self_healing.rs`
//! produces human-gated repair cases, and dreaming amendments live only in memory
//! events. This module adds the missing, deterministic **promotion** step: a
//! proposal that passes its benchmark ratchets may be *promoted* into seed data,
//! but only ever as a `.lino` seed edit written onto a branch — never a direct
//! push, and always behind the same `--apply --confirm` gate the dreaming planner
//! uses. Draft pull requests and human review stay the outer gate.
//!
//! The protocol is expressed as an append-only chain of [`MemoryEvent`]s so it
//! round-trips through the bundle export/import path exactly like every other
//! learned artifact:
//!
//! ```text
//! promotion_proposal   proposal link -> which seed file it edits
//!   promotion_evidence which ratchet ran, at what floor, cleared or not
//!   promotion_decision promoted | rejected
//!     promotion_applied     (only when promoted) the materialized seed edit
//!     promotion_rejection   (only when rejected) the change kept but NOT applied
//! ```
//!
//! Rejected proposals are preserved with their failing evidence, mirroring the
//! R425 `dreaming_candidate_failure` durability pattern
//! ([`crate::dreaming`]): a proposal that fails a ratchet is never silently
//! dropped, it becomes a durable `promotion_rejection` record.

use std::fmt::Write as _;
use std::path::PathBuf;

use crate::engine::stable_id;
use crate::memory::MemoryEvent;
use crate::self_improvement::LearningRun;

mod gates;
mod materialize;
pub use gates::{replay_promotion_gates, replay_promotion_gates_with, GateCommandOutput};
pub use materialize::apply_promotions;

const CODING_MODIFICATION_SUITE_LINO: &str =
    include_str!("../data/benchmarks/coding-modification-suite.lino");
const INDUSTRY_SUITE_LINO: &str = include_str!("../data/benchmarks/industry-suite.lino");

/// The default seed file learned program-plan rules are promoted into.
pub const LEARNED_PROGRAM_RULES_SEED_FILE: &str = "data/seed/learned-program-rules.lino";

/// One benchmark ratchet a promotion must clear before its edit can be applied.
///
/// The floor (`minimum_pass_count`) and `runner` are read from the checked-in
/// `data/benchmarks/*.lino` manifests so the gate always reflects the same
/// ratchet CI enforces; the caller supplies the freshly observed pass/fail
/// counts from replaying the suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionRatchet {
    /// Benchmark suite id, e.g. `issue_362_multilingual_coding_modification`.
    pub suite_id: String,
    /// Command that produces the report.
    pub runner: String,
    /// Ratchet floor recorded by the manifest.
    pub minimum_pass_count: usize,
    /// Passing cases from the replay.
    pub passed: usize,
    /// Failing cases from the replay.
    pub failed: usize,
    /// Required pass rate in basis points (`10_000` means no failures).
    pub minimum_pass_rate_basis_points: usize,
    /// Whether the canonical gate command exited successfully.
    pub command_succeeded: bool,
    /// Content-addressed digest of stdout/stderr and exit status.
    pub evidence_digest: Option<String>,
}

impl PromotionRatchet {
    /// Construct a ratchet from explicit counts.
    #[must_use]
    pub fn new(
        suite_id: impl Into<String>,
        runner: impl Into<String>,
        minimum_pass_count: usize,
        passed: usize,
        failed: usize,
    ) -> Self {
        Self {
            suite_id: suite_id.into(),
            runner: runner.into(),
            minimum_pass_count,
            passed,
            failed,
            minimum_pass_rate_basis_points: 0,
            command_succeeded: false,
            evidence_digest: None,
        }
    }

    /// The coding-modification suite ratchet (issue #362), floor and runner read
    /// from `data/benchmarks/coding-modification-suite.lino`.
    #[must_use]
    pub fn coding_modification(passed: usize, failed: usize) -> Self {
        Self::from_manifest(
            CODING_MODIFICATION_SUITE_LINO,
            "issue_362_multilingual_coding_modification",
            passed,
            failed,
        )
    }

    /// The permissive industry-suite ratchet (issue #304), floor and runner read
    /// from `data/benchmarks/industry-suite.lino`.
    #[must_use]
    pub fn industry(passed: usize, failed: usize) -> Self {
        Self::from_manifest(
            INDUSTRY_SUITE_LINO,
            "issue_304_industry_permissive_slice",
            passed,
            failed,
        )
    }

    /// The unit-specification gate: the `cargo test` unit suite must be green
    /// (at least one passing spec, no failures) before promotion.
    #[must_use]
    pub fn unit_specs(passed: usize, failed: usize) -> Self {
        let mut gate = Self::new(
            "formal_ai_unit_specifications",
            "cargo test --test unit issue_656 -- --nocapture",
            1,
            passed,
            failed,
        );
        gate.minimum_pass_rate_basis_points = 10_000;
        gate
    }

    fn from_manifest(manifest: &str, fallback_id: &str, passed: usize, failed: usize) -> Self {
        let suite_id = manifest_field(manifest, "id").unwrap_or_else(|| fallback_id.to_owned());
        let runner =
            manifest_field(manifest, "runner").unwrap_or_else(|| String::from("cargo test"));
        let minimum_pass_count = manifest_field(manifest, "minimum_pass_count")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(1);
        let minimum_pass_rate_basis_points = manifest_field(manifest, "pass_rate_basis_points")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        let mut gate = Self::new(suite_id, runner, minimum_pass_count, passed, failed);
        gate.minimum_pass_rate_basis_points = minimum_pass_rate_basis_points;
        gate
    }

    /// Whether this ratchet permits promotion: passed at or above the floor.
    #[must_use]
    pub const fn clears(&self) -> bool {
        let total = self.passed.saturating_add(self.failed);
        let rate_clears = self.minimum_pass_rate_basis_points == 0
            || (total > 0
                && self.passed.saturating_mul(10_000)
                    >= self.minimum_pass_rate_basis_points.saturating_mul(total));
        self.command_succeeded
            && self.evidence_digest.is_some()
            && self.passed >= self.minimum_pass_count
            && rate_clears
    }

    /// Stable slug describing the outcome of this gate.
    #[must_use]
    pub const fn status_slug(&self) -> &'static str {
        if self.clears() {
            "cleared"
        } else {
            "blocked"
        }
    }

    /// A typed evidence link naming which ratchet ran, at what floor, and how it
    /// resolved. Recorded on the promotion event chain so a reviewer can trace a
    /// decision back to the exact benchmark run.
    #[must_use]
    pub fn evidence_link(&self) -> String {
        format!(
            "benchmark:{}:{}:{}/{}@floor{}@rate{}@{}",
            self.suite_id,
            self.status_slug(),
            self.passed,
            self.passed + self.failed,
            self.minimum_pass_count,
            self.minimum_pass_rate_basis_points,
            self.evidence_digest.as_deref().unwrap_or("unreplayed")
        )
    }
}

/// A concrete `.lino` seed edit a promotion would materialize.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedEdit {
    /// Repository-relative seed file the edit targets, e.g.
    /// `data/seed/learned-program-rules.lino`.
    pub seed_file: String,
    /// The Links Notation body appended to the seed file.
    pub lino: String,
}

impl SeedEdit {
    /// Construct a seed edit.
    #[must_use]
    pub fn new(seed_file: impl Into<String>, lino: impl Into<String>) -> Self {
        Self {
            seed_file: seed_file.into(),
            lino: lino.into(),
        }
    }
}

/// Whether a proposal was promoted or rejected after replaying its gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionOutcome {
    /// Every ratchet cleared; the seed edit may be materialized on a branch.
    Promoted,
    /// At least one ratchet did not clear; the edit is preserved, not applied.
    Rejected,
}

impl PromotionOutcome {
    /// The stable slug used in serialization.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Promoted => "promoted",
            Self::Rejected => "rejected",
        }
    }
}

/// One open self-improvement proposal considered for promotion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionProposal {
    /// Stable, content-addressed proposal id.
    pub id: String,
    /// Link back to the originating proposal, e.g. `learned_rule:<id>`.
    pub source: String,
    /// Human-readable review summary.
    pub summary: String,
    /// The seed edit this proposal would apply once promoted.
    pub edit: SeedEdit,
    /// Benchmark ratchets replayed for this proposal, in order.
    pub gates: Vec<PromotionRatchet>,
}

impl PromotionProposal {
    /// Construct a proposal, deriving a stable id from its source and edit.
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        summary: impl Into<String>,
        edit: SeedEdit,
        gates: Vec<PromotionRatchet>,
    ) -> Self {
        let source = source.into();
        let id = stable_id(
            "promotion",
            &format!("{source}\0{}\0{}", edit.seed_file, edit.lino),
        );
        Self {
            id,
            source,
            summary: summary.into(),
            edit,
            gates,
        }
    }

    /// Whether every gate cleared. A proposal with no gates never promotes: a
    /// promotion must show positive benchmark evidence, not merely an absence of
    /// failures.
    #[must_use]
    pub fn passes_all_gates(&self) -> bool {
        !self.gates.is_empty() && self.gates.iter().all(PromotionRatchet::clears)
    }

    /// The ratchets that did not clear, in order.
    #[must_use]
    pub fn failing_gates(&self) -> Vec<&PromotionRatchet> {
        self.gates.iter().filter(|gate| !gate.clears()).collect()
    }

    /// The promotion decision implied by the gate replay.
    #[must_use]
    pub fn outcome(&self) -> PromotionOutcome {
        if self.passes_all_gates() {
            PromotionOutcome::Promoted
        } else {
            PromotionOutcome::Rejected
        }
    }
}

/// A proposal paired with the promotion decision reached for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionRecord {
    /// The evaluated proposal.
    pub proposal: PromotionProposal,
    /// The decision reached after replaying its gates.
    pub outcome: PromotionOutcome,
}

/// The local review branch and remaining commit/draft-PR steps for a run.
///
/// Materialization creates the branch before writing accepted edits. Committing,
/// pushing, and opening a draft pull request remain an explicit plan for a human
/// reviewer; the protocol never performs those network-visible actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionBranchPlan {
    /// The branch a promotion would land on.
    pub branch: String,
    /// The ordered `git`/`gh` commands that open the reviewed pull request.
    /// These are printed for a human to run; the protocol never executes them.
    pub commands: Vec<String>,
}

/// A complete promotion run: every considered proposal and its decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionRun {
    /// Stable, content-addressed run id.
    pub id: String,
    /// Every proposal considered, with its decision.
    pub records: Vec<PromotionRecord>,
}

impl PromotionRun {
    /// Replay every proposal's gates and record the promotion decision for each.
    #[must_use]
    pub fn evaluate(proposals: Vec<PromotionProposal>) -> Self {
        let records: Vec<PromotionRecord> = proposals
            .into_iter()
            .map(|proposal| {
                let outcome = proposal.outcome();
                PromotionRecord { proposal, outcome }
            })
            .collect();
        let fingerprint = records
            .iter()
            .map(|record| format!("{}:{}", record.proposal.id, record.outcome.slug()))
            .collect::<Vec<_>>()
            .join(",");
        Self {
            id: stable_id("promotion_run", &fingerprint),
            records,
        }
    }

    /// Records that were promoted.
    #[must_use]
    pub fn promoted(&self) -> Vec<&PromotionRecord> {
        self.records
            .iter()
            .filter(|record| record.outcome == PromotionOutcome::Promoted)
            .collect()
    }

    /// Records that were rejected.
    #[must_use]
    pub fn rejected(&self) -> Vec<&PromotionRecord> {
        self.records
            .iter()
            .filter(|record| record.outcome == PromotionOutcome::Rejected)
            .collect()
    }

    /// The branch/PR plan for the promoted edits — never executed automatically.
    #[must_use]
    pub fn branch_plan(&self) -> PromotionBranchPlan {
        let branch = format!("promotion/{}", self.id);
        let mut commands = vec![format!("git checkout -b {branch}")];
        let promoted = self.promoted();
        for record in &promoted {
            commands.push(format!("git add {}", record.proposal.edit.seed_file));
        }
        if promoted.is_empty() {
            commands.push(String::from(
                "# no proposals cleared their ratchets; nothing to add",
            ));
        } else {
            commands.push(format!(
                "git commit -m \"promote {} self-improvement proposal(s) (run {})\"",
                promoted.len(),
                self.id
            ));
            commands.push(String::from(
                "gh pr create --draft --fill  # human review remains the outer gate",
            ));
        }
        PromotionBranchPlan { branch, commands }
    }

    /// Render the run as human-readable Links Notation for the CLI plan output.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("promotion_run\n");
        push_field(&mut out, 1, "id", &self.id);
        write_count(&mut out, 1, "considered", self.records.len());
        write_count(&mut out, 1, "promoted", self.promoted().len());
        write_count(&mut out, 1, "rejected", self.rejected().len());
        for record in &self.records {
            out.push_str("  proposal\n");
            push_field(&mut out, 2, "id", &record.proposal.id);
            push_field(&mut out, 2, "source", &record.proposal.source);
            push_field(&mut out, 2, "decision", record.outcome.slug());
            push_field(&mut out, 2, "seed_file", &record.proposal.edit.seed_file);
            push_field(&mut out, 2, "summary", &record.proposal.summary);
            for gate in &record.proposal.gates {
                out.push_str("    gate\n");
                push_field(&mut out, 3, "suite", &gate.suite_id);
                push_field(&mut out, 3, "status", gate.status_slug());
                push_field(&mut out, 3, "evidence", &gate.evidence_link());
            }
        }
        let plan = self.branch_plan();
        out.push_str("  branch_plan\n");
        push_field(&mut out, 2, "branch", &plan.branch);
        for command in &plan.commands {
            push_field(&mut out, 2, "command", command);
        }
        out.trim_end().to_owned()
    }

    /// Materialize the promotion protocol as an append-only chain of memory
    /// events. Promoted and rejected proposals both leave a durable trail; a
    /// rejection keeps the change it did *not* apply, mirroring the R425
    /// `dreaming_candidate_failure` pattern.
    #[must_use]
    pub fn memory_events(&self) -> Vec<MemoryEvent> {
        let mut events = Vec::new();
        for record in &self.records {
            let proposal = &record.proposal;
            events.push(proposal_event(proposal));
            for gate in &proposal.gates {
                events.push(evidence_event(proposal, gate));
            }
            events.push(decision_event(record));
            match record.outcome {
                PromotionOutcome::Promoted => events.push(applied_event(proposal)),
                PromotionOutcome::Rejected => events.push(rejection_event(proposal)),
            }
        }
        events
    }
}

/// One materialized seed edit written to a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedSeedEdit {
    /// The seed file that was written.
    pub seed_file: String,
    /// The absolute path written inside the workspace.
    pub path: PathBuf,
    /// Bytes appended to the seed file.
    pub bytes_written: usize,
}

/// The outcome of applying a promotion run to a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionApplyOutcome {
    /// Seed edits materialized for promoted proposals.
    pub applied: Vec<AppliedSeedEdit>,
    /// Proposal ids preserved as `promotion_rejection` failure records, not
    /// applied.
    pub rejected: Vec<String>,
    /// The branch/PR plan the reviewer runs to land the applied edits.
    pub branch_plan: PromotionBranchPlan,
    /// Content-addressed deterministic Formal AI Agent session records that
    /// authored the exact bytes written for each target file.
    pub agent_session_ids: Vec<String>,
}

/// Bridge accumulated self-improvement proposals into promotion candidates.
///
/// Each adoptable [`crate::self_improvement::LearnedRuleProposal`] becomes a
/// promotion proposal whose seed edit is the learned substitution rule and whose
/// gate is the coding-modification ratchet the learning run already replayed.
/// This is the concrete "collect open proposals" step of the CLI.
#[must_use]
pub fn promotions_from_learning_run(run: &LearningRun) -> Vec<PromotionProposal> {
    run.adoptable_rules()
        .into_iter()
        .map(|rule| {
            PromotionProposal::new(
                format!("learned_rule:{}", rule.id),
                rule.summary.clone(),
                SeedEdit::new(LEARNED_PROGRAM_RULES_SEED_FILE, rule.seed_rule_lino.clone()),
                gates::required_gates(),
            )
        })
        .collect()
}

/// A deterministic demonstration run for protocol and bundle tests.
///
/// One proposal clears its ratchets and is promoted; the other fails the
/// coding-modification floor and is kept as a rejection record. Production CLI
/// runs require an explicit proposal document and fresh canonical gate replay.
#[must_use]
pub fn demonstration_promotion_run() -> PromotionRun {
    PromotionRun::evaluate(demonstration_promotion_proposals())
}

/// The proposals behind [`demonstration_promotion_run`].
#[must_use]
pub fn demonstration_promotion_proposals() -> Vec<PromotionProposal> {
    let mut promoted = PromotionProposal::new(
        "learned_rule:demo_reverse_sort",
        "Promote learned `reverse` modifier for the list-files program plan.",
        SeedEdit::new(
            LEARNED_PROGRAM_RULES_SEED_FILE,
            learned_program_rule_lino(
                "learned_reverse_sort",
                "list_files_arg",
                "reverse",
                "list_files_arg_reverse_sort",
            ),
        ),
        vec![
            PromotionRatchet::coding_modification(5, 0),
            PromotionRatchet::industry(12, 3),
            PromotionRatchet::unit_specs(1, 0),
        ],
    );
    for gate in &mut promoted.gates {
        gate.command_succeeded = true;
        gate.evidence_digest = Some(stable_id(
            "promotion_gate_output",
            &format!("demonstration:{}", gate.suite_id),
        ));
    }
    let rejected = PromotionProposal::new(
        "learned_rule:demo_untested_rewrite",
        "Reject an under-benchmarked rewrite that regresses the coding-modification floor.",
        SeedEdit::new(
            LEARNED_PROGRAM_RULES_SEED_FILE,
            learned_program_rule_lino(
                "learned_untested_rewrite",
                "list_files_arg",
                "shuffle",
                "list_files_arg_shuffle",
            ),
        ),
        vec![PromotionRatchet::coding_modification(2, 4)],
    );
    vec![promoted, rejected]
}

/// Render a learned program-plan substitution rule as Links Notation. Kept in
/// sync with `crate::self_improvement`'s learned-rule shape so a bridged
/// proposal and a demonstration proposal produce the same seed format.
fn learned_program_rule_lino(
    rule_id: &str,
    base_task: &str,
    modifier: &str,
    resolved_task: &str,
) -> String {
    format!(
        "substitution_rules\n  id \"learned_program_plan_rules\"\n  rule \"{rule_id}\"\n    order \"90\"\n    event \"learned\"\n    when \"request:modifier -> {modifier}\"\n    replace \"request:task -> {base_task}\"\n      with \"request:task -> {resolved_task}\""
    )
}

fn proposal_event(proposal: &PromotionProposal) -> MemoryEvent {
    MemoryEvent {
        id: stable_id("promotion_proposal", &proposal.id),
        kind: Some(String::from("promotion_proposal")),
        role: Some(String::from("system")),
        intent: Some(String::from("promote")),
        inputs: Some(proposal.source.clone()),
        outputs: Some(proposal.edit.seed_file.clone()),
        content: Some(proposal.summary.clone()),
        evidence: vec![proposal.source.clone()],
        ..MemoryEvent::default()
    }
}

fn evidence_event(proposal: &PromotionProposal, gate: &PromotionRatchet) -> MemoryEvent {
    MemoryEvent {
        id: stable_id(
            "promotion_evidence",
            &format!("{}\0{}", proposal.id, gate.suite_id),
        ),
        kind: Some(String::from("promotion_evidence")),
        role: Some(String::from("system")),
        intent: Some(String::from("promote")),
        inputs: Some(gate.suite_id.clone()),
        outputs: Some(format!(
            "passed={} failed={} floor={} {}",
            gate.passed,
            gate.failed,
            gate.minimum_pass_count,
            gate.status_slug()
        )),
        content: Some(gate.runner.clone()),
        evidence: vec![proposal.id.clone(), gate.evidence_link()],
        ..MemoryEvent::default()
    }
}

fn decision_event(record: &PromotionRecord) -> MemoryEvent {
    let proposal = &record.proposal;
    let mut evidence = vec![proposal.id.clone()];
    evidence.extend(proposal.gates.iter().map(PromotionRatchet::evidence_link));
    MemoryEvent {
        id: stable_id("promotion_decision", &proposal.id),
        kind: Some(String::from("promotion_decision")),
        role: Some(String::from("system")),
        intent: Some(String::from("promote")),
        inputs: Some(proposal.source.clone()),
        outputs: Some(record.outcome.slug().to_owned()),
        content: Some(format!(
            "Promotion {} for {}",
            record.outcome.slug(),
            proposal.source
        )),
        evidence,
        ..MemoryEvent::default()
    }
}

fn applied_event(proposal: &PromotionProposal) -> MemoryEvent {
    MemoryEvent {
        id: stable_id("promotion_applied", &proposal.id),
        kind: Some(String::from("promotion_applied")),
        role: Some(String::from("system")),
        intent: Some(String::from("promote")),
        inputs: Some(proposal.edit.seed_file.clone()),
        outputs: Some(proposal.edit.lino.clone()),
        content: Some(format!(
            "Materialized seed edit for {} into {}",
            proposal.source, proposal.edit.seed_file
        )),
        evidence: vec![proposal.id.clone()],
        ..MemoryEvent::default()
    }
}

/// The durable failure record for a rejected proposal. It keeps the change that
/// was *not* applied together with the failing benchmark evidence, so a rejected
/// promotion is preserved for refinement rather than dropped (R425).
fn rejection_event(proposal: &PromotionProposal) -> MemoryEvent {
    let failing: Vec<String> = proposal
        .failing_gates()
        .iter()
        .map(|gate| gate.evidence_link())
        .collect();
    let mut evidence = vec![proposal.id.clone()];
    evidence.extend(failing.iter().cloned());
    MemoryEvent {
        id: stable_id("promotion_rejection", &proposal.id),
        kind: Some(String::from("promotion_rejection")),
        role: Some(String::from("system")),
        intent: Some(String::from("promote")),
        inputs: Some(proposal.edit.seed_file.clone()),
        outputs: Some(proposal.edit.lino.clone()),
        content: Some(format!(
            "Rejected {}; kept for refinement. Failing gate(s): {}",
            proposal.source,
            failing.join(", ")
        )),
        demo_label: Some(proposal.source.clone()),
        evidence,
        ..MemoryEvent::default()
    }
}

// ---- Links Notation proposal document parsing/rendering -------------------

/// Render a set of promotion proposals as a `promotion_proposals` Links Notation
/// document that [`parse_promotion_proposals`] reads back. Symmetric so the CLI
/// can round-trip a proposals file.
#[must_use]
pub fn render_promotion_proposals(proposals: &[PromotionProposal]) -> String {
    let mut out = String::from("promotion_proposals\n");
    for proposal in proposals {
        out.push_str("  proposal\n");
        push_field(&mut out, 2, "source", &proposal.source);
        push_field(&mut out, 2, "summary", &proposal.summary);
        push_field(&mut out, 2, "seed_file", &proposal.edit.seed_file);
        push_field(&mut out, 2, "seed_lino", &proposal.edit.lino);
        // Proposal documents declare only canonical suite identities. Commands,
        // floors and observations are executable evidence and are never accepted
        // from this untrusted input document.
        for gate in gates::required_gates() {
            out.push_str("    gate\n");
            push_field(&mut out, 3, "suite", &gate.suite_id);
        }
    }
    out.trim_end().to_owned()
}

/// Parse a `promotion_proposals` Links Notation document.
///
/// The format mirrors [`render_promotion_proposals`]: a top-level
/// `promotion_proposals` header, then one indented `proposal` block per
/// candidate, each carrying `source`/`summary`/`seed_file`/`seed_lino` fields and
/// zero or more nested `gate` blocks.
///
/// # Errors
///
/// Returns a human-readable error when a proposal is missing a required field or
/// a gate has a non-numeric count.
pub fn parse_promotion_proposals(text: &str) -> Result<Vec<PromotionProposal>, String> {
    let mut proposals = Vec::new();
    let mut current: Option<DraftProposal> = None;
    let mut in_gate = false;
    for raw in text.lines() {
        let indent = raw.chars().take_while(|c| *c == ' ').count();
        let content = raw.trim();
        if content.is_empty() {
            continue;
        }
        if indent == 0 {
            // Top-level header (`promotion_proposals`); ignore.
            continue;
        }
        if indent == 2 && content == "proposal" {
            if let Some(draft) = current.take() {
                proposals.push(draft.finish()?);
            }
            current = Some(DraftProposal::default());
            in_gate = false;
            continue;
        }
        let Some(draft) = current.as_mut() else {
            continue;
        };
        if indent == 4 && content == "gate" {
            draft.gates.push(DraftGate::default());
            in_gate = true;
            continue;
        }
        let Some((key, value)) = split_field(content) else {
            continue;
        };
        if in_gate && indent >= 6 {
            let Some(gate) = draft.gates.last_mut() else {
                continue;
            };
            match key {
                "suite" => gate.suite = Some(value),
                "runner" | "minimum_pass_count" | "passed" | "failed" => {
                    return Err(format!(
                        "proposal gates must not provide `{key}`; commands and results come from fresh canonical replay"
                    ));
                }
                _ => {}
            }
            continue;
        }
        // A proposal-level field closes any open gate.
        in_gate = false;
        match key {
            "source" => draft.source = Some(value),
            "summary" => draft.summary = Some(value),
            "seed_file" => draft.seed_file = Some(value),
            "seed_lino" => draft.seed_lino = Some(value),
            _ => {}
        }
    }
    if let Some(draft) = current.take() {
        proposals.push(draft.finish()?);
    }
    Ok(proposals)
}

#[derive(Default)]
struct DraftProposal {
    source: Option<String>,
    summary: Option<String>,
    seed_file: Option<String>,
    seed_lino: Option<String>,
    gates: Vec<DraftGate>,
}

impl DraftProposal {
    fn finish(self) -> Result<PromotionProposal, String> {
        let source = self.source.ok_or("proposal missing `source`")?;
        let seed_file = self.seed_file.ok_or("proposal missing `seed_file`")?;
        let seed_lino = self.seed_lino.ok_or("proposal missing `seed_lino`")?;
        let summary = self.summary.unwrap_or_else(|| source.clone());
        self.gates
            .into_iter()
            .map(DraftGate::finish)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PromotionProposal::new(
            source,
            summary,
            SeedEdit::new(seed_file, seed_lino),
            gates::required_gates(),
        ))
    }
}

#[derive(Default)]
struct DraftGate {
    suite: Option<String>,
}

impl DraftGate {
    fn finish(self) -> Result<(), String> {
        let suite = self.suite.ok_or("gate missing `suite`")?;
        if gates::required_gates()
            .iter()
            .any(|canonical| canonical.suite_id == suite)
        {
            Ok(())
        } else {
            Err(format!("unknown promotion gate suite `{suite}`"))
        }
    }
}

// ---- small local helpers, mirroring src/self_improvement.rs ---------------

fn manifest_field(manifest: &str, key: &str) -> Option<String> {
    for line in manifest.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(key) else {
            continue;
        };
        if rest.starts_with(char::is_whitespace) {
            return Some(unquote(rest.trim()));
        }
    }
    None
}

fn split_field(content: &str) -> Option<(&str, String)> {
    let (key, rest) = content.split_once(' ')?;
    Some((key, unquote(rest.trim())))
}

fn unquote(value: &str) -> String {
    let value = value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value);
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn push_field(out: &mut String, depth: usize, key: &str, value: &str) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    let _ = writeln!(out, "{key} \"{}\"", quote(value));
}

fn write_count(out: &mut String, depth: usize, key: &str, value: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    let _ = writeln!(out, "{key} \"{value}\"");
}
