// Bounded admission for mutable source captures; retained evidence is never evicted here.
import fs from 'node:fs';
import path from 'node:path';
import {createHash, randomUUID} from 'node:crypto';
import {readBoundedFilePath} from './bounded-file.mjs';

export const SOURCE_CACHE_POLICY = Object.freeze(JSON.parse(fs.readFileSync(
  new URL('../../data/meta/source-cache-budget.json', import.meta.url), 'utf8')));
const LOCK = '.source-cache-write.lock';

function validatePolicy(policy) {
  if (policy.format !== 'source-cache-budget/v1' || !['maxBytes', 'maxFiles', 'maxEntries', 'maxBodyBytes', 'maxMetadataBytes']
    .every(key => Number.isSafeInteger(policy[key]) && policy[key] > 0) || policy.maxBodyBytes > policy.maxBytes)
    throw Error('source_cache_policy_invalid');
}
validatePolicy(SOURCE_CACHE_POLICY);

function regularDirectory(directory) {
  const state = fs.lstatSync(directory);
  if (!state.isDirectory() || state.isSymbolicLink()) throw Error('source_cache_directory_type:' + directory);
}

/** Enumerate at most the declared file budget; no recursive or unrelated-directory traversal. */
export function sourceCacheUsage(root, policy = SOURCE_CACHE_POLICY) {
  validatePolicy(policy);
  let bytes = 0, files = 0, entries = 0;
  const visit = (directory, objects) => {
    regularDirectory(directory);
    const handle = fs.opendirSync(directory);
    try {
      for (let entry = handle.readSync(); entry !== null; entry = handle.readSync()) {
        const filename = path.join(directory, entry.name);
        const state = fs.lstatSync(filename);
        if (!objects && entry.name === 'objects' && state.isDirectory() && !state.isSymbolicLink()) {
          visit(filename, true);
          continue;
        }
        if (!state.isFile() || state.isSymbolicLink()) throw Error('source_cache_entry_type:' + filename);
        if (!objects && entry.name === LOCK) continue;
        files += 1;
        bytes += state.size;
        if (!objects && entry.name.endsWith('.meta')) entries += 1;
        if (!Number.isSafeInteger(bytes) || files > policy.maxFiles || bytes > policy.maxBytes || entries > policy.maxEntries)
          throw Error('source_cache_budget_exceeded:' + root);
      }
    } finally { handle.closeSync(); }
  };
  visit(root, false);
  return {bytes, files, entries};
}

function existingRegular(filename) {
  try {
    if (!fs.lstatSync(filename).isFile()) throw Error('source_cache_entry_type:' + filename);
    return true;
  } catch (error) {
    if (error.code === 'ENOENT') return false;
    throw error;
  }
}

/** One fail-fast lock attempt, shared with native writers; stale locks never cause a retry loop. */
function cleanupFailure(primary, errors) {
  if (primary && !errors.length) throw primary;
  if (errors.length) throw new AggregateError(primary ? [primary, ...errors] : errors,
    (primary ? primary.message + '; ' : '') + 'source_cache_cleanup_failed:' + errors.map(error => error.message).join(';'),
    primary ? {cause: primary} : undefined);
}
function withWriteLock(root, action) {
  const filename = path.join(root, LOCK);
  const descriptor = fs.openSync(filename, 'wx', 0o600);
  let identity;
  try { identity = fs.fstatSync(descriptor, {bigint: true}); }
  catch (primary) { const errors = []; try { fs.closeSync(descriptor); } catch (error) { errors.push(error); } cleanupFailure(primary, errors); }
  let result, primary;
  try { result = action(); } catch (error) { primary = error; }
  const errors = [];
  try { fs.closeSync(descriptor); } catch (error) { errors.push(error); }
  try {
    const current = fs.lstatSync(filename, {bigint: true});
    if (!current.isFile() || current.dev !== identity.dev || current.ino !== identity.ino)
      throw Error('source_cache_lock_custody:' + filename);
    fs.unlinkSync(filename);
  } catch (error) { errors.push(error); }
  cleanupFailure(primary, errors);
  return result;
}

/** Publish the body before its metadata, with peak temporary bytes included in admission. */
export function writeBudgetedSourceCapture(root, metadataPath, record, policy = SOURCE_CACHE_POLICY) {
  validatePolicy(policy);
  root = path.resolve(root);
  metadataPath = path.resolve(metadataPath);
  if (record.bytes.length > policy.maxBodyBytes) throw Error('source_cache_body_budget_exceeded');
  if (path.dirname(metadataPath) !== root || !/^[a-f0-9]{16}\.meta$/u.test(path.basename(metadataPath))
      || !/^[a-f0-9]{64}$/u.test(record.sha256)) throw Error('source_cache_capture_path');
  if (createHash('sha256').update(record.bytes).digest('hex') !== record.sha256
      || /[\r\n]/u.test(record.source_url) || !/^[0-9]+$/u.test(record.fetched_at)) throw Error('source_cache_capture_identity');
  const metadata = new TextEncoder().encode('source-capture-v2\nurl=' + record.source_url
    + '\nfetched_at=' + record.fetched_at + '\nsha256=' + record.sha256 + '\n');
  if (metadata.length > policy.maxMetadataBytes) throw Error('source_cache_metadata_budget_exceeded');
  fs.mkdirSync(root, {recursive: true, mode: 0o700});
  regularDirectory(root);
  return withWriteLock(root, () => {
    const objects = path.join(root, 'objects');
    fs.mkdirSync(objects, {recursive: true, mode: 0o700});
    regularDirectory(objects);
    const bodyPath = path.join(objects, record.sha256 + '.body');
    const reuse = existingRegular(bodyPath);
    if (reuse && !readBoundedFilePath(bodyPath, policy.maxBodyBytes).equals(Buffer.from(record.bytes)))
      throw Error('source_cache_content_hash_mismatch:' + bodyPath);
    const hasMetadata = existingRegular(metadataPath);
    const usage = sourceCacheUsage(root, policy);
    if (usage.entries + (hasMetadata ? 0 : 1) > policy.maxEntries
        || usage.files + (reuse ? 1 : 2) > policy.maxFiles
        || usage.bytes + metadata.length + (reuse ? 0 : record.bytes.length) > policy.maxBytes)
      throw Error('source_cache_budget_exceeded:' + root);
    const temporary = [];
    const stage = (target, bytes) => {
      const pending = target + '.' + process.pid + '.' + randomUUID() + '.tmp';
      const descriptor = fs.openSync(pending, 'wx', 0o600);
      const owned = {pending, identity: fs.fstatSync(descriptor, {bigint: true})};
      temporary.push(owned);
      let primary;
      try { fs.writeFileSync(descriptor, bytes); fs.fsyncSync(descriptor); } catch (error) { primary = error; }
      const errors = [];
      try { owned.identity = fs.fstatSync(descriptor, {bigint: true}); } catch (error) { errors.push(error); }
      try { fs.closeSync(descriptor); } catch (error) { errors.push(error); }
      cleanupFailure(primary, errors);
      return owned;
    };
    const unchanged = owned => {
      const current = fs.lstatSync(owned.pending, {bigint: true}), identity = owned.identity;
      if (!current.isFile() || current.dev !== identity.dev || current.ino !== identity.ino
          || current.size !== identity.size || current.mtimeNs !== identity.mtimeNs || current.ctimeNs !== identity.ctimeNs)
        throw Error('source_cache_temporary_custody:' + owned.pending);
    };
    const publish = (owned, target) => { unchanged(owned); fs.renameSync(owned.pending, target); };
    let primary;
    try {
      if (!reuse) publish(stage(bodyPath, record.bytes), bodyPath);
      publish(stage(metadataPath, metadata), metadataPath);
    } catch (error) { primary = error; }
    const errors = [];
    for (const owned of temporary) {
      try { unchanged(owned); fs.unlinkSync(owned.pending); }
      catch (error) { if (error.code !== 'ENOENT') errors.push(error); }
    }
    cleanupFailure(primary, errors);
  });
}

/** Node host adapter; injected legacy IO remains the caller's storage policy. */
export function boundedSourceCacheIo(get) {
  const missing = action => {
    try { return action(); } catch (error) { if (error.code === 'ENOENT') return null; throw error; }
  };
  return {
    readText: filename => missing(() => new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(readBoundedFilePath(filename, SOURCE_CACHE_POLICY.maxMetadataBytes))),
    readBytes: filename => missing(() => new Uint8Array(readBoundedFilePath(filename, SOURCE_CACHE_POLICY.maxBodyBytes))),
    writeCapture: (root, metadataPath, record) => writeBudgetedSourceCapture(root, metadataPath, record),
    get,
  };
}
