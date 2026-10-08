// `crate::recursive_execution` (rust/src/recursive_execution.rs): failure-driven
// recursive task execution. Try the parent, descend only after failure, split a
// childless failure into smaller tasks, extend the tool at an irreducible leaf,
// then climb back up and validate the parent after every child passes.
//
// Termination is structural: a split may only run while the node's split depth
// is below the bound, a child that repeats its parent's identity or goal is
// discarded, and a node whose split yields nothing usable is irreducible.
//
// A `TaskExecutor` here is an object with async `attempt(task)`,
// `extend_for(task, failure)`, `split(task, failure, splitDepth)` and
// `retry_after_children(task)` methods (the Rust trait is synchronous; the
// orchestration executor runs external processes, so each call is awaited).
// The trait's default `split` (`selection_heuristics::balanced_split`) is not
// ported: an executor without `split` declares every failure irreducible.

/** Mirrors `DEFAULT_SPLIT_DEPTH_BOUND` in rust/src/recursive_execution.rs. */
export const DEFAULT_SPLIT_DEPTH_BOUND = 4;

/** Mirrors `RecursiveTask::leaf` in rust/src/recursive_execution.rs. */
export function recursiveLeaf(id, goal) {
  return { id, goal, children: [] };
}

/** Mirrors `RecursiveTask::branch` in rust/src/recursive_execution.rs. */
export function recursiveBranch(id, goal, children) {
  return { id, goal, children };
}

/** Mirrors `TaskAttempt::passed` in rust/src/recursive_execution.rs. */
export function attemptPassed(evidence) {
  return { passed: true, evidence };
}

/** Mirrors `TaskAttempt::failed` in rust/src/recursive_execution.rs. */
export function attemptFailed(evidence) {
  return { passed: false, evidence };
}

/** Mirrors `RecursiveRun::is_passed` in rust/src/recursive_execution.rs. */
export function isPassed(run) {
  return run.status === 'passed';
}

/** Mirrors `RecursiveRun::executed_leaf_count` in rust/src/recursive_execution.rs. */
export function executedLeafCount(run) {
  return run.children.length === 0 ? 1 : run.children.reduce((total, child) => total + executedLeafCount(child), 0);
}

/**
 * Mirrors `RecursiveRun::split_depth_reached` in rust/src/recursive_execution.rs:
 * how deep failure-driven splitting went below this node.
 */
export function splitDepthReached(run) {
  const below = run.children.reduce((deepest, child) => Math.max(deepest, splitDepthReached(child)), 0);
  return Math.min(255, below + (run.split_applied ? 1 : 0));
}

/**
 * Mirrors `RecursiveRun::blocked_leaves` in rust/src/recursive_execution.rs: the
 * irreducible blocked nodes, in execution order.
 */
export function blockedLeaves(run) {
  if (run.children.length === 0) return isPassed(run) ? [] : [run];
  return run.children.flatMap(blockedLeaves);
}

/** Mirrors `fn terminal_state` in rust/src/recursive_execution.rs. */
function terminalState(attempts) {
  return attempts.length > 0 && attempts[attempts.length - 1].passed ? 'passed' : 'blocked';
}

/** Mirrors `fn usable_split` in rust/src/recursive_execution.rs: drop children that cannot shrink the problem. */
function usableSplit(parent, children) {
  return children.filter((child) => child.id !== parent.id && child.goal.trim() !== parent.goal.trim());
}

/** Mirrors `fn execute` in rust/src/recursive_execution.rs. */
async function execute(root, executor, splitDepth, splitDepthBound) {
  const first = await executor.attempt(root);
  if (first.passed) {
    return { task: root, attempts: [first], children: [], extension_applied: false, split_applied: false, status: 'passed' };
  }
  let supplied;
  let splitApplied = false;
  if (root.children.length === 0) {
    const split = splitDepth < splitDepthBound && executor.split
      ? usableSplit(root, await executor.split(root, first, splitDepth))
      : [];
    splitApplied = split.length > 0;
    supplied = split;
  } else {
    supplied = root.children;
  }
  if (supplied.length === 0) {
    const extensionApplied = await executor.extend_for(root, first);
    const attempts = [first];
    if (extensionApplied) attempts.push(await executor.attempt(root));
    return { task: root, attempts, children: [], extension_applied: extensionApplied, split_applied: false, status: terminalState(attempts) };
  }
  const childDepth = Math.min(255, splitDepth + (splitApplied ? 1 : 0));
  const children = [];
  for (const child of supplied) children.push(await execute(child, executor, childDepth, splitDepthBound));
  const attempts = [first];
  if (children.every(isPassed)) attempts.push(await executor.retry_after_children(root));
  return { task: root, attempts, children, extension_applied: false, split_applied: splitApplied, status: terminalState(attempts) };
}

/**
 * Mirrors `fn solve_recursively_within` in rust/src/recursive_execution.rs: the
 * shrink-on-failure protocol over `root` with an explicit bound on splitting.
 */
export function solveRecursivelyWithin(root, executor, splitDepthBound) {
  return execute(root, executor, 0, splitDepthBound);
}

/**
 * Mirrors `fn solve_recursively` in rust/src/recursive_execution.rs: splitting
 * stops at `DEFAULT_SPLIT_DEPTH_BOUND`.
 */
export function solveRecursively(root, executor) {
  return solveRecursivelyWithin(root, executor, DEFAULT_SPLIT_DEPTH_BOUND);
}
