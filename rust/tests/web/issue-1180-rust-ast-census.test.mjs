// Issue #1180 R11, JavaScript root: the Rust node-kind histogram of
// `self_ast::ast_census` (rust/src/agentic_coding/self_ast.rs) has a JS twin,
// js/agentic/crate/rust_ast_census.mjs, parsing with the vendored
// web-tree-sitter runtime and the tree-sitter-rust grammar meta-language
// compiles in. Parity is pinned against the native histograms the repository
// already commits, which the Rust suite keeps byte-for-byte fresh:
// data/meta/self-ast.lino (the planner, issue_538_agentic) and every
// full-AST census under data/meta/self-ast/src (issue_673_self_ast_census).
// A census document is content-addressed (`content_id` is FNV-1a over the
// source), so only documents describing the current source are compared.

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as childProcess from 'node:child_process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { diffCensus, nodeHistoryIo } from '../../../js/agentic/crate/history_store.mjs';
import { historyAstCensus } from '../../../js/agentic/crate/rust_ast_census.mjs';
import { rustAstCensus } from '../../../js/agentic/node-host.mjs';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const VENDOR = path.join(REPO, 'js/vendor/tree-sitter');

/** FNV-1a 64 over UTF-8 bytes, as `stable_id` hashes a census source. */
function fnv1a(text) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of Buffer.from(text, 'utf8')) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return hash.toString(16).padStart(16, '0');
}

/** The `node_kinds` rows of a committed census document at `indent`. */
function committedKinds(text, indent) {
  const header = `${' '.repeat(indent - 2)}node_kinds\n`;
  const start = text.indexOf(header);
  if (start === -1) return null;
  const rows = [];
  for (const line of text.slice(start + header.length).split('\n')) {
    if (!line.startsWith(' '.repeat(indent)) || line.startsWith(' '.repeat(indent + 1))) break;
    const [kind, count] = line.trim().split(' ');
    rows.push({ kind, count: Number(count) });
  }
  return rows;
}

function censusDocuments(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) return censusDocuments(full);
    return entry.name.endsWith('.lino') ? [full] : [];
  });
}

test('the vendored runtime and grammar are the recorded upstream artifacts', () => {
  const provenance = fs.readFileSync(path.join(VENDOR, 'provenance.lino'), 'utf8');
  const files = [...provenance.matchAll(/^ {2}file (\S+)\n((?: {4}.*\n)+)/gmu)];
  assert.deepEqual(files.map((match) => match[1]), ['web-tree-sitter.mjs', 'web-tree-sitter.wasm', 'tree-sitter-rust.wasm']);
  for (const [, name, body] of files) {
    const recorded = body.match(/sha256 "([0-9a-f]{64})"/u)[1];
    const actual = createHash('sha256').update(fs.readFileSync(path.join(VENDOR, name))).digest('hex');
    assert.equal(actual, recorded, name);
  }
  // The grammar is the tree-sitter-rust version Cargo.lock resolves for
  // meta-language, so both runtimes parse with one grammar.
  const lock = fs.readFileSync(path.join(REPO, 'rust/Cargo.lock'), 'utf8');
  const locked = lock.match(/name = "tree-sitter-rust"\nversion = "([^"]+)"/u)[1];
  const grammar = files.find((match) => match[1] === 'tree-sitter-rust.wasm')[2].match(/version "([^"]+)"/u)[1];
  assert.equal(grammar, locked);
});

test('a function and a struct census as the native census does', async () => {
  const census = await rustAstCensus();
  assert.deepEqual(census('fn f() {}\n'), {
    named_node_count: 5,
    clean: true,
    node_kinds: [
      { kind: 'block', count: 1 },
      { kind: 'function_item', count: 1 },
      { kind: 'identifier', count: 1 },
      { kind: 'parameters', count: 1 },
      { kind: 'source_file', count: 1 },
    ],
  });
  assert.deepEqual(census('struct S {\n    field: i32,\n}\n'), {
    named_node_count: 7,
    clean: true,
    node_kinds: [
      { kind: 'field_declaration', count: 1 },
      { kind: 'field_declaration_list', count: 1 },
      { kind: 'field_identifier', count: 1 },
      { kind: 'primitive_type', count: 1 },
      { kind: 'source_file', count: 1 },
      { kind: 'struct_item', count: 1 },
      { kind: 'type_identifier', count: 1 },
    ],
  });
  assert.equal(census('fn broken( {\n').clean, false);
});

test('the planner histogram matches the committed native self-AST document', async () => {
  const census = await rustAstCensus();
  const document = fs.readFileSync(path.join(REPO, 'data/meta/self-ast.lino'), 'utf8');
  const target = document.match(/^ {2}target (\S+)$/mu)[1];
  const result = census(fs.readFileSync(path.join(REPO, 'rust', target), 'utf8'));
  assert.deepEqual(result.node_kinds, committedKinds(document, 4));
  assert.equal(result.named_node_count, Number(document.match(/^ {2}named_node_count (\d+)$/mu)[1]));
  assert.equal(String(result.clean), document.match(/^ {2}clean (\S+)$/mu)[1]);
});

test('every full-AST census of a current source matches the native histogram', async () => {
  const census = await rustAstCensus();
  const compared = [];
  const drifted = [];
  for (const file of censusDocuments(path.join(REPO, 'data/meta/self-ast/src'))) {
    const document = fs.readFileSync(file, 'utf8');
    const kinds = committedKinds(document, 6);
    if (kinds === null) continue;
    const target = document.match(/^ {2}target (\S+)$/mu)[1];
    const source = fs.readFileSync(path.join(REPO, 'rust', target), 'utf8');
    if (document.match(/content_id source_module_([0-9a-f]{16})/u)[1] !== fnv1a(source)) continue;
    compared.push(target);
    const result = census(source);
    try {
      assert.deepEqual(result.node_kinds, kinds);
      assert.equal(result.named_node_count, Number(document.match(/^ {4}named_node_count (\d+)$/mu)[1]));
    } catch {
      drifted.push(target);
    }
  }
  assert.ok(compared.length >= 80, `only ${compared.length} current full-AST census documents`);
  assert.deepEqual(drifted, []);
});

test('the history importer records the histogram delta through io.astCensus', async () => {
  const io = nodeHistoryIo({ fs, path, childProcess, astCensus: historyAstCensus(await rustAstCensus()) });
  assert.deepEqual(diffCensus(io, 'src/lib.rs', 'fn f() {}\n', 'struct S {\n    field: i32,\n}\n'), [
    { path: 'src/lib.rs', source: 'ast_census', item: 'block', delta: -1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'field_declaration', delta: 1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'field_declaration_list', delta: 1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'field_identifier', delta: 1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'function_item', delta: -1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'identifier', delta: -1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'parameters', delta: -1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'primitive_type', delta: 1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'struct_item', delta: 1 },
    { path: 'src/lib.rs', source: 'ast_census', item: 'type_identifier', delta: 1 },
  ]);
  // Without a census the importer records no histogram, as before.
  assert.deepEqual(diffCensus(nodeHistoryIo({ fs, path, childProcess }), 'src/lib.rs', 'fn f() {}\n', ''), []);
});
