import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';

const [, , tracePath, mode] = process.argv;
if (!tracePath || (mode && mode !== '--clauses')) throw new Error('Usage: node probe-ladder-requirement-routing.mjs <decoded-formal-ai-trace> [--clauses]');
const trace = readFileSync(tracePath, 'utf8');
const match = /^\s*\d+ \[trace\] agentic_received=([\s\S]+?)(?=^\s*\d+ \[trace\])/mu.exec(trace);
if (!match) throw new Error('The decoded trace has no original agentic_received event');

await installNodeHost(new WorkerHost());
const { planChatStepResolved } = await import('../../js/agentic/planner.mjs');
const { composeEditRequest } = await import('../../js/agentic/write_request.mjs');
const request = match[1].trim();
const tools = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read',
  'task', 'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];
if (mode === '--clauses') {
  const { resolveRequirementTarget } = await import('../../js/agentic/requirement_resolution.mjs');
  const { censusSourceClosureComplete } = await import('../../js/agentic/node-host.mjs');
  const clauses = request.split('\n\n', 1)[0].split(/\s+\(\d+\)\s+/u).slice(1);
  if (!clauses.length) throw new Error('The captured first paragraph has no numbered requirements');
  process.stdout.write(JSON.stringify({ censusComplete: censusSourceClosureComplete(), count: clauses.length }) + '\n');
  for (const clause of clauses) {
    // Compare the original clause with explicit literal boundaries. This probe
    // only asks for the first plan and never executes a tool.
    const quoted = clause
      .replace(/^Add ([a-z_\d]+)(\s+(?:as|to)\s)/u, 'Add "$1"$2')
      .replace(/ from (.+?) to ([^.]+)\.$/u, ' from "$1" to "$2".');
    const target = resolveRequirementTarget(clause);
    const routes = [];
    for (const content of [...new Set([clause, quoted])]) {
      const plan = await planChatStepResolved([{ role: 'user', content }], tools);
      routes.push({ content, edit: composeEditRequest(content), plan });
    }
    process.stdout.write(JSON.stringify({ clause, target, routes }) + '\n');
  }
  process.exit(0);
}

const plan = await planChatStepResolved([{ role: 'user', content: request }], tools);
const edit = composeEditRequest(request);
process.stdout.write(`${JSON.stringify({ requestLength: request.length, edit, plan }, null, 2)}\n`);
