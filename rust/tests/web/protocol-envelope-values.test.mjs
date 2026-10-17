import test from 'node:test';
import assert from 'node:assert/strict';
import {chatMessage, optionalValue, isMap} from '../../../js/server/protocol-values.mjs';
import {parseAnthropicRequest, anthropicToChatCompletionRequest} from '../../../js/server/anthropic.mjs';
import {parseGeminiRequest, geminiToChatCompletionRequest} from '../../../js/server/gemini.mjs';
import {parseResponsesRequest, toChatCompletionRequest} from '../../../js/server/responses-input.mjs';

test('each message owns fresh mutable defaults and retains explicit field overrides', () => {
  const first = chatMessage('user', 'oak'), second = chatMessage('user', 'oak');
  assert.notStrictEqual(first, second);
  assert.notStrictEqual(first.tool_calls, second.tool_calls);
  first.tool_calls.push({id: 'kept'});
  assert.deepEqual(second.tool_calls, []);
  const calls = [{id: 'external'}];
  const overridden = chatMessage('assistant', '', {tool_calls: calls, is_error: true, name: 'chosen'});
  assert.strictEqual(overridden.tool_calls, calls);
  assert.equal(overridden.is_error, true);
  assert.equal(overridden.name, 'chosen');
});
test('spread and optional-field access retain getter order, count and thrown identity', () => {
  const events = [];
  const extra = {get name() { events.push('name'); return 'tool'; },
    get content() { events.push('content'); return 'observed'; }};
  assert.equal(chatMessage('user', 'ignored', extra).content, 'observed');
  assert.deepEqual(events, ['name', 'content']);
  let count = 0;
  assert.equal(optionalValue({get field() { count += 1; return 0; }}, 'field'), 0);
  assert.equal(count, 1);
  const failure = new Error('original getter failure');
  assert.throws(() => optionalValue({get field() { throw failure; }}, 'field'), error => error === failure);
  assert.throws(() => chatMessage('user', '', {get field() { throw failure; }}), error => error === failure);
});
test('optional values distinguish absence from false, zero and empty text', () => {
  for (const value of [false, 0, '', [], {}]) assert.strictEqual(optionalValue({field: value}, 'field'), value);
  assert.equal(optionalValue({}, 'field'), null);
  assert.equal(optionalValue({field: null}, 'field'), null);
  assert.equal(optionalValue(Object.create({field: 'inherited'}), 'field'), 'inherited');
});
test('map recognition retains objects without invoking conversion hooks', () => {
  for (const value of [null, undefined, false, 0, '', [], () => {}]) assert.equal(isMap(value), false);
  const plain = Object.create(null);
  assert.equal(isMap(plain), true);
  assert.equal(isMap(new Proxy({}, {get() { throw Error('must not inspect property'); }})), true);
  const {proxy, revoke} = Proxy.revocable({}, {}); revoke();
  assert.throws(() => isMap(proxy), TypeError);
});
test('all three protocol consumers retain system, assistant tool call and tool result messages', () => {
  const anthropic = anthropicToChatCompletionRequest(parseAnthropicRequest(JSON.stringify({
    model: 'formal-ai', system: 'system context', messages: [
      {role: 'user', content: 'question'},
      {role: 'assistant', content: [{type: 'tool_use', id: 'call-one', name: 'read', input: {path: 'one'}}]},
      {role: 'user', content: [{type: 'tool_result', tool_use_id: 'call-one', content: 'answer'}]},
    ],
  })));
  const gemini = geminiToChatCompletionRequest(parseGeminiRequest(JSON.stringify({
    systemInstruction: {parts: [{text: 'system context'}]}, contents: [
      {role: 'user', parts: [{text: 'question'}]},
      {role: 'model', parts: [{functionCall: {id: 'call-one', name: 'read', args: {path: 'one'}}}]},
      {role: 'user', parts: [{functionResponse: {id: 'call-one', name: 'read', response: 'answer'}}]},
    ],
  })), 'formal-ai');
  const responses = toChatCompletionRequest(parseResponsesRequest(JSON.stringify({
    model: 'formal-ai', instructions: 'system context', input: [
      {role: 'user', content: 'question'},
      {type: 'function_call', call_id: 'call-one', name: 'read', arguments: '{"path":"one"}'},
      {type: 'function_call_output', call_id: 'call-one', output: 'answer'},
    ],
  })));
  for (const request of [anthropic, gemini, responses]) {
    assert.equal(request.messages[0].role, 'system');
    assert.equal(request.messages[0].content, 'system context');
    assert(request.messages.some(message => message.role === 'user' && message.content === 'question'));
    assert(request.messages.some(message => message.role === 'assistant' && message.tool_calls[0]?.id === 'call-one'));
    assert(request.messages.some(message => message.role === 'tool' && message.tool_call_id === 'call-one'
      && message.content === 'answer'));
  }
});
