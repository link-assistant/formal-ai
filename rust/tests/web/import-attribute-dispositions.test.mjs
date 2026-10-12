// Metadata coverage is separate from native grammar parsing/projection behavior.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
const read = path => readFileSync(new URL('../../../' + path, import.meta.url), 'utf8');
const seed = read('data/seed/grammar-projection-rules.lino');
test('canonical and deploy/embedded metadata mirrors retain every exact seed byte', () => {
  assert.equal(read('rust/embedded/data/seed/grammar-projection-rules.lino'), seed);
  assert.equal(read('js/seed/grammar-projection-rules.lino'), seed);
  const ids = [...seed.matchAll(/^\((\d+):/gmu)].map(match => match[1]);
  assert.equal(ids.length, new Set(ids).size); assert.equal(ids.at(-1), '628');
});
test('import parent and attribute child are explicitly Rust no-form, never a splice or rendering rule', () => {
  for (const kind of ['import_statement', 'import-attribute']) {
    const rows = seed.split('\n').filter(row => row.includes('(term: ' + kind + ')'));
    assert.equal(rows.length, 1); assert.ok(rows[0].includes('(def: rust) (lang: grammar-projection-noform)'));
  }
  assert.ok(seed.includes('(627: 1 (meta: (t: semantic) (n: 1) (term: object_assignment_pattern)'));
  assert.ok(seed.includes('(628: 1 (meta: (t: semantic) (n: 1) (term: import-attribute)'));
});
test('authored generator preserves existing final disposition order and the new child identity', () => {
  const generator = read('rust/examples/generate_grammar_projection_rules/main.rs');
  const body = generator.split('const NO_FORM_KINDS:')[1].split('\n];')[0];
  assert.match(body, /\("object_assignment_pattern", &\["rust"\]\),\s*\("import-attribute", &\["rust"\]\),\s*$/u);
  assert.equal([...body.matchAll(/\("import-attribute",/gu)].length, 1);
});

test('existing disposition declarations have no conflicting canonical spellings or duplicate targets', () => {
  const declarations = new Map();
  for (const match of seed.matchAll(/\(term: ([^)]+)\) \(def: ([^)]+)\) \(lang: grammar-projection-(refusal|noform)\)/gu)) {
    const [, spelling, target] = match, canonical = /\p{L}/u.test(spelling) ? spelling.replaceAll('_', '-') : spelling;
    const prior = declarations.get(canonical) ?? { spelling, targets: new Set() };
    assert.equal(prior.spelling, spelling); assert.equal(prior.targets.has(target), false);
    assert.equal(prior.targets.has('any'), false); assert.ok(target !== 'any' || prior.targets.size === 0);
    prior.targets.add(target); declarations.set(canonical, prior);
  }
  assert.ok(declarations.has('_')); assert.ok(declarations.has('-'));
  assert.ok(declarations.has('import-attribute')); assert.ok(declarations.has('import-statement'));
});
