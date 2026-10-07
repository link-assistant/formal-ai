// The js -> rust leg of the js-first cycle (R994, R1000, R1012): the
// JavaScript root translated to Rust by link-foundation/meta-language's
// self-translation at the commit data/meta/js-rust-translation.lino pins.
// The js-rust job of layered-ci.yml re-translates every module with that
// commit, requires the committed projections byte for byte, compiles them
// with rustc and runs the same equivalence assertions in the translated Rust
// (`node scripts/translate-js-rust.mjs --check --compile`). This suite needs
// no upstream checkout: it holds the committed translation to the current
// JavaScript and runs the JavaScript side of every assertion.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import {
  CALLS_FILE, LEDGER_FILE, inferredSignature, measure, parseCalls, parseLedger, projectionPath, readProjection,
  refusalClass, regressions, rustStringValue, scopeModules, translatedConstants, verify, workflowPin,
} from '../../../scripts/translate-js-rust.mjs';

const REPOSITORY = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const read = (path) => readFileSync(join(REPOSITORY, path), 'utf8');
const load = (path) => import(pathToFileURL(join(REPOSITORY, path)).href);
const sha256 = (text) => createHash('sha256').update(text, 'utf8').digest('hex');

test('the committed translation is for the current JavaScript of every module in scope (--verify)', () => {
  assert.deepEqual(verify(REPOSITORY), []);
});

test('the ledger pins one meta-language main commit, and the js-rust job checks out that commit', () => {
  const ledger = parseLedger(read(LEDGER_FILE));
  assert.equal(ledger.upstream, 'link-foundation/meta-language');
  assert.match(ledger.commit, /^[0-9a-f]{40}$/u);
  assert.equal(workflowPin(read('.github/workflows/layered-ci.yml')), ledger.commit);
  assert.equal(ledger.totals.modules, scopeModules(REPOSITORY).length);
  assert.equal(ledger.totals.translated_modules + ledger.totals.refused_modules, ledger.totals.modules);
});

test('issue_report.mjs translates whole: both of its items are Rust, none carried', () => {
  const projection = read(projectionPath('js/agentic/crate/issue_report.mjs'));
  const { translated, inferred, carried } = readProjection(projection);
  assert.deepEqual({ translated, inferred, carried }, { translated: 2, inferred: 0, carried: 0 });
  assert.match(projection, /^pub const LINO_FENCE_LANGUAGE: &str = "lino";$/mu);
  assert.match(projection, /^pub fn fenced_block\(language: String, content: String\) -> String \{$/mu);
  assert.deepEqual(translatedConstants(projection), [{ name: 'LINO_FENCE_LANGUAGE', type: '&str', value: 'lino' }]);
});

test('every call of calls.lino returns its recorded result in JavaScript and names a translated Rust function', async () => {
  const calls = parseCalls(read(CALLS_FILE));
  assert.deepEqual([...new Set(calls.map((call) => call.rust))], ['fenced_block', 'span', 'permission_key', 'verdict_slug', 'compare_tuples']);
  for (const call of calls) {
    const module = await load(call.module);
    assert.equal(module[call.javascript](...call.args.map((arg) => arg.value)), call.result.value, `${call.javascript} of ${call.module}`);
    const projection = read(projectionPath(call.module));
    assert.match(projection, new RegExp(`^pub fn ${call.rust}\\(`, 'mu'), `${call.rust} is translated Rust in ${projectionPath(call.module)}`);
  }
  // The byte-order mark row: JavaScript's trimEnd strips U+FEFF, so the
  // faithful twin does too (the hand-written str::trim_end does not).
  const { fencedBlock } = await load('js/agentic/crate/issue_report.mjs');
  assert.equal(fencedBlock('lino', 'x﻿'), '```lino\nx\n```');
});

test('every translated exported constant holds the value its JavaScript module exports', async () => {
  let compared = 0;
  for (const path of scopeModules(REPOSITORY).filter((entry) => entry.endsWith('.mjs'))) {
    const constants = translatedConstants(read(projectionPath(path)));
    if (constants.length === 0) continue;
    const module = await load(path);
    for (const constant of constants) {
      assert.ok(constant.name in module, `${path} exports ${constant.name}`);
      assert.equal(module[constant.name], constant.value, `${constant.name} of ${path}`);
      compared += 1;
    }
  }
  assert.ok(compared >= 60, `at least 60 translated constants are compared, found ${compared}`);
});

test('a projection keeps translated blocks byte for byte and reduces a carried item to its refused construct', () => {
  const code = 'pub fn twice(a: f64) -> f64 {\n    (a * 2f64)\n}';
  const upstream = [
    '// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=abc bytes=10',
    '',
    '// meta-language:prelude begin',
    '#![allow(unused)]',
    '// meta-language:prelude end',
    '',
    '// A copied comment.',
    '',
    `// meta-language:translated JavaScript export_statement items=1 sha256=${sha256(code)}`,
    '// | /** @param {number} a @returns {number} */',
    '// | export function twice(a) {',
    '// |   return a * 2;',
    '// | }',
    code,
    '',
    '// meta-language:carried JavaScript import_statement (unsupported)',
    "// | import { cached } from '../host.mjs';",
    '',
  ].join('\n');
  const measured = measure('js/agentic/crate/example.mjs', {
    code: upstream,
    details: [null, "import { cached }: import the assertion module as a whole at 0..37"],
  });
  assert.deepEqual(
    { translated: measured.translated, inferred: measured.inferred, carried: measured.carried, refusals: measured.refusals },
    { translated: 1, inferred: 0, carried: 1, refusals: ['import { … }'] },
  );
  assert.equal(measured.projection, [
    '// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=abc bytes=10',
    '// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)',
    '',
    '// meta-language:prelude begin',
    '#![allow(unused)]',
    '// meta-language:prelude end',
    '',
    `// meta-language:translated JavaScript export_statement items=1 sha256=${sha256(code)}`,
    '// | /** @param {number} a @returns {number} */',
    '// | export function twice(a) {',
    '// |   return a * 2;',
    '// | }',
    code,
    '',
    '// meta-language:carried JavaScript import_statement (unsupported)',
    '// formal-ai:refusal import { … }',
    '',
  ].join('\n'));
});

test('a refusal is counted by the construct it names, not by the occurrence', () => {
  assert.equal(refusalClass('unsupported', "import { cached }: import the assertion module as a whole, e.g. import assert from 'node:assert/strict' at 0..37"), 'import { … }');
  assert.equal(refusalClass('unsupported', 'method call .map(): methods of values are outside the portable core at 4..9'), 'method call .map()');
  assert.equal(refusalClass('unsupported', 'JSDoc type {{text: string}}: portable types are number, bigint, boolean, string, arrays T[] of them and @typedef data types at 0..54'), 'JSDoc type {…}');
  assert.equal(refusalClass('type', 'unknown name crate.functionWords at 10..23'), 'type: unknown name (a sibling item or an import)');
  assert.equal(refusalClass('syntax', 'malformed number 9a at 78..80'), 'syntax: malformed number');
  assert.equal(refusalClass('no definition', null), 'no definition');
});

test('the ratchet: a module never loses a translated item or gains an inferred signature', () => {
  const before = [
    { path: 'js/agentic/crate/a.mjs', translated: 3, inferred: 0 },
    { path: 'js/agentic/crate/b.mjs', translated: 1, inferred: 1 },
    { path: 'js/agentic/crate/gone.mjs', translated: 5, inferred: 0 },
  ];
  const after = [
    { path: 'js/agentic/crate/a.mjs', translated: 2, inferred: 1 },
    { path: 'js/agentic/crate/b.mjs', translated: 4, inferred: 0 },
    { path: 'js/agentic/crate/new.mjs', translated: 0, inferred: 0 },
  ];
  assert.deepEqual(regressions(before, after), [
    'js/agentic/crate/a.mjs: translated 3 -> 2',
    'js/agentic/crate/a.mjs: inferred signatures 0 -> 1 (give every parameter a JSDoc @param type)',
  ]);
});

test('a parameter without a JSDoc @param type makes an inferred signature', () => {
  const block = (lines) => lines.map((line) => `// | ${line}`).join('\n');
  assert.equal(inferredSignature(block(['/**', ' * @param {number} count', ' * @param {string} singular', ' */', 'function pluralize(count, singular) {'])), false);
  assert.equal(inferredSignature(block(['/** Mirrors `fn trailer`. */', 'const trailer = (name, value) => `${name}: ${value}`;'])), true);
  assert.equal(inferredSignature(block(['/** @param {number} value */', 'const isCount = value => value >= 0;'])), false);
  assert.equal(inferredSignature(block(["export const ROOT = 'demo';"])), false);
});

test('Rust string literals read back to the text they denote', () => {
  assert.equal(rustStringValue('"\\u{1e}%H\\u{1f}"'), '\u001e%H\u001f');
  assert.equal(rustStringValue('"a\\n\\"b\\"\\\\"'), 'a\n"b"\\');
});
