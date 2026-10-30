import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import YAML from 'yaml';
import {completedDesktopPublication} from '../../../scripts/release-extension-selection.mjs';

// Observed no-op producer 40991980924 caused verifier 40992077211 to fail.
const expected = {repository: 'link-assistant/formal-ai', runId: 40991980924, attempt: 1};
function fixture() {
  const run = {id: expected.runId, run_attempt: 1,
    repository: {full_name: expected.repository}, head_repository: {full_name: expected.repository},
    head_branch: 'main', path: '.github/workflows/desktop-release.yml', status: 'completed',
    event: 'workflow_run', conclusion: 'success', head_sha: '5d2f63a4a40b3d758c9b20cb1e9d33b8f98e9bbd'};
  const names = ['Resolve target release', 'base / Pin base commit', 'Select exact native release source',
    'Package VS Code extension (.vsix)', 'Produce native ${{ matrix.target }}',
    'Build CLI ${{ matrix.label }}', 'Build ${{ matrix.label }}', 'Publish SHA256SUMS.txt + provenance'];
  const jobs = {total_count: names.length, jobs: names.map((name, i) => ({id: i + 1, name,
    run_id: expected.runId, run_attempt: 1, status: 'completed', conclusion: i === 0 ? 'success' : 'skipped'}))};
  return {run, jobs};
}
function observe(value) {
  const reads = [];
  const base = 'repos/' + expected.repository + '/actions/runs/' + expected.runId + '/attempts/1';
  const result = completedDesktopPublication(expected, {json(endpoint) {
    reads.push(endpoint);
    if (endpoint === base) return value.run;
    assert.equal(endpoint, base + '/jobs?per_page=100');
    return value.jobs;
  }, bytes() {assert.fail('inactive producer must not download an artifact');}});
  assert.equal(reads.length, 2);
  return result;
}
test('real skipped-producer shape is inactive without release fallback or publication authority', () => {
  assert.deepEqual(observe(fixture()), {active: false,
    reason: 'authenticated-desktop-publication-inactive', publicationAuthority: false});
});
for (const [name, change] of [
  ['wrong run', f => f.run.id++], ['wrong attempt', f => f.run.run_attempt++],
  ['foreign repository', f => f.run.repository.full_name = 'other/repository'],
  ['foreign head', f => f.run.head_repository.full_name = 'other/repository'],
  ['other branch', f => f.run.head_branch = 'feature'], ['other workflow', f => f.run.path = '.github/workflows/other.yml'],
  ['unfinished run', f => f.run.status = 'pending'], ['failed run', f => f.run.conclusion = 'failure'],
  ['unsupported event', f => f.run.event = 'pull_request'], ['bad SHA', f => f.run.head_sha = 'main'],
  ['incomplete inventory', f => f.jobs.total_count++],
  ['missing source job', f => {f.jobs.jobs.splice(2, 1); f.jobs.total_count--;}],
  ['duplicate job identity', f => f.jobs.jobs[1].id = f.jobs.jobs[0].id],
  ['foreign job run', f => f.jobs.jobs[2].run_id++], ['foreign job attempt', f => f.jobs.jobs[2].run_attempt++],
  ['unfinished job', f => f.jobs.jobs[2].status = 'queued'],
  ['failed source', f => f.jobs.jobs[2].conclusion = 'failure'],
  ['failed resolver', f => f.jobs.jobs[0].conclusion = 'failure'],
  ['partial publication', f => f.jobs.jobs[7].conclusion = 'success'],
  ['partial product', f => f.jobs.jobs[4].conclusion = 'success'],
]) test('inactive classification refuses ' + name, () => {
  const value = fixture(); change(value); assert.throws(() => observe(value));
});
test('successful source selection continues normal artifact and release verification', () => {
  const value = fixture(); value.jobs.jobs[2].conclusion = 'success';
  assert.equal(observe(value).active, true);
});
test('workflow gates every download and selection upload on an active producer', () => {
  const workflow = YAML.parse(fs.readFileSync(new URL('../../../.github/workflows/verify-release-downloads.yml', import.meta.url), 'utf8'));
  assert.equal(workflow.jobs.resolve.outputs.active, '${{ steps.select.outputs.active }}');
  for (const name of ['all-public-bytes', 'macos-public-packages', 'browser-public-package']) {
    assert.equal(workflow.jobs[name].if, "needs.resolve.outputs.active == 'true'");
  }
  const upload = workflow.jobs.resolve.steps.find(step => step.uses === 'actions/upload-artifact@v7');
  assert.equal(upload.if, "steps.select.outputs.active == 'true'");
});
test('real CLI resolves recorded inactive API data without network, products or latest-release fallback', () => {
  const directory = fs.mkdtempSync(path.join(tmpdir(), 'formal-ai-inactive-publication-'));
  try {
    fs.writeFileSync(path.join(directory, 'fixture.json'), JSON.stringify(fixture()));
    fs.writeFileSync(path.join(directory, 'gh'), `#!/usr/bin/env node
const fs=require('node:fs'),path=require('node:path');
const fixture=JSON.parse(fs.readFileSync(path.join(__dirname,'fixture.json'),'utf8'));
const args=process.argv.slice(2),base='repos/link-assistant/formal-ai/actions/runs/40991980924/attempts/1';
if(args.length!==2||args[0]!=='api')throw Error('unexpected command');
fs.appendFileSync(path.join(__dirname,'calls.jsonl'),JSON.stringify(args)+'\\n');
if(args[1]===base)process.stdout.write(JSON.stringify(fixture.run));
else if(args[1]===base+'/jobs?per_page=100')process.stdout.write(JSON.stringify(fixture.jobs));
else throw Error('unexpected release/artifact/network request');
`, {mode: 0o755});
    const env = {...process.env, PATH: directory + path.delimiter + process.env.PATH,
      RUNNER_TEMP: directory, GITHUB_REPOSITORY: expected.repository,
      PRODUCER_RUN_ID: String(expected.runId), PRODUCER_ATTEMPT: '1',
      RELEASE_VERIFY_MODE: 'resolve', GITHUB_OUTPUT: path.join(directory, 'outputs'),
      RELEASE_DOWNLOAD_DIRECTORY: path.join(directory, 'downloads')};
    for (const key of ['GH_TOKEN', 'GITHUB_TOKEN', 'RELEASE_SELECTION', 'RELEASE_TAG', 'RELEASE_ID', 'RELEASE_SOURCE_SHA', 'RELEASE_SOURCE_TREE']) delete env[key];
    const cli = fileURLToPath(new URL('../../../scripts/verify-public-release-downloads.mjs', import.meta.url));
    const run = mode => spawnSync(process.execPath, [cli], {env: {...env, RELEASE_VERIFY_MODE: mode},
      encoding: 'utf8', timeout: 10000, maxBuffer: 65536});
    const result = run('resolve');
    assert.equal(result.error, undefined); assert.equal(result.status, 0, result.stderr);
    assert.equal(fs.readFileSync(env.GITHUB_OUTPUT, 'utf8'), 'active=false\n');
    assert.equal(JSON.parse(result.stdout).publicationAuthority, false);
    assert.equal(fs.existsSync(env.RELEASE_DOWNLOAD_DIRECTORY), false);
    assert.equal(fs.readFileSync(path.join(directory, 'calls.jsonl'), 'utf8').trim().split('\n').length, 2);
    const rejected = run('all');
    assert.equal(rejected.status, 1); assert.match(rejected.stderr, /only initial resolution can skip/);
    assert.equal(fs.existsSync(env.RELEASE_DOWNLOAD_DIRECTORY), false);
  } finally {fs.rmSync(directory, {recursive: true, force: true});}
});
