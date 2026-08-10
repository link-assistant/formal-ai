//! Event-sourced research, learning, verification, and recovery (issue #873).
//!
//! This module joins the existing research, proposal, benchmark, permission,
//! and orchestration primitives behind one data-defined cycle. Unknowns enter
//! as research frontiers. External observations retain provenance but their
//! cached payloads are disposable. New facts, procedures, and even the cycle's
//! own recipe are candidate versions until an immutable-majority baseline
//! passes. A rejected candidate never moves the active stable pointer.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::engine::stable_id;

/// The issue-requested default wall-clock budget. Callers can override it.
pub const DEFAULT_RESEARCH_TIME_LIMIT_SECONDS: u64 = 60 * 60;

/// The ordered meta-algorithm interpreted by [`recipe_steps`].
pub const RESEARCH_LEARNING_RECIPE: &str =
    include_str!("../data/meta/research-learning-recovery.lino");

/// How the cycle handles a recoverable action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomyMode {
    /// Ask only when more than one distinct recovery remains viable.
    AskOnAmbiguity,
    /// Rank the options from recorded outcomes and continue autonomously.
    FullTrust,
    /// Rank an option, but require permission before each external command.
    PerCommand,
}

/// Runtime policy for one cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CycleConfig {
    pub time_limit_seconds: u64,
    pub autonomy: AutonomyMode,
}

impl Default for CycleConfig {
    fn default() -> Self {
        Self {
            time_limit_seconds: DEFAULT_RESEARCH_TIME_LIMIT_SECONDS,
            autonomy: AutonomyMode::AskOnAmbiguity,
        }
    }
}

/// The same reducer versions facts, executable procedures, and its own recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeKind {
    Fact,
    Procedure,
    MetaAlgorithm,
}

impl KnowledgeKind {
    const fn slug(self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Procedure => "procedure",
            Self::MetaAlgorithm => "meta_algorithm",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionStatus {
    Candidate,
    Stable,
    Rejected,
}

impl VersionStatus {
    const fn slug(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Stable => "stable",
            Self::Rejected => "rejected",
        }
    }
}

/// One immutable verification result attached to the candidate it evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationGate {
    pub id: String,
    pub immutable: bool,
    pub passed: bool,
}

impl VerificationGate {
    #[must_use]
    pub fn immutable(id: impl Into<String>, passed: bool) -> Self {
        Self {
            id: id.into(),
            immutable: true,
            passed,
        }
    }

    #[must_use]
    pub fn adaptive(id: impl Into<String>, passed: bool) -> Self {
        Self {
            id: id.into(),
            immutable: false,
            passed,
        }
    }
}

/// An append-only knowledge snapshot. Payloads are retained because versions
/// are deliberate memory; disposable web captures live in [`SourceReceipt`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeVersion {
    pub id: String,
    pub parent: Option<String>,
    pub kind: KnowledgeKind,
    pub payload: String,
    pub status: VersionStatus,
    pub verification: Vec<VerificationGate>,
}

/// Provenance for one external observation. The locator and digest survive
/// eviction, so the observation can be recollected and compared later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReceipt {
    pub id: String,
    pub locator: String,
    pub content_id: String,
    pub cached_payload: Option<String>,
    pub recomputable: bool,
}

/// Outcome history used by full-trust ranking. Advantages and disadvantages
/// are integer evidence weights, not opaque model scores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryOption {
    pub id: String,
    pub prior_successes: u32,
    pub prior_failures: u32,
    pub advantages: u32,
    pub disadvantages: u32,
}

impl RecoveryOption {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            prior_successes: 0,
            prior_failures: 0,
            advantages: 0,
            disadvantages: 0,
        }
    }

    #[must_use]
    pub fn score(&self) -> i64 {
        i64::from(self.prior_successes) * 4 + i64::from(self.advantages)
            - i64::from(self.prior_failures) * 4
            - i64::from(self.disadvantages)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryDecision {
    AskUser { option_ids: Vec<String> },
    Selected { option_id: String },
    PermissionRequired { option_id: String },
    AwaitingContinuation { current_plan: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleState {
    Researching,
    Verifying,
    Stable,
    Recovering,
    AwaitingUser,
    AwaitingPermission,
    AwaitingContinuation,
}

impl CycleState {
    const fn slug(self) -> &'static str {
        match self {
            Self::Researching => "researching",
            Self::Verifying => "verifying",
            Self::Stable => "stable",
            Self::Recovering => "recovering",
            Self::AwaitingUser => "awaiting_user",
            Self::AwaitingPermission => "awaiting_permission",
            Self::AwaitingContinuation => "awaiting_continuation",
        }
    }
}

/// A hash-linked transition record. Recovery adds events; it never rewrites
/// the history that explains how a stable version was selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleEvent {
    pub sequence: u64,
    pub kind: String,
    pub detail: String,
    pub previous_id: String,
    pub id: String,
}

/// One general reducer for unknown → research → candidate → gate → recovery.
#[derive(Debug, Clone)]
pub struct ResearchLearningCycle {
    config: CycleConfig,
    baseline_gate_ids: BTreeSet<String>,
    versions: Vec<KnowledgeVersion>,
    active_version_id: String,
    sources: Vec<SourceReceipt>,
    events: Vec<CycleEvent>,
    state: CycleState,
    pending_options: Vec<String>,
}

impl ResearchLearningCycle {
    #[must_use]
    pub fn new(
        stable_payload: impl Into<String>,
        baseline_gate_ids: impl IntoIterator<Item = impl Into<String>>,
        config: CycleConfig,
    ) -> Self {
        let payload = stable_payload.into();
        let id = stable_id("knowledge_version", &format!("root:{payload}"));
        let mut cycle = Self {
            config,
            baseline_gate_ids: baseline_gate_ids.into_iter().map(Into::into).collect(),
            versions: vec![KnowledgeVersion {
                id: id.clone(),
                parent: None,
                kind: KnowledgeKind::MetaAlgorithm,
                payload,
                status: VersionStatus::Stable,
                verification: Vec::new(),
            }],
            active_version_id: id,
            sources: Vec::new(),
            events: Vec::new(),
            state: CycleState::Stable,
            pending_options: Vec::new(),
        };
        cycle.append_event("stable_initialized", cycle.active_version_id.clone());
        cycle
    }

    #[must_use]
    pub const fn config(&self) -> CycleConfig {
        self.config
    }

    #[must_use]
    pub const fn state(&self) -> CycleState {
        self.state
    }

    #[must_use]
    pub fn versions(&self) -> &[KnowledgeVersion] {
        &self.versions
    }

    #[must_use]
    pub fn sources(&self) -> &[SourceReceipt] {
        &self.sources
    }

    #[must_use]
    pub fn events(&self) -> &[CycleEvent] {
        &self.events
    }

    #[must_use]
    pub fn active_version(&self) -> &KnowledgeVersion {
        self.versions
            .iter()
            .find(|version| version.id == self.active_version_id)
            .expect("active version is retained in append-only history")
    }

    pub fn begin_unknown(&mut self, frontier: impl Into<String>) {
        self.state = CycleState::Researching;
        self.append_event("unknown_frontier", frontier.into());
    }

    pub fn record_source(
        &mut self,
        locator: impl Into<String>,
        payload: impl Into<String>,
        recomputable: bool,
    ) -> String {
        let locator = locator.into();
        let payload = payload.into();
        let content_id = stable_id("research_content", &payload);
        let id = stable_id(
            "source_receipt",
            &format!("{}:{content_id}:{}", locator, self.sources.len()),
        );
        self.sources.push(SourceReceipt {
            id: id.clone(),
            locator,
            content_id,
            cached_payload: Some(payload),
            recomputable,
        });
        self.append_event("source_captured", id.clone());
        id
    }

    /// Drop only the reproducible payload; the provenance receipt stays.
    pub fn evict_source(&mut self, receipt_id: &str) -> bool {
        let evicted = self
            .sources
            .iter_mut()
            .find(|source| source.id == receipt_id && source.recomputable)
            .is_some_and(|source| source.cached_payload.take().is_some());
        if evicted {
            self.append_event("source_payload_evicted", receipt_id.to_owned());
        }
        evicted
    }

    /// Rehydrate an evicted capture only when recollection produces the same
    /// content identity. Changed external data is recorded as a new receipt via
    /// [`Self::record_source`] instead of silently rewriting prior evidence.
    pub fn recollect_source(&mut self, receipt_id: &str, payload: impl Into<String>) -> bool {
        let payload = payload.into();
        let content_id = stable_id("research_content", &payload);
        let restored = self
            .sources
            .iter_mut()
            .find(|source| {
                source.id == receipt_id && source.recomputable && source.content_id == content_id
            })
            .is_some_and(|source| {
                source.cached_payload = Some(payload);
                true
            });
        if restored {
            self.append_event("source_payload_recollected", receipt_id.to_owned());
        }
        restored
    }

    pub fn propose_version(&mut self, kind: KnowledgeKind, payload: impl Into<String>) -> String {
        let payload = payload.into();
        let parent = self.active_version_id.clone();
        let id = stable_id(
            "knowledge_version",
            &format!("{parent}:{}:{}:{payload}", kind.slug(), self.versions.len()),
        );
        self.versions.push(KnowledgeVersion {
            id: id.clone(),
            parent: Some(parent),
            kind,
            payload,
            status: VersionStatus::Candidate,
            verification: Vec::new(),
        });
        self.state = CycleState::Verifying;
        self.append_event("candidate_proposed", id.clone());
        id
    }

    /// Promote only when every gate passes, every baseline gate is present as
    /// immutable, and immutable gates are a strict majority.
    pub fn verify_candidate(&mut self, candidate_id: &str, gates: Vec<VerificationGate>) -> bool {
        let immutable_ids = gates
            .iter()
            .filter(|gate| gate.immutable)
            .map(|gate| gate.id.as_str())
            .collect::<BTreeSet<_>>();
        let baseline_present = self
            .baseline_gate_ids
            .iter()
            .all(|required| immutable_ids.contains(required.as_str()));
        let immutable_majority =
            gates.iter().filter(|gate| gate.immutable).count() * 2 > gates.len();
        let passed = !gates.is_empty()
            && !self.baseline_gate_ids.is_empty()
            && baseline_present
            && immutable_majority
            && gates.iter().all(|gate| gate.passed);
        let Some(version) = self.versions.iter_mut().find(|version| {
            version.id == candidate_id && version.status == VersionStatus::Candidate
        }) else {
            self.append_event("candidate_missing", candidate_id.to_owned());
            return false;
        };
        version.verification = gates;
        version.status = if passed {
            VersionStatus::Stable
        } else {
            VersionStatus::Rejected
        };
        if passed {
            candidate_id.clone_into(&mut self.active_version_id);
            self.state = CycleState::Stable;
            self.append_event("candidate_promoted", candidate_id.to_owned());
        } else {
            self.state = CycleState::Recovering;
            self.append_event(
                "candidate_rejected_restore_stable",
                self.active_version_id.clone(),
            );
        }
        passed
    }

    /// Move the active pointer only to a version that previously passed its
    /// gate. Candidate and rejected states can be inspected but never activated.
    pub fn recover_stable(&mut self, version_id: &str) -> bool {
        let recoverable = self
            .versions
            .iter()
            .any(|version| version.id == version_id && version.status == VersionStatus::Stable);
        if recoverable {
            version_id.clone_into(&mut self.active_version_id);
            self.state = CycleState::Stable;
            self.append_event("stable_recovered", version_id.to_owned());
        }
        recoverable
    }

    /// Convert every error into a continuation decision. With no supplied
    /// alternative, restoring the current stable version and researching again
    /// is the deterministic recovery option.
    pub fn recover_from_error(
        &mut self,
        error_id: impl Into<String>,
        mut options: Vec<RecoveryOption>,
    ) -> RecoveryDecision {
        self.state = CycleState::Recovering;
        self.append_event("error_observed", error_id.into());
        if options.is_empty() {
            options.push(RecoveryOption::new("restore_stable_and_research"));
        }
        options.sort_by(|left, right| {
            right
                .score()
                .cmp(&left.score())
                .then_with(|| left.id.cmp(&right.id))
        });
        self.pending_options = options.iter().map(|option| option.id.clone()).collect();

        let decision = match self.config.autonomy {
            AutonomyMode::AskOnAmbiguity if options.len() > 1 => {
                self.state = CycleState::AwaitingUser;
                RecoveryDecision::AskUser {
                    option_ids: self.pending_options.clone(),
                }
            }
            AutonomyMode::PerCommand => {
                self.state = CycleState::AwaitingPermission;
                RecoveryDecision::PermissionRequired {
                    option_id: options[0].id.clone(),
                }
            }
            AutonomyMode::AskOnAmbiguity | AutonomyMode::FullTrust => {
                self.state = CycleState::Researching;
                RecoveryDecision::Selected {
                    option_id: options[0].id.clone(),
                }
            }
        };
        self.append_event("recovery_decision", decision_slug(&decision));
        decision
    }

    pub fn select_recovery(&mut self, option_id: &str) -> bool {
        if !self
            .pending_options
            .iter()
            .any(|pending| pending == option_id)
        {
            return false;
        }
        self.state = CycleState::Researching;
        self.append_event("recovery_selected", option_id.to_owned());
        true
    }

    /// Stop at the configured wall-clock bound with a resumable plan, never a
    /// terminal failure. Below the limit this returns `None`.
    pub fn check_time_limit(
        &mut self,
        elapsed_seconds: u64,
        current_plan: impl Into<String>,
    ) -> Option<RecoveryDecision> {
        if elapsed_seconds < self.config.time_limit_seconds {
            return None;
        }
        let current_plan = current_plan.into();
        self.state = CycleState::AwaitingContinuation;
        self.append_event("time_limit_reached", current_plan.clone());
        Some(RecoveryDecision::AwaitingContinuation { current_plan })
    }

    pub fn continue_with_permission(&mut self, additional_seconds: u64) {
        self.config.time_limit_seconds = self
            .config
            .time_limit_seconds
            .saturating_add(additional_seconds);
        self.state = CycleState::Researching;
        self.append_event("continuation_permitted", additional_seconds.to_string());
    }

    /// Canonical, reviewable snapshot. Event ids bind the ordered history.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("research_learning_cycle\n");
        let _ = writeln!(out, "  state \"{}\"", self.state.slug());
        let _ = writeln!(out, "  active_version \"{}\"", self.active_version_id);
        let _ = writeln!(
            out,
            "  time_limit_seconds \"{}\"",
            self.config.time_limit_seconds
        );
        for version in &self.versions {
            out.push_str("  version\n");
            let _ = writeln!(out, "    id \"{}\"", version.id);
            let _ = writeln!(out, "    kind \"{}\"", version.kind.slug());
            let _ = writeln!(out, "    status \"{}\"", version.status.slug());
            if let Some(parent) = &version.parent {
                let _ = writeln!(out, "    parent \"{parent}\"");
            }
        }
        for source in &self.sources {
            let _ = writeln!(out, "  source_receipt");
            let _ = writeln!(out, "    id \"{}\"", source.id);
            let _ = writeln!(out, "    locator \"{}\"", quote(&source.locator));
            let _ = writeln!(out, "    content_id \"{}\"", source.content_id);
            let _ = writeln!(out, "    cached \"{}\"", source.cached_payload.is_some());
            let _ = writeln!(out, "    recomputable \"{}\"", source.recomputable);
        }
        for event in &self.events {
            out.push_str("  event\n");
            let _ = writeln!(out, "    sequence \"{}\"", event.sequence);
            let _ = writeln!(out, "    kind \"{}\"", event.kind);
            let _ = writeln!(out, "    detail \"{}\"", quote(&event.detail));
            let _ = writeln!(out, "    previous_id \"{}\"", event.previous_id);
            let _ = writeln!(out, "    id \"{}\"", event.id);
        }
        out.trim_end().to_owned()
    }

    fn append_event(&mut self, kind: &str, detail: String) {
        let sequence = self.events.len() as u64;
        let previous_id = self
            .events
            .last()
            .map_or_else(String::new, |event| event.id.clone());
        let id = stable_id(
            "research_learning_event",
            &format!("{sequence}:{previous_id}:{kind}:{detail}"),
        );
        self.events.push(CycleEvent {
            sequence,
            kind: kind.to_owned(),
            detail,
            previous_id,
            id,
        });
    }
}

/// Parse the data-defined reducer phases. Adding a step to the recipe extends
/// the interpreted sequence without adding a control-flow branch.
#[must_use]
pub fn recipe_steps() -> Vec<String> {
    RESEARCH_LEARNING_RECIPE
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("step \"")
                .and_then(|value| value.strip_suffix('"'))
                .map(str::to_owned)
        })
        .collect()
}

fn decision_slug(decision: &RecoveryDecision) -> String {
    match decision {
        RecoveryDecision::AskUser { option_ids } => format!("ask_user:{}", option_ids.join(",")),
        RecoveryDecision::Selected { option_id } => format!("selected:{option_id}"),
        RecoveryDecision::PermissionRequired { option_id } => {
            format!("permission_required:{option_id}")
        }
        RecoveryDecision::AwaitingContinuation { current_plan } => {
            format!("awaiting_continuation:{current_plan}")
        }
    }
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
