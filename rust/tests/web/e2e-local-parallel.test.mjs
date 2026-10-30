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
import { loadPlaywrightConfig } from './support/playwright-configuration.mjs';

const E2E = `${REPO_ROOT}/rust/tests/e2e`;
const CONFIG = readFileSync(`${E2E}/playwright.local.config.js`, 'utf8');
const WORKFLOW = readFileSync(`${REPO_ROOT}/.github/workflows/e2e-local.yml`, 'utf8');

function loadConfig(env) {
  return loadPlaywrightConfig('rust/tests/e2e/playwright.local.config.js', env);
}

test('CI serves its prepared web assets without rebuilding during startup', () => {
  const { webServer } = loadConfig({ CI: 'true', E2E_PREBUILT_WEB: '1' });
  assert.equal(webServer.command, 'npx serve ../../../js --listen 3456 --no-clipboard');
  assert.equal(webServer.timeout, 120_000);
  assert.equal(webServer.stdout, 'pipe');
  assert.equal(webServer.stderr, 'pipe');
  assert.equal(webServer.reuseExistingServer, false);
  const prepare = WORKFLOW.indexOf('run: bun run build:web');
  const sync = WORKFLOW.indexOf('run: scripts/sync-seed.sh');
  const run = WORKFLOW.indexOf('- name: Run local e2e tests');
  assert(prepare >= 0 && sync > prepare && run > sync);
  assert.match(WORKFLOW.slice(run), /E2E_PREBUILT_WEB: '1'/u);
});

test('standalone startup still builds and syncs unless explicitly prepared', () => {
  for (const env of [{}, { CI: 'true' }, { E2E_PREBUILT_WEB: '0' }]) {
    const { webServer } = loadConfig(env);
    assert.match(webServer.command, /^bun run --cwd ..\/..\/.. build:web && ..\/..\/..\/scripts\/sync-seed.sh && npx serve /u);
    assert(webServer.timeout < loadConfig(env).globalTimeout);
  }
});

test('startup recovery regressions belong to exactly one real local browser project', () => {
  const config = loadConfig({ CI: 'true' });
  const spec = '**/worker-startup-recovery.spec.js';
  const matches = patterns => (patterns || []).includes(spec);
  assert(matches(config.testMatch), 'Recovery tests must be discovered by CI test listing');
  const selected = config.projects.filter(project =>
    matches(project.testMatch || config.testMatch) && !matches(project.testIgnore));
  assert.equal(selected.length, 1, 'Run recovery tests once, including after duration-based repartitioning');
});

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
