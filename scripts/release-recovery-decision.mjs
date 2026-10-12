import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { captureAdvertisement, verifyAdvertisement, validateDigestCoverage } from './release-advertisement.mjs';

const profiles = new WeakSet(), observations = new WeakSet();
function freezeEvidence(value) {
  if (value && typeof value === 'object') {
    for (const child of Object.values(value)) freezeEvidence(child);
    Object.freeze(value);
  }
  return value;
}
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const ownPath = 'scripts/release-recovery-decision.mjs';
const ownBytes = fs.readFileSync(fileURLToPath(import.meta.url));
const commit = value => assert.match(value, /^[a-f0-9]{40}$/u);
function git(root, args) {
  const result = spawnSync('git', ['-C', root, ...args], {
    timeout: 10000, maxBuffer: 16 * 1024 * 1024, encoding: 'utf8'
  });
  assert.equal(result.status, 0);
  assert.equal(result.signal, null);
  assert.equal(result.error, undefined);
  return result.stdout;
}
function ordinarySource(root, revision, name) {
  const entry = git(root, ['ls-tree', revision, '--', name]).trimEnd();
  assert.match(entry, /^100(?:644|755) blob [a-f0-9]{40}\t/u);
  assert.equal(entry.slice(entry.indexOf('\t') + 1), name);
  return git(root, ['show', revision + ':' + name]);
}
export function structuralRecoveryPlan(definition, version, assets) {
  assert.ok(Array.isArray(assets) && assets.length <= 512);
  const safe = assets.map(asset => {
    assert.ok(Number.isSafeInteger(asset.id) && asset.id > 0);
    assert.match(asset.name, /^[A-Za-z0-9_.-]+$/u);
    assert.equal(asset.state, 'uploaded');
    assert.ok(Number.isSafeInteger(asset.size) && asset.size > 0);
    if (asset.digest !== null) assert.match(asset.digest, /^sha256:[a-f0-9]{64}$/u);
    return asset;
  });
  assert.equal(new Set(safe.map(asset => asset.id)).size, safe.length);
  assert.equal(new Set(safe.map(asset => asset.name)).size, safe.length);
  // Only the maintained source contract derives the advertised member set.
  // This sentinel derives names; it is not an observed or accepted checksum.
  const names = validateDigestCoverage(definition, version,
    Buffer.from('0'.repeat(64) + '  source-contract-name-derivation'), []).expected;
  const present = new Set(safe.map(asset => asset.name));
  return freezeEvidence({
    version, expected: names, potentiallyMissing: names.filter(name => !present.has(name)),
    publicationComplete: false, authenticated: false,
    registryEvidence: 'Unavailable', runtimeEvidence: 'Unavailable',
    signerEvidence: 'Unavailable', writerLease: 'Unknown'
  });
}
export function recoveryProfile(root, directory, protocolCommit, protocolSha256, sourceCommit, version) {
  root = fs.realpathSync(root);
  commit(protocolCommit);
  commit(sourceCommit);
  assert.match(protocolSha256, /^[a-f0-9]{64}$/u);
  assert.match(version, /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
  const remote = git(root, ['remote', 'get-url', 'origin']).trimEnd();
  const match = /^(?:https:\/\/github\.com\/|git@github\.com:)([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+?)(?:\.git)?$/u.exec(remote);
  assert.ok(match, 'credential-free GitHub repository origin required');
  const repository = match[1];
  const helper = ordinarySource(root, protocolCommit, ownPath);
  assert.equal(helper, ownBytes.toString('utf8'));
  assert.equal(fs.lstatSync(path.join(root, ownPath)).isSymbolicLink(), false);
  assert.equal(sha256(fs.readFileSync(path.join(root, ownPath))), sha256(ownBytes));
  for (const name of ['scripts/release-advertisement.mjs', 'scripts/release-protocol.mjs']) {
    const file = path.join(root, name);
    assert.equal(fs.lstatSync(file).isSymbolicLink(), false);
    assert.equal(fs.readFileSync(file, 'utf8'), ordinarySource(root, protocolCommit, name));
  }
  const advertisement = verifyAdvertisement(root, directory, protocolCommit, protocolSha256);
  const cargo = ordinarySource(root, sourceCommit, 'rust/Cargo.toml');
  const sections = cargo.split(/^\[package\]\s*$/mu);
  assert.equal(sections.length, 2);
  const versions = [...sections[1].split(/^\[/mu)[0]
    .matchAll(/^\s*version\s*=\s*"([^"\r\n]+)"\s*$/gmu)];
  assert.equal(versions.length, 1);
  assert.equal(versions[0][1], version);
  const profile = freezeEvidence({
    root, directory, protocolCommit, protocolSha256, sourceCommit, version,
    repository, tag: 'v' + version, helperSha256: sha256(ownBytes),
    definition: advertisement.definition
  });
  profiles.add(profile);
  return profile;
}
function profileOnly(profile) {
  assert.ok(profiles.has(profile), 'unknown or copied source profile');
}
export function validateReleaseSnapshot(profile, release, assets) {
  profileOnly(profile);
  assert.ok(Number.isSafeInteger(release.id) && release.id > 0);
  assert.equal(release.tag_name, profile.tag);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert.ok(typeof release.published_at === 'string' && Number.isFinite(Date.parse(release.published_at)));
  assert.equal(release.url, 'https://api.github.com/repos/' + profile.repository + '/releases/' + release.id);
  return structuralRecoveryPlan(profile.definition, profile.version, assets);
}
export function observeReleaseRecovery(profile) {
  profileOnly(profile);
  const started = Date.now(), deadline = started + 90000;
  let total = 0;
  const raw = [];
  function api(endpoint) {
    assert.ok(endpoint.startsWith('repos/' + profile.repository + '/'));
    const remaining = deadline - Date.now();
    assert.ok(remaining > 0, 'shared observation deadline exceeded');
    const result = spawnSync('gh', ['api', '--hostname', 'github.com', endpoint], {
      encoding: 'utf8', timeout: Math.min(10000, remaining), maxBuffer: 1024 * 1024,
      env: { ...process.env, GH_HOST: 'github.com' }
    });
    const stdout = result.stdout || '', stderr = result.stderr || '';
    total += Buffer.byteLength(stdout) + Buffer.byteLength(stderr);
    assert.ok(total <= 6 * 1024 * 1024, 'bounded full observation metadata exceeded');
    raw.push({ endpoint, stdout, stderr, status: result.status, signal: result.signal });
    assert.equal(result.status, 0, 'API failure is Unavailable, never target absence');
    assert.equal(result.signal, null);
    assert.equal(result.error, undefined);
    return JSON.parse(stdout);
  }
  let result;
  try {
    const repository = api('repos/' + profile.repository + '/');
    assert.equal(repository.full_name, profile.repository);
    assert.match(repository.default_branch, /^[A-Za-z0-9_.\/-]+$/u);
    const main = api('repos/' + profile.repository + '/commits/' + encodeURIComponent(repository.default_branch));
    commit(main.sha);
    const tag = api('repos/' + profile.repository + '/commits/' + profile.tag);
    assert.equal(tag.sha, profile.sourceCommit);
    assert.equal(git(profile.root, ['rev-parse', profile.tag + '^{commit}']).trimEnd(), profile.sourceCommit);
    git(profile.root, ['merge-base', '--is-ancestor', profile.sourceCommit, main.sha]);
    const release = api('repos/' + profile.repository + '/releases/tags/' + profile.tag);
    const assets = [];
    let complete = false;
    for (let page = 1; page <= 6; page++) {
      const rows = api('repos/' + profile.repository + '/releases/' + release.id + '/assets?per_page=100&page=' + page);
      assert.ok(Array.isArray(rows) && rows.length <= 100);
      assets.push(...rows);
      assert.ok(assets.length <= 512, 'asset enumeration exceeds source cap');
      if (rows.length < 100) { complete = true; break; }
    }
    assert.equal(complete, true, 'truncated asset enumeration refused');
    const structural = validateReleaseSnapshot(profile, release, assets);
    const repeated = api('repos/' + profile.repository + '/releases/tags/' + profile.tag);
    assert.equal(repeated.id, release.id);
    assert.equal(repeated.updated_at, release.updated_at, 'publication changed during observation');
    verifyAdvertisement(profile.root, profile.directory, profile.protocolCommit, profile.protocolSha256);
    assert.equal(sha256(fs.readFileSync(path.join(profile.root, ownPath))), profile.helperSha256);
    result = freezeEvidence({
      ...structural, authenticated: true,
      assetState: structural.potentiallyMissing.length ? 'KnownMissing' : 'NamesPresentOnly',
      missingAssets: structural.potentiallyMissing,
      sourceCommit: profile.sourceCommit, protocolCommit: profile.protocolCommit,
      repository: profile.repository, raw
    });
  } catch (error) {
    result = freezeEvidence({
      state: 'Unavailable', reason: error.message, version: profile.version,
      publicationComplete: false, authenticated: false, raw
    });
  }
  observations.add(result);
  return result;
}
export function consumeRecoveryDecision(profile, observation) {
  profileOnly(profile);
  assert.ok(observations.has(observation), 'unknown, copied or structural-only observation');
  assert.equal(observation.version, profile.version);
  if (observation.authenticated) {
    assert.equal(observation.repository, profile.repository);
    assert.equal(observation.sourceCommit, profile.sourceCommit);
    assert.equal(observation.protocolCommit, profile.protocolCommit);
  }
  return freezeEvidence({
    schema: 'release-recovery-decision/v1', version: profile.version,
    state: 'UnavailableFullTargetEvidence', publicationComplete: false,
    shouldRelease: false, skipBump: true,
    sameVersionRepairProposal: observation.authenticated ? observation.missingAssets : [],
    automaticRepairAuthorized: false, writerLease: 'Unknown',
    unknown: ['crate-source-checksum', 'container-platform-digests', 'registry-policy',
      'native-runtime-attestations', 'desktop-runtime-signing', 'CLI-runtime',
      'browser-runtime', 'extension-registry-package-digests'],
    observationState: observation.state || observation.assetState
  });
}

// The ordinary staged caller selects the immutable protocol checkout, not caller JSON.
export function automaticRecoveryInputs(environment, version) {
  assert.match(version, /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
  assert.equal(environment.GITHUB_REF, 'refs/heads/main');
  assert.ok(['push', 'workflow_dispatch'].includes(environment.GITHUB_EVENT_NAME));
  commit(environment.GITHUB_SHA);
  assert.match(environment.GITHUB_REPOSITORY, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.equal(environment.GITHUB_WORKFLOW_REF,
    environment.GITHUB_REPOSITORY + '/.github/workflows/release.yml@refs/heads/main');
  assert.equal(environment.RELEASE_RECOVERY_PROTOCOL_ROOT, '_protocol');
  assert.equal(environment.RELEASE_RECOVERY_PROTOCOL_COMMIT, environment.GITHUB_SHA);
  return Object.freeze({protocolRoot: '_protocol', protocolCommit: environment.GITHUB_SHA, version});
}
export function prepareAutomaticRecovery(root, environment, version) {
  const selected = automaticRecoveryInputs(environment, version);
  root = fs.realpathSync(root);
  const protocolRoot = fs.realpathSync(path.join(root, selected.protocolRoot));
  assert.equal(git(protocolRoot, ['rev-parse', 'HEAD']).trimEnd(), selected.protocolCommit);
  assert.equal(ordinarySource(protocolRoot, selected.protocolCommit, ownPath), ownBytes.toString('utf8'));
  const tag = 'v' + version;
  const sourceCommit = git(root, ['rev-parse', tag + '^{commit}']).trimEnd();
  commit(sourceCommit);
  // A protocol snapshot is source-derived data, not an observation or publication proof.
  const directory = fs.mkdtempSync(path.join(root, '.release-recovery-'));
  const captured = captureAdvertisement(protocolRoot, directory, sourceCommit);
  const profile = recoveryProfile(protocolRoot, directory, sourceCommit,
    captured.protocolSha256, sourceCommit, version);
  assert.equal(profile.repository, environment.GITHUB_REPOSITORY);
  return profile;
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  if (process.argv[2] === 'automatic') {
    assert.equal(process.argv.length, 4);
    const profile = prepareAutomaticRecovery(process.cwd(), process.env, process.argv[3]);
    console.log(JSON.stringify(consumeRecoveryDecision(profile, observeReleaseRecovery(profile))));
    process.exit(0);
  }
  const [operation, root, directory, protocolCommit, protocolSha256, sourceCommit, version, ...extra] = process.argv.slice(2);
  assert.equal(operation, 'observe');
  assert.equal(extra.length, 0);
  const profile = recoveryProfile(root, directory, protocolCommit, protocolSha256, sourceCommit, version);
  console.log(JSON.stringify(consumeRecoveryDecision(profile, observeReleaseRecovery(profile))));
}
