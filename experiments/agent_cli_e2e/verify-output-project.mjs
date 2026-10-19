// Ordinary E2E completion is proved by bound effects and independent execution, not a minimum turn count.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { decodeDialogRecord } from '../../js/server/dialog-codec.mjs';
const digest = value => createHash('sha256').update(value).digest('hex');
const text = value => typeof value === 'string' ? value : Array.isArray(value) ? value.map(block => typeof block?.text === 'string' ? block.text : '').join('') : '';
export function completionResponse(body) {
  let events;
  const streaming = !body.trimStart().startsWith('{');
  if (body.trimStart().startsWith('{')) events = [JSON.parse(body)];else {
    assert.ok(body.endsWith('\n'), 'incomplete completion stream');
    const data = body.split(/\r?\n/u).filter(line => line.startsWith('data:')).map(line => line.slice(5).trim());
    assert.equal(data.at(-1), '[DONE]', 'incomplete completion stream');
    events = data.slice(0, -1).map(line => JSON.parse(line));
  }
  const calls = new Map();
  let content = '',
    finish = null;
  for (const [position, event] of events.entries()) {
    if (Array.isArray(event.choices) && event.choices.length === 0) {
      // include_usage adds one metadata-only trailer after the actual finish choice.
      assert.ok(streaming && finish !== null && position === events.length - 1, 'misplaced completion usage trailer');
      assert.equal(event.object, 'chat.completion.chunk');
      assert.ok(typeof event.id === 'string' && event.id.length > 0);
      assert.ok(typeof event.model === 'string' && event.model.length > 0);
      assert.ok(Number.isSafeInteger(event.created) && event.created >= 0);
      for (const earlier of events.slice(0, position)) for (const field of ['id', 'object', 'model', 'created']) {
        assert.equal(earlier[field], event[field], 'completion usage identity differs');
      }
      const usage = event.usage;
      assert.ok(usage && typeof usage === 'object' && !Array.isArray(usage), 'completion usage missing');
      for (const field of ['prompt_tokens', 'completion_tokens', 'total_tokens']) {
        assert.ok(Number.isSafeInteger(usage[field]) && usage[field] >= 0, 'invalid completion usage');
      }
      assert.equal(usage.total_tokens, usage.prompt_tokens + usage.completion_tokens, 'completion usage total differs');
      continue;
    }
    assert.equal(finish, null, 'completion payload after terminator');
    assert.ok(Array.isArray(event.choices) && event.choices.length === 1, 'one actual completion choice required');
    const choice = event.choices[0],
      message = choice.delta ?? choice.message;
    assert.ok(message && typeof message === 'object');
    if (typeof message.content === 'string') content += message.content;
    for (const [position, fragment] of (message.tool_calls ?? []).entries()) {
      const index = fragment.index ?? position;
      assert.ok(Number.isSafeInteger(index) && index >= 0);
      const call = calls.get(index) ?? {
        id: '',
        name: '',
        arguments: ''
      };
      if (fragment.id) call.id += fragment.id;
      if (fragment.function?.name) call.name += fragment.function.name;
      if (typeof fragment.function?.arguments === 'string') call.arguments += fragment.function.arguments;
      calls.set(index, call);
    }
    if (choice.finish_reason !== null && choice.finish_reason !== undefined) {
      assert.equal(finish, null, 'duplicate completion terminator');
      finish = choice.finish_reason;
    }
  }
  assert.ok(['stop', 'tool_calls'].includes(finish), 'missing successful completion terminator');
  const observed = [...calls.values()].map(call => ({
    ...call,
    arguments: JSON.parse(call.arguments)
  }));
  assert.ok(observed.every(call => call.id && call.name));
  assert.equal(new Set(observed.map(call => call.id)).size, observed.length, 'duplicate completion call id');
  assert.equal(finish === 'tool_calls', observed.length > 0, 'completion tool/terminal boundary differs');
  return {
    calls: observed,
    content,
    finish
  };
}
function relativeFile(value) {
  assert.ok(typeof value === 'string' && value.length > 0 && !path.isAbsolute(value));
  assert.ok(!/[\r\n\0\\]/u.test(value));
  assert.ok(value.split('/').every(part => part && part !== '.' && part !== '..'), 'unsafe effect path');
  return value;
}
// Normalize only the existing equivalent write-path field spellings; preserve every other argument.
function writeArguments(arguments_) {
  const fields = ['filePath', 'file_path', 'path'];
  const paths = fields.filter(key => arguments_[key] !== undefined).map(key => arguments_[key]);
  assert.ok(paths.length > 0 && paths.every(value => value === paths[0]), 'ambiguous emitted effect path');
  assert.equal(typeof arguments_.content, 'string');
  const remaining = Object.fromEntries(Object.entries(arguments_).filter(([key]) => !fields.includes(key)));
  return {
    ...remaining,
    path: relativeFile(paths[0])
  };
}
function observedFile(workspace, relative) {
  let location = workspace;
  for (const part of relativeFile(relative).split('/')) {
    location = path.join(location, part);
    assert.equal(fs.lstatSync(location).isSymbolicLink(), false, 'effect symlink');
  }
  const status = fs.statSync(location);
  assert.ok(status.isFile() && status.size <= 1024 * 1024);
  return fs.readFileSync(location);
}
export function verifyOutputProject({
  records,
  workspace,
  task,
  expectedFiles,
  verifier,
  expectedOutput
}) {
  assert.ok(typeof task === 'string' && task.length > 0);
  assert.ok(typeof expectedOutput === 'string' && expectedOutput.length > 0 && !/[\r\n\0]/u.test(expectedOutput));
  assert.ok(Array.isArray(expectedFiles) && expectedFiles.length > 0);
  expectedFiles = expectedFiles.map(relativeFile);
  assert.equal(new Set(expectedFiles).size, expectedFiles.length, 'duplicate configured effect');
  relativeFile(verifier);
  assert.ok(!verifier.startsWith('-') && !/\s/u.test(verifier), 'unsupported verification command path');
  assert.ok(expectedFiles.includes(verifier), 'verifier must be a declared effect');
  const command = 'sh ' + verifier;
  const calls = new Map(),
    returned = new Map(),
    writes = new Map();
  let verifiedCall = null,
    terminal = null,
    exchanges = 0;
  for (const record of records) {
    if (record.method !== 'POST' || record.path !== '/v1/chat/completions') continue;
    assert.equal(terminal, null, 'exchange after terminal completion');
    assert.ok(typeof record.request_body === 'string' && typeof record.response_body === 'string');
    const request = JSON.parse(record.request_body);
    assert.ok(Array.isArray(request.messages));
    assert.ok(request.messages.some(message => message.role === 'user' && text(message.content) === task), 'foreign configured task');
    const acknowledged = new Set();
    for (const message of request.messages) {
      if (message.role === 'assistant') for (const call of message.tool_calls ?? []) {
        const emitted = calls.get(call.id);
        assert.ok(emitted, 'unbound or stale assistant tool call');
        assert.equal(call.function?.name, emitted.name, 'returned tool name differs');
        const echoedArguments = JSON.parse(call.function.arguments);
        const write = ['write', 'write_file'].includes(emitted.name.toLowerCase());
        assert.deepEqual(write ? writeArguments(echoedArguments) : echoedArguments, write ? writeArguments(emitted.arguments) : emitted.arguments, 'returned tool arguments differ');
        acknowledged.add(call.id);
      }
      if (message.role !== 'tool') continue;
      const id = message.tool_call_id;
      if (!calls.has(id) || !acknowledged.has(id)) throw Error('unbound or stale tool result');
      const encoded = JSON.stringify(message);
      if (returned.has(id)) {
        assert.equal(returned.get(id), encoded, 'changed repeated tool result');
        continue;
      }
      assert.notEqual(message.is_error, true, 'failed tool result');
      returned.set(id, encoded);
      if (calls.get(id).name.toLowerCase() === 'bash' && calls.get(id).arguments.command === command) {
        assert.ok(text(message.content).includes(expectedOutput), 'verification output missing from actual returned tool result');
        verifiedCall = id;
      }
    }
    const response = completionResponse(record.response_body);
    exchanges += 1;
    for (const call of response.calls) {
      assert.equal(calls.has(call.id), false, 'duplicate current tool call id');
      calls.set(call.id, call);
      if (['write', 'write_file'].includes(call.name.toLowerCase())) {
        const file = writeArguments(call.arguments).path;
        if (expectedFiles.includes(file)) {
          assert.equal(writes.has(file), false, 'duplicate emitted effect');
          assert.equal(typeof call.arguments.content, 'string');
          writes.set(file, {
            id: call.id,
            bytes: Buffer.from(call.arguments.content)
          });
        }
      }
    }
    if (response.finish === 'stop') {
      assert.ok(verifiedCall && returned.has(verifiedCall), 'terminal completion before returned verification');
      assert.ok(response.content.includes(expectedOutput), 'terminal answer omitted requested output');
      terminal = response;
    }
  }
  assert.ok(terminal, 'terminal completion missing');
  for (const file of expectedFiles) {
    const write = writes.get(file);
    assert.ok(write && returned.has(write.id), 'missing bound file effect ' + file);
    assert.deepEqual(observedFile(workspace, file), write.bytes, 'physical effect bytes differ ' + file);
  }
  // A recorded result/final cannot replace independent execution in the actual sandbox.
  const execution = spawnSync('sh', [verifier], {
    cwd: workspace,
    encoding: 'utf8',
    timeout: 180000,
    maxBuffer: 1024 * 1024
  });
  assert.equal(execution.error, undefined);
  assert.equal(execution.signal, null);
  assert.equal(execution.status, 0, execution.stderr);
  assert.equal(execution.stdout, expectedOutput + '\n', 'independent verifier stdout differs');
  for (const file of expectedFiles) assert.deepEqual(observedFile(workspace, file), writes.get(file).bytes, 'effect changed during independent verification');
  return {
    schema: 'formal-ai-output-project-completion/v1',
    taskSha256: digest(task),
    exchanges,
    files: expectedFiles.map(file => ({
      path: file,
      sha256: digest(observedFile(workspace, file))
    })),
    verification: {
      callId: verifiedCall,
      command,
      exitCode: execution.status,
      stdoutSha256: digest(execution.stdout)
    },
    terminal: {
      finish: terminal.finish,
      contentSha256: digest(terminal.content)
    }
  };
}
export function readCompletionRecords(directory) {
  const records = [];
  for (const name of fs.readdirSync(directory).sort()) {
    if (!name.endsWith('.lino')) continue;
    const location = path.join(directory, name),
      status = fs.lstatSync(location);
    assert.ok(status.isFile() && !status.isSymbolicLink() && status.size <= 16 * 1024 * 1024);
    const raw = new TextDecoder('utf-8', {
      fatal: true
    }).decode(fs.readFileSync(location));
    assert.ok(raw.endsWith('\n'), 'incomplete actual dialog record');
    for (const line of raw.trimEnd().split('\n')) records.push(decodeDialogRecord(line));
  }
  assert.ok(records.length > 0, 'actual dialog records missing');
  return records.sort((left, right) => left.timestamp_unix_ms - right.timestamp_unix_ms);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [directory, workspace, task, firstFile, otherFiles, verifier, expectedOutput, report] = process.argv.slice(2);
  assert.ok(report);
  const result = verifyOutputProject({
    records: readCompletionRecords(directory),
    workspace,
    task,
    expectedFiles: [firstFile, ...otherFiles.split('\n').filter(Boolean)],
    verifier,
    expectedOutput
  });
  fs.writeFileSync(report, JSON.stringify(result, null, 2) + '\n', {
    flag: 'wx'
  });
  console.log('Actual file effects, returned verification, terminal completion and independent stdout verified');
}
