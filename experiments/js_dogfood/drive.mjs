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
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, join, resolve } from 'node:path';

import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

export const AGENT_CLI_TOOLS = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read', 'task',
  'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];

function argsOf(call) {
  try {
    return JSON.parse(call.arguments);
  } catch {
    return {};
  }
}

function within(dir, path) {
  const full = isAbsolute(path) ? path : join(dir, path);
  const resolved = resolve(full);
  if (!resolved.startsWith(resolve(dir))) throw new Error(`path escapes the sandbox: ${path}`);
  return resolved;
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
        const next = args.replaceAll ? text.split(oldString).join(newString) : text.replace(oldString, () => newString);
        writeFileSync(target, next);
        return '';
      }
      case 'bash': {
        try {
          return execFileSync('/bin/sh', ['-c', args.command], { cwd: dir, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 60000 });
        } catch (error) {
          return `${error.stdout ?? ''}${error.stderr ?? ''}`;
        }
      }
      case 'list':
        return readdirSync(within(dir, path ?? '.')).join('\n');
      default:
        return `Error: the dogfood driver does not execute ${call.tool}`;
    }
  } catch (error) {
    return `Error: ${error.message}`;
  }
}

/** Run a whole session; returns `{transcript, answer}`. */
export async function drive(planChatStep, dir, prompt, { tools = AGENT_CLI_TOOLS, steps = 12 } = {}) {
  const messages = [
    { role: 'system', content: `<env>\n  Working directory: ${dir}\n  Is directory a git repo: yes\n</env>` },
    { role: 'user', content: prompt },
  ];
  const transcript = [];
  for (let step = 0; step < steps; step += 1) {
    const plan = await planChatStep(messages, tools);
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
  const { transcript, answer } = await drive(planChatStep, resolve(dir), rest.join(' '), { steps });
  for (const entry of transcript) {
    console.log(`>> ${entry.tool} ${entry.arguments}`);
    if (entry.result) console.log(entry.result.split('\n').map((line) => `   ${line}`).join('\n'));
  }
  console.log(`== answer ==\n${answer}`);
  process.exit(0);
}

if (import.meta.url === `file://${process.argv[1]}`) await main(process.argv.slice(2));
