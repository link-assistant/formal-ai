// PR #1188, R1188-U7: the concise lexeme form reads as the long form.
//
// `lexeme en "read" "read the file"`, with the fields every surface shares as
// its children and long word lists continued on `words` lines, is expanded by
// `expandConciseLexemes` in js/seed_loader.js before the tree is built, so
// every reader sees one `surface` per word. The fixtures are shared with
// rust/tests/unit/concise_lexemes.rs, which pins the Rust twin. The converter
// (scripts/lib/notation-concise-lexemes.mjs) writes the concise form only
// where it parses back to the same tree.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { parseLino } from '../../../js/server/lino.mjs';
import { conciseLexemes } from '../../../scripts/lib/notation-concise-lexemes.mjs';
import { REPO_ROOT } from './support/browser-runtime.mjs';

const fixture = (name) => readFileSync(join(REPO_ROOT, 'rust/tests/fixtures/concise-lexemes', name), 'utf8');

/** The structure of a parsed tree, without the parser's indentation bookkeeping. */
const shape = (node) => ({ name: node.name, id: node.id, children: node.children.map(shape) });

test('the concise and the long form parse to one tree', () => {
  assert.deepEqual(shape(parseLino(fixture('concise.lino'))), shape(parseLino(fixture('long.lino'))));
});

test('every word becomes a surface with the shared fields', () => {
  const actor = parseLino(fixture('concise.lino')).children[0];
  const english = actor.children.find((child) => child.name === 'lexeme' && child.id === 'en');
  assert.deepEqual(
    english.children[0].children.map((field) => [field.name, field.id]),
    [['text', 'actor'], ['part_of_speech', 'noun'], ['grammatical_number', 'singular']],
  );
  const spanish = actor.children.find((child) => child.name === 'lexeme' && child.id === 'es');
  assert.deepEqual(spanish.children.map((surface) => surface.children[0].id), ['uno', 'dos', 'tres', 'cuatro']);
});

test('the converter writes the concise form and reads back the same tree', () => {
  const long = fixture('long.lino');
  const { text, converted } = conciseLexemes(long);
  assert.equal(converted, 3);
  assert.match(text, /^ {4}lexeme en actor$/mu);
  assert.match(text, /^ {4}lexeme ru "актёр" 'it''s' "a \\"b\\""$/mu);
  // A lexeme whose surfaces differ in their fields stays long.
  assert.match(text, /^ {4}lexeme es$/mu);
  assert.deepEqual(shape(parseLino(text)), shape(parseLino(long)));
});

test('a long word list continues on words lines', () => {
  const words = Array.from({ length: 30 }, (_, index) => `"word number ${index}"`);
  const long = ['m', '  lexeme en', ...words.flatMap((word) => ['    surface', `      text ${word}`])].join('\n');
  const { text } = conciseLexemes(long);
  assert.ok(text.split('\n').every((line) => line.length <= 120), text);
  assert.match(text, /^ {4}words "word number 0"/mu);
  assert.deepEqual(shape(parseLino(text)), shape(parseLino(long)));
});

test('a converted seed reads its words as surfaces', () => {
  const source = readFileSync(join(REPO_ROOT, 'data/seed/meanings-file-write.lino'), 'utf8');
  assert.match(source, /^ {4}lexeme en\n {6}words "read" "read the file"/mu, 'the seed-lexemes family is applied');
  const meaning = parseLino(source).children.find((child) => child.name === 'file_read_action');
  const english = meaning.children.find((child) => child.name === 'lexeme' && child.id === 'en');
  const words = english.children.map((surface) => surface.children.find((field) => field.name === 'text').id);
  assert.deepEqual(words.slice(0, 2), ['read', 'read the file']);
});
