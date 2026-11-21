import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createInterface } from 'node:readline';

const [client, command, ...prefix] = process.argv.slice(2);
assert.ok(client === 'claude' || client === 'agent');
const issue = 'https://github.com/link-assistant/formal-ai/issues/9999';
const settings = JSON.stringify({ permissions: { allow: ['Bash(formal-ai report create-issue *)'] } });
// An empty inline prompt tells the wrapper to leave the stream on stdin. The
// wrapper then emits Claude's native --print mode and forwards the JSON frames.
const flags = client === 'claude'
  ? [...(prefix.length ? ['--print='] : ['--print', '--model', 'formal-ai']), '--verbose', `--settings=${settings}`]
  : ['--compact-json', '--no-retry-on-rate-limits', '--permission-mode=auto',
    ...(prefix.length ? [] : ['--model=formalai/formal-ai', '--no-summarize-session', '--compaction-models=(same)'])];
const child = spawn(command, [...prefix, ...flags, '--input-format=stream-json', '--output-format=stream-json'], {
  stdio: ['pipe', 'pipe', 'inherit'],
});
const closed = once(child, 'close');
const lines = createInterface({ input: child.stdout });
const exchanges = [
  ['Hello', /Hi[!,]/u],
  ['Report', /GitHub issue/u],
  ['GitHub issue', /Both logs/u],
  ['Both logs', `Filed the issue on GitHub: ${issue}`],
];
const send = text => child.stdin.write(`${JSON.stringify({ type: 'user', message: {
  role: 'user', content: [{ type: 'text', text }],
} })}\n`);

let turn = 0;
let toolUsed = false;
let agentAnswer = '';
const deadline = setTimeout(() => child.kill(), 120_000);
try {
  send(exchanges[0][0]);
  for await (const line of lines) {
    if (!line.trim()) continue;
    process.stdout.write(`${line}\n`);
    const event = JSON.parse(line);
    if (event.type === 'system' && event.subtype === 'init') {
      assert.equal(event.permissionMode, 'default');
    }
    if (client === 'claude') {
      if (event.type === 'assistant' && event.message.content.some(item => item.type === 'tool_use'
        && item.name === 'Bash' && item.input.command.startsWith('formal-ai report create-issue '))) {
        toolUsed = true;
      }
      if (event.type !== 'result') continue;
      assert.equal(event.is_error, false, line);
    } else {
      if (event.type === 'result' && event.status === 'error') assert.fail(event.output);
      if (event.type === 'message' && event.role === 'assistant') {
        agentAnswer += event.content.map(item => item.text ?? '').join('');
      }
      if (event.type === 'tool_use' && event.name === 'bash' && event.input.command.startsWith('formal-ai report create-issue ')) {
        toolUsed = true;
      }
      if (event.type !== 'idle' || !agentAnswer) continue;
    }
    const expected = exchanges[turn][1];
    const answer = client === 'claude' ? event.result : agentAnswer;
    if (typeof expected === 'string') assert.ok(answer.includes(expected), answer);
    else assert.match(answer, expected);
    agentAnswer = '';
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
