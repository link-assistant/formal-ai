// Replays a Hive-Mind-style work-item request whose fetched body is an
// attempted English issue and prints the first line of the final answer.
// Usage: node experiments/issue-1557-replay.mjs <repo-root> <fixture.json> <issue-number>
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { pathToFileURL } from 'node:url';

const [root, fixturePath, number] = process.argv.slice(2);
const load = (path) => import(pathToFileURL(resolve(root, path)).href);
const { WorkerHost } = await load('js/server/worker-host.mjs');
const { installNodeHost } = await load('js/agentic/node-host.mjs');
const { planChatStep } = await load('js/agentic/planner.mjs');
const { executeResult } = await load('experiments/js_dogfood/drive.mjs');
await installNodeHost(new WorkerHost());
const { body } = JSON.parse(readFileSync(fixturePath, 'utf8')).attempted.find((item) => item.number === Number(number));
const url = 'https://github.com/link-assistant/formal-ai/issues/904';
const messages = [{ role: 'user', content: `${url}\n\nProceed.` }];
const directory = mkdtempSync(join(tmpdir(), 'formal-ai-issue-1557-'));
try {
  for (let turn = 0; turn < 12; turn++) {
    const step = await planChatStep(messages, ['fetch_url', 'read', 'write']);
    if (step.kind === 'final') {
      console.log(`${number}: ${step.answer.split('\n')[0]}`);
      break;
    }
    for (const call of step.calls) {
      const id = `replay-${messages.length}`;
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
      const receipt = call.tool === 'fetch_url' ? { content: body, is_error: false } : executeResult(directory, call);
      messages.push({ role: 'tool', tool_call_id: id, name: call.tool, ...receipt });
    }
  }
} finally {
  rmSync(directory, { recursive: true, force: true });
}
