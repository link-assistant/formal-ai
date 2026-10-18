// Authenticate retained execution inputs before any archived source is evaluated.
import path from 'node:path';
import {sha256Hex as digest}from '../agentic/crate/source_fetch.mjs';
import {verifyDriveEvidence, regularEvidenceBytes, retainedEvidenceBytes, evidenceRecords} from './verify-drive.mjs';
import {observeRuntimeExecutable} from './fresh-runtime.mjs';
import {readBoundedFileBytes} from './bounded-file.mjs';
import {decodeTyped} from '../logging/typed-codec.mjs';
import {blake3Hex} from '../crypto/blake3.mjs';


function bindUnique(map, key, value) {
  if (map.has(key) && JSON.stringify(map.get(key)) !== JSON.stringify(value)) {
    throw Error('ArchivedRuntimeAmbiguousInput');
  }
  map.set(key, value);
}
export function authenticatedRuntimeArchive(options) {
  const {directory, workspace, expectedAnchor, trustedPublicKey, expectedRuntimeSha256,
    expectedSourceIdentity, expectedTaskURL, bootstrapDirectory = path.dirname(path.dirname(directory))} = options;
  if (!expectedAnchor || typeof trustedPublicKey !== 'string' || !trustedPublicKey
      || !/^[a-f0-9]{64}$/u.test(expectedRuntimeSha256)
      || !/^[a-f0-9]{64}$/u.test(expectedSourceIdentity)
      || typeof expectedTaskURL !== 'string') throw Error('ArchivedRuntimeIndependentTrustRequired');
  const checked = verifyDriveEvidence(directory, workspace, expectedAnchor, {trustedPublicKey});
  const records = regularEvidenceBytes(directory, 'chain.lino').toString('utf8')
    .trimEnd().split('\n').map(decodeTyped);
  if (checked.logicalMemory?.taskURL !== expectedTaskURL) throw Error('ArchivedRuntimeTaskMismatch');
  if (records[0].payload.sourceId !== expectedSourceIdentity) throw Error('ArchivedRuntimeSourceMismatch');
  if (observeRuntimeExecutable().sha256 !== expectedRuntimeSha256) throw Error('ArchivedRuntimeExecutableMismatch');
  const contexts = evidenceRecords(records,'runtime_context');
  const inventories = evidenceRecords(records,'runtime_sources');
  if (contexts.length !== 1 || inventories.length !== 2) throw Error('ArchivedRuntimeInventoryRequired');
  const retained = retainedEvidenceBytes.bind(null,directory);
  const context = JSON.parse(retained(contexts[0].payload.evidence).toString('utf8'));
  const modules = new Map(), reads = new Map(), esm = new Map(), commonjs = new Map();
  let total = 0;
  for (const inventory of inventories) {
    const value = JSON.parse(retained(inventory.payload.evidence).toString('utf8'));
    if (value.runtimeExecutable?.sha256 !== expectedRuntimeSha256) throw Error('ArchivedRuntimeObservedExecutableMismatch');
    for (const source of value.sources) {
      const address = new URL(source.url);
      if (address.protocol !== 'file:' || address.search || address.hash) throw Error('ArchivedRuntimeSourceURL');
      const body = JSON.parse(retained(source.evidence).toString('utf8'));
      const bytes = Buffer.from(body.bytes, 'base64');
      if (bytes.toString('base64') !== body.bytes || bytes.length !== source.size
          || bytes.length > 4 * 1024 * 1024 || blake3Hex(bytes) !== source.blake3) {
        throw Error('ArchivedRuntimeSourceBytes');
      }
      const descriptor = {bytes: body.bytes, sha256: digest(bytes), size: bytes.length};
      if (source.format.startsWith('repository-read-')) {
        bindUnique(reads, source.url + '\0' + source.format, descriptor);
      } else if (['module', 'commonjs', 'json'].includes(source.format)) {
        bindUnique(modules, source.url, {format: source.format, ...descriptor});
      } else throw Error('ArchivedRuntimeUnsupportedSourceFormat');
    }
    for (const event of value.events) {
      if (event.kind === 'esm-resolve') bindUnique(esm,
        JSON.stringify([event.parentURL, event.specifier]), event.url);
      if (event.kind === 'resolve') bindUnique(commonjs,
        JSON.stringify([event.parent, event.request]), event.resolved);
    }
  }
  for (const entry of [...modules.values(), ...reads.values()]) total += entry.size;
  if (total > 32 * 1024 * 1024) throw Error('ArchivedRuntimeBudget');
  const expected = new Map([['bootstrap.cjs', 'js/provenance/runtime-journal.js'],
    ['loader.mjs', 'js/provenance/runtime-loader.mjs'],
    ['main.mjs', 'js/provenance/fresh-planner-main.mjs'],
    ['entry.cjs', 'js/provenance/runtime-entry.js']]);
  const bootstrap = {};
  if (context.bootstrapArtifacts.length !== expected.size) throw Error('ArchivedRuntimeBootstrapInventory');
  for (const artifact of context.bootstrapArtifacts) {
    if (expected.get(artifact.file) !== artifact.source) throw Error('ArchivedRuntimeBootstrapIdentity');
    expected.delete(artifact.file);
    const bytes = readBoundedFileBytes(bootstrapDirectory, artifact.file, 1024 * 1024);
    if (bytes.length !== artifact.size || blake3Hex(bytes) !== artifact.blake3) {
      throw Error('ArchivedRuntimeBootstrapBytes');
    }
    bootstrap[artifact.file] = {bytes: bytes.toString('base64'), sha256: digest(bytes), size: bytes.length};
  }
  return {schema: 'formal-ai-authenticated-runtime-archive-v1', repository: context.workingDirectory,
    modules: Object.fromEntries(modules), reads: Object.fromEntries(reads),
    esm: Object.fromEntries(esm), commonjs: Object.fromEntries(commonjs), bootstrap,
    context, expectedRuntimeSha256, expectedSourceIdentity, expectedTaskURL,
    scope: 'signed-retained-inputs-for-explicitly-pinned-js-runtime',
    fullReplay: {verified: false}, autonomousAuthorship: 'not-established'};
}
