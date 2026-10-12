import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { shellCommandForTask } from '../../../js/agentic/shell_command.mjs';
import { negatedCommandPolicy } from '../../../js/agentic/shell-command-negative-ownership.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
test('a seeded leading negation retains coordinated command scope', () => {
  for (const raw of [
    'Do not edit the repository, run node report.mjs or download artifacts.',
    'Never change files, run git status.',
    "Don't alter notes, run node --version.",
    'Никогда изменяй файлы, run node report.mjs.',
    'Nunca cambies archivos, run node report.mjs.',
  ]) assert.equal(shellCommandForTask(raw), null, raw);
});
test('command operands and quoted source retain their ordinary meaning', () => {
  for (const [raw, expected] of [
    ['Run node never.mjs.', 'node never.mjs'],
    ['Run node --version.', 'node --version'],
    ["Run node -e 'console.log(\"do not run cargo\")'", "node -e 'console.log(\"do not run cargo\")'"],
  ]) assert.equal(shellCommandForTask(raw), expected, raw);
});
test('separate requests and conditional consequents retain existing boundaries', () => {
  assert.equal(shellCommandForTask('Do not edit notes. Run node --version.'), 'node --version');
  assert.equal(shellCommandForTask('When finished, run node report.mjs.'), 'node report.mjs');
});
test('caller objects cannot supply a negation ownership surface', () => {
  let traps = 0;
  const fake = new Proxy({}, { get() { traps++; throw Error('UnknownCallerSurface'); } });
  assert.equal(negatedCommandPolicy(fake), false);
  assert.equal(traps, 0);
});
