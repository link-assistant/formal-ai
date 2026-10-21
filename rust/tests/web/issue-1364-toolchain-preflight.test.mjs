import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {boundedCommandSync} from '../../../js/server/bounded-command.mjs';
import {programContractAnswer, programRecoveryCommand} from '../../../js/agentic/crate/coding_program_contract.mjs';
before(async () => installNodeHost(new WorkerHost()));
const request = language => `Write a program in ${language} that prints "Hello, World!".`;
function recipe(language) {
  const value = programContractAnswer(request(language))?.execution_recipe;
  assert(value, language);
  return value;
}
function verifier(value) {
  const file = value.supporting_files.find(file => file.path === 'tests/verify-output.sh');
  assert(file);
  return file;
}
function missingToolRun(language, executables = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-1364-preflight-'));
  try {
    const value = recipe(language), script = verifier(value);
    fs.mkdirSync(path.join(root, 'bin'));
    fs.mkdirSync(path.dirname(path.join(root, script.path)), {recursive: true});
    fs.writeFileSync(path.join(root, value.path), value.source);
    fs.writeFileSync(path.join(root, script.path), script.source);
    for (const [name, source] of Object.entries(executables)) fs.writeFileSync(path.join(root, 'bin', name), source, {mode: 0o700});
    const result = boundedCommandSync('/bin/sh', [script.path], {
      cwd: root, env: {...process.env, PATH: path.join(root, 'bin')}, timeout: 3000, maxBuffer: 65536,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.cleanup_complete, true);
    return {status: result.status, stderr: result.stderr.toString(), compilerCalled: fs.existsSync(path.join(root, 'compiler-called'))};
  } finally {
    fs.rmSync(root, {recursive: true, force: true});
  }
}
test('all external tools are probed before any compiler stage in the original three languages', () => {
  for (const language of ['Rust', 'Kotlin', 'Scala']) {
    const source = verifier(recipe(language)).source;
    const firstProbe = source.indexOf('command -v ');
    assert(firstProbe >= 0, language);
    assert(firstProbe < source.indexOf('verification_dir=$(mktemp'), language);
    assert(source.includes('__formal_ai_owned_process_failed_0__'), language);
  }
});
test('missing scalac preserves exit127 and the owned stage identity before any process setup', () => {
  const result = missingToolRun('Scala');
  assert.equal(result.status, 127);
  assert.match(result.stderr, /scalac: command not found/u);
  assert.match(result.stderr, /__formal_ai_owned_process_failed_0__/u);
  assert(!result.stderr.includes('mktemp:'));
});
test('a missing runtime is detected before an available fake compiler is invoked', () => {
  // This is a fake executable, never a local Kotlin/Rust compilation.
  const result = missingToolRun('Kotlin', {kotlinc: '#!/bin/sh\n: > compiler-called\nexit 71\n'});
  assert.equal(result.status, 127);
  assert.match(result.stderr, /java: command not found/u);
  assert.equal(result.compilerCalled, false);
});
test('the Rust executable produced by compilation is not treated as an installed prerequisite', () => {
  const source = verifier(recipe('Rust')).source;
  assert(source.includes("command -v 'rustc'"));
  assert(!source.includes("command -v './main'"));
});
test('preflight failures resolve only the exact owned verifier stage for authorized recovery', () => {
  const value = recipe('Scala');
  const command = value.commands[0];
  const output = 'scalac: command not found\n__formal_ai_owned_process_failed_0__\n';
  assert.match(programRecoveryCommand(request('Scala'), value, command, output), /^scalac\s/u);
  assert.equal(programRecoveryCommand(request('Scala'), value, 'sh unrelated.sh', output), null);
  assert.equal(programRecoveryCommand(request('Scala'), {...value, source: 'different bytes'}, command, output), null);
});
