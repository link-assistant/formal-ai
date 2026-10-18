import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { isProseWord } from '../../../js/agentic/shell_command_policy.mjs';
import { shellCommandForTask } from '../../../js/agentic/shell_command.mjs';
import { terminalCommandVocabulary } from '../../../js/agentic/crate/seed_terminal_commands.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });

test('option prefixes preserve every seeded prose spelling as command data', () => {
  const words = terminalCommandVocabulary().prose_words;
  assert.ok(words.length > 0);
  for (const word of words) {
    assert.equal(isProseWord(word), true, word);
    assert.equal(isProseWord('-' + word), false, word);
    assert.equal(isProseWord('--' + word), false, word);
    assert.equal(isProseWord(',' + word + ','), true, word);
  }
  assert.equal(isProseWord("-la'"), false);
  assert.equal(isProseWord("'la'"), true);
});

test('original and held-out bare or quoted options retain the exact full command', () => {
  for (const command of ["bash -c 'ls -la'", 'ls -la', 'ls -in', 'ls --all', "bash -c 'printf --all'"]) {
    assert.equal(shellCommandForTask(command), command, command);
  }
  assert.equal(shellCommandForTask('bash all'), null);
  assert.equal(shellCommandForTask('git the'), null);
});
