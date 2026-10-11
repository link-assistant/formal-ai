import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const hash = value => assert.match(value, /^[a-f0-9]{64}$/u);
const commit = value => assert.match(value, /^[a-f0-9]{40}$/u);
const positive = value => assert.match(String(value), /^[1-9][0-9]*$/u);
export function validatePublicationBinding(binding) {
  assert.deepEqual(Object.keys(binding).sort(), ['attempt', 'packageName', 'protocolCommit', 'protocolSha256', 'run', 'sourceCommit', 'sourceTree', 'version'].sort());
  commit(binding.sourceCommit);
  commit(binding.sourceTree);
  commit(binding.protocolCommit);
  hash(binding.protocolSha256);
  positive(binding.run);
  positive(binding.attempt);
  assert.match(binding.version, /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
  assert.match(binding.packageName, /^(?:@[a-z0-9._-]+\/)?[a-z0-9._-]+$/u);
  return binding;
}
export function browserPublicationRecord(binding, archive, workerReceiptBytes, status) {
  validatePublicationBinding(binding);
  assert.ok(Buffer.isBuffer(archive.bytes) && archive.bytes.length > 0);
  assert.match(archive.name, /^[A-Za-z0-9_.-]+\.tgz$/u);
  assert.ok(Buffer.isBuffer(workerReceiptBytes) && workerReceiptBytes.length > 0 && workerReceiptBytes.length <= 1048576);
  assert.ok(['published', 'already-matching', 'skipped-unconfigured'].includes(status));
  return {
    schema: 'release-browser-publication/v1',
    binding: structuredClone(binding),
    archive: {
      name: archive.name,
      bytes: archive.bytes.length,
      sha256: digest(archive.bytes),
      integrity: 'sha512-' + createHash('sha512').update(archive.bytes).digest('base64')
    },
    workerReceiptSha256: digest(workerReceiptBytes),
    npm: {
      status,
      required: status !== 'skipped-unconfigured'
    },
    publicationAuthority: false
  };
}
export function validateBrowserPublicationRecord(record, expected, publicAsset, workerReceiptBytes, npmMetadata) {
  validatePublicationBinding(expected);
  assert.equal(record.schema, 'release-browser-publication/v1');
  assert.deepEqual(record.binding, expected);
  assert.equal(record.publicationAuthority, false);
  const archive = record.archive;
  assert.match(archive.name, /^[A-Za-z0-9_.-]+\.tgz$/u);
  assert.ok(Number.isSafeInteger(archive.bytes) && archive.bytes > 0);
  hash(archive.sha256);
  assert.match(archive.integrity, /^sha512-[A-Za-z0-9+/]{86}==$/u);
  assert.equal(publicAsset.name, archive.name);
  assert.equal(publicAsset.size, archive.bytes);
  assert.equal(publicAsset.digest, 'sha256:' + archive.sha256);
  assert.ok(Buffer.isBuffer(workerReceiptBytes) && workerReceiptBytes.length > 0 && workerReceiptBytes.length <= 1048576);
  assert.equal(record.workerReceiptSha256, digest(workerReceiptBytes));
  assert.ok(['published', 'already-matching', 'skipped-unconfigured'].includes(record.npm.status));
  assert.equal(record.npm.required, record.npm.status !== 'skipped-unconfigured');
  if (record.npm.required) {
    assert.equal(npmMetadata.name, expected.packageName);
    assert.equal(npmMetadata.version, expected.version);
    assert.equal(npmMetadata.dist.integrity, archive.integrity);
  } else assert.equal(npmMetadata, null);
  return {
    schema: 'release-browser-structural-qualification/v1',
    publicationAuthority: false,
    complete: false,
    authority: 'RequiresIndependentSourceAndDataAttestation',
    archiveSha256: archive.sha256
  };
}
import { readFileSync, writeFileSync, lstatSync, realpathSync, mkdirSync, existsSync } from 'node:fs';
import { join, resolve, basename, dirname } from 'node:path';
import { pathToFileURL } from 'node:url';
import { verifyReleaseProtocol, verifySelectedPackage } from './release-protocol.mjs';
import { verifySelectedWasm } from './build-selected-wasm.mjs';
function ordinaryBytes(path, maximum) {
  const absolute = resolve(path),
    stat = lstatSync(absolute);
  assert.ok(stat.isFile() && !stat.isSymbolicLink());
  assert.equal(realpathSync(absolute), absolute);
  assert.ok(stat.size > 0 && stat.size <= maximum);
  const bytes = readFileSync(absolute);
  assert.ok(bytes.length > 0 && bytes.length <= maximum);
  assert.equal(bytes.length, stat.size);
  return bytes;
}
export function captureBrowserPublication(root, protocolDirectory, outputDirectory, environment = process.env) {
  assert.equal(environment.GITHUB_ACTIONS, 'true', 'publication receipts are captured only by the actual CI producer');
  const sourceCommit = environment.PACKAGE_SOURCE_COMMIT;
  const protocolCommit = environment.RELEASE_PROTOCOL_COMMIT;
  commit(sourceCommit);
  commit(protocolCommit);
  hash(environment.RELEASE_PROTOCOL_SHA256);
  hash(environment.WASM_RECEIPT_SHA256);
  assert.equal(environment.PUBLICATION_REQUESTED, 'true');
  assert.match(environment.GITHUB_REPOSITORY, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  const protocol = verifyReleaseProtocol(root, protocolDirectory, protocolCommit, environment.RELEASE_PROTOCOL_SHA256);
  assert.ok(protocol.files.some(file => file.path === 'scripts/release-publication-observation.mjs'), 'receipt helper must be captured in the independently pinned workflow protocol');
  const selected = verifySelectedPackage(root, sourceCommit, 'packages/formal-ai-engine/package.json', true);
  const identity = {
    repository: environment.GITHUB_REPOSITORY,
    run: environment.GITHUB_RUN_ID,
    attempt: environment.GITHUB_RUN_ATTEMPT,
    job: environment.GITHUB_JOB,
    event: environment.GITHUB_EVENT_NAME,
    workflowCommit: protocolCommit
  };
  const wasm = verifySelectedWasm(root, join(root, 'js'), environment.WASM_RECEIPT_SHA256, {
    bindings: identity
  });
  assert.equal(wasm.sourceCommit, selected.sourceCommit);
  assert.equal(wasm.sourceTree, selected.sourceTree);
  assert.equal(wasm.packageVersion, selected.derivedVersion);
  const binding = {
    sourceCommit: selected.sourceCommit,
    sourceTree: selected.sourceTree,
    protocolCommit,
    protocolSha256: protocol.sha256,
    run: identity.run,
    attempt: identity.attempt,
    packageName: selected.packagePath && JSON.parse(ordinaryBytes(join(root, selected.packagePath), 1048576)).name,
    version: selected.derivedVersion
  };
  const filename = binding.packageName.replace(/^@/u, '').replace('/', '-') + '-' + binding.version + '.tgz';
  const archive = ordinaryBytes(join(root, 'packages/formal-ai-engine', filename), 134217728);
  const worker = ordinaryBytes(join(root, 'js/formal_ai_worker.receipt.json'), 1048576);
  assert.equal(digest(worker), environment.WASM_RECEIPT_SHA256);
  const record = browserPublicationRecord(binding, {
    name: filename,
    bytes: archive
  }, worker, environment.PUBLICATION_STATUS);
  record.producer = identity;
  const directory = resolve(outputDirectory);
  assert.equal(realpathSync(dirname(directory)), dirname(directory));
  assert.equal(existsSync(directory), false, 'a fresh data-only output directory is required');
  mkdirSync(directory);
  writeFileSync(join(directory, 'browser-publication.json'), JSON.stringify(record, null, 2) + '\n');
  writeFileSync(join(directory, 'worker-receipt.json'), worker);
  writeFileSync(join(directory, 'protocol-selection.json'), ordinaryBytes(join(protocolDirectory, 'protocol-selection.json'), 1048576));
  return {
    schema: 'release-browser-data-artifact/v1',
    directory,
    members: ['browser-publication.json', 'worker-receipt.json', 'protocol-selection.json'],
    complete: false,
    publicationAuthority: false
  };
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const [operation, root, protocol, output, ...extra] = process.argv.slice(2);
  assert.equal(operation, 'capture-browser');
  assert.equal(extra.length, 0);
  assert.ok(root && protocol && output);
  console.log(JSON.stringify(captureBrowserPublication(root, protocol, output)));
}
