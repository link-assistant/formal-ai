// The decomposition tree and the splitting executor of `crate::task_decomposition`
// (rust/src/task_decomposition.rs `decompose_task`, `SubTask`, `Decomposition`;
// rust/src/task_decomposition/recursive.rs `SplittingExecutor`;
// rust/src/task_decomposition/artifact.rs `artifact_links_notation`). The
// one-level checkable split itself lives in task_decomposition.mjs; this module
// builds the bounded tree on top of it, renders the reviewed Links Notation
// artifact, and gives the failure-driven controller (recursive_execution.mjs)
// the repository's real splitter. The artifact round-trip
// (`Decomposition::from_links_notation`) and the learning gate stay in Rust.

import { trim } from '../write_str.mjs';
import { stableId } from './engine_stable_id.mjs';
import { formatLinoRecord } from './links_format.mjs';
import {
  completionCriterionFor, plansFor, shippedLedger, splitOnceCheckable,
} from './task_decomposition.mjs';

/**
 * Mirrors `fn child_path` in rust/src/task_decomposition.rs: `1`, `2`, ... under the root; `1.1`, ... under a sub-task.
 * @param {string} parent
 * @param {number} index
 * @returns {string}
 */
function childPath(parent, index) {
  const number = index + 1;
  return parent === '' ? String(number) : `${parent}.${number}`;
}

/** Mirrors `fn leaf` in rust/src/task_decomposition.rs. */
function leaf(id, path, text, completionCriterion, depth, atomic, reason) {
  return { id, path, text, completion_criterion: completionCriterion, depth, atomic, reason, children: [] };
}

/** Mirrors `fn build` in rust/src/task_decomposition.rs. */
function build(text, path, depth, maxDepth, ledger) {
  const id = stableId('sub_task', `${path}:${depth}:${text}`);
  const parts = splitOnceCheckable(text, ledger).filter((part) => part !== text);
  const planned = parts.length < 2 ? plansFor(text, ledger) : null;
  const needsChildren = parts.length >= 2 || planned !== null;

  if (depth >= maxDepth && needsChildren) {
    return leaf(id, path, text, 'unresolved_depth_bound', depth, false, 'depth_bound');
  }
  if (planned !== null) {
    const children = planned.map((stage, index) => {
      const path2 = childPath(path, index);
      return leaf(
        stableId('sub_task', `${path2}:${depth + 1}:${stage.strategy_id}:${stage.stage_id}`),
        path2,
        stage.text,
        stage.completion_criterion,
        depth + 1,
        true,
        'direct_method',
      );
    });
    return { id, path, text, completion_criterion: 'all_children_pass', depth, atomic: false, reason: 'not_atomic', children };
  }
  if (parts.length < 2) {
    const criterion = completionCriterionFor(text);
    if (criterion !== null) return leaf(id, path, text, criterion, depth, true, 'direct_method');
    return leaf(id, path, text, 'unresolved_single_need', depth, false, 'single_need');
  }
  const children = parts.map((part, index) => build(part, childPath(path, index), depth + 1, maxDepth, ledger));
  return { id, path, text, completion_criterion: 'all_children_pass', depth, atomic: false, reason: 'not_atomic', children };
}

/**
 * Mirrors `fn decompose_task_with_ledger` in rust/src/task_decomposition.rs:
 * split `task` recursively until every leaf is atomic or `maxDepth` is reached.
 * @param {string} task
 * @param {number} maxDepth
 * @param {Set<string>} ledger the approved strategy ids
 */
export function decomposeTaskWithLedger(task, maxDepth, ledger) {
  const text = trim(task);
  return { task: text, max_depth: maxDepth, root: build(text, '', 0, maxDepth, ledger) };
}

/** Mirrors `fn decompose_task` in rust/src/task_decomposition.rs (the shipped, review-gated ledger). */
export function decomposeTask(task, maxDepth) {
  return decomposeTaskWithLedger(task, maxDepth, shippedLedger());
}

/** Mirrors `SubTask::collect_leaves` in rust/src/task_decomposition.rs. */
function collectLeaves(node, out) {
  if (node.children.length === 0) out.push(node);
  else for (const child of node.children) collectLeaves(child, out);
}

/** Mirrors `Decomposition::leaves` in rust/src/task_decomposition.rs: every leaf, in source order. */
export function leaves(decomposition) {
  const out = [];
  collectLeaves(decomposition.root, out);
  return out;
}

/** Mirrors `SubTask::to_recursive_task` in rust/src/task_decomposition.rs. */
export function toRecursiveTask(node) {
  return { id: node.id, goal: node.text, children: node.children.map(toRecursiveTask) };
}

/** Mirrors `SubTask::to_links_notation` in rust/src/task_decomposition.rs. */
export function subTaskLinksNotation(node) {
  const pairs = [
    ['record_type', 'sub_task'],
    ['path', node.path],
    ['text', node.text],
    ['completion_criterion', node.completion_criterion],
    ['depth', String(node.depth)],
    ['atomic', String(node.atomic)],
    ['atomicity_reason', node.reason],
    ...node.children.map((child) => ['child', child.id]),
  ];
  let out = formatLinoRecord(node.id, pairs);
  for (const child of node.children) out += `\n${subTaskLinksNotation(child)}`;
  return out;
}

/** Mirrors `fn artifact_id` in rust/src/task_decomposition/artifact.rs. */
function artifactId(task, maxDepth, treeDigest) {
  return stableId('task_decomposition', [task, String(maxDepth), treeDigest].join('\n'));
}

/**
 * Mirrors `Decomposition::artifact_links_notation` (also `to_links_notation`) in
 * rust/src/task_decomposition/artifact.rs: the whole decomposition as the
 * content-addressed Links Notation artifact a reviewer inspects.
 */
export function decompositionLinksNotation(decomposition) {
  const tree = subTaskLinksNotation(decomposition.root);
  const treeDigest = stableId('task_decomposition_tree', tree);
  const header = formatLinoRecord(artifactId(decomposition.task, decomposition.max_depth, treeDigest), [
    ['record_type', 'task_decomposition'],
    ['schema_version', '1'],
    ['task', decomposition.task],
    ['max_depth', String(decomposition.max_depth)],
    ['root', decomposition.root.id],
    ['tree_digest', treeDigest],
  ]);
  return [header, tree].join('\n');
}

/**
 * Mirrors `SplittingExecutor::with_ledger` in rust/src/task_decomposition/recursive.rs:
 * give any task executor the repository's real splitter. `attempt`,
 * `retry_after_children` and `extend_for` are delegated untouched; `split`
 * answers with one level of `decomposeTaskWithLedger` and records the split.
 * @param {object} inner the wrapped executor
 * @param {Set<string>} [ledger] the approved strategy ids (default: the shipped ledger)
 */
export function splittingExecutor(inner, ledger = shippedLedger()) {
  const atomicTasks = new Set();
  const splits = [];
  return {
    inner,
    splits,
    attempt: (task) => inner.attempt(task),
    extend_for: (task, failure) => inner.extend_for(task, failure),
    retry_after_children: (task) => inner.retry_after_children(task),
    split(task, failure, splitDepth) {
      if (atomicTasks.has(`${task.id}\0${task.goal}`)) {
        splits.push({ task_id: task.id, goal: task.goal, failure_evidence: failure.evidence, split_depth: splitDepth, children: [] });
        return [];
      }
      const decomposition = decomposeTaskWithLedger(task.goal, 1, ledger);
      for (const child of decomposition.root.children.filter((entry) => entry.atomic)) {
        atomicTasks.add(`${child.id}\0${child.text}`);
      }
      const children = decomposition.root.children.map(toRecursiveTask);
      splits.push({
        task_id: task.id,
        goal: task.goal,
        failure_evidence: failure.evidence,
        split_depth: splitDepth,
        children: children.map((child) => child.goal),
      });
      return children;
    },
  };
}

