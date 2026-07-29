//! Evidence-weighted importance for merged statements.
//!
//! The pre-#844 pipeline ranked statements by a static kind prior alone
//! (`super::weight_for_kind`): a `purpose` sentence always outranked a
//! `feature` sentence, however many sources said it. Issue #844 asks the rank to
//! also reflect *observed* evidence — how many sources assert the fact, and
//! whether any source denies it.
//!
//! Two numbers come out of a merged node, both integers so the ranking stays
//! bit-for-bit reproducible (`GOALS.md:54`):
//!
//! - `coverage` = distinct asserting sources × 100 / total sources. "Asserted by
//!   9 of 11 sources" is `81`.
//! - `authority` = average trust weight of the asserting sources. Eight
//!   unoriginal mirrors therefore score `0`, while one first-party source
//!   scores `100`.
//! - `agreement` = trusted asserting mass × 100 / (trusted asserting + denying
//!   mass). An unoriginal denial does not demote an original claim; a denial at
//!   the same tier yields `50`.
//!
//! Their product is the evidence score, blended with the static prior at
//! `2:1` in favour of observed evidence. The prior still contributes, but it
//! cannot let a sentence kind overrule a stronger network of trusted source
//! links — an install command repeated by the accepted answer and first-party
//! documentation can therefore outrank incidental prose:
//!
//! ```text
//! evidence = coverage × authority × agreement / 10_000
//! weight   = min(100, (prior + 2 × evidence) / 3)
//! ```
//!
//! The probability side is delegated to [`crate::relative_meta_logic`] rather
//! than reinvented: each asserting source becomes a [`Stance::Supports`]
//! evidence record at its own [`crate::relative_meta_logic::SourceTier`], each
//! denying source a [`Stance::Contradicts`] one, and
//! [`StatementAssessment::assess_assumed_true`] turns them into a posterior.
//! That is how every statement in the merged
//! context "carries a probability", and why an unoriginal mirror moves nothing.

use super::dedup::{DedupReport, MergedStatement};
use super::vocabulary;
use crate::language::Language;
use crate::relative_meta_logic::{RelativeEvidence, Stance, StatementAssessment, TruthValue};

/// Seed intents carrying the wording of an evidence report, and the placeholders
/// their templates fill (`data/seed/multilingual-responses-summarization.lino`).
const INTENT_EVIDENCE_SUMMARY: &str = "summarization_evidence_summary";
const INTENT_EVIDENCE_DENIED: &str = "summarization_evidence_denied";
const INTENT_DISPUTED_STATEMENT: &str = "summarization_disputed_statement";
const ASSERTED_PLACEHOLDER: &str = "{asserted}";
const TOTAL_PLACEHOLDER: &str = "{total}";
const DENIED_PLACEHOLDER: &str = "{denied}";
const STATEMENT_PLACEHOLDER: &str = "{statement}";
const EVIDENCE_PLACEHOLDER: &str = "{evidence}";

/// The importance breakdown of one merged statement. Every field is a
/// percentage in `0..=100`, so a caller can show the arithmetic instead of a
/// single opaque number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportanceScore {
    /// The static kind prior of the representative sentence.
    pub prior: u8,
    /// Distinct asserting sources, as a percentage of all sources seen.
    pub coverage: u8,
    /// Average trust weight of the distinct asserting sources.
    pub authority: u8,
    /// Trusted asserting mass as a percentage of asserting plus denying mass.
    pub agreement: u8,
    /// `coverage × authority × agreement / 10_000`.
    pub evidence: u8,
    /// The blended weight the summarizer ranks by.
    pub weight: u8,
}

impl ImportanceScore {
    /// Blend a static prior with observed coverage, authority, and agreement.
    #[must_use]
    pub fn blend(prior: u8, coverage: u8, authority: u8, agreement: u8) -> Self {
        let product = u32::from(coverage) * u32::from(authority) * u32::from(agreement) / 10_000;
        let evidence = u8::try_from(product.min(100)).unwrap_or(100);
        let blended = (u16::from(prior) + 2 * u16::from(evidence)) / 3;
        let weight = u8::try_from(blended.min(100)).unwrap_or(100);
        Self {
            prior,
            coverage,
            authority,
            agreement,
            evidence,
            weight,
        }
    }
}

/// A merged statement with its evidence-weighted score and probability.
#[derive(Debug, Clone)]
pub struct RankedStatement {
    /// The merged fact.
    pub statement: MergedStatement,
    /// Its importance breakdown.
    pub score: ImportanceScore,
    /// The posterior probability from [`crate::relative_meta_logic`].
    pub probability: TruthValue,
    /// Sources that deny this fact (the contradicting twin's sources).
    pub denied_by: Vec<String>,
    /// The evidence records the probability was computed from: one supporting
    /// record per asserting source, one contradicting record per denying source.
    /// Carried along so [`super::context`] can attach the same evidence to the
    /// world-model statement instead of rebuilding it.
    pub evidence: Vec<RelativeEvidence>,
}

impl RankedStatement {
    /// Human-readable evidence summary in English: `"asserted by 9 of 11
    /// sources"`, with `", denied by 2"` appended when the fact is contested.
    #[must_use]
    pub fn evidence_summary(&self, total_sources: usize) -> String {
        self.evidence_summary_in(total_sources, Language::English)
    }

    /// The same summary in `language`, worded by the seed rather than by this
    /// module (R379): the counts are the only thing computed here.
    #[must_use]
    pub fn evidence_summary_in(&self, total_sources: usize, language: Language) -> String {
        let denied = if self.denied_by.is_empty() {
            String::new()
        } else {
            vocabulary::rendered_response(
                INTENT_EVIDENCE_DENIED,
                language,
                &[(DENIED_PLACEHOLDER, &self.denied_by.len().to_string())],
            )
        };
        vocabulary::rendered_response(
            INTENT_EVIDENCE_SUMMARY,
            language,
            &[
                (
                    ASSERTED_PLACEHOLDER,
                    &self.statement.source_count().to_string(),
                ),
                (TOTAL_PLACEHOLDER, &total_sources.to_string()),
                (DENIED_PLACEHOLDER, &denied),
            ],
        )
    }

    /// Is this fact contested by at least one source?
    #[must_use]
    pub const fn is_contested(&self) -> bool {
        !self.denied_by.is_empty()
    }
}

/// Score and rank every node of `report`.
///
/// Ordering is by weight descending, then by source count descending, then by
/// signature key ascending — a total order over the nodes, so two runs over the
/// same evidence produce the same list, whatever order the sources arrived in.
/// Denied nodes stay in the list: a contradiction is reported as disagreement,
/// not silently dropped.
#[must_use]
pub fn rank(report: &DedupReport) -> Vec<RankedStatement> {
    let total = report.sources.len();
    let mut ranked: Vec<RankedStatement> = report
        .statements
        .iter()
        .map(|node| score(node, report, total))
        .collect();
    ranked.sort_by(|left, right| {
        right
            .score
            .weight
            .cmp(&left.score.weight)
            .then_with(|| {
                right
                    .statement
                    .source_count()
                    .cmp(&left.statement.source_count())
            })
            .then_with(|| {
                left.statement
                    .signature
                    .key()
                    .cmp(&right.statement.signature.key())
            })
    });
    ranked
}

/// Score one node against the whole report.
#[must_use]
pub fn score(
    node: &MergedStatement,
    report: &DedupReport,
    total_sources: usize,
) -> RankedStatement {
    let denier = report
        .statements
        .iter()
        .find(|other| other.signature == node.signature.negated());
    let denied_by: Vec<String> = denier
        .map(|twin| {
            twin.sources()
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    let asserting = node.source_count();
    let coverage = percentage(asserting, total_sources);
    let supporting_mass = trust_mass(node);
    let denying_mass = denier.map_or(0, trust_mass);
    let authority = percentage(supporting_mass, asserting.saturating_mul(100));
    let agreement = percentage(
        supporting_mass,
        supporting_mass.saturating_add(denying_mass),
    );
    let score = ImportanceScore::blend(node.prior(), coverage, authority, agreement);

    let mut evidence: Vec<RelativeEvidence> = node
        .evidence()
        .into_iter()
        .map(|(source, tier)| {
            RelativeEvidence::new(source, tier, Stance::Supports, TruthValue::TRUE)
        })
        .collect();
    if let Some(twin) = denier {
        for (source, tier) in twin.evidence() {
            evidence.push(RelativeEvidence::new(
                source,
                tier,
                Stance::Contradicts,
                TruthValue::TRUE,
            ));
        }
    }
    let assessment =
        StatementAssessment::assess_assumed_true(node.representative.text.clone(), &evidence);
    RankedStatement {
        statement: node.clone(),
        score,
        probability: assessment.posterior,
        denied_by,
        evidence,
    }
}

/// Sum the integer trust weights of the distinct sources asserting `node`.
fn trust_mass(node: &MergedStatement) -> usize {
    node.evidence()
        .into_iter()
        .map(|(_, tier)| usize::from(tier.weight_percent()))
        .sum()
}

/// `part × 100 / whole`, clamped to `100`, with an empty `whole` scoring `0`.
fn percentage(part: usize, whole: usize) -> u8 {
    if whole == 0 {
        return 0;
    }
    let value = part.saturating_mul(100) / whole;
    u8::try_from(value.min(100)).unwrap_or(100)
}

/// Turn a ranked list back into plain [`super::Statement`]s carrying the
/// evidence-weighted weight, so the rest of the pipeline ([`super::summarize`],
/// [`super::deformalize`]) can consume it unchanged.
///
/// A denied statement is rendered with its disagreement noted, because dropping
/// one side of a contradiction would report a consensus that does not exist.
#[must_use]
pub fn to_statements(ranked: &[RankedStatement], total_sources: usize) -> Vec<super::Statement> {
    to_statements_in(ranked, total_sources, Language::English)
}

/// [`to_statements`] with the disagreement note worded in `language`, from the
/// seed (R379).
#[must_use]
pub fn to_statements_in(
    ranked: &[RankedStatement],
    total_sources: usize,
    language: Language,
) -> Vec<super::Statement> {
    ranked
        .iter()
        .map(|item| {
            let mut statement = item.statement.representative.clone();
            statement.weight = item.score.weight;
            if item.is_contested() {
                statement.text = vocabulary::rendered_response(
                    INTENT_DISPUTED_STATEMENT,
                    language,
                    &[
                        (STATEMENT_PLACEHOLDER, statement.text.trim_end_matches('.')),
                        (
                            EVIDENCE_PLACEHOLDER,
                            &item.evidence_summary_in(total_sources, language),
                        ),
                    ],
                );
            }
            statement
        })
        .collect()
}
