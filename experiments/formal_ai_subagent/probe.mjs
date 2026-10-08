#!/usr/bin/env node
// Look inside the JavaScript Formal AI planner without running a session: the
// first plan step for a request, how the edit composer reads it, which path
// the read fallback picks, and how the quote reader segments it. These are the
// questions asked first whenever Formal AI fails a delegated task (see
// README.md in this folder), so they live here rather than in throwaway files.
//
// Usage:
//   node experiments/formal_ai_subagent/probe.mjs plan "<request>" [--tools read,write,edit,bash]
//   node experiments/formal_ai_subagent/probe.mjs edit "<request>"
//   node experiments/formal_ai_subagent/probe.mjs path "<request>"
//   node experiments/formal_ai_subagent/probe.mjs quotes "<request>"
//   node experiments/formal_ai_subagent/probe.mjs read-parse <file> <dir> <relative-path>
//
// `plan` defaults to the link-assistant Agent CLI tool list the dogfood
// driver advertises; pass --tools to try a reduced list.

import { readFileSync } from 'node:fs';

import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { AGENT_CLI_TOOLS, execute } from '../js_dogfood/drive.mjs';

await installNodeHost(new WorkerHost());

const [mode, ...rest] = process.argv.slice(2);
const toolsAt = rest.indexOf('--tools');
const tools = toolsAt >= 0 ? rest.splice(toolsAt, 2)[1].split(',') : AGENT_CLI_TOOLS;
const [request] = rest;

switch (mode) {
  case 'plan': {
    const { planChatStep } = await import('../../js/agentic/planner.mjs');
    console.log(JSON.stringify(await planChatStep([{ role: 'user', content: request }], tools), null, 2));
    break;
  }
  case 'edit': {
    const { composeEditRequest } = await import('../../js/agentic/write_request.mjs');
    console.log(JSON.stringify(composeEditRequest(request)));
    break;
  }
  case 'path': {
    const { firstPath } = await import('../../js/agentic/crate/capability_routing.mjs');
    console.log(JSON.stringify(firstPath(request)));
    break;
  }
  case 'quotes': {
    const { quotedSegmentSpans } = await import('../../js/agentic/crate/normal_markov.mjs');
    console.log(JSON.stringify(quotedSegmentSpans(request)));
    break;
  }
  case 'read-parse': {
    // Does the planner parse back what the Agent CLI's `read` returns?
    const [file, dir, relative] = rest;
    const { sourceFromReadResult } = await import('../../js/agentic/code_artifact.mjs');
    const result = execute(dir, { tool: 'read', arguments: JSON.stringify({ filePath: relative }) });
    const source = sourceFromReadResult(result);
    const actual = readFileSync(file, 'utf8');
    console.log(JSON.stringify({ parsed: source === null ? null : source.length, actual: actual.length, same: source === actual }));
    break;
  }
  default:
    console.error('usage: probe.mjs plan|edit|path|quotes|read-parse ...');
    process.exit(2);
}
process.exit(0);
