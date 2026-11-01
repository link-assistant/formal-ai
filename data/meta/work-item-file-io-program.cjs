'use strict';

// Registered source, embedded byte-for-byte by JS readText / Rust include_str.
// Source file bytes are data. No eval, subprocesses, downloaded code or network.
const fs = require('node:fs'),
  path = require('node:path'),
  crypto = require('node:crypto');
const LIMIT = 131072,
  FRAME = 'FORMAL_AI_FILE_IO_V1 ';
const digest = value => crypto.createHash('sha256').update(value).digest('hex');
const canonical = value => JSON.stringify(Object.fromEntries(Object.keys(value).sort().map(key => [key, value[key]])));
const decode = text => {
  if (typeof text !== 'string' || !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(text)) throw Error('InvalidBase64');
  const bytes = Buffer.from(text, 'base64');
  if (bytes.toString('base64') !== text) throw Error('InvalidBase64');
  return bytes;
};
// Text matching selects candidates only; complete CST validation follows a fresh Read.
function searchRequest(value) {
  return value.schema === 'formal-ai-owned-search-request/v1' && value.operation === 'search' && Object.keys(value).sort().join('|') === 'directory|identifiers|operation|root|schema' &&
         typeof value.directory === 'string' && value.directory.length <= 4096 && !value.directory.includes('\\') &&
         !value.directory.includes('\0') && value.directory.split('/').every(part => part && part !== '.' && part !== '..' &&
         !part.startsWith('.')) && Array.isArray(value.identifiers) && value.identifiers.length > 0 && value.identifiers.length <= 8 &&
         value.identifiers.every(identifier => typeof identifier === 'string' && /^[A-Za-z_][A-Za-z0-9_]{0,127}$/.test(identifier)) &&
         new Set(value.identifiers).size === value.identifiers.length;
}
function searchSources(value, cwd) {
  if (typeof value.root !== 'string' || value.root !== cwd || fs.realpathSync(value.root) !== value.root) throw Error('RootContextMismatch');
  const files = [],
    gaps = [],
    parents = [];
  let visitedFiles = 0,
    visitedDirectories = 0,
    bytesRead = 0,
    stopped = false;
  const gap = (kind, owner) => {
    if (gaps.length < 32) gaps.push({
      kind,
      path: owner
    });
  };
  const remember = owner => {
    const metadata = fs.lstatSync(owner);
    if (!metadata.isDirectory() || metadata.isSymbolicLink()) throw Error('SearchDirectoryRefused');
    parents.push({
      path: owner,
      dev: metadata.dev,
      ino: metadata.ino,
      mtimeMs: metadata.mtimeMs,
      ctimeMs: metadata.ctimeMs
    });
    return metadata;
  };
  const check = () => {
    if (fs.realpathSync(process.cwd()) !== value.root || fs.realpathSync(value.root) !== value.root) throw Error('RootContextChanged');
    for (const saved of parents) {
      const current = fs.lstatSync(saved.path);
      if (!current.isDirectory() || current.isSymbolicLink() || current.dev !== saved.dev || current.ino !== saved.ino || current.mtimeMs !== saved.mtimeMs || current.ctimeMs !== saved.ctimeMs) throw Error('SearchDirectoryChanged');
    }
  };
  let owner = value.root;
  remember(owner);
  for (const part of value.directory.split('/')) {
    owner = path.join(owner, part);
    remember(owner);
  }
  const walk = (directory, depth) => {
    if (stopped) return;
    if (depth > 12 || ++visitedDirectories > 128) {
      gap('SearchDirectoryBudget', directory);
      stopped = true;
      return;
    }
    check();
    const entries = [],
      handle = fs.opendirSync(path.join(value.root, directory));
    try {
      let entry;
      while (entries.length <= 512 && (entry = handle.readSync())) entries.push(entry.name);
    } finally {
      handle.closeSync();
    }
    if (entries.length > 512) {
      gap('SearchEntryBudget', directory);
      stopped = true;
      return;
    }
    entries.sort();
    for (const name of entries) {
      if (stopped) break;
      if (name.startsWith('.')) continue;
      const relative = directory + '/' + name,
        absolute = path.join(value.root, relative);
      try {
        const metadata = fs.lstatSync(absolute);
        if (metadata.isSymbolicLink()) {
          gap('SearchSymlinkRefused', relative);
          continue;
        }
        if (metadata.isDirectory()) {
          remember(absolute);
          walk(relative, depth + 1);
          continue;
        }
        if (!name.endsWith('.rs')) continue;
        if (++visitedFiles > 1024) {
          gap('SearchFileBudget', relative);
          stopped = true;
          break;
        }
        if (!metadata.isFile() || metadata.nlink !== 1 || metadata.size > LIMIT) {
          gap('SearchFileRefused', relative);
          continue;
        }
        if (bytesRead + metadata.size > 16777216) {
          gap('SearchByteBudget', relative);
          stopped = true;
          break;
        }
        check();
        const descriptor = fs.openSync(absolute, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
        let bytes;
        try {
          const opened = fs.fstatSync(descriptor);
          if (!opened.isFile() || opened.nlink !== 1 || opened.dev !== metadata.dev || opened.ino !== metadata.ino || opened.size !== metadata.size) throw Error('SearchFileChanged');
          // Read at most one byte beyond the allowed snapshot, never unbounded readFile.
          const buffer = Buffer.alloc(opened.size + 1);
          let length = 0,
            count;
          while (length < buffer.length && (count = fs.readSync(descriptor, buffer, length, buffer.length - length, null)) > 0) length += count;
          bytes = buffer.subarray(0, length);
          const after = fs.fstatSync(descriptor),
            current = fs.lstatSync(absolute);
          if (length !== opened.size || after.size !== opened.size || after.mtimeMs !== opened.mtimeMs || after.ctimeMs !== opened.ctimeMs || current.dev !== opened.dev || current.ino !== opened.ino || !current.isFile() || current.nlink !== 1) throw Error('SearchFileChanged');
          check();
        } finally {
          fs.closeSync(descriptor);
        }
        bytesRead += bytes.length;
        const source = bytes.toString('utf8');
        if (!Buffer.from(source, 'utf8').equals(bytes)) {
          gap('SearchUtf8Required', relative);
          continue;
        }
        const identifiers = value.identifiers.filter(identifier => source.includes(identifier));
        if (identifiers.length) {
          if (files.length >= 8) {
            gap('SearchMatchBudget', relative);
            stopped = true;
            break;
          }
          files.push({
            byteLength: bytes.length,
            identifiers,
            path: relative,
            sha256: digest(bytes)
          });
        }
      } catch (error) {
        gap(String(error.code ?? error.message), relative);
      }
    }
    check();
  };
  try {
    walk(value.directory, 0);
  } catch (error) {
    gap(String(error.code ?? error.message), value.directory);
  }
  return {
    schema: 'formal-ai-owned-search-receipt/v1',
    requestIdentity,
    operation: 'search',
    root: value.root,
    directory: value.directory,
    identifiers: value.identifiers,
    state: 'observed-search',
    complete: gaps.length === 0 && !stopped,
    files,
    gaps,
    visitedFiles,
    visitedDirectories,
    bytesRead,
    scope: 'bounded-nonhidden-rust-source-candidates; text matches are not bindings; fresh Read and CST validation required'
  };
}
let request = null,
  requestIdentity = null,
  staged = null;
function emit(receipt, status) {
  process.stdout.write(FRAME + Buffer.from(canonical(receipt)).toString('base64') + '\n');
  process.exitCode = status;
}
try {
  const input = decode(process.argv[1]);
  if (input.length > 196608) throw Error('RequestLimit');
  request = JSON.parse(input.toString('utf8'));
  const keys = ['afterBase64', 'beforeSha256', 'operation', 'path', 'root', 'schema'];
  const search = searchRequest(request);
  const file = Object.keys(request).sort().join('|') === keys.join('|') && request.schema === 'formal-ai-owned-file-request/v1' && ['root', 'read', 'create', 'replace'].includes(request.operation);
  if (!search && !file || canonical(request) !== input.toString('utf8')) throw Error('RequestContract');
  requestIdentity = digest(input);
  const cwd = fs.realpathSync(process.cwd());
  if (search) {
    emit(searchSources(request, cwd), 0);
  } else if (request.operation === 'root') {
    if (request.root !== null || request.path !== null || request.beforeSha256 !== null || request.afterBase64 !== null) throw Error('RootRequestContract');
    emit({
      schema: 'formal-ai-owned-file-receipt/v1',
      requestIdentity,
      operation: 'root',
      root: cwd,
      path: null,
      state: 'observed-root'
    }, 0);
  } else {
    if (typeof request.root !== 'string' || !path.isAbsolute(request.root) || fs.realpathSync(request.root) !== request.root || request.root !== cwd) throw Error('RootContextMismatch');
    if (typeof request.path !== 'string' || request.path.length > 4096 || request.path.startsWith('/') || request.path.includes('\\') || request.path.includes('\0') || request.path.split('/').some(part => !part || part === '.' || part === '..')) throw Error('RelativeOwnerRequired');
    const parts = request.path.split('/');
    if (parts.length > 128) throw Error('PathDepthLimit');
    let parent = request.root,
      missing = false;
    const parents = [];
    for (const part of parts.slice(0, -1)) {
      parent = path.join(parent, part);
      try {
        const metadata = fs.lstatSync(parent);
        if (!metadata.isDirectory() || metadata.isSymbolicLink()) throw Error('ParentOwnerRefused');
        parents.push({
          path: parent,
          dev: metadata.dev,
          ino: metadata.ino
        });
      } catch (error) {
        if (error.code !== 'ENOENT') throw error;
        if (request.operation === 'create') {
          fs.mkdirSync(parent, {
            mode: 0o755
          });
          const metadata = fs.lstatSync(parent);
          parents.push({
            path: parent,
            dev: metadata.dev,
            ino: metadata.ino
          });
        } else missing = true;
      }
    }
    const checkParents = () => {
      if (fs.realpathSync(process.cwd()) !== request.root || fs.realpathSync(request.root) !== request.root) throw Error('RootContextChanged');
      for (const observed of parents) {
        const current = fs.lstatSync(observed.path);
        if (!current.isDirectory() || current.isSymbolicLink() || current.dev !== observed.dev || current.ino !== observed.ino) throw Error('ParentOwnerChanged');
      }
    };
    checkParents();
    const owner = path.join(request.root, request.path);
    let metadata = null;
    if (!missing) {
      try {
        metadata = fs.lstatSync(owner);
      } catch (error) {
        if (error.code !== 'ENOENT') throw error;
      }
    }
    if (metadata && (!metadata.isFile() || metadata.isSymbolicLink() || metadata.nlink !== 1 || metadata.size > LIMIT)) throw Error('FileOwnerRefused');
    if (request.operation === 'read') {
      if (request.beforeSha256 !== null || request.afterBase64 !== null) throw Error('ReadRequestContract');
      if (!metadata) emit({
        schema: 'formal-ai-owned-file-receipt/v1',
        requestIdentity,
        operation: 'read',
        root: request.root,
        path: request.path,
        state: 'absent',
        contentBase64: null,
        sha256: null,
        byteLength: 0
      }, 0);else {
        const fd = fs.openSync(owner, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
        let bytes;
        try {
          const opened = fs.fstatSync(fd);
          if (opened.dev !== metadata.dev || opened.ino !== metadata.ino || opened.size > LIMIT) throw Error('ReadOwnerChanged');
          bytes = fs.readFileSync(fd);
          checkParents();
          const current = fs.lstatSync(owner);
          if (current.dev !== opened.dev || current.ino !== opened.ino || !current.isFile() || current.nlink !== 1) throw Error('ReadOwnerChanged');
        } finally {
          fs.closeSync(fd);
        }
        if (bytes.length > LIMIT || !Buffer.from(bytes.toString('utf8'), 'utf8').equals(bytes)) throw Error('SourceEncodingOrSize');
        emit({
          schema: 'formal-ai-owned-file-receipt/v1',
          requestIdentity,
          operation: 'read',
          root: request.root,
          path: request.path,
          state: 'observed-source',
          contentBase64: bytes.toString('base64'),
          sha256: digest(bytes),
          byteLength: bytes.length
        }, 0);
      }
    } else {
      const after = decode(request.afterBase64);
      if (after.length > LIMIT || !Buffer.from(after.toString('utf8'), 'utf8').equals(after)) throw Error('CandidateEncodingOrSize');
      if (request.operation === 'create' && (request.beforeSha256 !== null || metadata)) throw Error('CreatePreconditionChanged');
      if (request.operation === 'replace' && (!/^[a-f0-9]{64}$/.test(request.beforeSha256 ?? '') || !metadata)) throw Error('ReplacePreconditionRequired');
      let opened = null,
        fd = null;
      const unchanged = () => {
        checkParents();
        if (request.operation === 'create') {
          try {
            fs.lstatSync(owner);
            throw Error('CreatePreconditionChanged');
          } catch (error) {
            if (error.code !== 'ENOENT') throw error;
          }
          return;
        }
        const current = fs.lstatSync(owner);
        if (!current.isFile() || current.nlink !== 1 || current.dev !== opened.dev || current.ino !== opened.ino || digest(fs.readFileSync(owner)) !== request.beforeSha256) throw Error('ReplacePreconditionChanged');
      };
      try {
        if (request.operation === 'replace') {
          fd = fs.openSync(owner, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
          opened = fs.fstatSync(fd);
          if (opened.dev !== metadata.dev || opened.ino !== metadata.ino || digest(fs.readFileSync(fd)) !== request.beforeSha256) throw Error('ReplacePreconditionChanged');
        }
        unchanged();
        staged = path.join(parent, '.formal-ai-file-io-' + crypto.randomBytes(12).toString('hex'));
        const target = fs.openSync(staged, fs.constants.O_WRONLY | fs.constants.O_CREAT | fs.constants.O_EXCL | fs.constants.O_NOFOLLOW, metadata ? metadata.mode & 0o777 : 0o644);
        try {
          let offset = 0;
          while (offset < after.length) {
            const written = fs.writeSync(target, after, offset, after.length - offset, offset);
            if (!Number.isSafeInteger(written) || written <= 0) throw Error('WriteProgressInvalid');
            offset += written;
          }
          fs.fsyncSync(target);
        } finally {
          fs.closeSync(target);
        }
        if (!fs.readFileSync(staged).equals(after)) throw Error('StagedWriteMismatch');
        unchanged();
        if (request.operation === 'create') {
          fs.linkSync(staged, owner);
          fs.unlinkSync(staged);
        } else fs.renameSync(staged, owner);
        staged = null;
        const physical = fs.readFileSync(owner);
        if (!physical.equals(after)) throw Error('ImmediatePhysicalWriteMismatch');
        emit({
          schema: 'formal-ai-owned-file-receipt/v1',
          requestIdentity,
          operation: request.operation,
          root: request.root,
          path: request.path,
          state: 'effect-applied',
          beforeSha256: request.beforeSha256,
          afterSha256: digest(physical),
          byteLength: physical.length
        }, 0);
      } finally {
        if (fd !== null) fs.closeSync(fd);
      }
    }
  }
} catch (error) {
  if (staged) {
    try {
      fs.unlinkSync(staged);
    } catch {/* Preserve the original operation error. */}
  }
  emit({
    schema: request?.operation === 'search' ? 'formal-ai-owned-search-receipt/v1' : 'formal-ai-owned-file-receipt/v1',
    directory: request?.directory ?? null,
    identifiers: request?.identifiers ?? null,
    complete: false,
    requestIdentity,
    operation: request?.operation ?? null,
    root: request?.root ?? null,
    path: request?.path ?? null,
    state: 'failed',
    error: String(error.code ?? error.message)
  }, 1);
}
