// The principle of least action for task splitting and solution choice
// (issue #491; rust/src/least_action.rs).
//
// Work is planned as a balanced binary tree of subtasks: the highest levels
// of abstraction hold 1, 2, 4, 8, … tasks, and the deepest level holds the
// concrete subtasks. What is optimized is the number of smallest subtasks
// and the steps they cost. Two rules keep the scoring honest: a candidate
// that does not solve the entire range of inputs is never ranked, however
// short it is, and a subtask with no known solution is listed, never costed.
// Costs compare lexicographically: steps, then code size, then compute time,
// then memory, so a path that saves a millisecond by adding a step loses.

/**
 * The cost tuple of an `ActionCost` in comparison order.
 * @param {{steps: number, codeUnits: number, computeMs: number, memoryKb: number}} cost
 * @returns {Array<number>}
 */
function costKey(cost) {
  return [cost.steps, cost.codeUnits, cost.computeMs, cost.memoryKb];
}

/**
 * Lexicographic comparison of two number tuples of equal length: negative,
 * zero or positive.
 * @param {Array<number>} left
 * @param {Array<number>} right
 * @returns {number}
 */
function compareKeys(left, right) {
  for (let index = 0; index < left.length; index += 1) {
    if (left[index] !== right[index]) {
      return left[index] < right[index] ? -1 : 1;
    }
  }
  return 0;
}

/**
 * Mirrors `fn is_less_action_than` in rust/src/least_action.rs: fewer steps,
 * or equal steps and less code, and so on.
 * @param {{steps: number, codeUnits: number, computeMs: number, memoryKb: number}} cost
 * @param {{steps: number, codeUnits: number, computeMs: number, memoryKb: number}} other
 * @returns {boolean}
 */
export function isLessActionThan(cost, other) {
  return compareKeys(costKey(cost), costKey(other)) < 0;
}

/**
 * Mirrors `fn plan`: the balanced binary split of one root task over its
 * smallest subtasks. The ladder holds the powers of two below the subtask
 * count, and the final level holds the subtasks themselves.
 * @param {Array<{title: string, atomic: boolean, solutionSteps: number}>} subtasks
 * @returns {{levelCounts: Array<number>, smallestSubtasks: number, solved: number, unhandled: Array<string>, plannedSteps: number, depth: number}}
 */
export function plan(subtasks) {
  const levelCounts = [];
  for (let level = 1; level < subtasks.length; level *= 2) {
    levelCounts.push(level);
  }
  levelCounts.push(Math.max(subtasks.length, 1));
  const atomic = subtasks.filter((task) => task.atomic);
  return {
    levelCounts,
    smallestSubtasks: subtasks.length,
    solved: atomic.length,
    unhandled: subtasks.filter((task) => !task.atomic).map((task) => task.title),
    plannedSteps: atomic.reduce((total, task) => total + task.solutionSteps, 0),
    depth: levelCounts.length - 1,
  };
}

/**
 * Mirrors `fn least_action_plan`: of two plans over the same number of
 * smallest subtasks, the one with fewer planned steps, then fewer unhandled
 * paths; the first on a tie. `null` when the subtask counts differ.
 * @param {{smallestSubtasks: number, plannedSteps: number, unhandled: Array<string>}} left
 * @param {{smallestSubtasks: number, plannedSteps: number, unhandled: Array<string>}} right
 */
export function leastActionPlan(left, right) {
  if (left.smallestSubtasks !== right.smallestSubtasks) {
    return null;
  }
  const order = compareKeys(
    [left.plannedSteps, left.unhandled.length],
    [right.plannedSteps, right.unhandled.length],
  );
  return order > 0 ? right : left;
}

/**
 * Mirrors `fn rank_by_least_action`: the candidates that solve the entire
 * input range, least action first, ties broken by id.
 * @param {Array<{id: string, solvesEntireRange: boolean, cost: {steps: number, codeUnits: number, computeMs: number, memoryKb: number}}>} candidates
 */
export function rankByLeastAction(candidates) {
  return candidates
    .filter((candidate) => candidate.solvesEntireRange)
    .sort((left, right) => {
      const order = compareKeys(costKey(left.cost), costKey(right.cost));
      if (order !== 0) {
        return order;
      }
      if (left.id === right.id) {
        return 0;
      }
      return left.id < right.id ? -1 : 1;
    });
}

/**
 * Mirrors `fn least_action_solution`: the least-action candidate, or `null`
 * when none solves the entire range.
 * @param {Array<{id: string, solvesEntireRange: boolean, cost: {steps: number, codeUnits: number, computeMs: number, memoryKb: number}}>} candidates
 */
export function leastActionSolution(candidates) {
  return rankByLeastAction(candidates)[0] ?? null;
}
