import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { authenticateBrowserProducer, releaseAsset, verifyPublicEngineBytes, engineReceiptChecksums } from '../../../scripts/browser-engine-public-download.mjs';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function fixture() {
  const expected = {
    repository: 'owner/repo',
    tag: 'v1.2.3',
    sourceCommit: 'a'.repeat(40),
    sourceTree: 'b'.repeat(40),
    protocolCommit: 'c'.repeat(40),
    protocolSha256: 'd'.repeat(64),
    producerRun: 41,
    producerAttempt: 1,
    producerJobId: 7
  };
  const bytes = Buffer.from('CONTROLLED NON-PRODUCT ARCHIVE BYTES');
  const asset = {
    id: 9,
    name: 'link-assistant-formal-ai-engine-1.2.3.tgz',
    state: 'uploaded',
    size: bytes.length,
    digest: 'sha256:' + digest(bytes),
    url: 'https://api.github.com/repos/owner/repo/releases/assets/9',
    browser_download_url: 'https://github.com/owner/repo/releases/download/v1.2.3/link-assistant-formal-ai-engine-1.2.3.tgz'
  };
  const release = {
    tag_name: expected.tag,
    draft: false,
    prerelease: false,
    html_url: 'https://github.com/owner/repo/releases/tag/v1.2.3',
    assets: [asset]
  };
  const producer = {
    repository: expected.repository,
    run: '41',
    attempt: '1',
    job: 'package',
    event: 'workflow_run',
    workflowCommit: expected.protocolCommit
  };
  const publication = {
    schema: 'release-browser-publication/v1',
    binding: {
      sourceCommit: expected.sourceCommit,
      sourceTree: expected.sourceTree,
      protocolCommit: expected.protocolCommit,
      protocolSha256: expected.protocolSha256,
      run: '41',
      attempt: '1',
      packageName: '@link-assistant/formal-ai-engine',
      version: '1.2.3'
    },
    archive: {
      name: asset.name,
      bytes: bytes.length,
      sha256: digest(bytes),
      integrity: 'sha512-' + createHash('sha512').update(bytes).digest('base64')
    },
    workerReceiptSha256: 'e'.repeat(64),
    npm: {
      status: 'skipped-unconfigured',
      required: false
    },
    publicationAuthority: false,
    producer
  };
  const metadata = {
    publication,
    tagCommit: {
      sha: expected.sourceCommit,
      commit: {
        tree: {
          sha: expected.sourceTree
        }
      }
    },
    run: {
      id: 41,
      run_attempt: 1,
      repository: {
        full_name: 'owner/repo'
      },
      head_repository: {
        full_name: 'owner/repo'
      },
      head_branch: 'main',
      head_sha: expected.protocolCommit,
      path: '.github/workflows/publish-engine.yml',
      event: 'workflow_run',
      status: 'completed',
      conclusion: 'success'
    },
    jobs: {
      total_count: 1,
      jobs: [{
        id: 7,
        name: 'package',
        run_id: 41,
        run_attempt: 1,
        status: 'completed',
        conclusion: 'success'
      }]
    }
  };
  return {
    expected,
    bytes,
    asset,
    release,
    producer,
    publication,
    metadata
  };
}
test('completed real producer metadata binds independent public bytes without granting publication authority', () => {
  const f = fixture();
  assert.deepEqual(authenticateBrowserProducer(f.expected, f.metadata, {
    job: 'public-browser'
  }), f.producer);
  assert.equal(releaseAsset(f.release, 'owner/repo', 'v1.2.3', f.asset.name, 1024), f.asset);
  assert.equal(verifyPublicEngineBytes(f.expected, f.publication, f.asset, f.bytes), digest(f.bytes) + '  ' + f.asset.name + '\n');
});
for (const [name,
   mutate] of [['source',
   f => f.metadata.publication.binding.sourceCommit = 'f'.repeat(40)],
   ['tree',
   f => f.metadata.tagCommit.commit.tree.sha = 'f'.repeat(40)],
   ['protocol',
   f => f.metadata.run.head_sha = 'f'.repeat(40)],
   ['protocol manifest',
   f => f.metadata.publication.binding.protocolSha256 = 'f'.repeat(64)],
   ['foreign repository',
   f => f.metadata.run.head_repository.full_name = 'evil/repo'],
   ['workflow',
   f => f.metadata.run.path = '.github/workflows/other.yml'],
   ['untrusted branch',
   f => f.metadata.run.head_branch = 'feature'],
   ['attempt',
   f => f.metadata.jobs.jobs[0].run_attempt = 2],
   ['job',
   f => f.metadata.jobs.jobs[0].id = 8],
   ['failed producer',
   f => f.metadata.jobs.jobs[0].conclusion = 'failure'],
   ['partial jobs',
   f => f.metadata.jobs.total_count = 2],
   ['pending producer',
   f => {
  f.metadata.run.status = 'in_progress';
  f.metadata.jobs.jobs[0].status = 'in_progress';
}], ['identity', f => f.metadata.publication.producer.job = 'public-browser']]) test('refuses ' + name, () => {
  const f = fixture();
  mutate(f);
  assert.throws(() => authenticateBrowserProducer(f.expected, f.metadata, {
    job: 'public-browser'
  }));
});
test('same-run producer allowance requires its actual run, attempt and package job identity', () => {
  const f = fixture();
  f.metadata.run.status = 'in_progress';
  f.metadata.jobs.jobs[0].status = 'in_progress';
  assert.deepEqual(authenticateBrowserProducer(f.expected, f.metadata, f.producer), f.producer);
  for (const change of [{
    job: 'consumer'
  }, {
    run: '42'
  }, {
    attempt: '2'
  }, {
    repository: 'other/repo'
  }]) assert.throws(() => authenticateBrowserProducer(f.expected, f.metadata, {
    ...f.producer,
    ...change
  }));
});
for (const [name,
   mutate] of [['wrong origin',
   f => f.asset.browser_download_url = 'https://evil.invalid/archive.tgz'],
   ['wrong API source',
   f => f.asset.url = 'https://api.github.com/repos/evil/repo/releases/assets/9'],
   ['traversing asset',
   f => f.asset.name = '../archive.tgz'],
   ['draft',
   f => f.release.draft = true],
   ['duplicate',
   f => f.release.assets.push({
  ...f.asset
})], ['oversize', f => f.asset.size = 64 * 1024 * 1024 + 1], ['missing digest', f => delete f.asset.digest]]) test('public metadata refuses ' + name, () => {
  const f = fixture();
  mutate(f);
  assert.throws(() => releaseAsset(f.release, 'owner/repo', 'v1.2.3', 'link-assistant-formal-ai-engine-1.2.3.tgz', 64 * 1024 * 1024));
});
test('digest, length, source receipt and truncation controls retain all independent byte checks', () => {
  for (const mutate of [f => f.bytes = f.bytes.subarray(1), f => f.asset.size++, f => f.asset.digest = 'sha256:' + 'f'.repeat(64), f => f.publication.archive.integrity = 'sha512-' + 'f'.repeat(88), f => f.publication.archive.sha256 = 'f'.repeat(64)]) {
    const f = fixture();
    mutate(f);
    assert.throws(() => verifyPublicEngineBytes(f.expected, f.publication, f.asset, f.bytes));
  }
});
test('consumer uses public anonymous transport, original protocol and engine receipt before unchanged real Worker smoke', () => {
  const source = readFileSync(new URL('../../../scripts/browser-engine-public-download.mjs', import.meta.url), 'utf8');
  for (const text of ['downloadPublicReleaseAsset', 'verifyOriginalProtocol', 'verifyReleaseProtocol', 'original engine receipt differs']) assert.ok(source.includes(text));
  assert.match(source, /producerBindings:\s*producer/);
  assert.match(source, /workerSmoke\s*=\s*await coldPackageSmoke/);
  assert.doesNotMatch(source, /GITHUB_JOB\s*=|requireNative:false|npm.*--ignore-scripts.*false/);
});
test('checksum fragment covers archive and both same-producer receipts; missing and altered receipts fail', () => {
  const {
      expected,
      metadata,
      asset,
      bytes
    } = fixture(),
    publication = metadata.publication;
  const protocol = Buffer.from(JSON.stringify({
    commit: expected.protocolCommit
  }));
  const worker = {
    schema: 'selected-source-wasm/v1',
    nativeCompilationEvidence: true,
    sourceCommit: expected.sourceCommit,
    sourceTree: expected.sourceTree,
    packageVersion: '1.2.3',
    identity: publication.producer
  };
  const workerBytes = Buffer.from(JSON.stringify(worker));
  publication.binding.protocolSha256 = digest(protocol);
  publication.workerReceiptSha256 = digest(workerBytes);
  const archive = verifyPublicEngineBytes(expected, publication, asset, bytes);
  const rows = archive + engineReceiptChecksums(expected, publication, publication.producer, protocol, workerBytes);
  assert.equal(rows.split('\n').filter(Boolean).length, 3);
  assert.ok(rows.includes(digest(workerBytes) + '  formal-ai-engine-worker-1.2.3.json\n'));
  assert.ok(rows.includes(digest(protocol) + '  formal-ai-engine-protocol-1.2.3.json\n'));
  for (const [p, w] of [[Buffer.alloc(0), workerBytes], [protocol, Buffer.alloc(0)], [Buffer.from('changed'), workerBytes], [protocol, Buffer.from('changed')]]) assert.throws(() => engineReceiptChecksums(expected, publication, publication.producer, p, w));
  for (const key of ['sourceCommit', 'sourceTree', 'packageVersion', 'identity']) {
    const bad = {
      ...worker,
      [key]: key === 'identity' ? {
        ...worker.identity,
        run: '99'
      } : 'wrong'
    };
    const b = Buffer.from(JSON.stringify(bad));
    assert.throws(() => engineReceiptChecksums(expected, {
      ...publication,
      workerReceiptSha256: digest(b)
    }, publication.producer, protocol, b));
  }
  const source = readFileSync(new URL('../../../scripts/browser-engine-public-download.mjs', import.meta.url), 'utf8');
  assert.match(source, /request.mode === 'checksum' \? null : request.receiptPath/);
  assert.match(source, /request.mode === 'checksum' \? null : request.protocolPath/);
  assert.equal((source.match(/const receiptAsset = releaseAsset/g) || []).length, 1);
});
