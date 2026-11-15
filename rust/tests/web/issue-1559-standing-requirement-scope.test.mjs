// Issue #1559: a system prompt of one agent session must never become a
// learned standing requirement that is replayed in another repository's
// session. Twin of rust/tests/unit/issue_1559_standing_requirement_scope.rs.
//
// The recorded prompts are the solver prompt of the Scala test session
// (rust/tests/fixtures/issue-1559/scala-solver-prompt.txt) and the same prompt
// for the Rust test session of the issue's evidence: another repository and
// another branch.

import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { chatCompletionFromPlan, responseFromPlan } from '../../../js/server/agentic.mjs';
import { applyDreamingPlan, dreamingConfig, planMemoryDreaming } from '../../../js/server/dreaming-plan.mjs';
import { memoizedReplay, replayAnswerWithAmendments } from '../../../js/server/dreaming-replay.mjs';
import { runCoreDreamingOnce } from '../../../js/server/dreaming-runtime.mjs';
import { createMemory } from '../../../js/server/memory.mjs';
import { SyncStore, memoryEvent } from '../../../js/server/memory-store.mjs';
import { handleChatCompletions } from '../../../js/server/openai.mjs';
import {
  boundedStatement, containsTaskIdentifier, isSessionInstruction, isTopicStopword, isValidTopic, ruleMaxChars,
  statementTopic,
} from '../../../js/server/standing-requirement-bounds.mjs';
import { applyRetainedAmendments, retainedAmendments } from '../../../js/server/standing-requirements.mjs';
import {
  eventsInScope, recordedScopeEvidence, repositoryOf, requestScope, runInRequestScope, scopeKey, scopePairs,
} from '../../../js/server/standing-scope.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const SCALA_REPOSITORY = 'konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b';
const SCALA_BRANCH = 'issue-1-1f3e3886bcb8';
const RUST_REPOSITORY = 'konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c';
const RUST_BRANCH = 'issue-1-09b0c76bd0e4';
const SCALA_PROMPT = fs.readFileSync(
  new URL('../fixtures/issue-1559/scala-solver-prompt.txt', import.meta.url),
  'utf8',
);
const RUST_PROMPT = SCALA_PROMPT.replaceAll(SCALA_REPOSITORY, RUST_REPOSITORY).replaceAll(SCALA_BRANCH, RUST_BRANCH);
const AMENDMENT_LINE = 'Learned standing requirement';
const TOOLS = [{
  type: 'function',
  function: {
    name: 'bash',
    description: 'Run a shell command',
    parameters: { type: 'object', properties: { command: { type: 'string' } }, required: ['command'] },
  },
}];
const AGENT_SYSTEM = { role: 'system', content: 'You are Agent, the best coding agent on the planet.' };
// The stored amendment of the issue's evidence: topic "you", the whole prompt as its rule.
const LEGACY_AMENDMENT = memoryEvent({
  id: 'amendment_legacy_you',
  kind: 'meta_algorithm_amendment',
  role: 'system',
  content: SCALA_PROMPT,
  inputs: 'topic=you',
  outputs: `rule=${SCALA_PROMPT}`,
  evidence: ['chat_user_legacy', 'recipe:data/meta/dreaming-recipe.lino'],
});

function server(t, extraEnv = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-issue-1559-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const file = path.join(directory, 'memory.lino');
  const env = { ...process.env, FORMAL_AI_MEMORY_PATH: file };
  delete env.FORMAL_AI_SHARE_STANDING_REQUIREMENTS;
  Object.assign(env, extraEnv);
  const ctx = { worker: new WorkerHost(), agentMode: true, memory: createMemory(env), env };
  const chat = async (body) => {
    const response = await handleChatCompletions(ctx, { body: JSON.stringify({ model: 'formal-ai', ...body }) });
    assert.equal(response.statusCode, 200, response.body);
    return { json: JSON.parse(response.body), text: response.body };
  };
  const events = () => SyncStore.open(env).events;
  const dream = () => runCoreDreamingOnce(file, { env, replay: memoizedReplay() });
  const append = (...extra) => {
    const store = SyncStore.open(env);
    store.events = [...store.events, ...extra];
    store.persist();
  };
  return { ctx, env, file, chat, events, dream, append };
}

const scopeKeyOf = (event) => scopeKey(scopePairs(event.evidence));
const amendmentsOf = (events) => events.filter((event) => event.kind === 'meta_algorithm_amendment');
const valueOf = (field, key) => String(field ?? '').replace(new RegExp(`^${key}=`, 'u'), '');

function assertBoundedAmendments(events) {
  for (const amendment of amendmentsOf(events)) {
    const topic = valueOf(amendment.inputs, 'topic');
    const rule = valueOf(amendment.outputs, 'rule');
    assert.ok(isValidTopic(topic), `pronoun or function-word topic stored: ${topic}`);
    assert.equal(isTopicStopword(topic.toLowerCase()) && topic.toLowerCase() !== 'answer', false, topic);
    assert.ok([...rule].length <= ruleMaxChars(), `rule over the limit: ${rule.length}`);
    assert.ok(!containsTaskIdentifier(rule), `rule names one task: ${rule}`);
    for (const identifier of [SCALA_REPOSITORY, SCALA_BRANCH, RUST_REPOSITORY, RUST_BRANCH, 'You are an AI issue solver']) {
      assert.ok(!rule.includes(identifier), `rule carries ${identifier}`);
    }
  }
}

test('the solver prompt opens like a session instruction and yields no bounded statement', () => {
  assert.equal(isSessionInstruction(SCALA_PROMPT), true);
  assert.equal(boundedStatement(SCALA_PROMPT), null);
  // Without its opener, no single sentence of the prompt survives the bounds
  // with a task identifier in it.
  const body = SCALA_PROMPT.split('\n').slice(1).join('\n');
  const statement = boundedStatement(body);
  if (statement !== null) {
    assert.ok([...statement].length <= ruleMaxChars());
    assert.equal(containsTaskIdentifier(statement), false, statement);
  }
  assert.equal(boundedStatement(`Always push only to branch ${SCALA_BRANCH}.`), null);
  assert.equal(boundedStatement(`Always use gh pr list --repo ${SCALA_REPOSITORY}.`), null);
  assert.equal(boundedStatement('Always keep logs in /tmp/gh-issue-solver-1794699168252/ci-logs.'), null);
  assert.equal(boundedStatement('Never close issue #1 without a test.'), null);
  assert.equal(boundedStatement(`Always ${'answer briefly and '.repeat(20)}politely.`), null);
  assert.equal(boundedStatement('Always answer in Russian.'), 'Always answer in Russian.');
  assert.equal(repositoryOf(SCALA_PROMPT), `github.com/${SCALA_REPOSITORY}`);
  assert.equal(repositoryOf(RUST_PROMPT), `github.com/${RUST_REPOSITORY}`);
});

test('pronouns and function words are never topics in en, ru, hi and zh', () => {
  for (const [statement, topic] of [
    ['You must always answer in Russian.', 'answer'],
    ['You should always run the linter.', 'run'],
    ['Ты всегда должен отвечать по-русски.', 'отвечать'],
    ['Вы должны всегда писать тесты.', 'писать'],
    ['तुम्हें हमेशा परीक्षण लिखने चाहिए।', 'परीक्षण'],
    ['你必须总是用中文回答。', '回答'],
    ['你 必须 始终 写 测试', '测试'],
    ['It must never break the build.', 'break'],
  ]) {
    assert.equal(statementTopic(statement), topic, statement);
  }
  for (const pronoun of ['you', 'it', 'they', 'ты', 'вы', 'он', 'आप', 'तुम', '你', '您', '他们']) {
    assert.equal(isValidTopic(pronoun), false, pronoun);
  }
  assert.equal(isValidTopic('answer'), true);
});

test('replaying the two recorded prompts in order shows no text of the first prompt in the second session', { timeout: 120000 }, async (t) => {
  const host = server(t);
  const first = await host.chat({ messages: [AGENT_SYSTEM, { role: 'user', content: SCALA_PROMPT }], tools: TOOLS });
  assert.ok(first.text.includes(SCALA_REPOSITORY), 'the first session works on its own repository');

  const recorded = host.events().filter((event) => event.kind === 'message');
  assert.equal(recorded.length, 1);
  assert.equal(recorded[0].role, 'instruction', 'the solver prompt is marked as an instruction at receipt');
  assert.deepEqual(scopePairs(recorded[0].evidence).map(([key]) => key), ['repository', 'session']);
  assert.ok(recorded[0].evidence.includes(`scope:repository=github.com/${SCALA_REPOSITORY}`));

  host.dream();
  const afterDreaming = host.events();
  assert.equal(amendmentsOf(afterDreaming).length, 0, 'no requirement is learned from a session instruction');
  assertBoundedAmendments(afterDreaming);

  // The amendment the issue found on the Hive Mind host is in the store too.
  host.append(LEGACY_AMENDMENT);
  const second = await host.chat({ messages: [AGENT_SYSTEM, { role: 'user', content: RUST_PROMPT }], tools: TOOLS });
  assert.ok(second.text.includes(RUST_REPOSITORY), 'the second session works on its own repository');
  for (const leaked of [SCALA_REPOSITORY, SCALA_BRANCH, AMENDMENT_LINE]) {
    assert.equal(second.text.includes(leaked), false, `the second session shows ${leaked}`);
  }

  // The working-session summary the issue quoted is the agentic final answer.
  const rustMessages = [AGENT_SYSTEM, { role: 'user', content: RUST_PROMPT }];
  const rustScope = requestScope({ messages: rustMessages, tools: TOOLS, env: host.env });
  const plan = { kind: 'final', answer: 'Summary: the Rust hello world prints its greeting.' };
  const request = { model: 'formal-ai', messages: rustMessages, tools: TOOLS };
  const summary = runInRequestScope(rustScope, () => chatCompletionFromPlan(host.ctx, request, RUST_PROMPT, plan, 1));
  assert.equal(summary.choices[0].message.content, plan.answer);
  const summaryText = JSON.stringify(summary);
  for (const leaked of [SCALA_REPOSITORY, SCALA_BRANCH, AMENDMENT_LINE, 'You are an AI issue solver']) {
    assert.equal(summaryText.includes(leaked), false, `the second summary shows ${leaked}`);
  }
});

test('a requirement stated inside one agent session stays in that session', { timeout: 120000 }, async (t) => {
  const host = server(t);
  const scala = [AGENT_SYSTEM, { role: 'user', content: SCALA_PROMPT }];
  await host.chat({ messages: [...scala, { role: 'user', content: 'Always run the formatter before committing.' }], tools: TOOLS });
  host.dream();
  const amendments = amendmentsOf(host.events());
  assert.equal(amendments.length, 1, JSON.stringify(amendments));
  const [amendment] = amendments;
  assert.equal(valueOf(amendment.inputs, 'topic'), 'run');
  assert.equal(valueOf(amendment.outputs, 'rule'), 'Always run the formatter before committing.');
  assert.ok(amendment.evidence.includes(`scope:repository=github.com/${SCALA_REPOSITORY}`));
  assertBoundedAmendments(host.events());

  const scalaScope = requestScope({ messages: scala, tools: TOOLS });
  const rustScope = requestScope({ messages: [AGENT_SYSTEM, { role: 'user', content: RUST_PROMPT }], tools: TOOLS });
  assert.equal(amendmentsOf(eventsInScope(host.events(), scalaScope)).length, 1);
  assert.equal(amendmentsOf(eventsInScope(host.events(), rustScope)).length, 0);
  assert.equal(amendmentsOf(eventsInScope(host.events(), null)).length, 0);

  const second = await host.chat({
    messages: [AGENT_SYSTEM, { role: 'user', content: `${RUST_PROMPT}\nPlease run the formatter.` }],
    tools: TOOLS,
  });
  assert.equal(second.text.includes('formatter before committing'), false);
  assert.equal(second.text.includes(AMENDMENT_LINE), false);
});

test('sharing standing requirements between agent sessions is an explicit opt-in', () => {
  const messages = [AGENT_SYSTEM, { role: 'user', content: SCALA_PROMPT }];
  const isolated = requestScope({ messages, tools: TOOLS, env: {} });
  assert.deepEqual(recordedScopeEvidence(isolated).map((entry) => entry.split('=')[0]), ['scope:repository', 'scope:session']);
  const shared = requestScope({ messages, tools: TOOLS, env: { FORMAL_AI_SHARE_STANDING_REQUIREMENTS: '1' } });
  assert.deepEqual(recordedScopeEvidence(shared), []);
  const user = requestScope({ body: { user: 'alice' }, messages, tools: TOOLS, env: {} });
  assert.deepEqual(recordedScopeEvidence(user), ['scope:user=alice']);
  assert.deepEqual(recordedScopeEvidence(requestScope({ messages, tools: [], env: {} })), []);
});

test('the stored amendment of the issue is never applied', () => {
  assert.deepEqual(retainedAmendments([LEGACY_AMENDMENT]), []);
  const answer = { intent: 'unknown', answer: 'Done.', evidence_links: [] };
  assert.deepEqual(applyRetainedAmendments('Can you summarize what you changed?', answer, [LEGACY_AMENDMENT]), answer);
});

test('"always answer in Russian" is learned and applied in that user\'s later conversations only', { timeout: 120000 }, async (t) => {
  const host = server(t);
  host.ctx.agentMode = false;
  const ask = async (user, content) => (await host.chat({ messages: [{ role: 'user', content }], ...(user ? { user } : {}) })).json.choices[0].message.content;
  await ask('alice', 'Always answer in Russian.');
  host.dream();
  const amendments = amendmentsOf(host.events());
  assert.equal(amendments.length, 1, JSON.stringify(amendments));
  assert.equal(amendments[0].inputs, 'topic=answer');
  assert.equal(amendments[0].outputs, 'rule=Always answer in Russian.');
  assert.ok(amendments[0].evidence.includes('scope:user=alice'));
  assertBoundedAmendments(host.events());

  assert.match(await ask('alice', 'What is 2 + 2?'), /Learned standing requirement \(answer\): Always answer in Russian\./u);
  assert.match(await ask('alice', 'Name a prime number.'), /Always answer in Russian\./u);
  assert.doesNotMatch(await ask('bob', 'What is 2 + 2?'), /Russian/u);
  assert.doesNotMatch(await ask(null, 'What is 2 + 2?'), /Russian/u);
});

test('a statement made in two scopes stays scoped until replay verifies a generalization', { timeout: 120000 }, async (t) => {
  const host = server(t);
  host.ctx.agentMode = false;
  const ask = async (user, content) => (await host.chat({ messages: [{ role: 'user', content }], ...(user ? { user } : {}) })).json.choices[0].message.content;
  await ask('alice', 'Always answer in Russian.');
  await ask('bob', 'Always answer in Russian.');
  host.dream();
  // No recorded task replays the rule, so nothing verifies a generalization.
  const amendments = amendmentsOf(host.events());
  assert.deepEqual(amendments.map((amendment) => scopeKeyOf(amendment)).sort(), ['user=alice', 'user=bob']);
  assert.equal(amendments.some((amendment) => amendment.evidence.includes('scope:generalized')), false);
  assert.doesNotMatch(await ask('carol', 'What is 2 + 2?'), /Russian/u);
});

test('a statement made in two scopes and verified by replay becomes one generalized amendment', () => {
  const rule = 'Always include a LaTeX verification step.';
  const input = 'Explain a latex proof by induction';
  const requirement = (id, user) => memoryEvent({
    id, kind: 'message', role: 'user', content: rule, conversation_title: 'latex', evidence: [`scope:user=${user}`],
  });
  const verifiedRun = memoryEvent({
    id: 'run-verified',
    kind: 'test_run',
    role: 'derived',
    content: input,
    inputs: input,
    outputs: replayAnswerWithAmendments(input, [{ id: 'expected', topic: 'latex', rule }]),
    conversation_title: 'latex',
  });
  const replay = memoizedReplay();
  const learn = (events) => amendmentsOf(applyDreamingPlan(events, planMemoryDreaming(events, dreamingConfig(), replay), replay).events);

  const scoped = learn([requirement('req-alice', 'alice'), requirement('req-bob', 'bob')]);
  assert.deepEqual(scoped.map((amendment) => scopeKeyOf(amendment)).sort(), ['user=alice', 'user=bob']);

  const generalized = learn([requirement('req-alice', 'alice'), requirement('req-bob', 'bob'), verifiedRun]);
  assert.equal(generalized.length, 1, JSON.stringify(generalized));
  assert.deepEqual(scopePairs(generalized[0].evidence), []);
  assert.ok(generalized[0].evidence.includes('scope:generalized'));
  assert.deepEqual(generalized[0].evidence.filter((entry) => entry.startsWith('req-')), ['req-alice', 'req-bob']);
  assert.equal(retainedAmendments(eventsInScope(generalized, { user: 'carol' })).length, 1);
});

test('an agentic final answer lists applied amendments in its trace, never in its text', () => {
  const amendment = memoryEvent({
    id: 'amendment_scoped_answer',
    kind: 'meta_algorithm_amendment',
    role: 'system',
    content: 'Always answer in Russian.',
    inputs: 'topic=answer',
    outputs: 'rule=Always answer in Russian.',
    evidence: ['chat_user_1', 'scope:user=alice', 'recipe:data/meta/dreaming-recipe.lino'],
  });
  const ctx = { agentMode: true, memory: { events: () => [amendment] } };
  const request = { model: 'formal-ai', messages: [{ role: 'user', content: 'Summarize the work.' }], tools: TOOLS };
  const plan = { kind: 'final', answer: 'Implemented the change and pushed it.' };
  const alice = requestScope({ body: { user: 'alice' }, messages: request.messages, tools: TOOLS, env: {} });
  const bob = requestScope({ body: { user: 'bob' }, messages: request.messages, tools: TOOLS, env: {} });

  const chat = runInRequestScope(alice, () => chatCompletionFromPlan(ctx, request, 'Summarize the work.', plan, 1));
  const message = chat.choices[0].message;
  assert.equal(message.content, plan.answer);
  assert.deepEqual(message.thinking_steps.map((step) => step.source_event), ['meta_algorithm_amendment:amendment_scoped_answer']);
  assert.match(message.reasoning_content, /learned standing requirement \(answer\)/u);
  assert.equal(message.reasoning_content.includes('Always answer in Russian'), false);

  const response = runInRequestScope(alice, () => responseFromPlan(ctx, request, request, 'Summarize the work.', plan, 1));
  assert.equal(response.output[0].content[0].text, plan.answer);
  assert.deepEqual(response.evidence_links, ['meta_algorithm_amendment:amendment_scoped_answer']);
  assert.equal(JSON.stringify(response.output).includes('Always answer in Russian'), false);

  const other = runInRequestScope(bob, () => chatCompletionFromPlan(ctx, request, 'Summarize the work.', plan, 1));
  assert.equal(other.choices[0].message.thinking_steps, undefined);
  assert.equal(other.choices[0].message.content, plan.answer);
});
