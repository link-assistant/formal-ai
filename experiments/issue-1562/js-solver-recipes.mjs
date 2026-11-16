// Issue #1562: the recipe the JS work-item solver selects for each text, and
// whether the citation binding keeps it.
// Usage: node experiments/issue-1562/js-solver-recipes.mjs <file>...
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';

await ensureNodeHost({ worker: new WorkerHost() }, { sourceUsageRustGrammar: false });
const { programContractAnswer } = await import('../../js/agentic/crate/coding_program_contract.mjs');
const { unbindCitedRecipe } = await import('../../js/agentic/work_item_citations.mjs');
for (const file of process.argv.slice(2)) {
  const text = readFileSync(file, 'utf8');
  const answer = programContractAnswer(text);
  const recipe = answer?.execution_recipe ?? null;
  const unbound = answer ? unbindCitedRecipe(answer, text) : null;
  console.log(file, recipe ? `${recipe.path} ${recipe.language}` : 'no recipe', 'unbound:', unbound);
}
process.exit(0);
