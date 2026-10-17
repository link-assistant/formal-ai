// Controlled metadata and declared verifier boundary; no real package or registry execution.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { consolidateChecksums, publishedMetadataResolver } from '../../../scripts/release-checksums.mjs';
import { verifyAuthoritativeTransfer } from '../../../scripts/release-extension-policy.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const row = (hash, name) => hash + '  ' + name + '\n';
const a = 'a'.repeat(64),
  b = 'b'.repeat(64);
test('consolidation orders names and collapses identical rows', () => {
  assert.equal(consolidateChecksums([Buffer.from(row(a, 'z.zip') + row(b, 'a.zip')), Buffer.from(row(a, 'z.zip'))], () => assert.fail()), row(b, 'a.zip') + row(a, 'z.zip'));
});
test('conflicting shared updater rows resolve to the published producer digest', () => {
  const names = [];
  const result = consolidateChecksums([Buffer.from(row(a, 'latest-mac.yml') + row(b, 'latest-mac.yml') + row(a, 'unique.zip'))], name => {
    names.push(name);
    return b;
  });
  assert.deepEqual(names, ['latest-mac.yml']);
  assert.equal(result, row(b, 'latest-mac.yml') + row(a, 'unique.zip'));
});
test('ordinary artifact conflicts and unknown updater bytes are refused', () => {
  for (const name of ['package.vsix', 'latest-other.yml']) {
    assert.throws(() => consolidateChecksums([Buffer.from(row(a, name) + row(b, name))], () => b));
  }
  assert.throws(() => consolidateChecksums([Buffer.from(row(a, 'latest.yml') + row(b, 'latest.yml'))], () => 'c'.repeat(64)));
});
test('invalid, traversing and over-budget fragment records are refused', () => {
  for (const bytes of [Buffer.alloc(0), Buffer.from('bad\n'), Buffer.from(row(a, '../bad')), Buffer.alloc(1024 * 1024 + 1, 120), Buffer.from([255])]) {
    assert.throws(() => consolidateChecksums([bytes], () => b));
  }
});
test('metadata resolver verifies exact bytes, size, state and provider digest', () => {
  const bytes = Buffer.from('CONTROLLED UPDATER TEXT\n');
  const asset = {
    id: 7,
    name: 'latest.yml',
    state: 'uploaded',
    size: bytes.length,
    digest: 'sha256:' + sha(bytes)
  };
  const release = {
    tag_name: 'v1.2.3',
    draft: false,
    prerelease: false,
    assets: [asset]
  };
  const resolver = (changed, payload = bytes) => publishedMetadataResolver({
    REPO: 'owner/repository',
    TAG: 'v1.2.3'
  }, args => args.includes('Accept: application/octet-stream') ? payload : Buffer.from(JSON.stringify(changed)));
  assert.equal(resolver(release)('latest.yml'), sha(bytes));
  for (const mutate of [r => r.draft = true, r => r.tag_name = 'v2.0.0', r => r.assets.push(r.assets[0]), r => r.assets[0].state = 'new', r => r.assets[0].size++, r => r.assets[0].digest = 'sha256:' + a]) {
    const changed = structuredClone(release);
    mutate(changed);
    assert.throws(() => resolver(changed)('latest.yml'));
  }
  assert.throws(() => resolver(release, Buffer.from('changed'))('latest.yml'));
});
function fixture() {
  const expected = {
    repository: 'owner/repository',
    tag: 'v1.2.3',
    sourceCommit: 'a'.repeat(40),
    sourceTree: 'b'.repeat(40),
    protocolCommit: 'c'.repeat(40),
    runId: 11,
    attempt: 2,
    consumerCheckJobId: 44,
    consumerRunId: 10,
    consumerAttempt: 1
  };
  const name = 'formal-ai-vscode-1.2.3.vsix',
    artifactBytes = Buffer.from('CONTROLLED NOT REAL VSIX');
  const invocation = 'https://github.com/owner/repository/actions/runs/11/attempts/2';
  const evidence = {
    run: {
      id: 11,
      run_attempt: 2,
      repository: {
        full_name: expected.repository
      },
      head_repository: {
        full_name: expected.repository
      },
      head_branch: 'main',
      path: '.github/workflows/desktop-release.yml',
      event: 'workflow_run',
      status: 'completed',
      conclusion: 'success',
      head_sha: expected.protocolCommit
    },
    jobs: ['Package VS Code extension (.vsix)', 'Publish SHA256SUMS.txt + provenance'].map((name, i) => ({
      name,
      id: 20 + i,
      run_id: 11,
      run_attempt: 2,
      status: 'completed',
      conclusion: 'success'
    })),
    consumerChecks: {
      id: 44,
      run_id: 10,
      run_attempt: 1,
      status: 'completed',
      conclusion: 'success'
    },
    release: {
      tag_name: expected.tag,
      draft: false,
      prerelease: false,
      assets: [{
        id: 33,
        name,
        state: 'uploaded',
        size: artifactBytes.length,
        digest: 'sha256:' + sha(artifactBytes)
      }]
    },
    tagCommit: {
      sha: expected.sourceCommit,
      commit: {
        tree: {
          sha: expected.sourceTree
        }
      }
    },
    sourceBytes: Buffer.from(JSON.stringify({
      version: 1,
      release_tag: expected.tag,
      source_commit: expected.sourceCommit,
      source_tree: expected.sourceTree,
      producer_run: '11'
    })),
    artifactBytes,
    checksumBytes: Buffer.from(row(sha(artifactBytes), name))
  };
  const verify = request => [{
    verificationResult: {
      signature: {
        certificate: {
          runInvocationURI: invocation,
          buildSignerURI: 'https://github.com/owner/repository/.github/workflows/desktop-release.yml@refs/heads/main',
          buildSignerDigest: expected.protocolCommit,
          sourceRepositoryURI: 'https://github.com/owner/repository',
          issuer: 'https://token.actions.githubusercontent.com',
          runnerEnvironment: 'github-hosted'
        }
      },
      statement: {
        predicateType: 'https://slsa.dev/provenance/v1',
        predicate: {
          runDetails: {
            metadata: {
              invocationId: invocation
            }
          }
        },
        subject: [{
          name: request.name,
          digest: {
            sha256: sha(request.bytes)
          }
        }]
      }
    }
  }];
  return {
    expected,
    evidence,
    verify,
    name
  };
}
test('VSIX transfer verifies its unique checksum despite unrelated updater conflicts', () => {
  const f = fixture();
  f.evidence.checksumBytes = Buffer.concat([f.evidence.checksumBytes, Buffer.from(row(a, 'latest.yml') + row(b, 'latest.yml'))]);
  assert.equal(verifyAuthoritativeTransfer(f.expected, f.evidence, f.verify).sha256, sha(f.evidence.artifactBytes));
});
test('VSIX duplicate checksums, including identical duplicates, remain ambiguous', () => {
  for (const hash of [a, sha(fixture().evidence.artifactBytes)]) {
    const f = fixture();
    f.evidence.checksumBytes = Buffer.concat([f.evidence.checksumBytes, Buffer.from(row(hash, f.name))]);
    assert.throws(() => verifyAuthoritativeTransfer(f.expected, f.evidence, f.verify));
  }
});
test('missing checksum, mismatched digest, malformed unrelated rows and signature drift still refuse', () => {
  for (const mutate of [f => f.evidence.checksumBytes = Buffer.from(row(a,
   'other.zip')),
   f => f.evidence.checksumBytes = Buffer.from(row(a,
   f.name)),
   f => f.evidence.checksumBytes = Buffer.concat([f.evidence.checksumBytes,
   Buffer.from('malformed\n')]),
   f => f.evidence.release.assets[0].digest = 'sha256:' + a,
   f => f.evidence.sourceBytes = Buffer.from('{}'),
   f => f.evidence.run.head_sha = 'd'.repeat(40),
   f => f.verify = () => []]) {
    const f = fixture();
    mutate(f);
    assert.throws(() => verifyAuthoritativeTransfer(f.expected, f.evidence, f.verify));
  }
});
test('standard binary checksum marker is normalized to text output', () => {
  assert.equal(consolidateChecksums([Buffer.from(a + ' *windows.zip\n')], () => assert.fail()), row(a, 'windows.zip'));
});
import { readFileSync } from 'node:fs';
test('Desktop finalizer includes the independently public-downloaded engine in its existing signed manifest', () => {
  const script = readFileSync(new URL('../../../scripts/release-checksums.mjs', import.meta.url), 'utf8');
  assert.match(script, /await\s+verifyPublicBrowser/);
  assert.match(script, /mode:\s*'checksum'/);
  assert.match(script, /BROWSER_ENGINE_PROTOCOL_SHA256/);
  assert.ok(script.indexOf('engine-checksum.partial') < script.lastIndexOf('return consolidateChecksums'));
  const workflow = readFileSync(new URL('../../../.github/workflows/desktop-release.yml', import.meta.url), 'utf8');
  assert.match(workflow, /Capture the independently pinned engine protocol/);
  assert.match(workflow, /BROWSER_ENGINE_PROTOCOL_SHA256:.*steps.engine-protocol.outputs.protocol-sha256/);
  assert.match(workflow, /Attest consolidated digest and provenance data/);
});
test('engine checksum is consolidated exactly and conflicting engine bytes are never reconciled as updater metadata', () => {
  const name = 'link-assistant-formal-ai-engine-1.2.3.tgz';
  assert.equal(consolidateChecksums([Buffer.from(row(a, name)), Buffer.from(row(b, 'native.zip'))], () => assert.fail()), row(a, name) + row(b, 'native.zip'));
  assert.throws(() => consolidateChecksums([Buffer.from(row(a, name)), Buffer.from(row(b, name))], () => b));
});
