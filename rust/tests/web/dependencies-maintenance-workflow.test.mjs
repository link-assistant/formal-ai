import assert from 'node:assert/strict';
import {execFileSync,spawnSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,readFileSync,realpathSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {dependencyManifests} from '../../../scripts/dependency-manifest-inventory.mjs';
const workflow=readFileSync(new URL('../../../.github/workflows/dependencies-latest.yml',import.meta.url),'utf8');

function observedLocks(source) {
  const loop=source.match(/          while IFS= read -r lock; do\n([\s\S]*?)          done <<< "\$locks"/u);
  assert(loop,'maintained lock re-resolution loop missing');
  const directory=realpathSync(mkdtempSync(path.join(tmpdir(),'formal-ai-maintenance-locks-')));
  mkdirSync(path.join(directory,'nested'));
  const script='set -euo pipefail\n'+
    'bun() { printf "bun:%s\\n" "$PWD"; }\n'+
    'npm() { printf "npm:%s\\n" "$PWD"; }\n'+
    'while IFS= read -r lock; do\n'+loop[1]+
    'done <<\'LOCKS\'\nbun.lock\nnested/bun.lock\npackage-lock.json\nnested/package-lock.json\nignored.txt\nLOCKS\n';
  try {
    // Only shell fixture functions run; no dependency resolver or install executes.
    return execFileSync('bash',['-c',script],{cwd:directory,encoding:'utf8',timeout:3000})
      .trim().split('\n').map(line=>line.replace(directory,'ROOT'));
  } finally {rmSync(directory,{recursive:true,force:true});}
}

test('maintenance resolves root and nested locks with their actual owning tools',()=>{
  assert.deepEqual(observedLocks(workflow),['bun:ROOT','bun:ROOT/nested','npm:ROOT','npm:ROOT/nested']);
});

test('original nested-only case pattern demonstrably loses both root locks',()=>{
  const original=workflow.replace('bun.lock|*/bun.lock)','*/bun.lock)')
    .replace('package-lock.json|*/package-lock.json)','*/package-lock.json)');
  assert.deepEqual(observedLocks(original),['bun:ROOT/nested','npm:ROOT/nested']);
});

test('history-dependent maintenance gates receive full checkout history',()=>{
  assert.match(workflow,/- uses: actions\/checkout@v7\.0\.1\n        with:\n          fetch-depth: 0/u);
  assert.match(workflow,/Run registered gates \(stage rust\)/u);
});

function observeInventory(source, failAfterOutput) {
  const step=source.match(/      - name: Re-resolve the JavaScript lockfiles from the rewritten manifests\n        run: \|\n([\s\S]*?)(?=\n      - name:)/u);
  assert(step, 'complete resolver step required');
  const directory=realpathSync(mkdtempSync(path.join(tmpdir(),'formal-ai-lock-observation-')));
  const shell=`git() { printf 'package-lock.json\\n'; return ${failAfterOutput ? 23 : 0}; }\n`+
    'npm() { printf "resolved:%s\\n" "$PWD"; }\n'+
    'bun() { printf "resolved:%s\\n" "$PWD"; }\n'+step[1].replace(/^          /gmu,'');
  try {
    return spawnSync('bash',['-c',shell],{cwd:directory,encoding:'utf8',timeout:3000});
  } finally { rmSync(directory,{recursive:true,force:true}); }
}

test('failed lock inventory stops before any resolver even after partial output',()=>{
  const failed=observeInventory(workflow,true);
  assert.equal(failed.status,23,failed.stderr);
  assert.equal(failed.stdout,'','an incomplete inventory must not resolve a partial tree');
  const complete=observeInventory(workflow,false);
  assert.equal(complete.status,0,complete.stderr);
  assert.match(complete.stdout,/^resolved:/u);
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
  const step = source.match(/      - name: Re-resolve the Rust lockfiles from the rewritten manifests\n        run: \|\n([\s\S]*?)(?=\n      (?:#|- name:))/u);
  assert(step, 'complete Rust resolution step required');
  const shell = 'node() { printf "rust/Cargo.toml\\ntools/registry/Cargo.toml\\n"; return ' +
    (failInventory ? 23 : 0) + '; }\n' +
    'cargo() { printf "cargo:%s\\n" "$*"; }\n' + step[1].replace(/^          /gmu, '');
  return spawnSync('bash', ['-c', shell], {encoding: 'utf8', timeout: 3000});
}

test('the actual maintenance step resolves every active Cargo manifest and stops on partial inventory failure', () => {
  const result = observeCargoResolution(workflow);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.stdout.trim().split('\n'), [
    'cargo:update --workspace --manifest-path rust/Cargo.toml',
    'cargo:update --workspace --manifest-path tools/registry/Cargo.toml']);
  const failed = observeCargoResolution(workflow, true);
  assert.equal(failed.status, 23, failed.stderr);
  assert.equal(failed.stdout, '');
});
