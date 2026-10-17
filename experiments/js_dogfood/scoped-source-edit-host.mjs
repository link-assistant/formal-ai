// Filesystem effects for an independently request-owned existing-source edit.
// This adapter grants no request, source-language or transformation authority.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {types} from 'node:util';

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
function requireCondition(condition, code) {
  if (!condition) throw Object.assign(new Error(code), {code});
}
function record(value, keys) {
  requireCondition(value !== null && typeof value === 'object' && !types.isProxy(value), 'InvalidRecord');
  const prototype = Object.getPrototypeOf(value);
  requireCondition(prototype === Object.prototype || prototype === null, 'InvalidRecord');
  const descriptors = Object.getOwnPropertyDescriptors(value);
  requireCondition(Reflect.ownKeys(descriptors).every(key => typeof key === 'string'
    && keys.includes(key) && Object.hasOwn(descriptors[key], 'value')), 'UnknownOrAccessorField');
  return Object.fromEntries(Object.entries(descriptors).map(([key, descriptor]) => [key, descriptor.value]));
}
function oneAlias(value, aliases) {
  const present = aliases.filter(key => Object.hasOwn(value, key));
  requireCondition(present.length === 1 && typeof value[present[0]] === 'string', 'InvalidFieldAliases');
  return value[present[0]];
}
function sameFile(left, right) {
  return left.dev === right.dev && left.ino === right.ino;
}
function sameVersion(left, right) {
  return sameFile(left, right) && left.size === right.size
    && left.mtimeNs === right.mtimeNs && left.ctimeNs === right.ctimeNs;
}

/** Single-owner scratch custody, not a sandbox against hostile concurrent processes. */
export function createScopedSourceEditHost(options) {
  const {workspace, ownedPath, maximumBytes = 262144} = record(options,
    ['workspace', 'ownedPath', 'maximumBytes']);
  requireCondition(typeof workspace === 'string' && path.isAbsolute(workspace), 'InvalidWorkspace');
  requireCondition(Number.isSafeInteger(maximumBytes) && maximumBytes > 0
    && maximumBytes <= 1048576, 'InvalidByteBudget');
  requireCondition(typeof ownedPath === 'string' && !ownedPath.includes('\\')
    && !ownedPath.includes('\0') && !path.isAbsolute(ownedPath)
    && ownedPath.split('/').every(part => part && part !== '.' && part !== '..'), 'InvalidOwnedPath');
  const root = path.resolve(workspace), target = path.join(root, ownedPath);
  requireCondition(fs.realpathSync(root) === root, 'WorkspaceAlias');
  requireCondition(fs.lstatSync(root).isDirectory(), 'WorkspaceNotDirectory');
  let observed = null;
  let invalidated = false;
  let editCompleted = false;

  function guardPath() {
    let current = root;
    for (const [index, part] of ownedPath.split('/').entries()) {
      current = path.join(current, part);
      const status = fs.lstatSync(current, {bigint: true});
      requireCondition(!status.isSymbolicLink(), 'SymbolicSourcePath');
      if (index < ownedPath.split('/').length - 1) {
        requireCondition(status.isDirectory(), 'SourceParentNotDirectory');
      } else {
        requireCondition(status.isFile() && status.nlink === 1n, 'SourceNotPrivateRegularFile');
        requireCondition(status.size <= BigInt(maximumBytes), 'SourceByteBudget');
      }
    }
    requireCondition(fs.realpathSync(root) === root && fs.realpathSync(target) === target, 'SourcePathAlias');
    return fs.lstatSync(target, {bigint: true});
  }
  function readDescriptor(descriptor) {
    const before = fs.fstatSync(descriptor, {bigint: true});
    requireCondition(before.isFile() && before.nlink === 1n, 'SourceNotPrivateRegularFile');
    requireCondition(before.size <= BigInt(maximumBytes), 'SourceByteBudget');
    // Allocate only the bounded observed length plus one growth-detection byte.
    const buffer = Buffer.alloc(Number(before.size) + 1);
    let length = 0;
    while (length < buffer.length) {
      const count = fs.readSync(descriptor, buffer, length, buffer.length - length, length);
      if (count === 0) break;
      length += count;
    }
    const after = fs.fstatSync(descriptor, {bigint: true});
    requireCondition(sameVersion(before, after) && BigInt(length) === before.size, 'SourceChangedDuringRead');
    const bytes = buffer.subarray(0, length);
    const text = new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(bytes);
    requireCondition(Buffer.from(text).equals(bytes), 'SourceEncodingMismatch');
    return {text, bytes, sha256: sha256(bytes), status: after};
  }
  function openSource(writable) {
    const physical = guardPath();
    const descriptor = fs.openSync(target,
      (writable ? fs.constants.O_RDWR : fs.constants.O_RDONLY) | fs.constants.O_NOFOLLOW);
    try {
      requireCondition(sameFile(physical, fs.fstatSync(descriptor, {bigint: true})), 'SourceIdentityDrift');
      return descriptor;
    } catch (error) { fs.closeSync(descriptor); throw error; }
  }
  function requireCurrent(value) {
    if (observed !== null) {
      requireCondition(sameVersion(observed.status, value.status)
        && observed.sha256 === value.sha256, 'SourcePreimageDrift');
    }
  }
  function execute(call) {
    let tool = null;
    let wrote = false;
    try {
      requireCondition(!invalidated, 'SourceSessionInvalidated');
      const shape = record(call, ['tool', 'arguments']);
      tool = shape.tool;
      requireCondition(['read', 'edit'].includes(tool) && typeof shape.arguments === 'string', 'UndeclaredSourceTool');
      const args = record(JSON.parse(shape.arguments), tool === 'read'
        ? ['filePath', 'file_path', 'path']
        : ['filePath', 'file_path', 'path', 'oldString', 'old_string', 'newString', 'new_string']);
      requireCondition(oneAlias(args, ['filePath', 'file_path', 'path']) === ownedPath, 'UnownedSourcePath');
      if (tool === 'edit') requireCondition(observed !== null && !editCompleted, 'SourceReadRequired');
      const descriptor = openSource(tool === 'edit');
      try {
        const current = readDescriptor(descriptor);
        requireCurrent(current);
        if (tool === 'read') {
          observed = current;
          return {content: current.text,
            source_read: {path: ownedPath, success: true, complete: true, format: 'raw'}};
        }
        const oldString = oneAlias(args, ['oldString', 'old_string']);
        const newString = oneAlias(args, ['newString', 'new_string']);
        requireCondition(oldString.length > 0 && oldString === current.text, 'WholeObservedPreimageRequired');
        const replacement = Buffer.from(newString);
        requireCondition(replacement.length <= maximumBytes, 'ReplacementByteBudget');
        requireCondition(new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(replacement)
          === newString, 'ReplacementEncodingMismatch');
        requireCondition(!replacement.equals(current.bytes), 'SourceEditHasNoEffect');
        requireCondition(sameVersion(guardPath(), current.status), 'SourcePreimageDrift');
        let offset = 0;
        while (offset < replacement.length) {
          const count = fs.writeSync(descriptor, replacement, offset, replacement.length - offset, offset);
          requireCondition(count > 0, 'SourceWriteIncomplete');
          wrote = true;
          offset += count;
        }
        fs.ftruncateSync(descriptor, replacement.length);
        wrote = true;
        fs.fsyncSync(descriptor);
        const reread = readDescriptor(descriptor);
        requireCondition(sameFile(guardPath(), reread.status) && reread.bytes.equals(replacement), 'SourcePostimageMismatch');
        observed = reread;
        editCompleted = true;
        return {content: '', source_edit: {path: ownedPath, success: true,
          before_sha256: current.sha256, after_sha256: reread.sha256, verified: true}};
      } finally { fs.closeSync(descriptor); }
    } catch (error) {
      invalidated = true;
      const code = typeof error.code === 'string' ? error.code : 'SourceHostFailure';
      const result = {content: JSON.stringify({is_error: true, error: code}), is_error: true};
      if (tool === 'read') result.source_read = {
        path: ownedPath, success: false, complete: false, format: 'raw', error_code: code,
      };
      if (tool === 'edit') result.source_edit = {
        path: ownedPath, success: false, verified: false, may_have_written: wrote, error_code: code,
      };
      return result;
    }
  }
  return Object.freeze({execute});
}
