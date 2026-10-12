// Controlled schema fixtures provide no authenticated producer or publication authority.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { expectedIdentity, NATIVE_TARGETS } from '../../../scripts/native-release-artifact.mjs';
import { bindMacPackageExecutable, MAC_PACKAGE_TARGETS, collectReleaseEvidence, PROTOCOL_HELPERS, sha256 } from '../../../scripts/native-release-evidence.mjs';
import { validateNativePublicationData, observeNativePublication, consumeNativePublication } from '../../../scripts/release-native-publication.mjs';
const encode = value => Buffer.from(JSON.stringify(value, null, 2) + '\n');
function fixture() {
  const source = {
    version: 1,
    source_commit: 'a'.repeat(40),
    source_tree: 'b'.repeat(40),
    compiler_commit: 'c'.repeat(40),
    compiler_release: '1.91.0',
    cargo_lock_sha256: 'd'.repeat(64),
    native_inputs_sha256: 'e'.repeat(64),
    package_version: '2.7.11',
    producer_run: '91',
    profile: 'release',
    requested_features: 'default',
    root_default_features: ['portable'],
    rust_flags: '-Dwarnings'
  };
  const sourceBytes = encode(source),
    helperBytes = Object.fromEntries(PROTOCOL_HELPERS.map(name => [name, Buffer.from('actual selected fixture helper bytes ' + name + '\n')]));
  const protocol = {
    version: 1,
    commit: 'f'.repeat(40),
    tree: '1'.repeat(40),
    files: PROTOCOL_HELPERS.map(name => ({
      name,
      bytes: helperBytes[name].length,
      sha256: sha256(helperBytes[name])
    }))
  };
  const protocolBytes = encode(protocol),
    bindings = {
      sourceSha256: sha256(sourceBytes),
      sourceCommit: source.source_commit,
      sourceTree: source.source_tree,
      version: source.package_version,
      run: source.producer_run,
      compilerRelease: source.compiler_release,
      compilerCommit: source.compiler_commit,
      protocolSha256: sha256(protocolBytes),
      protocolCommit: protocol.commit,
      protocolTree: protocol.tree
    };
  const receipts = Object.keys(NATIVE_TARGETS).map(target => {
    const identity = expectedIdentity(source, target, bindings.sourceSha256, bindings.run);
    return {
      target,
      bytes: encode({
        version: 1,
        identity,
        producer: {
          job: 'native',
          attempt: '1',
          compiler: 'rustc 1.91.0\nrelease: 1.91.0\ncommit-hash: ' + source.compiler_commit + '\nhost: ' + identity.host + '\n',
          environment: {
            RUSTFLAGS: '-Dwarnings'
          }
        },
        executable: {
          name: identity.platform === 'win32' ? 'formal-ai.exe' : 'formal-ai',
          bytes: 59,
          sha256: '2'.repeat(64)
        }
      })
    };
  });
  const observation = (stderr = '') => ({
    status: 0,
    stdout: '',
    stderr
  });
  const signing = ['macos-arm64', 'macos-x64'].map(label => ({
    label,
    bytes: encode({
      version: 1,
      label,
      source_selection_sha256: bindings.sourceSha256,
      source_commit: source.source_commit,
      package_version: source.package_version,
      producer_run: source.producer_run,
      mode: 'adhoc',
      package: {
        name: 'formal-ai-desktop-' + label + '-' + source.package_version + '.dmg',
        bytes: 123,
        sha256: '3'.repeat(64)
      },
      native_component: {
        bytes: 99,
        sha256: '4'.repeat(64)
      },
      observations: {
        verify: observation(),
        display: observation('Signature=adhoc\n'),
        assess: null,
        stapler: null
      }
    })
  }));
  for (const entry of signing) {
    const record = JSON.parse(entry.bytes),
      parent = receipts.find(item => item.target === MAC_PACKAGE_TARGETS[entry.label]);
    record.package_executable_receipt = bindMacPackageExecutable(parent.bytes, {
      source,
      sourceSha256: bindings.sourceSha256,
      label: entry.label,
      packageObservation: record.package,
      executable: record.native_component,
      startup: {
        arguments: ['--version'],
        status: 0,
        signal: null,
        complete: true,
        stdout: 'formal-ai ' + source.package_version + '\n',
        stderr: ''
      },
      signingObservations: record.observations
    });
    record.package_executable_sha256 = sha256(Buffer.from(JSON.stringify(record.package_executable_receipt) + '\n'));
    entry.bytes = encode(record);
  }
  return {
    source,
    protocol,
    sourceBytes,
    protocolBytes,
    helperBytes,
    bindings,
    receipts,
    signing
  };
}
function inputs() {
  const f = fixture();
  f.bindings.attempt = 1;
  const result = collectReleaseEvidence(f);
  const assets = result.assets.map((asset, index) => ({
    id: index + 1,
    name: asset.name,
    state: 'uploaded',
    size: asset.bytes.length,
    digest: 'sha256:' + asset.sha256,
    content_type: 'application/json'
  }));
  return {
    f,
    assets
  };
}
test('complete schema fixtures remain structural and cannot issue publication proof', () => {
  const {
    f,
    assets
  } = inputs();
  const result = validateNativePublicationData(f, f.bindings, assets);
  assert.equal(result.authenticated, false);
  assert.equal(result.publicationComplete, false);
  assert.equal(result.nativeTargets, 8);
  assert.throws(() => consumeNativePublication({}, result));
});
test('missing duplicate changed or foreign data assets refuse', () => {
  const {
    f,
    assets
  } = inputs();
  for (const mutate of [rows => rows.pop(), rows => rows[1].id = rows[0].id, rows => rows[1].name = rows[0].name, rows => rows[0].digest = 'sha256:' + '0'.repeat(64), rows => rows[0].size++, rows => rows[0].content_type = 'application/octet-stream', rows => rows[0].state = 'new']) {
    const rows = structuredClone(assets);
    mutate(rows);
    assert.throws(() => validateNativePublicationData(f, f.bindings, rows));
  }
});
test('independently selected helper bytes are required and tampering refuses', () => {
  const {
    f,
    assets
  } = inputs();
  assert.throws(() => validateNativePublicationData({
    ...f,
    helperBytes: undefined
  }, f.bindings, assets));
  assert.throws(() => validateNativePublicationData({
    ...f,
    helperBytes: {
      ...f.helperBytes,
      [PROTOCOL_HELPERS[0]]: Buffer.from('tampered')
    }
  }, f.bindings, assets));
});
test('every selected native producer namespace and attempt must match', () => {
  const {
    f,
    assets
  } = inputs();
  for (const change of [r => r.producer.job = 'foreign', r => r.producer.attempt = '2']) {
    const receipts = f.receipts.map(entry => ({
      ...entry,
      bytes: Buffer.from(entry.bytes)
    }));
    const value = JSON.parse(receipts[0].bytes);
    change(value);
    receipts[0].bytes = encode(value);
    assert.throws(() => validateNativePublicationData({
      ...f,
      receipts
    }, f.bindings, assets));
  }
});
test('unknown and copied profile or fixture proof refuse before fixed transport', () => {
  for (const profile of [{}, Object.freeze({
    repository: 'owner/repo',
    sourceCommit: 'a'.repeat(40)
  })]) assert.throws(() => observeNativePublication(profile, 1, 1), /unknown or copied/);
  assert.throws(() => consumeNativePublication({}, Object.freeze({
    authenticated: true,
    publicationComplete: true
  })));
});
