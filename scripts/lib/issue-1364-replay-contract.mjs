// The live acceptance runner binds public task inputs and actual checkout observations.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {workflowTriggers} from './ci-speed-workflows.mjs';
import {parseChatRequest} from '../../js/server/chat-request.mjs';
import {parseAnthropicRequest, anthropicToChatCompletionRequest} from '../../js/server/anthropic.mjs';
import {parseResponsesRequest, toChatCompletionRequest} from '../../js/server/responses-input.mjs';
import {failureMessage} from '../../js/agentic/tool_result.mjs';
import {plainText} from '../../js/agentic/content.mjs';
import {jsonText} from '../../js/agentic/plan.mjs';
import {classifyTool} from '../../js/agentic/capability_router.mjs';
export const replaySchema = 'formal-ai-issue-1364-delivery/v1';
export const contentDigest = value => createHash('sha256').update(value).digest('hex');
export function runtimeImageDigest(reference) {
  const match = /^(?:ghcr\.io\/link-assistant\/formal-ai@)?(sha256:[a-f0-9]{64})$/u.exec(reference);
  assert(match, 'immutable actual runtime reference required');
  return match[1];
}
export function requireReplayTask(inputs, tool, issue, pullRequest) {
  assert.equal(inputs.schema, 'formal-ai-issue-1364-client-replay-inputs/v1');
  assert(Object.hasOwn(inputs.clients, tool));
  const task = inputs.clients[tool];
  assert.match(task.bodySha256 ?? '', /^[a-f0-9]{64}$/u);
  if (Object.hasOwn(task, 'body')) assert.equal(contentDigest(task.body), task.bodySha256);
  assert.equal(issue.html_url, task.issueUrl);
  assert.equal(contentDigest(issue.body), task.bodySha256, 'historical issue requirements changed');
  assert.equal(pullRequest.html_url, task.pullRequestUrl);
  assert.equal(pullRequest.state, 'open');
  assert.equal(pullRequest.head.ref, task.branch);
  assert.equal(pullRequest.base.ref, 'main');
  assert.match(pullRequest.head.sha, /^[a-f0-9]{40}$/u);
  return task;
}
export function adaptReplayEntrypoint(source, task, workspace) {
  assert(path.isAbsolute(workspace) && !workspace.includes(':') && !/[\r\n\0]/u.test(workspace));
  const original = "const dockerArgv = buildDockerArgv({ solveArgv: buildE2eSolveArgv({ issueUrl, tool, model, baseBranch, logDir: '/home/box/logs' }), solveCommand: '/opt/hive-e2e/src/solve.mjs', image, hostLogDir, forwardEnv: ['GH_TOKEN', ...modelKeys, 'FORMAL_AI_INSTALL_GRANT', 'GITHUB_ACTIONS'] });";
  assert.equal(source.split(original).length, 2, 'unreviewed pinned solve constructor');
  const target = '/home/box/formal-ai-issue-1364-workspace';
  const replacement = [
    "const replayArgv = buildE2eSolveArgv({issueUrl, tool, model, "
      + (task.baseBranch ? 'baseBranch: ' + JSON.stringify(task.baseBranch) + ', ' : '')
      + "logDir: '/home/box/logs'});",
    ...(task.pullRequestUrl ? ['replayArgv[0] = ' + JSON.stringify(task.pullRequestUrl) + ';'] : []),
    'replayArgv.push("--working-directory", ' + JSON.stringify(target) + ');',
    // Raw client transcripts stay in the owned evidence directory, not public issue attachments.
    'const attachmentFlag = replayArgv.indexOf("--attach-logs");',
    'if (attachmentFlag < 0 || attachmentFlag !== replayArgv.lastIndexOf("--attach-logs")) throw Error("pinned attachment flag changed");',
    'replayArgv.splice(attachmentFlag, 1);',
    // Use Hive's supported bookkeeping option; ordinary Git hooks remain in force.
    'if (replayArgv.some(value => value === "--attribution" || value.startsWith("--attribution="))) throw Error("pinned attribution policy changed");',
    'replayArgv.push("--attribution", "none");',
    "const dockerArgv = buildDockerArgv({solveArgv:replayArgv, solveCommand:'/opt/hive-e2e/src/solve.mjs', image, hostLogDir,",
    "  forwardEnv:['GH_TOKEN', ...modelKeys, 'FORMAL_AI_INSTALL_GRANT', 'GITHUB_ACTIONS'],",
    '  extraMounts:[' + JSON.stringify(workspace + ':' + target) + ']});',
  ].join('\n');
  let adapted = source.replace(original, () => replacement);
  if (task.workflowDirectory) {
    assert.equal(task.workflowDirectory, 'ci/');
    const workflow = "const workflowDir = !option('--issue') && isolation === 'branch' ? workflowDirectoryForLayer(evidence.layer) : '.github/workflows/';";
    assert.equal(adapted.split(workflow).length, 2, 'pinned workflow staging selection');
    assert.equal(adapted.split('let baseBranch;').length, 2, 'pinned isolated base selection');
    adapted = adapted.replace(workflow, "const workflowDir = 'ci/';")
      .replace('let baseBranch;', 'let baseBranch = ' + JSON.stringify(task.baseBranch) + ';');
  }
  return adapted;
}
export function taskSnapshot(directory, trackedPaths, issueUrl = null) {
  if (issueUrl !== null) {
    const owner = /^https:\/\/github\.com\/[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+\/issues\/([1-9][0-9]*)$/u.exec(issueUrl);
    assert(owner, 'canonical issue owner required for task evidence custody');
    const evidence = 'dev/log/self-authored/issue-' + owner[1] + '/evidence/';
    // Exact internal members of the pinned Hive producer, bound to this issue only.
    const internal = ['agent-stream.jsonl', 'claude-stream.jsonl', 'codex-stream.jsonl', 'session-id.txt'];
    assert(trackedPaths.every(relative => !internal.some(name => relative === evidence + name)),
      'internal task evidence belongs outside the delivered commit');
  }
  assert(trackedPaths.length > 0 && trackedPaths.length <= 256);
  let total = 0;
  const rows = trackedPaths.slice().sort().map(relative => {
    assert(relative && !relative.startsWith('/') && !relative.includes('\\') && !relative.includes('\0'));
    assert(relative.split('/').every(part => part && part !== '.' && part !== '..' && part !== '.git'));
    let component = directory;
    for (const part of relative.split('/')) {
      component = path.join(component, part);
      assert(!fs.lstatSync(component).isSymbolicLink(), 'task source symlink');
    }
    const before = fs.statSync(component, {bigint:true});
    assert(before.isFile() && before.size <= 1048576n, 'source-sized regular task member required');
    const bytes = fs.readFileSync(component); total += bytes.length;
    assert(total <= 8388608, 'bounded task source inventory');
    assert(!bytes.includes(0), 'binary task member');
    assert.deepEqual(Buffer.from(bytes.toString('utf8')), bytes, 'invalid UTF-8 source member');
    const after = fs.statSync(component, {bigint:true});
    const observed = stat => [stat.dev, stat.ino, stat.size, stat.mtimeNs, stat.ctimeNs].map(String);
    assert.deepEqual(observed(before), observed(after), 'task source changed during observation');
    return {path:relative, identity:observed(after), sha256:contentDigest(bytes)};
  });
  return rows;
}
export function requireNoopTask(before, after) {
  assert.equal(after.head, before.head, 'second pass created or moved a commit');
  assert.equal(after.branch, before.branch);
  assert.equal(after.status, '', 'second pass left task changes');
  assert.equal(after.commitRecords, before.commitRecords, 'second pass created a recorded commit');
  assert.deepEqual(after.files, before.files, 'second pass rewrote task source or changed its file membership');
}
export function requireActualTaskCi(checks) {
  assert(Array.isArray(checks) && checks.length > 0, 'actual task CI is missing');
  for (const check of checks) {
    if (check.__typename === 'StatusContext') assert.equal(check.state, 'SUCCESS');
    else {assert.equal(check.status, 'COMPLETED'); assert.equal(check.conclusion, 'SUCCESS');}
  }
}
export function requireTaskObligations({source, workflow, verifier, programPath}) {
  const header = source.split('\n').slice(0, 16).filter(line => line.trimStart().startsWith('//'));
  assert(header.some(line => /emit|print|greet|newline/iu.test(line)), 'program needs an explanatory comment');
  assert(header.some(line => line.includes(programPath)), 'program needs source-bound build/run instructions');
  const events = workflowTriggers(workflow);
  assert(events.includes('push') && events.includes('pull_request'), 'actual workflow needs both task events');
  assert(workflow.includes('tests/verify-output.sh'), 'actual workflow must execute the verifier');
  assert(verifier.includes('cmp ') && verifier.includes('set -eu'), 'actual verifier must preserve failure and exact bytes');
  return true;
}
function capturedArgumentValue(value) {
  // Custom tools carry literal text; ordinary function tools carry JSON.
  if (typeof value !== 'string') return value;
  try { return JSON.parse(value); } catch { return value; }
}
export function canonicalCapturedMessages(context) {
  assert(context.metadata.exchange_count > 0 && context.server_logs.length === context.metadata.exchange_count);
  const modelExchanges = context.server_logs.filter(row => row.method === 'POST' && row.request_body);
  assert(modelExchanges.length > 0, 'complete model exchanges required');
  const families = new Set(modelExchanges.map(row => {
    // Query parameters select transport features, not a different message protocol.
    const pathname = new URL(row.path, 'http://capture.invalid').pathname;
    return pathname.endsWith('/chat/completions') ? 'chat'
      : pathname.endsWith('/messages') ? 'anthropic' : pathname.endsWith('/responses') ? 'responses' : 'unknown';
  }));
  assert.equal(families.size, 1, 'mixed or unrecognized protocol capture');
  const family = [...families][0];
  assert.equal(modelExchanges.at(-1).response_tool_calls.length, 0, 'capture ended before a tool receipt');
  // Persisted conversation views merge cumulative client prefixes and can contain
  // duplicates. The final complete wire request is the authoritative tool history.
  // Explicit incremental Responses clients still need the recorded accumulated
  // history; the emitted-call identity comparison below applies to both paths.
  const last = modelExchanges.at(-1).request_body;
  const incremental = family === 'responses' && typeof JSON.parse(last).previous_response_id === 'string'
    && JSON.parse(last).previous_response_id.length > 0;
  const body = incremental ? JSON.stringify({model:'formal-ai',input:context.messages}) : last;
  let messages;
  if (family === 'chat') messages = parseChatRequest(body).messages;
  else if (family === 'anthropic') messages = anthropicToChatCompletionRequest(parseAnthropicRequest(body)).messages;
  else {
    assert.equal(family, 'responses', 'unrecognized model capture');
    messages = toChatCompletionRequest(parseResponsesRequest(body)).messages;
  }
  const emitted = modelExchanges.flatMap(row => row.response_tool_calls)
    .map(call => [call.name, capturedArgumentValue(call.arguments)]);
  const captured = messages.flatMap(row => row.tool_calls ?? [])
    .map(call => [call.function.name, capturedArgumentValue(call.function.arguments)]);
  assert.equal(captured.length, emitted.length, 'tool history is missing, duplicated, or predates this capture');
  assert.equal(jsonText(captured), jsonText(emitted), 'captured tool identities differ from actual model emissions');
  return messages;
}
export function auditCapturedActions(messages, failures = new Map()) {
  const pending = new Map();
  let writes = 0, calls = 0, completed = 0;
  for (const message of messages) {
    if (message.role === 'assistant') {
      const batch = new Set();
      for (const call of message.tool_calls ?? []) {
        assert(typeof call.id === 'string' && call.id && !batch.has(call.id) && !pending.has(call.id), 'unbound/ambiguous client call');
        batch.add(call.id); pending.set(call.id, call); calls++;
        if (['write','edit','multi_edit','subagent'].includes(classifyTool(call.function.name))) writes++;
      }
    } else if (message.role === 'tool') {
      const call = pending.get(message.tool_call_id);
      assert(call, 'orphaned or duplicate observed tool receipt'); pending.delete(message.tool_call_id); completed++;
      assert(!message.name || message.name.toLowerCase() === call.function.name.toLowerCase());
      if (failureMessage(plainText(message.content), message.is_error === true || message.isError === true, false) !== null) {
        const key = jsonText([call.function.name.toLowerCase(), capturedArgumentValue(call.function.arguments)]);
        const count = (failures.get(key) ?? 0) + 1; failures.set(key, count);
        assert(count <= 2, 'more than two identical failed actions');
      }
    }
  }
  assert(calls > 0 && completed > 0, 'tool capture missing');
  assert.equal(pending.size, 0, 'tool capture ended with missing results');
  return {calls, completed, writes, maximumIdenticalFailures: Math.max(0, ...failures.values())};
}
