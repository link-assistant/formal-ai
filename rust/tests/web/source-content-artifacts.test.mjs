import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import {aggregateSourceContent} from '../../../scripts/aggregate-source-content.mjs';
import {blake3Hex as hexadecimal} from '../../../js/crypto/blake3.mjs';
import {readSourceLinks, sourceFromLinks, ownedSourceLinks, writeSourceLinks} from '../../../js/meta/readable-source.js';

const identity = {commit: 'a'.repeat(40), tree: 'b'.repeat(40), run: '123', attempt: '1'};
function fixture() {
  const model = readSourceLinks(fs.readFileSync(new URL('../fixtures/source-model/Rust.lino', import.meta.url), 'utf8'));
  const source = sourceFromLinks(model);
  const sources = ['alpha', 'nested/beta'].map(name => ({path: 'rust/src/' + name + '.rs', bytes: Buffer.from(source)}));
  const receipts = sources.map((entry, index) => {
    const member = entry.path.slice('rust/'.length).replace(/\.rs$/u, '.lino');
    const retained = writeSourceLinks(ownedSourceLinks(entry.path.slice('rust/'.length), model));
    return {schema: 'native-readable-source-shard/v1', identity: {...identity}, shards: 2, shardIndex: index,
      receiptAuthority: 'navigation-and-producer-evidence-only', models: [{path: entry.path, member,
        sourceBlake3: hexadecimal(entry.bytes), modelBlake3: hexadecimal(Buffer.from(retained)),
        sourceFreeRestoration: true, independentGrammarReparse: true}]};
  });
  return {sources, receipts};
}

test('synthetic shard receipts cover every actual declared byte source and refer to separate recursive models', () => {
  const {sources, receipts} = fixture();
  const result = aggregateSourceContent(receipts, sources, identity);
  assert.equal(result.receiptAuthority, 'navigation-and-producer-evidence-only');
  assert.equal(result.representation, 'external-full-recursive-UTF8-Links-models');
  assert.deepEqual(result.models.map(row => row.path), sources.map(row => row.path));
  assert.deepEqual(result.models.map(row => row.artifact), ['source-content-models-0', 'source-content-models-1']);
  assert.equal(result.shards, 2);
  assert(!('source' in result), 'receipt is not source content');
});

test('receipt aggregation refuses missing, repeated, foreign, unordered-identity and invalid content evidence', () => {
  const mutations = [
    value => value.pop(),
    value => value[1].shardIndex = 0,
    value => value[0].identity.commit = 'c'.repeat(40),
    value => value[0].identity.attempt = '2',
    value => value[0].models[0].sourceBlake3 = '0'.repeat(64),
    value => value[0].models[0].modelBlake3 = 'not-a-digest',
    value => value[0].models[0].member = '../outside.lino',
    value => value[0].models[0].path = 'rust/src/unowned.rs',
    value => value[0].models[0].sourceFreeRestoration = false,
    value => value[0].models[0].independentGrammarReparse = false,
    value => value[0].models.push({...value[1].models[0]}),
    value => value[0].receiptAuthority = 'full-source',
    value => value[1].models = [],
  ];
  for (const [index, mutate] of mutations.entries()) {
    const {sources, receipts} = fixture(); mutate(receipts);
    assert.throws(() => aggregateSourceContent(receipts, sources, identity), undefined, 'mutation ' + index);
  }
  const {sources, receipts} = fixture();
  assert.throws(() => aggregateSourceContent(receipts, [...sources, sources[0]], identity));
  assert.throws(() => aggregateSourceContent([], sources, identity));
});
