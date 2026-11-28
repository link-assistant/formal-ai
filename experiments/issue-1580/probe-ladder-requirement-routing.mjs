import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { recordReadbackCommand } from '../../js/agentic/evidence_record/record_observation.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

const [, , tracePath, mode] = process.argv;
if (!tracePath || (mode && !['--clauses', '--scoped', '--targets', '--composite'].includes(mode))) {
  throw new Error('Usage: node probe-ladder-requirement-routing.mjs <decoded-formal-ai-trace> [--clauses|--scoped|--targets|--composite]');
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

// Execute only declared fixture paths and their read-only verification commands.
async function replayBoundedRequest(content, sources, outputs, plan, maxCalls) {
  const { executeResult } = await import('../js_dogfood/drive.mjs');
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-scoped-requirement-'));
  const messages = [{ role: 'user', content }];
  const calls = [];
  const paths = new Set([...Object.keys(sources), ...outputs]);
  const commands = new Map([...paths].flatMap(path => [
    ['sha256sum -- ' + path, path], [recordReadbackCommand(path), path],
  ]));
  let blocked = null;
  try {
    for (const [path, before] of Object.entries(sources)) {
      const file = join(directory, path);
      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, before);
    }
    for (let step = 0; step < maxCalls; step++) {
      if (plan?.kind !== 'tool_calls' || plan.calls.length !== 1) break;
      const [call] = plan.calls;
      const args = JSON.parse(call.arguments);
      const path = call.tool === 'bash' ? commands.get(args.command) : args.filePath ?? args.file_path ?? args.path;
      if (!['bash', 'read', 'write', 'edit'].includes(call.tool) || !paths.has(path)) {
        blocked = { tool: call.tool, path: path ?? null, command: args.command ?? null };
        break;
      }
      const id = 'scoped-clause-' + step;
      const receipt = executeResult(directory, call);
      calls.push({ tool: call.tool, path, isError: receipt.is_error === true });
      messages.push(
        { role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
        { role: 'tool', tool_call_id: id, name: call.tool, ...receipt },
      );
      plan = await planChatStepResolved(messages, tools);
    }
    const files = Object.fromEntries([...paths].filter(path => existsSync(join(directory, path)))
      .map(path => [path, readFileSync(join(directory, path), 'utf8')]));
    return { calls, files, blocked, next: plan?.kind ?? null, final: plan?.kind === 'final' ? plan.result : null };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

if (mode === '--composite') {
  const expectedSourceEffects = {
    'rust/src/web_search_core.rs': '"wikiquote"',
    'rust/src/thinking.rs': '"sensor_fusion"',
    'rust/src/seed/meanings/parse.rs': '"provenance"',
    'data/seed/meanings-verifiable-task.lino': '"alternating"',
  };
  const outputs = ['agent-ladder-effects/node-1.1.1.lino', '.agent-ladder/node-1.1.1-proof.md'];
  const sources = Object.fromEntries(Object.keys(expectedSourceEffects).map(path => [path, readFileSync(path, 'utf8')]));
  const plan = await planChatStepResolved([{ role: 'user', content: request }], tools);
  const { files, ...replay } = await replayBoundedRequest(request, sources, outputs, plan, 24);
  const effects = Object.fromEntries(Object.entries(expectedSourceEffects).map(([path, needle]) =>
    [path, (files[path]?.split(needle).length ?? 0) === sources[path].split(needle).length + 1]));
  const seedPath = 'data/seed/meanings-verifiable-task.lino';
  effects[seedPath] &&= files[seedPath] === sources[seedPath].replace('"continue this"', '"alternating"');
  const delivered = Object.fromEntries(outputs.map(path => [path, files[path] ?? null]));
  const grounded = outputs.every(path => Object.entries(expectedSourceEffects)
    .every(([source, needle]) => delivered[path]?.includes(source) && delivered[path].includes(needle.slice(1, -1))));
  const verified = replay.final?.disposition === 'finding' && grounded
    && delivered[outputs[0]].startsWith('node_path=1.1.1\nnode_depth=3\nnode_kind=requirement\nresult=')
    && delivered[outputs[1]].startsWith('node_path=1.1.1\n')
    && Object.values(effects).every(Boolean);
  process.stdout.write(JSON.stringify({ ...replay, effects, delivered, grounded, verified }, null, 2) + '\n');
  process.exit(verified ? 0 : 1);
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
        const before = readFileSync(scopedPath, 'utf8');
        const { files, calls, final } = await replayBoundedRequest(content, { [scopedPath]: before }, [], plan, 6);
        const previousLines = new Set(before.split('\n'));
        const changedLines = [...new Set(files[scopedPath].split('\n').filter(line => !previousLines.has(line)))];
        replay = { calls, changedLines, final };
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
