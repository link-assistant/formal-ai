import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createInterface } from 'node:readline';
import { aiderReportTurns } from './aider-report-turns.mjs';

const [command, ...prefix] = process.argv.slice(2);
const child = spawn(command, [
  ...prefix, '--no-git', '--no-analytics', '--no-check-update', '--no-show-release-notes',
  '--no-pretty', '--no-fancy-input', '--no-show-model-warnings',
], { env: { ...process.env, TERM: 'dumb' }, stdio: ['pipe', 'pipe', 'inherit'] });
const closed = once(child, 'close');
const lines = createInterface({ input: child.stdout });
const turns = aiderReportTurns;

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
  // A deadline kill closes with code null; name the line the script still awaited.
  assert.equal(code, 0, `Aider closed with ${code} while awaiting ${turns[index]?.[0] ?? 'nothing'}`);
  assert.equal(index, turns.length, 'Aider did not complete the report and show the GitHub URL');
} finally {
  clearTimeout(deadline);
  if (child.exitCode === null && child.signalCode === null) child.kill();
  await closed;
}
