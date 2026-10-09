// Run the actual Desktop Release resolver against source-bound GitHub replies.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const script = fileURLToPath(new URL('../../../scripts/desktop-release-resolve.sh', import.meta.url));
const HEAD = 'a'.repeat(40);
const release = (tag_name) => ({ tag_name, draft: false, prerelease: false });
const child = { sha: 'b'.repeat(40), parents: [{ sha: HEAD }] };
const foreign = { sha: 'c'.repeat(40), parents: [{ sha: 'c'.repeat(40) }] };

function resolve(context, scenario, extra = {}) {
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-desktop-source-'));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const bin = join(root, 'bin');
  mkdirSync(bin);
  const mock = '#!/usr/bin/env node\nconst scenario=' + JSON.stringify(scenario) + ';\n'
    + String.raw`const args = process.argv.slice(2);
const write = (value) => process.stdout.write(String(value ?? '') + '\n');
if (args[0] === 'api') {
  const endpoint = args[1];
  if (endpoint.includes('/tags?')) write(scenario.exact ?? '');
  else if (endpoint.includes('/releases?')) {
    if (scenario.releaseFailure) { console.error('HTTP 401'); process.exit(1); }
    write(JSON.stringify(scenario.releases ?? []));
  } else if (endpoint.includes('/commits/')) {
    const commit = scenario.commits?.[endpoint.split('/').at(-1)] ?? { sha: '', parents: [] };
    if (args.includes('--jq')) write(args.at(-1) === '.parents[0].sha' ? commit.parents[0]?.sha : commit.sha);
    else write(JSON.stringify(commit));
  } else process.exit(2);
} else if (args[0] === 'release' && args[1] === 'view') {
  const tag = args[2]?.startsWith('--') ? null : args[2];
  if (!tag) write(scenario.latest ?? '');
  else if (args.includes('assets')) write((scenario.assets ?? []).join('\n'));
  else write(JSON.stringify({ tagName: tag }));
} else process.exit(2);
`;
  const executable = join(bin, 'gh');
  writeFileSync(executable, mock);
  chmodSync(executable, 0o755);
  const outputFile = join(root, 'outputs');
  writeFileSync(outputFile, '');
  const result = spawnSync('bash', [script], {
    encoding: 'utf8',
    env: { ...process.env, PATH: bin + ':' + process.env.PATH, REPO: 'owner/repository',
      EVENT: 'workflow_run', WORKFLOW_RUN_HEAD_SHA: HEAD, INPUT_TAG: '', RELEASE_TAG: '',
      GITHUB_OUTPUT: outputFile, ...extra },
  });
  return { ...result, outputs: Object.fromEntries(readFileSync(outputFile, 'utf8').trim().split('\n')
    .filter(Boolean).map((line) => line.split('='))) };
}

test('an unrelated latest release cannot stand in for the completed run', (context) => {
  const result = resolve(context, { latest: 'v9.9.9', releases: [release('v9.9.9')],
    commits: { 'v9.9.9': foreign } });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, '');
  assert.equal(result.outputs.should_build, 'false');
});

test('a delayed run selects its matching older published child', (context) => {
  const result = resolve(context, { latest: 'v9.9.9',
    releases: [release('v9.9.9'), release('v1.2.3')],
    commits: { 'v9.9.9': foreign, 'v1.2.3': child } });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});

test('the issue479 child-release path still heals missing assets', (context) => {
  const result = resolve(context, { latest: 'v1.2.3', commits: { 'v1.2.3': child } });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});

test('an exact-head release retains the defensive resolution tier', (context) => {
  const result = resolve(context, { exact: 'v1.2.3', latest: 'v9.9.9',
    commits: { 'v1.2.3': { sha: HEAD, parents: [] }, 'v9.9.9': foreign } });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v1.2.3');
  assert.equal(result.outputs.should_build, 'true');
});

test('an explicit manual rebuild may still select an older release', (context) => {
  const result = resolve(context, { latest: 'v9.9.9', commits: { 'v9.9.9': foreign } },
    { EVENT: 'workflow_dispatch', INPUT_TAG: 'v9.9.9' });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.outputs.tag, 'v9.9.9');
  assert.equal(result.outputs.should_build, 'true');
});

test('a GitHub transport failure cannot authorize an unrelated release', (context) => {
  const result = resolve(context, { latest: 'v9.9.9', commits: { 'v9.9.9': foreign },
    releaseFailure: true });
  assert.notEqual(result.status, 0);
  assert.equal(result.outputs.should_build, undefined);
});
