//! Deterministic, disproof-first statement verification — issue #845.
//!
//! This module joins the symbolic proof engine, relative-meta-logic evidence,
//! and world-model JTMS into one replayable fact-checking operation. It never
//! invents sources or observations: proof results become explicitly labelled
//! symbolic evidence, caller-supplied source tiers retain their existing
//! weights, and unsupported statements remain on their declared prior.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::proof_engine::{attempt_proof, ProofMethod, ProofOutcome};
use crate::relative_meta_logic::{RelativeEvidence, SourceTier, Stance, TruthValue};
use crate::solver::SolverConfig;
use crate::world_model::{
    Context, ContextAccessError, GeneralContextPermission, RecalculationReport, WorldModel,
};

/// Which world-model boundary a fact-check operation may inspect.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AuditScope {
    /// Only statements accumulated in the current dialogue.
    #[default]
    CurrentDialogue,
    /// The shared general-memory context, after explicit permission.
    GeneralMemory,
}

/// Why a fact-checking operation could not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactCheckError {
    /// General-memory access was requested without an explicit grant.
    PermissionRequired,
    /// The supplied grant was not issued by this world model.
    InvalidPermission,
    /// The requested statement does not exist in the context.
    UnknownStatement(String),
}

impl From<ContextAccessError> for FactCheckError {
    fn from(error: ContextAccessError) -> Self {
        match error {
            ContextAccessError::PermissionDenied => Self::PermissionRequired,
            ContextAccessError::InvalidPermission => Self::InvalidPermission,
        }
    }
}

/// Whether a reported probability has admitted evidence behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbabilityBasis {
    /// The value comes only from the statement's prior.
    PriorOnly,
    /// Source evidence, proof evidence, or a dependency contributed.
    EvidenceWeighted,
}

/// Ordered stages in recursive disproof-first verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefutationStage {
    /// Try to refute the statement directly.
    DisproveStatement,
    /// If that fails, try to refute its symbolic negation.
    DisproveNegation,
    /// If neither side discharges, inspect dependent statements.
    Decompose,
    /// Stop recursion at the caller-configured depth.
    DepthBound,
}

/// Result of one stage in the refutation trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefutationOutcome {
    /// A proof or counterexample refuted the proposition considered by the stage.
    Refuted,
    /// A proof established that the proposition could not be refuted.
    Unrefuted,
    /// The symbolic engines did not discharge either side.
    Inconclusive,
}

/// One replayable proof/refutation attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefutationAttempt {
    /// Zero-based dependency recursion depth.
    pub depth: u8,
    /// Statement considered at this stage.
    pub statement_id: String,
    /// Operation attempted.
    pub stage: RefutationStage,
    /// Symbolic result.
    pub outcome: RefutationOutcome,
    /// Proof technique used when the proof engine discharged a side.
    pub proof_method: Option<ProofMethod>,
    /// Concrete counterexample when refutation produced one.
    pub counterexample: Option<String>,
}

/// Source evidence as it was admitted or rejected by the audit.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTrace {
    /// Caller- or proof-supplied stable label.
    pub source_label: String,
    /// Relative-meta-logic source tier.
    pub tier: SourceTier,
    /// Whether the source supports or contradicts the statement.
    pub stance: Stance,
    /// Declared source strength.
    pub strength: TruthValue,
    /// Whether relative-meta-logic assigns this evidence zero mass.
    pub ignored: bool,
    /// Whether the source was removed as a known placeholder/fabrication.
    pub rejected_as_fabricated: bool,
}

/// Complete verification record for one statement.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementVerification {
    /// Statement id in the context.
    pub statement_id: String,
    /// Statement text.
    pub text: String,
    /// Name of the formal system in which this value is interpreted.
    pub formal_system_name: String,
    /// Stable content id of that complete formal system.
    pub formal_system_id: String,
    /// Final relative probability after the JTMS fixpoint.
    pub probability: TruthValue,
    /// Whether the probability is prior-only or evidence-weighted.
    pub probability_basis: ProbabilityBasis,
    /// First counterexample for this statement, when available.
    pub counterexample: Option<String>,
    /// Admitted, ignored, and explicitly rejected evidence.
    pub evidence: Vec<EvidenceTrace>,
    /// Ordered recursive refutation attempts rooted at this statement.
    pub refutation_trace: Vec<RefutationAttempt>,
}

/// A whole-context audit and its JTMS recalculation trace.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextAudit {
    /// Boundary inspected by this operation.
    pub scope: AuditScope,
    /// Context id.
    pub context_id: String,
    /// Named formal system.
    pub formal_system_name: String,
    /// Stable content id of the formal system.
    pub formal_system_id: String,
    /// Every statement in deterministic id order.
    pub statements: Vec<StatementVerification>,
    /// Dependency links consulted by the final batch recalculation.
    pub recalculation: RecalculationReport,
}

impl ContextAudit {
    /// Find a statement verification by context statement id.
    #[must_use]
    pub fn statement(&self, id: &str) -> Option<&StatementVerification> {
        self.statements
            .iter()
            .find(|statement| statement.statement_id == id)
    }

    /// Deterministic Links Notation suitable for storage and replay comparison.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        let audit_id = format!("audit:{}", self.context_id);
        push_link(&mut out, &audit_id, "scope", scope_slug(self.scope));
        push_link(&mut out, &audit_id, "formal_system", &self.formal_system_id);
        push_link(
            &mut out,
            &self.formal_system_id,
            "name",
            &self.formal_system_name,
        );
        for statement in &self.statements {
            let id = format!("statement:{}", statement.statement_id);
            push_link(&mut out, &audit_id, "includes", &id);
            push_link(&mut out, &id, "text", &statement.text);
            push_link(
                &mut out,
                &id,
                "probability",
                &statement.probability.to_decimal_string(),
            );
            push_link(
                &mut out,
                &id,
                "basis",
                basis_slug(statement.probability_basis),
            );
            if let Some(counterexample) = &statement.counterexample {
                push_link(&mut out, &id, "counterexample", counterexample);
            }
            for evidence in &statement.evidence {
                let evidence_id = format!(
                    "evidence:{}:{}",
                    statement.statement_id, evidence.source_label
                );
                push_link(&mut out, &id, "evidence", &evidence_id);
                push_link(&mut out, &evidence_id, "source", &evidence.source_label);
                push_link(&mut out, &evidence_id, "tier", evidence.tier.slug());
                push_link(&mut out, &evidence_id, "status", evidence_status(evidence));
            }
            for (index, attempt) in statement.refutation_trace.iter().enumerate() {
                let attempt_id = format!("attempt:{}:{index}", statement.statement_id);
                push_link(&mut out, &id, "attempt", &attempt_id);
                push_link(&mut out, &attempt_id, "stage", stage_slug(attempt.stage));
                push_link(
                    &mut out,
                    &attempt_id,
                    "outcome",
                    outcome_slug(attempt.outcome),
                );
                push_link(&mut out, &attempt_id, "subject", &attempt.statement_id);
            }
        }
        for (index, link) in self.recalculation.checked_links.iter().enumerate() {
            let link_id = format!("recalculation:{index}");
            push_link(&mut out, &audit_id, "recalculated", &link_id);
            push_link(&mut out, &link_id, "statement", &link.statement_id);
            push_link(&mut out, &link_id, "depends_on", &link.depends_on);
        }
        out
    }
}

/// Deterministic fact-checking orchestrator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactChecker {
    max_refutation_depth: u8,
}

impl FactChecker {
    /// Reuse the solver's existing recursion bound instead of adding an
    /// independently drifting configuration surface.
    #[must_use]
    pub const fn from_solver_config(config: SolverConfig) -> Self {
        Self {
            max_refutation_depth: config.max_decomposition_depth,
        }
    }

    /// Verify one statement and recursively inspect its dependencies.
    pub fn verify_statement(
        &self,
        context: &mut Context,
        statement_id: &str,
    ) -> Result<StatementVerification, FactCheckError> {
        if context.statement(statement_id).is_none() {
            return Err(FactCheckError::UnknownStatement(statement_id.to_owned()));
        }
        let rejected = collect_rejected_evidence(context);
        let mut generated = BTreeMap::new();
        let mut counterexamples = BTreeMap::new();
        let mut trace = Vec::new();
        let mut visited = BTreeSet::new();
        self.refute_recursive(
            context,
            statement_id,
            0,
            &mut visited,
            &mut generated,
            &mut counterexamples,
            &mut trace,
        );
        let rejected_labels = rejected_labels(&rejected);
        let _ = context.apply_fact_check_evidence(&generated, &rejected_labels);
        Ok(build_statement_report(
            context,
            statement_id,
            trace,
            counterexamples.get(statement_id).cloned(),
            rejected.get(statement_id).map_or(&[][..], Vec::as_slice),
        ))
    }

    /// Verify every statement in one context and recalculate dependencies once.
    #[must_use]
    pub fn audit_context(&self, context: &mut Context) -> ContextAudit {
        self.audit_context_with_scope(context, AuditScope::CurrentDialogue)
    }

    /// Audit the current dialogue by default, or permissioned general memory.
    pub fn audit_world_model(
        &self,
        world_model: &mut WorldModel,
        scope: AuditScope,
        permission: Option<&GeneralContextPermission>,
    ) -> Result<ContextAudit, FactCheckError> {
        match scope {
            AuditScope::CurrentDialogue => {
                Ok(self.audit_context_with_scope(&mut world_model.current, scope))
            }
            AuditScope::GeneralMemory => {
                let permission = permission.ok_or(FactCheckError::PermissionRequired)?;
                let context = world_model.general_context_for_audit(permission)?;
                Ok(self.audit_context_with_scope(context, scope))
            }
        }
    }

    fn audit_context_with_scope(self, context: &mut Context, scope: AuditScope) -> ContextAudit {
        let ids = context.statements().keys().cloned().collect::<Vec<_>>();
        let rejected = collect_rejected_evidence(context);
        let mut generated = BTreeMap::new();
        let mut counterexamples = BTreeMap::new();
        let mut traces = BTreeMap::new();
        for id in &ids {
            let mut trace = Vec::new();
            self.refute_recursive(
                context,
                id,
                0,
                &mut BTreeSet::new(),
                &mut generated,
                &mut counterexamples,
                &mut trace,
            );
            traces.insert(id.clone(), trace);
        }
        let recalculation =
            context.apply_fact_check_evidence(&generated, &rejected_labels(&rejected));
        let statements = ids
            .iter()
            .map(|id| {
                build_statement_report(
                    context,
                    id,
                    traces.remove(id).unwrap_or_default(),
                    counterexamples.get(id).cloned(),
                    rejected.get(id).map_or(&[][..], Vec::as_slice),
                )
            })
            .collect();
        ContextAudit {
            scope,
            context_id: context.id.clone(),
            formal_system_name: context.formal_system().name.clone(),
            formal_system_id: context.formal_system().id(),
            statements,
            recalculation,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn refute_recursive(
        self,
        context: &Context,
        statement_id: &str,
        depth: u8,
        visited: &mut BTreeSet<String>,
        generated: &mut BTreeMap<String, Vec<RelativeEvidence>>,
        counterexamples: &mut BTreeMap<String, String>,
        trace: &mut Vec<RefutationAttempt>,
    ) {
        if !visited.insert(statement_id.to_owned()) {
            return;
        }
        let Some(statement) = context.statement(statement_id) else {
            return;
        };
        let direct = attempt_proof(
            &statement.text,
            &statement.text.to_lowercase(),
            "en",
            false,
            false,
        );
        let direct_proof_method = match &direct {
            ProofOutcome::Disproven {
                counterexample,
                method,
                ..
            } => {
                trace.push(attempt(
                    depth,
                    statement_id,
                    RefutationStage::DisproveStatement,
                    RefutationOutcome::Refuted,
                    Some(*method),
                    Some(counterexample.clone()),
                ));
                counterexamples.insert(statement_id.to_owned(), counterexample.clone());
                add_proof_evidence(
                    generated,
                    statement_id,
                    format!("refutation:{}", method.slug()),
                    Stance::Contradicts,
                );
                return;
            }
            ProofOutcome::Proven { proof } => {
                trace.push(attempt(
                    depth,
                    statement_id,
                    RefutationStage::DisproveStatement,
                    RefutationOutcome::Unrefuted,
                    Some(proof.method),
                    None,
                ));
                Some(proof.method)
            }
            ProofOutcome::PartialPlan { .. } | ProofOutcome::Inconclusive { .. } => {
                trace.push(attempt(
                    depth,
                    statement_id,
                    RefutationStage::DisproveStatement,
                    RefutationOutcome::Inconclusive,
                    direct.method(),
                    None,
                ));
                None
            }
        };

        let negated = format!("not ({})", statement.text);
        let negation = attempt_proof(&negated, &negated.to_lowercase(), "en", false, false);
        match &negation {
            ProofOutcome::Proven { proof } => {
                trace.push(attempt(
                    depth,
                    statement_id,
                    RefutationStage::DisproveNegation,
                    RefutationOutcome::Unrefuted,
                    Some(proof.method),
                    Some(proof.conclusion.clone()),
                ));
                counterexamples.insert(statement_id.to_owned(), proof.conclusion.clone());
                add_proof_evidence(
                    generated,
                    statement_id,
                    format!("negation:{}", proof.method.slug()),
                    Stance::Contradicts,
                );
                return;
            }
            ProofOutcome::Disproven {
                counterexample,
                method,
                ..
            } => {
                trace.push(attempt(
                    depth,
                    statement_id,
                    RefutationStage::DisproveNegation,
                    RefutationOutcome::Refuted,
                    Some(*method),
                    Some(counterexample.clone()),
                ));
                add_proof_evidence(
                    generated,
                    statement_id,
                    format!("negation_refuted:{}", method.slug()),
                    Stance::Supports,
                );
                return;
            }
            ProofOutcome::PartialPlan { .. } | ProofOutcome::Inconclusive { .. } => {
                trace.push(attempt(
                    depth,
                    statement_id,
                    RefutationStage::DisproveNegation,
                    RefutationOutcome::Inconclusive,
                    negation.method(),
                    None,
                ));
            }
        }

        if let Some(method) = direct_proof_method {
            add_proof_evidence(
                generated,
                statement_id,
                format!("proof:{}", method.slug()),
                Stance::Supports,
            );
            return;
        }

        if depth >= self.max_refutation_depth {
            trace.push(attempt(
                depth,
                statement_id,
                RefutationStage::DepthBound,
                RefutationOutcome::Inconclusive,
                None,
                None,
            ));
            return;
        }
        if statement.dependencies.is_empty() {
            return;
        }
        trace.push(attempt(
            depth,
            statement_id,
            RefutationStage::Decompose,
            RefutationOutcome::Inconclusive,
            None,
            None,
        ));
        for dependency in &statement.dependencies {
            self.refute_recursive(
                context,
                &dependency.on,
                depth.saturating_add(1),
                visited,
                generated,
                counterexamples,
                trace,
            );
        }
    }
}

fn attempt(
    depth: u8,
    statement_id: &str,
    stage: RefutationStage,
    outcome: RefutationOutcome,
    proof_method: Option<ProofMethod>,
    counterexample: Option<String>,
) -> RefutationAttempt {
    RefutationAttempt {
        depth,
        statement_id: statement_id.to_owned(),
        stage,
        outcome,
        proof_method,
        counterexample,
    }
}

fn add_proof_evidence(
    generated: &mut BTreeMap<String, Vec<RelativeEvidence>>,
    statement_id: &str,
    label: String,
    stance: Stance,
) {
    generated
        .entry(statement_id.to_owned())
        .or_default()
        .push(RelativeEvidence::new(
            label,
            SourceTier::OriginalFirstParty,
            stance,
            TruthValue::TRUE,
        ));
}

fn collect_rejected_evidence(context: &Context) -> BTreeMap<String, Vec<RelativeEvidence>> {
    context
        .statements()
        .iter()
        .filter_map(|(id, statement)| {
            let rejected = statement
                .evidence
                .iter()
                .filter(|evidence| is_placeholder_source(&evidence.source_label))
                .cloned()
                .collect::<Vec<_>>();
            (!rejected.is_empty()).then(|| (id.clone(), rejected))
        })
        .collect()
}

fn rejected_labels(
    rejected: &BTreeMap<String, Vec<RelativeEvidence>>,
) -> BTreeMap<String, BTreeSet<String>> {
    rejected
        .iter()
        .map(|(id, evidence)| {
            (
                id.clone(),
                evidence
                    .iter()
                    .map(|item| item.source_label.clone())
                    .collect(),
            )
        })
        .collect()
}

fn is_placeholder_source(label: &str) -> bool {
    let normalized = label.to_ascii_lowercase();
    normalized.contains("fabricated")
        || normalized.contains("example.org")
        || normalized.contains("example.com")
        || normalized.contains("example.net")
}

fn build_statement_report(
    context: &Context,
    statement_id: &str,
    refutation_trace: Vec<RefutationAttempt>,
    counterexample: Option<String>,
    rejected: &[RelativeEvidence],
) -> StatementVerification {
    let statement = context
        .statement(statement_id)
        .expect("fact-check report ids come from the context");
    let mut evidence = statement
        .evidence
        .iter()
        .map(|item| evidence_trace(item, false))
        .chain(rejected.iter().map(|item| evidence_trace(item, true)))
        .collect::<Vec<_>>();
    evidence.sort_by(|left, right| left.source_label.cmp(&right.source_label));
    let has_evidence = statement
        .evidence
        .iter()
        .any(|item| item.effective_mass() > 0.0)
        || !statement.dependencies.is_empty();
    StatementVerification {
        statement_id: statement.id.clone(),
        text: statement.text.clone(),
        formal_system_name: context.formal_system().name.clone(),
        formal_system_id: context.formal_system().id(),
        probability: statement.truth,
        probability_basis: if has_evidence {
            ProbabilityBasis::EvidenceWeighted
        } else {
            ProbabilityBasis::PriorOnly
        },
        counterexample,
        evidence,
        refutation_trace,
    }
}

fn evidence_trace(evidence: &RelativeEvidence, rejected_as_fabricated: bool) -> EvidenceTrace {
    EvidenceTrace {
        source_label: evidence.source_label.clone(),
        tier: evidence.tier,
        stance: evidence.stance,
        strength: evidence.strength,
        ignored: evidence.is_ignored() || rejected_as_fabricated,
        rejected_as_fabricated,
    }
}

fn push_link(out: &mut String, from: &str, relation: &str, to: &str) {
    let _ = writeln!(
        out,
        "(\"{}\" \"{}\" \"{}\")",
        escape(from),
        escape(relation),
        escape(to)
    );
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

const fn scope_slug(scope: AuditScope) -> &'static str {
    match scope {
        AuditScope::CurrentDialogue => "current_dialogue",
        AuditScope::GeneralMemory => "general_memory",
    }
}

const fn basis_slug(basis: ProbabilityBasis) -> &'static str {
    match basis {
        ProbabilityBasis::PriorOnly => "prior_only",
        ProbabilityBasis::EvidenceWeighted => "evidence_weighted",
    }
}

const fn stage_slug(stage: RefutationStage) -> &'static str {
    match stage {
        RefutationStage::DisproveStatement => "disprove_statement",
        RefutationStage::DisproveNegation => "disprove_negation",
        RefutationStage::Decompose => "decompose",
        RefutationStage::DepthBound => "depth_bound",
    }
}

const fn outcome_slug(outcome: RefutationOutcome) -> &'static str {
    match outcome {
        RefutationOutcome::Refuted => "refuted",
        RefutationOutcome::Unrefuted => "unrefuted",
        RefutationOutcome::Inconclusive => "inconclusive",
    }
}

const fn evidence_status(evidence: &EvidenceTrace) -> &'static str {
    if evidence.rejected_as_fabricated {
        "rejected_fabrication"
    } else if evidence.ignored {
        "ignored"
    } else {
        "admitted"
    }
}
