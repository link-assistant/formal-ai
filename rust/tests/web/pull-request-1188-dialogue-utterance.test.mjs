// A greeting is not a bare term for the open web. The opencode leg of the
// Agentic CLI Matrix (CI run 37710079108) sent "hi" with eleven tools
// advertised; the capability table read it as `(bare_term, retrieve, web)`,
// searched the web for it, read a dictionary page, and blew the greeting
// case's four-round bound. The routing seed now names the dialogue-act roles
// (`dialogue_utterance_role`), and a request made only of their surfaces is
// left to the engine, which answers it in the reply. Driven through the
// JavaScript planner the server runs; the Rust twin is
// `crate::capability_routing::is_dialogue_utterance`.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

const TOOLS = ['bash', 'edit', 'glob', 'grep', 'read', 'skill', 'task', 'todowrite', 'webfetch', 'websearch', 'write'];
const conversation = (text) => [
  { role: 'system', content: 'You are opencode.' },
  { role: 'user', content: text },
];

let planChatStep;
let routing;
let seedMeanings;
let solve;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
  routing = await import('../../../js/agentic/crate/capability_routing.mjs');
  seedMeanings = await import('../../../js/agentic/crate/seed_meanings.mjs');
  ({ solve } = await import('../../../js/agentic/host.mjs'));
});

describe('a dialogue act is answered by the engine, not searched', () => {
  test('the seed names dialogue roles, each with surfaces', () => {
    const roles = routing.dialogueUtteranceRoles();
    assert.ok(roles.includes('social_greeting'), roles.join(' '));
    for (const role of roles) {
      assert.ok(seedMeanings.wordsForRole(role).length > 0, `${role} has no surface in the seed`);
    }
  });

  for (const role of ['social_greeting', 'social_farewell', 'social_courtesy_response']) {
    test(`every surface of ${role} plans no tool call over a full toolset`, async () => {
      for (const surface of seedMeanings.wordsForRole(role)) {
        assert.ok(routing.isDialogueUtterance(surface), `${surface} is not read as a dialogue act`);
        assert.equal(await planChatStep(conversation(surface), TOOLS), null, surface);
      }
    });
  }

  test('the engine answers the greeting the planner left to it', async () => {
    const answer = await solve('hi', []);
    assert.notEqual(answer.intent, 'unknown');
    assert.notEqual(answer.intent, 'web_search');
    assert.match(String(answer.answer ?? answer.text ?? ''), /\p{L}{2,}/u);
  });

  test('a request beyond the greeting still reaches research', async () => {
    assert.equal(routing.isDialogueUtterance('hi, what is the weather in Paris'), false);
    assert.equal(routing.isDialogueUtterance('rust ownership'), false);
    const plan = await planChatStep(conversation('hi, what is the weather in Paris'), TOOLS);
    assert.equal(plan?.calls?.[0]?.tool, 'websearch');
  });
});

test('calendar day evidence requires a whole word outside CJK', () => {
  for (const prompt of ['Describe India.', 'Explain multimedia archives.', 'daylight']) {
    assert.ok(!routing.objectType(prompt).includes('time_expression'), prompt);
  }
  for (const prompt of ['day', 'days', 'dia', 'día', '今天', '日子']) {
    assert.ok(routing.objectType(prompt).includes('time_expression'), prompt);
  }
});

test('Wikipedia summary is not a calendar request', async () => {
  const prompt = 'Summarize the Wikipedia article on Rust in one paragraph.';
  assert.ok(!routing.objectType(prompt).includes('time_expression'));
  assert.notEqual(routing.routePlaced(prompt, ['calendar_create_event', 'web_search', 'summarize_topic'])?.capability, 'calendar_create_event');
  const response = await solve(prompt, []);
  assert.ok(response.intent.startsWith('summarize'), response.intent);
});
