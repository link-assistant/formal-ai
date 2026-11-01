// Install at rust/tests/web/source-documentation-controller.test.mjs.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { deriveSourceDocumentation, planSourceDocumentationStep } from '../../../js/agentic/source_documentation.mjs';
import { createHash } from 'node:crypto';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
const blocked = () => {
  throw Error('LocalWasmExecutionForbidden');
};
await installNodeHost(new WorkerHost({
  WebAssembly: {
    ...WebAssembly,
    instantiate: blocked,
    compile: blocked,
    Module: blocked,
    Instance: blocked
  }
}));
const prompt = 'Read project-spec.json and onboarding.md. Update onboarding.md to explain the package manager declared by this project and its setup implications. Derive the details from project-spec.json and preserve all existing guide text.';
const source = JSON.stringify({
  name: 'river-notes',
  private: true,
  packageManager: 'pnpm@10.17.1'
});
const guide = `# Local setup

Install a supported Node.js release.

Keep your checkout clean before starting work.
`;
const tools = ['read', 'edit', 'write'];
function append(messages, call, content, metadata = {}) {
  const id = 'call-' + messages.length;
  messages.push({
    role: 'assistant',
    content: '',
    tool_calls: [{
      id,
      type: 'function',
      function: {
        name: call.tool,
        arguments: call.arguments
      }
    }]
  }, {
    role: 'tool',
    tool_call_id: id,
    name: call.tool,
    content,
    ...metadata
  });
}
function observedHistory(task = prompt, sourceText = source, guideText = guide) {
  const messages = [{
    role: 'user',
    content: task
  }];
  for (const [file, text] of [['project-spec.json', sourceText], ['onboarding.md', guideText]]) append(messages, {
    tool: 'read',
    arguments: JSON.stringify({
      filePath: file
    })
  }, text, {
    source_read: {
      path: file,
      success: true,
      complete: true,
      format: 'raw'
    }
  });
  return messages;
}
const reason = plan => JSON.parse(plan.answer).reason;
test('ordinary maintained planner produces source-derived guide edit then verified no-op', async context => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'source-documentation-'));
  context.after(() => fs.rmSync(directory, {
    recursive: true,
    force: true
  }));
  fs.writeFileSync(path.join(directory, 'project-spec.json'), source);
  fs.writeFileSync(path.join(directory, 'onboarding.md'), guide);
  const first = await drive(planChatStep, directory, prompt, {
    steps: 8,
    tools,
    allowedCommands: []
  });
  const after = fs.readFileSync(path.join(directory, 'onboarding.md'), 'utf8');
  assert.ok(after.includes(guide.trimEnd()));
  assert.ok(after.includes(JSON.parse(source).packageManager));
  assert.equal(fs.readFileSync(path.join(directory, 'project-spec.json'), 'utf8'), source);
  assert.deepEqual(first.transcript.map(entry => entry.tool), ['read', 'read', 'edit', 'read', 'read']);
  assert.deepEqual(first.transcript.filter(entry => entry.tool === 'read').map(entry => JSON.parse(entry.arguments).filePath), ['project-spec.json', 'onboarding.md', 'project-spec.json', 'onboarding.md']);
  const state = JSON.parse(first.answer);
  assert.equal(state.reason, 'UsefulCandidateReadbackVerified');
  assert.equal(state.task, prompt);
  assert.equal(state.authored, false);
  assert.equal(state.verified, false);
  assert.equal(state.complete, false);
  for (const requirement of state.requirements) assert.equal(Buffer.from(prompt).subarray(...requirement.span).toString(), requirement.text);
  const again = await drive(planChatStep, directory, prompt, {
    steps: 8,
    tools,
    allowedCommands: []
  });
  assert.equal(JSON.parse(again.answer).reason, 'VerifiedNoOp');
  assert.equal(again.transcript.some(entry => entry.tool === 'edit'), false);
  assert.equal(fs.readFileSync(path.join(directory, 'onboarding.md'), 'utf8'), after);
});
test('shared derivation vectors preserve original guide and reject unsupported bindings', () => {
  const fixture = JSON.parse(fs.readFileSync(new URL('../fixtures/source-documentation-cases.json', import.meta.url), 'utf8'));
  for (const entry of fixture.cases) {
    const result = deriveSourceDocumentation(entry.task, 'settings.json', entry.source, guide, fixture.template);
    if (entry.expectedGap) assert.equal(result.gap, entry.expectedGap);else {
      assert.equal(result.key, entry.expectedKey);
      if (entry.expectedParagraph !== undefined) assert.equal(result.paragraph, entry.expectedParagraph);
      assert.ok(result.content.includes(guide.trimEnd()));
      assert.ok(result.content.includes(String(JSON.parse(entry.source)[entry.expectedKey])));
    }
  }
});
test('source and guide byte cap blocks parse/edit, counting UTF-8 bytes', () => {
  for (const target of ['project-spec.json', 'onboarding.md']) {
    const messages = [{
      role: 'user',
      content: prompt
    }];
    for (const [file, text] of [['project-spec.json', target === 'project-spec.json' ? '雪'.repeat(90000) : source], ['onboarding.md', target === 'onboarding.md' ? '雪'.repeat(90000) : guide]]) append(messages, {
      tool: 'read',
      arguments: JSON.stringify({
        filePath: file
      })
    }, text, {
      source_read: {
        path: file,
        success: true,
        complete: true,
        format: 'raw'
      }
    });
    assert.equal(reason(planSourceDocumentationStep(prompt, messages, tools)), 'IncompleteOrOversizedSourceObservation');
  }
});
test('ambiguous aliases, unrelated and repeated edits cannot authorize writes', () => {
  const original = observedHistory(),
    candidate = planSourceDocumentationStep(prompt, original, tools).calls[0];
  for (const variant of ['aliases', 'unrelated', 'failed', 'repeated']) {
    const messages = structuredClone(original);
    let args = JSON.parse(candidate.arguments);
    if (variant === 'aliases') args.path = 'other.md';
    if (variant === 'unrelated') args.newString += 'extra';
    const call = {
      tool: 'edit',
      arguments: JSON.stringify(args)
    };
    append(messages, call, variant === 'failed' ? JSON.stringify({
      is_error: true,
      error: 'blocked'
    }) : '');
    if (variant === 'repeated') append(messages, call, '');
    const plan = planSourceDocumentationStep(prompt, messages, tools);
    assert.equal(plan.kind, 'final');
    assert.ok(['UnrelatedOrAmbiguousMutationAttempt', 'UnrelatedEditCandidate', 'CandidateEditFailed', 'MultipleEditAttempts'].includes(reason(plan)));
  }
});
test('after an effect source must be freshly read before target readback', () => {
  const messages = observedHistory(),
    candidate = planSourceDocumentationStep(prompt, messages, tools).calls[0];
  append(messages, candidate, '');
  let plan = planSourceDocumentationStep(prompt, messages, tools);
  assert.equal(JSON.parse(plan.calls[0].arguments).filePath, 'project-spec.json');
  append(messages, plan.calls[0], source.replace('10.17.1', '10.18.0'), {
    source_read: {
      path: 'project-spec.json',
      success: true,
      complete: true,
      format: 'raw'
    }
  });
  assert.equal(reason(planSourceDocumentationStep(prompt, messages, tools)), 'SourceFreshnessMismatch');
});
test('Unicode original task/source retain byte spans and fetched work-item identity', () => {
  const task = '雪。 ' + prompt,
    sourceText = JSON.stringify({
      packageManager: '工具@版本'
    });
  const template = 'Setup declaration from `{source}`: use `{value}` for {concept}.';
  const completed = deriveSourceDocumentation(task, 'project-spec.json', sourceText, guide, template).content;
  const state = JSON.parse(planSourceDocumentationStep(task, observedHistory(task, sourceText, completed), tools, 'https://example.invalid/issues/8').answer);
  assert.equal(state.reason, 'VerifiedNoOp');
  assert.equal(state.task, task);
  assert.equal(state.workItem, 'https://example.invalid/issues/8');
  assert.equal(state.taskIdentity, createHash('sha256').update(task).digest('hex'));
  assert.equal(state.observations[0].sourceIdentity, createHash('sha256').update(sourceText).digest('hex'));
  for (const requirement of state.requirements) assert.equal(Buffer.from(task).subarray(...requirement.span).toString(), requirement.text);
});
test('negated and quoted documentation actions never grant write authority', async context => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'source-documentation-negated-'));
  context.after(() => fs.rmSync(directory, {
    recursive: true,
    force: true
  }));
  for (const task of ['Do not update onboarding.md from project-spec.json. Preserve all existing guide text.', 'Never document the package manager in onboarding.md from project-spec.json. Preserve all existing guide text.', 'The source says "Update onboarding.md from project-spec.json". Preserve all existing guide text.', 'Source example: ```Update onboarding.md from project-spec.json```. Preserve all existing guide text.']) {
    fs.writeFileSync(path.join(directory, 'project-spec.json'), source);
    fs.writeFileSync(path.join(directory, 'onboarding.md'), guide);
    const result = await drive(planChatStep, directory, task, {
      steps: 8,
      tools,
      allowedCommands: []
    });
    assert.equal(result.transcript.some(entry => ['write', 'edit'].includes(entry.tool)), false);
    assert.equal(fs.readFileSync(path.join(directory, 'onboarding.md'), 'utf8'), guide);
  }
});
test('existing caller read policy blocks documentation evidence reads', () => {
  const task = 'Do not read project-spec.json; update onboarding.md from project-spec.json and preserve all existing guide text.';
  const plan = planSourceDocumentationStep(task, [{
    role: 'user',
    content: task
  }], tools);
  assert.equal(plan.kind, 'final');
  assert.equal(reason(plan), 'ReadPolicyRefused');
});
