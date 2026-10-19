// Runtime source projection for annotation-free maintained TypeScript twins.
// Every owned input is read from its TS file. Shared parser code is pinned separately.
const fs = require('node:fs');
const path = require('node:path');
const {fileURLToPath} = require('node:url');
const {createHash} = require('node:crypto');
const {threadId} = require('node:worker_threads');
const read = fs.readFileSync.bind(fs);
const configuration = process.env.FORMAL_AI_TYPESCRIPT_PROJECTION;
if (!configuration) throw Error('TypeScriptProjectionConfigurationRequired');
const manifest = JSON.parse(read(configuration, 'utf8'));
const prefix = manifest.root + '/js/';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');

function project(file, operation) {
  if (file instanceof URL) file = fileURLToPath(file);
  if (typeof file !== 'string') return null;
  file = path.resolve(file);
  if (!file.startsWith(prefix) || !/\.(?:m?js)$/u.test(file)) return null;
  const owned = manifest.sources[file];
  const shared = manifest.sharedDependencies[file];
  if (!owned && !shared) throw Error('TypeScriptRuntimeProjectionMissing:' + file);
  const source = owned || shared;
  const bytes = read(source.path);
  if (digest(bytes) !== source.sha256) {
    throw Error('TypeScriptRuntimeProjectionChanged:' + source.logicalPath);
  }
  fs.appendFileSync(manifest.journal + process.pid + '-' + threadId + '.jsonl',
    JSON.stringify({typescript: owned ? source.logicalPath : null,
      sharedDependency: shared ? source.logicalPath : null,
      sha256: source.sha256, operation}) + '\n', {mode: 0o600});
  return bytes;
}
module.exports = {project};
