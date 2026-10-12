// R1188-U4: names are full English words. scripts/measure-abbreviations.mjs
// measures abbreviated file names and declared names against
// data/meta/abbreviation-ratchet.lino (gate check-abbreviations), with the
// abbreviation list of data/meta/notation-rules.lino.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import {
  abbreviatedWords,
  declaredNames,
  fullWordPath,
  readAbbreviationRules,
  wordsOfIdentifier,
} from '../../../scripts/measure-abbreviations.mjs';
import { REPO_ROOT } from './support/browser-runtime.mjs';

const abbreviations = new Map([
  ['nl', 'natural-language'],
  ['config', 'configuration'],
]);

test('a declared name splits at underscores and case changes', () => {
  assert.deepEqual(wordsOfIdentifier('parseArgs'), ['parse', 'args']);
  assert.deepEqual(wordsOfIdentifier('MAX_LEN'), ['max', 'len']);
});

test('a name is abbreviated when one of its words is on the list', () => {
  assert.deepEqual(abbreviatedWords('formal_ai_worker_nl_tools', abbreviations), ['nl']);
  assert.deepEqual(abbreviatedWords('natural_language_tools', abbreviations), []);
});

test('the full-word spelling keeps the separator of the stem', () => {
  assert.equal(
    fullWordPath('js/example/sample_nl_tools.js', abbreviations),
    'js/example/sample_natural_language_tools.js',
  );
  assert.equal(fullWordPath('vscode/scripts/config.test.mjs', abbreviations), 'vscode/scripts/configuration.test.mjs');
});

test('Rust patterns and lifetimes are not bindings', () => {
  assert.deepEqual(declaredNames('let pos: usize = 1;\nlet Some(x) = y;\nfn f(text: &str) {}', 'rust'), ['pos', 'f']);
});

test('the rules file states a reason for every exclusion and reads its list from the notation rules', () => {
  const text = readFileSync(join(REPO_ROOT, 'data/meta/abbreviation-rules.lino'), 'utf8');
  const rules = readAbbreviationRules(text);
  assert.equal(rules.abbreviationsFrom, 'data/meta/notation-rules.lino');
  assert.ok(rules.fileNames.excluded.includes('docs/case-studies'));
  assert.ok(rules.fileNames.excluded.includes('experiments/formal_ai_subagent/evidence'));
  assert.ok(!rules.fileNames.excluded.includes('js'));
  assert.ok(!rules.fileNames.excluded.includes('rust/tests'));
  assert.deepEqual(
    rules.bindings.map((entry) => entry.scope),
    ['js', 'rust/src'],
  );
  const excludedLines = text.split('\n').filter((line) => /^\s+(?:excluded|fixed-name) /u.test(line)).length;
  const reasonLines = text.split('\n').filter((line) => /^\s+reason "/u.test(line)).length;
  assert.equal(reasonLines, excludedLines);
});

test('the repository holds the abbreviation ratchet', () => {
  const output = execFileSync('node', ['scripts/measure-abbreviations.mjs', '--check'], { cwd: REPO_ROOT, encoding: 'utf8' });
  assert.match(output, /^file-names {2}abbreviated-file-names \d+$/mu);
  assert.match(output, /^bindings rust\/src {2}abbreviated-bindings \d+$/mu);
});
