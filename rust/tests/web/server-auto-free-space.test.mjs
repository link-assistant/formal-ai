// The consent-gated auto-free-space pass the JavaScript server runs on
// memory import (js/server/storage-policy.mjs), against the behaviour of
// rust/src/storage_policy.rs, rust/src/dreaming.rs and
// rust/src/dreaming/apply.rs (the Rust cases live in
// rust/tests/unit/memory_maintenance.rs).

import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { stableId } from '../../../js/server/ids.mjs';
import {
  SyncStore, exportLinksNotation, memoryEvent, parseLinksNotation,
} from '../../../js/server/memory-store.mjs';
import { dreamingConfig, planMemoryDreaming } from '../../../js/server/dreaming-plan.mjs';
import { memoryEventDebug, reconstructionRecord } from '../../../js/server/dreaming-retention.mjs';
import {
  AUTO_FREE_SPACE_CHOICE, applyAutoFreeSpaceWithSnapshot, autoFreeSpaceChoice, autoFreeSpacePreferencePath,
  measureStorage, persistAutoFreeSpaceChoice,
} from '../../../js/server/storage-policy.mjs';
import { debugStr } from '../../../js/server/dreaming-support.mjs';
import { memoizedReplay, replayAnswerWithAmendments } from '../../../js/server/dreaming-replay.mjs';

const noReplay = () => {
  throw new Error('these stores hold no task to replay');
};

/** rust/tests/unit/memory_maintenance.rs `recomputable_event`. */
function recomputableEvent(id, payload) {
  return memoryEvent({
    id,
    kind: 'source:http',
    role: 'cache',
    tool: 'web_search',
    content: payload,
    outputs: payload,
    evidence: ['rediscover:https://doc.rust-lang.org/std/'],
    write_count: 1,
  });
}

function tempMemory(name) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), `formal-ai-auto-free-${name}-`));
  return path.join(dir, 'memory.lino');
}

function writeStore(file, events) {
  fs.writeFileSync(file, exportLinksNotation(events, 2));
}

const ids = (events) => events.map((event) => event.id);

function deletedConversationFixture() {
  return [
    memoryEvent({ id: 'kept-user', kind: 'message', role: 'user', content: 'keep me', conversation_id: 'conv-live', write_count: 1 }),
    memoryEvent({ id: 'gone-user', kind: 'message', role: 'user', content: 'forget me', conversation_id: 'conv-gone', write_count: 1 }),
    memoryEvent({ id: 'gone-answer', kind: 'message', role: 'assistant', content: 'forgotten', conversation_id: 'conv-gone', write_count: 1 }),
    memoryEvent({ id: 'gone-marker', kind: 'conversation_deleted', role: 'system', conversation_id: 'conv-gone', write_count: 1 }),
  ];
}

const IMPORTED = 'demo_memory\n  event "imported-note"\n    kind "note"\n    role "user"\n    content "hello"\n';

test('without consent the import keeps every event', () => {
  const file = tempMemory('absent');
  writeStore(file, deletedConversationFixture());
  assert.equal(autoFreeSpaceChoice(file), AUTO_FREE_SPACE_CHOICE.NeverAsked);
  const store = SyncStore.open({ FORMAL_AI_MEMORY_PATH: file }, file);
  assert.equal(store.importLinksNotation(IMPORTED), 1);
  assert.deepEqual(ids(store.events), ['kept-user', 'gone-user', 'gone-answer', 'gone-marker', 'imported-note']);
  assert.deepEqual(ids(parseLinksNotation(fs.readFileSync(file, 'utf8'))), ids(store.events));
});

test('a declined choice keeps every event and is not "never asked"', () => {
  const file = tempMemory('disabled');
  writeStore(file, deletedConversationFixture());
  persistAutoFreeSpaceChoice(file, false);
  assert.equal(fs.readFileSync(autoFreeSpacePreferencePath(file), 'utf8'), 'disabled\n');
  assert.equal(autoFreeSpaceChoice(file), AUTO_FREE_SPACE_CHOICE.Declined);
  const store = SyncStore.open({ FORMAL_AI_MEMORY_PATH: file }, file);
  store.importLinksNotation(IMPORTED);
  assert.deepEqual(ids(store.events), ['kept-user', 'gone-user', 'gone-answer', 'gone-marker', 'imported-note']);
});

test('with consent the import purges a deleted conversation, marker included', () => {
  const file = tempMemory('enabled');
  writeStore(file, deletedConversationFixture());
  fs.writeFileSync(autoFreeSpacePreferencePath(file), '  enabled \n');
  assert.equal(autoFreeSpaceChoice(file), AUTO_FREE_SPACE_CHOICE.Enabled);
  const store = SyncStore.open({ FORMAL_AI_MEMORY_PATH: file }, file);
  assert.equal(store.importLinksNotation(IMPORTED), 1);
  assert.deepEqual(ids(store.events), ['kept-user', 'imported-note']);
  assert.deepEqual(ids(parseLinksNotation(fs.readFileSync(file, 'utf8'))), ['kept-user', 'imported-note']);
});

test('the preference sidecar sits next to the memory file', () => {
  assert.equal(autoFreeSpacePreferencePath('/data/memory.lino'), '/data/memory.lino.auto-free-space');
  const snapshot = measureStorage(path.join(os.tmpdir(), 'not-yet', 'memory.lino'));
  assert.ok(snapshot.capacity_bytes > 0 && snapshot.free_bytes <= snapshot.capacity_bytes);
});

test('duplicate recomputable records keep the most-used copy and leave a reconstruction record', () => {
  const file = tempMemory('duplicates');
  persistAutoFreeSpaceChoice(file, true);
  const payload = 'same cache payload'.repeat(20);
  const events = [
    recomputableEvent('cache-low-use', payload),
    recomputableEvent('cache-high-use', payload),
    memoryEvent({
      id: 'raw-user-message',
      kind: 'message',
      role: 'user',
      content: 'raw event stays',
      evidence: ['source:http:cache-high-use'],
      write_count: 1,
    }),
  ];
  const freed = applyAutoFreeSpaceWithSnapshot(events, file, 0, { capacity_bytes: 100, free_bytes: 100 }, noReplay);
  const removal = freed.plan.actions.find((action) => action.event_id === 'cache-low-use');
  assert.equal(removal.kind, 'RemoveDuplicateRecomputable');
  assert.equal(removal.reason, 'duplicate recomputable event; retained cache-high-use with usage 2');
  assert.equal(freed.outcome.removed_events, 1);

  // The reconstruction id hashes the Rust `{:?}` rendering of the record.
  const debug = 'MemoryEvent { id: "", kind: Some("cache_reconstruction"), role: Some("system"), intent: None, '
    + 'tool: Some("web_search"), inputs: Some("cache-low-use"), outputs: None, content: None, sent_at: None, '
    + 'demo_label: None, conversation_id: None, conversation_title: None, '
    + 'evidence: ["rediscover:https://doc.rust-lang.org/std/"], unknown_fields: [], access_count: 0, write_count: 0 }';
  const reconstruction = stableId('cache_reconstruction', debug);
  assert.equal(reconstructionRecord(events[0]).id, reconstruction);
  assert.deepEqual(ids(freed.events), ['cache-high-use', 'raw-user-message', reconstruction]);
  const record = freed.events[2];
  assert.equal(record.kind, 'cache_reconstruction');
  assert.equal(record.inputs, 'cache-low-use');
  assert.equal(record.write_count, 1);
});

test('pressure eviction stops at the target and never touches raw experience', () => {
  const file = tempMemory('pressure');
  const events = [0, 1, 2, 3, 4].map((index) => recomputableEvent(`cache-${index}`, `distinct cached payload ${index} ${'x'.repeat(600)}`));
  events.push(memoryEvent({ kind: 'message', role: 'user', content: 'irreplaceable raw question', write_count: 1 }));
  events.push(memoryEvent({ kind: 'message', role: 'assistant', content: 'irreplaceable raw answer', write_count: 1 }));
  const snapshot = { capacity_bytes: 100000, free_bytes: 20500 };
  assert.equal(applyAutoFreeSpaceWithSnapshot(events, file, 1000, snapshot, noReplay), null);
  persistAutoFreeSpaceChoice(file, true);
  const freed = applyAutoFreeSpaceWithSnapshot(events, file, 1000, snapshot, noReplay);
  assert.equal(freed.plan.required_reclaim_bytes, 500);
  assert.ok(freed.plan.selected_reclaim_bytes >= 500);
  assert.deepEqual(freed.plan.actions.map((action) => [action.kind, action.event_id]), [['EvictLowUseRecomputable', 'cache-0']]);
  assert.equal(freed.outcome.removed_events, 1);
  assert.ok(freed.events.some((event) => event.content === 'irreplaceable raw question'));
  assert.ok(freed.events.some((event) => event.content === 'irreplaceable raw answer'));
});

test('the planner keeps raw messages and learning out of every action', () => {
  const plan = planMemoryDreaming([
    memoryEvent({ id: 'raw-user-message', kind: 'message', role: 'user', content: 'irreplaceable user experience '.repeat(20) }),
    memoryEvent({ id: 'learned-skill', kind: 'learning_ledger', content: 'promoted learned experience '.repeat(20) }),
    recomputableEvent('external-cache', 'cached public source '.repeat(20)),
  ], dreamingConfig({ storage_capacity_bytes: 1000, free_bytes: 0 }), noReplay);
  assert.ok(plan.required_reclaim_bytes > 0);
  assert.deepEqual(plan.actions.map((action) => action.event_id), ['external-cache']);
});

test('the Debug rendering escapes like Rust `str`', () => {
  assert.equal(debugStr('a"b\\c\n\t\r\0­́ é\'x'), '"a\\"b\\\\c\\n\\t\\r\\0\\u{ad}\\u{301} é\'x"');
  assert.ok(memoryEventDebug(memoryEvent({ id: 'x', unknown_fields: [['k', 'v']] })).includes('unknown_fields: [("k", "v")]'));
});

/** rust/tests/unit/memory_maintenance.rs `requirement_event`. */
function requirementEvent(id, topic, statement) {
  return memoryEvent({ id, kind: 'message', role: 'user', content: statement, conversation_title: topic, write_count: 1 });
}

/** rust/tests/unit/memory_maintenance.rs `verified_task_run_event`: the output is the production replay. */
function verifiedTaskRunEvent(id, topic, input, requirement) {
  const outputs = replayAnswerWithAmendments(input, [{ id: `${id}-amendment`, topic, rule: requirement }]);
  return memoryEvent({
    id, kind: 'test_run', role: 'derived', content: input, inputs: input, outputs, conversation_title: topic, write_count: 1,
  });
}

const LATEX_RULE = 'Always compile proofs with LaTeX.';

test('consented freeing bakes the learned amendment and forgets the covered specifics first', () => {
  const file = tempMemory('amendment');
  persistAutoFreeSpaceChoice(file, true);
  const events = [
    requirementEvent('req-1', 'latex', LATEX_RULE),
    verifiedTaskRunEvent('run-1', 'latex', 'latex proof render pass 1', LATEX_RULE),
    verifiedTaskRunEvent('run-2', 'latex', 'latex proof render pass 2', LATEX_RULE),
    recomputableEvent('unrelated-cache', 'unrelated cached source '.repeat(5)),
  ];
  const replay = memoizedReplay();
  const freed = applyAutoFreeSpaceWithSnapshot(events, file, 0, { capacity_bytes: 1000, free_bytes: 0 }, replay);
  const amendment = freed.plan.amendments.find((entry) => entry.topic === 'latex');
  assert.equal(amendment.id, stableId('amendment', 'latex'));
  assert.equal(amendment.rule, LATEX_RULE);
  assert.deepEqual(amendment.source_requirement_ids, ['req-1']);
  assert.deepEqual(amendment.covered_event_ids, ['run-1', 'run-2']);
  assert.deepEqual(freed.plan.candidate_tasks.map((task) => [task.source_event_id, task.passed]), [['run-1', true], ['run-2', true]]);
  assert.deepEqual(freed.plan.patterns.map((pattern) => [pattern.structure, pattern.occurrences]), [['latex proof render pass #', 2]]);
  assert.deepEqual(freed.plan.synthesized_tasks, []);
  // Equal use and durability: the larger covered specific goes first, and one
  // covers the 200-byte deficit.
  const [first, second] = freed.plan.observations.slice(1, 3);
  const larger = first.estimated_bytes >= second.estimated_bytes ? first : second;
  assert.deepEqual(freed.plan.actions.map((action) => [action.kind, action.event_id]),
    [['ForgetCoveredSpecific', larger.event_id]]);
  assert.ok(freed.events.some((event) => event.id === 'unrelated-cache'));
  assert.ok(!freed.plan.actions.some((action) => action.event_id === 'req-1'));
  assert.equal(freed.outcome.learned_amendments, 1);
  assert.equal(freed.outcome.learned_patterns, 1);
  const baked = freed.events.find((event) => event.kind === 'meta_algorithm_amendment');
  assert.equal(baked.inputs, 'topic=latex');
  assert.equal(baked.outputs, `rule=${LATEX_RULE}`);
  assert.deepEqual(baked.evidence, ['req-1', 'recipe:data/meta/dreaming-recipe.lino']);
  const pattern = freed.events.find((event) => event.kind === 'dreaming_pattern');
  assert.equal(pattern.id, stableId('dreaming_pattern', 'latex\u0000latex proof render pass #'));
  assert.equal(pattern.content, 'Recurring structure latex proof render pass # observed 2 time(s)');

  // Re-applying against the mutated store does not append a second amendment.
  const again = applyAutoFreeSpaceWithSnapshot(freed.events, file, 0, { capacity_bytes: 100, free_bytes: 100 }, replay);
  assert.equal(again.outcome.learned_amendments, 0);
  assert.equal(again.events.filter((event) => event.kind === 'meta_algorithm_amendment').length, 1);
});

test('a repeated tool procedure becomes a held-out-validated algorithm proposal', () => {
  const file = tempMemory('algorithm');
  persistAutoFreeSpaceChoice(file, true);
  const events = ['a', 'b', 'c'].flatMap((name) => [
    memoryEvent({ id: `read-${name}`, kind: 'tool_call', role: 'tool', tool: 'read_file', inputs: `{"path":"${name}.rs"}`, conversation_id: `conv-${name}`, write_count: 1 }),
    memoryEvent({ id: `edit-${name}`, kind: 'tool_call', role: 'tool', tool: 'edit_file', inputs: `path=${name}.rs`, conversation_id: `conv-${name}`, write_count: 1 }),
  ]);
  const freed = applyAutoFreeSpaceWithSnapshot(events, file, 0, { capacity_bytes: 100, free_bytes: 100 }, noReplay);
  assert.equal(freed.plan.algorithm_candidates.length, 1);
  const candidate = freed.plan.algorithm_candidates[0];
  assert.equal(candidate.associative_root, 8);
  const steps = '9:operation9:read_file8:argument4:path9:parameter11:parameter_1'
    + '9:operation9:edit_file8:argument4:path9:parameter11:parameter_1';
  assert.equal(candidate.id, stableId('algorithm', steps));
  const evidence = `${steps}16:associative_root1:87:support6:conv-a7:support6:conv-b`
    + '14:held_out_trace6:conv-c10:start_step1:06:passed4:true';
  assert.equal(candidate.evidence_id, stableId('algorithm_evidence', evidence));
  assert.equal(freed.outcome.learned_algorithm_candidates, 1);
  const stored = freed.events.at(-1);
  assert.equal(stored.id, candidate.evidence_id);
  assert.equal(stored.kind, 'algorithm_learning_candidate');
  assert.equal(stored.inputs, 'steps=2');
  assert.deepEqual(stored.evidence, ['conv-a', 'conv-b', 'conv-c']);
  assert.ok(stored.content.startsWith(`algorithm_candidate "${candidate.id}"\n  evidence_id "${candidate.evidence_id}"\n`));
  assert.ok(stored.content.includes('  associative_root "8"\n'));
});
