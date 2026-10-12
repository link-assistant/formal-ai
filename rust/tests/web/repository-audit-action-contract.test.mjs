import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { hasHost } from '../../../js/agentic/host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { ownedStatementAuditOutput } from '../../../js/agentic/statement_audit.mjs';
before(async () => {
  if (!hasHost()) await installNodeHost(new WorkerHost());
});
const task = 'Audit all statement-bearing repository prose, code comments, and structured facts; weigh conflicting requirements and captured original-source evidence with probabilities; persist findings and associations; and write statement-audit.lino.';
const user = content => ({
  role: 'user',
  content
});
const command = 'formal-ai statement-audit --root . --output statement-audit.lino';
test('the original repository audit delegates the public command without literal authoring', async () => {
  const plan = await planChatStepResolved([user(task)], ['bash']);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls.length, 1);
  assert.equal(plan.calls[0].tool, 'bash');
  assert.equal(JSON.parse(plan.calls[0].arguments).command, command);
});
test('the original command result is consumed before completion without rewriting its artifact', async () => {
  const messages = [user(task), {
    role: 'assistant',
    content: '',
    tool_calls: [{
      id: 'audit_1',
      type: 'function',
      function: {
        name: 'bash',
        arguments: JSON.stringify({
          command
        })
      }
    }]
  }, {
    role: 'tool',
    tool_call_id: 'audit_1',
    name: 'bash',
    content: '{"statement_audit":{"contradictions":1,"findings":2,"output":"statement-audit.lino","root":".","statements":8}}'
  }];
  for (const tools of [['bash'], ['bash', 'write'], ['bash', 'write', 'read', 'edit', 'grep']]) {
    const plan = await planChatStepResolved(messages, tools);
    assert.equal(plan.kind, 'final');
    assert.match(plan.answer, /statement_audit/);
  }
});
test('ordinary statement questions do not scan a repository', async () => {
  const plan = await planChatStepResolved([user('Is this statement true?')], ['bash']);
  assert.ok(plan === null || plan.kind !== 'tool_calls' || plan.calls.every(call => !call.arguments.includes('statement-audit')));
});
test('role variants retain a renamed case-preserved output and complete source frame', async () => {
  const request = 'Audit repository statements; weigh requirements and evidence with probabilities; persist audit findings and associations; write Review.lino.';
  assert.equal(ownedStatementAuditOutput(request), 'Review.lino');
  const plan = await planChatStepResolved([user(request)], ['bash', 'write']);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(JSON.parse(plan.calls[0].arguments).command, 'formal-ai statement-audit --root . --output Review.lino');
});
test('unknown, conflicting and prerequisite Needs never acquire an audit action', async () => {
  for (const request of [task + ' Deploy the service.', 'Do not write. ' + task, 'Do not read. ' + task, task + ' Read input.txt before authoring.', task + ' Write other.lino.', task.replace('statement-audit.lino', '../outside.lino')]) {
    assert.equal(ownedStatementAuditOutput(request), null);
    const plan = await planChatStepResolved([user(request)], ['bash', 'read', 'write']);
    assert.ok(plan === null || plan.kind !== 'tool_calls', request);
  }
});
test('quoted audit-looking bytes remain literal payload and do not own the CLI', async () => {
  const request = 'Create file example.txt containing "' + task + '".';
  assert.equal(ownedStatementAuditOutput(request), null);
  const plan = await planChatStepResolved([user(request)], ['bash', 'write', 'read']);
  assert.ok(plan === null || plan.kind !== 'tool_calls' || plan.calls.every(call => !call.arguments.includes('formal-ai statement-audit')));
});
test('the original external evidence audit remains delegated to the named evidence file', async () => {
  const request = 'Fact-check every repository statement. Use evidence.json as the external evidence capture file and preserve statement-audit.lino.';
  const plan = await planChatStepResolved([user(request)], ['bash']);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(JSON.parse(plan.calls[0].arguments).command, 'formal-ai statement-audit --root . --evidence evidence.json --output statement-audit.lino');
});
test('unresolved delivery and foreign output never become a legacy default audit', async () => {
  for (const request of [task.replace('statement-audit.lino', '../outside.lino') + ' Deploy the service.', task.replace('and write statement-audit.lino.', 'write /tmp/outside.lino.'), task.replace('and write statement-audit.lino.', 'write report.json.')]) {
    assert.equal(ownedStatementAuditOutput(request), null);
    const plan = await planChatStepResolved([user(request)], ['bash', 'read', 'write']);
    assert.ok(plan === null || plan.kind !== 'tool_calls', request);
  }
});
