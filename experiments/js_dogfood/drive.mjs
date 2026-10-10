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
// The legacy execute projection keeps Agent CLI numbered Read blocks.
// drive attaches actual raw Read bytes and provider status in separate fields;
// Write/Edit text receipts and actual Bash process observations keep their APIs.

import { shellCapture } from './shell-capture.mjs';
import { grepCapture } from './grep-capture.mjs';
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
  return grepCapture(argv);
}

/** Adapter failures are transport metadata, distinct from authored file/stdout bytes. */
function toolFailure(message) {
  return JSON.stringify({ is_error: true, error: String(message) });
}

/** Read provider metadata is attached outside the actual file bytes. */
export function executeResult(dir, call) {
  if (call.tool !== 'read') return { content: execute(dir, call) };
  const args = argsOf(call);
  const path = args.filePath ?? args.file_path ?? args.path;
  try {
    const content = readFileSync(within(dir, path), 'utf8');
    return { content, source_read: { path, success: true, complete: true, format: 'raw' } };
  } catch (error) {
    return { content: toolFailure(error.message), is_error: true };
  }
}

/** Execute one tool call the way the Agent CLI does; returns its result text. */
export function execute(dir, call) {
  const args = argsOf(call);
  const path = args.filePath ?? args.file_path ?? args.path;
  try {
    switch (call.tool) {
      case 'read': {
        const receipt = executeResult(dir, call);
        if (receipt.is_error) return receipt.content;
        const text = receipt.content;
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
          return shellCapture(args.command, {cwd:commandDirectory, timeoutMs:BASH_TIMEOUT_MS});
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
export async function drive(planChatStep, dir, prompt, {
  tools = AGENT_CLI_TOOLS, steps = 12, fallthrough = null, allowedCommands = undefined,
} = {}) {
  if (!Array.isArray(tools) || new Set(tools).size !== tools.length || tools.some((tool) => !AGENT_CLI_TOOLS.includes(tool))) {
    return { transcript: [], answer: null, stop: 'invalid-tools', toolsAdvertised: [] };
  }
  tools = Object.freeze([...tools]);
  if (allowedCommands !== undefined) {
    if (!Array.isArray(allowedCommands)
        || allowedCommands.some(command => typeof command !== 'string')
        || new Set(allowedCommands).size !== allowedCommands.length) {
      return { transcript: [], answer: null, stop: 'invalid-command-policy', toolsAdvertised: tools };
    }
    allowedCommands = Object.freeze([...allowedCommands]);
  }
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
    if (!plan) return { transcript, answer: null, stop: 'no-plan', toolsAdvertised: tools };
    if (plan.kind === 'final') return { transcript, answer: plan.answer, stop: 'final', toolsAdvertised: tools };
    // Snapshot the complete batch before validation; getters cannot change executed arguments.
    const calls = plan.calls.map(call => Object.freeze({ tool: call.tool, arguments: call.arguments }));
    if (calls.some((call) => !tools.includes(call.tool))) {
      return { transcript, answer: null, stop: 'undeclared-tool', toolsAdvertised: tools };
    }
    if (allowedCommands !== undefined) {
      const deniedCalls = calls.filter(call => {
        if (call.tool !== 'bash') return false;
        try {
          if (typeof call.arguments !== 'string') return true;
          const arguments_ = JSON.parse(call.arguments);
          return !arguments_ || typeof arguments_.command !== 'string'
            || !allowedCommands.includes(arguments_.command);
        } catch {
          return true;
        }
      });
      if (deniedCalls.length) {
        return { transcript, answer: null, stop: 'command-policy-denied',
          toolsAdvertised: tools, deniedCalls };
      }
    }
    const toolCalls = calls.map((call, index) => ({
      id: `c${step}_${index}`, type: 'function', function: { name: call.tool, arguments: call.arguments },
    }));
    messages.push({ role: 'assistant', content: '', tool_calls: toolCalls });
    calls.forEach((call, index) => {
      const receipt = executeResult(dir, call);
      const { content: result, ...metadata } = receipt;
      transcript.push({ tool: call.tool, arguments: call.arguments, result, ...metadata });
      messages.push({ role: 'tool', tool_call_id: toolCalls[index].id, name: call.tool, ...receipt });
    });
  }
  return { transcript, answer: null, stop: 'steps', toolsAdvertised: tools };
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
