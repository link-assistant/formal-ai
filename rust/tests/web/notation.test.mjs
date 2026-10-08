// R1188-U6 and R1188-U7: the links notation we own prefers `-` over `_`, uses
// full English words and states shared structure once. scripts/measure-notation.mjs
// measures it against data/meta/notation-ratchet.lino (gate check-notation).

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { namesOfLine, parseTree, readRules } from '../../../scripts/lib/links-notation-names.mjs';
import { duplicatedBlocks, measureDocument, repeatedFields } from '../../../scripts/measure-notation.mjs';
import { REPO_ROOT } from './support/browser-runtime.mjs';

test('names are the unquoted lowercase identifier tokens of a line', () => {
  const names = namesOfLine('  role file_read_action_cue "a_quoted_text" WebSearch # comment_name').map((entry) => entry.name);
  assert.deepEqual(names, ['role', 'file_read_action_cue']);
  assert.deepEqual(
    namesOfLine('    aliases ("web_search" "read_file") defined-by: path/to_file.rs').map((entry) => entry.name),
    ['aliases', 'defined-by'],
  );
  // `$name` refers to the value a handler rule names, so a rename reaches it.
  assert.deepEqual(namesOfLine('      log source $sample_value_name'), [
    { name: 'log', start: 6 },
    { name: 'source', start: 10 },
    { name: 'sample_value_name', start: 18 },
  ]);
});

test('a name counts as abbreviated when one of its words is on the seeded list', () => {
  const measured = measureDocument('record\n  source_id 1\n  position 2\n', new Map([['id', 'identifier']]));
  assert.deepEqual([...measured.abbreviated], ['source_id']);
  assert.deepEqual([...measured.names].filter((name) => name.includes('_')), ['source_id']);
});

test('a repeated child block counts once, at its outermost occurrence', () => {
  const block = 'meaning a\n  lexeme en\n    surface\n      text "x"\n    surface\n      text "y"\n';
  const trees = [parseTree(block), parseTree(block.replace('meaning a', 'meaning b'))];
  const duplicated = duplicatedBlocks(trees, 3);
  assert.equal(duplicated.size, 1);
  assert.deepEqual([...duplicated.values()], [{ lines: 5, occurrences: 2 }]);
});

test('a leaf line that most sibling records repeat is a repeated field', () => {
  const ledger = ['ledger', ...[1, 2, 3, 4, 5].map((id) => `  requirement r${id}\n    manual "not yet confirmed"\n    test t${id}`)].join('\n');
  const repeated = repeatedFields([parseTree(ledger)]);
  assert.deepEqual([...repeated], [['requirement manual "not yet confirmed"', 5]]);
});

test('the rules file names the scopes, the exclusions and the abbreviations', () => {
  const rules = readRules(readFileSync(join(REPO_ROOT, 'data/meta/notation-rules.lino'), 'utf8'));
  assert.ok(rules.scopes.includes('data/seed') && rules.scopes.includes('data/meta'));
  assert.equal(rules.abbreviations.get('pos'), 'position');
  assert.ok(rules.properTerms.has('qid'));
  assert.ok(rules.excluded.every((entry) => entry.reason.length > 0));
});

test('the repository holds the notation ratchet', () => {
  const output = execFileSync('node', ['scripts/measure-notation.mjs', '--check'], { cwd: REPO_ROOT, encoding: 'utf8' });
  assert.match(output, /^data\/seed \(scope\)\s+\d+/mu);
});
