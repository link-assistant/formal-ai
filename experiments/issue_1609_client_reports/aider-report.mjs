import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createInterface } from 'node:readline';

const [command, ...prefix] = process.argv.slice(2);
const child = spawn(command, [
  ...prefix, '--no-git', '--no-analytics', '--no-check-update', '--no-show-release-notes',
  '--no-pretty', '--no-fancy-input', '--no-show-model-warnings',
], { env: { ...process.env, TERM: 'dumb' }, stdio: ['pipe', 'pipe', 'inherit'] });
const closed = once(child, 'close');
const lines = createInterface({ input: child.stdout });
const turns = [
  [/Hi[!,]/u, 'Report issue'],
  [/^3\. GitHub issue/u, '3'],
  [/^3\. Server log/u, '1'],
  [/^formal-ai report create-issue --session '[^']+' --source both /u, null],
  [/^https:\/\/github\.com\/link-assistant\/formal-ai\/issues\/9999$/u, 'n\n/exit'],
];

let index = 0;
const deadline = setTimeout(() => child.kill(), 120_000);
try {
  child.stdin.write('Hello\n');
  for await (const line of lines) {
    process.stdout.write(`${line}\n`);
    assert.doesNotMatch(line, /No filename provided before/u);
    const turn = turns[index];
    if (!turn || !turn[0].test(line)) continue;
    if (turn[1] === null) assert.match(line, /--title 'Formal AI: `Hello`'/u);
    child.stdin.write(`${turn[1] ?? `/run ${line.trim()}`}\n`);
    index += 1;
  }
  const [code] = await closed;
  assert.equal(code, 0);
  assert.equal(index, turns.length, 'Aider did not complete the report and show the GitHub URL');
} finally {
  clearTimeout(deadline);
  if (child.exitCode === null && child.signalCode === null) child.kill();
  await closed;
}
