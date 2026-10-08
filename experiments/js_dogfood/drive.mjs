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
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

/** How long one bash call may run before the driver kills it. */
const BASH_TIMEOUT_MS = Number(process.env.FORMAL_AI_BASH_TIMEOUT_MS ?? 60000);

export const AGENT_CLI_TOOLS = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read', 'task',
  'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];

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
  if (!resolved.startsWith(resolve(dir))) throw new Error(`path escapes the sandbox: ${path}`);
  // Run from the repository root, a plan event goes to a git-ignored sandbox,
  // never over the committed `.formal-ai/` plan log (PR #1188 G15).
  if (resolve(dir) === REPOSITORY_ROOT && (resolved === PLAN_EVENTS || resolved.startsWith(`${PLAN_EVENTS}${sep}`))) {
    return join(PLAN_EVENTS_SANDBOX, relative(PLAN_EVENTS, resolved));
  }
  return resolved;
}

/** Every file under `path` (a file or a directory), skipping VCS and dependency folders. */
function filesUnder(path) {
  if (!statSync(path).isDirectory()) return [path];
  return readdirSync(path).filter((name) => name !== '.git' && name !== 'node_modules').sort()
    .flatMap((name) => filesUnder(join(path, name)));
}

/**
 * The Agent CLI's `grep` (ripgrep underneath): "Found N matches" then each
 * file's absolute path and its `  Line N: text` hits, or "No files found".
 */
function grep(dir, args) {
  const pattern = new RegExp(args.pattern ?? args.query ?? '');
  const groups = [];
  let count = 0;
  for (const file of filesUnder(within(dir, args.path ?? '.'))) {
    const lines = readFileSync(file, 'utf8').split('\n');
    const hits = lines.map((line, index) => [index + 1, line]).filter(([, line]) => pattern.test(line));
    if (!hits.length) continue;
    count += hits.length;
    groups.push([`${file}:`, ...hits.map(([number, line]) => `  Line ${number}: ${line}`)].join('\n'));
  }
  return count ? `Found ${count} matches\n${groups.join('\n\n')}` : 'No files found';
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
        if (!text.includes(oldString)) return 'Error: oldString not found in content';
        // The Agent CLI refuses an ambiguous oldString rather than editing its first match.
        if (!args.replaceAll && text.indexOf(oldString) !== text.lastIndexOf(oldString)) {
          return 'Error: Found multiple matches for oldString. Provide more surrounding lines in oldString to identify the correct match.';
        }
        const next = args.replaceAll ? text.split(oldString).join(newString) : text.replace(oldString, () => newString);
        writeFileSync(target, next);
        return '';
      }
      case 'bash': {
        try {
          return execFileSync('/bin/sh', ['-c', args.command], { cwd: dir, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: BASH_TIMEOUT_MS });
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
        }
      }
      case 'list':
        return readdirSync(within(dir, path ?? '.')).join('\n');
      case 'grep':
        return grep(dir, args);
      default:
        return `Error: the dogfood driver does not execute ${call.tool}`;
    }
  } catch (error) {
    return `Error: ${error.message}`;
  }
}

/** Run a whole session; returns `{transcript, answer}`. */
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
    if (!plan) return { transcript, answer: null };
    if (plan.kind === 'final') return { transcript, answer: plan.answer };
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
  return { transcript, answer: null };
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
  const { transcript, answer } = await drive(planChatStep, resolve(dir), rest.join(' '), { steps, fallthrough });
  for (const entry of transcript) {
    console.log(`>> ${entry.tool} ${entry.arguments}`);
    if (entry.result) console.log(entry.result.split('\n').map((line) => `   ${line}`).join('\n'));
  }
  console.log(`== answer ==\n${answer}`);
  process.exit(0);
}

if (import.meta.url === `file://${process.argv[1]}`) await main(process.argv.slice(2));
