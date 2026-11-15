// Drives a Hive Mind session on one of the ten issues whose drafts ended
// `planned_not_executed` (issue #1483) to its end. The planner is this
// checkout's Formal AI; the workspace is a disposable git worktree at any
// commit, so the 1455 pre-fix checkout can be replayed with the current build.
// Tool calls are executed for real in the worktree: bash runs there (the `gh`
// read of the issue is answered from the recorded fixture), reads, writes and
// edits touch its files. Network fetches are answered as unavailable.
//
// Usage:
//   node experiments/issue-1483/session-outcome.mjs <number|all> [--at <commit>] [--steps N] [--keep]
// Prints one line per issue: calls, final reason, disposition and the diff stat.
import { existsSync, readFileSync, statSync, writeFileSync, mkdtempSync, rmSync, mkdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync, spawnSync } from 'node:child_process';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../js/agentic/planner.mjs';
import { workItemRecordOf, parsesAsJsonObject } from '../../js/agentic/work_item_report.mjs';
import { completeOwnedSourceReadFrame } from '../../js/agentic/tool_result.mjs';

await installNodeHost(new WorkerHost({}));
const root = new URL('../../', import.meta.url).pathname.replace(/\/$/u, '');
const args = process.argv.slice(2);
const option = (name, fallback) => {
  const at = args.indexOf(name);
  return at >= 0 ? args[at + 1] : fallback;
};
const fixtures = join(root, 'data/benchmarks/fixtures/work-item-1483');
const all = JSON.parse(readFileSync(join(fixtures, 'issues.json'), 'utf8')).issues;
const template = readFileSync(join(fixtures, 'hive-mind-prompt.txt'), 'utf8');
const tools = ['bash', 'read', 'glob', 'grep', 'list', 'edit', 'write', 'task', 'webfetch', 'websearch', 'codesearch', 'batch', 'todowrite', 'todoread'];
const verbose = process.env.VERBOSE === '1';
const at = option('--at', 'HEAD');
const steps = Number(option('--steps', '40'));
const first = args[0] && !args[0].startsWith('--') ? args[0] : 'all';
const numbers = first === 'all' ? Object.keys(all) : [first];

function worktree() {
  const parent = mkdtempSync(join(tmpdir(), 'formal-ai-1483-'));
  const path = join(parent, 'checkout');
  execFileSync('git', ['worktree', 'add', '--detach', '--quiet', path, at], { cwd: root });
  return path;
}

function drop(path) {
  execFileSync('git', ['worktree', 'remove', '--force', path], { cwd: root });
  rmSync(dirname(path), { recursive: true, force: true });
}

function execute(call, argsValue, issue, workspace) {
  const path = argsValue.filePath ?? argsValue.file_path ?? argsValue.path;
  const full = typeof path === 'string' ? (path.startsWith('/') ? path : join(workspace, path)) : null;
  if (call.tool === 'bash') {
    const command = String(argsValue.command ?? '');
    if (/\bgh (issue|api|pr) /u.test(command) || /api\.github\.com/u.test(command)) {
      return issue.title + '\n\n' + issue.body + '\n\n__formal_ai_exit=0\n';
    }
    const run = spawnSync('bash', ['-c', command], { cwd: workspace, encoding: 'utf8', timeout: 60_000, maxBuffer: 1 << 26 });
    const output = (run.stdout ?? '') + (run.stderr ?? '');
    return command.includes('__formal_ai_exit') ? output : output + `\n__formal_ai_exit=${run.status ?? 1}\n`;
  }
  if (call.tool === 'glob') {
    const name = String(argsValue.pattern ?? '').replace(/^\*\*\//u, '');
    const tracked = execFileSync('git', ['ls-files'], { cwd: workspace, encoding: 'utf8', maxBuffer: 1 << 28 }).split('\n').filter(Boolean);
    return tracked.filter(file => file === name || file.endsWith('/' + name)).join('\n');
  }
  if (call.tool === 'read') {
    return full && existsSync(full) && statSync(full).isFile() ? completeOwnedSourceReadFrame(readFileSync(full, 'utf8')) : 'Error: File not found: ' + path;
  }
  if (call.tool === 'write') {
    mkdirSync(dirname(full), { recursive: true });
    writeFileSync(full, String(argsValue.content ?? ''));
    return 'Wrote file successfully.';
  }
  if (call.tool === 'edit') {
    const old = argsValue.oldString ?? argsValue.old_string;
    const replacement = argsValue.newString ?? argsValue.new_string;
    if (!full || !existsSync(full)) return 'Error: File not found: ' + path;
    const before = readFileSync(full, 'utf8');
    if (!before.includes(old)) return 'Error: oldString not found in content';
    writeFileSync(full, before.replace(old, replacement));
    return 'Edit applied successfully.';
  }
  if (call.tool === 'grep') {
    const run = spawnSync('git', ['grep', '-n', '-I', '-e', String(argsValue.pattern ?? '')], { cwd: workspace, encoding: 'utf8', maxBuffer: 1 << 26 });
    return run.stdout.split('\n').slice(0, 100).join('\n') || 'No files found';
  }
  if (call.tool === 'webfetch' || call.tool === 'websearch' || call.tool === 'codesearch') return 'Error: network access is not available in this replay';
  return '';
}

for (const number of numbers) {
  const issue = all[number];
  const workspace = worktree();
  try {
    const prompt = template.split('{number}').join(number).split('{pull}').join(String(issue.draft)).split('{branch}').join('0123456789ab');
    const messages = [
      { role: 'system', content: `Working directory: ${workspace}` },
      { role: 'user', content: prompt },
    ];
    const calls = [];
    let final = null;
    for (let step = 0; step < steps; step += 1) {
      const plan = await planChatStepResolved(messages, tools);
      if (plan?.kind !== 'tool_calls') { final = plan; break; }
      const call = plan.calls[0];
      const argsValue = JSON.parse(call.arguments);
      const content = execute(call, argsValue, issue, workspace);
      calls.push(call.tool + ' ' + JSON.stringify(argsValue).slice(0, 160).replaceAll('\n', ' '));
      if (verbose) console.log(`   [${step}] ${calls.at(-1)}\n      -> ${content.slice(0, 300).replaceAll('\n', ' | ')}`);
      const id = 'c' + step;
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
        { role: 'tool', tool_call_id: id, name: call.tool, content });
    }
    const record = workItemRecordOf(final);
    const diff = execFileSync('git', ['status', '--porcelain'], { cwd: workspace, encoding: 'utf8' }).trim().split('\n').filter(Boolean);
    console.log(`#${number} calls=${calls.length} json=${parsesAsJsonObject(final?.answer ?? '')} reason=${record?.reason ?? '-'} disposition=${final?.result?.disposition ?? '-'} changed=${JSON.stringify(diff)}`);
    console.log('  ' + String(final?.answer ?? '(no final within the step budget)').split('\n')[0].slice(0, 220));
    if (process.env.RECORD_DIR) {
      writeFileSync(join(process.env.RECORD_DIR, `${number}.json`), JSON.stringify({ calls, answer: final?.answer ?? null, disposition: final?.result?.disposition ?? null, record, changed: diff, diff: execFileSync('git', ['diff'], { cwd: workspace, encoding: 'utf8' }) }, null, 1));
    }
    if (verbose) {
      console.log(String(final?.answer ?? '').slice(0, 4000));
      console.log(JSON.stringify(record, null, 1)?.slice(0, 6000));
      if (diff.length) console.log(execFileSync('git', ['diff'], { cwd: workspace, encoding: 'utf8' }).slice(0, 6000));
    }
  } finally {
    if (!args.includes('--keep')) drop(workspace);
  }
}
