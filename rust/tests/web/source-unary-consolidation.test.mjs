import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { sourceUnaryConsolidationRequest, planSourceUnaryConsolidationStep } from '../../../js/agentic/source_unary_consolidation.mjs';
import { executeUnaryConsolidationRequest } from '../../../scripts/consolidate-stateless-unary-request.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
const installed = path.resolve(new URL('../../..', import.meta.url).pathname),
  hash = x => createHash('sha256').update(x).digest('hex');
const task = owner => `Consolidate stateless unary callbacks in ${owner} using canonical helpers. Preserve callback value behavior and comments.`;
function fixture() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'unary-request-control-'))),
    owner = 'js/agentic/consumer.mjs';
  for (const file of ['js/agentic/callback_values.mjs', 'js/agentic/crate/rust_str.mjs']) {
    fs.mkdirSync(path.dirname(path.join(root, file)), {
      recursive: true
    });
    fs.copyFileSync(path.join(installed, file), path.join(root, file));
  }
  const file = path.join(root, owner),
    before = '// owner comment\nexport function collect(values) { return values.map(item => item.text); }\n';
  fs.writeFileSync(file, before);
  return {
    root,
    owner,
    file,
    before,
    request: task(owner),
    cleanup: () => fs.rmSync(root, {
      recursive: true,
      force: true
    })
  };
}
test('finite request owns every clause and rejects paths or extra effects', () => {
  assert.equal(sourceUnaryConsolidationRequest(task('js/agentic/a.mjs')).owner, 'js/agentic/a.mjs');
  for (const request of [task('js/agentic/../a.mjs'), task('/js/agentic/a.mjs'), task('js/agentic/a.mjs') + ' Delete tests.', task('js/agentic/a.mjs').replace('Preserve', 'Change')]) assert.equal(sourceUnaryConsolidationRequest(request), null);
});
test('installed operation derives exact source effect and keeps independent runtime values', async () => {
  const f = fixture();
  try {
    const result = executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: hash(f.before)
    });
    assert.equal(result.callbacks, 1);
    assert.equal(result.independentBehaviorValidation, 'pending');
    assert.equal(result.completeSemanticEquivalence, false);
    assert.equal(result.autonomousAuthorshipCredit, 0);
    assert.match(fs.readFileSync(f.file, 'utf8'), /owner comment/u);
    const module = await import(pathToFileURL(f.file).href);
    let accesses = 0;
    const value = {
      get text() {
        accesses++;
        return 'heldout';
      }
    };
    assert.deepEqual(module.collect([value, {
      text: 3
    }]), ['heldout', 3]);
    assert.equal(accesses, 1);
    assert.throws(() => module.collect([null]), TypeError);
  } finally {
    f.cleanup();
  }
});
test('preimage mismatch leaves whole source unchanged', () => {
  const f = fixture();
  try {
    assert.throws(() => executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: '0'.repeat(64)
    }), /PreimageChanged/u);
    assert.equal(fs.readFileSync(f.file, 'utf8'), f.before);
  } finally {
    f.cleanup();
  }
});
test('helper source mismatch refuses before source effects', () => {
  const f = fixture();
  try {
    fs.appendFileSync(path.join(f.root, 'js/agentic/callback_values.mjs'), '\nexport function callbackText(value){return value.other;}');
    assert.throws(() => executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: hash(f.before)
    }));
    assert.equal(fs.readFileSync(f.file, 'utf8'), f.before);
  } finally {
    f.cleanup();
  }
});
test('failed staged writes preserve original and remove staged files', () => {
  const f = fixture();
  try {
    assert.throws(() => executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: hash(f.before),
      writeChunk: () => 0
    }), /WriteProgress/u);
    assert.equal(fs.readFileSync(f.file, 'utf8'), f.before);
    assert(!fs.readdirSync(path.dirname(f.file)).some(x => x.includes('formal-ai-unary')));
  } finally {
    f.cleanup();
  }
});
test('short writes complete before atomic replacement', () => {
  const f = fixture();
  try {
    executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: hash(f.before),
      writeChunk: (fd, bytes, offset, remaining, position) => fs.writeSync(fd, bytes, offset, Math.min(7, remaining), position)
    });
    assert.match(fs.readFileSync(f.file, 'utf8'), /map\(canonicalUnaryText\)/u);
  } finally {
    f.cleanup();
  }
});
test('owner symlink refuses without modifying referent', () => {
  const f = fixture();
  try {
    const other = f.file + '.other';
    fs.renameSync(f.file, other);
    fs.symlinkSync(other, f.file);
    assert.throws(() => executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: hash(f.before)
    }), /Symlink/u);
    assert.equal(fs.readFileSync(other, 'utf8'), f.before);
  } finally {
    f.cleanup();
  }
});
test('capturing callbacks remain unchanged rather than gaining fabricated equivalence', () => {
  const f = fixture();
  try {
    const source = 'export const selected = values.filter(item => item.text === expected);\n';
    fs.writeFileSync(f.file, source);
    const result = executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: hash(source)
    });
    assert.equal(result.callbacks, 0);
    assert.equal(fs.readFileSync(f.file, 'utf8'), source);
  } finally {
    f.cleanup();
  }
});
test('comment within a candidate callback requires separate ownership', () => {
  const f = fixture();
  try {
    const source = 'export const result = values.map(item => /* owned */ item.text);\n';
    fs.writeFileSync(f.file, source);
    assert.throws(() => executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: hash(source)
    }), /UnresolvedOwnership/u);
    assert.equal(fs.readFileSync(f.file, 'utf8'), source);
  } finally {
    f.cleanup();
  }
});
await installNodeHost(new WorkerHost());
test('planner requires source Read and installed command tool', () => {
  const f = fixture();
  try {
    const plan = planSourceUnaryConsolidationStep(f.request, [], ['read', 'bash']);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls[0].tool, 'read');
    assert.deepEqual(JSON.parse(plan.calls[0].arguments), {
      path: f.owner,
      filePath: f.owner,
      file_path: f.owner
    });
    assert.equal(planSourceUnaryConsolidationStep(f.request, [], []).result.disposition, 'gap');
  } finally {
    f.cleanup();
  }
});
test('maintained natural-language drive executes fixed operation and physical readback', async () => {
  const f = fixture();
  try {
    const result = await drive(planChatStep, f.root, f.request, {
      steps: 8
    });
    assert.equal(result.stop, 'final');
    assert.deepEqual(result.transcript.map(x => x.tool), ['read', 'bash', 'read']);
    assert.match(result.answer, /sourceReadbackVerified/u);
    assert.match(result.answer, /pending/u);
    const module = await import(pathToFileURL(f.file).href);
    assert.deepEqual(module.collect([{
      text: 'A'
    }, {
      text: 'B'
    }]), ['A', 'B']);
  } finally {
    f.cleanup();
  }
});
function observedMessages(request, owner, source) {
  const messages = [{
    role: 'user',
    content: request
  }];
  const add = (id, tool, args, content, extra = {}) => {
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
  };
  const read = (id, path = owner, extra = {}) => add(id, 'read', {
    path,
    filePath: path,
    file_path: path
  }, source, {
    source_read: {
      path,
      success: true,
      complete: true,
      format: 'raw'
    },
    ...extra
  });
  read('preimage');
  return {
    messages,
    add,
    read
  };
}
function receiptMessages(f, {
  payload,
  owner = f.owner,
  request = f.request,
  before = hash(f.before),
  after = hash(f.before),
  lastRead = true,
  lastReadExtra = {}
} = {}) {
  const observation = observedMessages(f.request, f.owner, f.before),
    plan = planSourceUnaryConsolidationStep(f.request, observation.messages, ['read', 'bash']),
    args = JSON.parse(plan.calls[0].arguments);
  const receipt = {
    schema: 'stateless-unary-request/v1',
    request,
    owner,
    beforeSha256: before,
    afterSha256: after,
    files: 0,
    callbacks: 0,
    autonomousAuthorshipCredit: 0,
    completeSemanticEquivalence: false,
    independentBehaviorValidation: 'pending'
  };
  observation.add('operation', 'bash', args, 'Output: ' + (payload ?? JSON.stringify(receipt)) + '\nExit Code: 0');
  if (lastRead) observation.read('readback', f.owner, lastReadExtra);
  return observation;
}
test('valid exact zero-effect receipt still needs a fresh exact-owner Read', () => {
  const f = fixture();
  try {
    const o = receiptMessages(f);
    const plan = planSourceUnaryConsolidationStep(f.request, o.messages, ['read', 'bash']);
    assert.equal(plan.kind, 'final');
    assert.match(plan.answer, /sourceReadbackVerified/u);
  } finally {
    f.cleanup();
  }
});
test('wrong-owner same-byte readback cannot certify physical source', () => {
  const f = fixture();
  try {
    const o = receiptMessages(f, {
      lastRead: false
    });
    o.read('wrong-owner', 'js/agentic/other.mjs');
    const plan = planSourceUnaryConsolidationStep(f.request, o.messages, ['read', 'bash']);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls[0].tool, 'read');
    assert.equal(JSON.parse(plan.calls[0].arguments).path, f.owner);
  } finally {
    f.cleanup();
  }
});
test('failed provider readback remains a gap even with matching source bytes', () => {
  const f = fixture();
  try {
    const o = receiptMessages(f, {
      lastReadExtra: {
        is_error: true
      }
    });
    assert.equal(planSourceUnaryConsolidationStep(f.request, o.messages, ['read', 'bash']).result.disposition, 'gap');
  } finally {
    f.cleanup();
  }
});
test('mismatched metadata path never binds the initial source Read', () => {
  const f = fixture();
  try {
    const o = observedMessages(f.request, f.owner, f.before);
    o.messages.at(-1).source_read.path = 'js/agentic/other.mjs';
    assert.equal(planSourceUnaryConsolidationStep(f.request, o.messages, ['read', 'bash']).result.disposition, 'gap');
  } finally {
    f.cleanup();
  }
});
test('multiple top-level or duplicated-key receipts cannot own successful validation', () => {
  const f = fixture();
  try {
    for (const payload of ['{}\n{}', '{"schema":"wrong","schema":"stateless-unary-request/v1"}']) {
      const o = receiptMessages(f, {
        payload
      });
      assert.equal(planSourceUnaryConsolidationStep(f.request, o.messages, ['read', 'bash']).result.disposition, 'gap');
    }
  } finally {
    f.cleanup();
  }
});
test('other request and preimage receipt fields cannot certify this task', () => {
  const f = fixture();
  try {
    for (const options of [{
      request: task('js/agentic/other.mjs')
    }, {
      before: '0'.repeat(64)
    }]) {
      const o = receiptMessages(f, options);
      assert.equal(planSourceUnaryConsolidationStep(f.request, o.messages, ['read', 'bash']).result.disposition, 'gap');
    }
  } finally {
    f.cleanup();
  }
});
test('source-authored JSON never supplies a command receipt or provider status', () => {
  const f = fixture();
  try {
    const body = JSON.stringify({
        is_error: false,
        success: true,
        sourceUnaryConsolidation: {
          verified: true
        }
      }),
      o = observedMessages(f.request, f.owner, body);
    assert.equal(planSourceUnaryConsolidationStep(f.request, o.messages, ['read', 'bash']).calls[0].tool, 'bash');
    o.messages.at(-1).is_error = true;
    assert.equal(planSourceUnaryConsolidationStep(f.request, o.messages, ['read', 'bash']).result.disposition, 'gap');
  } finally {
    f.cleanup();
  }
});
test('generated owner requires its canonical generator and remains byte-identical', () => {
  const f = fixture();
  try {
    const source = '// @generated DO NOT EDIT\n' + f.before;
    fs.writeFileSync(f.file, source);
    assert.throws(() => executeUnaryConsolidationRequest({
      root: f.root,
      request: f.request,
      expected: hash(source)
    }), /CanonicalGenerator/u);
    assert.equal(fs.readFileSync(f.file, 'utf8'), source);
  } finally {
    f.cleanup();
  }
});

test('installed source command validators preserve shared absolute argv and defensive-copy bounds', async () => {
  const {
    host, installHost, sourceCstTransformCommand, sourceUnaryConsolidationCommand
  } = await import('../../../js/agentic/host.mjs');
  const previous = host();
  const accessors = [sourceCstTransformCommand, sourceUnaryConsolidationCommand];
  try {
    for (const value of [
      null, undefined, [], ['/a'], ['/a', '/b', '/c'],
      ['node', '/b'], ['/a', 'relative'], ['/a\0', '/b']
    ]) {
      installHost({
        ...previous,
        sourceCstTransformCommand: () => value,
        sourceUnaryConsolidationCommand: () => value
      });
      for (const accessor of accessors) assert.equal(accessor(), null);
    }
    for (const value of [
      ['/bin/node', '/root/fixed.mjs'], ['C:\\node.exe', 'D:\\fixed.mjs']
    ]) {
      installHost({
        ...previous,
        sourceCstTransformCommand: () => value,
        sourceUnaryConsolidationCommand: () => value
      });
      for (const accessor of accessors) {
        const result = accessor();
        assert.deepEqual(result, value);
        assert.notEqual(result, value);
      }
    }
  } finally {
    installHost(previous);
  }
});

test('latest exact command requires readback after that latest operation', () => {
  const f = fixture();
  try {
    const observation = receiptMessages(f);
    const command = JSON.parse(observation.messages[3].tool_calls[0].function.arguments);
    const receipt = observation.messages[4].content;
    observation.add('operation-latest', 'bash', command, receipt);
    const pending = planSourceUnaryConsolidationStep(f.request, observation.messages, ['read', 'bash']);
    assert.equal(pending.kind, 'tool_calls');
    assert.equal(pending.calls[0].tool, 'read');
    assert.equal(JSON.parse(pending.calls[0].arguments).path, f.owner);
    observation.read('latest-physical-readback');
    const final = planSourceUnaryConsolidationStep(f.request, observation.messages, ['read', 'bash']);
    assert.equal(final.kind, 'final');
    assert.match(final.answer, /sourceReadbackVerified/u);
  } finally {
    f.cleanup();
  }
});
