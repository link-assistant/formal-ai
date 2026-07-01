//! Relative statement probability, modelled on
//! [`relative-meta-logic`](https://github.com/link-foundation/relative-meta-logic).
//!
//! This module is the deterministic, non-neural core the maintainer asked for in
//! issue #535: *"if a user writes some statement, we should increase its
//! probability in the context, until we find evidence against it. If we also have
//! supporting evidence, that means we should take into account all trusted-source
//! fact statements (original first sources) … We should ignore any unoriginal
//! content or reposting."*
//!
//! It provides a bounded [`TruthValue`] in `[0, 1]`, the relative-meta-logic
//! aggregators (min / max / average / product / probabilistic sum), a
//! source-trust taxonomy that weights *original first-party* and *original
//! journalism* sources highest while **ignoring** reposts and aggregators, and a
//! [`StatementAssessment`] that raises a statement's assumed-true prior with
//! trusted supporting evidence and lowers it with contradicting evidence.
//!
//! Everything here is pure arithmetic over caller-supplied evidence: no clocks,
//! no randomness, no network. The handler layer turns extracted statements and
//! web-search grounding into [`RelativeEvidence`] and replays the resulting
//! assessments into the append-only symbolic event log, so a statement's
//! probability is reproducible and inspectable exactly like the rest of the
//! solver trace.

use std::fmt::{self, Display, Formatter};

/// Number of fractional digits every truth value is rounded to.
///
/// relative-meta-logic reasons with decimal truth values rather than raw binary
/// floats so that identical inputs always serialise to identical strings. We
/// emulate that by snapping every stored value to a fixed decimal grid, which
/// keeps the append-only probability trace byte-for-byte reproducible across
/// platforms.
const TRUTH_VALUE_DECIMALS: u32 = 6;

/// The default assumed-true prior for a user statement before any external
/// evidence is weighed.
///
/// The maintainer's rule is to *increase* a statement's probability in context
/// until contrary evidence appears, so the prior is deliberately above the
/// `0.5` "no information" midpoint: an unchallenged statement stays likely
/// rather than being treated as a coin flip.
pub const ASSUMED_TRUE_PRIOR: f64 = 0.6;

/// A truth value bounded to the closed unit interval `[0, 1]` and snapped to a
/// fixed decimal grid for reproducible serialisation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TruthValue(f64);

impl TruthValue {
    /// The certain-false bound.
    pub const FALSE: Self = Self(0.0);
    /// The certain-true bound.
    pub const TRUE: Self = Self(1.0);
    /// The maximal-uncertainty midpoint.
    pub const UNKNOWN: Self = Self(0.5);

    /// Construct a truth value, clamping to `[0, 1]` and rounding non-finite
    /// input to [`Self::UNKNOWN`] so the type can never hold `NaN`/∞.
    #[must_use]
    pub fn new(value: f64) -> Self {
        if !value.is_finite() {
            return Self::UNKNOWN;
        }
        Self(round_decimal(value.clamp(0.0, 1.0)))
    }

    /// The underlying `[0, 1]` magnitude.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }

    /// The logical complement `1 - v`.
    #[must_use]
    pub fn negate(self) -> Self {
        Self::new(1.0 - self.0)
    }

    /// Render with the module's fixed decimal precision.
    #[must_use]
    pub fn to_decimal_string(self) -> String {
        format!("{:.*}", TRUTH_VALUE_DECIMALS as usize, self.0)
    }
}

impl Display for TruthValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_decimal_string())
    }
}

impl From<f64> for TruthValue {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

/// The relative-meta-logic aggregators over a set of truth values.
///
/// These mirror the connective families the upstream project exposes: fuzzy
/// conjunction ([`Self::Min`]), fuzzy disjunction ([`Self::Max`]), the arithmetic
/// mean ([`Self::Average`]), the independent-AND product ([`Self::Product`]), and
/// the independent-OR probabilistic sum ([`Self::ProbabilisticSum`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregator {
    /// Fuzzy conjunction — the minimum. The empty set aggregates to `1`.
    Min,
    /// Fuzzy disjunction — the maximum. The empty set aggregates to `0`.
    Max,
    /// Arithmetic mean. The empty set aggregates to [`TruthValue::UNKNOWN`].
    Average,
    /// Independent conjunction — the product `∏ vᵢ`. Empty aggregates to `1`.
    Product,
    /// Independent disjunction — the probabilistic sum `1 - ∏(1 - vᵢ)`. Empty
    /// aggregates to `0`.
    ProbabilisticSum,
}

impl Aggregator {
    /// Stable slug for the append-only trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::Max => "max",
            Self::Average => "average",
            Self::Product => "product",
            Self::ProbabilisticSum => "probabilistic_sum",
        }
    }

    /// Combine `values` under this aggregator.
    #[must_use]
    pub fn combine(self, values: &[TruthValue]) -> TruthValue {
        match self {
            Self::Min => values
                .iter()
                .map(|value| value.0)
                .fold(1.0, f64::min)
                .into(),
            Self::Max => values
                .iter()
                .map(|value| value.0)
                .fold(0.0, f64::max)
                .into(),
            Self::Average => {
                if values.is_empty() {
                    return TruthValue::UNKNOWN;
                }
                let sum: f64 = values.iter().map(|value| value.0).sum();
                TruthValue::new(sum / values.len() as f64)
            }
            Self::Product => values
                .iter()
                .map(|value| value.0)
                .fold(1.0, |acc, value| acc * value)
                .into(),
            Self::ProbabilisticSum => probabilistic_sum(values.iter().map(|value| value.0)).into(),
        }
    }
}

/// The independent-OR probabilistic sum `1 - ∏(1 - vᵢ)` over an iterator of
/// `[0, 1]` magnitudes. Monotonically non-decreasing: adding another
/// non-negative term never lowers the result, and it saturates at `1`.
fn probabilistic_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let complement = values
        .into_iter()
        .fold(1.0, |acc, value| acc * (1.0 - value.clamp(0.0, 1.0)));
    1.0 - complement
}

/// The trust tier of an evidence source, from the maintainer's "original first
/// sources" rule.
///
/// Only *original* sources carry weight. A first-party statement (a government
/// about itself, a corporation about itself, a person's own social media) and
/// original journalism (a first-hand recording, filming, or report) are trusted
/// most. Independent corroboration counts for less. Reposts, mirrors, and
/// aggregators are [`Self::Unoriginal`] and contribute **nothing** — they are
/// ignored exactly as requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTier {
    /// The subject speaking about itself: government about its own affairs, a
    /// company about its own products, a person's own social-media account.
    OriginalFirstParty,
    /// Original journalism or a primary recording: first-hand reporting, filmed
    /// or recorded footage, an eyewitness account.
    OriginalJournalism,
    /// Independent secondary corroboration that still adds signal but is not a
    /// first source.
    IndependentCorroboration,
    /// A repost, mirror, aggregator, or otherwise unoriginal copy — ignored.
    Unoriginal,
}

impl SourceTier {
    /// Stable slug for the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::OriginalFirstParty => "original_first_party",
            Self::OriginalJournalism => "original_journalism",
            Self::IndependentCorroboration => "independent_corroboration",
            Self::Unoriginal => "unoriginal",
        }
    }

    /// The trust weight in `[0, 1]` this tier lends to a piece of evidence.
    /// [`Self::Unoriginal`] is exactly `0`, so unoriginal content never moves a
    /// statement's probability.
    #[must_use]
    pub const fn weight(self) -> f64 {
        match self {
            Self::OriginalFirstParty => 1.0,
            Self::OriginalJournalism => 0.85,
            Self::IndependentCorroboration => 0.5,
            Self::Unoriginal => 0.0,
        }
    }

    /// Whether this tier is an *original first* source that should be trusted at
    /// all. `false` only for [`Self::Unoriginal`].
    #[must_use]
    pub const fn is_original(self) -> bool {
        !matches!(self, Self::Unoriginal)
    }
}

/// Which side an evidence source takes on a statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    /// The source affirms the statement.
    Supports,
    /// The source contradicts the statement.
    Contradicts,
    /// The source is on-topic but takes no side.
    Neutral,
}

impl Stance {
    /// Stable slug for the trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Supports => "supports",
            Self::Contradicts => "contradicts",
            Self::Neutral => "neutral",
        }
    }
}

/// One piece of external evidence weighed against a statement.
#[derive(Debug, Clone, PartialEq)]
pub struct RelativeEvidence {
    /// A short human/label for the source (host, outlet, handle).
    pub source_label: String,
    /// The source's trust tier.
    pub tier: SourceTier,
    /// Which side the source takes.
    pub stance: Stance,
    /// How strongly the source asserts its stance, in `[0, 1]`.
    pub strength: TruthValue,
}

impl RelativeEvidence {
    /// Construct a supporting/contradicting/neutral evidence record.
    #[must_use]
    pub fn new(
        source_label: impl Into<String>,
        tier: SourceTier,
        stance: Stance,
        strength: impl Into<TruthValue>,
    ) -> Self {
        Self {
            source_label: source_label.into(),
            tier,
            stance,
            strength: strength.into(),
        }
    }

    /// The effective mass this evidence contributes: `tier weight × asserted
    /// strength`, or `0` when the tier is unoriginal or the stance is neutral.
    /// This is the single quantity the assessment aggregates.
    #[must_use]
    pub fn effective_mass(&self) -> f64 {
        if matches!(self.stance, Stance::Neutral) || !self.tier.is_original() {
            return 0.0;
        }
        self.tier.weight() * self.strength.get()
    }

    /// Whether this evidence is ignored (contributes no mass) — an unoriginal
    /// source or a neutral stance.
    #[must_use]
    pub fn is_ignored(&self) -> bool {
        self.effective_mass() <= 0.0
    }

    /// Stable slug summary for the trace.
    #[must_use]
    pub fn trace_payload(&self) -> String {
        format!(
            "source={} tier={} stance={} strength={} mass={:.6} ignored={}",
            self.source_label,
            self.tier.slug(),
            self.stance.slug(),
            self.strength,
            self.effective_mass(),
            self.is_ignored(),
        )
    }
}

/// The relative-probability assessment of a single statement.
///
/// The posterior is computed deterministically from the assumed-true `prior`:
///
/// 1. Supporting evidence masses are combined with a probabilistic sum and used
///    to raise the prior toward `1` (again by probabilistic sum), because
///    independent trusted confirmations should reinforce, not average away, a
///    statement.
/// 2. Contradicting evidence masses are combined with a probabilistic sum and
///    attenuate the raised value multiplicatively, pulling it back toward `0`.
///
/// With no evidence the posterior equals the prior — the statement keeps its
/// elevated assumed-true probability "until we find evidence against it".
#[derive(Debug, Clone, PartialEq)]
pub struct StatementAssessment {
    /// The statement text being assessed.
    pub statement: String,
    /// The assumed-true prior before evidence.
    pub prior: TruthValue,
    /// Combined supporting mass (probabilistic sum of supporting evidence).
    pub support: TruthValue,
    /// Combined contradicting mass (probabilistic sum of contradicting evidence).
    pub contradiction: TruthValue,
    /// The resulting posterior probability.
    pub posterior: TruthValue,
    /// Evidence that was ignored as unoriginal or neutral.
    pub ignored_sources: Vec<String>,
}

impl StatementAssessment {
    /// Assess `statement` from `prior` against `evidence`.
    #[must_use]
    pub fn assess(
        statement: impl Into<String>,
        prior: TruthValue,
        evidence: &[RelativeEvidence],
    ) -> Self {
        let support = probabilistic_sum(
            evidence
                .iter()
                .filter(|item| matches!(item.stance, Stance::Supports))
                .map(RelativeEvidence::effective_mass),
        );
        let contradiction = probabilistic_sum(
            evidence
                .iter()
                .filter(|item| matches!(item.stance, Stance::Contradicts))
                .map(RelativeEvidence::effective_mass),
        );
        // Raise the prior with supporting mass, then attenuate with
        // contradicting mass.
        let raised = probabilistic_sum([prior.get(), support]);
        let posterior = raised * (1.0 - contradiction);
        let ignored_sources = evidence
            .iter()
            .filter(|item| item.is_ignored())
            .map(|item| item.source_label.clone())
            .collect();
        Self {
            statement: statement.into(),
            prior,
            support: support.into(),
            contradiction: contradiction.into(),
            posterior: posterior.into(),
            ignored_sources,
        }
    }

    /// Assess `statement` from the module default [`ASSUMED_TRUE_PRIOR`].
    #[must_use]
    pub fn assess_assumed_true(
        statement: impl Into<String>,
        evidence: &[RelativeEvidence],
    ) -> Self {
        Self::assess(statement, TruthValue::new(ASSUMED_TRUE_PRIOR), evidence)
    }

    /// Whether the posterior ended above the assumed-true midpoint of `0.5`.
    #[must_use]
    pub fn is_probable(&self) -> bool {
        self.posterior.get() > 0.5
    }

    /// A stable one-line trace payload for the append-only probability log.
    #[must_use]
    pub fn trace_payload(&self) -> String {
        format!(
            "prior={} support={} contradiction={} posterior={} ignored={}",
            self.prior,
            self.support,
            self.contradiction,
            self.posterior,
            self.ignored_sources.len(),
        )
    }
}

/// Round to the module's fixed decimal grid, matching relative-meta-logic's
/// decimal truth values so identical inputs serialise identically.
fn round_decimal(value: f64) -> f64 {
    let scale = 10f64.powi(TRUTH_VALUE_DECIMALS as i32);
    (value * scale).round() / scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truth_value_clamps_and_rounds() {
        assert_eq!(TruthValue::new(-3.0).get(), 0.0);
        assert_eq!(TruthValue::new(7.0).get(), 1.0);
        assert_eq!(TruthValue::new(f64::NAN), TruthValue::UNKNOWN);
        assert_eq!(TruthValue::new(0.123_456_789).get(), 0.123_457);
    }

    #[test]
    fn truth_value_negate_is_complement() {
        assert_eq!(TruthValue::new(0.2).negate(), TruthValue::new(0.8));
        assert_eq!(TruthValue::TRUE.negate(), TruthValue::FALSE);
    }

    #[test]
    fn aggregators_have_expected_identities() {
        let values = [TruthValue::new(0.2), TruthValue::new(0.8)];
        assert_eq!(Aggregator::Min.combine(&values), TruthValue::new(0.2));
        assert_eq!(Aggregator::Max.combine(&values), TruthValue::new(0.8));
        assert_eq!(Aggregator::Average.combine(&values), TruthValue::new(0.5));
        assert_eq!(Aggregator::Product.combine(&values), TruthValue::new(0.16));
        // 1 - (1-0.2)(1-0.8) = 1 - 0.8*0.2 = 0.84
        assert_eq!(
            Aggregator::ProbabilisticSum.combine(&values),
            TruthValue::new(0.84),
        );
    }

    #[test]
    fn aggregators_handle_the_empty_set() {
        assert_eq!(Aggregator::Min.combine(&[]), TruthValue::TRUE);
        assert_eq!(Aggregator::Max.combine(&[]), TruthValue::FALSE);
        assert_eq!(Aggregator::Average.combine(&[]), TruthValue::UNKNOWN);
        assert_eq!(Aggregator::Product.combine(&[]), TruthValue::TRUE);
        assert_eq!(Aggregator::ProbabilisticSum.combine(&[]), TruthValue::FALSE,);
    }

    #[test]
    fn unoriginal_sources_are_ignored() {
        let repost = RelativeEvidence::new(
            "aggregator.example",
            SourceTier::Unoriginal,
            Stance::Supports,
            1.0,
        );
        assert_eq!(repost.effective_mass(), 0.0);
        assert!(repost.is_ignored());
    }

    #[test]
    fn neutral_evidence_is_ignored() {
        let neutral = RelativeEvidence::new(
            "gov.example",
            SourceTier::OriginalFirstParty,
            Stance::Neutral,
            1.0,
        );
        assert!(neutral.is_ignored());
    }

    #[test]
    fn no_evidence_keeps_the_assumed_true_prior() {
        let assessment = StatementAssessment::assess_assumed_true("the sky is blue", &[]);
        assert_eq!(assessment.posterior, TruthValue::new(ASSUMED_TRUE_PRIOR));
        assert!(assessment.is_probable());
    }

    #[test]
    fn trusted_support_raises_probability() {
        let evidence = [RelativeEvidence::new(
            "gov.example",
            SourceTier::OriginalFirstParty,
            Stance::Supports,
            0.9,
        )];
        let assessment = StatementAssessment::assess_assumed_true("official policy X", &evidence);
        assert!(
            assessment.posterior.get() > ASSUMED_TRUE_PRIOR,
            "trusted first-party support should raise probability: {}",
            assessment.posterior,
        );
    }

    #[test]
    fn contradicting_original_evidence_lowers_probability() {
        let evidence = [RelativeEvidence::new(
            "original.journal",
            SourceTier::OriginalJournalism,
            Stance::Contradicts,
            0.9,
        )];
        let assessment = StatementAssessment::assess_assumed_true("disputed claim", &evidence);
        assert!(
            assessment.posterior.get() < ASSUMED_TRUE_PRIOR,
            "contradicting original evidence should lower probability: {}",
            assessment.posterior,
        );
    }

    #[test]
    fn unoriginal_reposts_do_not_move_probability() {
        let with_reposts = [
            RelativeEvidence::new("mirror.a", SourceTier::Unoriginal, Stance::Supports, 1.0),
            RelativeEvidence::new("mirror.b", SourceTier::Unoriginal, Stance::Contradicts, 1.0),
        ];
        let assessment = StatementAssessment::assess_assumed_true("viral claim", &with_reposts);
        assert_eq!(assessment.posterior, TruthValue::new(ASSUMED_TRUE_PRIOR));
        assert_eq!(assessment.ignored_sources.len(), 2);
    }

    #[test]
    fn first_party_outweighs_corroboration_for_the_same_strength() {
        let strong = [RelativeEvidence::new(
            "subject.itself",
            SourceTier::OriginalFirstParty,
            Stance::Supports,
            0.8,
        )];
        let weak = [RelativeEvidence::new(
            "second.hand",
            SourceTier::IndependentCorroboration,
            Stance::Supports,
            0.8,
        )];
        let strong_assessment = StatementAssessment::assess_assumed_true("claim", &strong);
        let weak_assessment = StatementAssessment::assess_assumed_true("claim", &weak);
        assert!(strong_assessment.posterior.get() > weak_assessment.posterior.get());
    }

    #[test]
    fn independent_support_reinforces_rather_than_averages() {
        let single = [RelativeEvidence::new(
            "a",
            SourceTier::IndependentCorroboration,
            Stance::Supports,
            0.5,
        )];
        let double = [
            RelativeEvidence::new(
                "a",
                SourceTier::IndependentCorroboration,
                Stance::Supports,
                0.5,
            ),
            RelativeEvidence::new(
                "b",
                SourceTier::IndependentCorroboration,
                Stance::Supports,
                0.5,
            ),
        ];
        let single_assessment = StatementAssessment::assess_assumed_true("claim", &single);
        let double_assessment = StatementAssessment::assess_assumed_true("claim", &double);
        assert!(double_assessment.posterior.get() > single_assessment.posterior.get());
    }

    #[test]
    fn posterior_stays_within_bounds() {
        let overwhelming = [
            RelativeEvidence::new(
                "a",
                SourceTier::OriginalFirstParty,
                Stance::Contradicts,
                1.0,
            ),
            RelativeEvidence::new(
                "b",
                SourceTier::OriginalJournalism,
                Stance::Contradicts,
                1.0,
            ),
        ];
        let assessment = StatementAssessment::assess_assumed_true("false claim", &overwhelming);
        assert!((0.0..=1.0).contains(&assessment.posterior.get()));
        assert!(assessment.posterior.get() < 0.1);
    }
}
