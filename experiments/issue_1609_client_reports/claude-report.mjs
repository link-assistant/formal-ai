import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createInterface } from 'node:readline';

const [command, ...prefix] = process.argv.slice(2);
const issue = 'https://github.com/link-assistant/formal-ai/issues/9999';
const settings = JSON.stringify({ permissions: { allow: ['Bash(formal-ai report create-issue *)'] } });
// An empty inline prompt tells the wrapper to leave the stream on stdin. The
// wrapper then emits Claude's native --print mode and forwards the JSON frames.
const print = prefix.length ? ['--print='] : ['--print', '--model', 'formal-ai'];
const child = spawn(command, [...prefix, ...print, '--verbose',
  '--input-format=stream-json', '--output-format=stream-json', `--settings=${settings}`], {
  stdio: ['pipe', 'pipe', 'inherit'],
});
const closed = once(child, 'close');
const lines = createInterface({ input: child.stdout });
const exchanges = [
  ['Hello', /Hi[!,]/u],
  ['Report', /GitHub issue/u],
  ['GitHub issue', /Both logs/u],
  ['Both logs', issue],
];
const send = text => child.stdin.write(`${JSON.stringify({ type: 'user', message: {
  role: 'user', content: [{ type: 'text', text }],
} })}\n`);

let turn = 0;
let toolUsed = false;
const deadline = setTimeout(() => child.kill(), 120_000);
try {
  send(exchanges[0][0]);
  for await (const line of lines) {
    process.stdout.write(`${line}\n`);
    const event = JSON.parse(line);
    if (event.type === 'system' && event.subtype === 'init') {
      assert.equal(event.permissionMode, 'default');
    }
    if (event.type === 'assistant' && event.message.content.some(item => item.type === 'tool_use'
      && item.name === 'Bash' && item.input.command.startsWith('formal-ai report create-issue '))) {
      toolUsed = true;
    }
    if (event.type !== 'result') continue;
    assert.equal(event.is_error, false, line);
    const expected = exchanges[turn][1];
    if (typeof expected === 'string') assert.ok(event.result.includes(expected), line);
    else assert.match(event.result, expected);
    turn++;
    if (turn === exchanges.length) {
      child.stdin.end();
    } else {
      send(exchanges[turn][0]);
    }
  }
  const [code] = await closed;
  assert.equal(code, 0);
  assert.equal(turn, exchanges.length);
  assert.ok(toolUsed, 'the client never ran the report command');
} finally {
  clearTimeout(deadline);
  if (child.exitCode === null && child.signalCode === null) child.kill();
  await closed;
}
