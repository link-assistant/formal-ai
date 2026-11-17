// Issue #1162: print the program recipe (files, commands) per language so the
// build-output location and workflow can be inspected.
// Usage: node experiments/issue-1162/print-recipes.mjs [language...]
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import { attach, requestedIn } from '../../js/agentic/ci_workflow.mjs';

await installNodeHost(new WorkerHost());
const languages = process.argv.slice(2).length ? process.argv.slice(2) : ['Rust', 'Kotlin', 'Scala'];
for (const language of languages) {
  const prompt = `Write a ${language} program that prints "Hello, World!" to stdout. Create a GitHub Actions workflow that runs it on every push and pull request.`;
  const recipe = programContractAnswer(prompt)?.execution_recipe;
  if (!recipe) { console.log(`== ${language}: no recipe`); continue; }
  if (requestedIn(prompt)) attach(recipe, prompt);
  console.log(`== ${language}: ${recipe.path}\ncommands: ${JSON.stringify(recipe.commands)}`);
  for (const file of recipe.supporting_files) console.log(`--- ${file.path}\n${file.source}`);
}
