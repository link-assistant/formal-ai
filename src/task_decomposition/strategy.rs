//! Data-backed strategies for tasks whose prose does not contain its own
//! independently executable operation contracts.

use std::sync::OnceLock;

use crate::language::detect as detect_language;
use crate::seed::{self, ROLE_DECOMPOSABLE_TASK_NOUN, parser::parse_lino};

use super::learning::TaskStrategyLedger;

pub const STRATEGIES_LINO: &str =
    include_str!("../../data/meta/task-decomposition-strategies.lino");
pub const CONTRACT_LINO: &str = include_str!("../../data/meta/task-decomposition-invariant.lino");
const TASK_PLACEHOLDER: &str = "{task}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDecompositionContract {
    pub atomic: String,
    pub execution: String,
    pub binary: String,
    pub learning: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStrategyStage {
    pub id: String,
    pub text_intent: String,
    pub completion_criterion: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDecompositionStrategy {
    pub id: String,
    pub activation: String,
    pub stages: Vec<TaskStrategyStage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedTaskStage {
    pub strategy_id: String,
    pub stage_id: String,
    pub text: String,
    pub completion_criterion: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShippedStrategyApproval {
    pub strategy_id: String,
    pub failed_task_id: String,
    pub failure_evidence: String,
    pub suite: String,
    pub passed: usize,
    pub reviewer: String,
}

#[must_use]
pub fn strategies() -> &'static [TaskDecompositionStrategy] {
    static REGISTRY: OnceLock<Vec<TaskDecompositionStrategy>> = OnceLock::new();
    REGISTRY.get_or_init(parse_strategies)
}

#[must_use]
pub fn contract() -> Option<&'static TaskDecompositionContract> {
    static CONTRACT: OnceLock<Option<TaskDecompositionContract>> = OnceLock::new();
    CONTRACT.get_or_init(parse_contract).as_ref()
}

#[must_use]
pub(super) fn shipped_approvals() -> Vec<ShippedStrategyApproval> {
    let tree = parse_lino(STRATEGIES_LINO);
    tree.children
        .iter()
        .find(|node| node.name == "task_decomposition_strategies")
        .map(|root| {
            root.children
                .iter()
                .filter(|node| node.name == "approved_strategy" && !node.id.is_empty())
                .filter_map(|node| {
                    let failed_task_id = node.find_child_value("failed_task_id");
                    let failure_evidence = node.find_child_value("failure_evidence");
                    let suite = node.find_child_value("suite");
                    let passed = node.find_child_value("passed").parse::<usize>().ok()?;
                    let failed = node.find_child_value("failed").parse::<usize>().ok()?;
                    let reviewer = node.find_child_value("reviewer");
                    if failed_task_id.is_empty()
                        || failure_evidence.is_empty()
                        || suite.is_empty()
                        || passed == 0
                        || failed != 0
                        || reviewer.is_empty()
                    {
                        return None;
                    }
                    Some(ShippedStrategyApproval {
                        strategy_id: node.id.clone(),
                        failed_task_id: failed_task_id.to_owned(),
                        failure_evidence: failure_evidence.to_owned(),
                        suite: suite.to_owned(),
                        passed,
                        reviewer: reviewer.to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[must_use]
pub fn plans_for(task: &str, ledger: &TaskStrategyLedger) -> Option<Vec<PlannedTaskStage>> {
    if !missing_operation_contract(task) {
        return None;
    }
    let strategy = strategies().iter().find(|strategy| {
        strategy.activation == "missing_operation_contract" && ledger.allows(&strategy.id)
    })?;
    let language = detect_language(task).slug();
    let stages = strategy
        .stages
        .iter()
        .map(|stage| PlannedTaskStage {
            strategy_id: strategy.id.clone(),
            stage_id: stage.id.clone(),
            text: localized_stage_text(&stage.text_intent, language, task),
            completion_criterion: stage.completion_criterion.clone(),
        })
        .collect::<Vec<_>>();
    (!stages.is_empty()).then_some(stages)
}

#[must_use]
pub fn missing_operation_contract(task: &str) -> bool {
    let normalized = task.to_lowercase();
    let lexicon = seed::lexicon();
    let names_work_item = lexicon.mentions_role(ROLE_DECOMPOSABLE_TASK_NOUN, &normalized)
        || lexicon.mentions_role_raw(ROLE_DECOMPOSABLE_TASK_NOUN, &normalized);
    let asks_for_software = lexicon
        .mentions_role(seed::ROLE_SOFTWARE_AUTHORING_ACTION, &normalized)
        || lexicon.mentions_role_raw(seed::ROLE_SOFTWARE_AUTHORING_ACTION, &normalized);
    let references_work_item = task.split_whitespace().any(is_repository_work_item);
    references_work_item
        || names_work_item
        || (asks_for_software && concrete_target(task).is_none())
}

#[must_use]
pub fn concrete_target(task: &str) -> Option<String> {
    task.split_whitespace().find_map(|token| {
        let cleaned = token.trim_matches(|character: char| {
            matches!(
                character,
                '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':' | '"' | '\''
            )
        });
        if cleaned.is_empty() || is_repository_work_item(cleaned) {
            return None;
        }
        let file_like = cleaned.contains('/')
            || cleaned
                .rsplit_once('.')
                .is_some_and(|(stem, extension)| !stem.is_empty() && !extension.is_empty());
        let identifier_like = cleaned.contains('_')
            || (cleaned.contains('-')
                && cleaned
                    .chars()
                    .all(|character| character.is_alphanumeric() || character == '-'));
        (file_like || identifier_like).then(|| cleaned.trim_end_matches('.').to_owned())
    })
}

fn parse_strategies() -> Vec<TaskDecompositionStrategy> {
    let tree = parse_lino(STRATEGIES_LINO);
    let Some(root) = tree
        .children
        .iter()
        .find(|node| node.name == "task_decomposition_strategies")
    else {
        return Vec::new();
    };
    root.children
        .iter()
        .filter(|node| node.name == "strategy" && !node.id.is_empty())
        .filter_map(|node| {
            let stages = node
                .children
                .iter()
                .filter(|child| child.name == "stage" && !child.id.is_empty())
                .filter_map(|stage| {
                    let text_intent = stage.find_child_value("text_intent");
                    let completion_criterion = stage.find_child_value("completion_criterion");
                    (!text_intent.is_empty() && !completion_criterion.is_empty()).then(|| {
                        TaskStrategyStage {
                            id: stage.id.clone(),
                            text_intent: text_intent.to_owned(),
                            completion_criterion: completion_criterion.to_owned(),
                        }
                    })
                })
                .collect::<Vec<_>>();
            let activation = node.find_child_value("activation");
            (!activation.is_empty() && !stages.is_empty()).then(|| TaskDecompositionStrategy {
                id: node.id.clone(),
                activation: activation.to_owned(),
                stages,
            })
        })
        .collect()
}

fn parse_contract() -> Option<TaskDecompositionContract> {
    let tree = parse_lino(CONTRACT_LINO);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "task_decomposition_contract")?;
    if root.find_child_value("record_type") != "meta_invariant" {
        return None;
    }
    let atomic = root.find_child_value("atomic");
    let execution = root.find_child_value("execution");
    let binary = root.find_child_value("binary");
    let learning = root.find_child_value("learning");
    if atomic.is_empty() || execution.is_empty() || binary.is_empty() || learning.is_empty() {
        return None;
    }
    Some(TaskDecompositionContract {
        atomic: atomic.to_owned(),
        execution: execution.to_owned(),
        binary: binary.to_owned(),
        learning: learning.to_owned(),
    })
}

fn localized_stage_text(intent: &str, language: &str, task: &str) -> String {
    seed::localized_response(intent, language)
        .unwrap_or_default()
        .replace(TASK_PLACEHOLDER, task)
}

fn is_repository_work_item(token: &str) -> bool {
    let path = token
        .trim_end_matches(['.', '。', '!', '?'])
        .strip_prefix("https://github.com/")
        .or_else(|| token.strip_prefix("http://github.com/"));
    let Some(path) = path else {
        return false;
    };
    let segments = path.split('/').collect::<Vec<_>>();
    segments.len() == 4
        && matches!(segments[2], "issues" | "pull")
        && segments[3]
            .chars()
            .all(|character| character.is_ascii_digit())
}
