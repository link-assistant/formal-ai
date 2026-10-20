import test from 'node:test';
import assert from 'node:assert/strict';
import {requireExactProgramOutput, requireHarnessDelivery, requireReadyPullRequest,
  requireProgramMember} from '../../../scripts/lib/hive-delivery-evidence.mjs';
const observation = text => ({exitCode: 0, signal: null,
  stdoutBase64: Buffer.from(text).toString('base64'), stderrBase64: ''});
test('independent original stdout oracle rejects double prints, prefixes and failed execution', () => {
  const expected = 'Hello, World!\n';
  requireExactProgramOutput(observation(expected), expected);
  for (const text of ['', 'Hello, World!', expected + expected, 'prefix ' + expected, expected + '\n']) {
    assert.throws(() => requireExactProgramOutput(observation(text), expected));
  }
  for (const changed of [{exitCode: 1}, {signal: 'SIGTERM'}, {stderrBase64: 'ZXJyb3I='},
    {stdoutBase64: observation(expected).stdoutBase64 + '!!'}]) {
    assert.throws(() => requireExactProgramOutput({...observation(expected), ...changed}, expected));
  }
});
test('actual solve and every original harness assertion are mandatory', () => {
  const evidence = {tool: 'codex', model: 'formal-ai', passed: true, solveExitCode: 0,
    harness: [{ok: true}], evaluation: {results: [{ok: true}]},
    verdict: {harness: true, model: true}, pullRequestUrl: 'https://github.com/owner/repo/pull/7'};
  assert.deepEqual(requireHarnessDelivery(evidence, 'codex'), {repository: 'owner/repo', number: 7});
  for (const patch of [{solveExitCode: null}, {passed: false}, {harness: []},
    {evaluation: {results: [{ok: false}]}}, {verdict: {harness: true, model: null}},
    {pullRequestUrl: 'https://evil.invalid/owner/repo/pull/7'}]) {
    assert.throws(() => requireHarnessDelivery({...evidence, ...patch}, 'codex'));
  }
});
test('a real ready isolated PR and one safe original-language source are required', () => {
  const value = {number: 7, base: {repo: {full_name: 'owner/repo'}, ref: 'e2e/hello-world/1/codex-formal-ai'},
    head: {sha: 'a'.repeat(40), ref: 'issue-1-aabbcc'}, state: 'open', draft: false};
  assert.equal(requireReadyPullRequest(value, 'owner/repo', 7), 'a'.repeat(40));
  for (const patch of [{draft: true}, {state: 'closed'}, {head: {sha: 'a'.repeat(40), ref: 'main'}}]) {
    assert.throws(() => requireReadyPullRequest({...value, ...patch}, 'owner/repo', 7));
  }
  assert.equal(requireProgramMember([{filename: 'src/main.rs'}], 'Rust'), 'src/main.rs');
  for (const files of [[], [{filename: '../main.rs'}], [{filename: 'a.rs'}, {filename: 'b.rs'}]]) {
    assert.throws(() => requireProgramMember(files, 'Rust'));
  }
});

import {readFileSync} from 'node:fs';
import YAML from 'yaml';
const {parse} = YAML;
test('genuine image, solve and independent verification phases use same-run handoffs within the existing cap', () => {
  const root = new URL('../../../', import.meta.url);
  const {jobs} = parse(readFileSync(new URL('.github/workflows/hive-original-delivery.yml', root), 'utf8'));
  assert.equal(jobs.image.needs,'build');
  assert.deepEqual(jobs.delivery.needs,['build','image']);
  assert.deepEqual(jobs.verification.needs,['build','image','delivery']);
  assert.equal(jobs.verification.if,"${{ !cancelled() && needs.image.result == 'success' }}",'failed original solves still reach the original verifier');
  for(const job of Object.values(jobs)){
    assert.ok(job['timeout-minutes']<=30);
    for(const step of job.steps??[])if(step['timeout-minutes']!==undefined){
      assert.ok(step['timeout-minutes']*100<=job['timeout-minutes']*70,'step retains original job headroom');
    }
  }
  for(const id of ['image','delivery','verification']) {
    const job=jobs[id];
    assert.equal(job.env?.E2E_LOG_DIR,undefined);
    assert.equal(job.env?.HIVE_IMAGE_BUNDLE_DIR,undefined,'runner context belongs in a step');
    const initializers=['echo "E2E_LOG_DIR=$RUNNER_TEMP/hive-original-evidence" >> "$GITHUB_ENV"'];
    if(id!=='verification')initializers.push('echo "HIVE_IMAGE_BUNDLE_DIR=$RUNNER_TEMP/hive-image-bundle" >> "$GITHUB_ENV"');
    assert.deepEqual(job.steps[0].run.split('\n').filter(Boolean),initializers);
    assert.ok(job.steps.some(step=>step.with?.name==='hive-same-source-formal-ai-binary'));
    for(const step of job.steps)assert.equal(step.env?.E2E_LOG_DIR,undefined);
  }
  const hasRun=(job,text)=>jobs[job].steps.some(step=>(step.run||'').includes(text));
  assert.ok(hasRun('image','node scripts/build-hive-original-delivery-image.mjs'));
  assert.ok(!hasRun('image','e2e-hello-world.mjs'));
  const imageUpload=jobs.image.steps.find(step=>step.with?.name==='hive-same-source-task-image');
  const imageDownload=jobs.delivery.steps.find(step=>step.with?.name==='hive-same-source-task-image');
  assert.ok(imageUpload.uses.startsWith('actions/upload-artifact@'));
  assert.ok(imageDownload.uses.startsWith('actions/download-artifact@'));
  assert.equal(imageUpload.with.path,imageDownload.with.path);
  assert.equal(imageUpload.with['compression-level'],0,'the streamed gzip archive is not recompressed');
  assert.equal(jobs.image.permissions,undefined,'image construction has only workflow read permissions');
  assert.ok(!JSON.stringify(jobs.image).includes('secrets.'));
  assert.ok(!JSON.stringify(jobs.verification).includes('secrets.'));
  assert.ok(hasRun('delivery','node scripts/build-hive-original-delivery-image.mjs load'));
  assert.ok(hasRun('delivery','--seal-evidence'));
  assert.ok(!hasRun('delivery','node scripts/check-hive-original-delivery.mjs'));
  assert.ok(hasRun('verification','--verify-evidence'));
  assert.ok(hasRun('verification','node scripts/check-hive-original-delivery.mjs'));
  assert.ok(!hasRun('verification','e2e-hello-world.mjs'));
  const stage=jobs.delivery.steps.find(step=>step.uses?.startsWith('actions/upload-artifact@'));
  const staged=jobs.verification.steps.find(step=>step.with?.name==='hive-original-${{ matrix.tool }}-staged-evidence');
  const final=jobs.verification.steps.find(step=>step.uses?.startsWith('actions/upload-artifact@'));
  assert.equal(stage.with.name,staged.with.name);
  assert.equal(stage.with.path,staged.with.path);
  assert.equal(final.with.name,'hive-original-${{ matrix.tool }}-evidence');
  for(const upload of [stage,final]) {
    assert.equal(upload.if,'${{ !cancelled() }}');
    assert.equal(upload.with['if-no-files-found'],'error');
    assert.equal(upload.with['include-hidden-files'],false);
  }
  assert.deepEqual(jobs.delivery.strategy.matrix.tool,['claude','agent','codex']);
  assert.deepEqual(jobs.verification.strategy.matrix.tool,jobs.delivery.strategy.matrix.tool);
  const solve=jobs.delivery.steps.find(step=>step.name==='Run actual Hive solve and original assertions');
  assert.equal(solve['timeout-minutes'],21);assert.equal(solve.env.E2E_KEEP,'true');
  assert.ok(!jobs.delivery.steps.some(step=>step.uses==='dtolnay/rust-toolchain@stable'||step.uses?.startsWith('coursier/')),'compilers belong only to the independent verification host');
  assert.ok(jobs.verification.steps.findIndex(step=>(step.run||'').includes('--verify-evidence')) < jobs.verification.steps.findIndex(step=>(step.run||'').includes('node scripts/check-hive-original-delivery.mjs')));
});

import {spawnSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, writeFileSync, rmSync} from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const hiveHarnessCommand = 'node scripts/e2e-hello-world.mjs --tool "$E2E_TOOL" --model formal-ai --isolation branch --keep';
const observedLogTokens = new Set();
function actualHiveHarnessStep() {
  const root = new URL('../../../', import.meta.url);
  const workflow = parse(readFileSync(new URL('.github/workflows/hive-original-delivery.yml', root), 'utf8'));
  return workflow.jobs.delivery.steps.find(step => step.name === 'Run actual Hive solve and original assertions');
}

test('only the actual Hive child log stream is framed and the original command remains exact', () => {
  const step = actualHiveHarnessStep();
  const lines = step.run.trimEnd().split('\n');
  assert.equal(lines.at(-1), hiveHarnessCommand);
  assert.equal(lines.filter(line => line === hiveHarnessCommand).length, 1);
  assert.match(step.run, /randomBytes\(32\)/);
  assert.match(step.run, /trap .* EXIT/);
  assert.equal(step.working_directory, undefined);
  assert.equal(step['working-directory'], '.hive-mind');
  assert.equal(step['continue-on-error'], undefined);
  assert.equal(step.env.FORMAL_AI_INSTALL_GRANT, 'workspace');
  const root = new URL('../../../', import.meta.url);
  const workflow = parse(readFileSync(new URL('.github/workflows/hive-original-delivery.yml', root), 'utf8'));
  assert.equal(workflow.jobs.delivery.steps.filter(value => (value.run || '').includes('::stop-commands::')).length, 1);
});

for (const [status, newline] of [[0, true], [0, false], [7, false], [127, true]]) {
  test('Hive log framing preserves child exit ' + status + ' and trailing-newline=' + newline, () => {
    const temporary = mkdtempSync(path.join(os.tmpdir(), 'hive-log-framing-'));
    try {
      mkdirSync(path.join(temporary, 'scripts'));
      const argumentsPath = path.join(temporary, 'arguments.json');
      const payload = '##[add-matcher] /run/act/actions/setup-java/matchers/java.json\n' +
        JSON.stringify({nested: '::error::not an outer runner error', output: '完整日志'}) +
        (newline ? '\n' : '');
      const stderr = '::warning::nested stderr payload\n';
      writeFileSync(path.join(temporary, 'scripts/e2e-hello-world.mjs'),
        'import fs from "node:fs";\n' +
        'fs.writeFileSync(process.env.HIVE_FIXTURE_ARGUMENTS, JSON.stringify({arguments: process.argv.slice(2), tokenExported: process.env.hive_log_token !== undefined}));\n' +
        'process.stdout.write(' + JSON.stringify(payload) + ');\n' +
        'process.stderr.write(' + JSON.stringify(stderr) + ');\n' +
        'process.exitCode = ' + status + ';\n');
      const result = spawnSync('/bin/bash', ['--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', actualHiveHarnessStep().run], {
        cwd: temporary, encoding: 'utf8', timeout: 10000,
        env: {...process.env, E2E_TOOL: 'fixture-client', HIVE_FIXTURE_ARGUMENTS: argumentsPath}
      });
      assert.equal(result.error, undefined);
      assert.equal(result.status, status, result.stderr);
      const marker = result.stdout.match(/^::stop-commands::([a-f0-9]{64})\n/);
      assert.ok(marker, result.stdout);
      const token = marker[1];
      assert.equal(observedLogTokens.has(token), false, 'each invocation has fresh cryptographic framing');
      observedLogTokens.add(token);
      const resume = '\n::' + token + '::\n';
      assert.equal(result.stdout, marker[0] + payload + resume, 'all original stdout bytes are preserved between frames');
      assert.equal(result.stderr, stderr, 'all original stderr bytes are preserved');
      assert.equal(result.stdout.split('::' + token + '::').length, 2, 'exactly one restoration, including failure');
      assert.deepEqual(JSON.parse(readFileSync(argumentsPath, 'utf8')), {
        arguments: ['--tool', 'fixture-client', '--model', 'formal-ai', '--isolation', 'branch', '--keep'],
        tokenExported: false
      });
    } finally { rmSync(temporary, {recursive: true, force: true}); }
  });
}

test('all fake harness invocations used distinct framing tokens', () => {
  assert.equal(observedLogTokens.size, 4);
});

function isolatedRustResolutionStep() {
  const root = new URL('../../../', import.meta.url);
  const steps = parse(readFileSync(new URL('.github/workflows/hive-original-delivery.yml', root), 'utf8')).jobs.verification.steps;
  return {steps, resolve: steps.find(step => step.name === 'Resolve Rust compiler for isolated output verification')};
}

test('only Codex installs Rust and resolves the actual compiler before the unchanged independent oracle', () => {
  const {steps, resolve} = isolatedRustResolutionStep();
  const setup = steps.findIndex(step => step.uses === 'dtolnay/rust-toolchain@stable');
  const resolved = steps.indexOf(resolve);
  const independent = steps.findIndex(step => step.name === 'Independently compile delivered source and require exactly one output line');
  assert.ok(setup >= 0 && setup < resolved && resolved < independent);
  assert.equal(steps[setup].if, "matrix.tool == 'codex'");
  assert.equal(resolve.if, "matrix.tool == 'codex'");
  assert.equal(steps[independent].run, 'node scripts/check-hive-original-delivery.mjs');
  assert.match(resolve.run, /rustup which rustc/);
  assert.match(resolve.run, /test -x "\$hive_rust_compiler"/);
  assert.equal(resolve['continue-on-error'], undefined);
});

function fakeRustEnvironment({rustupStatus = 0, executable = true} = {}) {
  const temporary = mkdtempSync(path.join(os.tmpdir(), 'hive-rust-path-'));
  const shims = path.join(temporary, 'shims');
  const toolchain = path.join(temporary, 'actual toolchain', 'bin');
  const freshHome = path.join(temporary, 'fresh-oracle-home');
  for (const directory of [shims, toolchain, freshHome]) mkdirSync(directory, {recursive: true});
  const compiler = path.join(toolchain, 'rustc');
  writeFileSync(compiler, '#!/bin/sh\nprintf "fixture actual compiler\\n"\n', {mode: executable ? 0o700 : 0o600});
  writeFileSync(path.join(shims, 'rustc'), '#!/bin/sh\nprintf "fixture shim has no default toolchain\\n" >&2\nexit 93\n', {mode: 0o700});
  writeFileSync(path.join(shims, 'rustup'), '#!/bin/sh\n' +
    '[ "$1" = which ] && [ "$2" = rustc ] || exit 94\n' +
    'printf "%s\\n" "$FIXTURE_COMPILER"\nexit ' + rustupStatus + '\n', {mode: 0o700});
  const githubPath = path.join(temporary, 'github-path');
  writeFileSync(githubPath, '');
  const environment = {...process.env, PATH: shims + path.delimiter + process.env.PATH,
    FIXTURE_COMPILER: compiler, GITHUB_PATH: githubPath};
  const result = spawnSync('/bin/bash', ['--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', isolatedRustResolutionStep().resolve.run], {
    cwd: temporary, env: environment, encoding: 'utf8', timeout: 10000
  });
  return {temporary, shims, toolchain, freshHome, githubPath, environment, result,
    cleanup: () => rmSync(temporary, {recursive: true, force: true})};
}

test('the oracle fresh HOME resolves the real fake compiler rather than a rustup shim', () => {
  const context = fakeRustEnvironment();
  try {
    assert.equal(context.result.error, undefined);
    assert.equal(context.result.status, 0, context.result.stderr);
    assert.equal(readFileSync(context.githubPath, 'utf8'), context.toolchain + '\n');
    const without = spawnSync('/bin/sh', ['-c', 'rustc'], {
      encoding: 'utf8', timeout: 10000, env: {...context.environment, HOME: context.freshHome}
    });
    assert.equal(without.status, 93, 'the former fresh-HOME/shim boundary fails');
    const isolated = spawnSync('/bin/sh', ['-c', 'rustc'], {
      encoding: 'utf8', timeout: 10000,
      env: {PATH: context.toolchain + path.delimiter + context.environment.PATH, HOME: context.freshHome}
    });
    assert.equal(isolated.error, undefined);
    assert.equal(isolated.status, 0, isolated.stderr);
    assert.equal(isolated.stdout, 'fixture actual compiler\n');
  } finally { context.cleanup(); }
});

for (const options of [{rustupStatus: 17}, {executable: false}]) {
  test('failed Rust resolution cannot publish a toolchain path: ' + JSON.stringify(options), () => {
    const context = fakeRustEnvironment(options);
    try {
      assert.equal(context.result.error, undefined);
      assert.notEqual(context.result.status, 0);
      if (options.rustupStatus) assert.equal(context.result.status, options.rustupStatus);
      assert.equal(readFileSync(context.githubPath, 'utf8'), '');
    } finally { context.cleanup(); }
  });
}
