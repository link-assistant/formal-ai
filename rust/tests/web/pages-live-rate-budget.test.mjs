import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';
import { EventEmitter } from 'node:events';
import { createRequire } from 'node:module';

const { withWorkerDiagnostics } = createRequire(import.meta.url)('../e2e/tests/support/worker-diagnostics.js');

const root = new URL('../../../', import.meta.url);
const read = relative => fs.readFileSync(new URL(relative, root), 'utf8');
const original = read('rust/tests/e2e/playwright.pages.config.js');
const smokeConfig = read('rust/tests/e2e/playwright.pages.smoke.config.js');
const smoke = read('rust/tests/e2e/tests/pages-live-smoke.spec.js');

function loadConfig(source, full) {
  const module = { exports: {} };
  vm.runInNewContext(source, {
    module,
    process: { env: {} },
    require: name => name === './playwright.pages.config'
      ? full : { defineConfig: value => value, devices: { 'Desktop Chrome': {} } },
  });
  return module.exports;
}

test('live budget is one worker, one spec, zero retries; full matrix remains available', () => {
  const full = loadConfig(original);
  const live = loadConfig(smokeConfig, full);
  assert.equal(full.retries, 2);
  assert(full.testMatch.includes('**/demo.spec.js'));
  assert(full.testMatch.includes('**/multilingual-*.spec.js'));
  assert.equal(live.testMatch.join(','), '**/pages-live-smoke.spec.js');
  assert.equal(live.workers, 1);
  assert.equal(live.retries, 0);
  assert.equal(live.fullyParallel, false);
  assert.equal(live.use.serviceWorkers, 'block');
  assert.equal(live.globalTimeout, full.globalTimeout);
  assert.equal(live.timeout, full.timeout);
  assert.equal(live.use.baseURL, full.use.baseURL);
});

// Execute the real smoke's assertions against delivery/runtime failure contracts.
// This is an oracle control, not a browser or WASM acceptance substitute.
async function exercise(change = {}) {
  let body;
  const scenario = {
    status: 200, mounted: true, mode: 'wasm worker', override: null,
    answer: '2 + 2 = 4', docs: true, ...change,
  };
  const navigations = [];
  let count = 0;
  const page = Object.assign(new EventEmitter(), {
    addInitScript: async () => {},
    goto: async target => {
      navigations.push(target);
      return { status: () => scenario.status };
    },
    evaluate: async () => scenario.override,
    locator: selector => ({
      selector,
      count: async () => count,
      last: () => ({ selector: 'answer' }),
      fill: async value => assert.equal(value, 'Can you calculate 2 + 2?'),
      click: async () => { count += 2; },
    }),
  });
  function expected(actual, inverse = false) {
    const check = condition => assert.equal(Boolean(condition), !inverse);
    const text = () => actual.selector === '.status' ? scenario.mode
      : actual.selector === 'answer' ? scenario.answer
      : actual.selector === '.hero h1' ? 'Documentation' : 'Manual mode';
    return {
      get not() { return expected(actual, !inverse); },
      toBe: value => check(actual === value),
      toBeVisible: () => check(actual.selector === '.app' ? scenario.mounted
        : actual.selector.includes('docs-app-summary') ? scenario.docs : true),
      toHaveText: value => check(text() === value),
      toBeEnabled: () => check(true),
      toContainText: value => check(text().includes(value)),
      toHaveCount: value => check(count === value),
      toHaveClass: value => check(value.test('message assistant')),
    };
  }
  vm.runInNewContext(smoke, {
    require: name => {
      if (name === '@playwright/test') return { test: (_name, callback) => { assert.equal(body, undefined); body = callback; }, expect: expected };
      if (name === './support/worker-ready') return { WORKER_READY_TIMEOUT_MS: 20_000 };
      if (name === './support/worker-diagnostics') return { withWorkerDiagnostics };
      throw Error('UnexpectedSmokeImport:' + name);
    },
  });
  await body({ page }, { attach: async () => {} });
  return navigations;
}

test('smoke visits landing, one real-engine app context, then docs', async () => {
  assert.deepEqual(await exercise(), ['../', './', '../docs/']);
});
for (const [name, failure] of [
  ['HTTP 429 delivery', { status: 429 }],
  ['missing app mount', { mounted: false }],
  ['unavailable engine', { mode: 'engine unavailable' }],
  ['JavaScript engine fallback', { mode: 'js fallback (diagnostic override)' }],
  ['enabled diagnostic override', { override: '1' }],
  ['incorrect real answer', { answer: '2 + 2 = 5' }],
  ['missing documentation', { docs: false }],
]) {
  test(`smoke refuses ${name}`, async () => {
    await assert.rejects(exercise(failure));
  });
}
