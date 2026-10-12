// Self-translation between JavaScript, the meta language and Rust (R1024):
// the JavaScript side of the shared corpus in
// rust/tests/fixtures/self-translation/cases.lino, whose Rust side is
// rust/tests/unit/issue_1188_self_translation_corpus.rs. The format, the
// round-trip rule and the corpus are meta-language PR #196's
// (docs/self-translation.md); the one-contract-two-runtimes corpus is
// relative-meta-logic's (test-corpus/); the construct map is
// data/meta/self-translation/constructs.lino.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { CORPUS, checkCorpus, readCorpus, report } from '../../../scripts/self-translate.mjs';
import { constructMap } from '../../../scripts/self-translation/constructs.mjs';
import { selfTranslate, toMeta, translatedItems } from '../../../scripts/self-translation/envelope.mjs';
import { parseLinks } from '../../../scripts/self-translation/lino.mjs';
import { checkTwinCitations, citations, resolvableSegments } from '../../../scripts/check-twin-citations.mjs';

const REPOSITORY = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const corpusFile = (path) => readFileSync(join(REPOSITORY, CORPUS, path), 'utf8');
const corpus = readCorpus(REPOSITORY);
const caseNamed = (name) => corpus.cases.find((entry) => entry.name === name);

test('the committed corpus is what the translator produces (the --check gate)', () => {
  assert.deepEqual(checkCorpus(false, REPOSITORY), []);
});

test('a pure function translates to the exact Rust and back to the exact source', () => {
  const source = [
    '/**',
    ' * @param {number} a',
    ' * @returns {number}',
    ' */',
    'export function twice(a) {',
    '  return a * 2;',
    '}',
    '',
  ].join('\n');
  const rust = selfTranslate(source, 'JavaScript', 'Rust');
  assert.equal(rust.code, [
    '// formal-ai:self-translation:v1 source=JavaScript target=Rust sha256=ce9826f5097602c87c0bea2f9d7dbc2387c63b9d0e816f05533fb4c760c5cd9a bytes=95',
    '',
    '// formal-ai:prelude begin',
    '#![allow(clippy::missing_const_for_fn)]',
    '// formal-ai:prelude end',
    '',
    '// formal-ai:translated JavaScript function items=1 sha256=e661f16557d37417d78c5db0ad204bb99d961108c94129076e78622184c91529',
    '// | /**',
    '// |  * @param {number} a',
    '// |  * @returns {number}',
    '// |  */',
    '// | export function twice(a) {',
    '// |   return a * 2;',
    '// | }',
    '#[must_use]',
    'pub fn twice(a: f64) -> f64 {',
    '    a * 2.0',
    '}',
    '',
  ].join('\n'));
  assert.deepEqual(rust.items.map(({ term, status }) => `${term} ${status}`), ['comment translated', 'function translated']);
  assert.equal(selfTranslate(rust.code, 'Rust', 'JavaScript').code, source);
  assert.equal(selfTranslate(source, 'js', 'mjs').code, source);
});

test('an item outside the fragment is carried with the construct map reason, never dropped', () => {
  const source = 'export function count(items) {\n  return items.length;\n}\n';
  const rust = selfTranslate(source, 'JavaScript', 'Rust');
  assert.equal(rust.code.split('\n').slice(2).join('\n'), [
    '// formal-ai:carried JavaScript function (a parameter or result without a JSDoc number, boolean or string type: none)',
    '// | export function count(items) {',
    '// |   return items.length;',
    '// | }',
    '',
  ].join('\n'));
  assert.deepEqual(rust.items, [{ term: 'function', start: 0, end: 55, status: 'carried', reason: 'a parameter or result without a JSDoc number, boolean or string type: none' }]);
  assert.equal(selfTranslate(rust.code, 'Rust', 'JavaScript').code, source);
});

test('round trips restore both corpus sources byte for byte', () => {
  for (const [forth, back] of [['ratings-to-rust', 'ratings-back-to-javascript'], ['geometry-to-javascript', 'geometry-back-to-rust']]) {
    const there = caseNamed(forth);
    const home = caseNamed(back);
    assert.equal(home.source, there.expected, `${back} reads what ${forth} wrote`);
    assert.equal(home.expected, there.source, `${back} must give back ${forth}'s source`);
    const translated = selfTranslate(corpusFile(there.source), there.from, there.to).code;
    assert.equal(selfTranslate(translated, home.from, home.to).code, corpusFile(there.source));
  }
});

test('an edited block is translated again and every other block is restored', () => {
  const statuses = parseLinks(corpusFile('expected/ratings-edited-to-javascript.items.lino'))
    .filter((link) => link[3] !== 'comment')
    .map((link) => link[4]);
  assert.equal(statuses.filter((status) => status === 'translated').length, 1);
  assert.deepEqual([...new Set(statuses)].sort(), ['provenance', 'restored', 'translated']);
  const javascript = corpusFile('expected/ratings-edited-to-javascript.mjs');
  assert.match(javascript, /export function add\(a, b\) \{\n {2}return a \+ b \+ 1;\n\}/u);
});

test('the meta language is the pivot: both languages reimport to the same links', () => {
  const pairs = [
    ['sources/ratings.mjs', 'JavaScript', 'expected/ratings-to-rust.rs', 'Rust'],
    ['sources/geometry.rs', 'Rust', 'expected/geometry-to-javascript.mjs', 'JavaScript'],
  ];
  for (const [source, from, target, to] of pairs) {
    const original = translatedItems(corpusFile(source), from);
    // A translated block is restored when read back, so the reimport reads
    // the target code with its provenance comments removed.
    const code = corpusFile(target).split('\n').filter((line) => !line.startsWith('// formal-ai:translated ')).join('\n');
    const reimported = translatedItems(code, to);
    assert.ok(original.size >= 4, `${source} translates several items`);
    for (const [name, links] of original) assert.equal(reimported.get(name), links, `${name} reimports from ${target}`);
  }
  const meta = corpusFile('expected/ratings-to-meta.lino');
  assert.equal(meta, toMeta(corpusFile('sources/ratings.mjs'), 'JavaScript'));
  assert.match(meta, /^\(function add \(exported\) \(parameters \(a number\) \(b number\)\) \(returns number\) \(body \(return \(binary add \(variable a\) \(variable b\)\)\)\)\)$/mu);
  const fromMeta = corpusFile('expected/ratings-from-meta.rs');
  const translatedRust = corpusFile('expected/ratings-to-rust.rs');
  for (const definition of fromMeta.split('\n\n').filter((block) => !block.startsWith('#!['))) {
    assert.ok(translatedRust.includes(definition.trimEnd()), `the meta leg writes the Rust the envelope writes:\n${definition}`);
  }
});

const argumentOf = ({ type, value }) => (type === 'number' ? Number(value) : type === 'boolean' ? value === 'true' : value);

test('every corpus call returns the committed result on the JavaScript side', async () => {
  const modules = new Map();
  const moduleOf = async (entry) => {
    const path = entry.from === 'JavaScript' ? entry.source : entry.expected;
    if (!modules.has(path)) modules.set(path, await import(pathToFileURL(join(REPOSITORY, CORPUS, path)).href));
    return modules.get(path);
  };
  assert.ok(corpus.calls.length >= 25);
  for (const call of corpus.calls) {
    const module = await moduleOf(caseNamed(call.case));
    assert.equal(typeof module[call.javascript], 'function', `${call.case} exports ${call.javascript}`);
    const result = module[call.javascript](...call.arguments.map(argumentOf));
    assert.equal(String(result), call.result.value, `${call.javascript}(${call.arguments.map((arg) => arg.value).join(', ')})`);
    assert.equal(typeof result, call.result.type === 'text' ? 'string' : call.result.type);
  }
});

test('every Rust function the corpus compiles has a call, so both runtimes observe it', () => {
  const called = (caseName) => new Set(corpus.calls.filter((call) => call.case === caseName).map((call) => call.rust));
  const functions = (rust) => [...rust.matchAll(/^pub (?:const )?fn ([a-z_0-9]+)/gmu)].map((match) => match[1]);
  for (const [caseName, path] of [['ratings-to-rust', 'expected/ratings-to-rust.rs'], ['geometry-to-javascript', 'sources/geometry.rs']]) {
    const names = functions(corpusFile(path));
    assert.ok(names.length >= 4);
    for (const name of names) assert.ok(called(caseName).has(name), `${path} defines ${name} without a (call ${caseName} ...) row`);
  }
  const rustTest = readFileSync(join(REPOSITORY, 'rust/tests/unit/issue_1188_self_translation_corpus.rs'), 'utf8');
  for (const call of corpus.calls) {
    assert.ok(rustTest.includes(`("${call.case}", "${call.rust}")`), `the Rust test dispatches (${call.case}, ${call.rust})`);
  }
});

test('every construct map row has evidence in the corpus, and every refusal names a row', () => {
  const map = constructMap();
  const meta = corpusFile('expected/ratings-to-meta.lino') + toMeta(corpusFile('sources/geometry.rs'), 'Rust');
  const used = (head, id) => meta.includes(`(${head} ${id} `) || meta.includes(`(${head} ${id})`);
  for (const id of map.operators.keys()) assert.ok(used('binary', id), `operator ${id} is translated somewhere in the corpus`);
  for (const id of map.unaries.keys()) assert.ok(used('unary', id), `unary ${id}`);
  for (const id of map.builtins.keys()) assert.ok(used('builtin', id), `builtin ${id}`);
  for (const id of map.methods.keys()) assert.ok(used('method', id), `method ${id}`);
  for (const id of map.types.keys()) assert.ok(meta.includes(` ${id})`), `type ${id}`);
  for (const head of ['function', 'constant', 'let', 'return', 'if', 'choose', 'call']) assert.ok(meta.includes(`(${head} `), `construct ${head}`);
  assert.ok(corpusFile('expected/ratings-to-rust.rs').includes('\n// The items below sit outside portable-pure-v1; each is carried with the\n'), 'a comment group is copied');
  const reasons = corpus.cases
    .filter((entry) => entry.to !== 'Meta' && entry.from !== 'Meta')
    .flatMap((entry) => parseLinks(corpusFile(`expected/${entry.name}.items.lino`)))
    .filter((link) => link[4] === 'carried')
    .map((link) => link[5].quoted);
  const rows = [...map.carried.values()];
  for (const reason of reasons) assert.ok(rows.some((row) => reason === row || reason.startsWith(`${row}: `)), `${reason} is a carried row`);
  for (const row of rows) assert.ok(reasons.some((reason) => reason === row || reason.startsWith(`${row}: `)), `the carried row "${row}" is exercised`);
});

test('the report measures the twins against their cited Rust modules', () => {
  const rows = report(['js/agentic/crate'], REPOSITORY);
  assert.ok(rows.length >= 100);
  for (const row of rows) {
    assert.equal(row.translated + row.carried, row.items, row.module);
    assert.ok(row.identical <= row.named && row.named <= row.translated, row.module);
  }
  assert.ok(rows.reduce((sum, row) => sum + row.identical, 0) > 0, 'some translated definitions match the hand-written Rust exactly');
});

test('twin citations name a Rust definition that still exists', () => {
  assert.deepEqual(citations('/**\n * Mirrors `fn compile_task` in\n * rust/src/agentic_coding/procedure.rs.\n */'), [
    { symbol: 'fn compile_task', path: 'rust/src/agentic_coding/procedure.rs' },
  ]);
  assert.deepEqual(resolvableSegments('pub const fn is_script_combining_mark'), ['is_script_combining_mark']);
  assert.deepEqual(resolvableSegments('crate::calculation::calculation_expression_candidates'), ['calculation', 'calculation_expression_candidates']);
  assert.equal(resolvableSegments('the canonicalize / is_dir prologue of fn run_agent'), null);
  const result = checkTwinCitations(REPOSITORY);
  assert.deepEqual(result.stale, []);
  assert.ok(result.checked >= 700, `${result.checked} citations checked`);
});
