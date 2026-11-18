// Issue #1562: show the recipe the JS work-item solver selects for a text and
// for its obligation text, and the binding decision.
// Usage: node experiments/issue-1562/binding-of-text.mjs <file>...
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';

await ensureNodeHost({ worker: new WorkerHost() }, { sourceUsageRustGrammar: false });
const { programContractAnswer } = await import('../../js/agentic/crate/coding_program_contract.mjs');
const { bindingText, unbindCitedRecipe } = await import('../../js/agentic/work_item_citations.mjs');

const recipeOf = text => {
  const recipe = programContractAnswer(text)?.execution_recipe;
  return recipe ? `${recipe.path} (${recipe.language})` : null;
};
for (const path of process.argv.slice(2)) {
  const text = readFileSync(path, 'utf8');
  const obligations = bindingText(text);
  const answer = programContractAnswer(text);
  console.log(JSON.stringify({ path, whole: recipeOf(text), obligations: recipeOf(obligations), unbound: answer ? unbindCitedRecipe(answer, text) : null }));
  console.log(obligations.replace(/ {2,}/g, '·'));
}
process.exit(0);
