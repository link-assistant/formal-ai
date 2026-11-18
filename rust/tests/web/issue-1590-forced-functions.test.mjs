import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { startCli } from './support/structured-log-fixture.mjs';

const cases = JSON.parse(readFileSync(new URL('../../../data/parity/issue-1590-forced-functions.json', import.meta.url)));
const reviewCases = cases.filter(row => row.id.endsWith(' review'));
assert.deepEqual(reviewCases.map(row => row.id), ['English review', 'Spanish review', 'Chinese review', 'Russian review', 'Hindi review']);
const schemas = {
  sentiment: {
    type: 'object', required: ['label'], additionalProperties: false,
    properties: { label: { type: 'string', enum: ['positive', 'neutral', 'negative'] } },
  },
  extraction: {
    type: 'object', required: ['count'], additionalProperties: false,
    properties: { count: { type: 'integer', enum: [1, 2] } },
  },
};

function payload(route, row, stream = false) {
  const parameters = { ...schemas[row.schema ?? 'sentiment'] };
  if (row.unsupported) parameters[row.unsupported] = 2;
  const name = row.name ?? 'submit_label';
  const chat = route === '/v1/chat/completions';
  const description = row.description ?? (row.schema === 'extraction' ? 'Record the count' : 'Submit the sentiment label of the message');
  const spec = { name, description, parameters };
  if (!chat) spec.strict = true;
  const fn = definition => chat ? { type: 'function', function: definition } : { type: 'function', ...definition };
  const tool = row.hosted ? { type: row.hosted } : fn(spec);
  const tools = [tool];
  if (row.extra_function) {
    tools.unshift(fn({
      name: 'record_customer', parameters: {
        type: 'object', required: ['first', 'last'],
        properties: { first: { type: 'string' }, last: { type: 'string' } },
      },
    }));
  }
  if (row.extra_hosted) tools.unshift({ type: 'code_interpreter' });
  const choice = row.choice ?? (row.hosted ? { type: row.hosted } : chat ? { type: 'function', function: { name } } : { type: 'function', name });
  return {
    model: 'formal-ai', stream, tools, tool_choice: choice,
    ...(chat ? { messages: [{ role: 'user', content: row.prompt }] } : { input: row.prompt }),
  };
}

async function post(run, route, data) {
  const response = await fetch(run.url + route, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify(data), signal: AbortSignal.timeout(5000),
  });
  assert.equal(response.status, 200, response.ok ? undefined : await response.text());
  return response;
}

for (const agentMode of [false, true]) {
  test(`forced OpenAI functions over HTTP (agent mode ${agentMode})`, { timeout: 15000 }, async t => {
    const run = await startCli(t, {
      args: agentMode ? ['--agent-mode'] : [],
      env: { FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false' },
    });
    await run.ready();
    for (const route of ['/v1/chat/completions', '/v1/responses']) {
      const chat = route.endsWith('/completions');
      for (const row of cases) {
        if (row.gated && agentMode) continue;
        const body = await (await post(run, route, payload(route, row))).json();
        const context = `${route} / ${row.id}`;
        const choice = body.choices?.[0];
        const output = body.output?.[0];
        if (row.expected) {
          assert.equal(chat ? choice.finish_reason : output.type, chat ? 'tool_calls' : 'function_call', context);
          assert.equal(chat ? choice.message.tool_calls.length : body.output.length, 1, context);
          if (chat) assert.ok(!choice.message.content, context);
          const call = chat ? choice.message.tool_calls[0].function : output;
          assert.equal(call.name, row.name ?? 'submit_label', context);
          assert.deepEqual(JSON.parse(call.arguments), row.expected, context);
        } else if (row.gated) {
          assert.match(chat ? choice.message.content : output.content[0].text, /without explicit agent mode/u, context);
          assert.ok(!choice?.message.tool_calls?.length, context);
          if (!chat) assert.ok(body.output.every(item => item.type !== 'function_call'), context);
        } else {
          assert.equal(chat ? choice.finish_reason : output.content[0].type, chat ? 'stop' : 'refusal', context);
          assert.match(chat ? choice.message.refusal : output.content[0].refusal, /Could not derive arguments/u, context);
          assert.ok(!(chat ? choice.message.content : output.content[0].text), context);
        }
      }
      const data = await (await post(run, route, payload(route, cases[0], true))).text();
      const events = data.split('\n').filter(line => line.startsWith('data: {')).map(line => JSON.parse(line.slice(6)));
      if (chat) {
        assert.equal(events.find(event => event.choices[0].finish_reason)?.choices[0].finish_reason, 'tool_calls');
        assert.equal(events.flatMap(event => event.choices[0].delta.tool_calls ?? []).length, 1);
      } else {
        assert.equal(events.filter(event => event.type === 'response.function_call_arguments.done').length, 1);
        assert.equal(events.at(-1).response.output[0].type, 'function_call');
      }
      const refusal = await (await post(run, route, payload(route, cases.find(row => row.id === 'unknown'), true))).text();
      assert.match(refusal, chat ? /"refusal":/u : /event: response\.refusal\.done/u);
    }
  });
}
