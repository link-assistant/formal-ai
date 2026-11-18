import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync, mkdtempSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import YAML from 'yaml';
import {requireTaskCredentialConfiguration, taskCredentialScope,
  probeTaskPushAccess, requireTaskCredentialAccess} from '../../../scripts/lib/hive-delivery-credentials.mjs';
import {boundedCommand} from '../../../js/server/bounded-command.mjs';
import {createHash} from 'node:crypto';
import {preflightTaskCredentials} from '../../../scripts/hive-delivery-attempt.mjs';
import {admissionPolicy} from '../../../scripts/lib/hive-delivery-admission.mjs';

test('missing or incomplete optional credentials select the default layer', () => {
  for (const environment of [{}, {GH_TOKEN: 'default-repository-token'}, {AUTOMATION_APP_ID: '123'},
    {AUTOMATION_APP_PRIVATE_KEY: 'private-placeholder'}, {AUTOMATION_TOKEN: ' \n'}]) {
    assert.deepEqual(requireTaskCredentialConfiguration(environment),
      {configured: false, layer: 'default', deliveryVerified: false});
  }
});
test('configured token or complete App pair passes presence check without claiming target access or echoing secrets', () => {
  for (const [environment, layer] of [[{AUTOMATION_TOKEN: 'private-placeholder'}, 'token'],
    [{AUTOMATION_APP_ID: '123', AUTOMATION_APP_PRIVATE_KEY: 'private-placeholder'}, 'app']]) {
    assert.deepEqual(requireTaskCredentialConfiguration(environment), {configured: true, layer, deliveryVerified: false});
    assert(!JSON.stringify(requireTaskCredentialConfiguration(environment)).includes('private-placeholder'));
  }
});

test('the default token can authorize only the current repository and staged workflow execution', () => {
  const canonical = {owner: 'external', name: 'task', repository: 'external/task'};
  const target = taskCredentialScope(canonical, {layer: 'default', repository: 'host/project'});
  const repository = {id: 1, full_name: 'host/project', name: 'project', owner: {login: 'host'},
    permissions: {push: true}, private: false};
  const proof = requireTaskCredentialAccess(target, {repository, layer: 'default', currentRepository: 'host/project'});
  assert.equal(proof.workflowWrite, 'staged-act');
  assert.equal(proof.deliveryVerified, false);
  assert.throws(() => requireTaskCredentialAccess(target,
    {repository, layer: 'default', currentRepository: 'other/project'}));
  assert.throws(() => requireTaskCredentialAccess(target,
    {repository: {...repository, permissions: {push: false}}, layer: 'default', currentRepository: 'host/project'}));
  assert.deepEqual(taskCredentialScope(canonical, {layer: 'token', repository: 'host/project'}), canonical);
});

test('the actual resolver selects a minted App token or the usable token/default fallback', () => {
  const action = YAML.parse(readFileSync(new URL('../../../.github/actions/automation-token/action.yml', import.meta.url), 'utf8'));
  assert.equal(action.runs.steps.find(step => step.id === 'app-token')['continue-on-error'], true);
  const script = action.runs.steps.find(step => step.id === 'selected').run;
  for (const [app, fallback, layer, expected] of [['', 'job-placeholder', 'default', 'default'],
    ['', 'token-placeholder', 'token', 'token'], ['app-placeholder', 'job-placeholder', 'default', 'app']]) {
    const directory = mkdtempSync(path.join(tmpdir(), 'hive-credential-selection-'));
    try {
      const output = path.join(directory, 'output'), summary = path.join(directory, 'summary');
      writeFileSync(output, ''); writeFileSync(summary, '');
      execFileSync('bash', ['-c', script], {env: {PATH: process.env.PATH, APP_TOKEN: app,
        FALLBACK_TOKEN: fallback, FALLBACK_LAYER: layer, GITHUB_OUTPUT: output, GITHUB_STEP_SUMMARY: summary}});
      const selected = readFileSync(output, 'utf8');
      assert(selected.includes('layer=' + expected + '\n'));
      assert(selected.includes('token=' + (app || fallback) + '\n'));
      assert(readFileSync(summary, 'utf8').includes('`' + expected + '`'));
    } finally { rmSync(directory, {recursive: true, force: true}); }
  }
});

test('an actual receive-pack probe resolves the hosted default-token false role flag', async () => {
  const target = {owner: 'host', name: 'project', repository: 'host/project'};
  const repository = {id: 1, full_name: target.repository, name: target.name,
    owner: {login: target.owner}, permissions: {push: false}};
  const environment = {GITHUB_RUN_ID: '123', E2E_TOOL: 'agent', GH_TOKEN: 'private-placeholder'};
  let calls = 0;
  const pushProbe = await probeTaskPushAccess(target, {environment, transport: async (command, args, options) => {
    calls++;
    assert.equal(command, 'git');
    assert(args.includes('--dry-run') && args.includes('--no-verify') && args.includes('--no-follow-tags'));
    assert(args.includes('credential.helper=!gh auth git-credential'));
    assert.deepEqual(args.slice(-2), ['https://github.com/host/project.git',
      'HEAD:refs/heads/formal-ai-preflight/123-agent']);
    assert(!JSON.stringify(args).includes(environment.GH_TOKEN));
    assert.equal(options.env.GH_TOKEN, environment.GH_TOKEN);
    assert.equal(options.env.GIT_TERMINAL_PROMPT, '0');
    assert.equal(options.timeout, 20000);
    return {status: 0, signal: null, cleanup_complete: true, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0)};
  }});
  assert.equal(calls, 1);
  for (const permissions of [{push: false}, undefined]) {
    const proof = requireTaskCredentialAccess(target, {repository: {...repository, permissions},
      layer: 'default', currentRepository: target.repository, pushProbe});
    assert.equal(proof.pushAuthority, 'read-only-git-receive-pack');
    assert.equal(proof.workflowWrite, 'staged-act');
    assert.equal(proof.deliveryVerified, false);
  }
  for (const invalid of [null, {...pushProbe, success: false}, {...pushProbe, repository: 'other/project'},
    {...pushProbe, kind: 'upload-pack'}]) {
    assert.throws(() => requireTaskCredentialAccess(target, {repository, layer: 'default',
      currentRepository: target.repository, pushProbe: invalid}));
  }
  assert.throws(() => requireTaskCredentialAccess(target, {repository, layer: 'default',
    currentRepository: 'other/project', pushProbe}));
});

test('denied, timed-out and incompletely cleaned push probes fail once and redact the token', async () => {
  const target = {repository: 'host/project'};
  const environment = {GITHUB_RUN_ID: '123', E2E_TOOL: 'codex', GH_TOKEN: 'private-placeholder'};
  const successful = {status: 0, signal: null, cleanup_complete: true, stderr: Buffer.from('private-placeholder')};
  for (const failure of [{status: 1}, {signal: 'SIGTERM'}, {timed_out: true},
    {cleanup_complete: false}, {error: new Error('transport failure')}]) {
    let calls = 0;
    await assert.rejects(probeTaskPushAccess(target, {environment, transport: async () => {
      calls++; return {...successful, ...failure};
    }}), error => /receive-pack preflight failed/u.test(error.message) && !error.message.includes(environment.GH_TOKEN));
    assert.equal(calls, 1, 'uncertain authentication is never retried');
  }
});

test('the real Git dry-run creates no remote ref or objects', async () => {
  const directory = mkdtempSync(path.join(tmpdir(), 'hive-readonly-push-'));
  const source = path.join(directory, 'source'), remote = path.join(directory, 'remote.git');
  const gitEnvironment = {...process.env, GIT_CONFIG_GLOBAL: '/dev/null', GIT_CONFIG_NOSYSTEM: '1'};
  const git = args => execFileSync('git', args, {env: gitEnvironment, timeout: 5000, encoding: 'utf8'});
  try {
    git(['init', '--initial-branch=main', source]); git(['init', '--bare', '--initial-branch=main', remote]);
    writeFileSync(path.join(source, 'README.md'), 'Read-only authentication probe.\n');
    git(['-C', source, 'add', 'README.md']);
    git(['-C', source, '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
      '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', 'commit', '-m', 'fixture']);
    const proof = await probeTaskPushAccess({repository: 'host/project'}, {
      environment: {...gitEnvironment, GITHUB_RUN_ID: '123', E2E_TOOL: 'claude', GH_TOKEN: 'fixture-placeholder'},
      transport: async (command, args, options) => {
        assert.equal(args.at(-2), 'https://github.com/host/project.git');
        const local = [...args]; local[local.length - 2] = remote;
        return boundedCommand(command, local, {...options, cwd: source});
      }
    });
    assert.equal(proof.success, true);
    assert.equal(git(['--git-dir=' + remote, 'for-each-ref']), '');
    assert.match(git(['--git-dir=' + remote, 'count-objects', '-v']), /^count: 0\n/mu);
  } finally {rmSync(directory, {recursive: true, force: true});}
});

test('preclaim failures do not try to publish nonexistent solve evidence', () => {
  const workflow = YAML.parse(readFileSync(new URL('../../../.github/workflows/hive-original-delivery.yml', import.meta.url), 'utf8'));
  for (const name of ['Seal staged source/run evidence without granting success credit',
    'Stage actual original solve evidence for independent verification']) {
    assert.match(workflow.jobs.delivery.steps.find(step => step.name === name).if,
      /steps\.claim\.outputs\.execute == 'true'/u);
  }
});

test('the actual hosted credential entrypoint probes false role metadata before issuing outputs', async () => {
  const directory = mkdtempSync(path.join(tmpdir(), 'hive-hosted-preflight-'));
  try {
    const manifest = new URL('../../../data/benchmarks/issue-1364-client-replay-inputs.json', import.meta.url);
    const inputHash = createHash('sha256').update(readFileSync(manifest)).digest('hex');
    const selection = path.join(directory, 'selection.json'), source = 'a'.repeat(40);
    writeFileSync(selection, JSON.stringify({schema: admissionPolicy, admitted: true, inputHash,
      producer: {repository: 'host/project', run: '123', attempt: '1'}, selected: {source}}));
    const environment = {GITHUB_ACTIONS: 'true', GITHUB_REPOSITORY: 'host/project',
      GITHUB_WORKFLOW_REF: 'host/project/.github/workflows/hive-original-delivery.yml@refs/heads/main',
      GITHUB_RUN_ID: '123', GITHUB_RUN_ATTEMPT: '1', HIVE_ADMISSION_SOURCE: source,
      HIVE_ADMISSION_SELECTION: selection, HIVE_CANONICAL_INPUT_FILE: manifest.pathname,
      HIVE_CREDENTIAL_LAYER: 'default', E2E_TOOL: 'agent', GH_TOKEN: 'fixture-placeholder'};
    for (const status of [0, 128]) {
      const calls = [];
      const execute = () => preflightTaskCredentials(environment, {transport: async (file, args) => {
        calls.push({file, args});
        if (file === 'gh') {
          assert.equal(args.at(-1), 'repos/host/project');
          const body = {id: 1, full_name: 'host/project', name: 'project', owner: {login: 'host'},
            permissions: {push: false}};
          return {status: 0, signal: null, cleanup_complete: true,
            stdout: Buffer.from('HTTP/2.0 200 OK\r\n\r\n' + JSON.stringify(body)), stderr: Buffer.alloc(0)};
        }
        assert.equal(file, 'git'); assert(args.includes('--dry-run'));
        return {status, signal: null, cleanup_complete: true, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0)};
      }});
      if (status === 0) assert.equal((await execute()).pushAuthority, 'read-only-git-receive-pack');
      else await assert.rejects(execute, /receive-pack preflight failed/u);
      assert.equal(calls.filter(call => call.file === 'gh').length, 1);
      assert.equal(calls.filter(call => call.file === 'git').length, 1);
    }
  } finally {rmSync(directory, {recursive: true, force: true});}
});
