import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, existsSync, realpathSync, lstatSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { captureReleaseProtocol, verifyReleaseProtocol } from './release-protocol.mjs';
export const DEFINITION_PATH = '.github/release-advertisement.json';
const ownPaths = [DEFINITION_PATH, 'scripts/release-advertisement.mjs', 'scripts/generate-release-advertisement.mjs', 'scripts/release-protocol.mjs'];
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export function validateDefinition(definition) {
  assert.equal(definition.schema, 'release-advertisement/v1');
  assert.equal(definition.publicationAuthority, false);
  assert.ok(Array.isArray(definition.files) && definition.files.length > 0 && definition.files.length <= 128);
  const names = new Set();
  for (const file of definition.files) {
    assert.match(file.path, /^[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*$/u);
    assert.ok(!file.path.split('/').some(part => ['.', '..', '.git'].includes(part)));
    assert.ok(!names.has(file.path));
    names.add(file.path);
    assert.deepEqual(Object.keys(file), ['path']);
  }
  for (const kind of ['native', 'desktop', 'cli', 'containers']) {
    const rows = definition.targets[kind];
    assert.ok(Array.isArray(rows) && rows.length > 0 && rows.length <= 128);
    assert.equal(new Set(rows.map(row => JSON.stringify(row))).size, rows.length);
    for (const row of rows) for (const value of Object.values(row)) {
      assert.equal(typeof value, 'string');
      assert.match(value, /^[A-Za-z0-9_.-]+$/u);
    }
  }
  assert.equal(definition.targets.browser.releaseAttachment, 'required');
  assert.equal(definition.targets.browser.npm, 'required-if-configured');
  const extension = definition.targets.extensions;
  if (extension !== undefined) {
    assert.deepEqual(Object.keys(extension).sort(), ['outcomes', 'package', 'publisher', 'registries', 'releaseAttachment', 'template']);
    for (const field of ['publisher', 'package']) assert.match(extension[field], /^[a-z0-9][a-z0-9_.-]*$/u);
    assert.equal(extension.releaseAttachment, 'required');
    assert.equal(extension.template, extension.package + '-$' + '{version}.vsix');
    assert.ok(Array.isArray(extension.registries) && extension.registries.length > 0 && extension.registries.length <= 128);
    assert.equal(new Set(extension.registries.map(row => row.name)).size, extension.registries.length);
    for (const row of extension.registries) {
      assert.deepEqual(Object.keys(row).sort(), ['credential', 'job', 'name', 'policy', 'workflow']);
      assert.match(row.name, /^[a-z][a-z0-9-]*$/u);
      assert.match(row.credential, /^[A-Z][A-Z0-9_]*$/u);
      assert.match(row.job, /^[A-Za-z0-9_-]+$/u);
      assert.equal(row.policy, 'required-if-configured');
      assert.ok(names.has(row.workflow), 'registry producer source is not declared');
    }
    assert.deepEqual(Object.keys(extension.outcomes).sort(), ['delivery', 'publicationComplete', 'schema', 'statuses']);
    assert.equal(extension.outcomes.delivery, 'Unknown');
    assert.equal(extension.outcomes.publicationComplete, false);
    assert.equal(extension.outcomes.schema, 'formal-ai-extension-publication-outcome/v1');
    assert.ok(Array.isArray(extension.outcomes.statuses) && extension.outcomes.statuses.length > 0 && extension.outcomes.statuses.length <= 128);
    assert.equal(new Set(extension.outcomes.statuses).size, extension.outcomes.statuses.length);
    for (const status of extension.outcomes.statuses) assert.match(status, /^[a-z][a-z0-9-]*$/u);
  }
  return definition;
}
export function advertisementProtocolPaths(definition) {
  validateDefinition(definition);
  assert.equal(new Set(ownPaths).size, ownPaths.length, 'duplicate protocol-role source input');
  return [...new Set([...ownPaths, ...definition.files.map(file => file.path)])].sort();
}
export function validateSnapshotFiles(definition, files) {
  validateDefinition(definition);
  assert.ok(Array.isArray(files));
  assert.equal(files.length, definition.files.length);
  assert.deepEqual(files.map(file => file.path).sort(), definition.files.map(file => file.path).sort());
  let total = 0;
  for (const file of files) {
    assert.ok(['100644', '100755'].includes(file.mode));
    assert.ok(Number.isSafeInteger(file.bytes) && file.bytes > 0 && file.bytes <= 1024 * 1024);
    assert.match(file.sha256, /^[a-f0-9]{64}$/u);
    total += file.bytes;
  }
  assert.ok(total <= 16 * 1024 * 1024);
  return files;
}
function snapshotDefinition(directory, protocol) {
  const bytes = readFileSync(join(directory, DEFINITION_PATH));
  assert.ok(bytes.length <= 1024 * 1024);
  const definition = validateDefinition(JSON.parse(bytes));
  const expected = advertisementProtocolPaths(definition);
  assert.deepEqual(protocol.files.map(file => file.path).sort(), expected);
  let total = 0;
  const sourceFiles = definition.files.map(file => {
    const entry = protocol.files.find(entry => entry.path === file.path);
    assert.ok(entry);
    assert.ok(entry.bytes > 0 && entry.bytes <= 1024 * 1024);
    total += entry.bytes;
    return entry;
  });
  assert.ok(total <= 16 * 1024 * 1024);
  validateSnapshotFiles(definition, sourceFiles);
  return {
    definition,
    sourceFiles,
    definitionSha256: digest(bytes)
  };
}
export function captureAdvertisement(root, directory, revision) {
  const definition = validateDefinition(JSON.parse(readFileSync(join(root, DEFINITION_PATH))));
  let total = 0;
  for (const file of definition.files) {
    const stat = lstatSync(join(root, file.path));
    assert.ok(stat.isFile() && !stat.isSymbolicLink() && stat.size > 0 && stat.size <= 1024 * 1024);
    total += stat.size;
  }
  assert.ok(total <= 16 * 1024 * 1024);
  const protocol = captureReleaseProtocol(root, directory, revision, advertisementProtocolPaths(definition));
  const captured = snapshotDefinition(directory, protocol);
  const record = {
    schema: 'release-advertisement-snapshot/v1',
    commit: protocol.commit,
    tree: protocol.tree,
    protocolSha256: protocol.sha256,
    ...captured,
    publicationAuthority: false
  };
  writeFileSync(join(directory, 'release-advertisement.json'), JSON.stringify(record, null, 2) + '\n');
  return record;
}
export function verifyAdvertisement(root, directory, revision, expectedProtocolSha256) {
  assert.match(expectedProtocolSha256, /^[a-f0-9]{64}$/u, 'independent protocol digest required');
  const protocol = verifyReleaseProtocol(root, directory, revision, expectedProtocolSha256);
  const expected = {
    schema: 'release-advertisement-snapshot/v1',
    commit: protocol.commit,
    tree: protocol.tree,
    protocolSha256: protocol.sha256,
    ...snapshotDefinition(directory, protocol),
    publicationAuthority: false
  };
  const actual = JSON.parse(readFileSync(join(directory, 'release-advertisement.json')));
  assert.deepEqual(actual, expected, 'source-qualified advertisement differs');
  return actual;
}
export function validateDigestCoverage(definition, version, checksumBytes, assets) {
  validateDefinition(definition);
  assert.match(version, /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
  assert.ok(Buffer.isBuffer(checksumBytes) && checksumBytes.length > 0 && checksumBytes.length <= 1024 * 1024);
  assert.ok(Array.isArray(assets) && assets.length <= 512);
  const metadata = new Map();
  for (const asset of assets) {
    assert.equal(typeof asset.name, 'string');
    assert.match(asset.name, /^[A-Za-z0-9_.-]+$/u);
    assert.ok(!metadata.has(asset.name), 'duplicate published asset');
    assert.match(asset.digest, /^sha256:[a-f0-9]{64}$/u);
    assert.ok(Number.isSafeInteger(asset.size) && asset.size > 0);
    metadata.set(asset.name, asset);
  }
  const checksums = new Map();
  for (const line of checksumBytes.toString('utf8').trim().split(/\r?\n/u)) {
    const match = /^([a-f0-9]{64})  ([A-Za-z0-9_.-]+)$/u.exec(line);
    assert.ok(match, 'unsupported consolidated checksum record');
    assert.ok(!checksums.has(match[2]), 'duplicate checksum member');
    checksums.set(match[2], match[1]);
  }
  const expected = [...definition.targets.native.map(row => 'formal-ai-native-' + row.target + '-' + version + '.json'),
     ...definition.targets.desktopAssets.map(row => row.template.replace('$' + '{version}',
     version)),
     ...definition.targets.cli.map(row => row.template),
     'formal-ai-native-source-' + version + '.json',
     'formal-ai-native-protocol-' + version + '.json',
     ...definition.targets.desktop.filter(row => row.ebflag === '--mac').map(row => 'formal-ai-signing-' + row.label + '-' + version + '.json'),
     'formal-ai-worker-' + version + '.wasm',
     'formal-ai-wasm-' + version + '.json',
     definition.targets.extensions ? definition.targets.extensions.template.replace('$' + '{version}',
     version) : 'formal-ai-vscode-' + version + '.vsix',
     definition.targets.browser.package.replace(/^@/u,
     '').replace('/',
     '-') + '-' + version + '.tgz'];
  assert.equal(new Set(expected).size, expected.length, 'duplicate source-advertised asset');
  const missing = [];
  for (const name of expected) {
    if (!metadata.has(name) || !checksums.has(name)) {
      missing.push(name);
      continue;
    }
    assert.equal(metadata.get(name).digest, 'sha256:' + checksums.get(name), 'published API digest disagrees with checksum: ' + name);
  }
  return {
    state: missing.length ? 'MissingAdvertisedAssets' : 'DigestCoverageOnly',
    expected,
    missing,
    complete: false,
    attestationAuthority: 'Unknown',
    nativeExecutionAuthority: 'Unknown',
    registryCoverage: 'Unknown',
    npmConfiguration: 'Unknown',
    publicationAuthority: false
  };
}
export function verifyAdvertisementAttestation(file, repository, protocolCommit) {
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(protocolCommit, /^[a-f0-9]{40}$/u);
  const stat = lstatSync(file);
  assert.ok(stat.isFile() && !stat.isSymbolicLink());
  assert.ok(stat.size > 0 && stat.size <= 1024 * 1024, 'bounded data-only advertisement required');
  const canonical = realpathSync(file);
  const bytes = readFileSync(canonical),
    record = JSON.parse(new TextDecoder('utf8', {
      fatal: true
    }).decode(bytes));
  assert.ok(bytes.length > 0 && bytes.length <= 1024 * 1024);
  assert.equal(record.schema, 'release-advertisement-snapshot/v1');
  assert.equal(record.commit, protocolCommit);
  validateDefinition(record.definition);
  validateSnapshotFiles(record.definition, record.sourceFiles);
  const result = spawnSync('gh', ['attestation', 'verify', canonical, '--repo', repository, '--signer-workflow', repository + '/.github/workflows/desktop-release.yml', '--signer-digest', protocolCommit], {
    encoding: 'utf8',
    timeout: 30000,
    maxBuffer: 1024 * 1024
  });
  if (result.error) throw result.error;
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(digest(readFileSync(canonical)), digest(bytes), 'advertisement changed during attestation verification');
  return {
    schema: 'verified-advertisement-data-attestation/v1',
    sha256: digest(bytes),
    protocolCommit,
    repository,
    stdout: result.stdout,
    stderr: result.stderr,
    publicationAuthority: false,
    nativeExecutionAuthority: 'Unknown'
  };
}
if (process.argv[1] && existsSync(process.argv[1]) && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [operation, root, directory, revision, digest, output] = process.argv.slice(2);
  assert.ok(operation === 'capture' && process.argv.length === 6 || operation === 'verify' && [7, 8].includes(process.argv.length), 'exact capture/verify selector required');
  const result = operation === 'capture' ? captureAdvertisement(root, directory, revision) : verifyAdvertisement(root, directory, revision, digest);
  if (output) writeFileSync(output, JSON.stringify(result, null, 2) + '\n');
  if (operation === 'capture' && process.env.GITHUB_OUTPUT) writeFileSync(process.env.GITHUB_OUTPUT, 'protocol-sha256=' + result.protocolSha256 + '\n', {
    flag: 'a'
  });
  console.log(JSON.stringify(result));
}
