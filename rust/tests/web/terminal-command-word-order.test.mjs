import fs from 'node:fs';
import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {shellCommandForTask} from '../../../js/agentic/shell_command.mjs';
import {namedShellCommandInSentence} from '../../../js/agentic/shell_command_policy.mjs';
import {terminalCommandVocabulary} from '../../../js/agentic/crate/seed_terminal_commands.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const fixtures = JSON.parse(fs.readFileSync(new URL('../fixtures/terminal-command-word-order.json', import.meta.url)));
test('declared command word order preserves commands in all five languages', async () => {
  assert.deepEqual([...new Set(fixtures.cases.map(value => value.language))].sort(), ['en','es','hi','ru','zh']);
  for (const value of fixtures.cases) {
    assert.equal(shellCommandForTask(value.prompt), value.command, value.prompt);
    const plan = await planChatStep([{role:'user',content:value.prompt}], ['exec_command']);
    assert.equal(plan.calls.length, 1, value.prompt);
    assert.equal(JSON.parse(plan.calls[0].arguments).command, value.command, value.prompt);
  }
});
test('the original successful empty listing ends with its Hindi explanation', async () => {
  const messages = [{role:'user',content:'ls चलाएँ'}, {role:'assistant',content:'',
    tool_calls:[{id:'call_750',type:'function',function:{name:'exec_command',arguments:'{"cmd":"ls"}'}}]},
    {role:'tool',tool_call_id:'call_750',name:'exec_command',content:'{"output":"","exit_code":0}'}];
  const plan = await planChatStep(messages, ['exec_command']);
  assert.equal(plan.kind, 'final');
  assert.equal(plan.answer, 'यह फ़ोल्डर खाली है।');
});
test('explicit and quoted arguments keep execution-verb text as literal data', () => {
  for (const prompt of ['Run ls चलाएँ', "ls 'चलाएँ'", 'ls run']) {
    assert.equal(shellCommandForTask(prompt), prompt.startsWith('Run ') ? prompt.slice(4) : prompt);
  }
});
test('postposition follows caller grammar metadata rather than a language or task branch', () => {
  const vocab = terminalCommandVocabulary();
  assert.equal(namedShellCommandInSentence('pwd performLater', {...vocab,
    run_verbs:[...vocab.run_verbs,'performLater'],postposed_run_verbs:['performLater']}), 'pwd');
  assert.equal(namedShellCommandInSentence('pwd performLater', {...vocab,
    run_verbs:[...vocab.run_verbs,'performLater'],postposed_run_verbs:[]}), null);
});
