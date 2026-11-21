import assert from 'node:assert/strict';
import { performance } from 'node:perf_hooks';
import { compileRecipe, runRecipe } from '../../js/server/dynamic-meta/recipe.mjs';

const rules = Array.from({ length: 120 }, (_, index) => ({
  pattern: `word ${index.toString(36)} suffix`, replacement: 'match', terminal: true,
}));
const program = { rules };
const compiled = compileRecipe(program);
const repeatedPrefix = 'word '.repeat(10_000);

for (const [label, input] of [['repeated prefix misses', repeatedPrefix], ['first rule hits', `word 0 suffix ${repeatedPrefix}`]]) {
  assert.deepEqual(runRecipe(program, input, { compiled }), runRecipe(program, input));
  for (const [mode, dispatchTable] of [['dynamic', undefined], ['compiled', compiled]]) {
    for (let iteration = 0; iteration < 10; iteration += 1) runRecipe(program, input, { compiled: dispatchTable });
    const started = performance.now();
    for (let iteration = 0; iteration < 50; iteration += 1) runRecipe(program, input, { compiled: dispatchTable });
    console.log(`${label}, ${mode}, ${(performance.now() - started).toFixed(1)} ms`);
  }
}
