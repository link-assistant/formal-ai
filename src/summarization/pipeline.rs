//! Whole multi-source execution over capture, context, fact-check, and learning.
//!
//! This is the production composition required by issue #844. It deliberately
//! owns no second implementation of retrieval, probability, or proof:
//!
//! 1. [`super::gathering::execute_captured_gathering`] obtains replayable bytes;
//! 2. [`super::context::merge_into_formal_context`] deduplicates those bytes into
//!    one named world-model context;
//! 3. [`crate::fact_checking::FactChecker`] runs the disproof-first JTMS audit;
//! 4. presentation keeps only claims with admitted support above uncertainty;
//! 5. [`MultiSourceSummaryExecution::learning_proposal`] projects the exact
//!    observations and derived audit for human-gated auto-learning.
//!
//! A capture failure cannot enter steps 2–5 as evidence because failed URLs are
//! represented only by `CapturedGatheringFailure`.

use std::collections::BTreeSet;

use super::context::{merge_into_formal_context, MergedContext};
use super::gathering::{
    execute_captured_gathering, CapturedGatheringReport, CapturedSourceMetadata, GatheringPlan,
};
use super::importance::RankedStatement;
use crate::event_log::EventLog;
use crate::fact_checking::{ContextAudit, FactChecker, StatementVerification};
use crate::formal_system::FormalSystem;
use crate::links_format::format_lino_record;
use crate::relative_meta_logic::{Stance, TruthValue};
use crate::source_fetch::{CachedSourceClient, SourceCapture, SourceTransport};
use crate::world_model::Statement as WorldStatement;

/// Exact captures and every deterministic derivation needed to present them as
/// one checked context.
#[derive(Debug, Clone)]
pub struct MultiSourceSummaryExecution {
    /// Recursive exact-capture gathering and its diagnostics.
    pub gathering: CapturedGatheringReport,
    /// Deduplicated, evidence-ranked, named symbolic context.
    pub merged: MergedContext,
    /// Disproof-first audit of the merged context.
    pub audit: ContextAudit,
    /// Context statement ids admitted for presentation.
    pub presentable_statement_ids: Vec<String>,
    /// Context statement ids retained but withheld from presentation.
    pub withheld_statement_ids: Vec<String>,
}

impl MultiSourceSummaryExecution {
    /// Render only statements admitted by the production fact-checking gate.
    #[must_use]
    pub fn checked_summary(&self, config: &super::SummarizationConfig) -> String {
        let admitted = self
            .presentable_statement_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let survivors = self
            .merged
            .ranked
            .iter()
            .filter(|item| admitted.contains(&world_id(item)))
            .cloned()
            .collect::<Vec<_>>();
        self.merged.render(&survivors, config)
    }

    /// Record exact captures and the fact-check boundary in the common log.
    pub fn record(&self, log: &mut EventLog) {
        self.gathering.record(log);
        log.append(
            "fact_check:context",
            format!(
                "context={} formal_system={} statements={} presentable={} withheld={}",
                self.audit.context_id,
                self.audit.formal_system_id,
                self.audit.statements.len(),
                self.presentable_statement_ids.len(),
                self.withheld_statement_ids.len(),
            ),
        );
    }

    /// Deterministic, review-gated learning proposal for the whole execution.
    ///
    /// This combines exact capture observations with merge, contradiction, and
    /// fact-check decisions. It does not mutate seed data or durable memory.
    #[must_use]
    pub fn learning_proposal(&self) -> String {
        let presentable = self
            .presentable_statement_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut records = vec![
            self.gathering.learning_proposal(),
            format_lino_record(
                "multi_source_statement_merge",
                &[
                    ("context", self.audit.context_id.clone()),
                    ("formal_system", self.audit.formal_system_name.clone()),
                    ("formal_system_id", self.audit.formal_system_id.clone()),
                    ("sources", self.merged.total_sources().to_string()),
                    ("statements", self.audit.statements.len().to_string()),
                    (
                        "contradictions",
                        self.merged.report.contradictions.len().to_string(),
                    ),
                    ("decision", String::from("awaiting_human_review")),
                ],
            ),
        ];
        for statement in &self.audit.statements {
            records.push(format_lino_record(
                "merged_statement",
                &[
                    ("id", statement.statement_id.clone()),
                    ("text", statement.text.clone()),
                    ("probability", statement.probability.to_decimal_string()),
                    (
                        "probability_basis",
                        statement.probability_basis.slug().to_owned(),
                    ),
                    (
                        "presentation",
                        if presentable.contains(&statement.statement_id) {
                            String::from("presentable")
                        } else {
                            String::from("withheld")
                        },
                    ),
                ],
            ));
        }
        for pair in &self.merged.report.contradictions {
            records.push(format_lino_record(
                "contradiction",
                &[
                    ("asserted", pair.asserted.clone()),
                    ("denied", pair.denied.clone()),
                    ("terms", pair.terms.join("|")),
                ],
            ));
        }
        records.push(self.audit.links_notation());
        records.join("\n")
    }
}

/// Execute issue #844's complete production path.
///
/// The classifier may derive source tier, supplied attributes, and linked URLs
/// only from the provided exact capture. Live retrieval remains opt-in through
/// [`CachedSourceClient`], and offline replay uses the same operation.
pub fn execute_multi_source_summary<T, C>(
    context_id: &str,
    formal_system: FormalSystem,
    plan: &GatheringPlan,
    client: &CachedSourceClient<T>,
    checker: FactChecker,
    classify: C,
) -> MultiSourceSummaryExecution
where
    T: SourceTransport,
    C: Fn(&SourceCapture) -> CapturedSourceMetadata,
{
    let gathering = execute_captured_gathering(plan, client, classify);
    let mut merged =
        merge_into_formal_context(context_id, formal_system, &gathering.report.observations);
    let audit = checker.audit_context(&mut merged.context);
    for ranked in &mut merged.ranked {
        if let Some(statement) = audit.statement(&world_id(ranked)) {
            ranked.probability = statement.probability;
        }
    }
    let (presentable, withheld): (Vec<_>, Vec<_>) = audit
        .statements
        .iter()
        .partition(|statement| is_presentable(statement));
    let presentable_statement_ids = presentable
        .into_iter()
        .map(|statement| statement.statement_id.clone())
        .collect();
    let withheld_statement_ids = withheld
        .into_iter()
        .map(|statement| statement.statement_id.clone())
        .collect();
    MultiSourceSummaryExecution {
        gathering,
        merged,
        audit,
        presentable_statement_ids,
        withheld_statement_ids,
    }
}

fn is_presentable(statement: &StatementVerification) -> bool {
    statement.probability.get() > TruthValue::UNKNOWN.get()
        && statement.evidence.iter().any(|evidence| {
            !evidence.ignored
                && evidence.stance == Stance::Supports
                && evidence.strength.get() > 0.0
                && evidence.tier.weight() > 0.0
        })
}

fn world_id(ranked: &RankedStatement) -> String {
    WorldStatement::new(&ranked.statement.representative.text).id
}
