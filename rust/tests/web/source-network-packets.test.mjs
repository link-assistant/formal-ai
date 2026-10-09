import assert from 'node:assert/strict';
import test from 'node:test';
import { gzipSync } from 'node:zlib';
import { workflowPin } from '../../../scripts/translate-js-rust.mjs';
import { partitionSourcePaths, serializeSourceNetwork, createSourcePacket, sourceDigest, verifySourcePacket, verifySourceDistribution } from '../../../scripts/lib/source-network-packets.mjs';

test('every source has exactly one stable shard, including non-ASCII paths', () => {
  const paths = ['rust/src/α.rs', 'js/a.mjs', 'ts/a.mts', 'rust/src/z.rs', 'js/😀.js'];
  const shards = Array.from({ length: 8 }, (_, index) => partitionSourcePaths(paths, index, 8));
  assert.deepEqual(shards.flat().sort(), [...paths].sort());
  assert.equal(new Set(shards.flat()).size, paths.length);
  assert.deepEqual(shards, Array.from({ length: 8 }, (_, index) => partitionSourcePaths([...paths].reverse(), index, 8)));
});

test('duplicate paths and invalid shard identities refuse before serialization', () => {
  for (const [paths, index, count] of [[['a', 'a'], 0, 1], [[], -1, 2], [[], 2, 2], [[], 0, 0], [[], 0.1, 2]]) {
    assert.throws(() => partitionSourcePaths(paths, index, count));
  }
});

test('receipt records serialized bytes and exact UTF-8 source, independently of line endings', () => {
  const source = '\uFEFF// 😀\r\nconst value = "α";';
  const LinkNetwork = {
    parse: (text) => ({ toLino: () => '(1: (meta: (def: ' + encodeURIComponent(text) + ')))\n' }),
    fromLino: (document) => ({ reconstructText: () => decodeURIComponent(document.match(/\(def: ([^)]*)\)/u)[1]) }),
  };
  const result = serializeSourceNetwork({ path: 'js/arbitrary.mjs', language: 'JavaScript', source }, LinkNetwork);
  assert.equal(result.receipt.sourceBytes, Buffer.byteLength(source));
  assert.equal(result.receipt.serializedBytes, Buffer.byteLength(result.document));
  assert.notEqual(result.receipt.sourceSha256, result.receipt.serializedSha256);
  assert.equal(result.receipt.fidelity, 'lossless-network-serialization');
});

test('a source digest or altered reconstruction never counts as a complete network', () => {
  const source = 'let value = 2;\n';
  for (const restored of ['let value = 3;\n', source.trimEnd(), '', null]) {
    assert.throws(() => serializeSourceNetwork({ path: 'arbitrary.rs', language: 'rust', source }, {
      parse: () => ({ toLino: () => 'source_digest only' }),
      fromLino: () => ({ reconstructText: () => restored }),
    }), /round trip differs/u);
  }
});

test('decoder failures are propagated without a success receipt', () => {
  assert.throws(() => serializeSourceNetwork({ path: 'arbitrary.rs', language: 'rust', source: 'fn f() {}' }, {
    parse: () => ({ toLino: () => 'invalid serialized schema' }),
    fromLino: () => { throw new Error('malformed network'); },
  }), /malformed network/u);
});

test('all workflow serializer checkouts share one verified pin', () => {
  const first = 'a'.repeat(40), second = 'b'.repeat(40);
  const checkout = (commit) => 'repository: link-foundation/meta-language\n  ref: ' + commit + '\n';
  assert.equal(workflowPin(checkout(first) + checkout(first)), first);
  assert.equal(workflowPin(checkout(first) + checkout(second)), null);
  assert.equal(workflowPin('no upstream checkout'), null);
});

test('downloaded gzip packet reconstructs the exact source and refuses identity corruption', () => {
  const source = '// 😀\r\nconst value = 9;';
  const document = '(1: (meta: (def: ' + encodeURIComponent(source) + ')))\n';
  const compressed = gzipSync(Buffer.from(document));
  const receipt = {
    path: 'js/arbitrary.mjs', sourceBytes: Buffer.byteLength(source), sourceSha256: sourceDigest(Buffer.from(source)),
    serializedBytes: Buffer.byteLength(document), serializedSha256: sourceDigest(Buffer.from(document)),
    compressedBytes: compressed.length, compressedSha256: sourceDigest(compressed),
  };
  const LinkNetwork = { fromLino: (text) => ({ reconstructText: () => decodeURIComponent(text.match(/\(def: ([^)]*)\)/u)[1]) }) };
  assert.equal(verifySourcePacket(receipt, compressed, LinkNetwork), source);
  assert.throws(() => verifySourcePacket(receipt, Buffer.from('corrupt compressed bytes'), LinkNetwork), /compressed packet identity/u);
  assert.throws(() => verifySourcePacket({ ...receipt, serializedSha256: '0'.repeat(64) }, compressed, LinkNetwork),
    /serialized network identity/u);
  assert.throws(() => verifySourcePacket({ ...receipt, sourceBytes: receipt.sourceBytes - 1 }, compressed, LinkNetwork),
    /reconstructed source identity/u);
  assert.throws(() => verifySourcePacket(receipt, compressed, { fromLino: () => ({ reconstructText: () => source + '\n' }) }),
    /reconstructed source identity/u);
});


test('release distribution rejects missing, duplicate, stale, forged and corrupt source packets', () => {
  const sourceHead = 'a'.repeat(40), upstreamCommit = 'b'.repeat(40);
  const sourcePaths = Array.from({ length: 16 }, (_, index) => 'js/module-' + index + '.mjs');
  const sources = new Map(sourcePaths.map((path, index) => [path, Buffer.from('export const value = ' + index + ';\n')]));
  const packets = new Map();
  const reports = Array.from({ length: 8 }, (_, shardIndex) => {
    const paths = partitionSourcePaths(sourcePaths, shardIndex, 8);
    const receipts = paths.map((path) => {
      const source = sources.get(path), document = Buffer.from('(1: (def: ' + source.toString('base64') + '))\n');
      const packet = path + '.lino.gz', compressed = gzipSync(document);
      packets.set(shardIndex + '/' + packet, compressed);
      return { path, packet, sourceBytes: source.length, sourceSha256: sourceDigest(source),
        serializedBytes: document.length, serializedSha256: sourceDigest(document),
        compressedBytes: compressed.length, compressedSha256: sourceDigest(compressed),
        fidelity: 'lossless-network-serialization' };
    });
    return { sourceHead, baseHead: sourceHead, inputState: 'committed-head', upstreamCommit,
      shardIndex, shardCount: 8, totalOwnedSources: 16, checkedSources: paths.length,
      fidelity: 'lossless-network-serialization', sources: receipts };
  });
  const options = { reports, sourcePaths, sourceHead, upstreamCommit,
    sourceBytes: (path) => sources.get(path), packetBytes: (index, path) => packets.get(index + '/' + path) };
  assert.equal(verifySourceDistribution(options).checkedSources, 16);
  assert.throws(() => verifySourceDistribution({ ...options, reports: reports.slice(1) }), /missing a shard/u);
  assert.throws(() => verifySourceDistribution({ ...options, reports: [reports[0], ...reports.slice(0, 7)] }), /shard identity/u);
  for (const change of [
    (copy) => { copy[0].sourceHead = 'c'.repeat(40); },
    (copy) => { copy[0].upstreamCommit = 'c'.repeat(40); },
    (copy) => { copy[0].sources[0].path = '../outside.mjs'; },
    (copy) => { copy[0].sources[0].sourceSha256 = '0'.repeat(64); },
    (copy) => { copy[0].sources.reverse(); },
  ]) {
    const copy = structuredClone(reports); change(copy);
    assert.throws(() => verifySourceDistribution({ ...options, reports: copy }));
  }
  assert.throws(() => verifySourceDistribution({ ...options, packetBytes: () => Buffer.from('corrupt') }), /compressed packet/u);
  assert.throws(() => verifySourceDistribution({ ...options, sourceBytes: () => Buffer.from('changed source') }), /source identity/u);
});

test('production source producer imports without launching serialization and enumerates tracked production roots', async () => {
  const { main, ownedProductionSources } = await import('../../../scripts/check-source-networks.mjs');
  assert.equal(typeof main, 'function');
  const paths = ownedProductionSources();
  for (const root of ['rust/src/', 'js/', 'ts/']) assert.ok(paths.some((path) => path.startsWith(root)));
  assert.equal(paths.length, new Set(paths).size);
  assert.ok(paths.every((path) => !path.startsWith('rust/tests/') && !path.startsWith('js/vendor/')
    && !path.startsWith('js/seed/') && !path.endsWith('.bundle.js')));
});


test('compressed producer reconstructs its actual network once and preserves all source bytes', () => {
  const source = '\uFEFF// 😀\r\nconst value = "α";';
  let parses = 0, decodes = 0;
  const LinkNetwork = {
    parse: (text) => { parses += 1; return { toLino: () => '(1: (meta: (def: ' + encodeURIComponent(text) + ')))\n' }; },
    fromLino: (document) => { decodes += 1; return { reconstructText: () =>
      decodeURIComponent(document.match(/\(def: ([^)]*)\)/u)[1]) }; },
  };
  const result = createSourcePacket({ path: 'js/arbitrary.mjs', language: 'JavaScript', source }, LinkNetwork);
  assert.equal(parses, 1);
  assert.equal(decodes, 1);
  assert.equal(result.receipt.sourceSha256, sourceDigest(Buffer.from(source)));
  assert.equal(verifySourcePacket(result.receipt, result.compressed, LinkNetwork), source);
  assert.equal(decodes, 2);
  assert.throws(() => createSourcePacket({ path: 'js/arbitrary.mjs', language: 'JavaScript', source }, {
    ...LinkNetwork, fromLino: () => ({ reconstructText: () => source.trimEnd() + '\n' }),
  }), /reconstructed source identity/u);
});
