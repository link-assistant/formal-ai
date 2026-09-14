//! Computed source trust: a tier is *derived* from a primacy chain, never asserted.
//!
//! Issue #1073, requirement 4: "the trust hierarchy itself must be computed,
//! not assumed … whether something deserves trust must itself be determined
//! through primary sources". Before this module the repository declared a
//! `source_tier` per source in `data/seed/sources-registry.lino` and read it
//! back verbatim, so the hierarchy was exactly the assumption the requirement
//! forbids.
//!
//! Here a source carries a [`PrimacyChain`]: the hops that separate it from the
//! primary record, each hop naming its upstream and the primary document that
//! establishes the hop (a site's own policy page, its own charter, its own
//! licence). [`PrimacyChain::derive_tier`] turns that structure into a
//! [`SourceTier`] by pure arithmetic over the hops. A chain with no basis, or a
//! hop that cannot name its upstream, derives [`SourceTier::Unoriginal`] — an
//! unfounded claim of trust buys nothing rather than defaulting to "probably
//! fine".

use crate::relative_meta_logic::SourceTier;
use crate::seed::parser::LinoNode;

/// What one hop of a primacy chain does to the distance from the primary record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimacyKind {
    /// The subject publishing about itself: distance 0, first-party.
    SelfPublished,
    /// A first-hand record of the subject: distance 0, original journalism.
    FirstHandRecord,
    /// Editorial synthesis of other people's records: one hop away.
    EditorialSynthesis,
    /// A citation of a named upstream: one hop away.
    Citation,
    /// A copy that adds no verification: the chain is dead here.
    Repost,
}

impl PrimacyKind {
    /// Stable slug used in the seed data and in the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::SelfPublished => "self_published",
            Self::FirstHandRecord => "first_hand_record",
            Self::EditorialSynthesis => "editorial_synthesis",
            Self::Citation => "citation",
            Self::Repost => "repost",
        }
    }

    /// Parse a slug; unknown slugs are treated as [`Self::Repost`] so an
    /// unrecognized hop can never silently earn trust.
    #[must_use]
    pub fn from_slug(slug: &str) -> Self {
        match slug {
            "self_published" => Self::SelfPublished,
            "first_hand_record" => Self::FirstHandRecord,
            "editorial_synthesis" => Self::EditorialSynthesis,
            "citation" => Self::Citation,
            _ => Self::Repost,
        }
    }

    /// How many hops this step adds between the source and the primary record.
    #[must_use]
    pub const fn distance(self) -> u32 {
        match self {
            Self::SelfPublished | Self::FirstHandRecord => 0,
            Self::EditorialSynthesis | Self::Citation | Self::Repost => 1,
        }
    }

    /// Whether this hop must name an upstream to be well-founded.
    #[must_use]
    pub const fn requires_upstream(self) -> bool {
        self.distance() > 0
    }
}

/// One hop between a source and the primary record it rests on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimacyStep {
    /// What the hop does.
    pub kind: PrimacyKind,
    /// What this hop rests on: the upstream record, corpus, or reporter.
    pub upstream: String,
    /// The primary document that establishes this hop — the source's own policy,
    /// charter, or licence. Empty means the hop is unfounded.
    pub basis: String,
}

impl PrimacyStep {
    /// Build a hop.
    #[must_use]
    pub fn new(kind: PrimacyKind, upstream: impl Into<String>, basis: impl Into<String>) -> Self {
        Self {
            kind,
            upstream: upstream.into(),
            basis: basis.into(),
        }
    }

    /// Whether the hop is well-founded: it cites the primary document that
    /// establishes it, and names an upstream when the hop needs one.
    #[must_use]
    pub fn is_well_founded(&self) -> bool {
        if self.basis.trim().is_empty() {
            return false;
        }
        !self.kind.requires_upstream() || !self.upstream.trim().is_empty()
    }
}

/// Why a chain derived the tier it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivationReason {
    /// No hops at all: the tier would have to be assumed.
    NoPrimacyChain,
    /// A hop cites no primary document, or names no upstream where one is due.
    UnfoundedStep,
    /// A repost hop: the chain adds no verification past this point.
    RepostInChain,
    /// Distance 0, the subject speaking about itself.
    SpeaksForItself,
    /// Distance 0, a first-hand record of the subject.
    FirstHandRecord,
    /// Distance above 0, every hop well-founded and naming its upstream.
    NamedUpstreamChain,
}

impl DerivationReason {
    /// Stable slug for the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::NoPrimacyChain => "no_primacy_chain",
            Self::UnfoundedStep => "unfounded_step",
            Self::RepostInChain => "repost_in_chain",
            Self::SpeaksForItself => "speaks_for_itself",
            Self::FirstHandRecord => "first_hand_record",
            Self::NamedUpstreamChain => "named_upstream_chain",
        }
    }
}

/// The computed trust of one source: the tier plus the derivation behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedTrust {
    /// The derived tier.
    pub tier: SourceTier,
    /// How many hops separate the source from the primary record.
    pub distance: u32,
    /// Which rule of the derivation fired.
    pub reason: DerivationReason,
}

/// The hops that separate a source from the primary record it rests on.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrimacyChain {
    /// The hops, ordered from the source toward the primary record.
    pub steps: Vec<PrimacyStep>,
}

impl PrimacyChain {
    /// Build a chain from its hops.
    #[must_use]
    pub const fn new(steps: Vec<PrimacyStep>) -> Self {
        Self { steps }
    }

    /// Total distance from the primary record.
    #[must_use]
    pub fn distance(&self) -> u32 {
        self.steps.iter().map(|step| step.kind.distance()).sum()
    }

    /// Derive the trust tier from the chain's structure alone.
    ///
    /// This is the whole of requirement 4: the tier is a function of *how far
    /// the source stands from the primary record* and *whether every hop can
    /// name what it rests on*, so no caller can hand the system a tier it did
    /// not earn.
    #[must_use]
    pub fn derive_trust(&self) -> DerivedTrust {
        let distance = self.distance();
        if self.steps.is_empty() {
            return DerivedTrust {
                tier: SourceTier::Unoriginal,
                distance,
                reason: DerivationReason::NoPrimacyChain,
            };
        }
        if self.steps.iter().any(|step| !step.is_well_founded()) {
            return DerivedTrust {
                tier: SourceTier::Unoriginal,
                distance,
                reason: DerivationReason::UnfoundedStep,
            };
        }
        if self
            .steps
            .iter()
            .any(|step| matches!(step.kind, PrimacyKind::Repost))
        {
            return DerivedTrust {
                tier: SourceTier::Unoriginal,
                distance,
                reason: DerivationReason::RepostInChain,
            };
        }
        if distance == 0 {
            let first_hand = self
                .steps
                .iter()
                .any(|step| matches!(step.kind, PrimacyKind::FirstHandRecord));
            return if first_hand {
                DerivedTrust {
                    tier: SourceTier::OriginalJournalism,
                    distance,
                    reason: DerivationReason::FirstHandRecord,
                }
            } else {
                DerivedTrust {
                    tier: SourceTier::OriginalFirstParty,
                    distance,
                    reason: DerivationReason::SpeaksForItself,
                }
            };
        }
        DerivedTrust {
            tier: SourceTier::IndependentCorroboration,
            distance,
            reason: DerivationReason::NamedUpstreamChain,
        }
    }

    /// The derived tier alone.
    #[must_use]
    pub fn derive_tier(&self) -> SourceTier {
        self.derive_trust().tier
    }
}

/// Read the `primacy` hops declared under a Links Notation node.
///
/// The same shape is used by an episode's `source_assessment` records and by
/// `data/seed/sources-registry.lino`, so both the audited episode and the live
/// registry derive their trust from one parser rather than two.
#[must_use]
pub fn chain_from_node(node: &LinoNode) -> PrimacyChain {
    PrimacyChain::new(
        node.children
            .iter()
            .filter(|child| child.name == "primacy")
            .map(|child| {
                PrimacyStep::new(
                    PrimacyKind::from_slug(&child.id),
                    child.find_child_value("upstream"),
                    child.find_child_value("basis"),
                )
            })
            .collect(),
    )
}

/// One source weighed by the standard: its chain, its derived trust, and the
/// tier a caller (or a seed file) claimed for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAssessment {
    /// Registry id or stable label of the source.
    pub id: String,
    /// Human label for the trace.
    pub label: String,
    /// What the source is a source *about* — trust is relative to a subject.
    pub subject: String,
    /// The hops behind the source.
    pub chain: PrimacyChain,
    /// The tier a seed file or caller declared, when one was declared. The
    /// declaration is a mirror to be checked, never the authority.
    pub asserted_tier: Option<SourceTier>,
}

impl SourceAssessment {
    /// The computed trust of this source.
    #[must_use]
    pub fn derive_trust(&self) -> DerivedTrust {
        self.chain.derive_trust()
    }

    /// Whether a declared tier disagrees with the derived one. A source with no
    /// declaration never disagrees: it simply uses what the chain computes.
    #[must_use]
    pub fn assertion_disagrees(&self) -> bool {
        self.asserted_tier
            .is_some_and(|asserted| asserted != self.derive_trust().tier)
    }

    /// Whether this source's trust rests on a derivation at all.
    #[must_use]
    pub const fn is_derived(&self) -> bool {
        !self.chain.steps.is_empty()
    }

    /// Whether the source is primary for its subject: it speaks for itself or
    /// records it first-hand. Requirement 3's "official documentation" is
    /// exactly this predicate, computed rather than listed.
    #[must_use]
    pub fn is_primary_for_subject(&self) -> bool {
        matches!(
            self.derive_trust().reason,
            DerivationReason::SpeaksForItself | DerivationReason::FirstHandRecord
        )
    }
}

/// How a conflict between two sources was resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    /// The named source wins because it stands closer to the primary record.
    Prefer {
        /// The winning source's id.
        winner: String,
        /// The losing source's id.
        loser: String,
        /// The winner's distance from the primary record.
        winner_distance: u32,
        /// The loser's distance from the primary record.
        loser_distance: u32,
    },
    /// Neither source is more primary, so the conflict stays open.
    Unresolved {
        /// The sources that tied.
        tied: Vec<String>,
        /// The distance both sources share.
        distance: u32,
    },
}

impl ConflictResolution {
    /// Stable slug for the trace.
    #[must_use]
    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Prefer { .. } => "prefer_more_primary",
            Self::Unresolved { .. } => "unresolved_equal_primacy",
        }
    }
}

/// Resolve a disagreement between two sources toward the more primary one.
///
/// Requirement 4: "when sources conflict, resolution must be derived from that
/// computed hierarchy — the more primary source wins". Sources that are equally
/// far from the primary record do not produce a winner: the conflict is reported
/// unresolved so the caller must fall back to "not confirmed and not refuted"
/// instead of picking a side by preference.
#[must_use]
pub fn resolve_conflict(left: &SourceAssessment, right: &SourceAssessment) -> ConflictResolution {
    let left_trust = left.derive_trust();
    let right_trust = right.derive_trust();
    let left_weight = left_trust.tier.weight_percent();
    let right_weight = right_trust.tier.weight_percent();
    if left_weight != right_weight {
        let (winner, loser, winner_trust, loser_trust) = if left_weight > right_weight {
            (left, right, left_trust, right_trust)
        } else {
            (right, left, right_trust, left_trust)
        };
        return ConflictResolution::Prefer {
            winner: winner.id.clone(),
            loser: loser.id.clone(),
            winner_distance: winner_trust.distance,
            loser_distance: loser_trust.distance,
        };
    }
    if left_trust.distance != right_trust.distance {
        let (winner, loser, winner_trust, loser_trust) =
            if left_trust.distance < right_trust.distance {
                (left, right, left_trust, right_trust)
            } else {
                (right, left, right_trust, left_trust)
            };
        return ConflictResolution::Prefer {
            winner: winner.id.clone(),
            loser: loser.id.clone(),
            winner_distance: winner_trust.distance,
            loser_distance: loser_trust.distance,
        };
    }
    ConflictResolution::Unresolved {
        tied: vec![left.id.clone(), right.id.clone()],
        distance: left_trust.distance,
    }
}
