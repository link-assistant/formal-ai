import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {existsSync,mkdtempSync,mkdirSync,readFileSync,realpathSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import YAML from 'yaml';
import {dependencyManifests} from '../../../scripts/dependency-manifest-inventory.mjs';
import {lockFixture} from '../../../experiments/issue-1580/dependency-lock-fixture.mjs';
import '../../../experiments/issue-1580/dependency-lock-literal-paths.test.mjs';
const workflow=readFileSync(new URL('../../../.github/workflows/dependencies-latest.yml',import.meta.url),'utf8');

function observedLocks(source) {
  const fixture = lockFixture();
  try {
    const result = fixture.run('javascript', {source});
    assert.equal(result.status, 0, result.stderr);
    return result.calls.map(call => call.tool + ':' + call.cwd.replace(fixture.directory, 'ROOT')).sort();
  } finally {fixture.close();}
}

test('maintenance resolves root and nested locks with their actual owning tools', () => {
  assert.deepEqual(observedLocks(workflow), ['bun:ROOT', 'bun:ROOT/nested', 'npm:ROOT', 'npm:ROOT/nested']);
});

test('original nested-only case pattern demonstrably loses both root locks', () => {
  const original = YAML.parse(workflow);
  const step = original.jobs.update.steps.find(candidate =>
    candidate.name === 'Re-resolve the JavaScript lockfiles from the rewritten manifests');
  step.run = [
    'set -euo pipefail',
    'locks="$(git ls-files -- \'*bun.lock\' \'*package-lock.json\')"',
    'while IFS= read -r lock; do',
    '  directory="$(dirname "$lock")"',
    '  case "$lock" in',
    '    */bun.lock) (cd "$directory" && bun install --lockfile-only) ;;',
    '    */package-lock.json) (cd "$directory" && npm install --package-lock-only --ignore-scripts --no-audit --no-fund) ;;',
    '  esac',
    'done <<< "$locks"',
  ].join('\n');
  assert.deepEqual(observedLocks(YAML.stringify(original)), ['bun:ROOT/nested', 'npm:ROOT/nested']);
});

test('history-dependent maintenance gates receive full checkout history',()=>{
  assert.match(workflow,/- uses: actions\/checkout@v7\.0\.1\n        with:\n          fetch-depth: 0/u);
  assert.match(workflow,/Run registered gates \(stage rust\)/u);
});

function observeInventory(source, failAfterOutput) {
  const fixture = lockFixture();
  try {
    return fixture.run('javascript', {source, gitMode: failAfterOutput ? 'failed' : undefined});
  } finally {fixture.close();}
}

test('failed lock inventory stops before any resolver even after partial output', () => {
  const failed = observeInventory(workflow, true);
  assert.equal(failed.status, 23, failed.stderr);
  assert.deepEqual(failed.calls, [], 'an incomplete inventory must not resolve a partial tree');
  const complete = observeInventory(workflow, false);
  assert.equal(complete.status, 0, complete.stderr);
  assert.equal(complete.calls.length, 4);
});


function manifestFixture() {
  const directory = realpathSync(mkdtempSync(path.join(tmpdir(), 'formal-ai-manifest-coverage-')));
  const write = (name, text) => {
    mkdirSync(path.dirname(path.join(directory, name)), {recursive: true});
    writeFileSync(path.join(directory, name), text);
  };
  const policy = {schemaVersion: 1, activeCargo: ['rust/Cargo.toml', 'tools/registry/Cargo.toml'],
    preservedCargo: {'historical/Cargo.toml': {kind: 'historical-reproducer', reason: 'Preserve a published failure',
      issue: 'https://github.com/example/reproducer/issues/1'}}};
  write('scripts/dependency-manifests.json', JSON.stringify(policy));
  for (const name of [...policy.activeCargo, ...Object.keys(policy.preservedCargo)]) write(name, '[dependencies]\nregex = "1"\n');
  write('package.json', '{}');
  write('nested/package.json', '{}');
  execFileSync('git', ['init', '--quiet'], {cwd: directory});
  execFileSync('git', ['add', '.'], {cwd: directory});
  return {directory, policy, write, stage: () => execFileSync('git', ['add', '.'], {cwd: directory}),
    close: () => rmSync(directory, {recursive: true, force: true})};
}

test('tracked coverage includes nested active Cargo and every npm manifest while preserving the reproducer', () => {
  const fixture = manifestFixture();
  try {
    const found = dependencyManifests(fixture.directory);
    assert.deepEqual(found.activeCargo, ['rust/Cargo.toml', 'tools/registry/Cargo.toml']);
    assert.deepEqual(found.preservedCargo, ['historical/Cargo.toml']);
    assert.deepEqual(found.npm, ['nested/package.json', 'package.json']);
    assert.equal(found.cargo.length, 3);
  } finally {fixture.close();}
});

test('a new tracked Cargo manifest requires a classification', () => {
  const fixture = manifestFixture();
  try {
    fixture.write('new/Cargo.toml', '[dependencies]\nregex = "1"\n');
    fixture.stage();
    assert.throws(() => dependencyManifests(fixture.directory), /Unclassified Cargo manifests: new\/Cargo.toml/u);
  } finally {fixture.close();}
});

test('missing active files, duplicate classifications and unexplained preserved files fail', () => {
  const fixture = manifestFixture();
  try {
    rmSync(path.join(fixture.directory, 'tools/registry/Cargo.toml'));
    assert.throws(() => dependencyManifests(fixture.directory), /ENOENT/u);
    fixture.write('tools/registry/Cargo.toml', '[dependencies]\n');
    fixture.policy.activeCargo.push('rust/Cargo.toml');
    fixture.write('scripts/dependency-manifests.json', JSON.stringify(fixture.policy));
    assert.throws(() => dependencyManifests(fixture.directory), /duplicate Cargo classification/u);
    fixture.policy.activeCargo.pop();
    fixture.policy.preservedCargo['historical/Cargo.toml'].reason = '';
    fixture.write('scripts/dependency-manifests.json', JSON.stringify(fixture.policy));
    assert.throws(() => dependencyManifests(fixture.directory), /requires its purpose and issue/u);
  } finally {fixture.close();}
});

test('failed and incomplete Git observations cannot authorize a partial manifest inventory', () => {
  const fixture = manifestFixture();
  try {
    assert.throws(() => dependencyManifests(fixture.directory, () =>
      ({status: 23, stdout: 'rust/Cargo.toml\0', stderr: 'read failed'})), /Cannot enumerate complete/u);
    assert.throws(() => dependencyManifests(fixture.directory, () =>
      ({status: 0, stdout: 'rust/Cargo.toml', stderr: ''})), /Incomplete tracked-file inventory/u);
  } finally {fixture.close();}
});

function observeCargoResolution(source, failInventory = false) {
  const fixture = lockFixture();
  try {
    return fixture.run('cargo', {source, gitMode: failInventory ? 'failed' : undefined});
  } finally {fixture.close();}
}

test('the actual maintenance step resolves every active Cargo manifest and stops on partial inventory failure', () => {
  const result = observeCargoResolution(workflow);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.calls.map(call => [call.tool, ...call.args]), [
    ['cargo', 'update', '--workspace', '--manifest-path', 'rust/Cargo.toml'],
    ['cargo', 'update', '--workspace', '--manifest-path', 'tools/registry/Cargo.toml']]);
  const failed = observeCargoResolution(workflow, true);
  assert.equal(failed.status, 23, failed.stderr);
  assert.deepEqual(failed.calls, []);
});

// Run the real sparse patterns rather than treating an include-string match as checkout evidence.
test('both native admission checkouts retain every active Cargo manifest', {timeout: 15000}, () => {
  const native = YAML.parse(readFileSync(new URL('../../../.github/workflows/native-type-check.yml', import.meta.url), 'utf8'));
  const policy = JSON.parse(readFileSync(new URL('../../../scripts/dependency-manifests.json', import.meta.url), 'utf8'));
  const checkouts = native.jobs.clippy.steps.filter(step => ['native-checkout', 'native-checkout-retry'].includes(step.id));
  assert.equal(checkouts.length, 2);
  const directory = realpathSync(mkdtempSync(path.join(tmpdir(), 'formal-ai-native-manifests-')));
  const write = (name, content) => {
    mkdirSync(path.dirname(path.join(directory, name)), {recursive: true});
    writeFileSync(path.join(directory, name), content);
  };
  const git = (args, input) => execFileSync('git', [
    '-c', 'core.hooksPath=/dev/null', '-c', 'commit.gpgsign=false', ...args,
  ], {cwd: directory, input, encoding: 'utf8', timeout: 3000, maxBuffer: 256 * 1024});
  const locks = policy.activeCargo.map(name => name.replace(/Cargo[.]toml$/u, 'Cargo.lock'))
    .filter(name => existsSync(new URL('../../../' + name, import.meta.url)));
  const omitted = 'experiments/issue-1580/unrelated-sparse-fixture.txt';
  try {
    for (const name of policy.activeCargo) write(name, '[dependencies]\n');
    for (const name of locks) write(name, 'version = 4\n');
    write(omitted, 'unrelated experiment stays outside native admission\n');
    git(['init', '--quiet']);
    git(['add', '.']);
    git(['-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
      'commit', '--quiet', '-m', 'native sparse input fixture']);
    for (const checkout of checkouts) {
      assert.equal(checkout.with['sparse-checkout-cone-mode'], false);
      git(['sparse-checkout', 'set', '--no-cone', '--stdin'], checkout.with['sparse-checkout']);
      for (const name of policy.activeCargo) {
        assert.equal(existsSync(path.join(directory, name)), true, checkout.id + ' omitted active manifest ' + name);
        assert.equal(readFileSync(path.join(directory, name), 'utf8'), '[dependencies]\n');
      }
      for (const name of locks) assert.equal(readFileSync(path.join(directory, name), 'utf8'), 'version = 4\n', checkout.id + ' omitted owning lock ' + name);
      assert.equal(existsSync(path.join(directory, omitted)), false, 'retain the bounded experiment exclusion');
    }
  } finally {rmSync(directory, {recursive: true, force: true});}
});
