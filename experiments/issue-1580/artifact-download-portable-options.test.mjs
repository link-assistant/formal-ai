// Exercise Bash output production and Node's real NODE_OPTIONS parser.
// A resolver fixture supplies the Windows filesystem boundary on this host;
// hosted artifact transport must still prove actual Windows path loading.
import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, writeFileSync, rmSync, copyFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';

const actionPath = new URL('../../.github/actions/download-artifact/action.yml', import.meta.url);
const hook = fileURLToPath(new URL('../../.github/actions/download-artifact/retry-observations.cjs', import.meta.url));
const action = YAML.parse(readFileSync(actionPath, 'utf8'));
const attempts = action.runs.steps.filter(step => step.uses?.startsWith('actions/download-artifact@'));
function optionsFor(directory, scratch) {
  const prepare = action.runs.steps.find(step => step.id === 'observation_options');
  if (!prepare) return attempts[0].env.NODE_OPTIONS.replace('${{ github.action_path }}', directory);
  assert.equal(prepare.shell, 'bash');
  assert.equal(prepare.env.ACTION_DIRECTORY, '${{ github.action_path }}');
  const output = join(scratch, 'options-output');
  writeFileSync(output, '');
  const result = spawnSync('bash', ['--noprofile', '--norc', '-e', '-o', 'pipefail', '-c', prepare.run], {
    encoding: 'utf8', env: {...process.env, ACTION_DIRECTORY: directory, GITHUB_OUTPUT: output}, maxBuffer: 65536,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  const record = readFileSync(output, 'utf8').trimEnd();
  assert.ok(record.startsWith('node-options='));
  assert.equal(record.split('\n').length, 1);
  return record.slice('node-options='.length);
}
function execute(options, extraEnv = {}) {
  return spawnSync(process.execPath, ['-e', 'process.stdout.write("::error::Unable to download artifact(s): Artifact not found for name: fixture\\n");process.exitCode=1;'], {
    encoding: 'utf8', env: {...process.env, NODE_OPTIONS: options, INPUT_NAME: 'fixture', ARTIFACT_RETRY_ATTEMPT: '1', ...extraEnv}, maxBuffer: 65536,
  });
}
test('Windows action paths survive actual Node option parsing and reach the observation hook', () => {
  const scratch = mkdtempSync(join(tmpdir(), 'artifact-options-'));
  try {
    const directory = String.raw`D:\a\formal ai\formal-ai\./.github/actions/download-artifact`;
    const expected = directory.replaceAll('\\', '/') + '/retry-observations.cjs';
    const probe = join(scratch, 'resolver.cjs');
    writeFileSync(probe, `const Module=require('node:module');const original=Module._resolveFilename;Module._resolveFilename=function(request,...args){if(request===process.env.EXPECTED_WINDOWS_PRELOAD){process.stdout.write('PORTABLE_PRELOAD_OBSERVED\\n');return process.env.ACTUAL_PRELOAD;}return Reflect.apply(original,this,[request,...args]);};\n`);
    const result = execute('--require="' + probe + '" ' + optionsFor(directory, scratch), {
      EXPECTED_WINDOWS_PRELOAD: expected, ACTUAL_PRELOAD: hook,
    });
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.match(result.stdout, /^PORTABLE_PRELOAD_OBSERVED\n::notice::Artifact lookup failed; bounded retry remains:/u, result.stderr);
    assert.equal(result.stderr, '');
  } finally { rmSync(scratch, {recursive: true, force: true}); }
});
test('POSIX action paths containing spaces load the actual hook through NODE_OPTIONS', () => {
  const scratch = mkdtempSync(join(tmpdir(), 'artifact options '));
  try {
    copyFileSync(hook, join(scratch, 'retry-observations.cjs'));
    const result = execute(optionsFor(scratch, scratch));
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.match(result.stdout, /^::notice::Artifact lookup failed; bounded retry remains:/u, result.stderr);
    assert.equal(result.stderr, '');
  } finally { rmSync(scratch, {recursive: true, force: true}); }
});
test('every bounded attempt consumes the prepared portable options', () => {
  assert.equal(attempts.length, 3);
  const prepareIndex = action.runs.steps.findIndex(step => step.id === 'observation_options');
  assert.ok(prepareIndex >= 0);
  for (const [index, step] of attempts.entries()) {
    assert.ok(action.runs.steps.indexOf(step) > prepareIndex);
    assert.equal(step.env.NODE_OPTIONS, '${{ steps.observation_options.outputs.node-options }}');
    assert.equal(step.env.ARTIFACT_RETRY_ATTEMPT, String(index + 1));
    assert.equal(step.with['digest-mismatch'], 'error');
  }
});
