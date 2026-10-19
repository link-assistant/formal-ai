import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { gzipSync } from 'node:zlib';
import { partitionSourcePaths, sourceDigest, verifySourceDistribution } from '../../../scripts/lib/source-network-packets.mjs';
const { parse } = createRequire(import.meta.url)('yaml');
const read = (path) => readFileSync(new URL('../../../' + path, import.meta.url), 'utf8');

test('both production schedules and publication join require the same exhaustive sixteen shards', () => {
  const layered = parse(read('.github/workflows/layered-ci.yml'));
  const publication = parse(read('.github/workflows/publish-source-networks.yml'));
  const indices = Array.from({ length: 16 }, (_, index) => index);
  assert.deepEqual(layered.jobs['source-networks'].strategy.matrix.shard, indices);
  assert.deepEqual(publication.jobs.packets.strategy.matrix.shard, indices);
  for (const job of [layered.jobs['source-networks'], publication.jobs.packets]) {
    const step = job.steps.find((entry) => entry.run?.includes('check-source-networks.mjs'));
    assert.equal(step.env.TEST_BUDGET_SECONDS, 600);
    assert.match(step.run, /--shard-count 16(?:\s|$)/u);
  }
  const joinStep = publication.jobs.attach.steps.find((entry) => entry.run?.includes('verify-source-network-distribution.mjs'));
  assert.match(joinStep.run, /--shard-count 16(?:\s|$)/u);
  const extraction = joinStep.run.match(/for shard in ([0-9 ]+); do/u);
  assert.ok(extraction);
  assert.deepEqual(extraction[1].split(' ').map(Number), indices);
  const verifier = read('scripts/verify-source-network-distribution.mjs');
  assert.match(verifier, /Number\(value\('--shard-count', '8'\)\)/u);
  assert.match(verifier, /Array\.from\(\{ length: shardCount \}/u);
  assert.match(verifier, /upstreamCommit, shardCount,/u);
});

test('sixteen complete source packets join without omissions or parser substitution', () => {
  const sourceHead = 'a'.repeat(40), upstreamCommit = 'b'.repeat(40), shardCount = 16;
  const sourcePaths = Array.from({ length: 97 }, (_, index) => ['js/', 'ts/', 'rust/src/'][index % 3] + 'held-out-' + index + '.source');
  const sourceBytes = new Map(sourcePaths.map((path, index) => [path, Buffer.from('// π 😀\r\n' + index + ' '.repeat(index % 11))]));
  const packets = new Map();
  const reports = Array.from({ length: shardCount }, (_, shardIndex) => {
    const paths = partitionSourcePaths(sourcePaths, shardIndex, shardCount, (path) => sourceBytes.get(path).length);
    const sources = paths.map((path) => {
      // Synthetic already-serialized packet fixtures test the strict join, not live SDK parsing.
      const source = sourceBytes.get(path), serialized = Buffer.from('(synthetic-network ' + source.toString('base64') + ')');
      const compressed = gzipSync(serialized), packet = path + '.lino.gz';
      packets.set(shardIndex + '/' + packet, compressed);
      return { path, packet, sourceBytes: source.length, sourceSha256: sourceDigest(source), serializedBytes: serialized.length,
        serializedSha256: sourceDigest(serialized), compressedBytes: compressed.length, compressedSha256: sourceDigest(compressed),
        fidelity: 'lossless-network-serialization' };
    });
    return { sourceHead, baseHead: sourceHead, inputState: 'committed-head', upstreamCommit, shardIndex, shardCount,
      totalOwnedSources: sourcePaths.length, checkedSources: sources.length, fidelity: 'lossless-network-serialization', sources };
  });
  const options = { reports, sourcePaths, sourceHead, upstreamCommit, shardCount,
    sourceBytes: (path) => sourceBytes.get(path), packetBytes: (index, path) => packets.get(index + '/' + path) };
  assert.equal(verifySourceDistribution(options).checkedSources, 97);
  const assigned = reports.flatMap((report) => report.sources.map((receipt) => receipt.path));
  assert.deepEqual([...assigned].sort(), [...sourcePaths].sort());
  assert.equal(new Set(assigned).size, sourcePaths.length);
  assert.deepEqual(Array.from({ length: shardCount }, (_, index) => partitionSourcePaths([...sourcePaths].reverse(), index, shardCount,
    (path) => sourceBytes.get(path).length)), reports.map((report) => report.sources.map((receipt) => receipt.path)));
  assert.throws(() => verifySourceDistribution({ ...options, reports: reports.slice(0, 15) }), /missing a shard/u);
  assert.throws(() => verifySourceDistribution({ ...options, reports: [reports[0], ...reports.slice(0, 15)] }), /shard identity/u);
  assert.throws(() => verifySourceDistribution({ ...options, shardCount: 8 }), /missing a shard/u);
  const foreign = structuredClone(reports); foreign[15].sources[0].path = '../foreign';
  assert.throws(() => verifySourceDistribution({ ...options, reports: foreign }), /coverage differs/u);
  const stale = structuredClone(reports); stale[15].sources[0].sourceSha256 = '0'.repeat(64);
  assert.throws(() => verifySourceDistribution({ ...options, reports: stale }), /source identity/u);
  assert.throws(() => verifySourceDistribution({ ...options, packetBytes: () => Buffer.from('tampered') }), /compressed packet/u);
});
