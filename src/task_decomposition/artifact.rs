//! Durable, content-addressed task-decomposition artifacts.
//!
//! Execution must consume the tree a reviewer inspected, not silently run a
//! fresh decomposition that may differ. This module therefore round-trips the
//! exact tree and rejects changed fields, broken child links, and orphan nodes.

use std::collections::{BTreeMap, BTreeSet};

use crate::engine::stable_id;
use crate::links_format::format_lino_record;
use crate::meta_frame::AtomicityReason;
use crate::seed::parser::{parse_lino, LinoNode};

use super::{Decomposition, SubTask};

/// Why a serialized task decomposition could not be trusted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDecompositionArtifactError {
    /// Stable machine-readable failure reason.
    pub reason: String,
}

impl Decomposition {
    /// Restore an inspected decomposition only when its content address and
    /// complete child graph still match the serialized artifact.
    pub fn from_links_notation(document: &str) -> Result<Self, TaskDecompositionArtifactError> {
        let parsed = parse_lino(document);
        let header = parsed
            .children
            .iter()
            .find(|node| node.find_child_value("record_type") == "task_decomposition")
            .ok_or_else(|| error("missing_task_decomposition"))?;
        if header.find_child_value("schema_version") != "1" {
            return Err(error("unsupported_schema"));
        }
        let task = required(header, "task")?;
        let max_depth = required(header, "max_depth")?
            .parse::<u8>()
            .map_err(|_| error("invalid_max_depth"))?;
        let root_id = required(header, "root")?;
        let expected_tree_digest = required(header, "tree_digest")?;

        let nodes = parsed
            .children
            .iter()
            .filter(|node| node.find_child_value("record_type") == "sub_task")
            .map(|node| {
                if node.name.is_empty() {
                    return Err(error("missing_sub_task_id"));
                }
                Ok((node.name.clone(), node))
            })
            .collect::<Result<BTreeMap<_, _>, TaskDecompositionArtifactError>>()?;
        let serialized_node_count = parsed
            .children
            .iter()
            .filter(|node| node.find_child_value("record_type") == "sub_task")
            .count();
        if nodes.len() != serialized_node_count {
            return Err(error("duplicate_sub_task_id"));
        }

        let mut visited = BTreeSet::new();
        let root = restore_node(&root_id, &nodes, &mut visited)?;
        if visited.len() != nodes.len() {
            return Err(error("orphan_sub_task"));
        }
        validate_tree(&root, "", 0)?;
        let canonical_tree = root.to_links_notation();
        let tree_digest = stable_id("task_decomposition_tree", &canonical_tree);
        if tree_digest != expected_tree_digest {
            return Err(error("tree_integrity_failed"));
        }
        let expected_artifact_id = artifact_id(&task, max_depth, &tree_digest);
        if header.name != expected_artifact_id {
            return Err(error("artifact_integrity_failed"));
        }

        Ok(Self {
            task,
            max_depth,
            root,
        })
    }

    pub(super) fn artifact_links_notation(&self) -> String {
        let tree = self.root.to_links_notation();
        let tree_digest = stable_id("task_decomposition_tree", &tree);
        let header = format_lino_record(
            &artifact_id(&self.task, self.max_depth, &tree_digest),
            &[
                ("record_type", "task_decomposition".to_owned()),
                ("schema_version", "1".to_owned()),
                ("task", self.task.clone()),
                ("max_depth", self.max_depth.to_string()),
                ("root", self.root.id.clone()),
                ("tree_digest", tree_digest),
            ],
        );
        [header.as_str(), tree.as_str()].join("\n")
    }
}

fn restore_node(
    id: &str,
    nodes: &BTreeMap<String, &LinoNode>,
    visited: &mut BTreeSet<String>,
) -> Result<SubTask, TaskDecompositionArtifactError> {
    if !visited.insert(id.to_owned()) {
        return Err(error("cyclic_or_repeated_child"));
    }
    let node = nodes
        .get(id)
        .ok_or_else(|| error("missing_linked_sub_task"))?;
    let path = required(node, "path")?;
    let text = required(node, "text")?;
    let completion_criterion = required(node, "completion_criterion")?;
    let depth = required(node, "depth")?
        .parse::<u8>()
        .map_err(|_| error("invalid_depth"))?;
    let atomic = match required(node, "atomic")?.as_str() {
        "true" => true,
        "false" => false,
        _ => return Err(error("invalid_atomic")),
    };
    let reason = parse_reason(&required(node, "atomicity_reason")?)?;
    let children = node
        .children
        .iter()
        .filter(|child| child.name == "child")
        .map(|child| restore_node(&child.id, nodes, visited))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SubTask {
        id: id.to_owned(),
        path,
        text,
        completion_criterion,
        depth,
        atomic,
        reason,
        children,
    })
}

fn validate_tree(
    node: &SubTask,
    expected_path: &str,
    expected_depth: u8,
) -> Result<(), TaskDecompositionArtifactError> {
    if node.depth != expected_depth {
        return Err(error("invalid_child_depth"));
    }
    if node.path != expected_path {
        return Err(error("invalid_child_path"));
    }
    if !node.children.is_empty() && node.atomic {
        return Err(error("atomic_branch"));
    }
    if node.reason == AtomicityReason::DirectMethod && !node.atomic {
        return Err(error("invalid_direct_method_leaf"));
    }
    for (index, child) in node.children.iter().enumerate() {
        validate_tree(
            child,
            &super::child_path(expected_path, index),
            expected_depth.saturating_add(1),
        )?;
    }
    Ok(())
}

fn required(node: &LinoNode, name: &str) -> Result<String, TaskDecompositionArtifactError> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| child.id.clone())
        .ok_or_else(|| error(&format!("missing_{name}")))
}

fn parse_reason(value: &str) -> Result<AtomicityReason, TaskDecompositionArtifactError> {
    match value {
        "direct_method" => Ok(AtomicityReason::DirectMethod),
        "single_need" => Ok(AtomicityReason::SingleNeed),
        "depth_bound" => Ok(AtomicityReason::DepthBound),
        "not_atomic" => Ok(AtomicityReason::NotAtomic),
        _ => Err(error("invalid_atomicity_reason")),
    }
}

fn artifact_id(task: &str, max_depth: u8, tree_digest: &str) -> String {
    let max_depth = max_depth.to_string();
    let identity = [task, max_depth.as_str(), tree_digest].join("\n");
    stable_id("task_decomposition", &identity)
}

fn error(reason: &str) -> TaskDecompositionArtifactError {
    TaskDecompositionArtifactError {
        reason: reason.to_owned(),
    }
}
