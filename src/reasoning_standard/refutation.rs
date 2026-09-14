//! Refutation-first logic: a conclusion is earned by surviving attacks, not by
//! collecting agreement.
//!
//! Issue #1073, requirement 5: "reasoning must be refutation-first … a
//! deliberately wide variety of refutations must be attempted … only after
//! refutations are themselves refuted, or the alternative is positively proven,
//! may we lean toward a conclusion. Otherwise the honest output is: not
//! confirmed and not refuted."
//!
//! The ledger below is the machine-checkable form of that sentence. Probes are
//! recorded per conclusion, each naming the kind of attack it makes and the
//! mechanism it proposes; the ledger counts the *variety* of the attack, not the
//! number of restatements, and reaches a lean only when every probe is settled.
//! Anything else is [`LedgerState::Open`], which carries the blockers by name so
//! the honest verdict says what stopped the check rather than shrugging.

use std::collections::BTreeSet;

/// Which axis a refutation attacks along.
///
/// Requirement 5 asks for refutations that differ "in mechanism, in source, in
/// assumption", so the axis is part of the record and the ledger can tell three
/// genuinely different attacks from one doubt repeated three times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefutationAxis {
    /// A different causal mechanism would produce the same observation.
    Mechanism,
    /// A different source contradicts the one relied on.
    Source,
    /// An assumption the conclusion rests on may not hold.
    Assumption,
}

impl RefutationAxis {
    /// Stable slug for the data files and the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Mechanism => "mechanism",
            Self::Source => "source",
            Self::Assumption => "assumption",
        }
    }

    /// Parse a slug from the episode data.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "mechanism" => Some(Self::Mechanism),
            "source" => Some(Self::Source),
            "assumption" => Some(Self::Assumption),
            _ => None,
        }
    }
}

/// What happened when a refutation was pushed against the evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// The refutation was itself refuted by evidence: the conclusion survives
    /// this attack.
    Refuted,
    /// The refutation held up: its alternative is the better explanation.
    Survived,
    /// The refutation could not be checked at all.
    Unchecked,
}

impl ProbeOutcome {
    /// Stable slug for the data files and the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Refuted => "refuted",
            Self::Survived => "survived",
            Self::Unchecked => "unchecked",
        }
    }

    /// Parse a slug; anything unrecognized is [`Self::Unchecked`], so an
    /// unreadable outcome blocks a lean instead of granting one.
    #[must_use]
    pub fn from_slug(slug: &str) -> Self {
        match slug {
            "refuted" => Self::Refuted,
            "survived" => Self::Survived,
            _ => Self::Unchecked,
        }
    }
}

/// One attempt to refute a conclusion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefutationProbe {
    /// Stable id of the probe.
    pub id: String,
    /// The conclusion this probe attacks.
    pub conclusion: String,
    /// Which axis the attack runs along.
    pub axis: RefutationAxis,
    /// The alternative mechanism, source, or assumption the probe proposes.
    /// Two probes with the same mechanism are one refutation, not two.
    pub mechanism: String,
    /// What the probe denies about the conclusion.
    pub denies: String,
    /// How the probe settled.
    pub outcome: ProbeOutcome,
    /// Observations that settled the probe, by id.
    pub evidence: Vec<String>,
    /// What stopped the check, when the probe is unchecked.
    pub blocker: String,
}

impl RefutationProbe {
    /// Whether the probe is settled by evidence rather than by assertion.
    #[must_use]
    pub const fn is_evidenced(&self) -> bool {
        !self.evidence.is_empty()
    }

    /// The blocker this probe contributes, if it blocks a lean.
    #[must_use]
    pub fn blocking_reason(&self) -> Option<String> {
        match self.outcome {
            ProbeOutcome::Unchecked => Some(if self.blocker.trim().is_empty() {
                format!("{}:unchecked_without_stated_blocker", self.id)
            } else {
                format!("{}:{}", self.id, self.blocker)
            }),
            ProbeOutcome::Refuted | ProbeOutcome::Survived if !self.is_evidenced() => {
                Some(format!("{}:settled_without_evidence", self.id))
            }
            _ => None,
        }
    }
}

/// Whether the refutation record permits leaning toward a conclusion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerState {
    /// Every probe was itself refuted by evidence: the conclusion may be leaned
    /// toward.
    Discharged,
    /// A probe survived on evidence: its alternative is positively proven and
    /// the conclusion is refuted.
    AlternativeProven {
        /// The probe that survived.
        probe: String,
    },
    /// Neither: the honest output is "not confirmed and not refuted".
    Open {
        /// What stopped the check, by name.
        blockers: Vec<String>,
    },
}

impl LedgerState {
    /// Stable slug for the trace.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Discharged => "discharged",
            Self::AlternativeProven { .. } => "alternative_proven",
            Self::Open { .. } => "open",
        }
    }

    /// Whether a lean toward some conclusion is permitted at all.
    #[must_use]
    pub const fn permits_lean(&self) -> bool {
        !matches!(self, Self::Open { .. })
    }
}

/// The refutations attempted against one conclusion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefutationLedger {
    /// The conclusion under attack.
    pub conclusion: String,
    /// Every attempt made against it, in record order.
    pub probes: Vec<RefutationProbe>,
}

impl RefutationLedger {
    /// Collect the probes aimed at `conclusion`.
    #[must_use]
    pub fn for_conclusion(conclusion: &str, probes: &[RefutationProbe]) -> Self {
        Self {
            conclusion: conclusion.to_owned(),
            probes: probes
                .iter()
                .filter(|probe| probe.conclusion == conclusion)
                .cloned()
                .collect(),
        }
    }

    /// How many genuinely distinct attacks were made: distinct mechanisms, so a
    /// doubt restated does not inflate the count.
    #[must_use]
    pub fn variety(&self) -> usize {
        self.probes
            .iter()
            .map(|probe| probe.mechanism.trim())
            .filter(|mechanism| !mechanism.is_empty())
            .collect::<BTreeSet<_>>()
            .len()
    }

    /// How many of the three axis kinds the attempts span.
    #[must_use]
    pub fn axis_kinds(&self) -> usize {
        self.probes
            .iter()
            .map(|probe| probe.axis)
            .collect::<BTreeSet<_>>()
            .len()
    }

    /// Whether the attempted variety clears both thresholds.
    #[must_use]
    pub fn has_sufficient_variety(&self, minimum_attempts: usize, minimum_kinds: usize) -> bool {
        self.variety() >= minimum_attempts && self.axis_kinds() >= minimum_kinds
    }

    /// Settle the ledger.
    ///
    /// A surviving, evidenced probe positively proves its alternative and decides
    /// the question against the conclusion. Otherwise every probe must be refuted
    /// on evidence; anything unsettled leaves the ledger open with named blockers.
    #[must_use]
    pub fn state(&self) -> LedgerState {
        if self.probes.is_empty() {
            return LedgerState::Open {
                blockers: vec![format!("{}:no_refutation_attempted", self.conclusion)],
            };
        }
        if let Some(survivor) = self
            .probes
            .iter()
            .find(|probe| matches!(probe.outcome, ProbeOutcome::Survived) && probe.is_evidenced())
        {
            return LedgerState::AlternativeProven {
                probe: survivor.id.clone(),
            };
        }
        let blockers = self
            .probes
            .iter()
            .filter_map(RefutationProbe::blocking_reason)
            .collect::<Vec<_>>();
        if blockers.is_empty() {
            LedgerState::Discharged
        } else {
            LedgerState::Open { blockers }
        }
    }
}
