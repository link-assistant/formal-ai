// Issue #1562: what the JS worker solver answers for each fixture's whole text
// (the reading the Rust work-item route gives the fetched issue).
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';

const worker = new WorkerHost();
for (const file of process.argv.slice(2)) {
  const text = readFileSync(file, 'utf8');
  const answer = await worker.solve(text);
  console.log(`=== ${file}`);
  console.log(' intent', answer?.intent, 'link', answer?.response_link ?? answer?.responseLink);
  const recipe = answer?.execution_recipe ?? answer?.executionRecipe;
  console.log(' recipe', recipe ? JSON.stringify(recipe).slice(0, 400) : null);
}
process.exit(0);
