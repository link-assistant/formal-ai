// Issue #1180 R11 in the JavaScript root: the per-path symbol diff of the
// repository-history importer (js/agentic/crate/history_store.mjs
// `diffSymbols`) mirrors rust/src/history_context/commits.rs `diff_symbols`
// in full. An ES path carries the `es_meta_extract token_count` delta counted
// by the shared tokenizer twin (js/agentic/crate/es_tokenizer.mjs, the body of
// scripts/translate-es.mjs `tokenize`), then its named items; a Rust path
// carries the `ast_census` node-kind histogram delta, then its named items.
// The histogram is a tree-sitter parse no JavaScript root carries, so it comes
// through the optional `io.astCensus`, and without one it is empty, as the
// native importer built without the `meta-language` feature records it.

import assert from 'node:assert/strict';
import childProcess from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { before, test } from 'node:test';

import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { countTokens, tokenize } from '../../../js/agentic/crate/es_tokenizer.mjs';
import { tokenize as scriptTokenize } from '../../../scripts/translate-es.mjs';

let history;
let store;
let rules;

const git = (cwd, ...args) =>
  childProcess.execFileSync('git', args, {
    cwd,
    encoding: 'utf8',
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: 'Tester',
      GIT_AUTHOR_EMAIL: 'tester@example.com',
      GIT_COMMITTER_NAME: 'Tester',
      GIT_COMMITTER_EMAIL: 'tester@example.com',
    },
  });

/** A scratch repository with one commit per `[file, text]` step; returns the root and the shas. */
function repositoryWith(steps) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-1180-symbols-'));
  git(root, 'init', '-q', '-b', 'main');
  git(root, 'config', 'commit.gpgsign', 'false');
  const shas = [];
  for (const [file, text] of steps) {
    fs.writeFileSync(path.join(root, file), text);
    git(root, 'add', '.');
    git(root, 'commit', '-q', '-m', `edit ${file}`);
    shas.push(git(root, 'rev-parse', 'HEAD').trim());
  }
  return { root, shas };
}

before(async () => {
  await installNodeHost(new WorkerHost());
  history = await import('../../../js/agentic/crate/history_context.mjs');
  store = await import('../../../js/agentic/crate/history_store.mjs');
  rules = history.historyRules();
});

test('countTokens mirrors es_meta::count_tokens over the tokenizer tree', () => {
  // export, const, b, =, 2, ; are six leaves.
  assert.equal(countTokens(tokenize('export const b = 2;')), 6);
  // A group counts its children, not its delimiters: f ( a , b ) -> f a , b.
  assert.equal(countTokens(tokenize('f(a, b)')), 4);
  // A template counts each chunk once and the trees of each interpolation:
  // `x${a + 1}y` -> chunk x, a, +, 1, chunk y.
  assert.equal(countTokens(tokenize('`x${a + 1}y`')), 5);
  assert.equal(countTokens(tokenize('')), 0);
});

test('the translator script tokenizes through the same module and keeps the native error wording', () => {
  const source = 'const t = `a${b}c`; f(/re/g, [1, 2]);';
  assert.deepEqual(scriptTokenize(source), tokenize(source));
  assert.throws(() => tokenize('f(a'), (error) => error.kind === 'unclosed_group' && error.start === 1);
  assert.throws(() => scriptTokenize('f(a'), /unclosed group at byte 1/);
  assert.throws(() => scriptTokenize('"abc'), /unterminated string literal at byte 0/);
});

test('an ES path carries the token_count delta before its named items', () => {
  const { root, shas } = repositoryWith([
    ['app.js', 'export const a = 1;\n'],
    ['app.js', 'export const a = 1;\nexport const b = 2;\n'],
  ]);
  const io = store.nodeHistoryIo({ fs, path, childProcess });
  const changes = store.diffSymbols(io, root, shas[1], 'app.js', rules);
  assert.deepEqual(changes[0], { path: 'app.js', source: 'es_meta_extract', item: 'token_count', delta: 6 });
  assert.deepEqual(
    changes.slice(1),
    history.diffNamedItems('app.js', 'es_meta_extract', 'export const a = 1;\n', 'export const a = 1;\nexport const b = 2;\n', rules),
  );
  assert.ok(changes.slice(1).some((change) => change.item === 'app.js#const b' && change.delta === 1));
  fs.rmSync(root, { recursive: true, force: true });
});

test('an untokenizable side counts zero tokens, as the native map_or(0) does', () => {
  assert.deepEqual(store.diffEs('broken.js', 'f(', 'f(a);'), [
    { path: 'broken.js', source: 'es_meta_extract', item: 'token_count', delta: 3 },
  ]);
  assert.deepEqual(store.diffEs('same.js', 'a + b', 'b + a'), []);
});

test('a Rust path carries the injected census histogram, sorted and without zero deltas', () => {
  const { root, shas } = repositoryWith([
    ['lib.rs', 'pub fn first() {}\n'],
    ['lib.rs', 'pub fn first() {}\npub fn second() {}\n'],
  ]);
  const census = {
    'pub fn first() {}\n': [
      { kind: 'source_file', count: 1 },
      { kind: 'function_item', count: 1 },
      { kind: 'identifier', count: 1 },
    ],
    'pub fn first() {}\npub fn second() {}\n': [
      { kind: 'source_file', count: 1 },
      { kind: 'function_item', count: 2 },
      { kind: 'identifier', count: 2 },
      { kind: 'visibility_modifier', count: 2 },
    ],
  };
  const io = { ...store.nodeHistoryIo({ fs, path, childProcess }), astCensus: (source) => census[source] ?? [] };
  const changes = store.diffSymbols(io, root, shas[1], 'lib.rs', rules);
  const counted = changes.filter((change) => change.source === 'ast_census' && !change.item.includes('#'));
  assert.deepEqual(counted, [
    { path: 'lib.rs', source: 'ast_census', item: 'function_item', delta: 1 },
    { path: 'lib.rs', source: 'ast_census', item: 'identifier', delta: 1 },
    { path: 'lib.rs', source: 'ast_census', item: 'visibility_modifier', delta: 2 },
  ]);
  assert.deepEqual(changes.slice(0, counted.length), counted, 'the histogram comes before the named items');
  assert.ok(changes.some((change) => change.item === 'lib.rs#fn second' && change.delta === 1));

  // nodeHistoryIo offers no census: the histogram is empty, the named items remain.
  const plain = store.diffSymbols(store.nodeHistoryIo({ fs, path, childProcess }), root, shas[1], 'lib.rs', rules);
  assert.ok(plain.every((change) => change.item.includes('#')));
  assert.ok(plain.some((change) => change.item === 'lib.rs#fn second'));
  fs.rmSync(root, { recursive: true, force: true });
});
