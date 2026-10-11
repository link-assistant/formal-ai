import { advertisementProtocolPaths, validateDefinition, validateDigestCoverage, verifyAdvertisementAttestation, validateSnapshotFiles } from '../../../scripts/release-advertisement.mjs';
import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { ADVERTISEMENT_ROOTS, deriveAdvertisement, validateAdvertisement, deriveDefinition, loadSources } from '../../../scripts/generate-release-advertisement.mjs';
const root = process.env.ADVERTISEMENT_SOURCE_ROOT ?? process.cwd();
const sources = loadSources(root, process.env.ADVERTISEMENT_WORKFLOW ?? null);
const bindings = {
  commit: '1'.repeat(40),
  tree: '2'.repeat(40),
  protocolSha256: '3'.repeat(64)
};
test('advertisement derives exact maintained producer matrices and optional npm policy', () => {
  const actual = deriveAdvertisement(sources, bindings);
  assert.equal(actual.targets.native.length, 8);
  assert.equal(actual.targets.desktop.length, 6);
  assert.equal(actual.targets.cli.length, 5);
  assert.equal(actual.targets.containers.length, 4);
  assert.equal(actual.targets.browser.npm, 'required-if-configured');
  assert.equal(actual.publicationAuthority, false);
  validateAdvertisement(actual, sources, bindings);
});
test('missing source and altered source bindings refuse', () => {
  const missing = {
    ...sources
  };
  delete missing[ADVERTISEMENT_ROOTS[0]];
  assert.throws(() => deriveAdvertisement(missing, bindings));
  const record = deriveAdvertisement(sources, bindings);
  assert.throws(() => validateAdvertisement({
    ...record,
    commit: '4'.repeat(40)
  }, sources, bindings));
  assert.throws(() => validateAdvertisement({
    ...record,
    targets: {
      ...record.targets,
      native: record.targets.native.slice(1)
    }
  }, sources, bindings));
});
test('duplicate and dynamic matrix targets refuse without widening', () => {
  const key = ADVERTISEMENT_ROOTS[0],
    text = sources[key].toString();
  const line = text.split('\n').find(row => row.includes('target: x86_64-unknown-linux-gnu, binext'));
  assert.ok(line);
  assert.throws(() => deriveAdvertisement({
    ...sources,
    [key]: Buffer.from(text.replace(line, line + '\n' + line))
  }, bindings));
  assert.throws(() => deriveAdvertisement({
    ...sources,
    [key]: Buffer.from(text.replace('target: x86_64-unknown-linux-gnu, binext', 'target: ${{ inputs.target }}, binext'))
  }, bindings));
});
test('packaging asset metadata and conditional registry policy derive from producers', () => {
  const record = deriveAdvertisement(sources, bindings);
  assert.equal(record.targets.desktopAssets.length, 14);
  assert.equal(record.distribution.crate.name, 'formal-ai');
  assert.equal(record.distribution.dockerHub.policy, 'required-if-configured');
  const bad = {
    ...sources,
    'desktop/package.json': Buffer.from(JSON.stringify({
      ...JSON.parse(sources['desktop/package.json']),
      build: {}
    }))
  };
  assert.throws(() => deriveAdvertisement(bad, bindings));
});
test('source-advertised metadata refuses missing assets and digest disagreement', () => {
  const record = deriveAdvertisement(sources, bindings);
  validateDefinition(record);
  const empty = validateDigestCoverage(record, '1.2.3', Buffer.from('a'.repeat(64) + '  unrelated.txt\n'), []);
  assert.equal(empty.state, 'MissingAdvertisedAssets');
  assert.equal(empty.complete, false);
  const assets = empty.expected.map(name => ({
    name,
    size: 1,
    digest: 'sha256:' + 'a'.repeat(64)
  }));
  const checksums = Buffer.from(empty.expected.map(name => 'a'.repeat(64) + '  ' + name).join('\n') + '\n');
  const covered = validateDigestCoverage(record, '1.2.3', checksums, assets);
  assert.equal(covered.state, 'DigestCoverageOnly');
  assert.equal(covered.complete, false);
  assert.equal(covered.attestationAuthority, 'Unknown');
  assert.equal(covered.publicationAuthority, false);
  assert.throws(() => validateDigestCoverage(record, '1.2.3', checksums, [{
    ...assets[0],
    digest: 'sha256:' + 'b'.repeat(64)
  }, ...assets.slice(1)]));
  assert.throws(() => validateDigestCoverage(record, '1.2.3', checksums, [...assets, assets[0]]));
  assert.throws(() => validateDigestCoverage(record, '1.2.3', Buffer.concat([checksums, checksums]), assets));
});
test('unsafe definition paths and unexpected input declarations refuse', () => {
  const record = deriveAdvertisement(sources, bindings);
  assert.throws(() => validateDefinition({
    ...record,
    files: [{
      ...record.files[0],
      path: '../escape'
    }]
  }));
  assert.throws(() => validateDefinition({
    ...record,
    files: [{
      ...record.files[0],
      unexpected: true
    }]
  }));
});
test('checked source declaration matches full generated metadata', () => {
  const path = process.env.ADVERTISEMENT_DEFINITION ?? join(root, '.github/release-advertisement.json');
  assert.deepEqual(JSON.parse(readFileSync(path)), deriveDefinition(sources));
});
test('native receipt registry mismatch refuses source declaration', () => {
  const key = 'scripts/native-release-artifact.mjs';
  assert.throws(() => deriveAdvertisement({
    ...sources,
    [key]: Buffer.from(sources[key].toString().replace('x86_64-unknown-linux-gnu', 'unsupported-target'))
  }, bindings));
});
test('invalid attestation identities refuse before any process or network observation', () => {
  assert.throws(() => verifyAdvertisementAttestation('/nonexistent', '--repo', '1'.repeat(40)));
  assert.throws(() => verifyAdvertisementAttestation('/nonexistent', 'owner/repository', 'bad'));
});
test('version-only source changes preserve declared targets while snapshot bytes remain separately bound', () => {
  const record = deriveDefinition(sources);
  const key = 'rust/Cargo.toml';
  const changed = {
    ...sources,
    [key]: Buffer.from(sources[key].toString().replace(/^version\s*=\s*"[^"]+"/m, 'version = "9.8.7"'))
  };
  assert.deepEqual(deriveDefinition(changed), record);
});
test('independent snapshot byte metadata rejects missing, oversized, nonblob, and duplicate inputs', () => {
  const definition = deriveDefinition(sources);
  const files = definition.files.map(file => ({
    path: file.path,
    mode: '100644',
    bytes: 1,
    sha256: 'a'.repeat(64)
  }));
  validateSnapshotFiles(definition, files);
  assert.throws(() => validateSnapshotFiles(definition, files.slice(1)));
  assert.throws(() => validateSnapshotFiles(definition, [{
    ...files[0],
    bytes: 1024 * 1024 + 1
  }, ...files.slice(1)]));
  assert.throws(() => validateSnapshotFiles(definition, [{
    ...files[0],
    mode: '120000'
  }, ...files.slice(1)]));
  assert.throws(() => validateSnapshotFiles(definition, [files[1], ...files.slice(1)]));
});

test('shared protocol and producer roles use one source blob identity', () => {
  const definition = deriveDefinition(sources);
  const required = advertisementProtocolPaths(definition);
  const overlap = required.find(path => !definition.files.some(file => file.path === path));
  assert.ok(overlap);
  const shared = {...definition, files: [...definition.files, {path: overlap}]};
  const paths = advertisementProtocolPaths(shared);
  assert.deepEqual(paths, required);
  assert.equal(new Set(paths).size, paths.length);
  assert.ok(shared.files.some(file => file.path === overlap));
});
test('duplicates within the advertised source role still refuse', () => {
  const definition = deriveDefinition(sources);
  const duplicated = {...definition, files: [...definition.files, definition.files[0]]};
  assert.throws(() => advertisementProtocolPaths(duplicated));
});
test('unsafe advertised source paths still refuse before deduplicating roles', () => {
  const definition = deriveDefinition(sources);
  for (const path of ['../source.mjs', '.git/config', '/source.mjs'])
    assert.throws(() => advertisementProtocolPaths({...definition, files: [{path}]}));
});
