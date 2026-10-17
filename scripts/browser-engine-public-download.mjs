// Read-only public browser-package qualification. Provider metadata is data, not signing authority.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync, lstatSync, realpathSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import { createReadonlyGithub } from './release-extension-github.mjs';
const packageName = '@link-assistant/formal-ai-engine';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const revision = value => assert.match(value, /^[a-f0-9]{40}$/u);
const hash = value => assert.match(value, /^[a-f0-9]{64}$/u);
const positive = value => assert.match(String(value), /^[1-9][0-9]*$/u);
export function engineAssetName(tag) {
  assert.match(tag, /^v\d+\.\d+\.\d+$/u);
  return 'link-assistant-formal-ai-engine-' + tag.slice(1) + '.tgz';
}
export function releaseAsset(release, repository, tag, name, maximum) {
  assert.equal(release.tag_name, tag);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert.equal(release.html_url, 'https://github.com/' + repository + '/releases/tag/' + tag);
  assert.ok(Array.isArray(release.assets) && release.assets.length < 100);
  const found = release.assets.filter(asset => asset.name === name);
  assert.equal(found.length, 1, 'one exact public release asset required: ' + name);
  const asset = found[0];
  positive(asset.id);
  assert.equal(asset.state, 'uploaded');
  assert.ok(Number.isSafeInteger(asset.size) && asset.size > 0 && asset.size <= maximum);
  assert.match(asset.digest, /^sha256:[a-f0-9]{64}$/u);
  assert.equal(asset.browser_download_url, 'https://github.com/' + repository + '/releases/download/' + tag + '/' + name);
  assert.equal(asset.url, 'https://api.github.com/repos/' + repository + '/releases/assets/' + asset.id);
  return asset;
}
export function authenticateBrowserProducer(expected, {
  publication,
  run,
  jobs,
  tagCommit
}, consumer) {
  assert.match(expected.repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  engineAssetName(expected.tag);
  for (const key of ['sourceCommit', 'sourceTree', 'protocolCommit']) revision(expected[key]);
  hash(expected.protocolSha256);
  assert.equal(tagCommit.sha, expected.sourceCommit);
  assert.equal(tagCommit.commit.tree.sha, expected.sourceTree);
  assert.equal(publication.schema, 'release-browser-publication/v1');
  assert.equal(publication.publicationAuthority, false);
  const binding = publication.binding;
  positive(binding.run);
  positive(binding.attempt);
  assert.equal(binding.sourceCommit, expected.sourceCommit);
  assert.equal(binding.sourceTree, expected.sourceTree);
  assert.equal(binding.protocolCommit, expected.protocolCommit);
  assert.equal(binding.protocolSha256, expected.protocolSha256);
  assert.equal(binding.packageName, packageName);
  assert.equal(binding.version, expected.tag.slice(1));
  if (expected.producerRun !== undefined) assert.equal(String(binding.run), String(expected.producerRun));
  if (expected.producerAttempt !== undefined) assert.equal(String(binding.attempt), String(expected.producerAttempt));
  assert.equal(String(run.id), String(binding.run));
  assert.equal(String(run.run_attempt), String(binding.attempt));
  assert.equal(run.repository.full_name, expected.repository);
  assert.equal(run.head_repository.full_name, expected.repository);
  assert.equal(run.head_branch, 'main');
  assert.equal(run.head_sha, expected.protocolCommit);
  assert.equal(run.path, '.github/workflows/publish-engine.yml');
  assert.ok(['release', 'workflow_run', 'workflow_dispatch'].includes(run.event));
  assert.equal(jobs.total_count, jobs.jobs.length, 'complete producer job metadata required');
  const matches = jobs.jobs.filter(job => job.name === 'package');
  assert.equal(matches.length, 1);
  const job = matches[0];
  positive(job.id);
  assert.equal(String(job.run_id), String(run.id));
  assert.equal(String(job.run_attempt), String(run.run_attempt));
  if (expected.producerJobId !== undefined) assert.equal(String(job.id), String(expected.producerJobId));
  const sameProducer = consumer?.repository === expected.repository && String(consumer.run) === String(run.id) && String(consumer.attempt) === String(run.run_attempt) && consumer.job === 'package' && consumer.workflowCommit === expected.protocolCommit;
  if (sameProducer) {
    assert.ok(['in_progress', 'completed'].includes(run.status));
    assert.ok(['in_progress', 'completed'].includes(job.status));
    if (run.status === 'completed') assert.equal(run.conclusion, 'success');
    if (job.status === 'completed') assert.equal(job.conclusion, 'success');
  } else {
    assert.equal(run.status, 'completed');
    assert.equal(run.conclusion, 'success');
    assert.equal(job.status, 'completed');
    assert.equal(job.conclusion, 'success');
  }
  const identity = {
    repository: expected.repository,
    run: String(run.id),
    attempt: String(run.run_attempt),
    job: 'package',
    event: run.event,
    workflowCommit: expected.protocolCommit
  };
  assert.deepEqual(publication.producer, identity);
  hash(publication.workerReceiptSha256);
  assert.ok(['published', 'already-matching', 'skipped-unconfigured'].includes(publication.npm.status));
  assert.equal(publication.npm.required, publication.npm.status !== 'skipped-unconfigured');
  return identity;
}
export function verifyPublicEngineBytes(expected, publication, asset, bytes) {
  assert.ok(Buffer.isBuffer(bytes) && bytes.length > 0 && bytes.length <= 64 * 1024 * 1024);
  assert.equal(asset.name, engineAssetName(expected.tag));
  assert.equal(bytes.length, asset.size);
  assert.equal('sha256:' + sha(bytes), asset.digest);
  assert.deepEqual(publication.archive, {
    name: asset.name,
    bytes: bytes.length,
    sha256: sha(bytes),
    integrity: 'sha512-' + createHash('sha512').update(bytes).digest('base64')
  });
  return sha(bytes) + '  ' + asset.name + '\n';
}
export function engineReceiptChecksums(request, publication, producer, protocolBytes, workerBytes) {
  assert.ok(Buffer.isBuffer(protocolBytes) && protocolBytes.length > 0 && protocolBytes.length <= 1024 * 1024);
  assert.ok(Buffer.isBuffer(workerBytes) && workerBytes.length > 0 && workerBytes.length <= 1024 * 1024);
  assert.equal(sha(protocolBytes), publication.binding.protocolSha256, 'public protocol receipt differs');
  assert.equal(sha(workerBytes), publication.workerReceiptSha256, 'original engine receipt differs; a Desktop receipt is not interchangeable');
  const protocol = JSON.parse(protocolBytes),
    worker = JSON.parse(workerBytes);
  assert.equal(protocol.commit, request.protocolCommit);
  assert.equal(worker.schema, 'selected-source-wasm/v1');
  assert.equal(worker.nativeCompilationEvidence, true);
  assert.equal(worker.sourceCommit, request.sourceCommit);
  assert.equal(worker.sourceTree, request.sourceTree);
  assert.equal(worker.packageVersion, request.tag.slice(1));
  assert.deepEqual(worker.identity, producer);
  return sha(workerBytes) + '  formal-ai-engine-worker-' + request.tag.slice(1) + '.json\n' + sha(protocolBytes) + '  formal-ai-engine-protocol-' + request.tag.slice(1) + '.json\n';
}
function ordinary(file, maximum) {
  const stat = lstatSync(file);
  assert.ok(stat.isFile() && !stat.isSymbolicLink());
  assert.equal(realpathSync(file), resolve(file));
  assert.ok(stat.size > 0 && stat.size <= maximum);
  const bytes = readFileSync(file);
  assert.equal(bytes.length, stat.size);
  return bytes;
}
async function verifyOriginalProtocol(request, publication, release, directory, download, deadline) {
  let protocolPath = request.mode === 'checksum' ? null : request.protocolPath;
  if (!protocolPath) {
    const asset = releaseAsset(release, request.repository, request.tag, 'formal-ai-engine-protocol-' + request.tag.slice(1) + '.json', 1024 * 1024);
    protocolPath = join(directory, asset.name);
    await download({
      asset,
      repository: request.repository,
      tag: request.tag,
      destination: protocolPath,
      maxBytes: 1024 * 1024,
      deadline
    });
  }
  const bytes = ordinary(protocolPath, 1024 * 1024);
  assert.equal(sha(bytes), publication.binding.protocolSha256);
  const record = JSON.parse(bytes);
  assert.equal(record.commit, request.protocolCommit);
  assert.ok(Array.isArray(record.files) && record.files.length > 0 && record.files.length <= 100);
  const captured = join(directory, 'original-protocol');
  mkdirSync(captured);
  writeFileSync(join(captured, 'protocol-selection.json'), bytes, {
    flag: 'wx'
  });
  let total = 0;
  for (const file of record.files) {
    assert.match(file.path, /^[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*$/u);
    assert.ok(!file.path.split('/').some(part => ['.', '..', '.git'].includes(part)));
    const content = execFileSync('git', ['-C', request.root, 'show', request.protocolCommit + ':' + file.path], {
      maxBuffer: 4 * 1024 * 1024,
      timeout: 10000
    });
    total += content.length;
    assert.ok(total <= 16 * 1024 * 1024);
    const target = join(captured, file.path);
    mkdirSync(dirname(target), {
      recursive: true
    });
    writeFileSync(target, content, {
      flag: 'wx'
    });
  }
  const {
    verifyReleaseProtocol
  } = await import('./release-protocol.mjs');
  verifyReleaseProtocol(request.root, captured, request.protocolCommit, request.protocolSha256);
  return {
    path: protocolPath,
    bytes
  };
}
export async function verifyPublicBrowser(request) {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'public package execution/download is hosted CI only');
  assert.ok(['smoke', 'checksum'].includes(request.mode));
  assert.match(request.repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  engineAssetName(request.tag);
  revision(request.sourceCommit);
  revision(request.sourceTree);
  const directory = resolve(request.outputDirectory);
  mkdirSync(directory, {
    recursive: false
  });
  const deadline = Date.now() + (request.mode === 'checksum' ? 180000 : 480000),
    github = createReadonlyGithub(join(directory, 'metadata'), {
      deadline
    });
  const {
    downloadPublicReleaseAsset
  } = await import('./github-public-release-download.mjs');
  const base = 'repos/' + request.repository,
    release = github.json(base + '/releases/tags/' + request.tag);
  const publicationAsset = releaseAsset(release, request.repository, request.tag, 'browser-publication.json', 1024 * 1024);
  const publicationPath = join(directory, 'browser-publication.json');
  const publicationTransfer = await downloadPublicReleaseAsset({
    asset: publicationAsset,
    repository: request.repository,
    tag: request.tag,
    destination: publicationPath,
    maxBytes: 1024 * 1024,
    deadline
  });
  const publication = JSON.parse(ordinary(publicationPath, 1024 * 1024));
  positive(publication.binding.run);
  positive(publication.binding.attempt);
  const run = github.json(base + '/actions/runs/' + publication.binding.run + '/attempts/' + publication.binding.attempt);
  request = {
    ...request,
    protocolCommit: request.protocolCommit ?? run.head_sha,
    protocolSha256: request.protocolSha256 ?? publication.binding.protocolSha256
  };
  const producer = authenticateBrowserProducer(request, {
    publication,
    run,
    jobs: github.json(base + '/actions/runs/' + publication.binding.run + '/attempts/' + publication.binding.attempt + '/jobs?per_page=100'),
    tagCommit: github.json(base + '/commits/' + request.tag)
  }, {
    repository: process.env.GITHUB_REPOSITORY,
    run: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
    job: process.env.GITHUB_JOB,
    workflowCommit: process.env.GITHUB_WORKFLOW_SHA
  });
  const protocol = await verifyOriginalProtocol(request, publication, release, directory, downloadPublicReleaseAsset, deadline);
  const asset = releaseAsset(release, request.repository, request.tag, engineAssetName(request.tag), 64 * 1024 * 1024);
  const archivePath = join(directory, asset.name);
  const archiveTransfer = await downloadPublicReleaseAsset({
    asset,
    repository: request.repository,
    tag: request.tag,
    destination: archivePath,
    maxBytes: 64 * 1024 * 1024,
    deadline
  });
  const checksum = verifyPublicEngineBytes(request, publication, asset, ordinary(archivePath, 64 * 1024 * 1024));
  let receipt = request.mode === 'checksum' ? null : request.receiptPath;
  let workerTransfer = null;
  if (!receipt) {
    const receiptAsset = releaseAsset(release, request.repository, request.tag, 'formal-ai-engine-worker-' + request.tag.slice(1) + '.json', 1024 * 1024);
    receipt = join(directory, receiptAsset.name);
    workerTransfer = await downloadPublicReleaseAsset({
      asset: receiptAsset,
      repository: request.repository,
      tag: request.tag,
      destination: receipt,
      maxBytes: 1024 * 1024,
      deadline
    });
  }
  const workerBytes = ordinary(receipt, 1024 * 1024);
  const receipts = engineReceiptChecksums(request, publication, producer, protocol.bytes, workerBytes);
  writeFileSync(join(directory, 'engine-checksum.partial'), checksum + receipts, {
    flag: 'wx'
  });
  const evidence = {
    schema: 'browser-engine-public-download/v1',
    producer,
    sourceCommit: request.sourceCommit,
    sourceTree: request.sourceTree,
    protocolCommit: request.protocolCommit,
    protocolSha256: request.protocolSha256,
    archive: archiveTransfer,
    publication: publicationTransfer,
    workerReceipt: workerTransfer,
    workerSmoke: null,
    publicationAuthority: false
  };
  if (request.mode === 'smoke') {
    assert.equal(execFileSync('git', ['-C', request.root, 'rev-parse', 'HEAD'], {
      encoding: 'utf8'
    }).trim(), request.sourceCommit);
    assert.equal(execFileSync('git', ['-C', request.root, 'rev-parse', 'HEAD^{tree}'], {
      encoding: 'utf8'
    }).trim(), request.sourceTree);
    const {
      coldPackageSmoke
    } = await import('./browser-engine-cold-install.mjs');
    evidence.workerSmoke = await coldPackageSmoke({
      root: request.root,
      archive: archivePath,
      version: request.tag.slice(1),
      receipt,
      receiptDigest: publication.workerReceiptSha256,
      publicationStatus: publication.npm.status,
      producerBindings: producer,
      deadline
    });
  }
  writeFileSync(join(directory, 'evidence.json'), JSON.stringify(evidence, null, 2) + '\n', {
    flag: 'wx'
  });
  return evidence;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [request, ...extra] = process.argv.slice(2);
  assert.equal(extra.length, 0);
  assert.ok(request);
  verifyPublicBrowser(JSON.parse(ordinary(request, 1024 * 1024))).then(result => console.log(JSON.stringify(result))).catch(error => {
    console.error(error);
    process.exitCode = 1;
  });
}
