// Issue #1550 — the recursive procedure every answer goes through, in both
// runtimes (the Rust twin is rust/src/meta_executor.rs). At every level it
//
//   1. splits the formalized request into tasks by the relations the shared
//      grammar names (sequence, alternative, aggregation), in any supported
//      language and never by an English conjunction written in the code;
//   2. derives a test for each task as a first-class value that is executed;
//   3. drafts every candidate a method can give (the turn's cache row, the
//      direct derivation by the host solver, the composition of the selected
//      sub-results);
//   4. selects the smallest sufficient passing candidate by least action
//      (rankByLeastAction, the principle of issue #491) and records why;
//   5. composes the selected drafts by structure and checks the composition
//      against the parent test;
//   6. recurses into non-atomic tasks within the depth, breadth and budget
//      bounds, all of which are visible in the trace.
//
// Every word and number the procedure reads is data in the structural rule
// `task-decomposition` and the per-language `connective-*` vocabularies of
// data/seed/text-formalization.lino. The events this module emits contain
// only integers and strings derived from the request, so the JavaScript and
// native runtimes emit the same sequence for the same request and host.

import { grammarVocabulary, structuralRuleValues } from './formalization_grammar.mjs';
import { rankByLeastAction } from './least_action.mjs';
import { attemptFailed, attemptPassed, recursiveLeaf, solveRecursively } from './recursive_execution.mjs';
import { containsCjk } from './seed_meanings.mjs';
import { isAlphanumeric, isWhitespace, trimMatches } from './rust_str.mjs';

const RULE = 'task-decomposition';

/** The event kinds, in the order a level emits them. */
export const META_EXEC_EVENT_KINDS = Object.freeze([
  'meta_exec:bounds',
  'meta_exec:task',
  'meta_exec:test',
  'meta_exec:draft',
  'meta_exec:compose',
  'meta_exec:select',
  'meta_exec:unresolved',
  'meta_exec:result',
]);

function ruleValues(kind) {
  return structuralRuleValues(RULE, kind);
}

function ruleValue(kind) {
  const values = ruleValues(kind);
  if (values.length !== 1) throw Error('unique_task_decomposition_fact');
  return values[0];
}

function ruleNumber(kind) {
  const text = ruleValue(kind);
  if (!/^[0-9]+$/.test(text)) throw Error('task_decomposition_number');
  return Number(text);
}

/** The relations a level is split by, in precedence order. */
export function decompositionRelations() {
  return ruleValues('relation');
}

/**
 * The recursion bounds: the seed's values unless the caller overrides one.
 * @param {{depth?: number, breadth?: number, budget?: number}} [overrides]
 */
export function decompositionBounds(overrides = {}) {
  const pick = (name, kind) => {
    const value = overrides?.[name];
    return Number.isInteger(value) && value >= 0 ? value : ruleNumber(kind);
  };
  return {
    depth: pick('depth', 'depth-bound'),
    breadth: pick('breadth', 'breadth-bound'),
    budget: pick('budget', 'budget-bound'),
  };
}

function renderFields(fields) {
  return fields.map(([name, value]) => `${name}=${value}`).join(' ');
}

function emit(state, kind, fields) {
  state.events.push({ kind, payload: renderFields(fields) });
}

function quotedMask(characters) {
  const pairs = ruleValues('quote-pair').map((pair) => Array.from(pair));
  if (pairs.some((pair) => pair.length !== 2)) throw Error('task_decomposition_quote_pair');
  const mask = [];
  let closer = null;
  for (const character of characters) {
    if (closer !== null) {
      mask.push(true);
      if (character === closer) closer = null;
      continue;
    }
    const pair = pairs.find((candidate) => candidate[0] === character);
    if (pair) {
      closer = pair[1];
      mask.push(true);
      continue;
    }
    mask.push(false);
  }
  return mask;
}

function markerMatches(lowered, original, mask, at, marker, cjk) {
  const end = at + marker.length;
  if (end > lowered.length) return false;
  for (let offset = 0; offset < marker.length; offset += 1) {
    if (mask[at + offset] || lowered[at + offset] !== marker[offset]) return false;
  }
  if (cjk) return true;
  if (isAlphanumeric(marker[0]) && at > 0 && isAlphanumeric(original[at - 1])) return false;
  if (isAlphanumeric(marker[marker.length - 1]) && end < original.length
    && isAlphanumeric(original[end])) return false;
  return true;
}

function cleanPart(characters) {
  const edges = ruleValues('edge-punctuation');
  return trimMatches(characters.join(''), (character) =>
    isWhitespace(character) || edges.includes(character));
}

/**
 * One level of the split: the first relation, in precedence order, whose
 * connectives cut the request into at least two non-empty parts. Text inside
 * a quotation is never cut.
 * @param {string} text
 * @param {string} language
 * @returns {{relation: string, parts: string[]} | null}
 */
export function splitRequest(text, language) {
  const original = Array.from(String(text));
  const lowered = original.map((character) => character.toLowerCase());
  const mask = quotedMask(original);
  for (const relation of decompositionRelations()) {
    const vocabulary = ruleValue(relation + '-vocabulary');
    const markers = grammarVocabulary(language, vocabulary)
      .map((word) => ({
        chars: Array.from(word).map((character) => character.toLowerCase()),
        cjk: containsCjk(word),
      }))
      .filter((marker) => marker.chars.length > 0);
    // Longest connective first, so "and then" is one sequence marker.
    markers.sort((left, right) => right.chars.length - left.chars.length);
    const parts = [];
    let start = 0;
    let at = 0;
    while (at < original.length) {
      const marker = markers.find((candidate) =>
        markerMatches(lowered, original, mask, at, candidate.chars, candidate.cjk));
      if (marker) {
        parts.push(original.slice(start, at));
        at += marker.chars.length;
        start = at;
      } else {
        at += 1;
      }
    }
    parts.push(original.slice(start));
    const cleaned = parts.map(cleanPart).filter((part) => part.length > 0);
    if (parts.length > 1 && cleaned.length >= 2) return { relation, parts: cleaned };
  }
  return null;
}

/**
 * The task tree of a request: a task splits while a relation applies and the
 * depth and breadth bounds allow; otherwise it is atomic and says why.
 * @param {string} text
 * @param {string} language
 * @param {{depth: number, breadth: number}} bounds
 * @param {number} [depth]
 * @param {string} [id]
 */
export function planTasks(text, language, bounds, depth = 0, id = 't0') {
  const split = splitRequest(text, language);
  const node = { id, text, depth, relation: null, atomic: null, children: [] };
  if (!split) {
    node.atomic = 'no_relation';
  } else if (depth >= bounds.depth) {
    node.atomic = 'depth_bound';
  } else if (split.parts.length > bounds.breadth) {
    node.atomic = 'breadth_bound';
  } else {
    node.relation = split.relation;
    node.children = split.parts.map((part, index) =>
      planTasks(part, language, bounds, depth + 1, `${id}.${index + 1}`));
  }
  return node;
}

/** The text of every atomic task, left to right. */
export function taskLeaves(node) {
  return node.children.length ? node.children.flatMap(taskLeaves) : [node.text];
}

/** The shape of a task tree without its words: `sequence(leaf,aggregation(leaf,leaf))`. */
export function decompositionShape(node) {
  return node.children.length
    ? `${node.relation}(${node.children.map(decompositionShape).join(',')})`
    : 'leaf';
}

/**
 * The test of a task, as a value: the checks it executes and, for a
 * composite task, how many of its sub-tasks must pass.
 * @param {{id: string, relation: string|null, children: object[]}} node
 */
export function deriveTaskTest(node) {
  const checks = [ruleValue('leaf-check')];
  let requires = null;
  if (node.children.length) {
    checks.push(`${ruleValue('composite-check')}:${node.relation}`);
    requires = ruleValue(`${node.relation}-requires`);
  }
  return { task: node.id, checks, requires };
}

function childrenSatisfy(test, childOutcomes) {
  const resolved = childOutcomes.filter((outcome) => outcome.status === 'resolved').length;
  return test.requires === 'any' ? resolved >= 1 : resolved === childOutcomes.length;
}

/**
 * Execute a task test against one candidate. Returns every check executed and
 * the first one that failed.
 */
export function runTaskTest(test, candidate, childOutcomes) {
  const unresolved = ruleValues('unresolved-intent');
  const unresolvedSuffixes = ruleValues('unresolved-suffix');
  const executed = [];
  let failed = null;
  for (const check of test.checks) {
    executed.push(check);
    let passed;
    if (check === ruleValue('leaf-check')) {
      const intent = String(candidate?.intent ?? '');
      passed = Boolean(candidate) && !unresolved.includes(intent)
        && !unresolvedSuffixes.some((suffix) => intent.endsWith(suffix))
        && trimMatches(String(candidate.content ?? ''), isWhitespace).length > 0;
    } else {
      passed = childrenSatisfy(test, childOutcomes);
    }
    if (!passed && failed === null) failed = check;
  }
  return { passed: failed === null, failed, executed };
}

function compose(node, test, childOutcomes) {
  const resolved = childOutcomes.filter((outcome) => outcome.status === 'resolved');
  if (!childrenSatisfy(test, childOutcomes) || resolved.length === 0) return null;
  const chosen = test.requires === 'any' ? [resolved[0]] : resolved;
  const content = chosen.map((outcome) => outcome.candidate.content).join('\n\n');
  const confidence = Math.min(...chosen.map((outcome) => Number(outcome.candidate.confidence ?? 0)));
  const steps = ruleNumber('composition-cost')
    + chosen.reduce((total, outcome) => total + outcome.candidate.steps, 0);
  return {
    method: 'composition',
    intent: `${node.relation}_composition`,
    content,
    confidence,
    steps,
    parts: chosen.map((outcome) => outcome.task),
  };
}

function draftOf(method, steps, raw) {
  if (!raw) return null;
  return {
    method,
    intent: String(raw.intent ?? ''),
    content: String(raw.content ?? ''),
    confidence: Number(raw.confidence ?? 0),
    steps,
    source: raw,
  };
}

async function runTask(node, host, state, options) {
  emit(state, 'meta_exec:task', [
    ['id', node.id],
    ['depth', node.depth],
    ['relation', node.relation ?? '-'],
    ['children', node.children.length],
    ['atomic', node.atomic ?? '-'],
    ['text', node.text],
  ]);
  if (options.withheldTests.has(node.id)) {
    emit(state, 'meta_exec:test', [['task', node.id], ['checks', '-'], ['withheld', 'true']]);
    emit(state, 'meta_exec:unresolved', [['task', node.id], ['reason', 'no_test']]);
    return { task: node.id, status: 'unresolved', reason: 'no_test', candidate: null, fallback: null };
  }
  const test = deriveTaskTest(node);
  emit(state, 'meta_exec:test', [
    ['task', node.id],
    ['checks', test.checks.join(',')],
    ['requires', test.requires ?? '-'],
  ]);
  const childOutcomes = [];
  for (const child of node.children) {
    childOutcomes.push(await runTask(child, host, state, options));
  }

  const candidates = [];
  const remembered = state.memo.get(node.text) ?? (host.cached ? await host.cached(node.text, node) : null);
  const cached = draftOf('cache', ruleNumber('cache-cost'), remembered);
  if (cached) candidates.push(cached);
  let budgetExhausted = false;
  if (state.budget > 0) {
    state.budget -= 1;
    const direct = draftOf('derivation', ruleNumber('derivation-cost'), await host.draft(node.text, node));
    if (direct) candidates.push(direct);
  } else {
    budgetExhausted = true;
  }
  if (node.children.length) {
    const composition = compose(node, test, childOutcomes);
    emit(state, 'meta_exec:compose', [
      ['task', node.id],
      ['relation', node.relation],
      ['children', node.children.map((child) => child.id).join(',')],
      ['resolved', `${childOutcomes.filter((outcome) => outcome.status === 'resolved').length}/${childOutcomes.length}`],
      ['built', composition ? 'true' : 'false'],
    ]);
    if (composition) candidates.push(composition);
  }

  const ranked = [];
  for (const candidate of candidates) {
    const verdict = runTaskTest(test, candidate, childOutcomes);
    candidate.passed = verdict.passed;
    candidate.codeUnits = Array.from(candidate.content).length;
    emit(state, 'meta_exec:draft', [
      ['task', node.id],
      ['method', candidate.method],
      ['steps', candidate.steps],
      ['code_units', candidate.codeUnits],
      ['executed', verdict.executed.join(',')],
      ['passed', verdict.passed ? 'true' : 'false'],
      ['failed', verdict.failed ?? '-'],
    ]);
    ranked.push({
      id: candidate.method,
      solvesEntireRange: verdict.passed,
      cost: { steps: candidate.steps, codeUnits: candidate.codeUnits, computeMs: 0, memoryKb: 0 },
    });
  }
  if (budgetExhausted) {
    emit(state, 'meta_exec:draft', [['task', node.id], ['method', 'derivation'], ['skipped', 'budget']]);
  }
  const order = rankByLeastAction(ranked);
  const fallback = candidates.find((candidate) => candidate.method !== 'composition'
    && candidate.method !== 'cache') ?? null;
  if (order.length === 0) {
    const reason = candidates.length === 0
      ? (budgetExhausted ? 'budget' : 'no_draft')
      : (node.children.length && !childrenSatisfy(test, childOutcomes) ? 'children' : 'no_passing_draft');
    emit(state, 'meta_exec:unresolved', [['task', node.id], ['reason', reason]]);
    return { task: node.id, status: 'unresolved', reason, candidate: null, fallback };
  }
  const best = order[0];
  const selected = candidates.find((candidate) => candidate.method === best.id && candidate.passed);
  emit(state, 'meta_exec:select', [
    ['task', node.id],
    ['method', selected.method],
    ['reason', 'least_action'],
    ['steps', selected.steps],
    ['code_units', selected.codeUnits],
    ['passing', `${order.length}/${candidates.length}`],
  ]);
  state.memo.set(node.text, selected.source ?? selected);
  return { task: node.id, status: 'resolved', reason: null, candidate: selected, fallback };
}

/**
 * Run the whole procedure on one request.
 *
 * `host.draft(text, task)` is the direct derivation (the host solver on the
 * task's text) and returns `{intent, content, confidence, method?}` or null;
 * the optional `host.cached(text, task)` returns a stored row of an earlier
 * run of this procedure. Options: `bounds` overrides the seed's bounds and
 * `withheldTests` lists task ids whose test is removed.
 * @param {{text: string, language: string}} request
 * @param {{draft: Function, cached?: Function}} host
 * @param {{bounds?: object, withheldTests?: Iterable<string>}} [options]
 */
export async function executeProcedure(request, host, options = {}) {
  const bounds = decompositionBounds(options.bounds);
  const state = { events: [], budget: bounds.budget, memo: new Map() };
  emit(state, 'meta_exec:bounds', [
    ['depth', bounds.depth],
    ['breadth', bounds.breadth],
    ['budget', bounds.budget],
  ]);
  const root = planTasks(String(request.text), String(request.language), bounds);
  const outcome = await runTask(root, host, state, {
    withheldTests: new Set(options.withheldTests ?? []),
  });
  emit(state, 'meta_exec:result', [
    ['task', root.id],
    ['status', outcome.status],
    ['method', outcome.candidate?.method ?? '-'],
    ['budget_left', state.budget],
  ]);
  return { bounds, root, outcome, events: state.events };
}

/**
 * Issue #1550 step 1: the in-process executor of `solveRecursively`
 * (recursive_execution.mjs). One attempt runs the whole procedure on the
 * task's goal; the procedure splits by the grammar itself, so the executor
 * never splits a failure again and never extends a tool. The external-CLI
 * executor of orchestration stays one more executor of the same protocol.
 */
export function inProcessExecutor(host, language, options = {}) {
  const executor = {
    run: null,
    async attempt(task) {
      executor.run = await executeProcedure({ text: task.goal, language }, host, options);
      const last = executor.run.events[executor.run.events.length - 1];
      const evidence = `${last.kind} ${last.payload}`;
      return executor.run.outcome.status === 'resolved' ? attemptPassed(evidence) : attemptFailed(evidence);
    },
    split: async () => [],
    extend_for: async () => false,
    retry_after_children: async (task) => executor.attempt(task),
  };
  return executor;
}

/**
 * The single production entry: `solveRecursively` over the turn as the root
 * task, executed in process. Returns the procedure's run.
 * @param {{text: string, language: string}} request
 * @param {{draft: Function, cached?: Function}} host
 * @param {{bounds?: object, withheldTests?: Iterable<string>}} [options]
 */
export async function solveTurn(request, host, options = {}) {
  const executor = inProcessExecutor(host, String(request.language), options);
  await solveRecursively(recursiveLeaf('t0', String(request.text)), executor);
  return executor.run;
}

let nesting = 0;

/** Whether a sub-task solve is running, so the solver starts no nested run. */
export function isNested() {
  return nesting > 0;
}

/**
 * Run `body` as a sub-task solve: `isNested()` holds while it runs.
 * @param {() => Promise<any>} body
 */
export async function nested(body) {
  nesting += 1;
  try {
    return await body();
  } finally {
    nesting -= 1;
  }
}

const answerDraft = (answer) =>
  answer ? { intent: answer.intent, content: answer.content, confidence: answer.confidence } : null;

/**
 * Every answer goes through the procedure: the turn is the root task, its
 * direct derivation is `answer` (what the solver already derived), every
 * sub-task of a split request is solved by `solveSubTask` (nested, so it
 * starts no procedure of its own), and the composition replaces the root
 * answer only when least action selects it. Returns the answer with
 * `metaExecution` and the trace (in `solverEvents`) attached and the procedure's run (null when nested).
 * @param {{text: string, language: string}} request
 * @param {object} answer
 * @param {(text: string) => Promise<object>} solveSubTask
 * @param {{bounds?: object, withheldTests?: Iterable<string>}} [options]
 */
export async function solveAnswer(request, answer, solveSubTask, options = {}) {
  if (!answer || isNested()) return { answer, run: null };
  const host = {
    draft: async (text, task) => (task.depth === 0 ? answerDraft(answer) : answerDraft(await nested(() => solveSubTask(text)))),
  };
  const run = await solveTurn(request, host, options);
  const selected = run.outcome.candidate;
  const metaExecution = {
    status: run.outcome.status,
    method: selected?.method ?? null,
    shape: decompositionShape(run.root),
    events: run.events.map((event) => `${event.kind} ${event.payload}`),
  };
  const adopted = selected?.method === 'composition'
    ? { ...answer, intent: selected.intent, content: selected.content, confidence: selected.confidence }
    : answer;
  // The trace joins the solver events of the answer, as the native solver
  // appends it to its event log.
  const solverEvents = Array.isArray(answer.solverEvents)
    ? { solverEvents: answer.solverEvents.concat(run.events.map(({ kind, payload }) => ({ kind, payload }))) }
    : {};
  return { answer: { ...adopted, ...solverEvents, metaExecution }, run };
}
