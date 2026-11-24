import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

const [, , tracePath, mode] = process.argv;
if (!tracePath || (mode && !['--clauses', '--scoped', '--targets'].includes(mode))) {
  throw new Error('Usage: node probe-ladder-requirement-routing.mjs <decoded-formal-ai-trace> [--clauses|--scoped|--targets]');
}
const trace = readFileSync(tracePath, 'utf8');
const match = /^\s*\d+ \[trace\] agentic_received=([\s\S]+?)(?=^\s*\d+ \[trace\])/mu.exec(trace);
if (!match) throw new Error('The decoded trace has no original agentic_received event');

await installNodeHost(new WorkerHost());
const { planChatStepResolved } = await import('../../js/agentic/planner.mjs');
const { composeEditRequest } = await import('../../js/agentic/write_request.mjs');
const request = match[1].trim();
const tools = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read',
  'task', 'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];

// Replay a single source-bound clause in a disposable copy of the file. Limit
// the driver to this one file and its ordinary digest verification command.
async function replayScopedClause(content, path, before, plan) {
  const { executeResult } = await import('../js_dogfood/drive.mjs');
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-scoped-requirement-'));
  const file = join(directory, path);
  const messages = [{ role: 'user', content }];
  const calls = [];
  try {
    mkdirSync(dirname(file), { recursive: true });
    writeFileSync(file, before);
    for (let step = 0; step < 6; step++) {
      if (plan?.kind !== 'tool_calls' || plan.calls.length !== 1) break;
      const [call] = plan.calls;
      const args = JSON.parse(call.arguments);
      const destination = args.filePath ?? args.file_path ?? args.path;
      const allowed = call.tool === 'bash'
        ? args.command === 'sha256sum -- ' + path
        : ['read', 'write', 'edit'].includes(call.tool) && destination === path;
      if (!allowed) break;
      const id = 'scoped-clause-' + step;
      const receipt = executeResult(directory, call);
      calls.push({ tool: call.tool, path, isError: receipt.is_error === true });
      messages.push(
        { role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
        { role: 'tool', tool_call_id: id, name: call.tool, ...receipt },
      );
      plan = await planChatStepResolved(messages, tools);
    }
    const after = readFileSync(file, 'utf8');
    const previousLines = new Set(before.split('\n'));
    const changedLines = [...new Set(after.split('\n').filter(line => !previousLines.has(line)))];
    return { calls, changedLines, final: plan?.kind === 'final' ? plan.result : null };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

if (mode) {
  const { resolveRequirementTarget } = await import('../../js/agentic/requirement_resolution.mjs');
  const { censusSourceClosureComplete, generatedSeedOwnerDeclarations } = await import('../../js/agentic/node-host.mjs');
  const clauses = request.split('\n\n', 1)[0].split(/\s+\(\d+\)\s+/u).slice(1);
  if (!clauses.length) throw new Error('The captured first paragraph has no numbered requirements');
  process.stdout.write(JSON.stringify({ censusComplete: censusSourceClosureComplete(), count: clauses.length }) + '\n');
  for (const clause of clauses) {
    const target = resolveRequirementTarget(clause);
    if (mode === '--targets') {
      process.stdout.write(JSON.stringify({ clause, target }) + '\n');
      continue;
    }
    // Compare the original clause with explicit literal boundaries.
    const quoted = clause
      .replace(/^Add ([a-z_\d]+)(\s+(?:as|to)\s)/u, 'Add "$1"$2')
      .replace(/ from (.+?) to ([^.]+)\.$/u, ' from "$1" to "$2".');
    let inputs = [...new Set([clause, quoted])];
    let scopedPath = null;
    if (mode === '--scoped') {
      const member = /^Add ([a-z_\d]+)\s+(?:as|to)\s/u.exec(clause);
      const replacement = /^Change .+ from ([\w ]+) to ([\w ]+)\.$/u.exec(clause);
      const owner = target && replacement && generatedSeedOwnerDeclarations().find(candidate =>
        candidate.module_path === target.module_path && candidate.symbol === target.symbol);
      if (target && member) {
        scopedPath = 'rust/' + target.module_path;
        inputs = ['Edit the tracked file `' + scopedPath + '`: add ' +
          JSON.stringify(member[1]) + ' to the ' + target.symbol + ' list.'];
      } else if (owner) {
        scopedPath = owner.seed_path;
        inputs = ['In the file ' + scopedPath + ', replace ' + JSON.stringify(replacement[1]) +
          ' with ' + JSON.stringify(replacement[2]) + '.'];
      } else {
        inputs = [];
      }
    }
    const routes = [];
    for (const content of inputs) {
      const plan = await planChatStepResolved([{ role: 'user', content }], tools);
      let replay;
      if (scopedPath !== null) {
        if (!/^(?:rust\/src|data\/seed)\/(?:[a-zA-Z0-9_-]+\/)*[a-zA-Z0-9_-]+\.(?:rs|lino)$/u.test(scopedPath)) {
          throw new Error('Unsupported source path for scoped replay: ' + scopedPath);
        }
        replay = await replayScopedClause(content, scopedPath, readFileSync(scopedPath, 'utf8'), plan);
      }
      routes.push({ content, edit: composeEditRequest(content), plan, replay });
    }
    process.stdout.write(JSON.stringify({ clause, target, routes }) + '\n');
  }
  process.exit(0);
}

const plan = await planChatStepResolved([{ role: 'user', content: request }], tools);
const edit = composeEditRequest(request);
process.stdout.write(`${JSON.stringify({ requestLength: request.length, edit, plan }, null, 2)}\n`);
