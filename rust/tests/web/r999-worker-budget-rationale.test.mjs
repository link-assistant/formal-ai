// R999 (doctrine 2026-10-06, amending R995): a worker module's line ceiling in
// data/meta/worker-line-budget/ may rise when the growth is a JavaScript twin
// of a native handler, and the shard's rationale names the handler keys. The
// budget still forbids unexplained growth.
//
// The amendment is checked from the shards themselves:
// - every js/worker module has a budget shard with a written rationale, so no
//   ceiling exists without a reason;
// - no module is above its recorded ceiling, so growth always passes through a
//   reviewed shard edit (scripts/check-worker-line-budget.rs is the CI gate;
//   this keeps the same rule beside the amendment);
// - every rationale that invokes R999 names what grew: a handler key of
//   data/seed/handler-precedence.lino or a function the module defines.

import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const WORKER_DIR = `${REPO_ROOT}/js/worker`;
const BUDGET_DIR = `${REPO_ROOT}/data/meta/worker-line-budget`;

/** The budget shards, by module file name. */
function budgets() {
  const shards = new Map();
  for (const name of readdirSync(BUDGET_DIR).filter((entry) => entry.endsWith('.lino'))) {
    const text = readFileSync(`${BUDGET_DIR}/${name}`, 'utf8');
    const module = /^\s*module "([^"]+)"/m.exec(text)?.[1];
    const ceiling = Number(/^\s*ceiling (\d+)/m.exec(text)?.[1]);
    const rationale = /^\s*rationale "([\s\S]*)"\s*$/m.exec(text)?.[1] ?? '';
    shards.set(module, { name, ceiling, rationale });
  }
  return shards;
}

/** Line count as the gate counts it (`str::lines`: a trailing newline ends the last line). */
function lineCount(text) {
  if (text === '') return 0;
  const lines = text.split('\n');
  return text.endsWith('\n') ? lines.length - 1 : lines.length;
}

function workerModules() {
  return readdirSync(WORKER_DIR).filter((entry) => entry.endsWith('.js')).sort();
}

function handlerKeys() {
  const precedence = readFileSync(`${REPO_ROOT}/data/seed/handler-precedence.lino`, 'utf8');
  return new Set([...precedence.matchAll(/^\s*handler ([a-z0-9_]+)\s*$/gm)].map((match) => match[1]));
}

test('R999: every worker module has a budget shard with a written rationale', () => {
  const shards = budgets();
  for (const module of workerModules()) {
    const shard = shards.get(module);
    assert.ok(shard, `${module} has a budget shard`);
    assert.ok(shard.rationale.trim().length > 0, `${shard.name} says why ${module} may have its lines`);
  }
});

test('R999: no worker module grows past its reviewed ceiling', () => {
  const shards = budgets();
  for (const module of workerModules()) {
    const lines = lineCount(readFileSync(`${WORKER_DIR}/${module}`, 'utf8'));
    const { ceiling, name } = shards.get(module);
    assert.ok(lines <= ceiling, `${module} has ${lines} lines, past the ceiling ${ceiling} in ${name}`);
  }
});

test('R999: a ceiling raised under the amendment names the handler it twins', () => {
  const keys = handlerKeys();
  assert.ok(keys.size > 0, 'handler-precedence.lino declares handler keys');
  let invoked = 0;
  for (const [module, { name, rationale }] of budgets()) {
    if (!/\bR999\b|R997-R999/.test(rationale)) continue;
    invoked += 1;
    const source = readFileSync(`${WORKER_DIR}/${module}`, 'utf8');
    const identifiers = [...rationale.matchAll(/\b[A-Za-z_][A-Za-z0-9_]*\b/g)].map((match) => match[0]);
    const named = identifiers.filter(
      (identifier) =>
        keys.has(identifier) ||
        (/[a-z][A-Z]/.test(identifier) && new RegExp(`function\\s+${identifier}\\b`).test(source)),
    );
    assert.ok(named.length > 0, `${name} invokes R999 but names no handler key or function of ${module}`);
  }
  assert.ok(invoked > 0, 'at least one shard has used the amendment');
});

test('generated seed registry preserves every canonical seed within its existing budget', async () => {
  const { meaningSeeds, registryFileOf } = await import('../../../scripts/generate-worker-crate-modules.mjs');
  const { runInNewContext } = await import('node:vm');
  const registry = readFileSync(REPO_ROOT + '/data/meta/seed-registry.lino', 'utf8');
  const meanings = meaningSeeds(registry);
  const responses = meaningSeeds(registry, 'response');
  const actual = readFileSync(WORKER_DIR + '/formal_ai_worker_crate_seed_registry.js', 'utf8');
  const verify = text => {
    const context = { self: {} };
    runInNewContext(text, context, { timeout: 1000 });
    assert.deepEqual(Array.from(context.self.FORMAL_AI_CRATE_MEANING_SEEDS), meanings);
    assert.deepEqual(Array.from(context.self.FORMAL_AI_CRATE_RESPONSE_SEEDS), responses);
  };
  assert.equal(actual, registryFileOf(meanings, responses));
  verify(actual);
  const targetBudget = budgets().get('js/worker/formal_ai_worker_crate_seed_registry.js')
    ?? budgets().get('formal_ai_worker_crate_seed_registry.js');
  assert(targetBudget);
  assert(lineCount(actual) <= targetBudget.ceiling);
  for (const changed of [
    registryFileOf(meanings.slice(1), responses),
    registryFileOf([...meanings, meanings[0]], responses),
    registryFileOf([...meanings].reverse(), responses),
    registryFileOf(meanings, responses.slice(1)),
  ]) assert.throws(() => verify(changed));
});
