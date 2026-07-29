//! Lightweight, capture-independent preflight for merged statements.
//!
//! This compatibility path predates the production capture boundary. It turns
//! an already-ranked fact into a [`StatementPlan`] and can conservatively hide
//! unsupported claims when no [`crate::source_fetch::SourceCapture`] or named
//! context is available. It does not perform proof search and must not be called
//! a production fact check.
//!
//! Production multi-source execution goes through
//! [`super::pipeline::execute_multi_source_summary`], which composes exact
//! captures, a named [`crate::formal_system::FormalSystem`], and
//! [`crate::fact_checking::FactChecker`] before presentation.
//!
//! The verdict is read off the assessment, which keeps the gate deterministic
//! (`GOALS.md:54`) and free of any judgement this crate cannot justify:
//!
//! | evidence | verdict | presented? |
//! |---|---|---|
//! | no trusted supporting mass (only [`crate::relative_meta_logic::SourceTier::Unoriginal`] mirrors) | [`Verdict::Unsupported`] | no |
//! | contradicted down to or below the `0.5` midpoint | [`Verdict::Refuted`] | no |
//! | contradicted but still probable | [`Verdict::Contested`] | yes, as disagreement |
//! | supported, uncontradicted | [`Verdict::Confirmed`] | yes |
//!
//! Withholding is not deletion: a withheld statement stays in the report with
//! its verdict and its grounding query, and stays in the merged context with
//! both sides of its contradiction. The gate decides what is *presented*, never
//! what is *known* — which is why a refuted claim can be re-presented unchanged
//! once new evidence flips its posterior.

use super::importance::RankedStatement;
use crate::statement_verification::StatementPlan;

/// The outcome of rechecking one statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Trusted sources support it and none contradict it.
    Confirmed,
    /// Trusted sources support it and others contradict it, but it is still
    /// more likely than not. Presented *as a disagreement*.
    Contested,
    /// Contradicting evidence pulled it to or below the midpoint.
    Refuted,
    /// Nothing trusted supports it: every asserting source is unoriginal, so the
    /// evidence moved the prior not at all.
    Unsupported,
}

impl Verdict {
    /// May a statement with this verdict be shown?
    #[must_use]
    pub const fn is_presentable(self) -> bool {
        matches!(self, Self::Confirmed | Self::Contested)
    }

    /// Stable slug for traces and tests.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Contested => "contested",
            Self::Refuted => "refuted",
            Self::Unsupported => "unsupported",
        }
    }

    /// Why the statement got this verdict, in one clause.
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Confirmed => "supported by trusted sources, uncontradicted",
            Self::Contested => "sources disagree, still probable",
            Self::Refuted => "contradicting evidence outweighs support",
            Self::Unsupported => "no trusted source asserts it",
        }
    }
}

/// One ranked statement after the compatibility preflight ran over it.
#[derive(Debug, Clone)]
pub struct RecheckedStatement {
    /// The ranked, merged statement.
    pub ranked: RankedStatement,
    /// The verification plan: grounding query plus assessment.
    pub plan: StatementPlan,
    /// The verdict read off the assessment.
    pub verdict: Verdict,
}

impl RecheckedStatement {
    /// The representative sentence.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.plan.statement
    }

    /// The query that would re-ground this statement against live sources.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.plan.query
    }

    /// A one-line trace: verdict, posterior, and evidence counts.
    ///
    /// Only `name=value` fields, like every other trace payload in the crate, so
    /// the line is a machine record rather than a sentence to translate (R379).
    #[must_use]
    pub fn trace_payload(&self) -> String {
        [
            format!("verdict={}", self.verdict.slug()),
            self.plan.assessment.trace_payload(),
            format!("sources={}", self.ranked.statement.source_count()),
            format!("denied={}", self.ranked.denied_by.len()),
        ]
        .join(" ")
    }
}

/// The result of rechecking a whole ranked list.
#[derive(Debug, Clone, Default)]
pub struct RecheckReport {
    /// Every statement, in ranked order, with its verdict.
    pub checked: Vec<RecheckedStatement>,
}

impl RecheckReport {
    /// The statements cleared for presentation, in ranked order.
    #[must_use]
    pub fn survivors(&self) -> Vec<&RecheckedStatement> {
        self.checked
            .iter()
            .filter(|item| item.verdict.is_presentable())
            .collect()
    }

    /// The statements the gate withheld, in ranked order.
    #[must_use]
    pub fn withheld(&self) -> Vec<&RecheckedStatement> {
        self.checked
            .iter()
            .filter(|item| !item.verdict.is_presentable())
            .collect()
    }

    /// Every grounding query, in ranked order — what a caller with network
    /// access would fetch to re-verify the summary.
    #[must_use]
    pub fn queries(&self) -> Vec<&str> {
        self.checked.iter().map(RecheckedStatement::query).collect()
    }

    /// A deterministic, byte-comparable render of the whole gate.
    #[must_use]
    pub fn trace(&self) -> String {
        self.checked
            .iter()
            .map(RecheckedStatement::trace_payload)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Run the capture-independent compatibility preflight over `ranked`.
///
/// The evidence handed to each plan is the evidence the merge already collected
/// ([`RankedStatement::evidence`]): one supporting record per asserting source at
/// its own tier, one contradicting record per denying source. No new evidence is
/// invented, and none is dropped.
#[must_use]
pub fn recheck(ranked: &[RankedStatement]) -> RecheckReport {
    let checked = ranked
        .iter()
        .map(|item| {
            let plan =
                StatementPlan::new(item.statement.representative.text.clone(), &item.evidence);
            RecheckedStatement {
                ranked: item.clone(),
                verdict: verdict_for(&plan),
                plan,
            }
        })
        .collect();
    RecheckReport { checked }
}

/// Read the verdict off a plan's assessment.
fn verdict_for(plan: &StatementPlan) -> Verdict {
    let assessment = &plan.assessment;
    if assessment.support.get() <= 0.0 {
        // Every supporting source was unoriginal or neutral, so the posterior is
        // still the bare prior: the merge found repetition, not evidence.
        return Verdict::Unsupported;
    }
    if assessment.contradiction.get() <= 0.0 {
        return Verdict::Confirmed;
    }
    if assessment.is_probable() {
        Verdict::Contested
    } else {
        Verdict::Refuted
    }
}
