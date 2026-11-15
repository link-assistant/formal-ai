// Issue #1558: drive a recorded Agent CLI session through the server's agentic
// entry and answer its read, glob and `find` calls from this checkout, the way
// the Agent CLI tools answer them (link-assistant/agent js/src/tool/read.ts and
// glob.ts). The request prefix is the one recorded in the session of PR #1493:
// the Agent CLI system prompts, the solve prompt and the issue fetch, followed
// by the recorded issue text of the work item under test.
import { readFileSync, statSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import path from 'node:path';

const FIXTURES = 'data/benchmarks/fixtures/issue-1558';
const RECORDED_WORKSPACE = /\/tmp\/gh-issue-solver-1793442214945/gu;
export const ISSUES = [1484, 1485, 1487, 1488, 1507];

/** The request body of the recorded session, rewritten to `issue` and `workspace`, ending with the issue text. */
export function recordedRequest(issue, workspace, { issueText, repository = 'link-assistant/formal-ai', tools = null } = {}) {
  const body = JSON.parse(readFileSync(`${FIXTURES}/agent-cli-request.json`, 'utf8'));
  const text = issueText ?? readFileSync(`${FIXTURES}/issue-${issue}.txt`, 'utf8');
  const messages = JSON.parse(JSON.stringify(body.messages)
    .replace(RECORDED_WORKSPACE, workspace)
    .replaceAll('link-assistant/formal-ai/issues/1485', `${repository}/issues/${issue}`)
    .replaceAll('issue-1485-', `issue-${issue}-`));
  const call = messages.at(-1).tool_calls[0];
  messages.push({ role: 'tool', tool_call_id: call.id, content: text });
  return { model: body.model, tools: tools ? body.tools.filter(tool => tools.includes(tool.function.name)) : body.tools, messages };
}

/** Repository files tracked by git whose name is `name`, as the glob tool lists them: absolute, one per line. */
function trackedFiles(root, name) {
  const listed = execFileSync('git', ['ls-files', '-z', '--', `:(glob)**/${name}`], { cwd: root, encoding: 'utf8', maxBuffer: 1 << 24 });
  return listed.split('\0').filter(Boolean).filter(file => path.basename(file) === name).slice(0, 100);
}

/** The Agent CLI read frame, or its "File not found" error. */
function readAnswer(root, file) {
  const absolute = path.resolve(root, file);
  let stat = null;
  try { stat = statSync(absolute); } catch { stat = null; }
  if (!stat?.isFile() || !absolute.startsWith(root + path.sep)) return { content: `Error: File not found: ${absolute}`, failed: true };
  const lines = readFileSync(absolute, 'utf8').split('\n').slice(0, 2000);
  const body = lines.map((line, index) => `${String(index + 1).padStart(5, '0')}| ${line}`).join('\n');
  return { content: `<file>\n${body}\n\n(End of file - total ${lines.length} lines)\n</file>`, failed: false };
}

/** The answer to one tool call, or null for an action the emulation does not perform. */
export function answerCall(root, name, args) {
  if (name === 'read') return readAnswer(root, args.filePath);
  if (name === 'glob' && typeof args.pattern === 'string' && args.pattern.startsWith('**/')) {
    const files = trackedFiles(root, args.pattern.slice(3)).map(file => path.join(root, file));
    return { content: files.length ? files.join('\n') : 'No files found', failed: false };
  }
  const find = name === 'bash' && /^find \. -type f -name '([^'/]+)' /u.exec(args.command ?? '');
  if (find) return { content: trackedFiles(root, find[1]).map(file => `./${file}`).join('\n') + '\n', failed: false };
  return null;
}

/**
 * Plans the session step by step until a final answer, an action outside the
 * emulation, or `limit` steps. Returns every planned call with its answer.
 */
export async function runSession({ worker, agenticOutcome, chatCompletionFromPlan, parseChatRequest }, issue, root, { limit = 40, ...source } = {}) {
  const request = recordedRequest(issue, root, source);
  const prompt = request.messages.findLast(message => message.role === 'user').content;
  const calls = [];
  for (let step = 0; step < limit; step += 1) {
    const parsed = parseChatRequest(JSON.stringify({ ...request, stream: false }));
    const outcome = await agenticOutcome({ worker, agentMode: true }, parsed, () => 'refused');
    if (outcome.kind !== 'planned') return { calls, final: null, stop: outcome.kind, messages: request.messages };
    const message = chatCompletionFromPlan({}, parsed, prompt, outcome.plan, 0).choices[0].message;
    if (!message.tool_calls?.length) return { calls, final: message.content, stop: 'final', messages: request.messages };
    request.messages.push(message);
    for (const call of message.tool_calls) {
      const args = JSON.parse(call.function.arguments);
      const answer = answerCall(root, call.function.name, args);
      calls.push({ tool: call.function.name, arguments: call.function.arguments, args, answer });
      if (answer === null) return { calls, final: null, stop: 'unemulated', messages: request.messages };
      request.messages.push({ role: 'tool', tool_call_id: call.id, content: answer.content });
    }
  }
  return { calls, final: null, stop: 'limit', messages: request.messages };
}
