//! Review-gated procedural learning for verified repository rewrites.
//!
//! Successful exact-observation executions enter an append-only frontier. Two
//! independent executions with the same data-authored stage policy infer an
//! inert candidate. Only a zero-failure held-out gate and a named reviewer can
//! promote that candidate into the content-addressed recipe ledger used for an
//! unseen equivalent rewrite.

use std::collections::BTreeMap;
use std::fmt;

use crate::engine::stable_id;
use crate::links_format::push_lino_node;
use crate::normal_markov::{RewriteHalt, RewriteProgram, RewriteRule};
use crate::seed::parser::{LinoNode, parse_lino};

const LEARNING_CONTRACT: &str =
    include_str!("../data/meta/workspace-change-learning-contract.lino");
const EXECUTION_POLICY: &str = include_str!("../data/meta/workspace-change-execution-policy.lino");
const MAX_REWRITE_STEPS: usize = 100_000;

pub const WORKSPACE_CHANGE_TASK_FAMILY: &str = "verified_workspace_rewrite";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceChangeLearningError {
    pub reason: String,
}

impl fmt::Display for WorkspaceChangeLearningError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for WorkspaceChangeLearningError {}

fn error(reason: impl Into<String>) -> WorkspaceChangeLearningError {
    WorkspaceChangeLearningError {
        reason: reason.into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkspaceChangePolicy {
    task_family: String,
    stages: Vec<String>,
    minimum_executions: usize,
}

fn policy() -> Result<WorkspaceChangePolicy, WorkspaceChangeLearningError> {
    let contract_tree = parse_lino(LEARNING_CONTRACT);
    let contract = root_named(&contract_tree, "workspace_change_learning_contract")?;
    let policy_tree = parse_lino(EXECUTION_POLICY);
    let execution = root_named(&policy_tree, "workspace_change_execution_policy")?;
    if required(contract, "schema_version")? != "1" || required(execution, "schema_version")? != "1"
    {
        return Err(error("workspace_learning_schema_invalid"));
    }
    let task_family = required(contract, "task_family")?;
    if task_family != required(execution, "task_family")?
        || task_family != WORKSPACE_CHANGE_TASK_FAMILY
    {
        return Err(error("workspace_learning_task_family_mismatch"));
    }
    if required(contract, "candidate_inert")? != "true"
        || required(execution, "candidate_effect")? != "inert_until_reviewed"
        || required(execution, "failure_effect")? != "no_partial_write"
    {
        return Err(error("workspace_learning_safety_policy_invalid"));
    }
    let minimum_executions = required(contract, "minimum_independent_executions")?
        .parse::<usize>()
        .map_err(|_| error("workspace_learning_minimum_invalid"))?;
    if minimum_executions < 2 {
        return Err(error("workspace_learning_minimum_unsafe"));
    }
    let stages = execution
        .children
        .iter()
        .filter(|node| node.name == "stage" && !node.id.is_empty())
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if stages.len() < 2 {
        return Err(error("workspace_learning_stages_incomplete"));
    }
    Ok(WorkspaceChangePolicy {
        task_family,
        stages,
        minimum_executions,
    })
}

/// A bounded Normal Markov execution before its client read-back is judged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRewriteExecution {
    pub pattern: String,
    pub replacement: String,
    pub output: String,
    pub steps: usize,
    source_fingerprint: String,
}

/// How a compiled pattern is allowed to match the source bytes.
///
/// The distinction is not cosmetic. A substring rewrite of `OLD` into `NEW_OLD`
/// never terminates, because every application puts the pattern back; that is
/// why the substring form has to refuse an operand pair whose replacement
/// contains its pattern. Renaming an *identifier*, though, is the one edit
/// programmers make most often where exactly that containment is the point:
/// `SESSION_TRAILER` becomes `AGENT_SESSION_TRAILER` by prefixing it. Under
/// word scope the inner occurrence is not a match -- its left neighbour is `_`,
/// an identifier character -- so the rewrite reaches a fixed point in one pass
/// and the refusal is unnecessary (#1069).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewriteScope {
    /// Every byte-sequence occurrence of the pattern is a match.
    Substring,
    /// Only occurrences whose neighbouring characters are not identifier
    /// characters are matches, so a longer name that merely contains the
    /// pattern is left alone -- and may be produced.
    Word,
}

/// Frame character for word-scoped rewriting.
///
/// A normal algorithm has no anchors: a rule is a plain substring pair. Word
/// scope is therefore expressed the way normal algorithms have always expressed
/// context sensitivity -- by carrying the context *into* the pattern. Framing
/// the source in a character it cannot contain gives the first and last
/// positions a neighbour like every other position, so one rule shape covers
/// matches at the edges too.
const REWRITE_FRAME: char = '\u{0}';

/// Byte offsets of the word-scoped occurrences of `pattern` in `source`.
///
/// A match is word-scoped when neither neighbouring character is an identifier
/// character, which is the same rule an editor's "whole word" search uses.
#[must_use]
pub fn word_scoped_matches(source: &str, pattern: &str) -> Vec<usize> {
    if pattern.is_empty() {
        return Vec::new();
    }
    source
        .match_indices(pattern)
        .filter(|(at, _)| {
            let before = source[..*at].chars().next_back();
            let after = source[at + pattern.len()..].chars().next();
            !before.is_some_and(is_identifier_character)
                && !after.is_some_and(is_identifier_character)
        })
        .map(|(at, _)| at)
        .collect()
}

fn is_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// Whether `text` is a single identifier-shaped word.
#[must_use]
pub fn is_identifier_word(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_identifier_character)
}

/// The context-carrying rules that rename `pattern` to `replacement` in
/// `framed`, one rule per distinct pair of neighbouring characters observed.
///
/// Each rule re-emits the neighbours it matched, so two occurrences that share
/// a separator both still find their context after the first is rewritten.
fn word_scoped_rules(framed: &str, pattern: &str, replacement: &str) -> Vec<RewriteRule> {
    let mut rules: Vec<RewriteRule> = Vec::new();
    for at in word_scoped_matches(framed, pattern) {
        let before = framed[..at].chars().next_back().unwrap_or(REWRITE_FRAME);
        let after = framed[at + pattern.len()..]
            .chars()
            .next()
            .unwrap_or(REWRITE_FRAME);
        let rule = RewriteRule::new(
            format!("{before}{pattern}{after}"),
            format!("{before}{replacement}{after}"),
        );
        if !rules.contains(&rule) {
            rules.push(rule);
        }
    }
    rules
}

/// Compile and execute one safe substring substitution against caller-owned
/// source bytes.
pub fn execute_workspace_rewrite(
    source: &str,
    pattern: &str,
    replacement: &str,
) -> Result<WorkspaceRewriteExecution, WorkspaceChangeLearningError> {
    execute_scoped_workspace_rewrite(source, pattern, replacement, RewriteScope::Substring)
}

/// Compile and execute one safe substitution under an explicit match scope.
pub fn execute_scoped_workspace_rewrite(
    source: &str,
    pattern: &str,
    replacement: &str,
    scope: RewriteScope,
) -> Result<WorkspaceRewriteExecution, WorkspaceChangeLearningError> {
    if pattern.is_empty() || pattern == replacement {
        return Err(error("workspace_rewrite_operands_unsafe"));
    }
    let (rules, subject) = match scope {
        RewriteScope::Substring => {
            if replacement.contains(pattern) {
                return Err(error("workspace_rewrite_operands_unsafe"));
            }
            (
                vec![RewriteRule::new(pattern, replacement)],
                source.to_owned(),
            )
        }
        RewriteScope::Word => {
            // Both operands being single words is what makes the run finite: a
            // one-word replacement cannot contain the pattern as a whole word
            // unless the two are equal, which is already refused above.
            if !is_identifier_word(pattern) || !is_identifier_word(replacement) {
                return Err(error("workspace_rewrite_operands_unsafe"));
            }
            if source.contains(REWRITE_FRAME) {
                return Err(error("workspace_rewrite_frame_conflict"));
            }
            let framed = format!("{REWRITE_FRAME}{source}{REWRITE_FRAME}");
            let rules = word_scoped_rules(&framed, pattern, replacement);
            (rules, framed)
        }
    };
    if rules.is_empty() {
        return Err(error("workspace_rewrite_no_match"));
    }
    let outcome = RewriteProgram::new(rules, MAX_REWRITE_STEPS).execute(&subject);
    if outcome.halt == RewriteHalt::StepLimit {
        return Err(error("workspace_rewrite_step_limit"));
    }
    let output = match scope {
        RewriteScope::Substring => outcome.output,
        RewriteScope::Word => outcome.output.trim_matches(REWRITE_FRAME).to_owned(),
    };
    if outcome.trace.is_empty() || output == source {
        return Err(error("workspace_rewrite_no_match"));
    }
    Ok(WorkspaceRewriteExecution {
        pattern: pattern.to_owned(),
        replacement: replacement.to_owned(),
        output,
        steps: outcome.trace.len(),
        source_fingerprint: stable_id("workspace_rewrite_source", source),
    })
}

/// One independently identified execution accepted only after exact read-back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceChangeLearningObservation {
    pub id: String,
    pub task_id: String,
    pub execution_fingerprint: String,
    pub task_family: String,
    pub stages: Vec<String>,
}

impl WorkspaceChangeLearningObservation {
    fn from_execution(
        task_id: &str,
        execution: &WorkspaceRewriteExecution,
        observed: &str,
    ) -> Result<Self, WorkspaceChangeLearningError> {
        if task_id.trim().is_empty() {
            return Err(error("workspace_learning_task_id_required"));
        }
        if observed != execution.output {
            return Err(error("workspace_learning_exact_observation_required"));
        }
        let policy = policy()?;
        let execution_identity = format!(
            "{}\n{}\n{}\n{}\n{}",
            execution.source_fingerprint,
            execution.pattern,
            execution.replacement,
            stable_id("workspace_rewrite_output", &execution.output),
            execution.steps,
        );
        let execution_fingerprint =
            stable_id("workspace_change_learning_execution", &execution_identity);
        let identity = format!(
            "{}\n{}\n{}",
            task_id, policy.task_family, execution_fingerprint
        );
        Ok(Self {
            id: stable_id("workspace_change_learning_observation", &identity),
            task_id: task_id.trim().to_owned(),
            execution_fingerprint,
            task_family: policy.task_family,
            stages: policy.stages,
        })
    }
}

/// Automatically inferred procedure. It has no execution method by design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceChangeRecipeCandidate {
    pub id: String,
    pub task_family: String,
    pub stages: Vec<String>,
    pub evidence_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceChangeLearningFrontier {
    observations: BTreeMap<String, WorkspaceChangeLearningObservation>,
}

impl WorkspaceChangeLearningFrontier {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            observations: BTreeMap::new(),
        }
    }

    pub fn record_execution(
        &mut self,
        task_id: &str,
        execution: &WorkspaceRewriteExecution,
        observed: &str,
    ) -> Result<Option<WorkspaceChangeRecipeCandidate>, WorkspaceChangeLearningError> {
        let observation =
            WorkspaceChangeLearningObservation::from_execution(task_id, execution, observed)?;
        if self.observations.values().any(|existing| {
            existing.task_id == observation.task_id
                || existing.execution_fingerprint == observation.execution_fingerprint
        }) {
            return Err(error("workspace_learning_execution_not_independent"));
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
    ) -> Result<Option<WorkspaceChangeRecipeCandidate>, WorkspaceChangeLearningError> {
        let policy = policy()?;
        if self.observations.len() < policy.minimum_executions {
            return Ok(None);
        }
        if self.observations.values().any(|observation| {
            observation.task_family != policy.task_family || observation.stages != policy.stages
        }) {
            return Err(error("workspace_learning_execution_policy_drift"));
        }
        let identity =
            candidate_identity(&policy.task_family, &policy.stages, self.observations.len());
        Ok(Some(WorkspaceChangeRecipeCandidate {
            id: stable_id("workspace_change_recipe_candidate", &identity),
            task_family: policy.task_family,
            stages: policy.stages,
            evidence_count: self.observations.len(),
        }))
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("workspace_change_learning_frontier\n");
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
pub struct WorkspaceChangeLearningGate {
    pub suite: String,
    pub passed: usize,
    pub failed: usize,
}

impl WorkspaceChangeLearningGate {
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
pub struct WorkspaceChangeLearningApproval {
    pub reviewer: String,
    pub granted: bool,
}

impl WorkspaceChangeLearningApproval {
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
    candidate: WorkspaceChangeRecipeCandidate,
    suite: String,
    passed: usize,
    reviewer: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceChangeRecipeLedger {
    recipes: BTreeMap<String, ApprovedRecipe>,
}

impl WorkspaceChangeRecipeLedger {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            recipes: BTreeMap::new(),
        }
    }

    pub fn promote(
        &mut self,
        candidate: &WorkspaceChangeRecipeCandidate,
        gate: WorkspaceChangeLearningGate,
        approval: WorkspaceChangeLearningApproval,
    ) -> Result<(), WorkspaceChangeLearningError> {
        let WorkspaceChangeLearningApproval { reviewer, granted } = approval;
        let policy = policy()?;
        let expected_id = stable_id(
            "workspace_change_recipe_candidate",
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
            return Err(error("workspace_learning_candidate_integrity_failed"));
        }
        if !gate.is_green() {
            return Err(error("workspace_learning_green_gate_required"));
        }
        if !granted || reviewer.trim().is_empty() {
            return Err(error("workspace_learning_human_approval_required"));
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
        let id = stable_id("workspace_change_recipe_ledger", &body);
        let mut out = String::new();
        push_lino_node(&mut out, 0, "workspace_change_recipe_ledger", Some(&id));
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

    pub fn from_links_notation(document: &str) -> Result<Self, WorkspaceChangeLearningError> {
        let tree = parse_lino(document);
        let root = root_named(&tree, "workspace_change_recipe_ledger")?;
        if required(root, "schema_version")? != "1" || required(root, "human_gated")? != "true" {
            return Err(error("workspace_learning_ledger_policy_invalid"));
        }
        let mut ledger = Self::new();
        for node in root
            .children
            .iter()
            .filter(|node| node.name == "approved_recipe")
        {
            let stages = node
                .children
                .iter()
                .filter(|child| child.name == "stage")
                .map(|child| child.id.clone())
                .collect::<Vec<_>>();
            let candidate = WorkspaceChangeRecipeCandidate {
                id: required(node, "candidate")?,
                task_family: node.id.clone(),
                stages,
                evidence_count: parse_usize(node, "evidence_count")?,
            };
            let gate = WorkspaceChangeLearningGate {
                suite: required(node, "suite")?,
                passed: parse_usize(node, "passed")?,
                failed: parse_usize(node, "failed")?,
            };
            let approval = WorkspaceChangeLearningApproval::granted(required(node, "reviewer")?);
            ledger.promote(&candidate, gate, approval)?;
        }
        let expected = stable_id("workspace_change_recipe_ledger", &ledger.review_body());
        if root.id != expected {
            return Err(error("workspace_learning_ledger_content_address_mismatch"));
        }
        Ok(ledger)
    }
}

/// Execute an unseen equivalent substitution only through a promoted recipe.
pub fn execute_workspace_rewrite_with_recipe(
    ledger: &WorkspaceChangeRecipeLedger,
    source: &str,
    pattern: &str,
    replacement: &str,
) -> Result<WorkspaceRewriteExecution, WorkspaceChangeLearningError> {
    let stages = ledger
        .plan_for(WORKSPACE_CHANGE_TASK_FAMILY)
        .ok_or_else(|| error("workspace_learning_recipe_not_approved"))?;
    if stages != policy()?.stages {
        return Err(error("workspace_learning_recipe_stage_drift"));
    }
    execute_workspace_rewrite(source, pattern, replacement)
}

fn candidate_identity(task_family: &str, stages: &[String], evidence_count: usize) -> String {
    format!("{}\n{}\n{}", task_family, stages.join("\n"), evidence_count)
}

fn root_named<'a>(
    tree: &'a LinoNode,
    name: &str,
) -> Result<&'a LinoNode, WorkspaceChangeLearningError> {
    tree.children
        .iter()
        .find(|node| node.name == name)
        .ok_or_else(|| error(format!("workspace_learning_missing_{name}")))
}

fn required(node: &LinoNode, name: &str) -> Result<String, WorkspaceChangeLearningError> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| child.id.clone())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(format!("workspace_learning_missing_{name}")))
}

fn parse_usize(node: &LinoNode, name: &str) -> Result<usize, WorkspaceChangeLearningError> {
    required(node, name)?
        .parse::<usize>()
        .map_err(|_| error(format!("workspace_learning_invalid_{name}")))
}
