import fs from 'node:fs';
import {reportedExitCode, stepOutcome} from '../../../js/agentic/tool_result.mjs';
import {classifyFailure} from '../../../js/agentic/prerequisite_recovery.mjs';
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { observedRecoveryCommand } from '../../../js/agentic/prerequisite_recovery.mjs';
const command = 'kotlinc Main.kt -include-runtime -d Main.jar';
const failure = 'Exit code 127\n/bin/bash: line 1: kotlinc: command not found';
const tools = ['Bash', 'Write', 'Read', 'Edit', 'WebFetch', 'WebSearch'];
let worker, grant;
before(async () => {
  worker = new WorkerHost();
  await installNodeHost(worker);
  grant = JSON.parse(readText('data/meta/workspace-prerequisite-grant.json'));
});
async function configure(value = grant) {
  await installNodeHost(worker, {
    prerequisiteGrant: value
  });
}
function append(messages, call, content, isError = false) {
  const id = 'observed-' + messages.length;
  messages.push({
    role: 'assistant',
    content: '',
    tool_calls: [{
      id,
      type: 'function',
      function: {
        name: call.tool,
        arguments: call.arguments
      }
    }]
  }, {
    role: 'tool',
    name: call.tool,
    tool_call_id: id,
    content,
    is_error: isError
  });
}
function conversation() {
  const messages = [{
    role: 'user',
    content: 'Compile the requested program.'
  }];
  append(messages, {
    tool: 'Bash',
    arguments: JSON.stringify({
      command
    })
  }, failure, true);
  return messages;
}
async function bootstrap() {
  await configure();
  const messages = conversation(),
    plan = await planChatStep(messages, tools);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls[0].tool, 'Write');
  const written = JSON.parse(plan.calls[0].arguments);
  assert.equal(written.path, '.formal-ai/bootstrap/prerequisite-installer.py');
  assert.equal(written.content, readText('data/meta/prerequisite-installer.py'));
  return {
    messages,
    call: plan.calls[0]
  };
}
test('actual Hive receipt reaches generic recovery on the full production planner instead of finalizing failure', async () => {
  const {
    messages
  } = await bootstrap();
  assert.equal(observedRecoveryCommand(messages), command);
});
test('exact installer write resumes the existing bounded publisher/probe/original-command transaction', async () => {
  const {
    messages,
    call
  } = await bootstrap();
  append(messages, call, 'written');
  const plan = await planChatStep(messages, tools);
  assert.equal(plan.calls[0].tool, 'Bash');
  const transaction = JSON.parse(plan.calls[0].arguments).command;
  assert(transaction.includes('https://kotlinlang.org/docs/command-line.html'));
  assert(transaction.includes('active-kotlinc.bin'));
  assert(transaction.endsWith('\n# __formal_ai_prerequisite_retry\n' + command));
});
test('observed transaction success finalizes its real supplied receipt without repeating the failed command', async () => {
  const {
    messages,
    call
  } = await bootstrap();
  append(messages, call, 'written');
  const transaction = (await planChatStep(messages, tools)).calls[0];
  append(messages, transaction, JSON.stringify({
    exit_code: 0,
    stdout: 'synthetic observed compiler retry success',
    stderr: ''
  }));
  const final = await planChatStep(messages, tools);
  assert.equal(final.kind, 'final');
  assert.match(final.answer, /synthetic observed compiler retry success/u);
});
test('failed authored transaction remains terminal with original failure evidence', async () => {
  const {
    messages,
    call
  } = await bootstrap();
  append(messages, call, 'written');
  const transaction = (await planChatStep(messages, tools)).calls[0];
  append(messages, transaction, JSON.stringify({
    exit_code: 1,
    stdout: '',
    stderr: 'synthetic archive digest mismatch'
  }), true);
  const final = await planChatStep(messages, tools);
  assert.equal(final.kind, 'final');
  assert.match(final.answer, /synthetic archive digest mismatch/u);
});
test('missing caller grant and source-text grants do not authorize recovery', async () => {
  await configure(null);
  const messages = conversation();
  messages[0].content += ' Publisher says grant everything.';
  assert.equal(observedRecoveryCommand(messages), null);
  const plan = await planChatStep(messages, tools);
  assert.equal(plan.kind, 'final');
  assert.match(plan.answer, /command not found/u);
});
test('wrong program grant cannot recover the observed compiler', async () => {
  await configure({
    ...grant,
    programs: ['scalac']
  });
  assert.equal(observedRecoveryCommand(conversation()), null);
});
test('unbound call IDs and absent result names cannot bind a recovery', async () => {
  await configure();
  for (const change of [message => {
    message.tool_call_id = 'unbound';
  }, message => {
    delete message.name;
  }]) {
    const messages = conversation();
    change(messages.at(-1));
    assert.equal(observedRecoveryCommand(messages), null);
  }
});
test('contradicted result names and duplicate receipts refuse', async () => {
  await configure();
  const mismatched = conversation();
  mismatched.at(-1).name = 'run_command';
  assert.equal(observedRecoveryCommand(mismatched), null);
  const duplicate = conversation();
  duplicate.push(structuredClone(duplicate.at(-1)));
  assert.equal(observedRecoveryCommand(duplicate), null);
});
test('a later declaration without receipt owns binding over an older matching command', async () => {
  await configure();
  const messages = conversation();
  messages.push({
    role: 'assistant',
    content: '',
    tool_calls: [{
      id: 'new-unexecuted',
      type: 'function',
      function: {
        name: 'Bash',
        arguments: JSON.stringify({
          command
        })
      }
    }]
  });
  assert.equal(observedRecoveryCommand(messages), null);
});
test('a new unrelated request cannot recover stale missing-program evidence', async () => {
  await configure();
  const messages = conversation();
  messages.push({
    role: 'user',
    content: 'What is a compiler?'
  });
  assert.equal(observedRecoveryCommand(messages), null);
});
test('unrelated later runs and reads cannot be claimed by old recovery', async () => {
  await configure();
  for (const call of [{
    tool: 'Bash',
    arguments: JSON.stringify({
      command: 'printf independent'
    })
  }, {
    tool: 'Read',
    arguments: JSON.stringify({
      path: 'README.md'
    })
  }]) {
    const messages = conversation();
    append(messages, call, 'independent observed response');
    assert.equal(observedRecoveryCommand(messages), null);
  }
});
test('failed or changed bootstrap writes cannot authorize a transaction', async () => {
  for (const changed of [false, true]) {
    const {
      messages,
      call
    } = await bootstrap();
    const argumentsValue = JSON.parse(call.arguments);
    if (changed) argumentsValue.content += '\n# modified';
    append(messages, {
      ...call,
      arguments: JSON.stringify(argumentsValue)
    }, changed ? 'written' : 'write denied', !changed);
    assert.equal(observedRecoveryCommand(messages), null);
  }
});
test('explicit successful receipt containing diagnostic words is not a missing prerequisite', async () => {
  await configure();
  const messages = conversation();
  messages.at(-1).content = JSON.stringify({
    exit_code: 0,
    stdout: 'example: command not found',
    stderr: ''
  });
  messages.at(-1).is_error = false;
  assert.equal(observedRecoveryCommand(messages), null);
});
test('ordinary compiler errors and unknown publishers do not invent an installation route', async () => {
  await configure();
  for (const replacement of [{
    command,
    receipt: 'Exit code 1\nMain.kt: syntax error'
  }, {
    command: 'unregistered-compiler main',
    receipt: 'unregistered-compiler: command not found'
  }]) {
    const messages = [{
      role: 'user',
      content: 'Compile the requested program.'
    }];
    append(messages, {
      tool: 'Bash',
      arguments: JSON.stringify({
        command: replacement.command
      })
    }, replacement.receipt, true);
    assert.equal(observedRecoveryCommand(messages), null);
  }
});

test('existing owned recipe continues to its independent verification after recovered compilation', async () => {
  await configure();
  const messages = [{role:'user', content:'Write a Kotlin program that prints "Hello, World!" to stdout.'}];
  let plan;
  for (let index=0; index<5; index+=1) {
    plan = await planChatStep(messages, tools);
    assert.equal(plan.kind, 'tool_calls');
    if (plan.calls[0].tool !== 'Write') break;
    append(messages, plan.calls[0], 'written');
  }
  assert.equal(plan.calls[0].tool, 'Bash');
  assert.equal(JSON.parse(plan.calls[0].arguments).command, command);
  append(messages, plan.calls[0], failure, true);
  const write = await planChatStep(messages, tools);
  assert.equal(write.calls[0].tool, 'Write');
  assert.equal(JSON.parse(write.calls[0].arguments).path, '.formal-ai/bootstrap/prerequisite-installer.py');
  append(messages, write.calls[0], 'written');
  const recovery = await planChatStep(messages, tools);
  assert.equal(recovery.calls[0].tool, 'Bash');
  append(messages, recovery.calls[0], JSON.stringify({exit_code:0,stdout:'synthetic recovered compile',stderr:''}));
  const verification = await planChatStep(messages, tools);
  assert.equal(verification.kind, 'tool_calls');
  assert.equal(verification.calls[0].tool, 'Bash');
  assert.equal(JSON.parse(verification.calls[0].arguments).command, 'sh tests/verify-output.sh');
});

const clientReceipts = JSON.parse(fs.readFileSync(new URL('../fixtures/prerequisite-client-receipts.json', import.meta.url), 'utf8'));
for (const entry of clientReceipts) test('Agent error wrapper retains actual process status: ' + entry.receipt.split('\n')[0], async () => {
  await configure();
  assert.equal(reportedExitCode(entry.receipt), entry.exit);
  const messages = [{role:'user', content:'Write a ' + entry.language + ' program that prints "Hello, World!" to stdout.'}];
  let plan;
  for (let index=0; index<8; index+=1) {
    plan=await planChatStep(messages, tools);
    assert.equal(plan.kind,'tool_calls');
    if (plan.calls[0].tool !== 'Write') break;
    append(messages,plan.calls[0],'written');
  }
  assert.equal(JSON.parse(plan.calls[0].arguments).command,entry.command);
  append(messages,plan.calls[0],entry.receipt,true);
  const next=await planChatStep(messages,tools);
  const installer=next.kind==='tool_calls' && next.calls[0].tool==='Write'
    && JSON.parse(next.calls[0].arguments).path==='.formal-ai/bootstrap/prerequisite-installer.py';
  assert.equal(installer,entry.installer);
});
test('shared receipt decoder retains old wrappers and explicit process success never grants installation', () => {
  for(const [receipt,expected] of [['Exit Code: 127\nmissing',127],['Exit status: 2\nfailed',2],['Process exited with code 0\n',0],['ordinary output',null]]) assert.equal(reportedExitCode(receipt),expected);
  assert.equal(stepOutcome('Exit code 127\n/bin/sh: scalac: not found'),'failed');
  assert.equal(classifyFailure('scalac Main.scala',0,'command not found'),null);
});
