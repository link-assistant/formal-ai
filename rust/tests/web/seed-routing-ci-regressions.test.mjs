// Exact CI semantic failures and the general boundaries their repairs retain.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { meaning, wordIn } from '../../../js/agentic/crate/seed_meanings.mjs';
import { resolveSurface } from '../../../js/agentic/crate/text_formalization.mjs';
import { roundTrip } from '../../../js/agentic/crate/round_trip_translation.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { planCommitStep } from '../../../js/agentic/git_commit.mjs';
import { mentionsRole } from '../../../js/agentic/write_lexicon.mjs';
import { realm, solve } from '../../../js/agentic/host.mjs';
import { objectType, routePlaced } from '../../../js/agentic/crate/capability_routing.mjs';
import { EventLog, buildEvidenceLinks } from '../../../js/agentic/crate/event_log.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const tools = ['bash', 'read', 'grep', 'write', 'edit'];
const newline = String.fromCharCode(10);

test('complete commit cues own an existing-tree commit in every seeded language', async () => {
  for (const instruction of ['Please review these changes and commit them.', 'Проверь изменения и закоммить изменения.', 'इन परिवर्तनों की समीक्षा करें और बदलाव कमिट करें।', '请检查并提交这些更改。', 'Revisa los cambios y haz commit.']) {
    const prompt = instruction + newline + '?? Main.scala';
    const plan = await planChatStep([{ role: 'user', content: prompt }], tools);
    assert.equal(plan.calls[0].tool, 'bash', instruction);
    assert.match(JSON.parse(plan.calls[0].arguments).command, /git commit/u);
  }
});

test('commit cue masking preserves independent authoring and token boundaries', () => {
  for (const prompt of ['Implement a parser and haz commit.', 'Revisa haz committer y haz commit.', 'Revisa src/haz-committer.mjs y haz commit.', 'Append "haz commit" to notes.txt.']) {
    assert.equal(planCommitStep(prompt, [{ role: 'user', content: prompt }], tools), null, prompt);
  }
});

test('avoidance aliases cover contracted and expanded seeded instruction forms', async () => {
  for (const instruction of ['do not use', 'don t use', "don't use", 'don’t use']) {
    assert.ok(mentionsRole('conversation_preference_avoid', 'please ' + instruction + ' these anymore'));
    const result = await solve('`quick` is subjective opinion, please ' + instruction + ' these anymore.');
    assert.equal(result.intent, 'conversation_preference');
    assert.equal(result.answer, "Understood. I'll avoid `quick` in this dialog.");
  }
});

test('named transform metadata retains the original content-addressed provenance', () => {
  for (const name of ['requirement_list', 'custom_transform']) {
    const log = new EventLog();
    const id = log.append('text_transform', name);
    const links = buildEvidenceLinks('payload', log, 'response:result');
    assert.ok(links.includes('text_transform:' + name));
    assert.ok(links.includes('text_transform:' + id));
  }
});

test('proof promotion belongs to leading proof directives, not embedded test wording', () => {
  for (const prompt of ['Prove that there are infinitely many primes.', 'Докажи что простых чисел бесконечно много.', 'साबित करो कि अभाज्य संख्याएँ अनंत हैं।', '证明素数无穷。', 'Demuestra que hay infinitos números primos.']) {
    assert.ok(realm().solverPromotedHandlers(prompt).includes('proof_request'), prompt);
  }
  const prompt = 'Clona este proyecto en el commit indicado, localiza dónde se define el tiempo de espera por defecto, súbelo a sesenta segundos y ejecuta solo la prueba que comprueba ese valor por defecto.';
  assert.ok(!realm().solverPromotedHandlers(prompt).includes('proof_request'));
  assert.deepEqual(routePlaced(prompt, []), { kind: 'honest_gap', needed: 'grep', missing: 'shell' });
  assert.ok(objectType(prompt).includes('bare_term'));
});

test('authored returning concept loads all five languages and distinguishes return from equality', () => {
  const loaded = meaning('returning');
  assert.deepEqual(loaded.defined_by, ['coding_return']);
  assert.deepEqual(loaded.roles, ['coding_return_action']);
  for (const [language, surface] of [['en', 'returning'], ['ru', 'возвращая'], ['hi', 'लौटाते हुए'], ['zh', '正在返回'], ['es', 'devolviendo']]) {
    assert.equal(wordIn(loaded, language), surface);
    assert.equal(resolveSurface(surface, language), 'returning');
  }
  assert.equal(resolveSurface('返回结果', 'zh'), 'coding_return');
  assert.equal(resolveSurface('等于', 'zh'), 'coding_assertion_equality');
  for (const surface of ['return', 'returning', 'equals']) {
    assert.ok(roundTrip(surface, 'en', 'zh').survives, surface);
  }
});
