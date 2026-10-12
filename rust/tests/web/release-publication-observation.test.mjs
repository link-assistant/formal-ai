import test from 'node:test';
import assert from 'node:assert/strict';
import { browserPublicationRecord, validateBrowserPublicationRecord } from '../../../scripts/release-publication-observation.mjs';
const binding = {
  sourceCommit: 'a'.repeat(40),
  sourceTree: 'b'.repeat(40),
  protocolCommit: 'c'.repeat(40),
  protocolSha256: 'd'.repeat(64),
  run: '1',
  attempt: '1',
  packageName: '@example/engine',
  version: '1.2.3'
};
const bytes = Buffer.from('private fixture only'),
  worker = Buffer.from('{}');
function example(status = 'published') {
  const record = browserPublicationRecord(binding, {
    name: 'engine-1.2.3.tgz',
    bytes
  }, worker, status);
  const asset = {
    name: record.archive.name,
    size: record.archive.bytes,
    digest: 'sha256:' + record.archive.sha256
  };
  const npm = status === 'skipped-unconfigured' ? null : {
    name: binding.packageName,
    version: binding.version,
    dist: {
      integrity: record.archive.integrity
    }
  };
  return {
    record,
    asset,
    npm
  };
}
test('structural fixture cannot grant completion or publication authority', () => {
  for (const status of ['published', 'already-matching', 'skipped-unconfigured']) {
    const e = example(status),
      result = validateBrowserPublicationRecord(e.record, binding, e.asset, worker, e.npm);
    assert.equal(result.complete, false);
    assert.equal(result.publicationAuthority, false);
  }
});
test('each source/run/protocol binding refuses a different authenticated expectation', () => {
  for (const key of Object.keys(binding)) {
    const e = example(),
      expected = {
        ...binding,
        [key]: key === 'version' ? '1.2.4' : key === 'packageName' ? '@example/other' : key === 'run' || key === 'attempt' ? '2' : 'e'.repeat(binding[key].length)
      };
    assert.throws(() => validateBrowserPublicationRecord(e.record, expected, e.asset, worker, e.npm));
  }
});
test('API digest, length, name, worker and npm integrity refusals', () => {
  const e = example();
  for (const asset of [{
    ...e.asset,
    digest: 'sha256:' + 'f'.repeat(64)
  }, {
    ...e.asset,
    size: e.asset.size + 1
  }, {
    ...e.asset,
    name: 'other.tgz'
  }]) assert.throws(() => validateBrowserPublicationRecord(e.record, binding, asset, worker, e.npm));
  assert.throws(() => validateBrowserPublicationRecord(e.record, binding, e.asset, Buffer.from('changed'), e.npm));
  assert.throws(() => validateBrowserPublicationRecord(e.record, binding, e.asset, worker, {
    ...e.npm,
    dist: {
      integrity: 'wrong'
    }
  }));
});
test('optional policy cannot be changed by a boolean or unknown outcome', () => {
  const e = example('skipped-unconfigured');
  assert.throws(() => validateBrowserPublicationRecord({
    ...e.record,
    npm: {
      ...e.record.npm,
      required: true
    }
  }, binding, e.asset, worker, null));
  assert.throws(() => browserPublicationRecord(binding, {
    name: 'a.tgz',
    bytes
  }, worker, true));
  assert.throws(() => browserPublicationRecord(binding, {
    name: 'a.tgz',
    bytes
  }, worker, 'unknown'));
  assert.throws(() => validateBrowserPublicationRecord(e.record, binding, e.asset, worker, {}));
});
