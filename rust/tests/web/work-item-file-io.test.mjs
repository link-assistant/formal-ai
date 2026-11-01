import test, { before } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, realpathSync, mkdirSync, writeFileSync, readFileSync, rmSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkItemFileProgress, fileIoSearchStep, fileIoReadStep, fileIoEffectStep, fileIoCommand, fileIoPendingStep } from '../../../js/agentic/work_item_file_io.mjs';
import { responseArgumentsForTool } from '../../../js/agentic/crate/protocol_responses.mjs';
const evidence = JSON.parse(readFileSync('data/benchmarks/fixtures/work-item-file-io/client-schemas.json', 'utf8'));
before(async () => installNodeHost(new WorkerHost({
  WebAssembly: {
    compile() {
      throw Error('LocalProductWasmForbidden');
    },
    instantiate() {
      throw Error('LocalProductWasmForbidden');
    }
  }
})));
const task = 'Update the requested repository file.';
function workspace() {
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'formal-ai-client-file-io-')));
  mkdirSync(join(root, 'src'));
  writeFileSync(join(root, 'src/owner.mjs'), 'export const enabled = false;\n');
  return root;
}
function history(root) {
  return [{
    role: 'system',
    content: '<env>\nWorking directory: ' + root + '\n</env>'
  }, {
    role: 'user',
    content: task
  }];
}
function observe(messages, tool, args, content, extra = {}) {
  const id = 'file-io-' + messages.length;
  messages.push({
    role: 'assistant',
    tool_calls: [{
      id,
      type: 'function',
      function: {
        name: tool,
        arguments: JSON.stringify(args)
      }
    }]
  }, {
    role: 'tool',
    tool_call_id: id,
    name: tool,
    content,
    ...extra
  });
}
function execute(root, messages, step) {
  const call = step.calls[0];
  assert.equal(call.tool, 'exec_command');
  const args = JSON.parse(responseArgumentsForTool(evidence.codexTools, call.tool, call.arguments, task, root));
  assert.equal(typeof args.cmd, 'string');
  assert.equal(args.command, undefined);
  const result = spawnSync('/bin/sh', ['-c', args.cmd], {
    cwd: root,
    encoding: 'utf8',
    timeout: 3000,
    maxBuffer: 262144
  });
  assert.equal(result.error, undefined);
  const frame = 'Chunk ID: fixture\nWall time: 0.01 seconds\nProcess exited with code ' + result.status + '\nOutput:\n' + result.stdout;
  observe(messages, call.tool, args, frame, {
    is_error: result.status !== 0
  });
  return result;
}
test('Codex required cmd uses registered real IO, guarded replace and actual readback', () => {
  const root = workspace(),
    messages = history(root);
  try {
    execute(root, messages, fileIoReadStep(messages, ['exec_command', 'apply_patch'], 'src/owner.mjs', task).step);
    let progress = WorkItemFileProgress.scan(messages);
    const before = progress.sourceReadFor('src/owner.mjs').source;
    assert.equal(before, 'export const enabled = false;\n');
    assert.equal(progress.attempts[0].transport.kind, 'registered-run-file-io');
    execute(root, messages, fileIoEffectStep(messages, ['exec_command', 'apply_patch'], 'src/owner.mjs', before, 'export const enabled = true;\n').step);
    progress = WorkItemFileProgress.scan(messages);
    assert.equal(progress.gap, null);
    assert.equal(JSON.parse(progress.attempts[1].arguments).oldString, before);
    execute(root, messages, fileIoReadStep(messages, ['exec_command', 'apply_patch'], 'src/owner.mjs', task).step);
    progress = WorkItemFileProgress.scan(messages);
    assert.equal(progress.sourceReadFor('src/owner.mjs').source, 'export const enabled = true;\n');
    assert.equal(readFileSync(join(root, 'src/owner.mjs'), 'utf8'), 'export const enabled = true;\n');
  } finally {
    rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
test('Claude projected absolute aliases resolve only under exact caller root', () => {
  const root = workspace(),
    messages = history(root);
  try {
    const source = readFileSync(join(root, 'src/owner.mjs'), 'utf8');
    const args = JSON.parse(responseArgumentsForTool(evidence.claudeTools, 'Read', JSON.stringify({
      filePath: 'src/owner.mjs'
    }), task, root));
    assert.equal(args.file_path, join(root, 'src/owner.mjs'));
    observe(messages, 'Read', args, source, {
      source_read: {
        path: args.file_path,
        success: true,
        complete: true,
        format: 'raw'
      }
    });
    const progress = WorkItemFileProgress.scan(messages);
    assert.equal(progress.sourceReadFor('src/owner.mjs').source, source);
    const outside = history(root);
    observe(outside, 'Read', {
      file_path: root + '-foreign/src/owner.mjs'
    }, source, {
      source_read: {
        path: root + '-foreign/src/owner.mjs',
        success: true,
        complete: true,
        format: 'raw'
      }
    });
    assert.equal(WorkItemFileProgress.scan(outside).sourceReadFor('src/owner.mjs'), null);
  } finally {
    rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
test('stale preimage and symlink paths fail without credit or altered source', () => {
  const root = workspace();
  try {
    const before = readFileSync(join(root, 'src/owner.mjs'), 'utf8');
    for (const command of [fileIoCommand('replace', root, 'src/owner.mjs', 'stale source', 'wrong bytes'), (() => {
      symlinkSync('owner.mjs', join(root, 'src/link.mjs'));
      return fileIoCommand('read', root, 'src/link.mjs');
    })()]) {
      const result = spawnSync('/bin/sh', ['-c', command], {
        cwd: root,
        encoding: 'utf8',
        timeout: 3000,
        maxBuffer: 262144
      });
      assert.equal(result.error, undefined);
      assert.notEqual(result.status, 0);
      assert.equal(readFileSync(join(root, 'src/owner.mjs'), 'utf8'), before);
    }
  } finally {
    rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
test('missing outer process status earns no observation; synthetic pending metadata only requests bounded polling', () => {
  const root = workspace();
  try {
    const command = fileIoCommand('read', root, 'src/owner.mjs');
    const missing = history(root);
    observe(missing, 'exec_command', {
      cmd: command
    }, 'ordinary unframed output');
    assert.equal(WorkItemFileProgress.scan(missing).sourceReadFor('src/owner.mjs')?.complete, false);
    const pending = history(root);
    observe(pending, 'exec_command', {
      cmd: command
    }, JSON.stringify({
      output: '',
      session_id: 41
    }));
    const progress = WorkItemFileProgress.scan(pending);
    assert.equal(progress.sourceReadFor('src/owner.mjs'), null);
    const poll = fileIoPendingStep(progress, ['exec_command', 'write_stdin']);
    assert.equal(poll.step.calls[0].tool, 'write_stdin');
    assert.equal(JSON.parse(poll.step.calls[0].arguments).chars, '');
  } finally {
    rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
test('registered bounded search returns candidate custody and requires fresh reads', () => {
  const root = workspace(),
    messages = history(root);
  try {
    mkdirSync(join(root, 'src/nested'));
    writeFileSync(join(root, 'src/nested/consumer.rs'), '// candidate_engine is syntax search, not binding proof.\n');
    writeFileSync(join(root, 'src/other.rs'), 'fn unrelated() {}\n');
    mkdirSync(join(root, 'src/.private'));
    writeFileSync(join(root, 'src/.private/secret.rs'), 'candidate_engine');
    execute(root, messages, fileIoSearchStep(messages, ['exec_command'], 'src', ['candidate_engine'], task).step);
    const progress = WorkItemFileProgress.scan(messages),
      search = progress.searchFor('src', ['candidate_engine']);
    assert.equal(search.status, 'reported-success');
    assert.equal(search.complete, true);
    assert.equal(search.transport.actualExitCode, 0);
    assert.equal(search.root, root);
    assert.deepEqual(search.files.map(file => file.path), ['src/nested/consumer.rs']);
    assert.equal(search.files[0].sha256, createHash('sha256').update(readFileSync(join(root, 'src/nested/consumer.rs'))).digest('hex'));
    assert.equal(progress.attempts.length, 0);
    assert.equal(progress.sourceReadFor('src/nested/consumer.rs'), null);
    assert.equal(progress.searchFor('src', ['unrelated']), null);
    assert.equal(progress.searchFor('other', ['candidate_engine']), null);
    assert.ok(search.receipt.scope.includes('text matches are not bindings'));
    execute(root, messages, fileIoReadStep(messages, ['exec_command'], 'src/nested/consumer.rs', task).step);
    assert.ok(WorkItemFileProgress.scan(messages).sourceReadFor('src/nested/consumer.rs').complete);
  } finally {
    rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
test('search symlinks and match overflow are explicitly incomplete', () => {
  const root = workspace();
  try {
    for (let index = 0; index < 9; index++) writeFileSync(join(root, 'src/consumer' + index + '.rs'), 'candidate_engine::invoke();\n');
    let messages = history(root);
    execute(root, messages, fileIoSearchStep(messages, ['exec_command'], 'src', ['candidate_engine'], task).step);
    let search = WorkItemFileProgress.scan(messages).searchFor('src', ['candidate_engine']);
    assert.equal(search.complete, false);
    assert.equal(search.files.length, 8);
    assert.ok(search.receipt.gaps.some(gap => gap.kind === 'SearchMatchBudget'));
    mkdirSync(join(root, 'tiny'));
    symlinkSync(join(root, 'src/consumer0.rs'), join(root, 'tiny/alias.rs'));
    messages = history(root);
    execute(root, messages, fileIoSearchStep(messages, ['exec_command'], 'tiny', ['candidate_engine'], task).step);
    search = WorkItemFileProgress.scan(messages).searchFor('tiny', ['candidate_engine']);
    assert.equal(search.complete, false);
    assert.deepEqual(search.files, []);
    assert.ok(search.receipt.gaps.some(gap => gap.kind === 'SearchSymlinkRefused'));
    assert.equal(fileIoSearchStep(messages, ['exec_command'], '../src', ['candidate_engine'], task).gap, 'FileIoSearchScopeRequired');
    assert.equal(fileIoSearchStep(messages, ['exec_command'], 'src', ['bad-name'], task).gap, 'FileIoSearchScopeRequired');
  } finally {
    rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
