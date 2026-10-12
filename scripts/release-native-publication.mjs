import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { createHash } from 'node:crypto';
import { spawnSync, execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { productionProducerProfile, observeProductionProducer, joinProductionProducerEvidence } from './lib/release-completion-producer.mjs';
import { collectReleaseEvidence, PROTOCOL_HELPERS, SIGNING_LABELS } from './native-release-evidence.mjs';
import { NATIVE_TARGETS } from './native-release-artifact.mjs';
const profiles = new WeakMap(),
  proofs = new WeakSet();
const ownPath = 'scripts/release-native-publication.mjs';
const maximum = 1024 * 1024;
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const sourceBytes = fs.readFileSync(fileURLToPath(import.meta.url));
const loadedHelpers = new Map(PROTOCOL_HELPERS.map(name => [name, sha256(fs.readFileSync(new URL('./' + name, import.meta.url)))]));
const targetRegistry = JSON.stringify(NATIVE_TARGETS);
function stableSources(profile) {
  assert.equal(JSON.stringify(NATIVE_TARGETS), targetRegistry);
  assert.equal(sha256(fs.readFileSync(path.join(profile.root, ownPath))), sha256(sourceBytes));
  for (const [name, digest] of loadedHelpers) {
    const file = path.join(profile.root, 'scripts', name);
    assert.equal(fs.lstatSync(file).isSymbolicLink(), false);
    assert.equal(sha256(fs.readFileSync(file)), digest);
  }
}
function ordinarySource(root, revision, name) {
  const options = {
    cwd: root,
    timeout: 10000,
    maxBuffer: maximum
  };
  const entry = execFileSync('git', ['ls-tree', '-z', revision, '--', name], options).toString('utf8');
  assert.match(entry, /^100(?:644|755) blob [a-f0-9]{40}\t[^\0]+\0$/u);
  assert.equal(entry.slice(entry.indexOf('\t') + 1, -1), name);
  return execFileSync('git', ['show', revision + ':' + name], options);
}
export function nativePublicationProfile(root, protocolCommit, sourceCommit, sourceTree, version) {
  root = fs.realpathSync(root);
  for (const value of [protocolCommit, sourceCommit, sourceTree]) assert.match(value, /^[a-f0-9]{40}$/u);
  assert.match(version, /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
  assert.deepEqual(ordinarySource(root, protocolCommit, ownPath), sourceBytes);
  assert.equal(fs.lstatSync(path.join(root, ownPath)).isSymbolicLink(), false);
  assert.equal(sha256(fs.readFileSync(path.join(root, ownPath))), sha256(sourceBytes));
  const producer = productionProducerProfile(root, protocolCommit, '.github/workflows/desktop-release.yml', 'finalize');
  const helperBytes = {};
  for (const name of PROTOCOL_HELPERS) {
    const bytes = ordinarySource(root, protocolCommit, 'scripts/' + name);
    assert.deepEqual(fs.readFileSync(path.join(root, 'scripts', name)), bytes);
    assert.equal(fs.lstatSync(path.join(root, 'scripts', name)).isSymbolicLink(), false);
    assert.equal(sha256(bytes), loadedHelpers.get(name));
    helperBytes[name] = bytes;
  }
  const profile = Object.freeze({
    root,
    repository: producer.repository,
    producer,
    protocolCommit,
    sourceCommit,
    sourceTree,
    version,
    tag: 'v' + version,
    helperSha256: sha256(sourceBytes)
  });
  profiles.set(profile, {
    helperBytes
  });
  stableSources(profile);
  return profile;
}
function profileOnly(profile) {
  assert.ok(profiles.has(profile), 'unknown or copied native publication profile');
}
/** Exact schemas and digests are necessary but fixture inputs never issue trusted proof. */
export function validateNativePublicationData(inputs, binding, assets) {
  assert.ok(inputs && Array.isArray(inputs.receipts) && Array.isArray(inputs.signing));
  assert.ok(Array.isArray(assets) && assets.length <= 512);
  assert.ok(inputs.helperBytes && PROTOCOL_HELPERS.every(name => Buffer.isBuffer(inputs.helperBytes[name]) && inputs.helperBytes[name].length <= maximum));
  const names = new Set(),
    ids = new Set();
  for (const asset of assets) {
    assert.ok(Number.isSafeInteger(asset.id) && asset.id > 0);
    assert.equal(ids.has(asset.id), false);
    ids.add(asset.id);
    assert.match(asset.name, /^[A-Za-z0-9_.-]+$/u);
    assert.equal(names.has(asset.name), false);
    names.add(asset.name);
    assert.equal(asset.state, 'uploaded');
    assert.ok(Number.isSafeInteger(asset.size) && asset.size > 0);
    assert.match(asset.digest, /^sha256:[a-f0-9]{64}$/u);
  }
  for (const entry of inputs.receipts) {
    const record = JSON.parse(entry.bytes.toString('utf8'));
    assert.equal(record.producer.job, 'native');
    assert.equal(record.producer.attempt, String(binding.attempt));
  }
  const result = collectReleaseEvidence({
    ...inputs,
    bindings: binding
  });
  assert.equal(result.complete, true, JSON.stringify(result.failures));
  for (const asset of result.assets) {
    const metadata = assets.filter(row => row.name === asset.name);
    assert.equal(metadata.length, 1, 'missing or ambiguous published data asset');
    assert.equal(metadata[0].size, asset.bytes.length);
    assert.equal(metadata[0].digest, 'sha256:' + asset.sha256);
    assert.equal(metadata[0].content_type, 'application/json');
  }
  return Object.freeze({
    state: 'StructuralNativeDataOnly',
    authenticated: false,
    publicationComplete: false,
    nativeTargets: inputs.receipts.length,
    signingModes: Object.freeze(result.signingModes.map(Object.freeze)),
    dataNames: Object.freeze(result.assets.map(asset => asset.name))
  });
}
/** Fixed GET and attestation transport; accepts no callback, boolean proof or executable payload. */
export function observeNativePublication(profile, runId, runAttempt) {
  profileOnly(profile);
  stableSources(profile);
  for (const id of [runId, runAttempt]) assert.ok(Number.isSafeInteger(id) && id > 0);
  const deadline = Date.now() + 90000,
    raw = [];
  let total = 0;
  const execute = args => {
    assert.ok(Date.now() < deadline, 'shared data observation deadline exceeded');
    const result = spawnSync('gh', args, {
      timeout: Math.min(10000, deadline - Date.now()),
      maxBuffer: maximum,
      env: {
        ...process.env,
        GH_HOST: 'github.com'
      }
    });
    const stdout = result.stdout || Buffer.alloc(0),
      stderr = result.stderr || Buffer.alloc(0);
    total += stdout.length + stderr.length;
    assert.ok(total <= 8 * maximum, 'bounded original data observations exceeded');
    raw.push(Object.freeze({
      args: Object.freeze([...args]),
      stdoutBase64: stdout.toString('base64'),
      stderrBase64: stderr.toString('base64'),
      status: result.status,
      signal: result.signal
    }));
    assert.equal(result.status, 0, 'data observation unavailable, never target absence');
    assert.equal(result.signal, null);
    assert.equal(result.error, undefined);
    return stdout;
  };
  const prefix = 'repos/' + profile.repository + '/';
  const api = endpoint => JSON.parse(execute(['api', '--hostname', 'github.com', prefix + endpoint]).toString('utf8'));
  let directory;
  try {
    const producerProof = observeProductionProducer(profile.producer, runId, runAttempt);
    joinProductionProducerEvidence([profile.producer], [producerProof]);
    assert.ok(Date.now() < deadline, 'producer observation exhausted shared deadline');
    const repository = api('');
    assert.equal(repository.full_name, profile.repository);
    assert.equal(repository.default_branch, 'main');
    const main = api('commits/main'),
      tag = api('commits/' + profile.tag);
    assert.equal(tag.sha, profile.sourceCommit);
    assert.equal(tag.commit.tree.sha, profile.sourceTree);
    execFileSync('git', ['merge-base', '--is-ancestor', profile.sourceCommit, main.sha], {
      cwd: profile.root,
      timeout: Math.min(10000, deadline - Date.now()),
      maxBuffer: maximum
    });
    const release = api('releases/tags/' + profile.tag);
    assert.equal(release.tag_name, profile.tag);
    assert.equal(release.draft, false);
    assert.equal(release.prerelease, false);
    assert.ok(Number.isSafeInteger(release.id) && release.id > 0);
    assert.equal(release.url, 'https://api.github.com/' + prefix + 'releases/' + release.id);
    for (const value of [release.published_at, release.updated_at]) {
      assert.match(value, /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/u);
      const time = Date.parse(value);
      assert.ok(Number.isFinite(time) && time <= Date.now());
      assert.equal(new Date(time).toISOString().replace('.000Z', 'Z'), value);
    }
    const assets = [];
    let complete = false;
    for (let page = 1; page <= 6; page++) {
      const rows = api('releases/' + release.id + '/assets?per_page=100&page=' + page);
      assert.ok(Array.isArray(rows) && rows.length <= 100);
      assets.push(...rows);
      assert.ok(assets.length <= 512);
      if (rows.length < 100) {
        complete = true;
        break;
      }
    }
    assert.equal(complete, true, 'truncated published asset enumeration');
    assert.equal(new Set(assets.map(asset => asset.name)).size, assets.length);
    assert.equal(new Set(assets.map(asset => asset.id)).size, assets.length);
    for (const asset of assets) assert.ok(Number.isSafeInteger(asset.id) && asset.id > 0);
    const names = [...Object.keys(NATIVE_TARGETS).map(target => 'formal-ai-native-' + target + '-' + profile.version + '.json'), 'formal-ai-native-source-' + profile.version + '.json', 'formal-ai-native-protocol-' + profile.version + '.json', ...SIGNING_LABELS.map(label => 'formal-ai-signing-' + label + '-' + profile.version + '.json')];
    const missing = names.filter(name => !assets.some(asset => asset.name === name));
    if (missing.length) {
      const repeated = api('releases/tags/' + profile.tag);
      assert.equal(repeated.id, release.id);
      assert.equal(repeated.updated_at, release.updated_at);
      stableSources(profile);
      return Object.freeze({
        state: 'KnownMissingNativeDataAssets',
        missing: Object.freeze(missing),
        authenticatedMetadata: true,
        publicationComplete: false,
        automaticRepairAuthorized: false,
        raw: Object.freeze(raw)
      });
    }
    directory = fs.mkdtempSync(path.join(tmpdir(), 'native-publication-data-'));
    const data = {};
    for (const name of names) {
      const asset = assets.find(asset => asset.name === name);
      assert.equal(asset.content_type, 'application/json');
      assert.equal(asset.state, 'uploaded');
      assert.ok(Number.isSafeInteger(asset.size) && asset.size > 0 && asset.size <= maximum);
      assert.match(asset.digest, /^sha256:[a-f0-9]{64}$/u);
      const bytes = execute(['api', '--hostname', 'github.com', prefix + 'releases/assets/' + asset.id, '-H', 'Accept: application/octet-stream']);
      assert.equal(bytes.length, asset.size);
      assert.equal('sha256:' + sha256(bytes), asset.digest);
      JSON.parse(new TextDecoder('utf-8', {
        fatal: true
      }).decode(bytes));
      const file = path.join(directory, name);
      fs.writeFileSync(file, bytes, {
        flag: 'wx'
      });
      execute(['attestation', 'verify', file, '--repo', profile.repository, '--signer-workflow', profile.repository + '/.github/workflows/desktop-release.yml', '--signer-digest', profile.protocolCommit]);
      assert.equal(sha256(fs.readFileSync(file)), sha256(bytes));
      data[name] = bytes;
    }
    const source = JSON.parse(data['formal-ai-native-source-' + profile.version + '.json']);
    assert.equal(source.producer_run, String(runId));
    const binding = {
      sourceSha256: sha256(data['formal-ai-native-source-' + profile.version + '.json']),
      sourceCommit: profile.sourceCommit,
      sourceTree: profile.sourceTree,
      version: profile.version,
      run: runId,
      attempt: runAttempt,
      compilerRelease: source.compiler_release,
      compilerCommit: source.compiler_commit,
      protocolSha256: sha256(data['formal-ai-native-protocol-' + profile.version + '.json']),
      protocolCommit: profile.protocolCommit,
      protocolTree: profile.producer.sourceTree
    };
    const structural = validateNativePublicationData({
      sourceBytes: data['formal-ai-native-source-' + profile.version + '.json'],
      protocolBytes: data['formal-ai-native-protocol-' + profile.version + '.json'],
      helperBytes: profiles.get(profile).helperBytes,
      receipts: Object.keys(NATIVE_TARGETS).map(target => ({
        target,
        bytes: data['formal-ai-native-' + target + '-' + profile.version + '.json']
      })),
      signing: SIGNING_LABELS.map(label => ({
        label,
        bytes: data['formal-ai-signing-' + label + '-' + profile.version + '.json']
      }))
    }, binding, assets);
    const repeated = api('releases/tags/' + profile.tag);
    assert.equal(repeated.id, release.id);
    assert.equal(repeated.updated_at, release.updated_at);
    stableSources(profile);
    const proof = Object.freeze({
      ...structural,
      state: 'PublishedNativeDataAttested',
      authenticated: true,
      repository: profile.repository,
      protocolCommit: profile.protocolCommit,
      sourceCommit: profile.sourceCommit,
      sourceTree: profile.sourceTree,
      version: profile.version,
      runId,
      runAttempt,
      raw: Object.freeze(raw),
      nativeRuntimeAuthority: 'SourceProducerReported',
      signerAuthority: 'RequiresPerPackagePolicy',
      desktopPackageEvidence: 'Missing',
      cliArchiveEvidence: 'Missing',
      registryEvidence: 'Missing',
      writerLease: 'Unknown'
    });
    proofs.add(proof);
    return proof;
  } catch (error) {
    return Object.freeze({
      state: 'Unavailable',
      authenticated: false,
      publicationComplete: false,
      reason: error.message,
      raw: Object.freeze(raw)
    });
  } finally {
    if (directory) fs.rmSync(directory, {
      recursive: true,
      force: true
    });
  }
}
export function consumeNativePublication(profile, proof) {
  profileOnly(profile);
  stableSources(profile);
  assert.ok(proofs.has(proof), 'unknown, copied or fixture-only native data proof');
  for (const key of ['repository', 'protocolCommit', 'sourceCommit', 'sourceTree', 'version']) assert.equal(proof[key], profile[key]);
  return Object.freeze({
    state: proof.state,
    authenticated: true,
    nativeTargets: proof.nativeTargets,
    signingModes: proof.signingModes,
    publicationComplete: false,
    writerLease: 'Unknown',
    requiredRemaining: ['native runtime producer namespace', 'Desktop packages', 'CLI archives', 'browser package', 'crate registry checksum/runtime', 'container platform/runtime digests', 'configured extension registries']
  });
}
