// Issue #702 (R702-14, R702-15): the JavaScript halves of the nested-context
// work. The Rust twins are rust/tests/unit/issue_702_nested_contexts.rs:
//
// - `rust_and_browser_coding_inheritance_are_cycle_safe_not_depth_capped`
//   only reads the browser source for the absence of a depth constant; here the
//   worker's own `codingLanguageChain` (js/worker/formal_ai_worker_coding_idioms_and_text_manipulation.js) walks
//   an `extends` chain deeper than any former cap and stops on a cycle.
// - `nested_context_auto_learning_is_derived_and_review_gated` and
//   `real_agent_cli_learning_artifact_is_byte_reproducible`: the JavaScript
//   report (js/agentic/learning_report/context_hierarchy_learning.mjs) renders
//   the committed Agent CLI artifact byte for byte, from persisted memory.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { before, test } from 'node:test';

import * as contextHierarchyLearning from '../../../js/agentic/learning_report/context_hierarchy_learning.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost, evaluate } from '../../../js/server/worker-host.mjs';

const read = (path) => readFileSync(new URL(`../../../${path}`, import.meta.url), 'utf8');

let context;
before(async () => {
  context = await new WorkerHost().boot();
  await installNodeHost(new WorkerHost());
});

/** A coding-idiom catalog of `language` nodes, each extending the next. */
function catalog(edges) {
  return {
    name: 'coding_idioms',
    value: '',
    children: edges.map(([slug, parent]) => ({
      name: 'language',
      value: slug,
      children: parent ? [{ name: 'extends', value: parent, children: [] }] : [],
    })),
  };
}

/** The slugs `codingLanguageChain` visits in the worker realm, nearest first. */
function workerChain(edges, slug) {
  const expression = `JSON.stringify(codingLanguageChain(${JSON.stringify(catalog(edges))}, ${JSON.stringify(slug)}).map((node) => node.value))`;
  return JSON.parse(evaluate(context, expression));
}

test('the browser worker follows an extends chain 320 levels deep, nearest first', () => {
  const depth = 320;
  const edges = Array.from({ length: depth }, (_, index) => [`level_${index}`, index + 1 < depth ? `level_${index + 1}` : '']);
  const chain = workerChain(edges, 'level_0');
  assert.equal(chain.length, depth);
  assert.equal(chain[0], 'level_0');
  assert.equal(chain[1], 'level_1');
  assert.equal(chain.at(-1), 'level_319');
});

test('the browser worker stops on an extends cycle instead of looping', () => {
  assert.deepEqual(workerChain([['a', 'b'], ['b', 'c'], ['c', 'a']], 'a'), ['a', 'b', 'c']);
  assert.deepEqual(workerChain([['self', 'self']], 'self'), ['self']);
  assert.deepEqual(workerChain([['known', 'unknown']], 'known'), ['known']);
  assert.deepEqual(workerChain([['known', '']], 'missing'), []);
});

test('the JavaScript report renders the committed Agent CLI artifact byte for byte', () => {
  assert.equal(
    contextHierarchyLearning.renderDocument(),
    read('dev/log/issues/702/pulls/818/agent-cli/context-hierarchy-learning-report.lino'),
  );
});

test('the report ranking is derived from persisted memory and stays review gated', () => {
  const baseline = read('data/meta/issue-702-context-hierarchy-learning.lino');
  const first = contextHierarchyLearning.renderDocumentFrom(baseline);
  const second = contextHierarchyLearning.renderDocumentFrom(baseline.split('accessCount "8"').join('accessCount "18"'));
  assert.notEqual(first, second, 'the ranking must be derived from memory');
  for (const needle of [
    'decision "awaiting_human_review"',
    'promotion_gate "nested_context_runtime_and_parity_fixtures_pass"',
    'lesson:shared-context-hierarchy',
    'lesson:lazy-nearest-first',
    'lesson:explicit-inheritance',
    'lesson:cycle-safe-unbounded',
  ]) {
    assert.ok(first.includes(needle), needle);
  }
  assert.ok(!first.includes('decision "promoted"'));
  assert.ok(contextHierarchyLearning.isContextHierarchyLearningTask(contextHierarchyLearning.CONTEXT_HIERARCHY_LEARNING_TASK));
  assert.ok(contextHierarchyLearning.finalAnswer(first).includes('human-review-gated report'));
});
