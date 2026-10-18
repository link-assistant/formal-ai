#!/usr/bin/env node
// Receipt aggregation never mistakes identities or counters for source content.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {blake3Hex as hexadecimal} from '../js/crypto/blake3.mjs';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

/** @param {string} directory @returns {string[]} */
function receiptFiles(directory) {
  assert(fs.lstatSync(directory).isDirectory() && !fs.lstatSync(directory).isSymbolicLink());
  return fs.readdirSync(directory, {withFileTypes: true}).flatMap(entry => {
    const target = path.join(directory, entry.name);
    assert(!entry.isSymbolicLink(), 'ReceiptSymlinkRefused');
    if (entry.isDirectory()) return receiptFiles(target);
    assert(entry.isFile() && entry.name === 'receipt.json', 'UnexpectedSourceReceiptMember');
    assert(fs.lstatSync(target).size <= 1024 * 1024, 'SourceReceiptBudgetExceeded');
    return [target];
  });
}

/** Full model members stay in their immutable CI artifacts; this joins their evidence only.
 * @param {object[]} receipts @param {Array<{path:string,bytes:Uint8Array}>} sources
 * @param {{commit:string,tree:string,run:string,attempt:string}} identity */
export function aggregateSourceContent(receipts, sources, identity) {
  assert(receipts.length > 0 && receipts.length <= 32 && receipts.length <= sources.length);
  const expected = new Map(sources.map(source => [source.path, hexadecimal(source.bytes)]));
  assert.equal(expected.size, sources.length);
  const rows = [], shards = new Set();
  for (const receipt of receipts) {
    assert.equal(receipt.schema, 'native-readable-source-shard/v1');
    assert.deepEqual(receipt.identity, identity);
    assert.equal(receipt.shards, receipts.length);
    assert(Number.isSafeInteger(receipt.shardIndex) && receipt.shardIndex >= 0 && receipt.shardIndex < receipts.length);
    assert(!shards.has(receipt.shardIndex)); shards.add(receipt.shardIndex);
    assert.equal(receipt.receiptAuthority, 'navigation-and-producer-evidence-only');
    assert(Array.isArray(receipt.models));
    for (const row of receipt.models) {
      assert(expected.has(row.path), 'UnownedSourceContent');
      assert.equal(row.sourceBlake3, expected.get(row.path));
      assert.equal(row.member, row.path.slice('rust/'.length).replace(/\.rs$/u, '.lino'));
      assert.match(row.modelBlake3, /^[0-9a-f]{64}$/u);
      assert.equal(row.sourceFreeRestoration, true);
      assert.equal(row.independentGrammarReparse, true);
      rows.push({...row, artifact: 'source-content-models-' + receipt.shardIndex});
    }
  }
  assert.equal(new Set(rows.map(row => row.path)).size, rows.length, 'DuplicateSourceContent');
  assert.deepEqual(rows.map(row => row.path).sort(), [...expected.keys()].sort(), 'IncompleteSourceContent');
  return {schema: 'native-readable-source-content/v1', identity,
    representation: 'external-full-recursive-UTF8-Links-models',
    receiptAuthority: 'navigation-and-producer-evidence-only', shards: receipts.length,
    models: rows.sort((left, right) => left.path < right.path ? -1 : left.path > right.path ? 1 : 0)};
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2); assert.equal(args.length, 2);
  const files = execFileSync('git', ['ls-files', '-z', '--', 'rust/src'], {cwd: root, encoding: 'utf8'})
    .split('\0').filter(file => file.endsWith('.rs')).sort();
  const sources = files.map(file => ({path: file, bytes: fs.readFileSync(path.join(root, file))}));
  const git = reference => execFileSync('git', ['rev-parse', reference], {cwd: root, encoding: 'utf8'}).trim();
  const identity = {commit: git('HEAD'), tree: git('HEAD^{tree}'),
    run: process.env.GITHUB_RUN_ID, attempt: process.env.GITHUB_RUN_ATTEMPT};
  const receipts = receiptFiles(path.resolve(args[0])).map(file => JSON.parse(fs.readFileSync(file, 'utf8')));
  const result = aggregateSourceContent(receipts, sources, identity);
  fs.mkdirSync(path.dirname(path.resolve(args[1])), {recursive: true});
  fs.writeFileSync(path.resolve(args[1]), JSON.stringify(result, null, 2) + '\n', {flag: 'wx'});
}
