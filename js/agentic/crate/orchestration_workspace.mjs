// `crate::orchestration::workspace` (rust/src/orchestration/workspace.rs): the
// content-addressed snapshot of a workspace, the change set between two
// snapshots, and the guarded copy / apply steps dispatch composes with. Every
// error is the `io::Error` text Rust renders, so a controller error reads the
// same in both roots.

import { createHash } from 'node:crypto';
import { constants } from 'node:os';
import fs from 'node:fs';
import path from 'node:path';

import { agenticMessage } from '../messages.mjs';

/**
 * Mirrors `std::io::Error` in rust/src/orchestration/workspace.rs: `message` is its `Display`, `kind` the
 * `io::ErrorKind` name and `debug` its `Debug` rendering.
 */
export class IoError extends Error {
  /** @param {string} kind @param {string} message @param {string} [debug] */
  constructor(kind, message, debug = `Custom { kind: ${kind}, error: ${JSON.stringify(message)} }`) {
    super(message);
    this.kind = kind;
    this.debug = debug;
  }
}

/** The `io::ErrorKind` and the seed message id of each OS error the controller reports. */
const OS_ERRORS = {
  ENOENT: ['NotFound', 'orchestration_io_not_found'],
  EACCES: ['PermissionDenied', 'orchestration_io_permission_denied'],
  EPERM: ['PermissionDenied', 'orchestration_io_not_permitted'],
  EEXIST: ['AlreadyExists', 'orchestration_io_already_exists'],
  ENOTDIR: ['NotADirectory', 'orchestration_io_not_a_directory'],
  EISDIR: ['IsADirectory', 'orchestration_io_is_a_directory'],
  ENOTEMPTY: ['DirectoryNotEmpty', 'orchestration_io_directory_not_empty'],
};

/** Mirrors `io::Error::new(kind, message)` in rust/src/orchestration/workspace.rs. */
export const ioError = (kind, message) => new IoError(kind, message);

/** Mirrors `io::Error::other(message)` in rust/src/orchestration/workspace.rs. */
export const ioOther = (message) => new IoError('Other', String(message), `Custom { kind: Other, error: ${JSON.stringify(String(message))} }`);

/**
 * Rust built-in `std::io::Error`, which the native calls of rust/src/orchestration/workspace.rs return.
 * The `io::Error` a failed Node call corresponds to; an `IoError` passes through.
 * @param {Error & {code?: string}} error
 */
export function ioErrorFrom(error) {
  if (error instanceof IoError) return error;
  const known = OS_ERRORS[error?.code];
  if (!known) return ioOther(error?.message ?? String(error));
  const number = Math.abs(constants.errno[error.code] ?? 0);
  const text = agenticMessage(known[1]);
  return new IoError(known[0], `${text} (os error ${number})`,
    `Os { code: ${number}, kind: ${known[0]}, message: ${JSON.stringify(text)} }`);
}

const IGNORED_ROOTS = ['.git', 'target', '.formal-ai', '.formal-ai-orchestration'];

/** The components of a path (`Path::components` without the root). */
const partsOf = (value) => value.split(path.sep).filter((part) => part !== '' && part !== '.');

/** Mirrors `Path::starts_with` in rust/src/orchestration/workspace.rs. `Path::starts_with`: component-wise prefix. */
function startsWith(value, prefix) {
  const left = partsOf(value);
  const right = partsOf(prefix);
  return right.length <= left.length && right.every((part, index) => part === left[index]);
}

/** Mirrors `fn ignored` in rust/src/orchestration/workspace.rs. */
function ignored(entry, root) {
  const relative = path.relative(root, entry);
  if (relative === '' || relative.startsWith('..')) return false;
  return IGNORED_ROOTS.includes(partsOf(relative)[0]);
}

/** Mirrors `fn sha256` in rust/src/orchestration/workspace.rs. */
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');

/** Byte order of the UTF-8 text, the order a Rust `String` key sorts in. */
const byteOrder = (left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right));

/** Mirrors `fn relative_path`: the root-relative path with `/` separators. */
const relativePath = (root, entry) => path.relative(root, entry).split('\\').join('/');

/**
 * Mirrors `WalkDir::new(..).follow_links(false)` in rust/src/orchestration/workspace.rs.
 * The entries `WalkDir::new(root).follow_links(false)` yields below `root`
 * (the root itself first) after `filter_entry(keep)`: `[{path, type}]`.
 */
function walk(root, keep) {
  const out = [];
  const visit = (entry) => {
    let stat;
    try {
      stat = fs.lstatSync(entry);
    } catch (error) {
      throw ioErrorFrom(error);
    }
    const type = stat.isSymbolicLink() ? 'symlink' : stat.isDirectory() ? 'dir' : stat.isFile() ? 'file' : 'special';
    out.push({ path: entry, type });
    if (type !== 'dir') return;
    let names;
    try {
      names = fs.readdirSync(entry);
    } catch (error) {
      throw ioErrorFrom(error);
    }
    for (const name of names.sort(byteOrder)) {
      const child = path.join(entry, name);
      if (keep(child)) visit(child);
    }
  };
  visit(root);
  return out;
}

/**
 * Mirrors `fn snapshot` in rust/src/orchestration/workspace.rs: the digest and size of every file below `root`,
 * keyed by `/`-separated relative path. A symlink or special file is refused.
 * @param {string} root
 * @returns {Map<string, {sha256: string, bytes: number}>}
 */
export function snapshot(root) {
  const files = new Map();
  for (const entry of walk(root, (candidate) => !ignored(candidate, root))) {
    if (entry.type === 'symlink') throw ioError('InvalidData', `workspace_symlink:${entry.path}`);
    if (entry.type === 'dir') continue;
    if (entry.type !== 'file') throw ioError('InvalidData', `workspace_special_file:${entry.path}`);
    let bytes;
    try {
      bytes = fs.readFileSync(entry.path);
    } catch (error) {
      throw ioErrorFrom(error);
    }
    files.set(relativePath(root, entry.path), { sha256: sha256(bytes), bytes: bytes.length });
  }
  return files;
}

/**
 * Mirrors `fn changes` in rust/src/orchestration/workspace.rs: the added, modified and removed files between two
 * snapshots, in path byte order.
 * @returns {Array<{path: string, kind: string, before_sha256: string|null, after_sha256: string|null, bytes_changed: number}>}
 */
export function changes(before, after) {
  const paths = [...new Set([...before.keys(), ...after.keys()])].sort(byteOrder);
  const out = [];
  for (const entry of paths) {
    const old = before.get(entry);
    const next = after.get(entry);
    if (old?.sha256 === next?.sha256) continue;
    const kind = old === undefined ? 'added' : next === undefined ? 'removed' : 'modified';
    out.push({
      path: entry,
      kind,
      before_sha256: old ? old.sha256 : null,
      after_sha256: next ? next.sha256 : null,
      bytes_changed: (old?.bytes ?? 0) + (next?.bytes ?? 0),
    });
  }
  return out;
}

/**
 * Mirrors `fn copy_workspace` in rust/src/orchestration/workspace.rs: copy `source` into the new directory
 * `destination`, leaving out the ignored roots and `excluded`.
 */
export function copyWorkspace(source, destination, excluded) {
  if (fs.existsSync(destination)) throw ioError('AlreadyExists', `candidate_workspace_exists:${destination}`);
  try {
    fs.mkdirSync(destination, { recursive: true });
  } catch (error) {
    throw ioErrorFrom(error);
  }
  const keep = (entry) => !ignored(entry, source) && !startsWith(entry, excluded);
  for (const entry of walk(source, keep)) {
    const relative = path.relative(source, entry.path);
    if (relative === '') continue;
    const target = path.join(destination, relative);
    try {
      if (entry.type === 'dir') {
        fs.mkdirSync(target, { recursive: true });
      } else if (entry.type === 'file') {
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.copyFileSync(entry.path, target);
      } else {
        throw ioError('InvalidData', `workspace_symlink:${entry.path}`);
      }
    } catch (error) {
      throw ioErrorFrom(error);
    }
  }
}

/** Mirrors `fn safe_relative` in rust/src/orchestration/workspace.rs: a relative path that never leaves its root. */
function safeRelative(value) {
  if (path.isAbsolute(value) || value.split(path.sep).includes('..') || value.split('/').includes('..')) {
    throw ioError('InvalidInput', `workspace_escape:${value}`);
  }
  return value;
}

/** Mirrors `fn validate_change_shape` in rust/src/orchestration/workspace.rs. */
function validateChangeShape(change) {
  const before = change.before_sha256 !== null && change.before_sha256 !== undefined;
  const after = change.after_sha256 !== null && change.after_sha256 !== undefined;
  const valid = (change.kind === 'added' && !before && after)
    || (change.kind === 'modified' && before && after)
    || (change.kind === 'removed' && before && !after);
  if (!valid) throw ioError('InvalidData', `invalid_workspace_change:${change.path}`);
}

/** Mirrors `fn require_hash` in rust/src/orchestration/workspace.rs: the file at `file` must hold exactly `expected` (null: absent). */
function requireHash(file, expected, errorKind, relative) {
  let actual = null;
  let stat = null;
  try {
    stat = fs.lstatSync(file);
  } catch (error) {
    if (error.code !== 'ENOENT') throw ioErrorFrom(error);
  }
  if (stat !== null) {
    if (!stat.isFile()) throw ioError('InvalidData', `${errorKind}:${relative}`);
    try {
      actual = sha256(fs.readFileSync(file));
    } catch (error) {
      throw ioErrorFrom(error);
    }
  }
  if (actual !== (expected ?? null)) throw ioError('InvalidData', `${errorKind}:${relative}`);
}

/**
 * Mirrors `fn validate_changes` in rust/src/orchestration/workspace.rs: every selected change still describes the
 * original workspace (`workspace_drift`) and the candidate (`candidate_drift`).
 */
export function validateChanges(original, candidate, selected) {
  for (const change of selected) {
    validateChangeShape(change);
    const relative = safeRelative(change.path);
    requireHash(path.join(original, relative), change.before_sha256, 'workspace_drift', change.path);
    requireHash(path.join(candidate, relative), change.after_sha256, 'candidate_drift', change.path);
  }
}

/**
 * Mirrors `fn apply_changes` in rust/src/orchestration/workspace.rs: validate, then bring the selected candidate
 * changes into the original workspace.
 */
export function applyChanges(original, candidate, selected) {
  validateChanges(original, candidate, selected);
  for (const change of selected) {
    const relative = safeRelative(change.path);
    const target = path.join(original, relative);
    try {
      if (change.kind === 'removed') {
        if (fs.existsSync(target)) fs.rmSync(target);
      } else {
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.copyFileSync(path.join(candidate, relative), target);
      }
    } catch (error) {
      throw ioErrorFrom(error);
    }
  }
}
