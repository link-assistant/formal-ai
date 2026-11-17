// Issue #1162: the recipe each real test-repository issue body yields:
// program path, workflow path, commands and the expected stdout.
// Usage: node experiments/issue-1162/issue-body-recipes.mjs ISSUE.md...
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import { attach, requestedIn } from '../../js/agentic/ci_workflow.mjs';

await installNodeHost(new WorkerHost());
for (const file of process.argv.slice(2)) {
  const prompt = readFileSync(file, 'utf8');
  const recipe = programContractAnswer(prompt)?.execution_recipe;
  if (!recipe) { console.log(`${file}: no recipe`); continue; }
  const wants = requestedIn(prompt);
  if (wants) attach(recipe, prompt);
  console.log(JSON.stringify({ file, program: recipe.path, workflowRequested: wants, files: recipe.supporting_files.map(f => f.path), commands: recipe.commands }));
}
