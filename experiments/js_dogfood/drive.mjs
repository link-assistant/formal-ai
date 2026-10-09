#!/usr/bin/env node
// Drive the JavaScript Formal AI planner (`planChatStep`, the function the JS
// server runs for /v1/chat/completions in agent mode) through a whole agentic
// session in-process, executing every planned tool call against a real
// directory the way the link-assistant Agent CLI would.
//
// This is the fast inner loop of the PR #1188 dogfooding ledger
// (docs/case-studies/pull-request-1188/formal-ai-dogfood.md): no server, no
// CLI, no Rust build. A task that passes here is then confirmed through the
// real Agent CLI against `node js/server/main.mjs --agent-mode`.
//
// Usage:
//   node experiments/js_dogfood/drive.mjs --dir <sandbox> [--steps 12] <prompt>
//
// The tool result shapes follow the Agent CLI 0.26 transcripts recorded in the
// server's dialog log: `read` returns the numbered `<file>` block, `write` and
// `edit` return an empty string, `bash` returns stdout+stderr.

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, symlinkSync, writeFileSync, writeSync } from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import { template } from '../../js/agentic/work_item_steps.mjs';

import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

/** How long one bash call may run before the driver kills it. */
const BASH_TIMEOUT_MS = Number(process.env.FORMAL_AI_BASH_TIMEOUT_MS ?? 60000);

// Only advertise adapters this in-process driver actually executes.
export const AGENT_CLI_TOOLS = ['bash', 'edit', 'grep', 'list', 'read', 'write'];

function argsOf(call) {
  try {
    return JSON.parse(call.arguments);
  } catch {
    return {};
  }
}

/** The repository root this driver sits in. */
export const REPOSITORY_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
/** Formal AI's plan-event folder, committed in this repository. */
const PLAN_EVENTS = join(REPOSITORY_ROOT, '.formal-ai');
/** Where a run from the repository root writes its plan events instead (git-ignored). */
export const PLAN_EVENTS_SANDBOX = join(REPOSITORY_ROOT, 'experiments/formal_ai_subagent/sandboxes/plan-events');

function within(dir, path) {
  const full = isAbsolute(path) ? path : join(dir, path);
  const resolved = resolve(full);
  if (resolved !== resolve(dir) && !resolved.startsWith(resolve(dir) + sep)) throw new Error(`path escapes the sandbox: ${path}`);
  // Run from the repository root, a plan event goes to a git-ignored sandbox,
  // never over the committed `.formal-ai/` plan log (PR #1188 G15).
  if (resolve(dir) === REPOSITORY_ROOT && (resolved === PLAN_EVENTS || resolved.startsWith(`${PLAN_EVENTS}${sep}`))) {
    return join(PLAN_EVENTS_SANDBOX, relative(PLAN_EVENTS, resolved));
  }
  return resolved;
}

/**
 * The Agent CLI's `grep` (ripgrep underneath): "Found N matches" then each
 * file's absolute path and its `  Line N: text` hits, or "No files found".
 */
function grep(dir, args) {
  const argv = ['--json', '--sort', 'path', '--regexp', args.pattern ?? args.query ?? ''];
  if (args.include) argv.push('--glob', args.include);
  argv.push('--', within(dir, args.path ?? '.'));
  let output;
  try {
    output = execFileSync('rg', argv, { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
  } catch (error) {
    if (error.status === 1) return 'No files found';
    throw error;
  }
  const groups = new Map();
  let count = 0;
  for (const record of output.split('\n').filter(Boolean).map((line) => JSON.parse(line))) {
    if (record.type !== 'match') continue;
    const file = record.data.path.text;
    const line = record.data.lines.text.replace(/\r?\n$/u, '');
    if (!groups.has(file)) groups.set(file, []);
    groups.get(file).push('  Line ' + record.data.line_number + ': ' + line);
    count += 1;
  }
  return count ? 'Found ' + count + ' matches\n' + [...groups].map(([file, hits]) => [file + ':', ...hits].join('\n')).join('\n\n') : 'No files found';
}

/** Adapter failures are transport metadata, distinct from authored file/stdout bytes. */
function toolFailure(message) {
  return JSON.stringify({ is_error: true, error: String(message) });
}

/** Execute one tool call the way the Agent CLI does; returns its result text. */
export function execute(dir, call) {
  const args = argsOf(call);
  const path = args.filePath ?? args.file_path ?? args.path;
  try {
    switch (call.tool) {
      case 'read': {
        const text = readFileSync(within(dir, path), 'utf8');
        const lines = text.split('\n');
        const body = lines.map((line, index) => `${String(index + 1).padStart(5, '0')}| ${line}`).join('\n');
        return `<file>\n${body}\n\n(End of file - total ${lines.length} lines)\n</file>`;
      }
      case 'write': {
        const target = within(dir, path);
        mkdirSync(dirname(target), { recursive: true });
        writeFileSync(target, args.content ?? '');
        return '';
      }
      case 'edit': {
        const target = within(dir, path);
        const text = existsSync(target) ? readFileSync(target, 'utf8') : '';
        const oldString = args.oldString ?? args.old_string ?? '';
        const newString = args.newString ?? args.new_string ?? '';
        if (oldString === '') {
          writeFileSync(target, newString);
          return '';
        }
        if (!text.includes(oldString)) return toolFailure('oldString not found in content');
        // The Agent CLI refuses an ambiguous oldString rather than editing its first match.
        if (!args.replaceAll && text.indexOf(oldString) !== text.lastIndexOf(oldString)) {
          return toolFailure('Found multiple matches for oldString. Provide more surrounding lines in oldString to identify the correct match.');
        }
        const next = args.replaceAll ? text.split(oldString).join(newString) : text.replace(oldString, () => newString);
        writeFileSync(target, next);
        return '';
      }
      case 'bash': {
        let eventDirectory = null;
        let commandDirectory = dir;
        const eventPrefix = template('plan-event-append-command')?.split('{lock_path}')[0];
        if (resolve(dir) === REPOSITORY_ROOT && eventPrefix && args.command.startsWith(eventPrefix)) {
          mkdirSync(PLAN_EVENTS_SANDBOX, { recursive: true });
          eventDirectory = mkdtempSync(join(tmpdir(), 'formal-ai-plan-events-'));
          symlinkSync(PLAN_EVENTS_SANDBOX, join(eventDirectory, '.formal-ai'), process.platform === 'win32' ? 'junction' : 'dir');
          commandDirectory = eventDirectory;
        }
        try {
          const output = execFileSync('/bin/sh', ['-c', args.command], { cwd: commandDirectory, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: BASH_TIMEOUT_MS });
          return 'Output: ' + output + '\nExit Code: 0';
        } catch (error) {
          // A call killed at the timeout reports the kill, never an exit code:
          // a killed `node --test` exits 1 on SIGTERM, which is not the
          // command's own failure (PR #1188 G77).
          if (error.code === 'ETIMEDOUT' || (error.signal && typeof error.status !== 'number')) {
            const timeout = error.code === 'ETIMEDOUT' ? `\nTimeout: ${BASH_TIMEOUT_MS} ms` : '';
            return `Output: ${error.stdout ?? ''}\nError: ${error.stderr ?? ''}\nSignal: ${error.signal ?? 'SIGTERM'}${timeout}`;
          }
          // A failing command reports its exit code the way the Agent CLI's
          // shell envelope does, so a recipe's failed precondition blocks it.
          if (typeof error.status !== 'number') return `${error.stdout ?? ''}${error.stderr ?? ''}`;
          return `Output: ${error.stdout ?? ''}\nError: ${error.stderr ?? ''}\nExit Code: ${error.status}`;
        } finally {
          if (eventDirectory !== null) rmSync(eventDirectory, { recursive: true, force: true });
        }
      }
      case 'list':
        return readdirSync(within(dir, path ?? '.')).join('\n');
      case 'grep':
        return grep(dir, args);
      default:
        return toolFailure(`the dogfood driver does not execute ${call.tool}`);
    }
  } catch (error) {
    return toolFailure(error.message);
  }
}

/**
 * Run a whole session; returns `{transcript, answer, stop}`. `answer` is null
 * when the session ends without one, and `stop` then says why: `no-plan` (the
 * planner and its fall-through planned nothing) or `steps` (the step budget
 * ran out mid-task, PR #1188 G84).
 */
export async function drive(planChatStep, dir, prompt, { tools = AGENT_CLI_TOOLS, steps = 12, fallthrough = null } = {}) {
  const messages = [
    { role: 'system', content: `<env>\n  Working directory: ${dir}\n  Is directory a git repo: yes\n</env>` },
    { role: 'user', content: prompt },
  ];
  const transcript = [];
  for (let step = 0; step < steps; step += 1) {
    // The server's fall-through: no planned step means the solver answers and
    // the symbolic command reroute may still turn that answer into tool calls
    // (js/server/agentic.mjs commandReroutePlan).
    const plan = (await planChatStep(messages, tools)) ?? (fallthrough ? await fallthrough(messages, tools) : null);
    if (!plan) return { transcript, answer: null, stop: 'no-plan' };
    if (plan.kind === 'final') return { transcript, answer: plan.answer, stop: 'final' };
    const toolCalls = plan.calls.map((call, index) => ({
      id: `c${step}_${index}`, type: 'function', function: { name: call.tool, arguments: call.arguments },
    }));
    messages.push({ role: 'assistant', content: '', tool_calls: toolCalls });
    plan.calls.forEach((call, index) => {
      const result = execute(dir, call);
      transcript.push({ tool: call.tool, arguments: call.arguments, result });
      messages.push({ role: 'tool', tool_call_id: toolCalls[index].id, name: call.tool, content: result });
    });
  }
  return { transcript, answer: null, stop: 'steps' };
}

/** What the driver prints when a session ends with no answer: never a bare `null`. */
export function unanswered(stop, steps, calls) {
  if (stop === 'steps') return `(no answer: the ${steps}-step budget ran out after ${calls} tool calls, before the task finished; run again with a larger --steps)`;
  return `(no answer: the planner planned no step after ${calls} tool calls)`;
}

async function main(argv) {
  let dir = null;
  let steps = 12;
  const rest = [];
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === '--dir') dir = argv[++index];
    else if (argv[index] === '--steps') steps = Number(argv[++index]);
    else rest.push(argv[index]);
  }
  if (!dir || rest.length === 0) throw new Error('usage: drive.mjs --dir <sandbox> [--steps N] <prompt>');
  await installNodeHost(new WorkerHost());
  const { planChatStep } = await import('../../js/agentic/planner.mjs');
  const { solve } = await import('../../js/agentic/host.mjs');
  const { planSymbolicCommandReroute } = await import('../../js/agentic/command_reroute.mjs');
  const { latestUserRequest } = await import('../../js/agentic/content.mjs');
  const fallthrough = async (messages, tools) => {
    const symbolic = await solve(latestUserRequest(messages) ?? '', []);
    return planSymbolicCommandReroute(messages, tools, symbolic) ?? { kind: 'final', answer: symbolic.answer };
  };
  const { transcript, answer, stop } = await drive(planChatStep, resolve(dir), rest.join(' '), { steps, fallthrough });
  const out = [];
  for (const entry of transcript) {
    out.push(`>> ${entry.tool} ${entry.arguments}`);
    if (entry.result) out.push(entry.result.split('\n').map((line) => `   ${line}`).join('\n'));
  }
  out.push(`== answer ==\n${answer ?? unanswered(stop, steps, transcript.length)}`);
  // Written synchronously: process.exit drops what a pipe has not drained yet,
  // which cut the answer off a long transcript.
  writeSync(1, `${out.join('\n')}\n`);
  process.exit(0);
}

if (import.meta.url === `file://${process.argv[1]}`) await main(process.argv.slice(2));
