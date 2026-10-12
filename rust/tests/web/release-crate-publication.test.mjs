import test from 'node:test';
import assert from 'node:assert/strict';
import { cratePublicationRecord, validateCratePublicationRecord } from '../../../scripts/release-crate-publication.mjs';
const binding = {
  repository: 'link-assistant/formal-ai',
  sourceCommit: 'a'.repeat(40),
  sourceTree: 'b'.repeat(40),
  protocolCommit: 'c'.repeat(40),
  protocolSha256: 'd'.repeat(64),
  run: '1',
  attempt: '1',
  job: 'verify-package',
  packageName: 'formal-ai',
  version: '1.2.3'
};
const sourceBytes = Buffer.from(JSON.stringify({
  source_commit: binding.sourceCommit,
  source_tree: binding.sourceTree,
  package_version: binding.version,
  producer_run: binding.run
}));
function fixture() {
  const record = cratePublicationRecord(binding, {
    name: 'formal-ai-1.2.3.crate',
    bytes: Buffer.from('private checksum fixture')
  }, sourceBytes);
  return {
    record,
    registry: {
      crate: 'formal-ai',
      num: '1.2.3',
      checksum: record.archive.sha256,
      yanked: false
    }
  };
}
test('matching checksum data alone cannot mint authenticated completion', () => {
  const f = fixture();
  const result = validateCratePublicationRecord(f.record, binding, sourceBytes, f.registry);
  assert.equal(result.state, 'Unknown');
  assert.equal(result.complete, false);
  assert.equal(result.publicationAuthority, false);
});
test('all independently selected binding fields must agree', () => {
  for (const key of Object.keys(binding)) {
    const f = fixture(),
      other = {
        ...binding
      };
    other[key] = ['run', 'attempt'].includes(key) ? '2' : ['sourceCommit', 'sourceTree', 'protocolCommit'].includes(key) ? 'e'.repeat(40) : key === 'protocolSha256' ? 'e'.repeat(64) : key === 'version' ? '1.2.4' : key === 'repository' ? 'foreign/repository' : 'other';
    assert.throws(() => validateCratePublicationRecord(f.record, other, sourceBytes, f.registry));
  }
});
test('registry presence cannot replace checksum/name/version/yanked qualification', () => {
  const f = fixture();
  for (const registry of [{
    ...f.registry,
    checksum: 'e'.repeat(64)
  }, {
    ...f.registry,
    crate: 'foreign'
  }, {
    ...f.registry,
    num: '1.2.4'
  }, {
    ...f.registry,
    yanked: true
  }, {}]) {
    assert.throws(() => validateCratePublicationRecord(f.record, binding, sourceBytes, registry));
  }
});
test('unknown fields, unbounded archive, forged source and authority refuse', () => {
  const f = fixture();
  for (const record of [{
    ...f.record,
    complete: true
  }, {
    ...f.record,
    publicationAuthority: true
  }, {
    ...f.record,
    archive: {
      ...f.record.archive,
      bytes: 134217729
    }
  }, {
    ...f.record,
    archive: {
      ...f.record.archive,
      extra: true
    }
  }]) assert.throws(() => validateCratePublicationRecord(record, binding, sourceBytes, f.registry));
  assert.throws(() => validateCratePublicationRecord(f.record, binding, Buffer.from('{}'), f.registry));
  assert.throws(() => cratePublicationRecord({
    ...binding,
    run: 1
  }, {
    name: 'formal-ai-1.2.3.crate',
    bytes: Buffer.from('x')
  }, sourceBytes));
});
import { crateChecksumFromTransferDescriptor } from '../../../scripts/release-crate-publication.mjs';
const expected = {
  repository: binding.repository,
  run: binding.run,
  attempt: binding.attempt,
  protocol: binding.protocolCommit,
  source: binding.sourceCommit,
  tree: binding.sourceTree,
  version: binding.version,
  compilerSha256: 'e'.repeat(64),
  producerJob: 'auto_verify-package'
};
function transfer() {
  const {
    producerJob,
    ...identity
  } = expected;
  return {
    schema: 'release-stage-transfer/v1',
    kind: 'package',
    identity,
    producerJob,
    complete: true,
    status: 0,
    signal: null,
    inventory: [{
      path: 'rust/target/package/formal-ai-1.2.3.crate',
      bytes: 10,
      sha256: 'f'.repeat(64),
      mode: 0o644
    }, {
      path: 'rust/target/package/staged-package-contents.json',
      bytes: 20,
      sha256: 'a'.repeat(64),
      mode: 0o644
    }]
  };
}
test('existing transfer inventory supplies checksum without downloading archive or inventing source credit', () => {
  const result = crateChecksumFromTransferDescriptor(transfer(), expected);
  assert.equal(result.checksum, 'f'.repeat(64));
  assert.equal(result.bytes, 10);
  assert.equal(result.complete, false);
});
test('partial, foreign, duplicate, untyped and failed package descriptors refuse', () => {
  for (const mutation of [
    r => r.kind = 'binary',
    r => r.complete = false,
    r => r.status = 1,
    r => r.signal = 'SIGTERM',
    r => r.producerJob = 'auto_compile-release',
    r => r.identity.source = 'f'.repeat(40),
    r => r.inventory.pop(),
    r => r.inventory.push(r.inventory[0]),
    r => r.inventory[0].mode = 0o120777,
    r => r.inventory[0].sha256 = 'wrong',
    r => r.inventory[0].path = '../fake.crate'
  ]) {
    const record = transfer();
    mutation(record);
    assert.throws(() => crateChecksumFromTransferDescriptor(record, expected));
  }
});
import { createHash } from 'node:crypto';
import { crateRegistryDataRecord } from '../../../scripts/release-crate-publication.mjs';
const sum = bytes => createHash('sha256').update(bytes).digest('hex');
function registryFixture() {
  const contents = Buffer.from(JSON.stringify([{
    name: 'formal-ai-1.2.3/Cargo.toml',
    sha256: 'b'.repeat(64)
  }]));
  const descriptor = transfer(),
    entry = descriptor.inventory[1];
  entry.bytes = contents.length;
  entry.sha256 = sum(contents);
  return {
    contents,
    descriptor: Buffer.from(JSON.stringify(descriptor)),
    registry: Buffer.from('CI-only downloaded archive fixture'),
    observation: {
      url: 'https://static.crates.io/crates/formal-ai/formal-ai-1.2.3.crate',
      status: 200
    }
  };
}
test('registry data capture binds original package descriptor and full-content witness without completion credit', () => {
  const f = registryFixture();
  const result = crateRegistryDataRecord(expected, f.descriptor, f.contents, f.registry, f.observation);
  assert.equal(result.descriptorSha256, sum(f.descriptor));
  assert.equal(result.registryArchive.sha256, sum(f.registry));
  assert.equal(result.packageContentsSha256, sum(f.contents));
  assert.equal(result.complete, false);
  assert.equal(result.publicationAuthority, false);
});
test('registry source, URL, status, missing witness, traversal, duplicates and extra witness fields refuse', () => {
  const f = registryFixture();
  for (const observation of [{
    ...f.observation,
    status: 404
  }, {
    ...f.observation,
    url: 'https://foreign.example/archive'
  }, {
    ...f.observation,
    complete: true
  }]) assert.throws(() => crateRegistryDataRecord(expected, f.descriptor, f.contents, f.registry, observation));
  assert.throws(() => crateRegistryDataRecord(expected, f.descriptor, Buffer.from('[]'), f.registry, f.observation));
  for (const contents of [[{
    name: '../Cargo.toml',
    sha256: 'a'.repeat(64)
  }], [{
    name: 'Cargo.toml',
    sha256: 'a'.repeat(64)
  }, {
    name: 'Cargo.toml',
    sha256: 'a'.repeat(64)
  }], [{
    name: 'Cargo.toml',
    sha256: 'a'.repeat(64),
    credit: true
  }]]) {
    const bytes = Buffer.from(JSON.stringify(contents)),
      descriptor = JSON.parse(f.descriptor);
    descriptor.inventory[1].bytes = bytes.length;
    descriptor.inventory[1].sha256 = sum(bytes);
    assert.throws(() => crateRegistryDataRecord(expected, Buffer.from(JSON.stringify(descriptor)), bytes, f.registry, f.observation));
  }
});
test('existing registry comparison admits UTF8/spaced ordinary member names and ordinary read-only modes', () => {
  const f = registryFixture(),
    contents = Buffer.from(JSON.stringify([{
      name: 'formal-ai-1.2.3/data/é space.lino',
      sha256: 'a'.repeat(64)
    }]));
  const descriptor = JSON.parse(f.descriptor);
  descriptor.inventory[0].mode = 0o444;
  descriptor.inventory[1].bytes = contents.length;
  descriptor.inventory[1].sha256 = sum(contents);
  assert.equal(crateRegistryDataRecord(expected, Buffer.from(JSON.stringify(descriptor)), contents, f.registry, f.observation).complete, false);
});
