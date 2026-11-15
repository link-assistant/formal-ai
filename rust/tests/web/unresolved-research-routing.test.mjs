import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {performance} from 'node:perf_hooks';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host, installHost} from '../../../js/agentic/host.mjs';
import {unresolvedWebResearchQueryFor} from '../../../js/agentic/web_research.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';

let realHost;
before(async () => {
  await installNodeHost(new WorkerHost(), {sourceUsageRustGrammar: false});
  realHost = host();
});
// These units emulate the observed unknown-intent collaborator contract. The
// unchanged workspace-search integration on main exercises the full solver.
async function emulatedSolve(answer, check) {
  const calls = [];
  installHost({...realHost, solve: async (prompt, history) => {
    calls.push({prompt, history: structuredClone(history)});
    return {intent: answer(prompt, history)};
  }});
  try { await check(calls); } finally { installHost(realHost); }
}
const remove = 'Remove all uses of add.';
const user = content => ({role: 'user', content});

test('emulation: identical single block keeps its query and both admission guards without a third solve', async () => {
  await emulatedSolve(() => 'unknown', async calls => {
    assert.equal(await unresolvedWebResearchQueryFor([user(remove)]), 'Remove all uses of add');
    assert.equal(calls.length, 2, 'unresolved classification and conversation-summary classification both run');
    assert.deepEqual(calls.map(call => call.prompt), [remove, remove]);
    assert.deepEqual(calls.map(call => call.history), [[], []]);
  });
});

test('emulation: meaningful history still refuses a conversation-summary classification', async () => {
  const preceding = [user('The private project code is violet.'), {role: 'assistant', content: 'I will retain that context.'}];
  await emulatedSolve((_prompt, history) => history.length ? 'summarize_conversation' : 'unknown', async calls => {
    assert.equal(await unresolvedWebResearchQueryFor([...preceding, user(remove)]), null);
    assert.equal(calls.length, 2);
    assert.deepEqual(calls[0].history, []);
    assert.deepEqual(calls[1].history, preceding);
  });
});

test('emulation: a resolved first classification still stops before summary or block work', async () => {
  await emulatedSolve(() => 'calculation', async calls => {
    assert.equal(await unresolvedWebResearchQueryFor([user(remove)]), null);
    assert.equal(calls.length, 1);
  });
});

test('emulation: distinct blocks retain their individual classification and first unresolved research subject', async () => {
  const known = 'Count 1 plus 1.';
  const research = 'Find current weather in London.';
  const compound = known + '\n\n' + research;
  await emulatedSolve(prompt => prompt === known ? 'calculation' : 'unknown', async calls => {
    assert.equal(await unresolvedWebResearchQueryFor([user(compound)]), 'Find current weather in London');
    assert.deepEqual(calls.map(call => call.prompt), [compound, compound, known, research]);
  });
});

test('emulation: original scoped search and mutation-negative routing assertions stay fast with observed unknown intent', async () => {
  await emulatedSolve(() => 'unknown', async calls => {
    const started = performance.now();
    const conversation = prompt => [{role: 'system', content: '<env>\n  Working directory: /work/demo\n</env>'}, user(prompt)];
    const tools = ['read', 'write', 'edit', 'bash'];
    const scoped = await planChatStep(conversation('Count the lines containing foo in f.txt.'), tools);
    assert.equal(JSON.parse(scoped.calls[0].arguments).command, "grep -rnHw --exclude-dir=.git -- 'foo' 'f.txt'");
    for (const prompt of ['Rename all usages of add to plus.', remove]) {
      const plan = await planChatStep(conversation(prompt), tools);
      assert.ok(!(plan?.calls?.[0]?.arguments ?? '').includes('grep -rn'), `${prompt}: ${JSON.stringify(plan)}`);
    }
    assert.ok(calls.some(call => call.prompt === remove), 'the real fallback reaches the emulated solver');
    assert.ok(performance.now() - started < 10000, 'offline routing unit stays inside its focused test budget');
  });
});
