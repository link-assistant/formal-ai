//! Review-gated learning for task-decomposition strategies.
//!
//! A failed recursive execution can propose an already registered, typed
//! strategy. It cannot activate that strategy: promotion additionally requires
//! a green regression gate and an explicit reviewer approval.

use std::collections::BTreeMap;

use crate::engine::stable_id;
use crate::links_format::push_lino_node;
use crate::recursive_execution::{RecursiveExecution, RecursiveRun};
use crate::seed::parser::{parse_lino, LinoNode};

use super::{strategy, Decomposition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStrategyProposal {
    pub id: String,
    pub strategy_id: String,
    pub failed_task_id: String,
    pub failure_evidence: String,
}

impl TaskStrategyProposal {
    #[must_use]
    pub fn from_failed_run(decomposition: &Decomposition, run: &RecursiveRun) -> Option<Self> {
        if run.status != RecursiveExecution::Blocked
            || run.task.id != decomposition.root.id
            || !strategy::missing_operation_contract(&decomposition.task)
        {
            return None;
        }
        let failed = first_blocked_leaf(run)?;
        let failure_evidence = failed.attempts.last()?.evidence.clone();
        let strategy_id = strategy::strategies().first()?.id.clone();
        let id = proposal_id(&strategy_id, &failed.task.id, &failure_evidence);
        Some(Self {
            id,
            strategy_id,
            failed_task_id: failed.task.id.clone(),
            failure_evidence,
        })
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "task_strategy_proposal", Some(&self.id));
        push_lino_node(&mut out, 2, "strategy_id", Some(&self.strategy_id));
        push_lino_node(&mut out, 2, "failed_task_id", Some(&self.failed_task_id));
        push_lino_node(
            &mut out,
            2,
            "failure_evidence",
            Some(&self.failure_evidence),
        );
        push_lino_node(&mut out, 2, "status", Some("human_review_required"));
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLearningGate {
    pub suite: String,
    pub passed: usize,
    pub failed: usize,
}

impl TaskLearningGate {
    #[must_use]
    pub fn passed(suite: impl Into<String>, passed: usize) -> Self {
        Self {
            suite: suite.into(),
            passed,
            failed: 0,
        }
    }

    #[must_use]
    pub fn failed(suite: impl Into<String>, passed: usize, failed: usize) -> Self {
        Self {
            suite: suite.into(),
            passed,
            failed,
        }
    }

    const fn is_green(&self) -> bool {
        self.passed > 0 && self.failed == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLearningApproval {
    pub reviewer: String,
    pub granted: bool,
}

impl TaskLearningApproval {
    #[must_use]
    pub fn granted(reviewer: impl Into<String>) -> Self {
        Self {
            reviewer: reviewer.into(),
            granted: true,
        }
    }

    #[must_use]
    pub fn declined(reviewer: impl Into<String>) -> Self {
        Self {
            reviewer: reviewer.into(),
            granted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLearningError {
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TaskStrategyReview {
    id: String,
    strategy_id: String,
    proposal_id: String,
    failed_task_id: String,
    failure_evidence: String,
    suite: String,
    passed: usize,
    reviewer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskStrategyLedger {
    reviews: BTreeMap<String, TaskStrategyReview>,
}

impl TaskStrategyLedger {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reviews: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn shipped() -> Self {
        let reviews = strategy::shipped_approvals()
            .into_iter()
            .map(|approval| {
                let proposal_id = proposal_id(
                    &approval.strategy_id,
                    &approval.failed_task_id,
                    &approval.failure_evidence,
                );
                let review = TaskStrategyReview::new(
                    approval.strategy_id.clone(),
                    proposal_id,
                    approval.failed_task_id,
                    approval.failure_evidence,
                    approval.suite,
                    approval.passed,
                    approval.reviewer,
                );
                (approval.strategy_id, review)
            })
            .collect();
        Self { reviews }
    }

    /// Registered strategies activated by reviewed, green evidence.
    #[must_use]
    pub fn approved_strategy_ids(&self) -> Vec<&str> {
        self.reviews.keys().map(String::as_str).collect()
    }

    /// Whether this durable ledger permits a registered strategy to plan.
    #[must_use]
    pub fn allows(&self, strategy_id: &str) -> bool {
        self.reviews.contains_key(strategy_id)
    }

    /// Promote an execution-derived proposal only after tests and human review.
    pub fn promote(
        &mut self,
        proposal: &TaskStrategyProposal,
        gate: TaskLearningGate,
        approval: TaskLearningApproval,
    ) -> Result<(), TaskLearningError> {
        if proposal.id
            != proposal_id(
                &proposal.strategy_id,
                &proposal.failed_task_id,
                &proposal.failure_evidence,
            )
        {
            return Err(error("proposal_integrity_failed"));
        }
        if !strategy::strategies()
            .iter()
            .any(|candidate| candidate.id == proposal.strategy_id)
        {
            return Err(error("unknown_strategy"));
        }
        if !gate.is_green() {
            return Err(error("regression_gate_failed"));
        }
        if !approval.granted || approval.reviewer.trim().is_empty() {
            return Err(error("human_review_required"));
        }
        let review = TaskStrategyReview::new(
            proposal.strategy_id.clone(),
            proposal.id.clone(),
            proposal.failed_task_id.clone(),
            proposal.failure_evidence.clone(),
            gate.suite,
            gate.passed,
            approval.reviewer,
        );
        self.reviews.insert(proposal.strategy_id.clone(), review);
        Ok(())
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let body = self.review_body();
        let id = stable_id("task_strategy_ledger", &body);
        let mut out = String::new();
        push_lino_node(&mut out, 0, "task_strategy_ledger", Some(&id));
        push_lino_node(&mut out, 2, "schema_version", Some("1"));
        push_lino_node(&mut out, 2, "human_gated", Some("true"));
        out.push_str(&body);
        out
    }

    /// Restore only a content-addressed ledger carrying green tests and an
    /// explicit reviewer for every activated strategy.
    pub fn from_links_notation(document: &str) -> Result<Self, TaskLearningError> {
        let tree = parse_lino(document);
        let root = tree
            .children
            .iter()
            .find(|node| node.name == "task_strategy_ledger")
            .ok_or_else(|| error("missing_ledger"))?;
        if root.find_child_value("schema_version") != "1" {
            return Err(error("unsupported_ledger_schema"));
        }
        if root.find_child_value("human_gated") != "true" {
            return Err(error("ledger_not_human_gated"));
        }
        let mut reviews = BTreeMap::new();
        for node in root
            .children
            .iter()
            .filter(|node| node.name == "approved_strategy")
        {
            let strategy_id = required(node, "strategy_id")?;
            let proposal_identity = required(node, "proposal_id")?;
            let failed_task_id = required(node, "failed_task_id")?;
            let failure_evidence = required(node, "failure_evidence")?;
            let suite = required(node, "suite")?;
            let passed = parse_usize(node, "passed")?;
            let failed = parse_usize(node, "failed")?;
            let reviewer = required(node, "reviewer")?;
            if passed == 0 || failed != 0 {
                return Err(error("regression_gate_failed"));
            }
            if reviewer.trim().is_empty() {
                return Err(error("human_review_required"));
            }
            if !strategy::strategies()
                .iter()
                .any(|candidate| candidate.id == strategy_id)
            {
                return Err(error("unknown_strategy"));
            }
            if proposal_identity != proposal_id(&strategy_id, &failed_task_id, &failure_evidence) {
                return Err(error("proposal_integrity_failed"));
            }
            let review = TaskStrategyReview::new(
                strategy_id.clone(),
                proposal_identity,
                failed_task_id,
                failure_evidence,
                suite,
                passed,
                reviewer,
            );
            if review.id != node.id {
                return Err(error("review_integrity_failed"));
            }
            if reviews.insert(strategy_id, review).is_some() {
                return Err(error("duplicate_strategy_review"));
            }
        }
        let restored = Self { reviews };
        let expected = parse_lino(&restored.links_notation())
            .children
            .first()
            .map(|node| node.id.clone())
            .unwrap_or_default();
        if root.id != expected {
            return Err(error("ledger_integrity_failed"));
        }
        Ok(restored)
    }

    fn review_body(&self) -> String {
        let mut out = String::new();
        for review in self.reviews.values() {
            push_lino_node(&mut out, 2, "approved_strategy", Some(&review.id));
            push_lino_node(&mut out, 4, "strategy_id", Some(&review.strategy_id));
            push_lino_node(&mut out, 4, "proposal_id", Some(&review.proposal_id));
            push_lino_node(&mut out, 4, "failed_task_id", Some(&review.failed_task_id));
            push_lino_node(
                &mut out,
                4,
                "failure_evidence",
                Some(&review.failure_evidence),
            );
            push_lino_node(&mut out, 4, "suite", Some(&review.suite));
            push_lino_node(&mut out, 4, "passed", Some(&review.passed.to_string()));
            push_lino_node(&mut out, 4, "failed", Some("0"));
            push_lino_node(&mut out, 4, "reviewer", Some(&review.reviewer));
        }
        out
    }
}

impl TaskStrategyReview {
    fn new(
        strategy_id: String,
        proposal_id: String,
        failed_task_id: String,
        failure_evidence: String,
        suite: String,
        passed: usize,
        reviewer: String,
    ) -> Self {
        let id = stable_id(
            "task_strategy_review",
            &format!(
                "{strategy_id}\n{proposal_id}\n{failed_task_id}\n{failure_evidence}\n\
                 {suite}\n{passed}\n{reviewer}"
            ),
        );
        Self {
            id,
            strategy_id,
            proposal_id,
            failed_task_id,
            failure_evidence,
            suite,
            passed,
            reviewer,
        }
    }
}

fn first_blocked_leaf(run: &RecursiveRun) -> Option<&RecursiveRun> {
    if run.children.is_empty() {
        return (run.status == RecursiveExecution::Blocked).then_some(run);
    }
    run.children.iter().find_map(first_blocked_leaf)
}

fn error(reason: &str) -> TaskLearningError {
    TaskLearningError {
        reason: reason.to_owned(),
    }
}

fn proposal_id(strategy_id: &str, failed_task_id: &str, failure_evidence: &str) -> String {
    stable_id(
        "task_strategy_proposal",
        &format!("{strategy_id}\n{failed_task_id}\n{failure_evidence}"),
    )
}

fn required(node: &LinoNode, name: &str) -> Result<String, TaskLearningError> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| child.id.clone())
        .ok_or_else(|| error(&format!("missing_{name}")))
}

fn parse_usize(node: &LinoNode, name: &str) -> Result<usize, TaskLearningError> {
    required(node, name)?
        .parse::<usize>()
        .map_err(|_| error(&format!("invalid_{name}")))
}
