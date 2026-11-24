// Issue #1569: three agent-ladder leaves of run 42472083551 failed before
// reaching their own work, and with them every composite leg.
//   depth-1-1, depth-1-2: the sccache download answered HTTP 502/503. The
//     install step failed the job, and its post step failed again with
//     "Unable to locate executable file: undefined".
//   depth-3-1.2.1: oven-sh/setup-bun reported success without installing Bun.
//     Its promise never settled and Node exited 0. The next step failed with
//     "bun: command not found".
// The cache install is now allowed to fail, and the existing compiler probe
// falls back to the real compiler. Bun is installed by a composite that
// retries an install which ended without a Bun and proves that `bun` runs.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import fs from 'node:fs';
import {createRequire} from 'node:module';
import os from 'node:os';
import path from 'node:path';
import {test} from 'node:test';

const root = path.resolve(import.meta.dirname, '../../..');
const YAML = createRequire(import.meta.url)('yaml');
const load = file => YAML.parse(fs.readFileSync(path.join(root, file), 'utf8'));
const sccache = load('.github/actions/setup-sccache/action.yml');
const bun = load('.github/actions/setup-bun-verified/action.yml');
const named = (steps, name) => steps.find(step => step.name === name || step.id === name);

/** Run a composite `run:` block with only `bin` and the system tools on PATH. */
function runStep(t, source, {env = {}, tools = {}} = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'issue-1569-leaf-'));
  t.after(() => fs.rmSync(directory, {recursive: true, force: true}));
  const bin = path.join(directory, 'bin');
  fs.mkdirSync(bin);
  for (const [name, body] of Object.entries(tools)) fs.writeFileSync(path.join(bin, name), body, {mode: 0o700});
  const environmentFile = path.join(directory, 'environment');
  fs.writeFileSync(environmentFile, '');
  const run = spawnSync('bash', ['-e', '-c', source], {cwd: directory, encoding: 'utf8', timeout: 20000,
    env: {PATH: bin + path.delimiter + '/usr/bin:/bin', GITHUB_ENV: environmentFile, ...env}});
  return {...run, directory, environment: fs.readFileSync(environmentFile, 'utf8')};
}

test('a failed sccache download does not fail the job, in its main or post stage', () => {
  const install = named(sccache.runs.steps, 'Install sccache');
  assert.equal(install.id, 'install');
  assert.equal(install['continue-on-error'], true);
  assert.equal(install.uses, 'mozilla-actions/sccache-action@v0.0.11');
  const desktop = named(load('.github/workflows/desktop-release.yml').jobs.native.steps, 'Cache Rust compiler outputs');
  assert.equal(desktop['continue-on-error'], true);
});

test('without an installed sccache the probe falls back to the real compiler and names the failed install', t => {
  const probe = named(sccache.runs.steps, 'Start the sccache server');
  assert.equal(probe.env.SCCACHE_INSTALL_OUTCOME, '${{ steps.install.outcome }}');
  const observed = runStep(t, probe.run, {env: {SCCACHE_INSTALL_OUTCOME: 'failure'}});
  assert.equal(observed.status, 0, observed.stderr);
  assert.match(observed.stdout, /::warning title=sccache install failed::/u);
  assert.match(observed.stdout, /::warning title=sccache unavailable::/u);
  assert.match(observed.environment, /^RUSTC_WRAPPER=$/mu);
});

test('a healthy install prints no install warning', t => {
  const probe = named(sccache.runs.steps, 'Start the sccache server');
  const tool = '#!/bin/sh\n[ "$1" = rustc ] && exit 0\nexit 0\n';
  const observed = runStep(t, probe.run, {env: {SCCACHE_INSTALL_OUTCOME: 'success'}, tools: {sccache: tool}});
  assert.equal(observed.status, 0, observed.stderr);
  assert.doesNotMatch(observed.stdout, /sccache install failed|sccache unavailable/u);
  assert.equal(observed.environment, '');
});

test('the Bun composite retries only an install that reported no Bun', () => {
  const [first, backoff, second, verify] = bun.runs.steps;
  assert.equal(first.uses, 'oven-sh/setup-bun@v2');
  assert.equal(first['continue-on-error'], true);
  assert.equal(second.uses, 'oven-sh/setup-bun@v2');
  assert.equal(second['continue-on-error'], undefined, 'the second attempt is the last one');
  const retry = "${{ !cancelled() && steps.first.outputs.bun-path == '' }}";
  assert.equal(backoff.if, retry);
  assert.equal(second.if, retry);
  assert.deepEqual(second.with, first.with);
  assert.equal(verify.name, 'Verify Bun runs');
  assert.equal(verify.if, undefined, 'the verification always runs');
});

test('the verification refuses a missing Bun, a wrong Bun, and accepts the pinned one', t => {
  const verify = named(bun.runs.steps, 'Verify Bun runs').run;
  const pinned = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'issue-1569-pin-')), '.bun-version');
  t.after(() => fs.rmSync(path.dirname(pinned), {recursive: true, force: true}));
  fs.writeFileSync(pinned, '1.4.2\n');
  const env = {BUN_VERSION_FILE: pinned};
  const missing = runStep(t, verify, {env});
  assert.equal(missing.status, 1);
  assert.match(missing.stdout, /::error title=Bun is not installed::/u);
  const wrong = runStep(t, verify, {env, tools: {bun: '#!/bin/sh\necho 1.3.0\n'}});
  assert.equal(wrong.status, 1);
  assert.match(wrong.stdout, /::error title=Bun version mismatch::.*pins 1\.4\.2 and bun --version reports 1\.3\.0/u);
  const right = runStep(t, verify, {env, tools: {bun: '#!/bin/sh\necho 1.4.2\n'}});
  assert.equal(right.status, 0, right.stdout + right.stderr);
  assert.match(right.stdout, /^bun 1\.4\.2 at /mu);
  fs.writeFileSync(pinned, 'latest\n');
  const unpinned = runStep(t, verify, {env, tools: {bun: '#!/bin/sh\necho 1.5.0\n'}});
  assert.equal(unpinned.status, 0, 'a tag is not compared');
});

test('both ladder Bun setups use the verified composite', () => {
  const workflow = load('.github/workflows/issue-1028-agent-ladder.yml');
  const setups = Object.entries(workflow.jobs).flatMap(([job, body]) =>
    (body.steps ?? []).filter(step => step.name === 'Setup Bun').map(step => [job, step]));
  assert.ok(setups.length >= 2);
  for (const [job, step] of setups) {
    assert.equal(step.uses, './.github/actions/setup-bun-verified', job);
    assert.deepEqual(step.with, {'no-cache': 'true'}, job);
  }
  assert.ok(!fs.readFileSync(path.join(root, '.github/workflows/issue-1028-agent-ladder.yml'), 'utf8').includes('oven-sh/setup-bun'));
});


test('native transport replay uses the verified pinned Bun installer', () => {
  const workflow = load('.github/workflows/native-type-check.yml');
  const setup = workflow.jobs.clippy.steps.find(step => step.id === 'bun');
  assert.equal(setup.uses, './.github/actions/setup-bun-verified');
  assert.deepEqual(setup.with, {'bun-version-file': '.bun-version', 'no-cache': true});
  assert.equal(setup['continue-on-error'], undefined, 'a failed final install must fail the job');
  assert.equal(setup.if, "${{ !cancelled() && inputs.compile-inputs-changed && steps.native-lint.outcome == 'success' }}");
});
