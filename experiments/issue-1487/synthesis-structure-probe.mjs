// Issue #1487: what surface structure does a coding prompt keep, and which
// coding-structure meanings does the worker discover for it?
import { installTextHost } from '../../scripts/lib/text-capability-measures.mjs';
installTextHost();
const { surfaceStructure } = await import('../../js/agentic/crate/proposition_structure.mjs');
const prompts = process.argv.slice(2).length ? process.argv.slice(2) : [
  'Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.',
  'Implement Python function count_vowels(text: str) -> int. Return twice the number of vowels in the text.',
  'Implement Python function count_vowels(text: str) -> int. Return the number of characters that are not vowels in the text.',
  'Write a Python function is_small(x) that returns True when x < 10.',
  'Write a Python function is_small(x) that returns True when x <= 10.',
  'Write a Python function both(a, b) that returns a and b.',
  'Write a Python function either(a, b) that returns a or b.',
];
for (const prompt of prompts) console.log(JSON.stringify(prompt), JSON.stringify(surfaceStructure(prompt)));
