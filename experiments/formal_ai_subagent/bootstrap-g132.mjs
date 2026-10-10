import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { digest, admitManifest, executeAdmittedCase } from './cohort-runner.mjs';

// Operational bootstrap only. One unchanged ask, seven behavioral obligations.
// Fixture materialization is supplied work and receives zero autonomous credit.
const root = resolve(process.argv[2] ?? process.cwd());
const output = resolve(process.argv[3]);
mkdirSync(output);
const workspace = join(output, 'workspace'); mkdirSync(workspace);
const requestPath = join(root, 'experiments/formal_ai_subagent/evidence/formal-ai-only-1188/g132/T1804-request.json');
const original = JSON.parse(readFileSync(requestPath, 'utf8'));
const originalOracle = join(root, 'experiments/formal_ai_subagent/evidence/formal-ai-only-1188/module-observation/original-selected-summary.test.mjs');
for (const name of ['solver_formalization.mjs', 'translation_formalization.mjs']) {
  copyFileSync(join(root, 'js/agentic/crate', name), join(workspace, name));
}
copyFileSync(originalOracle, join(workspace, 'selected-summary.test.mjs'));
const binding = path => ({ path, sha256: digest(readFileSync(path)) });
const manifest = {
  schemaVersion: 1, cohortId: 'g132-original-one-task-bootstrap',
  bindings: [requestPath, originalOracle, fileURLToPath(import.meta.url),
    join(root, 'experiments/js_dogfood/drive.mjs'), join(root, 'js/agentic/planner.mjs'),
    join(root, 'js/agentic/crate/solver_formalization.mjs'),
    join(root, 'js/agentic/crate/translation_formalization.mjs'),
    join(workspace, 'selected-summary.test.mjs')].map(binding),
  cases: [{ runId: 'g132-original', taskKind: 'self-coding', category: 'feature-implementation',
    expectedRelation: 'output-larger', task: original.prompt, taskSHA256: digest(original.prompt),
    workspace, allowedEffects: [join(workspace, 'repair-summary.mjs'), join(workspace, '.formal-ai/general-change-plan.lino')],
    oracle: binding(originalOracle), timeoutMilliseconds: 1000 }],
};
writeFileSync(join(output, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
const admission = admitManifest(manifest, join(output, 'journal.jsonl'));
const load = path => import(pathToFileURL(join(root, path)).href);
const { drive } = await load('experiments/js_dogfood/drive.mjs');
const { WorkerHost } = await load('js/server/worker-host.mjs');
const { installNodeHost } = await load('js/agentic/node-host.mjs');
await installNodeHost(new WorkerHost());
const { planChatStep } = await load('js/agentic/planner.mjs');
const { solve } = await load('js/agentic/host.mjs');
const { planSymbolicCommandReroute } = await load('js/agentic/command_reroute.mjs');
const { latestUserRequest } = await load('js/agentic/content.mjs');
const fallthrough = async (messages, tools) => {
  const symbolic = await solve(latestUserRequest(messages) ?? '', []);
  return planSymbolicCommandReroute(messages, tools, symbolic) ?? { kind: 'final', answer: symbolic.answer };
};
process.env.SELECTED_SUMMARY_MODULE = join(workspace, 'repair-summary.mjs');
const result = await executeAdmittedCase(admission, 'g132-original', 'unchanged-original',
  (task, directory) => drive(planChatStep, directory, task, { steps: 24, fallthrough,
    allowedCommands: ['node --test selected-summary.test.mjs'] }));
writeFileSync(join(output, 'result.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ accepted: result.accepted, stop: result.stop, failure: result.failure,
  tools: result.transcript?.length, usage: result.usage, originalTaskBytes: Buffer.byteLength(original.prompt),
  attemptedTasks: 1, behavioralObligations: 7, representativeBaseline: false }));
process.exit(0);
