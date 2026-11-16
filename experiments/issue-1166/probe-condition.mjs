// Probe: what the program contract and surface structure read from conditional and repeated output prompts.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { explicitStdout, programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import { surfaceStructure } from '../../js/agentic/crate/proposition_structure.mjs';
await installNodeHost(new WorkerHost());
const prompts = process.argv.slice(2).length ? process.argv.slice(2) : [
  'Write a Python program that takes x and y as arguments; when x is greater than y, print "big", otherwise print "small".',
  'Write a Python program that prints "Hi" twice.',
];
for (const prompt of prompts) {
  const answer = programContractAnswer(prompt);
  console.log(JSON.stringify(prompt), '=>', JSON.stringify(explicitStdout(prompt)), answer?.execution_recipe?.language, answer?.obligation_gaps);
}
for (const atom of ['x is greater than y', 'x > y', 'y < x', 'x is not less than y', 'x is less than or equal to y', 'x equals y', 'x is at most y', 'x is smaller than y', 'x exceeds y'])
  console.log(atom, JSON.stringify(surfaceStructure(atom)));
