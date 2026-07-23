import { readdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

const [dialogDirectory, client, outputPath, expectedResult, emptyFlag] =
  process.argv.slice(2);
if (!dialogDirectory || !client || !outputPath || !expectedResult) {
  throw new Error(
    'usage: verify-dialog.mjs DIALOG_DIR CLIENT OUTPUT EXPECTED_RESULT [EMPTY]',
  );
}
const expectEmpty = emptyFlag === 'EMPTY';

const records = [];
for (const name of await readdir(dialogDirectory)) {
  if (!name.endsWith('.jsonl')) continue;
  const content = await readFile(join(dialogDirectory, name), 'utf8');
  for (const line of content.split('\n').filter(Boolean)) records.push(JSON.parse(line));
}
records.sort((left, right) => left.timestamp_unix_ms - right.timestamp_unix_ms);

const prompt = 'Find hive-mind-control center folder on my desktop';
const exchanges = records.filter((record) => record.request_body?.includes(prompt));
if (exchanges.length < 2) {
  throw new Error(`${client}: expected a complete two-turn dialog, got ${exchanges.length}`);
}

function anthropicStreamCalls(responseBody) {
  const calls = new Map();
  for (const line of responseBody.split('\n')) {
    if (!line.startsWith('data: ')) continue;
    let event;
    try {
      event = JSON.parse(line.slice(6));
    } catch {
      continue;
    }
    if (event.type === 'content_block_start' && event.content_block?.type === 'tool_use') {
      calls.set(event.index, {
        id: event.content_block.id,
        name: event.content_block.name,
        partialJson: '',
      });
    }
    const partialJson = event.delta?.partial_json;
    if (event.type === 'content_block_delta' && partialJson && calls.has(event.index)) {
      calls.get(event.index).partialJson += partialJson;
    }
  }
  return [...calls.values()].map(({ partialJson, ...call }) => ({
    ...call,
    arguments: JSON.parse(partialJson || '{}'),
  }));
}

const calls = exchanges.flatMap((record) => {
  if (record.response_tool_calls?.length) return record.response_tool_calls;
  return anthropicStreamCalls(record.response_body ?? '');
});
const localCalls = calls.filter((call) => JSON.stringify(call.arguments).includes('find '));
if (localCalls.length !== 1) {
  throw new Error(`${client}: expected exactly one find call\n${JSON.stringify(calls, null, 2)}`);
}
const command = JSON.stringify(localCalls[0].arguments);
for (const expected of ['FORMAL_AI_DESKTOP_DIR', '-type d', 'hive', 'control', 'center']) {
  if (!command.includes(expected)) throw new Error(`${client}: command omitted ${expected}: ${command}`);
}
if (calls.some((call) => call.name.toLowerCase().includes('web'))) {
  throw new Error(`${client}: local lookup emitted a web call`);
}

const resultExchange = exchanges.find(
  (record) =>
    (expectEmpty || record.request_body.includes(expectedResult)) &&
    /(role["']?\s*:\s*["']tool|tool_result|function_call_output)/u.test(record.request_body),
);
if (!resultExchange) throw new Error(`${client}: client never returned the tool result`);

const finalExchange = [...exchanges]
  .reverse()
  .find((record) => record.response_body?.includes(expectedResult));
if (!finalExchange) throw new Error(`${client}: final answer omitted the expected result`);

const sequence = [
  { role: 'user', content: prompt },
  { role: 'assistant', tool_call: localCalls[0] },
  { role: 'tool', content: expectEmpty ? '' : expectedResult },
  { role: 'assistant', content: expectedResult },
];
await writeFile(
  outputPath,
  `${JSON.stringify({ client, exchange_count: exchanges.length, sequence }, null, 2)}\n`,
);
