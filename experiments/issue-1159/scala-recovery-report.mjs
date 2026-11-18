// Issue #1159: replay the Agent CLI Scala run (`scalac: not found`, exit 127)
// through the JS planner under the workspace grant, and print each planned
// step and the final answer, including the recovery report.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { readText } from '../../js/agentic/host.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';

const tools = ['bash', 'read', 'write', 'edit'];
const worker = new WorkerHost();
await installNodeHost(worker);
const grant = process.argv.includes('--no-grant') ? null : JSON.parse(readText('data/meta/workspace-prerequisite-grant.json'));
await installNodeHost(worker, { prerequisiteGrant: grant });
const messages = [{ role: 'user', content: process.env.REQUEST ?? 'Compile the requested program.' }];
let n = 0;
const record = (tool, args, result) => {
  const id = 'scala-' + n++;
  messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: tool, arguments: args } }] },
    { role: 'tool', name: tool, tool_call_id: id, content: result });
};
record('bash', JSON.stringify({ command: 'scalac Main.scala' }), 'Output: \nError: /bin/sh: 1: scalac: not found\nExit Code: 127');
for (let step = 0; step < 4; step++) {
  const plan = await planChatStep(messages, tools);
  if (plan.kind === 'final') { console.log('FINAL:\n' + plan.answer); break; }
  const call = plan.calls[0];
  console.log('CALL', call.tool, call.arguments.slice(0, 160).replace(/\n/g, '\\n'));
  record(call.tool, call.arguments, call.tool === 'write' ? '' : 'Output: \nExit Code: 0');
}
