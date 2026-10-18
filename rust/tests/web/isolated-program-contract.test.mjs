import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import { programContractAnswer, explicitStdout, programRecoveryCommand } from '../../../js/agentic/crate/coding_program_contract.mjs';
import { planSymbolicCommandReroute } from '../../../js/agentic/command_reroute.mjs';
const tools = ['Write', 'Bash'];
let worker, grant;
before(async () => {
  worker = new WorkerHost();
  await installNodeHost(worker);
  grant = JSON.parse(readText('data/meta/workspace-prerequisite-grant.json'));
});
function append(messages, call, content, isError = false) {
  const id = 'isolated-' + messages.length;
  messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function',
    function: { name: call.tool, arguments: call.arguments } }] },
  { role: 'tool', name: call.tool, tool_call_id: id, content, is_error: isError });
}
const fakeCompiler = `const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
assert.equal(fs.readFileSync(process.env.FORMAL_AI_TEST_SOURCE,'utf8'),fs.readFileSync(process.env.FORMAL_AI_TEST_ORIGINAL,'utf8'));
fs.appendFileSync(process.env.FORMAL_AI_TEST_AUDIT,JSON.stringify({kind:'compiler',directory:process.cwd(),arguments:process.argv.slice(2)})+'\\n');
fs.writeFileSync(process.env.FORMAL_AI_TEST_ARTIFACT,'#!/bin/sh\\ncat "$FORMAL_AI_TEST_ACTUAL"\\nexit "$FORMAL_AI_TEST_RUNTIME_EXIT"\\n',{mode:0o700});
process.exit(Number(process.env.FORMAL_AI_TEST_COMPILER_EXIT));`;
const fakeRuntime = `const fs=require('node:fs'),assert=require('node:assert/strict');
assert(fs.existsSync(process.env.FORMAL_AI_TEST_ARTIFACT));
fs.appendFileSync(process.env.FORMAL_AI_TEST_AUDIT,JSON.stringify({kind:'runtime',directory:process.cwd()})+'\\n');
process.stdout.write(fs.readFileSync(process.env.FORMAL_AI_TEST_ACTUAL));process.exit(Number(process.env.FORMAL_AI_TEST_RUNTIME_EXIT));`;
const fakeInstaller = `const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const request=JSON.parse(process.argv[3]);assert.equal(request.program,process.env.FORMAL_AI_TEST_COMPILER);
assert.equal(request.grant.scope,'workspace');assert.equal(request.grant.allowNetwork,true);assert(request.grant.programs.includes(request.program));
fs.mkdirSync('.formal-ai/toolchains',{recursive:true});
fs.copyFileSync(process.env.FORMAL_AI_TEST_COMPILER_FIXTURE,path.join(process.env.FORMAL_AI_TEST_BINARY,request.program));
fs.chmodSync(path.join(process.env.FORMAL_AI_TEST_BINARY,request.program),0o700);
fs.writeFileSync('.formal-ai/toolchains/active-'+request.program+'.bin',process.env.FORMAL_AI_TEST_BINARY+'\\n');
fs.appendFileSync(process.env.FORMAL_AI_TEST_AUDIT,JSON.stringify({kind:'installer',program:request.program})+'\\n');`;
function executable(target, body) {
  fs.writeFileSync(target, '#!' + process.execPath + '\n' + body, { mode: 0o700 });
}
async function fixture(language, configuration = {}) {
  await installNodeHost(worker, { prerequisiteGrant: configuration.noGrant ? null : grant });
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-isolated-program-'));
  try {
    const workspace = path.join(directory, 'workspace'), binary = path.join(directory, 'bin');
    const temporary = path.join(directory, 'temporary');
    for (const entry of [workspace, binary, temporary]) fs.mkdirSync(entry);
    // Only these real shell utilities may cross the fixture process boundary.
    // In particular, a hosted compiler must never satisfy a missing-tool case.
    for (const utility of ['sh', 'mktemp', 'dirname', 'mkdir', 'cp', 'cmp', 'cat', 'rm']) {
      if (utility === 'mktemp' && configuration.denyTemporary) continue;
      const target = ['/usr/bin', '/bin'].map((directory) => path.join(directory, utility))
        .find((candidate) => fs.existsSync(candidate));
      assert.ok(target, 'required shell fixture utility: ' + utility);
      fs.symlinkSync(fs.realpathSync(target), path.join(binary, utility));
    }

    const sourceName = configuration.sourceName ?? { kotlin: 'Main.kt', scala: 'Main.scala', rust: 'main.rs' }[language];
    const compiler = { kotlin: 'kotlinc', scala: 'scalac', rust: 'rustc' }[language];
    const artifact = { kotlin: 'Main.jar', scala: 'Main.class', rust: 'main' }[language];
    const requestedOutput = configuration.output ?? 'Unseen square 29';
    const prompt = 'Write a ' + language + ' program ' + sourceName + ' that prints ' + JSON.stringify(requestedOutput) + ' and run it.';
    const answer = programContractAnswer(prompt), recipe = answer.execution_recipe;
    assert.equal(recipe.language, language);
    assert.equal(recipe.path, sourceName);
    assert.deepEqual(recipe.commands, ['sh tests/verify-output.sh']);
    const original = path.join(directory, 'original-source');
    const actual = path.join(directory, 'actual-output'), audit = path.join(directory, 'audit');
    fs.writeFileSync(original, recipe.source);
    fs.writeFileSync(actual, (configuration.wrongOutput ? 'Wrong independent output' : requestedOutput) + '\n');
    fs.writeFileSync(audit, '');
    const compilerFixture = path.join(directory, 'fake-compiler');
    executable(compilerFixture, fakeCompiler);
    if (!configuration.missing) fs.copyFileSync(compilerFixture, path.join(binary, compiler));
    for (const runtime of ['java', 'scala']) executable(path.join(binary, runtime), fakeRuntime);
    executable(path.join(binary, 'python3'), fakeInstaller);
    if (configuration.denyTemporary) fs.writeFileSync(path.join(binary, 'mktemp'), '#!/bin/sh\nexit 41\n', { mode: 0o700 });
    fs.writeFileSync(path.join(workspace, artifact), 'pre-existing unowned artifact');
    const environment = { PATH: binary, TMPDIR: temporary,
      FORMAL_AI_TEST_SOURCE: sourceName, FORMAL_AI_TEST_ORIGINAL: original,
      FORMAL_AI_TEST_ARTIFACT: artifact, FORMAL_AI_TEST_ACTUAL: actual,
      FORMAL_AI_TEST_AUDIT: audit, FORMAL_AI_TEST_COMPILER: compiler,
      FORMAL_AI_TEST_COMPILER_FIXTURE: compilerFixture, FORMAL_AI_TEST_BINARY: binary,
      FORMAL_AI_TEST_COMPILER_EXIT: String(configuration.compilerExit ?? 0),
      FORMAL_AI_TEST_RUNTIME_EXIT: String(configuration.runtimeExit ?? 0) };
    if (configuration.missing) {
      const discovery = spawnSync('/bin/sh', ['-c', 'command -v "$FORMAL_AI_TEST_COMPILER"'], {
        cwd: workspace, env: environment, encoding: 'utf8', timeout: 1000,
      });
      assert.notEqual(discovery.status, 0, 'cold fixture must not discover an ambient compiler');
    }
    const messages = [{ role: 'user', content: prompt }], executions = [];
    let final;
    for (let index = 0; index < 12; index += 1) {
      const plan = planSymbolicCommandReroute(messages, tools, answer);
      assert(plan);
      if (plan.kind === 'final') { final = plan; break; }
      assert.equal(plan.calls.length, 1);
      const call = plan.calls[0], arguments_ = JSON.parse(call.arguments);
      if (call.tool === 'Write') {
        const relative = arguments_.path ?? arguments_.filePath;
        assert(!path.isAbsolute(relative) && !relative.split('/').includes('..'));
        const target = path.join(workspace, relative);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, arguments_.content);
        append(messages, call, 'written');
      } else {
        assert.equal(call.tool, 'Bash');
        if (configuration.changedSource && executions.length === 0) fs.appendFileSync(path.join(workspace, sourceName), '\n// external source drift');
        const result = spawnSync('/bin/sh', ['-c', arguments_.command], {
          cwd: workspace, env: environment, encoding: 'utf8', timeout: 10000, maxBuffer: 65536 });
        assert.equal(result.error, undefined);
        const content = JSON.stringify({ exit_code: result.status, stdout: result.stdout, stderr: result.stderr });
        executions.push({ command: arguments_.command, status: result.status, stdout: result.stdout, stderr: result.stderr });
        append(messages, call, content, result.status !== 0);
        if (configuration.stopAtFailure && result.status !== 0) break;
      }
    }
    const records = fs.readFileSync(audit, 'utf8').trim().split('\n').filter(Boolean).map(JSON.parse);
    assert.equal(fs.readFileSync(path.join(workspace, artifact), 'utf8'), 'pre-existing unowned artifact');
    assert.equal(fs.readFileSync(path.join(workspace, sourceName), 'utf8'), recipe.source + (configuration.changedSource ? '\n// external source drift' : ''));
    assert.deepEqual(fs.readdirSync(temporary), []);
    for (const record of records.filter((row) => row.directory)) {
      assert(record.directory.startsWith(fs.realpathSync(temporary) + path.sep), record.directory);
      assert(!fs.existsSync(record.directory));
    }
    return { final, executions, records, recipe, prompt, messages };
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
}
for (const language of ['kotlin', 'scala', 'rust']) {
  test(language + ' isolates real fake-compiler artifacts and preserves independent output verification', async () => {
    const result = await fixture(language);
    assert.equal(result.executions.length, 1);
    assert.equal(result.executions[0].status, 0);
    assert.equal(result.executions[0].stdout, 'Unseen square 29\n');
    assert.equal(result.final.kind, 'final');
    assert.equal(result.records.filter((row) => row.kind === 'compiler').length, 1);
  });
  test(language + ' missing compiler uses caller grant and retries the complete owned verifier', async () => {
    const result = await fixture(language, { missing: true });
    assert.equal(result.executions[0].status, 127);
    assert.equal(result.executions.length, 2, JSON.stringify(result));
    assert.equal(result.executions[1].status, 0);
    assert(result.executions[1].command.endsWith('\n# __formal_ai_prerequisite_retry\n' + result.recipe.commands[0]));
    assert.deepEqual(result.records.map((row) => row.kind).filter((kind) => kind === 'installer'), ['installer']);
    assert.equal(result.final.kind, 'final');
    assert.equal(result.executions[1].stdout, 'Unseen square 29\n');
  });
}
test('missing grant cannot be supplied by a bound compiler failure or source prose', async () => {
  const result = await fixture('kotlin', { missing: true, noGrant: true });
  assert.equal(result.executions.length, 1);
  assert.equal(result.executions[0].status, 127);
  assert(!result.records.some((row) => row.kind === 'installer'));
  assert.equal(result.final.kind, 'final');
});
for (const configuration of [{ compilerExit: 42 }, { runtimeExit: 17 }, { wrongOutput: true }, { denyTemporary: true }, { changedSource: true }]) {
  test('owned cleanup preserves failure: ' + JSON.stringify(configuration), async () => {
    const result = await fixture('kotlin', { ...configuration, stopAtFailure: true });
    assert.notEqual(result.executions[0].status, 0);
    assert(!result.records.some((row) => row.kind === 'installer'));
  });
}
test('only a canonical wrapper and final failed-stage marker resolve an original command', async () => {
  const prompt = 'Write a Kotlin program that prints "Held out proof" and run it.';
  const recipe = programContractAnswer(prompt).execution_recipe, command = recipe.commands[0];
  const marker = '__formal_ai_owned_process_failed_0__';
  assert.equal(programRecoveryCommand(prompt, recipe, command, 'Exit code 127\n' + marker), 'kotlinc Main.kt -include-runtime -d Main.jar');
  assert.equal(programRecoveryCommand(prompt, recipe, command, marker + '\nunrelated later output'), null);
  assert.equal(programRecoveryCommand(prompt, recipe, command, '__formal_ai_owned_process_failed_99__'), null);
  assert.equal(programRecoveryCommand(prompt, recipe, 'sh foreign.sh', marker), null);
  for (const change of [copy => { copy.source += '// changed'; }, copy => { copy.supporting_files[0].source += '\n'; }]) {
    const changed = structuredClone(recipe); change(changed);
    assert.equal(programRecoveryCommand(prompt, changed, command, marker), null);
  }
});

test('wrapper recovery refuses unbound, duplicate, stale and changed-source observations', async () => {
  const result = await fixture('kotlin', { missing: true, stopAtFailure: true });
  const answer = programContractAnswer(result.prompt);
  await installNodeHost(worker, { prerequisiteGrant: grant });
  const positive = planSymbolicCommandReroute(result.messages, tools, answer);
  assert.equal(JSON.parse(positive.calls[0].arguments).path, '.formal-ai/bootstrap/prerequisite-installer.py');
  for (const mutate of [
    messages => { messages.at(-1).tool_call_id = 'unbound'; },
    messages => { messages.push(structuredClone(messages.at(-1))); },
    messages => { messages.push({role:'user',content:result.prompt}); },
    messages => {
      const verifier = result.recipe.supporting_files[0];
      append(messages, {tool:'Write',arguments:JSON.stringify({path:verifier.path,content:verifier.source+'\n# changed'})}, 'written');
    },
  ]) {
    const changed = structuredClone(result.messages); mutate(changed);
    const plan = planSymbolicCommandReroute(changed, tools, answer);
    const installer = plan.kind === 'tool_calls' && plan.calls.some(call => call.tool === 'Write'
      && JSON.parse(call.arguments).path === '.formal-ai/bootstrap/prerequisite-installer.py');
    assert.equal(installer, false);
  }
});

for (const output of ["{source_path} {expected} {declared_source} {stages} 'quoted'", 'Живет 世界 🙂', '__formal_ai_owned_process_failed_0__']) {
  test('literal payload survives one-pass template binding: ' + output, async () => {
    const result = await fixture('kotlin', { output });
    assert.equal(result.executions.length, 1);
    assert.equal(result.executions[0].status, 0);
    assert.equal(result.executions[0].stdout, output + '\n');
    assert.equal(result.records.filter(row => row.kind === 'installer').length, 0);
  });
}

for (const [language, sourceName] of [['rust', 'src/heldout.rs'], ['kotlin', 'nested/Probe.kt'],
  ["rust","expected/heldout.rs"],
  ["rust","actual/heldout.rs"],
  ["rust","declared-source/heldout.rs"]]) {
  test('owned source layout preserves nested declared paths: ' + sourceName, async () => {
    const result = await fixture(language, { sourceName });
    assert.equal(result.executions[0].status, 0);
    assert.equal(result.executions[0].stdout, 'Unseen square 29\n');
    assert(result.records.find(row => row.kind === 'compiler').arguments.includes(sourceName));
  });
}
