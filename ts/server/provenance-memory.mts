// The actual shared memory store, with explicit single-session byte custody.
// The lock coordinates participating owners; it is not a hostile-process sandbox.
import fileSystem from 'node:fs';
import path from 'node:path';
import { SyncStore, memoryEvent, sharedMemoryPath, writeAtomic } from './memory-store.mjs';
import { encodeTyped, decodeTyped } from '../logging/typed-codec.mjs';
import { rawBytes } from '../provenance/session.mjs';
import { blake3Hex } from '../crypto/blake3.mjs';
import { resolveEvidenceDirectory } from '../provenance/drive-capture.mjs';
import { readBoundedRuntimeFile } from '../provenance/fresh-runtime.mjs';
const MAXIMUM_MEMORY_BYTES = 64 * 1024 * 1024;
function privateRegular(filename) {
  const state = fileSystem.lstatSync(filename, {
    bigint: true
  });
  if (!state.isFile() || state.isSymbolicLink() || state.nlink !== 1n || state.size > BigInt(MAXIMUM_MEMORY_BYTES) || typeof process.getuid === 'function' && state.uid !== BigInt(process.getuid()) || process.platform !== 'win32' && (state.mode & 0o077n) !== 0n) throw Error('MemoryCustodyFileNotPrivate');
  return state;
}
export function openMemoryCustody({
  environment = process.env,
  file = sharedMemoryPath(environment),
  workspace,
  sessionId: sessionIdentifier,
  taskURL
} = {}) {
  if (typeof file !== 'string' || typeof workspace !== 'string' || typeof sessionIdentifier !== 'string' || !sessionIdentifier || typeof taskURL !== 'string' || taskURL.length > 4096) throw Error('MemoryCustodyConfiguration');
  let address;
  try {
    address = new URL(taskURL);
  } catch {
    throw Error('MemoryCustodyTaskURL');
  }
  if (address.protocol !== 'https:') throw Error('MemoryCustodyTaskURL');
  const parent = resolveEvidenceDirectory(path.dirname(path.resolve(file)), workspace, 'MemoryInsideWorkspace');
  fileSystem.mkdirSync(parent, {
    recursive: true,
    mode: 0o700
  });
  const parentState = fileSystem.statSync(parent);
  if (!parentState.isDirectory() || process.platform !== 'win32' && (parentState.mode & 0o022) !== 0) throw Error('MemoryCustodyDirectory');
  const filename = path.join(parent, path.basename(file)),
    ownerFile = filename + '.provenance-owner.lino';
  const revisionFile = filename + '.provenance-revision.lino';
  const ownerBytes = Buffer.from(encodeTyped({
    schema: 'formal-ai-memory-owner-v1',
    sessionId: sessionIdentifier,
    taskURL,
    pid: process.pid
  }) + '\n');
  const ownerDescriptor = fileSystem.openSync(ownerFile, 'wx', 0o600);
  fileSystem.writeFileSync(ownerDescriptor, ownerBytes);
  fileSystem.closeSync(ownerDescriptor);
  let closed = false,
    store,
    expected,
    revision = 0,
    revisionBytes = null;
  function readCurrent() {
    const state = privateRegular(filename),
      bytes = readBoundedRuntimeFile(parent, path.basename(filename), MAXIMUM_MEMORY_BYTES);
    const after = privateRegular(filename);
    if (state.dev !== after.dev || state.ino !== after.ino || state.size !== after.size || state.mtimeNs !== after.mtimeNs || state.ctimeNs !== after.ctimeNs) throw Error('MemoryChangedDuringRead');
    return {
      state: after,
      bytes,
      hash: blake3Hex(bytes)
    };
  }
  function validateOwner() {
    const bytes = readBoundedRuntimeFile(parent, path.basename(ownerFile), 65536);
    if (!bytes.equals(ownerBytes)) throw Error('MemoryOwnerChanged');
  }
  function releaseOwner() {
    validateOwner();
    fileSystem.unlinkSync(ownerFile);
    closed = true;
  }
  try {
    try {
      privateRegular(filename);
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
      fileSystem.closeSync(fileSystem.openSync(filename, 'wx', 0o600));
    }
    expected = readCurrent();
    store = SyncStore.open(environment, filename, expected.bytes);
    if (!store.compatible) throw Error('MemorySchemaUnavailable');
    if (expected.bytes.length === 0) {
      store.persist({
        privateMode: true
      });
      expected = readCurrent();
    }
    try {
      privateRegular(revisionFile);
      const bytes = readBoundedRuntimeFile(parent, path.basename(revisionFile), 65536);
      const text = bytes.toString('utf8');
      if (!text.endsWith('\n')) throw Error('MemoryRevisionIncomplete');
      const record = decodeTyped(text.slice(0, -1));
      if (record.schema !== 'formal-ai-memory-revision-v1' || !Number.isSafeInteger(record.revision) || record.revision < 0 || record.memoryHash !== expected.hash) throw Error('MemoryRevisionMismatch');
      revision = record.revision;
      revisionBytes = Buffer.from(bytes);
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
    }
  } catch (error) {
    releaseOwner();
    throw error;
  }
  function assertCurrent() {
    if (closed) throw Error('MemoryCustodyClosed');
    validateOwner();
    if (revisionBytes !== null) {
      privateRegular(revisionFile);
      const currentRevision = readBoundedRuntimeFile(parent, path.basename(revisionFile), 65536);
      if (!currentRevision.equals(revisionBytes)) throw Error('MemoryRevisionOutOfBand');
    } else if (fileSystem.existsSync(revisionFile)) throw Error('MemoryRevisionOutOfBand');
    const current = readCurrent();
    if (current.hash !== expected.hash || current.state.dev !== expected.state.dev || current.state.ino !== expected.state.ino || current.state.mtimeNs !== expected.state.mtimeNs || current.state.ctimeNs !== expected.state.ctimeNs) throw Error('MemoryOutOfBandEdit');
    return current;
  }
  function snapshot() {
    const current = assertCurrent();
    return {
      schema: 'formal-ai-logical-memory-v1',
      path: filename,
      taskURL,
      revision,
      memoryHash: current.hash,
      bytes: Buffer.from(current.bytes),
      revisionBytes:revisionBytes===null?null:Buffer.from(revisionBytes)
    };
  }
  function advance(kind, messageBytes) {
    if (!['message_in', 'message_out'].includes(kind) || !(messageBytes instanceof Uint8Array)) throw Error('MemoryMessageShape');
    const before = assertCurrent();
    if (revision === Number.MAX_SAFE_INTEGER) throw Error('MemoryRevisionExhausted');
    const next = revision + 1,
      messageHash = blake3Hex(messageBytes);
    const identity = blake3Hex(Buffer.from(encodeTyped({
      sessionId: sessionIdentifier,
      revision: next,
      kind,
      messageHash
    })));
    store.events.push(memoryEvent({
      id: 'provenance_message_' + identity,
      kind: 'provenance_message',
      role: kind === 'message_in' ? 'user' : 'assistant',
      inputs: kind,
      content: encodeTyped(rawBytes(messageBytes)),
      conversation_id: sessionIdentifier,
      write_count: 1
    }));
    const intended = Buffer.from(store.toLinksNotation(), 'utf8');
    if (intended.length > MAXIMUM_MEMORY_BYTES) throw Error('MemoryCapacityExceeded');
    store.persist({
      privateMode: true
    });
    const after = readCurrent();
    if (!after.bytes.equals(intended)) throw Error('MemoryPersistMismatch');
    revision = next;
    expected = after;
    const nextRevision = Buffer.from(encodeTyped({
      schema: 'formal-ai-memory-revision-v1',
      revision,
      memoryHash: after.hash,
      sessionId: sessionIdentifier
    }) + '\n');
    writeAtomic(revisionFile, nextRevision, {
      privateMode: true
    });
    privateRegular(revisionFile);
    revisionBytes = readBoundedRuntimeFile(parent, path.basename(revisionFile), 65536);
    if (!revisionBytes.equals(nextRevision)) throw Error('MemoryRevisionPersistMismatch');
    return {
      schema: 'formal-ai-logical-memory-transition-v1',
      kind,
      revision,
      messageHash,
      previousMemoryHash: before.hash,
      memoryHash: after.hash,
      bytes: Buffer.from(after.bytes)
    };
  }
  function close() {
    if (closed) return;
    try {
      assertCurrent();
    } finally {
      releaseOwner();
    }
  }
  return {
    snapshot,
    advance,
    close
  };
}
