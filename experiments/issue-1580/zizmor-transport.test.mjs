import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';
import {advisoryTransportFailure, image, runAudit} from '../../scripts/run-zizmor.mjs';

// Exact uncoloured diagnostic from run 44970848479, lines 3204–3212.
const transport = `fatal: no audit was performed
'known-vulnerable-actions' audit failed on file://.github/actions/automation-token/action.yml

Caused by:
    0: error in 'known-vulnerable-actions' audit
    1: request error while accessing GitHub API
    2: Cache error: Cache error: error sending request for url (https://api.github.com/advisories?ecosystem=actions&affects=actions%2Fcreate-github-app-token%40v3.4.0)
`;
const result = (status, stderr = '', stdout = '') => ({status, stderr, stdout});
async function replay(audits, persona = 'regular', pull = result(0)) {
  const calls = [], waits = []; let stdout = '', stderr = '';
  const status = await runAudit(persona, {
    execute(command, args, options) {
      calls.push({command, args, options});
      const value = args[0] === 'pull' ? pull : audits.shift();
      assert(value, 'unexpected additional audit'); return value;
    },
    pause: async milliseconds => { waits.push(milliseconds); },
    output: {stdout: {write(text) {stdout += text;}}, stderr: {write(text) {stderr += text;}}}
  });
  return {status, calls, waits, stdout, stderr};
}
test('both personas repeat the complete online audit only after the captured transport failure', async () => {
  for (const persona of ['regular', 'pedantic']) {
    const observed = await replay([result(1, transport), result(0, '', 'audited every input\n')], persona);
    assert.equal(observed.status, 0); assert.equal(observed.calls.length, 3);
    assert.deepEqual(observed.calls[1], observed.calls[2]);
    const args = observed.calls[1].args;
    assert(args.includes(image)); assert(args.includes(`--persona=${persona}`));
    assert(args.includes('--format=github')); assert(!args.includes('--no-online-audits'));
    assert(args.includes(`--min-confidence=${persona === 'regular' ? 'medium' : 'high'}`));
    assert.equal(args.includes('--min-severity=high'), persona === 'pedantic');
    assert.deepEqual(args.slice(-3), ['--', '.github/workflows', '.github/actions']);
    assert.deepEqual(observed.waits, [2000]); assert(observed.stderr.includes(transport));
    assert(observed.stdout.includes('audited every input'));
  }
});
test('three failed complete online audits remain a failure with every diagnostic preserved', async () => {
  const observed = await replay(Array.from({length: 3}, () => result(1, transport)));
  assert.equal(observed.status, 1); assert.equal(observed.calls.length, 4);
  assert.deepEqual(observed.waits, [2000, 5000]);
  assert.equal(observed.stderr.split(transport).length - 1, 3);
});
test('security findings, authentication, configuration, process and image failures are never retried', async () => {
  for (const failure of [result(14, 'high severity finding'), result(1, '401 Unauthorized'),
    result(1, 'invalid configuration'), result(1, transport, '::error::known vulnerable action\n'),
    result(1, transport.replace('error sending request for url', 'HTTP 403 response for url')),
    {...result(null, transport), signal: 'SIGTERM'}, {...result(0), error: new Error('spawn failed')}]) {
    const observed = await replay([failure]);
    assert.notEqual(observed.status, 0); assert.equal(observed.calls.length, 2);
    assert.deepEqual(observed.waits, []);
  }
  const observed = await replay([], 'regular', result(125, 'registry unavailable'));
  assert.equal(observed.status, 125); assert.equal(observed.calls.length, 1);
  await assert.rejects(() => replay([], 'auditor'), /Expected regular or pedantic/u);
  assert.equal(advisoryTransportFailure(result(0, transport)), false);
});
test('the actual CLI retains process exit status and never passes the token in Docker arguments', context => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-zizmor-'));
  context.after(() => rmSync(directory, {recursive: true, force: true}));
  const log = join(directory, 'docker.jsonl');
  writeFileSync(join(directory, 'docker'), `#!/usr/bin/env node
const fs = require('node:fs');
fs.appendFileSync(process.env.DOCKER_CALLS, JSON.stringify(process.argv.slice(2)) + '\\n');
process.exit(process.argv[2] === 'pull' ? 0 : 14);
`, {mode: 0o755});
  const observed = spawnSync(process.execPath, ['scripts/run-zizmor.mjs', 'pedantic'], {
    encoding: 'utf8', env: {...process.env, PATH: directory + ':' + process.env.PATH,
      DOCKER_CALLS: log, GH_TOKEN: 'finite-test-secret'}
  });
  assert.equal(observed.status, 14, observed.stderr);
  const calls = readFileSync(log, 'utf8').trim().split('\n').map(line => JSON.parse(line));
  assert.equal(calls.length, 2); assert(calls[1].includes('GH_TOKEN'));
  assert(!JSON.stringify(calls).includes('finite-test-secret'));
  assert(calls[1].includes(resolve('.') + ':/workspace:ro'));
});
test('workflow integration retains both security gates and declares the dynamic macOS shell', () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/workflows.yml', 'utf8'));
  for (const event of ['push', 'pull_request']) {
    for (const path of ['scripts/run-zizmor.mjs', 'experiments/issue-1580/zizmor-transport.test.mjs']) {
      assert(workflow.on[event].paths.includes(path), `${event} must audit changes to ${path}`);
    }
  }
  const job = workflow.jobs.zizmor;
  const audits = job.steps.filter(step => step.run?.includes('scripts/run-zizmor.mjs'));
  assert.deepEqual(audits.map(step => step.run), ['node scripts/run-zizmor.mjs regular', 'node scripts/run-zizmor.mjs pedantic']);
  for (const step of audits) assert.equal(step.env.GH_TOKEN, '${{ github.token }}');
  assert(job.steps[0].with['sparse-checkout'].includes('/scripts/run-zizmor.mjs'));
  const releases = YAML.parse(readFileSync('.github/workflows/verify-release-downloads.yml', 'utf8'));
  assert.equal(releases.jobs['macos-public-packages'].defaults.run.shell, 'bash');
});

test('both CodeQL producers expose identical complete steps without YAML aliases', () => {
  const source = readFileSync('.github/workflows/security.yml', 'utf8');
  const jobs = YAML.parse(source).jobs;
  assert.deepEqual(jobs.codeql.steps, jobs['codeql-readonly'].steps);
  assert.deepEqual(jobs.codeql.env, jobs['codeql-readonly'].env);
  assert(!/^[ ]+(?:steps|env): [*&]/mu.test(source));
});
