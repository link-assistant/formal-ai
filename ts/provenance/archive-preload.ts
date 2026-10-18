// Loaded as .cjs by the explicit archived-runtime launcher, before engine modules.
const fileSystem = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const crypto = require('node:crypto');
const {pathToFileURL, fileURLToPath} = require('node:url');
const configuration = JSON.parse(fileSystem.readFileSync(process.env.FORMAL_AI_RUNTIME_CONFIG, 'utf8'));
const archive = configuration.sourceArchive;
if (!archive || archive.schema !== 'formal-ai-authenticated-runtime-archive-v1') {
  throw Error('ArchivedRuntimeConfigurationRequired');
}
const originalRead = fileSystem.readFileSync;
const builtin = new Set(Module.builtinModules.map(name => name.replace(/^node:/u, '')));
function bytesOf(entry) {
  const bytes = Buffer.from(entry.bytes, 'base64');
  if (bytes.toString('base64') !== entry.bytes || bytes.length !== entry.size
      || crypto.createHash('sha256').update(bytes).digest('hex') !== entry.sha256) {
    throw Error('ArchivedRuntimeTransportBytes');
  }
  return bytes;
}
function logicalURL(file) {
  if (typeof file === 'string') return pathToFileURL(path.resolve(file)).href;
  if (file instanceof URL && file.protocol === 'file:') return file.href;
  return null;
}
function owned(url) {
  if (url === null) return false;
  const filename = fileURLToPath(url);
  return Object.hasOwn(archive.modules,url) || filename === archive.repository || filename.startsWith(archive.repository + path.sep);
}
fileSystem.readFileSync = function(file, options) {
  const url = logicalURL(file);
  if (url === null || !owned(url)) return originalRead.apply(this, arguments);
  if (options !== undefined && options !== null && typeof options !== 'string') {
    throw Error('ArchivedRuntimeUnsupportedReadOptions');
  }
  if(typeof options==='string'&&!['utf8','utf-8'].includes(options))throw Error('ArchivedRuntimeUnsupportedReadEncoding');
  const representation = typeof options === 'string' ? 'string' : 'buffer';
  const observed = archive.reads[url + '\0repository-read-' + representation];
  if (observed) {
    const bytes = bytesOf(observed);
    return representation === 'string' ? bytes.toString('utf8') : bytes;
  }
  throw Object.assign(Error('ArchivedRuntimeUnobservedRead'),{sourceURL:url,representation});
};
const originalCompile = Module.prototype._compile;
Module.prototype._compile = function(content, filename) {
  const entry = archive.modules[pathToFileURL(filename).href];
  if (entry) {
    if (entry.format !== 'commonjs') throw Error('ArchivedRuntimeModuleFormat');
    return originalCompile.call(this, bytesOf(entry).toString('utf8'), filename);
  }
  if (owned(pathToFileURL(filename).href)) throw Error('ArchivedRuntimeUnobservedCompile');
  return originalCompile.apply(this, arguments);
};
const originalResolve = Module._resolveFilename;
Module._resolveFilename = function(request, parent) {
  if (builtin.has(request.replace(/^node:/u, ''))) return originalResolve.apply(this, arguments);
  const resolved = archive.commonjs[JSON.stringify([parent?.filename ?? null, request])];
  if (typeof resolved === 'string') return resolved;
  if (typeof parent?.filename==='string' && owned(pathToFileURL(parent.filename).href)) throw Error('ArchivedRuntimeUnobservedResolution');
  return originalResolve.apply(this, arguments);
};
for (const extension of ['.js', '.cjs', '.json']) {
  const originalExtension = Module._extensions[extension] ?? (extension === '.cjs' ? Module._extensions['.js'] : undefined);
  Module._extensions[extension] = function(module, filename) {
    const entry = archive.modules[pathToFileURL(filename).href];
    if (entry) {
      const content = bytesOf(entry).toString('utf8');
      if (extension === '.json') {
        if (entry.format !== 'json') throw Error('ArchivedRuntimeModuleFormat');
        if(process.env.FORMAL_AI_RUNTIME_BOOTSTRAP){
          require(process.env.FORMAL_AI_RUNTIME_BOOTSTRAP).source(pathToFileURL(filename).href,'json',content,'actual-authenticated-json-parse-input');
        }
        module.exports = JSON.parse(content.replace(/^\uFEFF/u, ''));
        return;
      }
      if (entry.format !== 'commonjs') throw Error('ArchivedRuntimeModuleFormat');
      module._compile(content, filename);
      return;
    }
    if (owned(pathToFileURL(filename).href)) throw Error('ArchivedRuntimeUnobservedCompile');
    if (typeof originalExtension !== 'function') throw Error('ArchivedRuntimeExtensionUnavailable');
    return originalExtension(module, filename);
  };
}
Module.syncBuiltinESMExports();
module.exports = {archive, bytesOf};
