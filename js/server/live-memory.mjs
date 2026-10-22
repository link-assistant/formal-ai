// Live memory startup policy; canonical native twin: memory_sync/live_path.rs.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { inspectMemoryUpgrade, sharedMemoryPath, TARGET_SCHEMA, exportLinksNotation } from './memory-store.mjs';
import { migrateMemory } from './memory-upgrade.mjs';
const MAXIMUM_BYTES = 64 * 1024 * 1024;
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function observed(file) {
  const initial = fs.lstatSync(file);
  if (!initial.isFile() || initial.isSymbolicLink()) throw Error('memory_live_regular_file_required');
  const descriptor = fs.openSync(file, fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW ?? 0) | (fs.constants.O_NONBLOCK ?? 0));
  try {
    const metadata = fs.fstatSync(descriptor);
    if (!metadata.isFile() || metadata.dev !== initial.dev || metadata.ino !== initial.ino || metadata.size > MAXIMUM_BYTES) throw Error('memory_live_identity_or_size');
    const bytes = Buffer.alloc(metadata.size + 1);
    let offset = 0;
    for (let attempt = 0; attempt <= bytes.length; attempt++) {
      const count = fs.readSync(descriptor, bytes, offset, Math.min(65536, bytes.length - offset), offset);
      if (count === 0) break;
      offset += count;
      if (offset === bytes.length) throw Error('memory_live_size_changed');
    }
    const current = fs.lstatSync(file),
      final = fs.fstatSync(descriptor);
    if (current.isSymbolicLink() || current.dev !== metadata.dev || current.ino !== metadata.ino || final.size !== metadata.size || final.mtimeMs !== metadata.mtimeMs || final.ctimeMs !== metadata.ctimeMs || offset !== metadata.size) throw Error('memory_live_changed');
    return {
      bytes: bytes.subarray(0, offset),
      metadata
    };
  } finally {
    fs.closeSync(descriptor);
  }
}
function privateCurrent(file) {
  const result = observed(file),
    status = inspectMemoryUpgrade(result.bytes, true);
  if (!status.compatible || status.detected_schema_version !== TARGET_SCHEMA) throw Error('memory_live_sibling_incompatible');
  if (process.platform !== 'win32' && ((result.metadata.mode & 0o077) !== 0 || result.metadata.uid !== process.getuid())) throw Error('memory_live_sibling_not_private');
  return result;
}
/** Preserve unknown bytes; validated legacy documents use the existing backup transaction. */
export async function resolveLiveMemory(env = process.env, diagnostics = null) {
  const original = sharedMemoryPath(env);
  let first;
  try {
    first = observed(original);
  } catch (error) {
    if (error.code === 'ENOENT') return {
      path: original,
      originalPath: original,
      mode: 'new'
    };
    throw error;
  }
  const status = inspectMemoryUpgrade(first.bytes, true),
    sourceSha256 = digest(first.bytes);
  if (status.compatible) {
    if (status.migration_required) {
      const receipt = await migrateMemory(original, null, null, () => {
        if (digest(observed(original).bytes) !== sourceSha256) throw Error('memory_live_preimage_changed');
      });
      diagnostics?.log('info', 'memory.live_migration', {
        component: 'memory',
        path: original,
        source_sha256: sourceSha256,
        changed: receipt.changed
      });
    }
    return {
      path: original,
      originalPath: original,
      mode: status.migration_required ? 'migrated' : 'current'
    };
  }
  const selected = original + '.schema-' + TARGET_SCHEMA + '.' + sourceSha256 + '.lino';
  let created = false;
  try {
    const descriptor = fs.openSync(selected, 'wx', 0o600);
    try {
      const bytes = Buffer.from(exportLinksNotation([], TARGET_SCHEMA));
      let offset = 0;
      for (let attempt = 0; attempt < bytes.length && offset < bytes.length; attempt++) {
        const count = fs.writeSync(descriptor, bytes, offset, bytes.length - offset);
        if (!Number.isInteger(count) || count <= 0) throw Error('memory_live_write_no_progress');
        offset += count;
      }
      if (offset !== bytes.length) throw Error('memory_live_write_incomplete');
      fs.fsyncSync(descriptor);
      created = true;
    } finally {
      fs.closeSync(descriptor);
    }
  } catch (error) {
    if (error.code !== 'EEXIST') throw error;
  }
  privateCurrent(selected);
  if (digest(observed(original).bytes) !== sourceSha256) throw Error('memory_live_preimage_changed');
  diagnostics?.log('warn', 'memory.live_replacement', {
    component: 'memory',
    original_path: original,
    path: selected,
    source_sha256: sourceSha256,
    reason: status.refusal_code,
    created,
    original_preserved: true
  });
  return {
    path: selected,
    originalPath: original,
    sourceSha256,
    mode: 'preserved-incompatible',
    reason: status.refusal_code
  };
}
