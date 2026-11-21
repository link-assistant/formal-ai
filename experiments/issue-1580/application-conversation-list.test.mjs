import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import vm from 'node:vm';
import { test } from 'node:test';

const app = new URL('../../js/app/application.jsx', import.meta.url);
const hooks = new URL('../../js/app/application-conversation-hooks.jsx', import.meta.url);
const before = new URL('./fixtures/conversation-list-hook/application-before.jsx.gz', import.meta.url);
const read = url => readFileSync(url, 'utf8');
const segments = [
  ['  // Issue #27: on mount', '  const {\n    handleExportMemory'],
  ['  const handleShowDeletedConversations =', '  const { handleContextResizePointerDown'],
  ['  const handleDeleteConversation =', '  // Issue #386: copy a whole conversation'],
];
const plain = value => JSON.parse(JSON.stringify(value));

test('authored App remains below its existing source warning threshold', () => {
  assert.ok(read(app).trimEnd().split('\n').length < 1400);
});

test('the complete conversation-list bodies and comments remain unchanged', () => {
  const original = gunzipSync(readFileSync(before)).toString('utf8');
  for (const [start, end] of segments) {
    const body = original.slice(original.indexOf(start), original.indexOf(end));
    assert.ok(read(hooks).includes(body), start);
    assert.ok(!read(app).includes(body), start);
  }
  assert.match(read(app), /} = useConversationList\(\{/);
});

function mount({ events = [], unavailable = false, error = null } = {}) {
  const effects = [], recorded = [], values = {}, calls = [];
  const memory = { listEvents: async () => { if (error) throw error; return events; } };
  const context = vm.createContext({
    React: { useCallback: fn => fn, useEffect: fn => effects.push(fn) },
    window: unavailable ? undefined : { FormalAiMemory: memory },
    groupConversations: (rows, options) => {
      calls.push({ rows, options });
      return rows.filter(row => options.showDeleted || !row.deleted);
    },
    messagesForConversation: (rows, id) => rows.filter(row => row.id === id),
    recordMemoryEvent: async event => recorded.push(event),
  });
  const source = read(hooks).replace(/^import[\s\S]*?;\n/gm, '').replace(/export function /g, 'function ');
  vm.runInContext(source + '\nglobalThis.load = useConversationList;', context);
  const props = {
    initialPreferences: { current: { currentConversationId: 'current' } },
    conversationEventsRef: { current: [] }, conversationTitlesRef: { current: new Map() },
    showDeletedConversationsRef: { current: false }, currentConversationRef: { current: 'current' },
  };
  for (const key of ['Conversations', 'Messages', 'DemoMode', 'CurrentConversationId', 'Prompt', 'ShowDeletedConversations']) {
    props['set' + key] = value => { values[key] = value; };
  }
  return { api: context.load(props), props, effects, recorded, values, calls };
}

test('refresh preserves deletion selection, cached events and known titles', async () => {
  const events = [{ id: 'current', title: 'Current' }, { id: 'old', title: 'Old', deleted: true }];
  const state = mount({ events });
  assert.equal(await state.api.refreshConversations(), events);
  assert.equal(state.props.conversationEventsRef.current, events);
  assert.deepEqual(state.values.Conversations, events.slice(0, 1));
  assert.equal(state.props.conversationTitlesRef.current.get('current'), 'Current');
  state.props.showDeletedConversationsRef.current = true;
  await state.api.refreshConversations();
  assert.deepEqual(state.values.Conversations, events);
  await state.api.refreshConversations(false);
  assert.deepEqual(state.values.Conversations, events.slice(0, 1));
  state.api.handleShowDeletedConversations({ target: { checked: 1 } });
  await new Promise(setImmediate);
  assert.equal(state.values.ShowDeletedConversations, true);
  assert.equal(state.calls.at(-1).options.showDeleted, true);
});

test('unavailable and failed memory retain the empty projection contract', async () => {
  for (const options of [{ unavailable: true }, { error: new Error('unreadable memory') }]) {
    const state = mount(options);
    assert.deepEqual(plain(await state.api.refreshConversations()), []);
    assert.deepEqual(plain(state.props.conversationEventsRef.current), []);
    assert.deepEqual(state.values, {});
  }
});

test('initial hydration restores the selected thread and respects unmount', async () => {
  const events = [{ id: 'current', title: 'Current' }, { id: 'other', title: 'Other' }];
  for (const cancelled of [false, true]) {
    const state = mount({ events });
    const cleanup = state.effects[0]();
    if (cancelled) cleanup();
    await new Promise(setImmediate);
    if (cancelled) {
      assert.equal(state.values.Messages, undefined);
      assert.equal(state.values.DemoMode, undefined);
    } else {
      assert.deepEqual(state.values.Messages, events.slice(0, 1));
      assert.equal(state.values.DemoMode, false);
    }
  }
});

test('deletion records an event, clears only the selected thread and refreshes', async () => {
  for (const id of ['current', 'other']) {
    const state = mount();
    await state.api.handleDeleteConversation({ id, title: 'Saved title' });
    assert.equal(state.recorded.length, 1);
    assert.equal(state.recorded[0].kind, 'conversation_deleted');
    assert.equal(state.recorded[0].conversationId, id);
    assert.equal(state.recorded[0].conversationTitle, 'Saved title');
    assert.ok(!Number.isNaN(Date.parse(state.recorded[0].sentAt)));
    assert.equal(state.values.ShowDeletedConversations, false);
    assert.equal(state.calls.at(-1).options.showDeleted, false);
    if (id === 'current') {
      assert.equal(state.props.currentConversationRef.current, '');
      assert.equal(state.values.CurrentConversationId, '');
      assert.deepEqual(plain(state.values.Messages), []);
      assert.equal(state.values.Prompt, '');
      assert.equal(state.values.DemoMode, false);
    } else {
      assert.equal(state.props.currentConversationRef.current, 'current');
      assert.equal(state.values.Messages, undefined);
    }
    await state.api.handleDeleteConversation(null);
    await state.api.handleDeleteConversation({});
    assert.equal(state.recorded.length, 1);
  }
});
