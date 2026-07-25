//! Merge deduplicated statements into a [`Context`], not a list.
//!
//! Issue #844's fifth requirement: the result of summarizing many sources is a
//! *context* — a links network of statements, each carrying a probability, with
//! contradictions kept as [`crate::relative_meta_logic::Stance::Contradicts`]
//! edges rather than resolved by dropping a side. That container already exists
//! ([`crate::world_model`]), so this module is a translation layer, not a new
//! engine:
//!
//! | merge concept | context representation |
//! |---|---|
//! | a merged fact | a [`crate::world_model::Statement`] whose `truth` is its posterior |
//! | a source asserting it | a [`crate::relative_meta_logic::RelativeEvidence`] record at the source's tier |
//! | an absorbed sentence | a `variant:… → statement:…` link (the merge's receipt) |
//! | provenance | a `source:… → statement:…` link |
//! | a contradiction | mutual [`Dependency::contradicts`] edges between the twins |
//!
//! Because the statements are inserted through
//! [`Context::extend_statements`], the JTMS fixpoint runs once over the finished
//! graph: a contradicted pair settles at the probability its evidence supports,
//! and both sides stay inspectable. Nothing is presented as consensus that the
//! sources disagree about.

use super::dedup::{DedupReport, SourcedStatement};
use super::importance::{rank, RankedStatement};
use crate::relative_meta_logic::TruthValue;
use crate::world_model::{Context, Dependency, Statement as WorldStatement};

/// A merged context plus the reports that explain how it was built.
#[derive(Debug, Clone)]
pub struct MergedContext {
    /// The links network: one statement per merged fact.
    pub context: Context,
    /// The deduplication report — merge links, contradictions, source list.
    pub report: DedupReport,
    /// The evidence-weighted ranking, highest weight first.
    pub ranked: Vec<RankedStatement>,
}

impl MergedContext {
    /// How many distinct sources contributed.
    #[must_use]
    pub const fn total_sources(&self) -> usize {
        self.report.sources.len()
    }

    /// The probability the context settled on for the fact whose representative
    /// text is `text`.
    #[must_use]
    pub fn probability_of(&self, text: &str) -> Option<TruthValue> {
        let id = WorldStatement::new(text).id;
        self.context.statement(&id).map(|statement| statement.truth)
    }

    /// One line per disagreement, naming both sides and who asserts each.
    ///
    /// This is what a caller shows instead of silently picking a winner.
    #[must_use]
    pub fn disagreements(&self) -> Vec<String> {
        self.report
            .contradictions
            .iter()
            .filter_map(|pair| {
                let asserted = self.ranked_by_id(&pair.asserted)?;
                let denied = self.ranked_by_id(&pair.denied)?;
                let total = self.total_sources();
                Some(format!(
                    "\"{}\" ({}) contradicts \"{}\" ({})",
                    asserted.statement.representative.text.trim_end_matches('.'),
                    asserted.evidence_summary(total),
                    denied.statement.representative.text.trim_end_matches('.'),
                    denied.evidence_summary(total),
                ))
            })
            .collect()
    }

    /// The ranked entry for a merged-statement id.
    #[must_use]
    pub fn ranked_by_id(&self, id: &str) -> Option<&RankedStatement> {
        self.ranked.iter().find(|item| item.statement.id == id)
    }

    /// The merged context rendered as indented Links Notation, so a reviewer can
    /// read the statements, their probabilities, and their edges.
    #[must_use]
    pub fn links_notation(&self) -> String {
        self.context.links_notation()
    }

    /// Render the ranked facts as prose through the existing pipeline:
    /// evidence-weighted statements → [`super::summarize`] →
    /// [`super::deformalize`].
    ///
    /// This renders *everything* the merge established. Callers presenting a
    /// summary to a user should prefer [`Self::checked_summary`], which runs the
    /// recheck gate first.
    #[must_use]
    pub fn summary(&self, config: &super::SummarizationConfig) -> String {
        let statements = super::importance::to_statements(&self.ranked, self.total_sources());
        let summarized = super::summarize(&statements, config);
        super::deformalize(&summarized)
    }

    /// Run the fact-checking path over every ranked statement.
    #[must_use]
    pub fn recheck(&self) -> super::recheck::RecheckReport {
        super::recheck::recheck(&self.ranked)
    }

    /// Render only the statements that pass the recheck gate — the
    /// "recheck before presenting" requirement, wired end to end.
    #[must_use]
    pub fn checked_summary(&self, config: &super::SummarizationConfig) -> String {
        let survivors: Vec<RankedStatement> = self
            .recheck()
            .survivors()
            .into_iter()
            .map(|item| item.ranked.clone())
            .collect();
        let statements = super::importance::to_statements(&survivors, self.total_sources());
        let summarized = super::summarize(&statements, config);
        super::deformalize(&summarized)
    }
}

/// Deduplicate `observations`, rank them by evidence, and merge them into a
/// context identified by `id`.
#[must_use]
pub fn merge_into_context(id: &str, observations: &[SourcedStatement]) -> MergedContext {
    let report = super::dedup::deduplicate(observations);
    let ranked = rank(&report);
    let mut context = Context::new(id);

    // World-statement ids are content-addressed over the text, so the twin's id
    // is known before either statement is inserted.
    let world_ids: Vec<(String, String)> = ranked
        .iter()
        .map(|item| {
            (
                item.statement.id.clone(),
                WorldStatement::new(&item.statement.representative.text).id,
            )
        })
        .collect();
    let world_id_of = |merged_id: &str| -> Option<String> {
        world_ids
            .iter()
            .find(|(merged, _)| merged == merged_id)
            .map(|(_, world)| world.clone())
    };

    let mut statements = Vec::with_capacity(ranked.len());
    for item in &ranked {
        let mut statement = WorldStatement::new(&item.statement.representative.text);
        for evidence in &item.evidence {
            statement = statement.with_evidence(evidence.clone());
        }
        for pair in &report.contradictions {
            let twin = if pair.asserted == item.statement.id {
                Some(&pair.denied)
            } else if pair.denied == item.statement.id {
                Some(&pair.asserted)
            } else {
                None
            };
            if let Some(twin_id) = twin.and_then(|twin| world_id_of(twin)) {
                statement = statement.with_dependency(Dependency::contradicts(twin_id));
            }
        }
        statements.push(statement);
    }
    let _ = context.extend_statements(statements);

    for item in &ranked {
        let Some(world_id) = world_id_of(&item.statement.id) else {
            continue;
        };
        let statement_atom = format!("statement:{world_id}");
        for variant in &item.statement.variants {
            // The merge receipt: this sentence, from this source, folded into
            // that fact. Retracting the link is how a split is reflected in the
            // network.
            context.assert_link(
                &format!(
                    "variant:{}",
                    crate::engine::stable_id("variant", &variant.text)
                ),
                &statement_atom,
            );
            context.assert_link(&format!("source:{}", variant.source), &statement_atom);
        }
    }
    context.assert_link(
        &format!("context:{id}"),
        &format!("sources:{}", report.sources.len()),
    );

    MergedContext {
        context,
        report,
        ranked,
    }
}
