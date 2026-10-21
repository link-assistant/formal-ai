#!/usr/bin/env node
// CI-only optional installed-engine acceptance; no native build or fake producer.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import net from 'node:net';
import { createHash, randomBytes } from 'node:crypto';
import { spawn } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { createInstalledSourceCommandDescriptor, installedSourceScripts }
  from './create-installed-source-command-descriptor.mjs';
import { createParityEnvironment } from './check-server-parity.mjs';
import { readBoundedFileBytes } from '../js/provenance/bounded-file.mjs';
import { quoteShellWord } from '../js/agentic/portable/string_contracts.mjs';
import { fillSlots } from '../js/agentic/crate/seed_reports.mjs';
import { parseLino, childrenNamed, childValue } from '../js/server/lino.mjs';
import { verifyNativeSourceOperation }
  from '../rust/tests/fixtures/native-source-operations/oracle.mjs';

const maximumOutput = 1024 * 1024;
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const tools = ['read', 'bash'].map(name => ({
  type: 'function', function: { name, parameters: { type: 'object' } },
}));

/** Never interpret arbitrary planner shell text; compare with the installed protocol. */
export function installedOperationArguments(command, descriptor, kind, task, before, template) {
  const relative = kind === 'unary'
    ? installedSourceScripts.unaryConsolidationSha256
    : kind === 'parameter' ? installedSourceScripts.sourceCstTransformSha256 : null;
  if (!relative || !/^[a-f0-9]{64}$/u.test(before)) throw Error('UnknownInstalledOperation');
  const script = path.join(descriptor.resourceHome, relative);
  const expected = fillSlots(template, [
    ['node', quoteShellWord(descriptor.nodeExecutable)], ['script', quoteShellWord(script)],
    ['task', quoteShellWord(task)], ['digest', before],
  ]);
  assert.equal(command, expected, 'planner must select exactly the installed operation');
  return [script, '--request', task, '--expected', before];
}

function killProcessGroup(child) {
  try {
    if (process.platform !== 'win32' && child.pid) process.kill(-child.pid, 'SIGKILL');
    else child.kill('SIGKILL');
  } catch (error) {
    if (error.code !== 'ESRCH') child.kill('SIGKILL');
  }
}

/** Bound process time, output, descendant pipes and the post-kill settlement. */
export function executeBoundedSourceCommand(command, argumentsValue, options, timeout = 20000) {
  assert.ok(Number.isSafeInteger(timeout) && timeout > 0 && timeout <= 20000);
  return new Promise((resolve, reject) => {
    const child = spawn(command, argumentsValue, {
      ...options, detached: process.platform !== 'win32',
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let output = '', error = '', bytes = 0, settled = false, termination;
    let forced;
    const finish = (failure, result) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      clearTimeout(forced);
      if (failure) reject(failure); else resolve(result);
    };
    const terminate = reason => {
      if (termination) return;
      termination = reason;
      killProcessGroup(child);
      forced = setTimeout(() => {
        child.stdout.destroy();
        child.stderr.destroy();
        child.unref();
        finish(Error(reason));
      }, 250);
    };
    const timer = setTimeout(() => terminate('SourceCommandTimeout'), timeout);
    const capture = target => chunk => {
      bytes += chunk.length;
      if (bytes > maximumOutput) { terminate('SourceCommandOutputBounds'); return; }
      if (target === 'output') output += chunk; else error += chunk;
    };
    child.stdout.on('data', capture('output'));
    child.stderr.on('data', capture('error'));
    child.on('error', failure => finish(failure));
    child.on('close', (code, signal) => {
      if (termination) finish(Error(termination));
      else if (code !== 0 || signal) {
        finish(Error(JSON.stringify({ command, code, signal, output, error })));
      } else finish(null, { output, error, code, signal });
    });
  });
}

async function stopServer(child) {
  killProcessGroup(child);
  if (child.stdout.destroyed && child.stderr.destroyed) return;
  await new Promise(resolve => {
    const timer = setTimeout(() => {
      child.stdout.destroy();
      child.stderr.destroy();
      child.unref();
      resolve();
    }, 250);
    child.once('close', () => { clearTimeout(timer); resolve(); });
  });
}

async function freePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  const port = server.address().port;
  await new Promise(resolve => server.close(resolve));
  return port;
}

async function boundedJson(response) {
  assert.ok(response.ok, `HTTP ${response.status}`);
  const chunks = [];
  let size = 0;
  for await (const chunk of response.body) {
    size += chunk.length;
    if (size > maximumOutput) throw Error('NativeHttpOutputBounds');
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}

async function runCase(kind, directory, descriptor, base, token, template, evidence) {
  const owner = kind === 'unary' ? 'js/agentic/consumer.mjs' : 'source/increment.mjs';
  const task = kind === 'unary'
    ? `Consolidate stateless unary callbacks in ${owner} using canonical helpers. ` +
      'Preserve callback value behavior and comments.'
    : `Rename parameter value of function increment to input in ${owner}`;
  const fixture = kind === 'unary' ? 'unary-consumer.mjs' : 'parameter-consumer.mjs';
  const read = relative => readBoundedFileBytes(directory, relative, maximumOutput);
  const copy = (from, to) => {
    fs.mkdirSync(path.dirname(path.join(directory, to)), { recursive: true });
    fs.writeFileSync(path.join(directory, to),
      readBoundedFileBytes(descriptor.resourceHome, from, maximumOutput), { flag: 'wx' });
  };
  copy(`rust/tests/fixtures/native-source-operations/${fixture}`, owner);
  for (const helper of ['js/agentic/callback_values.mjs', 'js/agentic/crate/rust_str.mjs']) {
    copy(helper, helper);
  }
  // Membership is fixture-local; no repository branch, index or source is modified.
  await executeBoundedSourceCommand('git', ['init', '--quiet', directory], { cwd: directory });
  await executeBoundedSourceCommand('git', ['add', '--', owner], { cwd: directory });
  const before = digest(read(owner));
  const helperBefore = digest(read('js/agentic/callback_values.mjs'));
  const messages = [{ role: 'user', content: task }];
  const record = { kind, task, beforeSha256: before,
    beforeSource: read(owner).toString('utf8'), attempts: [], toolResults: [],
    authorshipCredit: 0 };
  evidence.cases.push(record);
  let ran = false, readback = false, oracle = null;
  for (let step = 0; step < 4; step++) {
    const response = await fetch(base + '/v1/chat/completions', {
      method: 'POST', headers: {
        'content-type': 'application/json', authorization: `Bearer ${token}`,
      }, body: JSON.stringify({ model: 'formal-ai', messages, tools }),
      signal: AbortSignal.timeout(5000),
    });
    const result = await boundedJson(response);
    const answer = result.choices?.[0]?.message;
    assert.ok(answer, 'native chat response required');
    record.attempts.push(answer);
    if (!answer.tool_calls?.length) {
      assert.ok(ran && oracle, 'final answer cannot substitute for physical effects');
      if (kind === 'unary') {
        assert.ok(readback, 'native unary route must request fresh physical readback');
        assert.match(answer.content, /"sourceReadbackVerified":true/u);
        assert.match(answer.content, /"independentBehaviorValidation":"pending"/u);
      }
      record.afterSha256 = digest(read(owner));
      record.afterSource = read(owner).toString('utf8');
      assert.notEqual(record.afterSha256, before);
      assert.equal(digest(read('js/agentic/callback_values.mjs')), helperBefore);
      record.oracle = oracle;
      record.nativeRequestedReadback = readback;
      record.wholeProductionTaskAccepted = false;
      return;
    }
    assert.equal(answer.tool_calls.length, 1, 'one bounded operation per step');
    messages.push(answer);
    const call = answer.tool_calls[0];
    const argumentsValue = JSON.parse(call.function.arguments);
    if (call.function.name === 'read') {
      const paths = ['path', 'filePath', 'file_path']
        .filter(key => key in argumentsValue).map(key => argumentsValue[key]);
      assert.ok(paths.length && paths.every(value => value === owner));
      const content = read(owner).toString('utf8');
      if (ran) { assert.ok(oracle); readback = true; }
      const result = { role: 'tool', tool_call_id: call.id, name: 'read', content,
        source_read: { path: owner, success: true, complete: true, format: 'raw' } };
      messages.push(result);
      record.toolResults.push(result);
    } else {
      assert.equal(call.function.name, 'bash');
      assert.equal(ran, false, 'only one source mutation is admitted');
      assert.ok(messages.some(message => message.source_read), 'real preimage read required');
      const argv = installedOperationArguments(argumentsValue.command,
        descriptor, kind, task, before, template);
      const operation = await executeBoundedSourceCommand(descriptor.nodeExecutable, argv, { cwd: directory });
      record.operation = operation;
      ran = true;
      oracle = await verifyNativeSourceOperation(kind, path.join(directory, owner));
      const result = { role: 'tool', tool_call_id: call.id, name: 'bash',
        content: `Output: ${operation.output.trimEnd()}\nExit Code: 0` };
      messages.push(result);
      record.toolResults.push(result);
    }
  }
  throw Error('NativeSourceOperationStepBounds');
}

export async function checkNativeSourceOperations(binary, resourceHome, evidencePath) {
  // This guard is an additional CI assertion, never permission for local native execution.
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'native smoke is CI-only');
  assert.ok(path.isAbsolute(binary));
  assert.equal(fs.realpathSync(binary), binary);
  const descriptor = createInstalledSourceCommandDescriptor(resourceHome);
  const seed = parseLino(readBoundedFileBytes(resourceHome,
    'data/meta/agentic-messages.lino', maximumOutput).toString('utf8'));
  const container = childrenNamed(seed, 'agentic_messages')[0] ?? seed;
  const entry = childrenNamed(container, 'message').find(node =>
    node.value === 'installed-source-operation-command');
  assert.ok(entry, 'installed command protocol must be canonical');
  const template = childValue(entry, 'text');
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-native-source-'));
  const token = randomBytes(24).toString('hex');
  const port = await freePort();
  const environment = createParityEnvironment(home, token);
  const server = spawn(binary, ['serve', '--agent-mode', '--source-command-host',
    JSON.stringify(descriptor), '--port', String(port)], {
    cwd: resourceHome, env: environment, detached: process.platform !== 'win32', stdio: ['ignore', 'pipe', 'pipe'],
  });
  let diagnostics = '', bytes = 0;
  const capture = chunk => {
    bytes += chunk.length;
    if (bytes > maximumOutput) killProcessGroup(server);
    else diagnostics += chunk;
  };
  server.stdout.on('data', capture);
  server.stderr.on('data', capture);
  let spawnFailure;
  server.on('error', error => { spawnFailure = error; });
  const evidence = { schema: 'native-installed-source-smoke/v1',
    scope: 'two-tiny-optional-installed-engine-controls', cases: [],
    productionWholeTaskAccepted: 0, authorshipCredit: 0, modelUsage: 'unknown',
    dependencyTrust: 'caller-trusted-resource-home-not-sealed-transitive-closure' };
  const deadline = setTimeout(() => killProcessGroup(server), 90000);
  try {
    const base = `http://127.0.0.1:${port}`;
    const readyBefore = Date.now() + 30000;
    let ready = false;
    for (let attempt = 0; attempt < 60 && Date.now() < readyBefore; attempt++) {
      if (spawnFailure) throw spawnFailure;
      assert.equal(server.exitCode, null, diagnostics);
      assert.ok(bytes <= maximumOutput);
      try {
        const response = await fetch(base + '/health', { signal: AbortSignal.timeout(1000) });
        if (response.ok) { await response.body?.cancel(); ready = true; break; }
        await response.body?.cancel();
      } catch { /* Readiness has a separate fixed deadline. */ }
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    assert.ok(ready, 'native server readiness deadline');
    for (const kind of ['unary', 'parameter']) {
      const directory = path.join(home, kind);
      fs.mkdirSync(directory);
      await runCase(kind, directory, descriptor, base, token, template, evidence);
    }
    evidence.status = 'passed';
  } catch (error) {
    evidence.status = 'failed';
    evidence.error = error.stack;
    throw error;
  } finally {
    clearTimeout(deadline);
    await stopServer(server);
    evidence.diagnostics = diagnostics;
    fs.writeFileSync(evidencePath, JSON.stringify(evidence, null, 2) + '\n');
    fs.rmSync(home, { recursive: true, force: true });
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  assert.equal(process.argv.length, 6);
  assert.equal(process.argv[2], '--rust-binary');
  assert.equal(process.argv[4], '--resource-home');
  await checkNativeSourceOperations(path.resolve(process.argv[3]),
    fs.realpathSync(process.argv[5]), 'native-source-operations-evidence.json');
}
