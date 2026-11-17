// Issue #1162 acceptance item 5: how the current recipe answers variants of the
// Hello World request. Prints path, stdout literal, workflow path per variant.
import fs from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import { attach, requestedIn } from '../../js/agentic/ci_workflow.mjs';

await installNodeHost(new WorkerHost());
const base = fs.readFileSync('rust/tests/fixtures/issue-1162/hello-world-kotlin-issue.md', 'utf8');
const variants = {
  original: base,
  changedString: base.replaceAll('Hello, World!', 'Hello, Formal AI!'),
  unicode: base.replaceAll('Hello, World!', 'Привет, мир! 🌍'),
  alternateFile: base.replace('Create a file with the appropriate extension for Kotlin', 'Create the file `Greeting.kt`'),
  paraphrased: 'Write a Kotlin program that outputs exactly `Hello, World!` and add a GitHub Actions workflow that runs it on every push and pull request and checks the output.',
  repeated: base + '\nAgain: the program must print exactly "Hello, World!". The output is "Hello, World!".\n',
  printTwice: 'Write a Kotlin program that first prints `Hello, World!` and then prints `Hello, World!` again. Add a GitHub Actions workflow that runs it on push.',
  scala: base.replaceAll('Kotlin', 'Scala'),
  rust: base.replaceAll('Kotlin', 'Rust'),
};
for (const [name, issue] of Object.entries(variants)) {
  const answer = programContractAnswer(issue);
  const recipe = answer?.execution_recipe;
  if (!recipe) { console.log(name, 'NO RECIPE', JSON.stringify(answer)?.slice(0, 300)); continue; }
  if (requestedIn(issue)) attach(recipe, issue);
  const verify = recipe.supporting_files.find(file => /verify/.test(file.path));
  const workflow = recipe.supporting_files.find(file => /\.ya?ml$/.test(file.path));
  console.log(JSON.stringify({ name, path: recipe.path, files: recipe.supporting_files.map(file => file.path),
    expected: verify?.source.match(/printf '[^']*' ('[^']*') > "\$verification_dir\/expected"/u)?.[1], workflow: workflow?.path,
    source: recipe.source.split('\n').filter(line => /print/i.test(line) && !line.startsWith('//')) }));
}
