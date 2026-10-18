// Runtime derivations are separate from the caller workspace and bundled seeds.
import fs from 'node:fs';
import path from 'node:path';
import { createHash, randomBytes } from 'node:crypto';
import { toLino, fromLino } from '../agentic/crate/derivation.mjs';

export function runtimeCachePaths({ env = process.env, platform = process.platform, cwd = process.cwd(), readOnlySeedRoots = [] } = {}) {
  const paths = platform === 'win32' ? path.win32 : path.posix;
  let runtimeRoot;
  if (env.FORMAL_AI_CACHE_DIR !== undefined) {
    if (typeof env.FORMAL_AI_CACHE_DIR !== 'string' || !env.FORMAL_AI_CACHE_DIR.trim()) throw Error('runtime_cache_root_invalid');
    runtimeRoot = paths.resolve(cwd, String(env.FORMAL_AI_CACHE_DIR));
  } else if (env.XDG_CACHE_HOME && paths.isAbsolute(env.XDG_CACHE_HOME)) {
    runtimeRoot = paths.join(env.XDG_CACHE_HOME, 'formal-ai');
  } else if (platform === 'win32' && env.LOCALAPPDATA && paths.isAbsolute(env.LOCALAPPDATA)) {
    runtimeRoot = paths.join(env.LOCALAPPDATA, 'formal-ai', 'Cache');
  } else {
    const home = platform === 'win32' ? env.USERPROFILE || env.HOME : env.HOME;
    if (!home || !paths.isAbsolute(home)) throw Error('runtime_cache_home_unavailable');
    runtimeRoot = paths.join(home, ...(platform === 'darwin' ? ['Library', 'Caches', 'formal-ai'] : platform === 'win32' ? ['AppData', 'Local', 'formal-ai', 'Cache'] : ['.cache', 'formal-ai']));
  }
  return { runtimeRoot, derivationDirectory: paths.join(runtimeRoot, 'derivations'), readOnlySeedRoots: [...readOnlySeedRoots] };
}

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function locatorParts(locator) {
  const match = /^([A-Za-z0-9_-]+)(?:@([a-f0-9]{64}))?$/u.exec(locator);
  if (!match) throw Error('derivation_answer_id_unusable');
  return { answerId: match[1], recordDigest: match[2] ?? null };
}

export function persistRuntimeDerivation(record, options = {}) {
  const { answerId, recordDigest: selector } = locatorParts(record.answer_id);
  if (selector !== null) throw Error('derivation_answer_id_unusable');
  const { derivationDirectory } = runtimeCachePaths(options);
  const directory = path.join(derivationDirectory, answerId);
  const pending = path.join(derivationDirectory, '.pending');
  for (const folder of [directory, pending]) {
    fs.mkdirSync(folder, { recursive: true, mode: 0o700 });
    const status = fs.lstatSync(folder);
    if (!status.isDirectory() || status.isSymbolicLink()) throw Error('derivation_directory_unusable');
  }
  const bytes = Buffer.from(toLino(record));
  const recordDigest = digest(bytes);
  const file = path.join(directory, recordDigest + '.lino');
  const temporary = path.join(pending, answerId + '.' + process.pid + '.' + randomBytes(16).toString('hex') + '.tmp');
  let descriptor, identity;
  try {
    descriptor = fs.openSync(temporary, 'wx', 0o600);
    identity = fs.fstatSync(descriptor, { bigint: true });
    fs.writeFileSync(descriptor, bytes);
    fs.fsyncSync(descriptor);
    fs.closeSync(descriptor);
    descriptor = undefined;
    if (!fs.readFileSync(temporary).equals(bytes)) throw Error('derivation_record_custody_failed');
    // Hard-link publication is atomic and never replaces an existing object.
    // The fully written staging inode is on the same volume as its final link.
    try { fs.linkSync(temporary, file); }
    catch (error) { if (error.code !== 'EEXIST') throw error; }
    const status = fs.lstatSync(file);
    if (!status.isFile() || status.isSymbolicLink() || !fs.readFileSync(file).equals(bytes)) throw Error('derivation_record_custody_failed');
    return { path: file, locator: answerId + '@' + recordDigest, recordDigest };
  } finally {
    if (descriptor !== undefined) fs.closeSync(descriptor);
    if (identity !== undefined) {
      let current;
      try { current = fs.lstatSync(temporary, { bigint: true }); }
      catch (error) { if (error.code !== 'ENOENT') throw error; }
      if (current) {
        if (!current.isFile() || current.isSymbolicLink() || current.dev !== identity.dev || current.ino !== identity.ino) throw Error('derivation_temporary_custody_failed');
        fs.unlinkSync(temporary);
      }
    }
  }
}

export function loadRuntimeDerivation(locator, options = {}) {
  const { answerId, recordDigest } = locatorParts(locator);
  const { derivationDirectory } = runtimeCachePaths(options);
  const directory = path.join(derivationDirectory, answerId);
  let names;
  try {
    const status = fs.lstatSync(directory);
    if (!status.isDirectory() || status.isSymbolicLink()) throw Error('derivation_directory_unusable');
    names = fs.readdirSync(directory);
  } catch (error) {
    if (error.code === 'ENOENT') return null;
    throw error;
  }
  if (names.some(name => !/^[a-f0-9]{64}\.lino$/u.test(name))) throw Error('derivation_record_custody_failed');
  if (recordDigest === null && names.length > 1) throw Error('derivation_record_ambiguous: use ' + answerId + '@<record-digest>');
  const name = recordDigest === null ? names[0] : recordDigest + '.lino';
  if (!name || !names.includes(name)) return null;
  const file = path.join(directory, name), status = fs.lstatSync(file);
  if (!status.isFile() || status.isSymbolicLink()) throw Error('derivation_record_custody_failed');
  const bytes = fs.readFileSync(file);
  if (digest(bytes) !== name.slice(0, -5)) throw Error('derivation_record_custody_failed');
  const text = bytes.toString('utf8');
  if (!Buffer.from(text).equals(bytes)) throw Error('derivation_record_custody_failed');
  const record = fromLino(text);
  if (!record || record.answer_id !== answerId || toLino(record) !== bytes.toString('utf8')) throw Error('derivation_record_custody_failed');
  return record;
}
