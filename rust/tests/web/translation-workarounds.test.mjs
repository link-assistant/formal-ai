// The temporary workarounds of the js -> rust translation (R1188-U30, R994).
// Where the pinned link-foundation/meta-language cannot yet translate, a
// recorded workaround in this repository completes the translation, and it is
// retired when the upstream issue its record names is fixed. This suite pins
// the record (data/meta/translation-workarounds.lino) to what the projections
// hold: every rule a projection applies is recorded with its upstream issue,
// every recorded rule is in use and covered by a call of calls.lino, and every
// `use` an import translates to names an item its module's translation has.
// It also pins each mechanism on small inputs, and the full-blocker scan
// (data/meta/translation-blockers.lino) that chose them.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  BLOCKERS_FILE, CALLS_FILE, LEDGER_FILE, parseCalls, parseLedger, projectionPath, readProjection, scopeModules,
  upstreamBlocks,
} from '../../../scripts/translate-js-rust.mjs';
import {
  WORKAROUNDS_FILE, assembleCrate, importGraph, namedImports, pruneImport, readWorkarounds, rerootModule, resolveImport,
  translationClosure,
} from '../../../scripts/lib/translation-workarounds.mjs';
import { itemBlockers, moduleContext, portableType, unlockOrder } from '../../../scripts/lib/translation-blockers.mjs';

const REPOSITORY = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const read = (path) => readFileSync(join(REPOSITORY, path), 'utf8');
const sha256 = (text) => createHash('sha256').update(text, 'utf8').digest('hex');
const records = readWorkarounds(read(WORKAROUNDS_FILE));
const projections = scopeModules(REPOSITORY).map((path) => ({ path, file: projectionPath(path), text: read(projectionPath(path)) }));

test('every workaround is recorded with the refusal it lifts, its upstream issue, its scope and how it works', () => {
  assert.deepEqual(records.map((record) => record.name), ['import-pruning', 'crate-assembly']);
  for (const record of records) {
    assert.ok(record.lifts && record.how, `${record.name} names what it lifts and how`);
    assert.match(record.upstream, /^https:\/\/github\.com\/link-foundation\/(?:meta-language|relative-meta-logic)\/issues\/\d+$/u, `${record.name} names the upstream issue that retires it`);
    assert.ok(['projection', 'compile'].includes(record.scope), `${record.name} acts on the projection or the compile check`);
  }
});

test('the projections apply only recorded workarounds, and every projection-scope workaround is in use', () => {
  const recorded = new Set(records.map((record) => record.name));
  const used = new Set();
  for (const { file, text } of projections) {
    const read = readProjection(text);
    assert.equal(read.applied, read.expectedApplied, `${file}: the header names the workarounds its blocks hold`);
    for (const [, rule] of text.matchAll(/^\/\/ formal-ai:workaround (\S+) /gmu)) {
      assert.ok(recorded.has(rule), `${file} applies the unrecorded workaround ${rule}`);
      used.add(rule);
    }
  }
  for (const record of records.filter((entry) => entry.scope === 'projection')) {
    assert.ok(used.has(record.name), `${record.name} is recorded but no projection applies it; retire the record`);
  }
});

test('the ledger counts workaround items apart from the items meta-language translated', () => {
  const ledger = parseLedger(read(LEDGER_FILE));
  let upstream = 0;
  let workaround = 0;
  for (const { text } of projections) {
    upstream += (text.match(/^\/\/ meta-language:translated /gmu) ?? []).length;
    workaround += (text.match(/^\/\/ formal-ai:workaround \S+ (?!carried )/gmu) ?? []).length;
  }
  assert.equal(ledger.totals.translated_items, upstream);
  assert.equal(ledger.totals.workaround_items, workaround);
  assert.equal(ledger.workarounds.reduce((sum, row) => sum + row.count, 0), workaround);
});

test('every recorded workaround is covered by a call of calls.lino', () => {
  const calls = parseCalls(read(CALLS_FILE));
  const holding = (rule) => projections.filter(({ text }) => text.includes(`// formal-ai:workaround ${rule} `));
  for (const record of records) {
    const covered = record.scope === 'compile'
      // The compile check assembles every root as one crate, so every call runs through it.
      ? calls.length > 0
      : calls.some((call) => holding(record.name).some(({ path, text }) => path === call.module
        || new RegExp(`^// formal-ai:workaround ${record.name} [^\\n]*\\n(?:// \\|[^\\n]*\\n)*[^\\n]*\\b${call.rust}\\b`, 'mu').test(text)));
    assert.ok(covered, `${record.name}: no call of ${CALLS_FILE} runs through Rust it wrote`);
  }
});

test('every use an import translates to names an item its module translated', () => {
  const byModule = new Map(projections.map(({ file, text }) => [file, text]));
  let checked = 0;
  for (const { file, text } of projections) {
    for (const [line, module, list] of text.matchAll(/^use crate::(\w+)::(?:\{(.*)\}|(\w+(?: as \w+)?));$/gmu)
      .map((match) => [match[0], match[1], match[2] ?? match[3]])) {
      const target = byModule.get(`${file.slice(0, file.lastIndexOf('/'))}/${module}.rs`);
      assert.ok(target, `${file}: ${line} names no module of its root`);
      for (const entry of list.split(', ')) {
        const name = entry.split(' as ')[0];
        assert.match(target, new RegExp(`^pub (?:fn|const|static|struct|enum) ${name}\\b`, 'mu'), `${file}: ${line} names ${name}, which ${module}.rs does not define`);
        checked += 1;
      }
    }
  }
  assert.ok(checked > 0, 'at least one translated import is checked');
});

test('import-pruning keeps the bound names, and carries an import with none', () => {
  const code = 'use crate::m::{a, Big};';
  const block = {
    kind: 'translated',
    term: 'import_statement',
    source: "import { a, b, Big } from './m.mjs';",
    text: [`// meta-language:translated JavaScript import_statement items=1 sha256=${sha256('use crate::m::{a, b, Big};')}`, "// | import { a, b, Big } from './m.mjs';", 'use crate::m::{a, b, Big};'].join('\n'),
  };
  assert.equal(pruneImport(block, new Set(['a', 'b', 'Big'])), block);
  const pruned = pruneImport(block, new Set(['a', 'Big']));
  assert.equal(pruned.kind, 'workaround');
  assert.equal(pruned.text, [`// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=${sha256(code)}`, "// | import { a, b, Big } from './m.mjs';", code].join('\n'));
  // The pruned block reads back as a workaround block.
  const [parsed] = upstreamBlocks(`// meta-language:self-translation:v1 source=JavaScript\n\n${pruned.text}\n`).blocks;
  assert.deepEqual({ kind: parsed.kind, rule: parsed.rule, term: parsed.term }, { kind: 'workaround', rule: 'import-pruning', term: 'import_statement' });
  const single = pruneImport(block, new Set(['b']));
  assert.match(single.text, /^use crate::m::b;$/mu);
  const none = pruneImport(block, new Set());
  assert.deepEqual({ kind: none.kind, refusal: none.refusal }, { kind: 'carried', refusal: 'import of names its module does not translate' });
  assert.equal(none.marker, '// formal-ai:workaround import-pruning carried JavaScript import_statement');
});

test('the import graph resolves a module directory\'s relative named imports, and the closure adds importers and imports', () => {
  assert.deepEqual(namedImports("import { a, b as c } from './m.mjs';\nimport fs from 'node:fs';\n"), [
    { specifier: './m.mjs', names: [{ imported: 'a', local: 'a' }, { imported: 'b', local: 'c' }] },
  ]);
  const modules = new Set(['js/x/a.mjs', 'js/x/b.mjs', 'js/x/c.mjs', 'js/x/d.mjs']);
  assert.equal(resolveImport('js/x/a.mjs', './b.mjs', modules), 'js/x/b.mjs');
  assert.equal(resolveImport('js/x/a.mjs', '../host.mjs', modules), null);
  assert.equal(resolveImport('js/x/a.mjs', './missing.mjs', modules), null);
  const graph = importGraph(new Map([
    ['js/x/a.mjs', "import { f } from './b.mjs';"],
    ['js/x/b.mjs', "import { g } from './c.mjs';"],
    ['js/x/c.mjs', ''],
    ['js/x/d.mjs', "import { h } from './b.mjs';\nimport { k } from './e.mjs';"],
  ]));
  assert.deepEqual(graph.get('js/x/d.mjs'), ['js/x/b.mjs']);
  // c changes: b and a and d import it (through b), and translating them needs b's and c's signatures.
  assert.deepEqual(translationClosure(['js/x/c.mjs'], graph), ['js/x/a.mjs', 'js/x/b.mjs', 'js/x/c.mjs', 'js/x/d.mjs']);
  assert.deepEqual(translationClosure(['js/x/a.mjs'], graph), ['js/x/a.mjs', 'js/x/b.mjs', 'js/x/c.mjs']);
});

test('crate-assembly re-roots a module\'s own paths at it and keeps the paths of other modules', () => {
  const text = [
    'use crate::math::{double};',
    'pub fn sum(n: f64) -> f64 { crate::ml_sum_loop1(n, crate::ml::zero()) }',
    'pub fn ml_sum_loop1(n: f64, t: f64) -> f64 { let s = "crate::kept"; double(n) + t }',
    'pub mod ml { pub fn zero() -> f64 { 0f64 } }',
    '// | crate::comment',
  ].join('\n');
  assert.equal(rerootModule(text, 'sum', new Set(['sum', 'math'])), [
    'use crate::math::{double};',
    'pub fn sum(n: f64) -> f64 { crate::sum::ml_sum_loop1(n, crate::sum::ml::zero()) }',
    'pub fn ml_sum_loop1(n: f64, t: f64) -> f64 { let s = "crate::kept"; double(n) + t }',
    'pub mod ml { pub fn zero() -> f64 { 0f64 } }',
    '// | crate::comment',
  ].join('\n'));
  const crate = assembleCrate([{ module: 'math', text: 'pub fn double(x: f64) -> f64 { x * 2f64 }' }, { module: 'sum', text }], ['fn main() {}']);
  assert.match(crate.root, /^pub mod math;\npub mod sum;\nfn main\(\) \{\}\n$/mu);
  assert.deepEqual(crate.files.map((file) => file.name), ['math.rs', 'sum.rs']);
});

test('the blocker scan names every construct of an item the portable core refuses', () => {
  const context = moduleContext("import { helper } from './h.mjs';\nconst LIMIT = 3;\nfunction sibling() { return 1; }\n");
  assert.deepEqual([...context.imports], ['helper']);
  assert.deepEqual([...context.siblings].sort(), ['LIMIT', 'sibling']);
  const source = [
    '/** @param {string[]} words @param {Map<string, number>} counts */',
    'export function f(words, counts) {',
    '  const out = [];',
    '  for (const word of words) out.push(word.slice(1));',
    '  if (words.some((word) => word === null)) return helper(sibling(), /x/u.test(words[0]));',
    '  return out.map((word) => ({ word }));',
    '}',
  ].join('\n');
  assert.deepEqual(itemBlockers(source, context), [
    'JSDoc type {…}', 'arrow callback of .map()', 'arrow callback of .some()', 'call of a sibling function', 'call of an imported function',
    'method call .map()', 'method call .push()', 'method call .slice()', 'method call .some()', 'method call .test()', 'null',
    'object without a $ tag', 'regular expression',
  ]);
  assert.deepEqual(itemBlockers("import { a } from './m.mjs';"), ['import of names its module carries']);
  assert.deepEqual(itemBlockers("import { a } from '../host.mjs';"), ['import from outside the module directory']);
  assert.deepEqual(itemBlockers("import fs from 'node:fs';"), ['default or namespace import']);
  assert.deepEqual(itemBlockers('/** @param {number} x @returns {number} */\nfunction twice(x) {\n  return Math.max(x, 2) * 2;\n}'), []);
  assert.equal(portableType('string[]'), true);
  assert.equal(portableType('Array<number>'), true);
  assert.equal(portableType('{ text: string }'), false);
});

test('the unlock order lifts first the construct that frees the most items', () => {
  const order = unlockOrder([['a'], ['a'], ['b', 'c'], ['b'], ['c']], 3);
  assert.deepEqual(order.map((row) => [row.construct, row.cumulative]), [['a', 2], ['b', 3], ['c', 5]]);
});

test('the committed blocker table is the scan of the carried items the projections hold', () => {
  const table = read(BLOCKERS_FILE);
  const ledger = parseLedger(read(LEDGER_FILE));
  assert.equal(Number(/^ {2}carried_items (\d+)$/mu.exec(table)?.[1]), ledger.totals.carried_items);
  assert.match(table, /^unlock 1 "/mu);
});
