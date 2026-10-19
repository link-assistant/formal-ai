// Complete readable CST packets. Identities authenticate content; they never replace it.
import { createHash } from 'node:crypto';
import { gzipSync, gunzipSync } from 'node:zlib';
import { sourceToLinks, sourceFromLinks, readSourceLinks, writeSourceLinks, ownedSourceLinks, sourceOwner, verifySourceLinks } from '../../js/meta/readable-source.js';
export const sourceDigest = bytes => createHash('sha256').update(bytes).digest('hex');
const canonicalLanguage = language => language === 'rust' ? 'Rust' : language;
const decoder = new TextDecoder('utf-8', {
  fatal: true,
  ignoreBOM: true
});

/** Validate before a caller uses the packet member as a filesystem name. */
export function validateSourceCstReceipt(receipt, expectedPath) {
  if (typeof expectedPath !== 'string' || !expectedPath || expectedPath.includes('\\') || expectedPath.includes(':') || expectedPath.includes('\0') || expectedPath.split('/').some(part => !part || ['.', '..', '.git'].includes(part))) {
    throw Error('UnsafeSourceCstOwner');
  }
  if (receipt?.schema !== 'readable-recursive-source-cst/v1' || receipt.path !== expectedPath || receipt.packet !== expectedPath + '.cst.lino.gz' || receipt.fidelity !== 'readable-recursive-cst' || !['JavaScript', 'TypeScript', 'Rust'].includes(receipt.language)) {
    throw Error('SourceCstReceiptIdentityMismatch');
  }
  for (const key of ['source', 'serialized', 'compressed']) {
    if (!Number.isSafeInteger(receipt[key + 'Bytes']) || receipt[key + 'Bytes'] < (key === 'source' ? 0 : 1) || !/^[a-f0-9]{64}$/u.test(receipt[key + 'Sha256'] ?? '')) throw Error('InvalidSourceCstIdentity');
  }
}

/** Serialize an actual complete model. This operation claims restoration, not understanding. */
export function packSourceCstModel({
  path,
  source,
  language
}, model) {
  const owned = sourceOwner(model) === null ? ownedSourceLinks(path, model) : model;
  if (sourceOwner(owned) !== path || sourceFromLinks(owned) !== source) throw Error('SourceCstPreimageMismatch');
  const actualLanguage = owned[2].find(item => Array.isArray(item) && item[0] === 'language')?.[1]?.quoted;
  if (actualLanguage !== canonicalLanguage(language)) throw Error('SourceCstLanguageMismatch');
  const document = writeSourceLinks(owned),
    serialized = Buffer.from(document, 'utf8');
  const compressed = gzipSync(serialized),
    sourceBytes = Buffer.from(source, 'utf8');
  const receipt = {
    schema: 'readable-recursive-source-cst/v1',
    path,
    language: actualLanguage,
    packet: path + '.cst.lino.gz',
    fidelity: 'readable-recursive-cst',
    sourceBytes: sourceBytes.length,
    sourceSha256: sourceDigest(sourceBytes),
    serializedBytes: serialized.length,
    serializedSha256: sourceDigest(serialized),
    compressedBytes: compressed.length,
    compressedSha256: sourceDigest(compressed)
  };
  const restored = restoreSourceCstPacket(receipt, compressed, path);
  if (restored.source !== source) throw Error('SourceCstRestorationMismatch');
  return {
    compressed,
    receipt
  };
}

/** Reload only the persisted recursive content; original source files are never consulted. */
export function restoreSourceCstPacket(receipt, compressed, expectedPath = receipt?.path) {
  validateSourceCstReceipt(receipt, expectedPath);
  if (!Buffer.isBuffer(compressed) || compressed.length !== receipt.compressedBytes || sourceDigest(compressed) !== receipt.compressedSha256) throw Error('SourceCstCompressedIdentityMismatch');
  const serialized = gunzipSync(compressed, {
    maxOutputLength: receipt.serializedBytes
  });
  if (serialized.length !== receipt.serializedBytes || sourceDigest(serialized) !== receipt.serializedSha256) {
    throw Error('SourceCstSerializedIdentityMismatch');
  }
  const document = readSourceLinks(decoder.decode(serialized));
  if (sourceOwner(document) !== expectedPath) throw Error('SourceCstOwnerMismatch');
  const actualLanguage = document[2].find(item => Array.isArray(item) && item[0] === 'language')?.[1]?.quoted;
  if (actualLanguage !== receipt.language) throw Error('SourceCstLanguageMismatch');
  const source = sourceFromLinks(document),
    bytes = Buffer.from(source, 'utf8');
  if (bytes.length !== receipt.sourceBytes || sourceDigest(bytes) !== receipt.sourceSha256) {
    throw Error('SourceCstSourceIdentityMismatch');
  }
  return {
    document,
    source
  };
}

/** The pinned actual grammar must accept both original source and fresh packet restoration. */
export function createSourceCstPacket(options, frontend) {
  const language = canonicalLanguage(options.language);
  const model = sourceToLinks(options.source, language, frontend);
  const packet = packSourceCstModel({
    ...options,
    language
  }, model);
  const restored = restoreSourceCstPacket(packet.receipt, packet.compressed, options.path);
  verifySourceLinks(restored.document, frontend);
  return packet;
}
