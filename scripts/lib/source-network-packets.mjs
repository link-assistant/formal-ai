// Lossless network packets for owned source: full serialized bytes and verified reconstruction.
import { createHash } from 'node:crypto';
import { gzipSync, gunzipSync } from 'node:zlib';

export const sourceDigest = (bytes) => createHash('sha256').update(bytes).digest('hex');

/** Disjoint, exhaustive shards over a canonical source path list. */
export function partitionSourcePaths(paths, shardIndex, shardCount) {
  if (!Number.isSafeInteger(shardCount) || shardCount < 1 || !Number.isSafeInteger(shardIndex)
    || shardIndex < 0 || shardIndex >= shardCount) throw new Error('invalid source shard');
  if (paths.some((path) => typeof path !== 'string') || new Set(paths).size !== paths.length) {
    throw new Error('source paths must be distinct strings');
  }
  return [...paths].sort((left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right)))
    .filter((_, index) => index % shardCount === shardIndex);
}

/** Encode the complete pinned parser network without discarding its AST or metadata. */
function sourceNetworkDocument({ path, language, source }, LinkNetwork) {
  const sourceBytes = Buffer.from(source, 'utf8');
  const document = LinkNetwork.parse(source, language).toLino();
  const serializedBytes = Buffer.from(document, 'utf8');
  return {
    document,
    receipt: {
      path, language, sourceBytes: sourceBytes.length, sourceSha256: sourceDigest(sourceBytes),
      serializedBytes: serializedBytes.length, serializedSha256: sourceDigest(serializedBytes),
      fidelity: 'lossless-network-serialization',
    },
  };
}

/** The upstream decoder must preserve actual source bytes, not its digest alone. */
export function serializeSourceNetwork(options, LinkNetwork) {
  const result = sourceNetworkDocument(options, LinkNetwork);
  const restored = LinkNetwork.fromLino(result.document).reconstructText();
  if (typeof restored !== 'string' || !Buffer.from(restored, 'utf8').equals(Buffer.from(options.source, 'utf8'))) {
    throw new Error('source network round trip differs: ' + options.path);
  }
  return result;
}

/** Decode the actual compressed packet exactly once after its network and gzip identities are bound. */
export function createSourcePacket(options, LinkNetwork) {
  const { document, receipt } = sourceNetworkDocument(options, LinkNetwork);
  const compressed = gzipSync(Buffer.from(document, 'utf8'));
  const packetReceipt = { ...receipt, packet: options.path + '.lino.gz',
    compressedBytes: compressed.length, compressedSha256: sourceDigest(compressed) };
  if (verifySourcePacket(packetReceipt, compressed, LinkNetwork) !== options.source) {
    throw new Error('packet reconstruction differs: ' + options.path);
  }
  return { compressed, receipt: packetReceipt };
}

/** Validate a downloaded packet's compressed/network identities and reconstruct the recorded source bytes. */
export function verifySourcePacket(receipt, compressed, LinkNetwork) {
  if (sourceDigest(compressed) !== receipt.compressedSha256 || compressed.length !== receipt.compressedBytes) {
    throw new Error('compressed packet identity differs: ' + receipt.path);
  }
  const bytes = gunzipSync(compressed);
  if (sourceDigest(bytes) !== receipt.serializedSha256 || bytes.length !== receipt.serializedBytes) {
    throw new Error('serialized network identity differs: ' + receipt.path);
  }
  const source = LinkNetwork.fromLino(new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(bytes)).reconstructText();
  const restored = Buffer.from(source, 'utf8');
  if (restored.length !== receipt.sourceBytes || sourceDigest(restored) !== receipt.sourceSha256) {
    throw new Error('reconstructed source identity differs: ' + receipt.path);
  }
  return source;
}


/** Certify exact shard coverage and packet identities against a committed release source tree.
 * Producers already ran the full upstream decoder; this join checks their content-bound packets.
 */
export function verifySourceDistribution({ reports, sourcePaths, sourceBytes, packetBytes,
  sourceHead, upstreamCommit, shardCount = 8 }) {
  if (!/^[a-f0-9]{40,64}$/u.test(sourceHead) || !/^[a-f0-9]{40}$/u.test(upstreamCommit)) {
    throw new Error('distribution requires full source and upstream commits');
  }
  if (reports.length !== shardCount) throw new Error('source distribution is missing a shard');
  const seen = new Set();
  for (const report of reports) {
    if (!Number.isSafeInteger(report.shardIndex) || report.shardIndex < 0
      || report.shardIndex >= shardCount || seen.has(report.shardIndex)) {
      throw new Error('source distribution shard identity differs');
    }
    seen.add(report.shardIndex);
    const paths = partitionSourcePaths(sourcePaths, report.shardIndex, shardCount);
    if (report.sourceHead !== sourceHead || report.baseHead !== sourceHead
      || report.inputState !== 'committed-head' || report.upstreamCommit !== upstreamCommit
      || report.shardCount !== shardCount || report.totalOwnedSources !== sourcePaths.length
      || report.checkedSources !== paths.length || report.fidelity !== 'lossless-network-serialization'
      || !Array.isArray(report.sources) || report.sources.length !== paths.length) {
      throw new Error('source distribution report identity differs');
    }
    for (let index = 0; index < paths.length; index += 1) {
      const receipt = report.sources[index], path = paths[index];
      if (receipt.path !== path || receipt.packet !== path + '.lino.gz'
        || receipt.fidelity !== 'lossless-network-serialization') {
        throw new Error('source distribution coverage differs');
      }
      const source = sourceBytes(path), compressed = packetBytes(report.shardIndex, receipt.packet);
      if (source.length !== receipt.sourceBytes || sourceDigest(source) !== receipt.sourceSha256) {
        throw new Error('release source identity differs: ' + path);
      }
      if (compressed.length !== receipt.compressedBytes || sourceDigest(compressed) !== receipt.compressedSha256) {
        throw new Error('distribution compressed packet identity differs: ' + path);
      }
      const serialized = gunzipSync(compressed);
      if (serialized.length !== receipt.serializedBytes || sourceDigest(serialized) !== receipt.serializedSha256) {
        throw new Error('distribution serialized packet identity differs: ' + path);
      }
    }
  }
  return { sourceHead, upstreamCommit, shardCount, checkedSources: sourcePaths.length };
}
