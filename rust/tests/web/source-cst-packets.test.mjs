import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { gzipSync } from 'node:zlib';
import { readSourceLinks, sourceFromLinks, ownedSourceLinks, writeSourceLinks } from '../../../js/meta/readable-source.js';
import { packSourceCstModel, restoreSourceCstPacket, validateSourceCstReceipt, sourceDigest } from '../../../scripts/lib/source-cst-packets.mjs';
import { partitionSourcePaths, verifySourceDistribution } from '../../../scripts/lib/source-network-packets.mjs';
const cases = [['JavaScript', 'js/保留.mjs'], ['TypeScript', 'ts/保留.mts'], ['Rust', 'rust/src/保留.rs']];
function fixture(language, path) {
  const model = readSourceLinks(readFileSync(new URL('../fixtures/source-model/' + language + '.lino', import.meta.url), 'utf8'));
  const source = sourceFromLinks(model);
  return {
    model,
    source,
    packet: packSourceCstModel({
      path,
      language,
      source
    }, model)
  };
}
for (const [language, path] of cases) {
  test(language + ' packet owns readable recursive content and restores without the original file', t => {
    const {
      source,
      packet
    } = fixture(language, path);
    const directory = mkdtempSync(join(tmpdir(), 'formal-ai-cst-packet-'));
    t.after(() => rmSync(directory, {
      recursive: true,
      force: true
    }));
    const original = join(directory, 'original.source'),
      receipt = join(directory, 'receipt.json'),
      bytes = join(directory, 'model.gz');
    writeFileSync(original, source);
    writeFileSync(receipt, JSON.stringify(packet.receipt));
    writeFileSync(bytes, packet.compressed);
    rmSync(original);
    const adapter = new URL('../../../scripts/lib/source-cst-packets.mjs', import.meta.url).href;
    const code = 'import fs from "node:fs";import {restoreSourceCstPacket} from ' + JSON.stringify(adapter) + ';const receipt=JSON.parse(fs.readFileSync(process.argv[1]));' + 'process.stdout.write(restoreSourceCstPacket(receipt,fs.readFileSync(process.argv[2])).source);';
    const restored = execFileSync(process.execPath, ['--input-type=module', '-e', code, receipt, bytes], {
      encoding: 'utf8',
      timeout: 10000
    });
    assert.equal(restored, source);
    const result = restoreSourceCstPacket(packet.receipt, packet.compressed, path);
    assert.equal(result.source, source);
    const notation = writeSourceLinks(result.document);
    assert.match(notation, /\(owned-source/u);
    assert.match(notation, /\(form /u);
    assert(!notation.includes('tokenIndex'));
    assert(!notation.includes('byteRange'));
    assert.equal(packet.receipt.fidelity, 'readable-recursive-cst');
    assert.equal(result.document[2][0], 'source');
  });
  test(language + ' packet rejects changed source, owner, language and each corrupted identity', () => {
    const {
      source,
      model,
      packet
    } = fixture(language, path);
    assert.throws(() => packSourceCstModel({
      path,
      language,
      source: source + '\n'
    }, model), /Preimage/u);
    assert.throws(() => packSourceCstModel({
      path,
      language,
      source
    }, ownedSourceLinks('foreign/path.rs', model)), /Preimage/u);
    for (const key of ['sourceSha256', 'serializedSha256', 'compressedSha256']) {
      assert.throws(() => restoreSourceCstPacket({
        ...packet.receipt,
        [key]: '0'.repeat(64)
      }, packet.compressed));
    }
    for (const key of ['sourceBytes', 'serializedBytes', 'compressedBytes']) {
      assert.throws(() => restoreSourceCstPacket({
        ...packet.receipt,
        [key]: packet.receipt[key] + 1
      }, packet.compressed));
    }
    assert.throws(() => restoreSourceCstPacket(packet.receipt, Buffer.from('damaged')));
    assert.throws(() => restoreSourceCstPacket(packet.receipt, packet.compressed, 'foreign/path.rs'));
    const foreign = {
      ...packet.receipt,
      language: language === 'Rust' ? 'JavaScript' : 'Rust'
    };
    assert.throws(() => restoreSourceCstPacket(foreign, packet.compressed), /Language/u);
  });
}
test('unsafe packet names refuse before any caller reads a member', () => {
  const {
    packet
  } = fixture(...cases[0]);
  for (const path of ['../outside.mjs', '/outside.mjs', 'js/../outside.mjs', 'js//outside.mjs', 'C:/outside.mjs', 'js\\outside.mjs', 'js/\0bad', '.git/config']) {
    assert.throws(() => validateSourceCstReceipt({
      ...packet.receipt,
      path,
      packet: path + '.cst.lino.gz'
    }, path));
  }
  for (const packetPath of ['../outside.gz', packet.receipt.packet + '.extra', 'js/missing.gz']) {
    assert.throws(() => validateSourceCstReceipt({
      ...packet.receipt,
      packet: packetPath
    }, packet.receipt.path));
  }
});
test('a histogram or digest cannot stand in for the complete persisted model', () => {
  const {
    packet
  } = fixture(...cases[0]);
  for (const notation of ['source_digest only', '(source_census_diagnostic (named_node_count 42))', '(source (language "JavaScript"))']) {
    const serialized = Buffer.from(notation),
      compressed = gzipSync(serialized);
    const forged = {
      ...packet.receipt,
      serializedBytes: serialized.length,
      serializedSha256: sourceDigest(serialized),
      compressedBytes: compressed.length,
      compressedSha256: sourceDigest(compressed)
    };
    assert.throws(() => restoreSourceCstPacket(forged, compressed));
  }
});
test('strict release join checks actual CST bytes for all three roots and refuses legacy-only success', () => {
  const head = 'a'.repeat(40),
    upstreamCommit = 'b'.repeat(40);
  const entries = cases.map(([language, path]) => ({
    language,
    path,
    ...fixture(language, path)
  }));
  const byPath = new Map(entries.map(entry => [entry.path, entry]));
  const sourcePaths = entries.map(entry => entry.path);
  const paths = partitionSourcePaths(sourcePaths, 0, 1, path => Buffer.byteLength(byPath.get(path).source));
  const packets = new Map();
  const sources = paths.map(path => {
    const entry = byPath.get(path);
    packets.set(entry.packet.receipt.packet, entry.packet.compressed);
    return entry.packet.receipt;
  });
  const report = {
    sourceHead: head,
    baseHead: head,
    inputState: 'committed-head',
    upstreamCommit,
    shardIndex: 0,
    shardCount: 1,
    totalOwnedSources: paths.length,
    checkedSources: paths.length,
    fidelity: 'readable-recursive-cst',
    readableCstFidelity: 'readable-recursive-cst',
    sources
  };
  const options = {
    reports: [report],
    sourcePaths,
    sourceHead: head,
    upstreamCommit,
    shardCount: 1,
    requireReadableCst: true,
    sourceBytes: path => Buffer.from(byPath.get(path).source),
    packetBytes: (_index, path) => packets.get(path)
  };
  assert.equal(verifySourceDistribution(options).checkedSources, paths.length);
  for (const mutate of [copy => {
    delete copy.readableCstFidelity;
  }, copy => {
    delete copy.sources[0].fidelity;
  }, copy => {
    copy.sources[0].packet = '../outside.gz';
  }, copy => {
    copy.sources[0].sourceSha256 = '0'.repeat(64);
  }, copy => {
    copy.sources[0] = copy.sources[1];
  }]) {
    const copy = structuredClone(report);
    mutate(copy);
    assert.throws(() => verifySourceDistribution({
      ...options,
      reports: [copy]
    }));
  }
  assert.throws(() => verifySourceDistribution({
    ...options,
    packetBytes: (_index, path) => path.endsWith('.cst.lino.gz') ? Buffer.from('changed actual CST') : packets.get(path)
  }));
});
