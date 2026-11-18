#!/usr/bin/env node
// Hosted, read-only verification of every byte in one immutable public release.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { validateDigestCoverage } from './release-advertisement.mjs';
import { tmpdir as temporaryDirectory } from 'node:os';
import { selectCompletedSource, completedDesktopPublication } from './release-extension-selection.mjs';
import { createReadonlyGithub } from './release-extension-github.mjs';
export const MAX_ASSET_BYTES = 2 * 1024 ** 3;
export const MAX_TOTAL_BYTES = 12 * 1024 ** 3;
const sha = bytes => {
  return createHash('sha256').update(bytes).digest('hex');
};
const commit = value => {
  return assert.match(value, /^[a-f0-9]{40}$/u);
};
const stable = value => {
  return assert.match(value, /^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
};
export function validateInventory(release, assets, repository, tag) {
  assert.match(repository, /^[A-Za-z0-9_-]+\/[A-Za-z0-9_.-]+$/u);
  stable(tag);
  assert.equal(release.tag_name, tag);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert.ok(Number.isSafeInteger(release.id) && release.id > 0);
  assert.ok(release.published_at);
  assert.ok(Array.isArray(assets) && assets.length > 0 && assets.length <= 100);
  const names = new Set(),
    ids = new Set();
  let total = 0;
  for (const asset of assets) {
    assert.match(asset.name, /^[A-Za-z0-9][A-Za-z0-9_.-]*$/u);
    assert.ok(!['.', '..'].includes(asset.name));
    assert.ok(!names.has(asset.name) && !ids.has(asset.id), 'duplicate public asset');
    names.add(asset.name);
    ids.add(asset.id);
    assert.ok(Number.isSafeInteger(asset.id) && asset.id > 0);
    assert.equal(asset.state, 'uploaded');
    assert.ok(Number.isSafeInteger(asset.size) && asset.size > 0 && asset.size <= MAX_ASSET_BYTES);
    assert.match(asset.digest, /^sha256:[a-f0-9]{64}$/u);
    assert.equal(asset.browser_download_url, 'https://github.com/' + repository + '/releases/download/' + tag + '/' + asset.name);
    total += asset.size;
    assert.ok(total <= MAX_TOTAL_BYTES, 'release byte budget exceeded');
  }
  assert.ok(names.has('SHA256SUMS.txt'), 'published checksums required');
  return {
    releaseId: release.id,
    tag,
    repository,
    totalBytes: total,
    assetCount: assets.length
  };
}
export function checksumRows(bytes) {
  assert.ok(Buffer.isBuffer(bytes) && bytes.length > 0 && bytes.length <= 1024 * 1024);
  const text = new TextDecoder('utf-8', {
    fatal: true
  }).decode(bytes);
  const rows = new Map();
  for (const line of text.trimEnd().split(/\r?\n/u)) {
    const match = /^([a-f0-9]{64})  ([A-Za-z0-9][A-Za-z0-9_.-]*)$/u.exec(line);
    assert.ok(match, 'invalid checksum row');
    assert.ok(!rows.has(match[2]), 'duplicate checksum row');
    rows.set(match[2], match[1]);
  }
  return rows;
}
export function assessCoverage({
  definition,
  version,
  checksumBytes,
  assets,
  downloads
}) {
  const rows = checksumRows(checksumBytes);
  const metadata = new Map(assets.map(asset => {
    return [asset.name, asset];
  }));
  const observed = new Map();
  for (const result of downloads) {
    const asset = metadata.get(result.name);
    assert.ok(asset, 'unpublished download');
    assert.ok(!observed.has(result.name), 'duplicate download observation');
    assert.equal(result.id, asset.id);
    assert.equal(result.size, asset.size);
    assert.equal(result.sha256, asset.digest.slice(7));
    assert.equal(result.completed, true);
    assert.equal(result.transport, 'anonymous-public-release-download');
    assert.equal(result.publicationAuthority, false);
    observed.set(result.name, result);
  }
  assert.equal(observed.size, assets.length, 'not every published asset was downloaded');
  for (const [name, hash] of rows) {
    assert.ok(metadata.has(name), 'checksum row names missing asset: ' + name);
    assert.equal(observed.get(name)?.sha256, hash, 'checksum differs from actual public bytes: ' + name);
  }
  for (const asset of assets) {
    if (/(?:formal-ai-engine.*\.tgz|formal-ai-engine-(?:worker|protocol).*\.json)$/u.test(asset.name)) {
      assert.ok(rows.has(asset.name), 'engine checksum row required: ' + asset.name);
    }
  }
  const advertisement = validateDigestCoverage(definition, version, checksumBytes, assets);
  assert.deepEqual(advertisement.missing, [], 'source-advertised public assets lack checksum coverage');
  return {
    checksumRows: rows.size,
    advertisedAssets: advertisement.expected.length,
    byteVerifiedAssets: observed.size,
    publicationAuthority: false
  };
}
async function githubJson(repository, suffix) {
  const token = process.env.GH_TOKEN || process.env.GITHUB_TOKEN;
  assert.ok(token, 'authenticated metadata API required');
  const response = await fetch('https://api.github.com/repos/' + repository + suffix, {
    headers: {
      authorization: 'Bearer ' + token,
      accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28'
    },
    redirect: 'error',
    signal: AbortSignal.timeout(30000)
  });
  assert.equal(response.status, 200, 'GitHub metadata request failed');
  const chunks = [];
  let size = 0;
  for await (const chunk of response.body) {
    size += chunk.length;
    assert.ok(size <= 2 * 1024 * 1024, 'metadata byte limit');
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}
async function sourceFile(repository, revision, file) {
  commit(revision);
  const record = await githubJson(repository, '/contents/' + file + '?ref=' + revision);
  assert.equal(record.type, 'file');
  assert.equal(record.encoding, 'base64');
  assert.ok(record.size > 0 && record.size <= 1024 * 1024);
  const bytes = Buffer.from(record.content, 'base64');
  assert.equal(bytes.length, record.size);
  const blob = createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex');
  assert.equal(blob, record.sha, 'authenticated source blob differs');
  return {
    bytes,
    sha256: sha(bytes),
    blob: record.sha
  };
}
export function releaseRoute({
  repository,
  tag = '',
  producerRunId,
  producerAttempt,
  selection = null
}) {
  if (producerRunId) {
    assert.ok(selection, 'completed producer source receipt required');
    assert.equal(selection.repository, repository);
    assert.equal(selection.runId, Number(producerRunId), 'source receipt belongs to another producer run');
    assert.equal(selection.attempt, Number(producerAttempt), 'source receipt belongs to another producer attempt');
    assert.equal(selection.selectionOnly, true);
    assert.equal(selection.publicationAuthorized, false);
    stable(selection.tag);
    commit(selection.sourceCommit);
    commit(selection.sourceTree);
    commit(selection.protocolCommit);
    if (tag) assert.equal(tag, selection.tag, 'requested tag differs from completed producer source');
    return {
      tag: selection.tag,
      path: '/releases/tags/' + encodeURIComponent(selection.tag)
    };
  }
  assert.equal(selection, null);
  if (tag) stable(tag);
  return {
    tag,
    path: tag ? '/releases/tags/' + encodeURIComponent(tag) : '/releases/latest'
  };
}
export function bindCompletedSource(selection, {
  tag,
  sourceCommit,
  sourceTree
}) {
  if (!selection) return;
  assert.equal(tag, selection.tag, 'published tag differs from completed source receipt');
  assert.equal(sourceCommit, selection.sourceCommit, 'published source differs from completed source receipt');
  assert.equal(sourceTree, selection.sourceTree, 'published tree differs from completed source receipt');
}
function completedProducerSource(repository, runId, attempt) {
  assert.match(String(runId), /^[1-9][0-9]*$/u);
  assert.match(String(attempt), /^[1-9][0-9]*$/u);
  const directory = fs.mkdtempSync(path.join(process.env.RUNNER_TEMP || temporaryDirectory(), 'public-release-source-'));
  const actual = createReadonlyGithub(directory, {
    deadline: Date.now() + 150000
  });
  // The maintained selector owns run/job/archive authentication. This wrapper
  // only narrows the source artifact's size before the authenticated download.
  const bounded = {
    ...actual,
    json(endpoint) {
      const value = actual.json(endpoint);
      if (/\/actions\/artifacts\/[1-9][0-9]*$/u.test(endpoint)) {
        assert.ok(Number.isSafeInteger(value.size_in_bytes) && value.size_in_bytes > 0 && value.size_in_bytes <= 2 * 1024 * 1024, 'source-selection artifact exceeds small read-only evidence budget');
      }
      return value;
    },
    bytes(endpoint) {
      const bytes = actual.bytes(endpoint);
      assert.ok(bytes.length <= 2 * 1024 * 1024);
      return bytes;
    }
  };
  const expected = {repository, runId, attempt};
  const disposition = completedDesktopPublication(expected, bounded);
  if (!disposition.active) return disposition;
  return selectCompletedSource({
    repository,
    runId,
    attempt
  }, bounded);
}
export async function resolveRelease({
  repository,
  tag = '',
  releaseId,
  producerRunId,
  producerAttempt
}) {
  const selected = producerRunId ? completedProducerSource(repository, producerRunId, producerAttempt) : null;
  if (selected?.active === false) return {repository, ...selected};
  const route = releaseRoute({
    repository,
    tag,
    producerRunId,
    producerAttempt,
    selection: selected
  });
  const release = await githubJson(repository, route.path);
  tag = release.tag_name;
  stable(tag);
  if (releaseId !== undefined) assert.equal(release.id, Number(releaseId), 'release identity changed');
  const listed = await githubJson(repository, '/releases/' + release.id + '/assets?per_page=100&page=1');
  const assets = listed.map(({
    id,
    name,
    size,
    state,
    digest,
    browser_download_url
  }) => {
    return {
      id,
      name,
      size,
      state,
      digest,
      browser_download_url
    };
  });
  if (assets.length === 100) {
    const extra = await githubJson(repository, '/releases/' + release.id + '/assets?per_page=100&page=2');
    assert.equal(extra.length, 0, 'more than 100 release assets');
  }
  const inventory = validateInventory(release, assets, repository, tag);
  const reference = await githubJson(repository, '/git/ref/tags/' + encodeURIComponent(tag));
  assert.equal(reference.ref, 'refs/tags/' + tag);
  let object = reference.object;
  for (let depth = 0; object.type === 'tag'; depth++) {
    assert.ok(depth < 4, 'annotated tag depth exceeded');
    commit(object.sha);
    object = (await githubJson(repository, '/git/tags/' + object.sha)).object;
  }
  assert.equal(object.type, 'commit');
  commit(object.sha);
  const source = await githubJson(repository, '/git/commits/' + object.sha);
  assert.equal(source.sha, object.sha);
  commit(source.tree.sha);
  const advertisement = await sourceFile(repository, source.sha, '.github/release-advertisement.json');
  bindCompletedSource(selected, {
    tag,
    sourceCommit: source.sha,
    sourceTree: source.tree.sha
  });
  const producer = selected ? {
    runId: selected.runId,
    attempt: selected.attempt,
    workflowSourceCommit: selected.protocolCommit,
    sourceReceiptSelection: selected
  } : null;
  const workflowCommit = producer?.workflowSourceCommit ?? source.sha;
  const workflow = await sourceFile(repository, workflowCommit, '.github/workflows/desktop-release.yml');
  assert.match(workflow.bytes.toString('utf8'), /^name: Desktop Release\r?\n/u);
  return {
    schema: 'public-release-download-selection/v1',
    ...inventory,
    sourceCommit: source.sha,
    sourceTree: source.tree.sha,
    metadataTransport: 'authenticated-github-api',
    assets,
    definition: JSON.parse(advertisement.bytes),
    advertisement: {
      sourceCommit: source.sha,
      sha256: advertisement.sha256,
      blob: advertisement.blob
    },
    producerWorkflow: {
      path: '.github/workflows/desktop-release.yml',
      sourceCommit: workflowCommit,
      sha256: workflow.sha256,
      run: producer,
      releaseProductionBinding: selected ? 'authenticated-completed-native-source-receipt' : 'not asserted by manual download verification'
    },
    publicationAuthority: false
  };
}
const retained = name => {
  return /^(?:SHA256SUMS\.txt|BUILD-PROVENANCE\.txt|formal-ai-(?:native|signing|wasm|engine)[A-Za-z0-9_.-]*\.json)$/u.test(name);
};
export function macosAssetNames(selection, architecture) {
  assert.ok(['arm64', 'x64'].includes(architecture));
  const version = selection.tag.slice(1),
    target = architecture === 'arm64' ? 'aarch64-apple-darwin' : 'x86_64-apple-darwin';
  return ['SHA256SUMS.txt', 'formal-ai-native-source-' + version + '.json', 'formal-ai-signing-macos-' + architecture + '-' + version + '.json', 'formal-ai-native-' + target + '-' + version + '.json', ...['dmg', 'zip'].map(ext => {
    return 'formal-ai-desktop-macos-' + architecture + '-' + version + '.' + ext;
  })];
}
export async function runVerification({
  selection,
  outputDirectory,
  mode = 'all',
  architecture
}) {
  assert.ok(['all', 'macos'].includes(mode));
  const directory = path.resolve(outputDirectory);
  assert.equal(directory, outputDirectory);
  fs.mkdirSync(directory, {
    recursive: true
  });
  assert.ok(!fs.lstatSync(directory).isSymbolicLink());
  const {
    downloadPublicReleaseAsset
  } = await import('./github-public-release-download.mjs');
  const wanted = mode === 'macos' ? macosAssetNames(selection, architecture) : selection.assets.map(asset => {
    return asset.name;
  });
  const assets = wanted.map(name => {
    const asset = selection.assets.find(item => {
      return item.name === name;
    });
    assert.ok(asset, 'missing selected asset ' + name);
    return asset;
  });
  const downloads = [];
  const journal = path.join(directory, 'download-observations.json');
  fs.writeFileSync(journal, JSON.stringify({
    schema: 'public-release-download-progress/v1',
    downloads: [],
    verificationComplete: false,
    publicationAuthority: false
  }) + '\n', {
    flag: 'wx'
  });
  let retainedBytes = 0;
  const started = Date.now(),
    deadline = started + 11 * 60 * 1000;
  for (const asset of assets) {
    assert.ok(Date.now() < deadline, 'public verification deadline exceeded');
    const destination = path.join(directory, asset.name);
    const keep = mode === 'macos' || retained(asset.name) && asset.size <= 1024 * 1024;
    try {
      const result = await downloadPublicReleaseAsset({
        asset,
        repository: selection.repository,
        tag: selection.tag,
        destination,
        maxBytes: MAX_ASSET_BYTES,
        deadline
      });
      assert.equal(result.repository, selection.repository);
      assert.equal(result.tag, selection.tag);
      assert.equal(result.id, asset.id);
      assert.equal(result.name, asset.name);
      assert.equal(result.size, asset.size);
      assert.equal(result.sha256, asset.digest.slice(7));
      assert.equal(result.completed, true);
      assert.equal(result.transport, 'anonymous-public-release-download');
      assert.equal(result.publicationAuthority, false);
      downloads.push(result);
      fs.writeFileSync(journal, JSON.stringify({
        schema: 'public-release-download-progress/v1',
        downloads,
        verificationComplete: false,
        publicationAuthority: false
      }, null, 2) + '\n');
      if (keep && mode === 'all') {
        retainedBytes += asset.size;
        assert.ok(retainedBytes <= 16 * 1024 * 1024, 'retained evidence limit');
      }
    } finally {
      if (!keep && fs.existsSync(destination)) fs.unlinkSync(destination);
    }
  }
  const checksumBytes = fs.readFileSync(path.join(directory, 'SHA256SUMS.txt'));
  const coverage = mode === 'all' ? assessCoverage({
    definition: selection.definition,
    version: selection.tag.slice(1),
    checksumBytes,
    assets: selection.assets,
    downloads
  }) : null;
  if (mode === 'macos') {
    const rows = checksumRows(checksumBytes);
    for (const result of downloads.filter(item => {
      return item.name !== 'SHA256SUMS.txt';
    })) {
      assert.equal(rows.get(result.name), result.sha256, 'selected Mac checksum missing or incorrect');
    }
  }
  const outcome = {
    schema: 'public-release-download-verification/v1',
    state: mode === 'all' ? 'all-public-bytes-and-checksums-verified' : 'selected-public-bytes-verified',
    repository: selection.repository,
    tag: selection.tag,
    releaseId: selection.releaseId,
    sourceCommit: selection.sourceCommit,
    sourceTree: selection.sourceTree,
    producerWorkflow: selection.producerWorkflow,
    downloads,
    coverage,
    started: new Date(started).toISOString(),
    finished: new Date().toISOString(),
    publicationAuthority: false
  };
  fs.writeFileSync(path.join(directory, 'download-evidence.json'), JSON.stringify(outcome, null, 2) + '\n', {
    flag: 'wx'
  });
  return outcome;
}
export async function main(environment = process.env) {
  const repository = environment.GITHUB_REPOSITORY || 'link-assistant/formal-ai';
  const selection = await resolveRelease({
    repository,
    tag: environment.RELEASE_TAG || '',
    releaseId: environment.RELEASE_ID || undefined,
    producerRunId: environment.PRODUCER_RUN_ID || undefined,
    producerAttempt: environment.PRODUCER_ATTEMPT || undefined
  });
  if (selection.active === false) {
    assert.equal(environment.RELEASE_VERIFY_MODE, 'resolve', 'only initial resolution can skip an inactive producer');
    assert.ok(!environment.RELEASE_SELECTION && !environment.RELEASE_TAG && !environment.RELEASE_ID,
      'inactive producer cannot replace a pinned release');
    if (environment.GITHUB_OUTPUT) fs.appendFileSync(environment.GITHUB_OUTPUT, 'active=false\n');
    console.log(JSON.stringify(selection));
    return selection;
  }
  if (environment.RELEASE_SELECTION) {
    const stat = fs.lstatSync(environment.RELEASE_SELECTION);
    assert.ok(stat.isFile() && !stat.isSymbolicLink() && stat.size <= 1024 * 1024);
    const pinned = JSON.parse(fs.readFileSync(environment.RELEASE_SELECTION, 'utf8'));
    for (const field of ['schema', 'repository', 'tag', 'releaseId', 'sourceCommit', 'sourceTree', 'assets', 'advertisement', 'producerWorkflow']) {
      assert.deepEqual(selection[field], pinned[field], 'resolved public selection changed: ' + field);
    }
  }
  if (environment.RELEASE_SOURCE_SHA) assert.equal(selection.sourceCommit, environment.RELEASE_SOURCE_SHA);
  if (environment.RELEASE_SOURCE_TREE) assert.equal(selection.sourceTree, environment.RELEASE_SOURCE_TREE);
  const outputDirectory = path.resolve(environment.RELEASE_DOWNLOAD_DIRECTORY || 'release-download-evidence');
  fs.mkdirSync(outputDirectory, {
    recursive: true
  });
  fs.writeFileSync(path.join(outputDirectory, 'selection.json'), JSON.stringify(selection, null, 2) + '\n', {
    flag: 'wx'
  });
  if (environment.GITHUB_OUTPUT) {
    for (const [key, value] of Object.entries({
      active: 'true',
      tag: selection.tag,
      release_id: selection.releaseId,
      source_sha: selection.sourceCommit,
      source_tree: selection.sourceTree
    })) {
      fs.appendFileSync(environment.GITHUB_OUTPUT, key + '=' + value + '\n');
    }
  }
  if (environment.RELEASE_VERIFY_MODE === 'resolve') return selection;
  return runVerification({
    selection,
    outputDirectory,
    mode: environment.RELEASE_VERIFY_MODE || 'all',
    architecture: environment.MACOS_ARCH
  });
}
if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  main().catch(error => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
