// The archive is constructor-authenticated; unknown engine imports fail closed.
import fileSystem from 'node:fs';
import {createRequire} from 'node:module';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath, pathToFileURL} from 'node:url';
const configuration = JSON.parse(fileSystem.readFileSync(process.env.FORMAL_AI_RUNTIME_CONFIG, 'utf8'));
const archive = configuration.sourceArchive;
if (!archive || archive.schema !== 'formal-ai-authenticated-runtime-archive-v1') {
  throw Error('ArchivedRuntimeConfigurationRequired');
}
const ownURL = value => typeof value === 'string' && value.startsWith('file:')
  && fileURLToPath(value).startsWith(archive.repository + path.sep);
export async function resolve(specifier, context, nextResolve) {
  const observed = archive.esm[JSON.stringify([context.parentURL ?? null, specifier])];
  if (typeof observed === 'string' && archive.modules[observed]) {
    return {url: observed, format: archive.modules[observed].format, shortCircuit: true};
  }
  if ((Object.hasOwn(archive.modules,specifier)||Object.hasOwn(archive.modules,context.parentURL??'')||ownURL(context.parentURL)) && !specifier.startsWith('node:')) {
    throw Error('ArchivedRuntimeUnobservedResolution');
  }
  return nextResolve(specifier, context);
}
export async function load(url, context, nextLoad) {
  const entry = archive.modules[url];
  if (entry) {
    let bytes = Buffer.from(entry.bytes, 'base64');
    if (bytes.toString('base64') !== entry.bytes || bytes.length !== entry.size
        || crypto.createHash('sha256').update(bytes).digest('hex') !== entry.sha256) {
      throw Error('ArchivedRuntimeTransportBytes');
    }
    if(archive.reads[url+'\0repository-read-string']){
      const delivered=fileSystem.readFileSync(fileURLToPath(url),'utf8');
      bytes=Buffer.from(delivered);
      if(bytes.toString('base64')!==entry.bytes)throw Error('ArchivedRuntimeCompilerReadMismatch');
    }
    if (process.env.FORMAL_AI_RUNTIME_BOOTSTRAP) {
      const observer = createRequire(import.meta.url)(process.env.FORMAL_AI_RUNTIME_BOOTSTRAP);
      observer.source(url, entry.format, bytes, 'actual-authenticated-archive-esm-load-return');
    }
    if (entry.format === 'commonjs') return {format: 'commonjs', source: bytes, shortCircuit: true};
    return {format: entry.format, source: bytes, shortCircuit: true};
  }
  if (ownURL(url)) throw Error('ArchivedRuntimeUnobservedLoad');
  return nextLoad(url, context);
}
