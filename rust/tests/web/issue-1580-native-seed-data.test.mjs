import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { Parser } from 'links-notation';
import { childValue, parseLino } from '../../../js/server/lino.mjs';
import { WorkerHost, evaluate } from '../../../js/server/worker-host.mjs';

const seed = name => readFileSync(new URL(`../../../data/seed/${name}.lino`, import.meta.url), 'utf8');

test('the executable language seed is canonical notation and retains the exact R check command', () => {
  const source = seed('hello-world-programs');
  assert.doesNotThrow(() => new Parser().parse(source));
  const program = parseLino(source).children.find(node => node.name === 'write_program');
  const language = program.children.find(node => node.name === 'target' && node.value === 'r');
  assert.equal(childValue(language, 'check_command'), `Rscript -e 'invisible(parse("hello.R"))'`);
  const renderer = program.children.find(node => node.name === 'print_renderer' && node.value === 'r');
  assert.equal(childValue(renderer, 'syntax'), String.raw`cat({literal}, "\n", sep = "")`);
});

test('handler table and policy operands are reference lists rather than packed strings', () => {
  const source = seed('handler-rules');
  assert.doesNotThrow(() => new Parser().parse(source));
  assert.equal(source.includes('|'), false, 'handler operands must use reference lists');
});

test('the actual handler readers and their consumers retain every grouped reference', async () => {
  const context = await new WorkerHost().boot();
  const value = expression => JSON.parse(evaluate(context, `JSON.stringify(${expression})`));
  assert.deepEqual(value(`handlerRulesTokenize('row sample ("a b" "c")')`),
    ['row', 'sample', '("a b" "c")']);
  assert.deepEqual(value('documentationMethodProvider("pandas")'),
    { root: 'https://pandas.pydata.org/docs/', path: 'reference/api' });
  assert.deepEqual(value('documentationMethodProvider("numpy")'),
    { root: 'https://numpy.org/doc/stable/', path: 'reference/generated' });
  assert.deepEqual(value('howToNominalSearch({id:"wikibooks"}, "make tea", "en")'),
    { namespace: '102', prefix: 'Cookbook:', subject: 'tea' });
  assert.deepEqual(value('crateModule("crate/seed.mjs").splitPipeList(handlerRulesPolicy("procedural_how_to", "ordered-section-labels"))'),
    ['procedure', 'instructions', 'steps', 'method']);
  const codes = value('crateModule("crate/seed.mjs").splitPipeList(handlerRulesTableValue("currency-quote-provider", "iso-codes"))');
  assert.equal(codes[0], 'AED');
  assert.equal(codes.at(-1), 'ZWG');
  assert.equal(new Set(codes).size, codes.length);
  for (const code of ['AUD', 'CAD', 'EUR', 'INR', 'JPY', 'USD', 'ZWG']) {
    assert(codes.includes(code), code);
    assert.equal(value(`currencyCodeFromWord(${JSON.stringify(code)})`), code);
  }
});
