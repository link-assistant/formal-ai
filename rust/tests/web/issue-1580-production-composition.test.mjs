// A generic prose join cannot satisfy a concrete program-output contract.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let executor;
const host = new WorkerHost();
before(async () => {
  await installNodeHost(host);
  executor = await import('../../../js/agentic/crate/meta_executor.mjs');
});

test('production composition preserves concrete output obligations, including skill gaps', async () => {
  for (const intent of ['write_program_skill_gap', 'write_program_unspecified', 'write_program']) {
    const root = { intent, content: 'The source implementation is unavailable.', confidence: 0 };
    const { answer, run } = await executor.solveAnswer(
      { text: 'Create source and import assert', language: 'en' }, root,
      async () => ({ intent: 'read_file', content: 'This surface needs a file-reading tool.', confidence: 1 }),
    );
    assert.equal(answer.intent, intent);
    assert.equal(answer.content, root.content);
    assert.notEqual(run.outcome.candidate?.method, 'composition');
    assert.ok(run.events.some(({ kind }) => kind === 'meta_exec:compose'));
  }
});

test('production still composes independently answered tasks for an untyped unresolved root', async () => {
  const { answer, run } = await executor.solveAnswer(
    { text: 'First task and second task', language: 'en' },
    { intent: 'unknown', content: '', confidence: 0 },
    async (text) => ({ intent: 'answered', content: text.startsWith('First') ? 'one' : 'two', confidence: 1 }),
  );
  assert.equal(run.outcome.candidate.method, 'composition');
  assert.match(answer.content, /one[\s\S]*two/u);
});

test('concept queries trim terminal punctuation and preserve punctuation inside the subject', async () => {
  for (const punctuation of ['?', '？', '!', '！', '。', ',', '，', ';', '；', ':', '：']) {
    const prompt = `Redis是什么${punctuation}`;
    const query = await host.run('extractConceptQuery(__conceptPrompt)', { __conceptPrompt: prompt });
    assert.ok(query, prompt);
    assert.equal(query.term, 'redis', prompt);
  }
  const query = await host.run('extractConceptQuery("What is foo.bar?")');
  assert.equal(query.term, 'foo.bar');
});
