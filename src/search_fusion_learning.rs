//! Review-gated procedural learning for statement-level search fusion.
//!
//! Successful production executions enter an append-only frontier. Two or
//! more independently identified runs with the same complete policy infer a
//! reusable candidate. The candidate cannot plan or execute until a held-out
//! gate is green and a named reviewer promotes it into the durable ledger.

use std::collections::BTreeMap;
use std::fmt;

use crate::engine::stable_id;
use crate::links_format::push_lino_node;
use crate::search_fusion::{
    execute_search_fusion, SearchFusionExecution, SearchSourceClassification,
};
use crate::seed::parser::{parse_lino, LinoNode};
use crate::source_fetch::{CachedSourceClient, SourceTransport};

const LEARNING_CONTRACT: &str = include_str!("../data/meta/search-fusion-learning-contract.lino");
const SOURCE_POLICY: &str = include_str!("../data/meta/search-fusion-source-policy.lino");

pub const SEARCH_FUSION_TASK_FAMILY: &str = "captured_search_statement_fusion";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFusionLearningError {
    pub reason: String,
}

impl fmt::Display for SearchFusionLearningError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for SearchFusionLearningError {}

fn error(reason: impl Into<String>) -> SearchFusionLearningError {
    SearchFusionLearningError {
        reason: reason.into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FusionPolicy {
    task_family: String,
    stages: Vec<String>,
    minimum_executions: usize,
}

fn policy() -> Result<FusionPolicy, SearchFusionLearningError> {
    let contract_tree = parse_lino(LEARNING_CONTRACT);
    let contract = root_named(&contract_tree, "search_fusion_learning_contract")?;
    let policy_tree = parse_lino(SOURCE_POLICY);
    let source = root_named(&policy_tree, "search_fusion_source_policy")?;
    let task_family = required(contract, "task_family")?;
    if task_family != required(source, "task_family")? || task_family != SEARCH_FUSION_TASK_FAMILY {
        return Err(error("learning_task_family_mismatch"));
    }
    if required(contract, "candidate_inert")? != "true" {
        return Err(error("learning_candidate_must_be_inert"));
    }
    let minimum_executions = required(contract, "minimum_independent_executions")?
        .parse::<usize>()
        .map_err(|_| error("learning_minimum_executions_invalid"))?;
    if minimum_executions < 2 {
        return Err(error("learning_minimum_executions_unsafe"));
    }
    let stages = source
        .children
        .iter()
        .filter(|node| node.name == "stage" && !node.id.is_empty())
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if stages.len() < 2 {
        return Err(error("learning_policy_stages_incomplete"));
    }
    Ok(FusionPolicy {
        task_family,
        stages,
        minimum_executions,
    })
}

/// One accepted search-fusion execution recorded outside the active planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFusionLearningObservation {
    pub id: String,
    pub task_id: String,
    pub execution_fingerprint: String,
    pub task_family: String,
    pub query: String,
    pub stages: Vec<String>,
}

impl SearchFusionLearningObservation {
    fn from_execution(
        task_id: &str,
        execution: &SearchFusionExecution,
    ) -> Result<Self, SearchFusionLearningError> {
        if task_id.trim().is_empty() {
            return Err(error("learning_task_id_required"));
        }
        validate_execution(execution)?;
        let policy = policy()?;
        let execution_identity = format!("{}\n{}", execution.answer.query, execution.trace());
        let execution_fingerprint = stable_id("search_fusion_execution", &execution_identity);
        let identity = format!(
            "{}\n{}\n{}",
            task_id, policy.task_family, execution_fingerprint
        );
        Ok(Self {
            id: stable_id("search_fusion_learning_observation", &identity),
            task_id: task_id.trim().to_owned(),
            execution_fingerprint,
            task_family: policy.task_family,
            query: execution.answer.query.clone(),
            stages: policy.stages,
        })
    }
}

fn validate_execution(execution: &SearchFusionExecution) -> Result<(), SearchFusionLearningError> {
    if execution.research.search.captures.is_empty()
        || execution.answer.sources.is_empty()
        || execution.observations.is_empty()
        || execution.answer.statements.is_empty()
    {
        return Err(error("learning_execution_incomplete"));
    }
    if execution.observations.iter().any(|observation| {
        observation.source_url.is_empty()
            || observation.original_text.is_empty()
            || observation.formalization.is_empty()
            || observation.language.is_empty()
    }) {
        return Err(error("learning_formalization_receipt_incomplete"));
    }
    if execution.answer.statements.iter().any(|statement| {
        statement.sources.is_empty()
            || statement.sources.iter().any(|source| {
                source.url.is_empty()
                    || source.title.is_empty()
                    || source.quote.is_empty()
                    || source.read_more != source.url
                    || source.tier == crate::relative_meta_logic::SourceTier::Unoriginal
            })
    }) {
        return Err(error("learning_ranked_provenance_incomplete"));
    }
    Ok(())
}

/// Automatically inferred procedure. It is deliberately not executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFusionRecipeCandidate {
    pub id: String,
    pub task_family: String,
    pub stages: Vec<String>,
    pub evidence_count: usize,
}

/// Append-only observations from which one stable procedure can be inferred.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchFusionLearningFrontier {
    observations: BTreeMap<String, SearchFusionLearningObservation>,
}

impl SearchFusionLearningFrontier {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            observations: BTreeMap::new(),
        }
    }

    /// Record a successful execution and infer a candidate once the
    /// data-authored minimum number of independently identified runs exists.
    pub fn record_execution(
        &mut self,
        task_id: &str,
        execution: &SearchFusionExecution,
    ) -> Result<Option<SearchFusionRecipeCandidate>, SearchFusionLearningError> {
        let observation = SearchFusionLearningObservation::from_execution(task_id, execution)?;
        if self.observations.values().any(|existing| {
            existing.task_id == observation.task_id
                || existing.execution_fingerprint == observation.execution_fingerprint
        }) {
            return Err(error("learning_execution_not_independent"));
        }
        self.observations
            .insert(observation.id.clone(), observation);
        self.infer_candidate()
    }

    #[must_use]
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    fn infer_candidate(
        &self,
    ) -> Result<Option<SearchFusionRecipeCandidate>, SearchFusionLearningError> {
        let policy = policy()?;
        if self.observations.len() < policy.minimum_executions {
            return Ok(None);
        }
        if self.observations.values().any(|observation| {
            observation.task_family != policy.task_family || observation.stages != policy.stages
        }) {
            return Err(error("learning_execution_policy_drift"));
        }
        let identity =
            candidate_identity(&policy.task_family, &policy.stages, self.observations.len());
        Ok(Some(SearchFusionRecipeCandidate {
            id: stable_id("search_fusion_recipe_candidate", &identity),
            task_family: policy.task_family,
            stages: policy.stages,
            evidence_count: self.observations.len(),
        }))
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("search_fusion_learning_frontier\n");
        push_lino_node(&mut out, 2, "candidate_inert", Some("true"));
        for observation in self.observations.values() {
            push_lino_node(&mut out, 2, "observation", Some(&observation.id));
            push_lino_node(&mut out, 4, "task_id", Some(&observation.task_id));
            push_lino_node(
                &mut out,
                4,
                "execution_fingerprint",
                Some(&observation.execution_fingerprint),
            );
            push_lino_node(&mut out, 4, "task_family", Some(&observation.task_family));
            push_lino_node(&mut out, 4, "query", Some(&observation.query));
            for stage in &observation.stages {
                push_lino_node(&mut out, 4, "stage", Some(stage));
            }
        }
        if let Ok(Some(candidate)) = self.infer_candidate() {
            push_lino_node(&mut out, 2, "candidate", Some(&candidate.id));
            push_lino_node(&mut out, 4, "status", Some("human_review_required"));
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFusionLearningGate {
    pub suite: String,
    pub passed: usize,
    pub failed: usize,
}

impl SearchFusionLearningGate {
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

    fn is_green(&self) -> bool {
        !self.suite.trim().is_empty() && self.passed > 0 && self.failed == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFusionLearningApproval {
    pub reviewer: String,
    pub granted: bool,
}

impl SearchFusionLearningApproval {
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
struct ApprovedRecipe {
    candidate: SearchFusionRecipeCandidate,
    suite: String,
    passed: usize,
    reviewer: String,
}

/// Durable reviewed recipe book used by later equivalent research tasks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchFusionRecipeLedger {
    recipes: BTreeMap<String, ApprovedRecipe>,
}

impl SearchFusionRecipeLedger {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            recipes: BTreeMap::new(),
        }
    }

    pub fn promote(
        &mut self,
        candidate: &SearchFusionRecipeCandidate,
        gate: SearchFusionLearningGate,
        approval: SearchFusionLearningApproval,
    ) -> Result<(), SearchFusionLearningError> {
        let SearchFusionLearningApproval { reviewer, granted } = approval;
        let policy = policy()?;
        let expected_id = stable_id(
            "search_fusion_recipe_candidate",
            &candidate_identity(
                &candidate.task_family,
                &candidate.stages,
                candidate.evidence_count,
            ),
        );
        if candidate.id != expected_id
            || candidate.task_family != policy.task_family
            || candidate.stages != policy.stages
            || candidate.evidence_count < policy.minimum_executions
        {
            return Err(error("learning_candidate_integrity_failed"));
        }
        if !gate.is_green() {
            return Err(error("learning_green_gate_required"));
        }
        if !granted || reviewer.trim().is_empty() {
            return Err(error("learning_human_approval_required"));
        }
        self.recipes.insert(
            candidate.task_family.clone(),
            ApprovedRecipe {
                candidate: candidate.clone(),
                suite: gate.suite,
                passed: gate.passed,
                reviewer: reviewer.trim().to_owned(),
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn plan_for(&self, task_family: &str) -> Option<&[String]> {
        self.recipes
            .get(task_family)
            .map(|recipe| recipe.candidate.stages.as_slice())
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let body = self.review_body();
        let id = stable_id("search_fusion_recipe_ledger", &body);
        let mut out = String::new();
        push_lino_node(&mut out, 0, "search_fusion_recipe_ledger", Some(&id));
        push_lino_node(&mut out, 2, "schema_version", Some("1"));
        push_lino_node(&mut out, 2, "human_gated", Some("true"));
        out.push_str(&body);
        out
    }

    fn review_body(&self) -> String {
        let mut out = String::new();
        for recipe in self.recipes.values() {
            push_lino_node(
                &mut out,
                2,
                "approved_recipe",
                Some(&recipe.candidate.task_family),
            );
            push_lino_node(&mut out, 4, "candidate", Some(&recipe.candidate.id));
            push_lino_node(
                &mut out,
                4,
                "evidence_count",
                Some(&recipe.candidate.evidence_count.to_string()),
            );
            for stage in &recipe.candidate.stages {
                push_lino_node(&mut out, 4, "stage", Some(stage));
            }
            push_lino_node(&mut out, 4, "suite", Some(&recipe.suite));
            push_lino_node(&mut out, 4, "passed", Some(&recipe.passed.to_string()));
            push_lino_node(&mut out, 4, "failed", Some("0"));
            push_lino_node(&mut out, 4, "reviewer", Some(&recipe.reviewer));
        }
        out
    }

    pub fn from_links_notation(document: &str) -> Result<Self, SearchFusionLearningError> {
        let tree = parse_lino(document);
        let root = root_named(&tree, "search_fusion_recipe_ledger")?;
        if required(root, "schema_version")? != "1" || required(root, "human_gated")? != "true" {
            return Err(error("learning_ledger_policy_invalid"));
        }
        let mut ledger = Self::new();
        for node in root
            .children
            .iter()
            .filter(|node| node.name == "approved_recipe")
        {
            let task_family = node.id.clone();
            let stages = node
                .children
                .iter()
                .filter(|child| child.name == "stage")
                .map(|child| child.id.clone())
                .collect::<Vec<_>>();
            let evidence_count = parse_usize(node, "evidence_count")?;
            let candidate = SearchFusionRecipeCandidate {
                id: required(node, "candidate")?,
                task_family,
                stages,
                evidence_count,
            };
            let gate = SearchFusionLearningGate {
                suite: required(node, "suite")?,
                passed: parse_usize(node, "passed")?,
                failed: parse_usize(node, "failed")?,
            };
            let approval = SearchFusionLearningApproval::granted(required(node, "reviewer")?);
            ledger.promote(&candidate, gate, approval)?;
        }
        let expected = stable_id("search_fusion_recipe_ledger", &ledger.review_body());
        if root.id != expected {
            return Err(error("learning_ledger_content_address_mismatch"));
        }
        Ok(ledger)
    }
}

/// Execute only when a reviewed recipe activates the captured-search family.
pub fn execute_search_fusion_with_recipe<T, C>(
    ledger: &SearchFusionRecipeLedger,
    client: &CachedSourceClient<T>,
    query: &str,
    target_language: &str,
    page_limit: usize,
    classify: C,
) -> Result<SearchFusionExecution, SearchFusionLearningError>
where
    T: SourceTransport,
    C: Fn(&str) -> SearchSourceClassification,
{
    let plan = ledger
        .plan_for(SEARCH_FUSION_TASK_FAMILY)
        .ok_or_else(|| error("learning_recipe_not_approved"))?;
    if plan != policy()?.stages {
        return Err(error("learning_recipe_stage_drift"));
    }
    execute_search_fusion(client, query, target_language, page_limit, classify)
        .map_err(|fetch| error(format!("learning_recipe_fetch:{fetch}")))
}

fn candidate_identity(task_family: &str, stages: &[String], evidence_count: usize) -> String {
    let mut identity = String::from(task_family);
    identity.push('\n');
    identity.push_str(&stages.join("\n"));
    identity.push('\n');
    identity.push_str(&evidence_count.to_string());
    identity
}

fn root_named<'a>(
    tree: &'a LinoNode,
    name: &str,
) -> Result<&'a LinoNode, SearchFusionLearningError> {
    tree.children
        .iter()
        .find(|node| node.name == name)
        .ok_or_else(|| error(format!("learning_missing_{name}")))
}

fn required(node: &LinoNode, name: &str) -> Result<String, SearchFusionLearningError> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| child.id.clone())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(format!("learning_missing_{name}")))
}

fn parse_usize(node: &LinoNode, name: &str) -> Result<usize, SearchFusionLearningError> {
    required(node, name)?
        .parse::<usize>()
        .map_err(|_| error(format!("learning_invalid_{name}")))
}
