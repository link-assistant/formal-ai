import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { canDeliverFinal } from '../../../js/agentic/plan.mjs';
import { pinnedLineNeed } from '../../../js/agentic/planner/owned_goals.mjs';
await installNodeHost(new WorkerHost());
const tools = ["bash", "batch", "codesearch", "edit", "glob", "grep", "list", "read", "task", "todoread", "todowrite", "webfetch", "websearch", "write"];
const cases = [{
  "request": "Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. The first line must be exactly `checklist=v2`.",
  "expectedWrites": [{
    "path": "release-checklist.md",
    "content": "checklist=v2\n\nverify the tag, publish the crate, push the image.\n"
  }]
}, {
  "request": "Create a.txt with exactly this content «alpha». The first line must be exactly `beta`.",
  "expectedWrites": []
}, {
  "request": "Create a.txt with exactly this content «alpha». The first line must be exactly `beta`. Then do unknown work.",
  "expectedWrites": []
}, {
  "request": "Create a.txt with exactly this content «alphaExtra». The first line must be exactly `alpha`.",
  "expectedWrites": []
}, {
  "request": "Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. The first line must be exactly `checklist=v2`. Then deploy it.",
  "expectedWrites": []
}, {
  "request": "Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. The first line must be exactly `checklist=v2`. Then do unknown future work.",
  "expectedWrites": []
}, {
  "request": "Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. The first line must be exactly `checklist=v2`. Do not write any files.",
  "expectedWrites": []
}, {
  "request": "Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. The first line must be exactly `checklist=v2`. Read README.md first.",
  "expectedWrites": []
}, {
  "request": "Do not write any files. Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. The first line must be exactly `checklist=v2`.",
  "expectedWrites": []
}, {
  "request": "Read README.md first. Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. The first line must be exactly `checklist=v2`.",
  "expectedWrites": []
}, {
  "request": "Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. Deploy it. The first line must be exactly `checklist=v2`.",
  "expectedWrites": []
}, {
  "request": "Create a.txt with exactly this content «alpha». The first line must be exactly `alpha`.",
  "expectedWrites": [{
    "path": "a.txt",
    "content": "alpha"
  }]
}];
function physicalFiles(directory) {
  return fs.readdirSync(directory).flatMap(name => {
    const file = path.join(directory, name);
    return fs.statSync(file).isDirectory() ? physicalFiles(file) : [file];
  });
}
function physicalResult(call, directory, writes, receipts) {
  const argumentsValue = JSON.parse(call.arguments);
  const relative = argumentsValue.path ?? argumentsValue.filePath ?? argumentsValue.file_path;
  const local = target => {
    const file = path.resolve(directory, target);
    assert(file.startsWith(directory + path.sep), 'fixture path must stay isolated');
    return file;
  };
  if (call.tool === 'write') {
    const content = argumentsValue.content ?? argumentsValue.contents ?? argumentsValue.text;
    const file = local(relative);
    fs.mkdirSync(path.dirname(file), {
      recursive: true
    });
    fs.writeFileSync(file, content);
    assert.equal(fs.readFileSync(file, 'utf8'), content);
    writes.push({
      path: relative,
      content
    });
    return JSON.stringify({
      success: true,
      path: relative,
      bytes: Buffer.byteLength(content)
    });
  }
  if (call.tool === 'read') {
    try {
      return fs.readFileSync(local(relative), 'utf8');
    } catch (error) {
      return JSON.stringify({
        is_error: true,
        error: error.message
      });
    }
  }
  if (call.tool === 'list') return JSON.stringify(fs.readdirSync(directory));
  if (call.tool !== 'bash') return JSON.stringify({ is_error: true, error: 'unsupported physical fixture tool' });
  const command = argumentsValue.command;
  assert(command.startsWith('mkdir -p -- .formal-ai && (') || /^cat [A-Za-z0-9_.\/-]+$/u.test(command), 'undeclared command');
  const receipt = spawnSync('sh', ['-c', command], {
    cwd: directory,
    env: {
      PATH: process.env.PATH
    },
    encoding: 'utf8',
    timeout: 2800,
    maxBuffer: 1048576
  });
  receipts.push({
    command,
    status: receipt.status,
    error: receipt.error
  });
  return JSON.stringify({
    stdout: receipt.stdout,
    stderr: receipt.stderr,
    exit_code: receipt.status,
    timed_out: receipt.error?.code === 'ETIMEDOUT',
    complete: receipt.status !== null
  });
}
for (const fixture of cases) {
  test(fixture.request, async () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-first-line-'));
    try {
      const messages = [{
        role: 'user',
        content: fixture.request
      }];
      const plans = [],
        writes = [],
        receipts = [];
      for (let turn = 0; turn < 6; turn++) {
        const plan = await planChatStepResolved(messages, tools);
        plans.push(plan);
        if (plan?.kind !== 'tool_calls') break;
        for (const [index, call] of plan.calls.entries()) {
          const identifier = 'physical-first-line-' + turn + '-' + index;
          const content = physicalResult(call, directory, writes, receipts);
          messages.push({
            role: 'assistant',
            tool_calls: [{
              id: identifier,
              type: 'function',
              function: {
                name: call.tool,
                arguments: call.arguments
              }
            }]
          });
          messages.push({
            role: 'tool',
            name: call.tool,
            tool_call_id: identifier,
            content
          });
        }
      }
      assert.equal(plans.at(-1)?.kind, 'final', 'original six-turn cap');
      assert.deepEqual(writes, fixture.expectedWrites);
      if (fixture.expectedWrites.length === 0) {
        assert.equal(physicalFiles(directory).length, 0, 'negative request has zero physical file effects');
        assert.equal(receipts.length, 0, 'negative request executes no shell effect');
        assert.equal(canDeliverFinal(plans.at(-1)), false, 'refusal is not delivered intent');
      } else {
        assert.equal(plans.length, 4, 'original literal write and readback sequence');
        assert.equal(canDeliverFinal(plans.at(-1)), true);
        assert.equal(receipts.length, 2, 'real plan event and readback receipts');
        for (const receipt of receipts) assert.equal(receipt.status, 0);
        for (const write of writes) assert.equal(fs.readFileSync(path.join(directory, write.path), 'utf8'), write.content);
      }
    } finally {
      fs.rmSync(directory, {
        recursive: true,
        force: true
      });
    }
  });
}
test('consumed first-line modifier carries exact immutable spans', () => {
  const fixture = cases[0];
  const content = fixture.expectedWrites[0].content;
  const need = pinnedLineNeed(fixture.request, content, 96);
  assert.equal(need.unit, 'utf16');
  assert.deepEqual(need.span, [96, 143]);
  assert.equal(fixture.request.slice(...need.literalSpan), need.expected);
  assert.equal(content.split('\n')[0], need.expected);
  assert.equal(pinnedLineNeed(fixture.request, 'wrong\n' + content, 96), null);
  assert.equal(pinnedLineNeed(fixture.request + ' Then deploy it.', content, 96), null);
});
