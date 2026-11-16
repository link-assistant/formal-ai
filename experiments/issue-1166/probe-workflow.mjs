// Probe: which workflow path and triggers the executor writes for the #1450 body.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import * as CI from '../../js/agentic/ci_workflow.mjs';
import { programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import { requestTargetBinding, validWorkflowTarget } from '../../js/agentic/git_commit.mjs';
await installNodeHost(new WorkerHost());
const body = readFileSync(process.argv[2] ?? new URL('./issue-1450-body.md', import.meta.url), 'utf8');
console.log('binding', JSON.stringify(requestTargetBinding(body, 'workflow-binding-patterns', validWorkflowTarget)));
const answer = programContractAnswer(body);
const recipe = answer.execution_recipe;
console.log('program', recipe.path, recipe.language, JSON.stringify(recipe.commands));
console.log(recipe.source);
CI.attach(recipe, body);
for (const f of recipe.supporting_files) { console.log('---', f.path); console.log(f.source); }
console.log('gaps', JSON.stringify(answer.obligation_gaps, null, 1));
