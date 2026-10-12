// R1188-U10 and U11: the local web E2E suite starts its long specs first and
// runs parallel at test level. The legs are planned longest-first from
// data/meta/playwright-test-durations.lino by scripts/plan-test-shards.mjs
// (.github/workflows/e2e-local.yml), the config runs the recorded long specs
// in a leading `chromium-long` project, and `fullyParallel` spreads a file's
// tests over the workers, except in a file whose tests share state, which
// says so with `test.describe.configure({ mode: 'default' })`.

import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const E2E = `${REPO_ROOT}/rust/tests/e2e`;
const CONFIG = readFileSync(`${E2E}/playwright.local.config.js`, 'utf8');
const WORKFLOW = readFileSync(`${REPO_ROOT}/.github/workflows/e2e-local.yml`, 'utf8');

test('the legs are planned longest-first, never cut by test count', () => {
  assert.match(WORKFLOW, /plan-test-shards\.mjs --shard "\$SHARD" --of 3 --durations "\$durations"/u);
  assert.match(WORKFLOW, /plan-test-shards\.mjs --of 3 --durations "\$durations" --check/u);
  assert.doesNotMatch(WORKFLOW, /--shard=/u);
});

test('the long specs run in the project that starts first', () => {
  const projects = CONFIG.slice(CONFIG.indexOf('projects: ['));
  assert.ok(projects.indexOf("name: 'chromium-long'") < projects.indexOf("name: 'chromium',"));
  assert.match(projects, /testMatch: LONG_SPECS/u);
  assert.match(projects, /testIgnore: LONG_SPECS/u);
  const durations = readFileSync(`${REPO_ROOT}/data/meta/playwright-test-durations.lino`, 'utf8');
  assert.match(durations, /^ {2}test "tests\/[\w.-]+\.spec\.js"\n {4}seconds \d/mu);
});

test('tests run in parallel, and only files that share state stay in one worker', () => {
  assert.match(CONFIG, /^ {2}fullyParallel: true,$/mu);
  const specs = readdirSync(`${E2E}/tests`).filter((file) => file.endsWith('.spec.js'));
  for (const file of specs) {
    const source = readFileSync(`${E2E}/tests/${file}`, 'utf8');
    if (/\bbeforeAll\b/u.test(source)) {
      assert.match(source, /test\.describe\.configure\(\{ mode: '(?:default|serial)' \}\)/u, `${file} shares state in beforeAll`);
    }
  }
});
