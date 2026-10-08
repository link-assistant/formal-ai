#!/usr/bin/env node
// Live JavaScript callers of the shared repository protocol.
// node js/server/repository-workspace-command-line.mjs solve|swe-bench|coding-ladder --workspace DIR --task FILE
// node js/server/repository-workspace-command-line.mjs authoring --task FILE
// Authoring JSON declares repository, scratch workspace, task, produced files and evidence;
// its optional commit field is false unless explicitly true.
import fs from 'node:fs';
import { pathToFileURL } from 'node:url';
import { WorkerHost } from './worker-host.mjs';
import { installNodeHost } from '../agentic/node-host.mjs';
import { runRepositoryCase } from './repository-workspace.mjs';
import { runNodeAuthoring } from './repository-authoring.mjs';

export async function runRepositoryCommandLine(argumentsList, { install = async () => installNodeHost(new WorkerHost()) } = {}) {
  const [action, ...flags] = argumentsList;
  const caller = { solve: 'solve', 'swe-bench': 'swe_bench', 'coding-ladder': 'coding_ladder' }[action];
  if (!caller && action !== 'authoring') throw new Error('expected solve, swe-bench, coding-ladder or authoring');
  const options = {};
  for (let position = 0; position < flags.length; position += 2) {
    if (!['--workspace', '--task'].includes(flags[position]) || !flags[position + 1]) throw new Error('invalid repository caller option');
    options[flags[position].slice(2)] = flags[position + 1];
  }
  if (!options.task || (caller && !options.workspace)) throw new Error('--task and structural --workspace are required');
  const task = JSON.parse(fs.readFileSync(options.task, 'utf8'));
  await install();
  const outcome = caller ? await runRepositoryCase(options.workspace, task, { caller }) : await runNodeAuthoring(task);
  return { code: outcome.stopped_at ? 1 : 0, outcome };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const result = await runRepositoryCommandLine(process.argv.slice(2));
    process.stdout.write(JSON.stringify(result.outcome, null, 2) + '\n');
    process.exitCode = result.code;
  } catch (error) {
    process.stderr.write(error.message + '\n'); process.exitCode = 1;
  }
}
