import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
function exactKeys(value, keys) {
  assert.ok(value && typeof value === 'object' && !Array.isArray(value));
  assert.deepEqual(Object.keys(value).sort(), [...keys].sort());
}
function identity(binding) {
  exactKeys(binding, ['repository', 'sourceCommit', 'sourceTree', 'protocolCommit', 'protocolSha256', 'run', 'attempt', 'job', 'packageName', 'version']);
  assert.match(binding.repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  for (const key of ['sourceCommit', 'sourceTree', 'protocolCommit']) assert.match(binding[key], /^[a-f0-9]{40}$/u);
  assert.match(binding.protocolSha256, /^[a-f0-9]{64}$/u);
  for (const key of ['run', 'attempt']) assert.match(binding[key], /^[1-9][0-9]*$/u);
  assert.match(binding.job, /^[A-Za-z0-9_-]+$/u);
  assert.match(binding.packageName, /^[A-Za-z0-9_-]+$/u);
  assert.match(binding.version, /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
}
export function cratePublicationRecord(binding, archive, selectedSourceReceiptBytes) {
  identity(binding);
  exactKeys(archive, ['name', 'bytes']);
  assert.equal(archive.name, binding.packageName + '-' + binding.version + '.crate');
  assert.ok(Buffer.isBuffer(archive.bytes) && archive.bytes.length > 0 && archive.bytes.length <= 134217728);
  assert.ok(Buffer.isBuffer(selectedSourceReceiptBytes) && selectedSourceReceiptBytes.length > 0 && selectedSourceReceiptBytes.length <= 1048576);
  const source = JSON.parse(selectedSourceReceiptBytes.toString('utf8'));
  assert.equal(source.source_commit, binding.sourceCommit);
  assert.equal(source.source_tree, binding.sourceTree);
  assert.equal(source.package_version, binding.version);
  assert.equal(source.producer_run, binding.run);
  return {
    schema: 'release-crate-publication/v1',
    binding: structuredClone(binding),
    archive: {
      name: archive.name,
      bytes: archive.bytes.length,
      sha256: sha256(archive.bytes)
    },
    sourceReceiptSha256: sha256(selectedSourceReceiptBytes),
    publicationAuthority: false
  };
}
export function validateCratePublicationRecord(record, expected, sourceBytes, registryVersion) {
  identity(expected);
  exactKeys(record, ['schema', 'binding', 'archive', 'sourceReceiptSha256', 'publicationAuthority']);
  assert.equal(record.schema, 'release-crate-publication/v1');
  assert.deepEqual(record.binding, expected);
  assert.equal(record.publicationAuthority, false);
  exactKeys(record.archive, ['name', 'bytes', 'sha256']);
  assert.equal(record.archive.name, expected.packageName + '-' + expected.version + '.crate');
  assert.ok(Number.isSafeInteger(record.archive.bytes) && record.archive.bytes > 0 && record.archive.bytes <= 134217728);
  assert.match(record.archive.sha256, /^[a-f0-9]{64}$/u);
  assert.ok(Buffer.isBuffer(sourceBytes) && sourceBytes.length > 0 && sourceBytes.length <= 1048576);
  assert.equal(record.sourceReceiptSha256, sha256(sourceBytes));
  const source = JSON.parse(sourceBytes.toString('utf8'));
  assert.equal(source.source_commit, expected.sourceCommit);
  assert.equal(source.source_tree, expected.sourceTree);
  assert.equal(source.package_version, expected.version);
  assert.equal(source.producer_run, expected.run);
  assert.equal(registryVersion.crate, expected.packageName);
  assert.equal(registryVersion.num, expected.version);
  assert.equal(registryVersion.checksum, record.archive.sha256);
  assert.equal(registryVersion.yanked, false);
  return Object.freeze({
    state: 'Unknown',
    reason: 'Requires authenticated source-owned producer and registry observations',
    complete: false,
    publicationAuthority: false
  });
}
export function crateChecksumFromTransferDescriptor(descriptor, expected) {
  exactKeys(expected, ['repository', 'run', 'attempt', 'protocol', 'source', 'tree', 'version', 'compilerSha256', 'producerJob']);
  assert.equal(descriptor.schema, 'release-stage-transfer/v1');
  assert.equal(descriptor.kind, 'package');
  assert.equal(descriptor.complete, true);
  assert.equal(descriptor.status, 0);
  assert.equal(descriptor.signal, null);
  assert.match(expected.producerJob, /^(?:auto|manual)_verify-package$/u);
  assert.equal(descriptor.producerJob, expected.producerJob);
  for (const key of ['repository', 'run', 'attempt', 'protocol', 'source', 'tree', 'version', 'compilerSha256']) assert.equal(descriptor.identity[key], expected[key]);
  assert.match(expected.repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  for (const key of ['run', 'attempt']) assert.match(expected[key], /^[1-9][0-9]*$/u);
  for (const key of ['protocol', 'source', 'tree']) assert.match(expected[key], /^[a-f0-9]{40}$/u);
  assert.match(expected.compilerSha256, /^[a-f0-9]{64}$/u);
  assert.match(expected.version, /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
  assert.ok(Array.isArray(descriptor.inventory));
  const names = ['rust/target/package/formal-ai-' + expected.version + '.crate', 'rust/target/package/staged-package-contents.json'];
  assert.deepEqual(descriptor.inventory.map(entry => entry.path).sort(), [...names].sort());
  for (const entry of descriptor.inventory) {
    exactKeys(entry, ['path', 'bytes', 'sha256', 'mode']);
    assert.ok(Number.isSafeInteger(entry.bytes) && entry.bytes > 0 && entry.bytes <= 67108864);
    assert.match(entry.sha256, /^[a-f0-9]{64}$/u);
    assert.ok(Number.isInteger(entry.mode) && entry.mode >= 0 && entry.mode <= 0o777);
  }
  const archive = descriptor.inventory.find(entry => entry.path === names[0]);
  return Object.freeze({
    checksum: archive.sha256,
    bytes: archive.bytes,
    complete: false,
    authority: 'Requires independently authenticated descriptor producer'
  });
}
export function crateRegistryDataRecord(expected, descriptorBytes, packageContentsBytes, registryBytes, registryObservation) {
  assert.ok(Buffer.isBuffer(descriptorBytes) && descriptorBytes.length > 0 && descriptorBytes.length <= 1048576);
  assert.ok(Buffer.isBuffer(packageContentsBytes) && packageContentsBytes.length > 0 && packageContentsBytes.length <= 1048576);
  assert.ok(Buffer.isBuffer(registryBytes) && registryBytes.length > 0 && registryBytes.length < 67108864);
  exactKeys(registryObservation, ['url', 'status']);
  assert.equal(registryObservation.url, 'https://static.crates.io/crates/formal-ai/formal-ai-' + expected.version + '.crate');
  assert.equal(registryObservation.status, 200);
  const descriptor = JSON.parse(descriptorBytes.toString('utf8'));
  const archive = crateChecksumFromTransferDescriptor(descriptor, expected);
  const entry = descriptor.inventory.find(item => item.path === 'rust/target/package/staged-package-contents.json');
  assert.equal(entry.bytes, packageContentsBytes.length);
  assert.equal(entry.sha256, sha256(packageContentsBytes));
  const contents = JSON.parse(packageContentsBytes.toString('utf8'));
  assert.ok(Array.isArray(contents) && contents.length > 0);
  assert.ok(contents.length <= 65536);
  const names = [];
  for (const item of contents) {
    exactKeys(item, ['name', 'sha256']);
    assert.ok(typeof item.name === 'string' && item.name.length > 0 && item.name.length <= 4096);
    assert.ok(!item.name.includes(String.fromCharCode(0)) && !item.name.startsWith('/') && !item.name.split('/').includes('..'));
    assert.match(item.sha256, /^[a-f0-9]{64}$/u);
    names.push(item.name);
  }
  assert.deepEqual(names, [...new Set(names)].sort());
  return {
    schema: 'release-crate-registry-data/v1',
    identity: structuredClone(expected),
    descriptorSha256: sha256(descriptorBytes),
    packageArchive: archive,
    packageContentsSha256: sha256(packageContentsBytes),
    registryArchive: {
      url: registryObservation.url,
      status: 200,
      bytes: registryBytes.length,
      sha256: sha256(registryBytes)
    },
    producerQualification: 'Requires authenticated successful source-owned registry comparison producer',
    complete: false,
    publicationAuthority: false
  };
}
