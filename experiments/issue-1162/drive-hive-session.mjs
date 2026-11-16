// Drive the JS planner through one Hive Mind Hello World session with
// scripted tool results, and print each planned call. Used to learn the step
// sequence pinned by rust/tests/unit/issue_115{7,8,9}_*.rs and
// issue_1160_regular_prompt.rs.
// Usage: node experiments/issue-1162/drive-hive-session.mjs [Kotlin|Scala|Rust] [scenario]
import fs from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';

await installNodeHost(new WorkerHost());
const [language = 'Kotlin', scenario = 'plain'] = process.argv.slice(2);
const ISSUE = 'https://github.com/example/test-hello-world/issues/1';
const PR = 'https://github.com/example/test-hello-world/pull/2';
const BRANCH = 'issue-1-0123456789ab';
const prompt = `Resolve the GitHub issue at ${ISSUE} in this repository.\nKeep the solution on branch ${BRANCH}.\nUpdate the pull request at ${PR}.\n\nImplement and verify the solution before reporting completion.\nProceed.\n`;
const body = fs.readFileSync('rust/tests/fixtures/issue-1162/hello-world-kotlin-issue.md', 'utf8').replaceAll('Kotlin', language);
const tools = ['bash', 'read', 'write', 'edit', 'webfetch'];
const messages = [{ role: 'user', content: process.env.PROMPT_FILE ? fs.readFileSync(process.env.PROMPT_FILE, 'utf8') : prompt }];
function result(tool, args) {
  const command = args.command ?? '';
  if (tool === 'webfetch' || /gh issue view/u.test(command)) return body;
  if (scenario === 'missing' && /\b(scalac|kotlinc|rustc)\b/u.test(command) && !/install|cs setup|which|command -v/u.test(command)) {
    return JSON.stringify({ exit_code: 127, stdout: '', stderr: `/bin/sh: 1: ${command.match(/\b(scalac|kotlinc|rustc)\b/u)[1]}: not found` });
  }
  if (/git commit|git push/u.test(command)) return scenario === 'unchanged' ? '__formal_ai_commit_unchanged\n0123456789abcdef' : '0123456789abcdef';
  if (/verify-output|java -jar|scala |\.\/main|kotlin /u.test(command)) return 'Hello, World!\n';
  return '';
}
for (let step = 0; step < Number(process.env.STEPS ?? 30); step += 1) {
  const plan = await planChatStep(messages, tools);
  if (!plan) { console.log(`#${step} null`); break; }
  if (plan.kind === 'final') { console.log(`#${step} FINAL\n${plan.text ?? plan.content ?? JSON.stringify(plan)}`); break; }
  const call = plan.calls[0];
  const args = JSON.parse(call.arguments);
  console.log(`#${step} ${call.tool} ${args.command ?? args.path ?? args.url ?? call.arguments.slice(0, 200)}`);
  const id = `c${step}`;
  messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
  messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: result(call.tool, args) });
}
