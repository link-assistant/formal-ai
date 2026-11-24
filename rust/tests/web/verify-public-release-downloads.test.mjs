import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import {createRequire} from 'node:module';
const YAML = createRequire(import.meta.url)('yaml');
import { REPO_ROOT } from '../../../js/server/lino.mjs';
import { runVerification, releaseRoute, bindCompletedSource, validateInventory, checksumRows, assessCoverage, macosAssetNames, MAX_ASSET_BYTES, MAX_TOTAL_BYTES } from '../../../scripts/verify-public-release-downloads.mjs';
import { validateDigestCoverage } from '../../../scripts/release-advertisement.mjs';
const repository = 'link-assistant/formal-ai',
  tag = 'v1.2.3';
const hash = '1'.repeat(64),
  version = tag.slice(1);
const definition = JSON.parse(fs.readFileSync(REPO_ROOT + '/.github/release-advertisement.json', 'utf8'));
const expected = validateDigestCoverage(definition, version, Buffer.from(hash + '  probe\n'), []).expected;
function asset(name, id = 1) {
  return {
    name,
    id,
    size: 10,
    state: 'uploaded',
    digest: 'sha256:' + hash,
    browser_download_url: 'https://github.com/' + repository + '/releases/download/' + tag + '/' + name
  };
}
function fixture() {
  const assets = [...expected, 'SHA256SUMS.txt'].map((name, index) => {
    return asset(name, index + 1);
  });
  const release = {
    id: 10,
    tag_name: tag,
    draft: false,
    prerelease: false,
    published_at: '2026-10-17T00:00:00Z'
  };
  const downloads = assets.map(item => {
    return {
      id: item.id,
      name: item.name,
      size: item.size,
      sha256: hash,
      repository,
      tag,
      completed: true,
      transport: 'anonymous-public-release-download',
      publicationAuthority: false
    };
  });
  const checksumBytes = Buffer.from(expected.map(name => {
    return hash + '  ' + name + '\n';
  }).join(''));
  return {
    release,
    assets,
    downloads,
    checksumBytes,
    definition,
    version
  };
}
test('source-derived advertised fixture requires every observed public byte and never grants publication authority', () => {
  const value = fixture();
  assert.equal(validateInventory(value.release, value.assets, repository, tag).assetCount, value.assets.length);
  const result = assessCoverage(value);
  assert.equal(result.advertisedAssets, expected.length);
  assert.equal(result.byteVerifiedAssets, value.assets.length);
  assert.equal(result.publicationAuthority, false);
});
for (const [name, change] of [['draft', f => {
  return f.release.draft = true;
}], ['prerelease', f => {
  return f.release.prerelease = true;
}], ['wrong release tag', f => {
  return f.release.tag_name = 'v1.2.4';
}], ['unpublished', f => {
  return f.release.published_at = null;
}], ['duplicate name', f => {
  return f.assets.push({
    ...f.assets[0],
    id: 999
  });
}], ['duplicate id', f => {
  return f.assets[1].id = f.assets[0].id;
}], ['path traversal', f => {
  return f.assets[0].name = '../escape';
}], ['URL substitution', f => {
  return f.assets[0].browser_download_url = 'https://example.com/asset';
}], ['unknown digest', f => {
  return f.assets[0].digest = null;
}], ['oversized asset', f => {
  return f.assets[0].size = MAX_ASSET_BYTES + 1;
}], ['more than 100 assets', f => {
  return f.assets = Array.from({
    length: 101
  }, (_, i) => {
    return asset('asset-' + i, i + 1);
  });
}], ['total byte budget', f => {
  return f.assets = Array.from({
    length: 7
  }, (_, i) => {
    return {
      ...asset('asset-' + i, i + 1),
      size: MAX_ASSET_BYTES
    };
  });
}]]) test('inventory refuses ' + name, () => {
  const value = fixture();
  change(value);
  assert.throws(() => {
    return validateInventory(value.release, value.assets, repository, tag);
  });
});
for (const [name, change] of [['missing download', f => {
  return f.downloads.pop();
}], ['duplicate download', f => {
  return f.downloads.push(f.downloads[0]);
}], ['wrong downloaded digest', f => {
  return f.downloads[0].sha256 = '2'.repeat(64);
}], ['wrong downloaded size', f => {
  return f.downloads[0].size++;
}], ['partial stream', f => {
  return f.downloads[0].completed = false;
}], ['authenticated asset API instead of public transport', f => {
  return f.downloads[0].transport = 'github-asset-api';
}], ['publication claim', f => {
  return f.downloads[0].publicationAuthority = true;
}], ['unpublished checksum row', f => {
  return f.checksumBytes = Buffer.concat([f.checksumBytes, Buffer.from(hash + '  absent.tgz\n')]);
}], ['checksum-byte mismatch', f => {
  return f.checksumBytes = Buffer.from(f.checksumBytes.toString().replace(hash, '2'.repeat(64)));
}], ['missing advertised checksum', f => {
  return f.checksumBytes = Buffer.from(f.checksumBytes.toString().split('\n').slice(1).join('\n'));
}]]) test('coverage refuses ' + name, () => {
  const value = fixture();
  change(value);
  assert.throws(() => {
    return assessCoverage(value);
  });
});
test('new durable engine receipt is not silently exempt from checksum coverage', () => {
  const value = fixture(),
    extra = asset('formal-ai-engine-worker-' + version + '.json', 999);
  value.assets.push(extra);
  value.downloads.push({
    ...value.downloads[0],
    id: extra.id,
    name: extra.name
  });
  assert.throws(() => {
    return assessCoverage(value);
  }, /engine checksum row required/);
  value.checksumBytes = Buffer.concat([value.checksumBytes, Buffer.from(hash + '  ' + extra.name + '\n')]);
  assert.equal(assessCoverage(value).byteVerifiedAssets, value.assets.length);
});
for (const bytes of [Buffer.from(hash + '  a\n' + hash + '  a\n'), Buffer.from(hash + '  ../escape\n'), Buffer.from('bad\n'), Buffer.from([255])]) {
  test('checksum parser rejects malformed, duplicate or non-UTF8 rows ' + bytes.length, () => {
    return assert.throws(() => {
      return checksumRows(bytes);
    });
  });
}
test('Mac selection requires fresh DMG and ZIP plus original source/signing/native receipts', () => {
  for (const architecture of ['arm64', 'x64']) {
    const names = macosAssetNames({
      tag
    }, architecture);
    assert.equal(names.length, 6);
    assert.ok(names.some(name => {
      return name.endsWith('.dmg');
    }));
    assert.ok(names.some(name => {
      return name.endsWith('.zip');
    }));
    assert.ok(names.includes('formal-ai-native-source-' + version + '.json'));
  }
  assert.throws(() => {
    return macosAssetNames({
      tag
    }, 'universal');
  });
  assert.equal(MAX_TOTAL_BYTES, 12 * 1024 ** 3);
});
function boundedMacUploadPaths(workflow) {
  const start = workflow.indexOf('name: public-macos-evidence-');
  assert.ok(start > 0);
  const block = workflow.slice(start).split('path: |')[1].split('if-no-files-found:')[0];
  const paths = block.split('\n').map(line => line.trim()).filter(Boolean);
  const prefix = '$' + '{{ runner.temp }}/public-macos-packages/macos-public-evidence-*/';
  const allowed = ['outcome.json', 'dmg/electron.stdout.log', 'dmg/electron.stderr.log', 'zip/electron.stdout.log', 'zip/electron.stderr.log'].map(tail => prefix + tail);
  assert.deepEqual(paths.sort(), allowed.sort());
  for (const file of paths) assert.ok(!file.endsWith('/') && !file.includes('**'));
  return paths;
}
test('hosted workflow uploads only Mac outcome and exact renderer log files', () => {
  const source = fs.readFileSync(new URL('../../../.github/workflows/verify-release-downloads.yml', import.meta.url), 'utf8');
  assert.equal(boundedMacUploadPaths(source).length, 5);
  const recursive = source.replace('macos-public-evidence-*/outcome.json', 'macos-public-evidence-*/');
  assert.throws(() => boundedMacUploadPaths(recursive));
  const applications = source.replace('macos-public-evidence-*/outcome.json', 'macos-public-evidence-*/*.app/**');
  assert.throws(() => boundedMacUploadPaths(applications));
});
test('browser uses selected release source while the protocol helper remains in trusted outer checkout', () => {
  const source = fs.readFileSync(new URL('../../../.github/workflows/verify-release-downloads.yml', import.meta.url), 'utf8');
  const browser = source.slice(source.indexOf('  browser-public-package:'));
  for (const required of ['ref: $' + '{{ needs.resolve.outputs.source_sha }}', 'path: _source', '--prefix _source/vscode', 'node _source/vscode/node_modules/playwright/cli.js', "root: path.resolve('_source')", 'node scripts/browser-engine-public-download.mjs "$REQUEST_FILE"', 'browser-public-evidence/engine-checksum.partial']) assert.ok(browser.includes(required), required);
  assert.doesNotMatch(source, /actions\/(?:checkout|setup-node)@v6/u);
  assert.doesNotMatch(browser, /root: process\.cwd\(\)/u);
});
const originalProducer = {
  repository,
  tag,
  sourceCommit: 'a'.repeat(40),
  sourceTree: 'b'.repeat(40),
  protocolCommit: 'c'.repeat(40),
  runId: 42,
  attempt: 2,
  selectionOnly: true,
  publicationAuthorized: false
};
function routeFor(selection = originalProducer, override = {}) {
  return releaseRoute({
    repository,
    producerRunId: 42,
    producerAttempt: 2,
    selection,
    ...override
  });
}
test('an older completed Desktop run selects its authenticated tag before a different latest release', () => {
  const newerLatest = {
    tag_name: 'v1.2.4'
  };
  assert.equal(routeFor().path, '/releases/tags/' + tag);
  assert.notEqual(routeFor().path, '/releases/latest');
  assert.notEqual(routeFor().tag, newerLatest.tag_name);
  assert.deepEqual(releaseRoute({
    repository
  }), {
    tag: '',
    path: '/releases/latest'
  });
});
for (const [name, change] of [['original run', value => value.runId = 99], ['original attempt', value => value.attempt = 1], ['repository', value => value.repository = 'other/repository'], ['source receipt absent', () => null]]) test('completed producer route rejects mismatched ' + name, () => {
  const selection = {
    ...originalProducer
  };
  const changed = change(selection);
  assert.throws(() => routeFor(name === 'source receipt absent' ? null : selection));
});
test('producer source cannot be rebound to another published tag, commit or tree', () => {
  const source = {
    tag,
    sourceCommit: originalProducer.sourceCommit,
    sourceTree: originalProducer.sourceTree
  };
  bindCompletedSource(originalProducer, source);
  for (const [field, value] of [['tag', 'v1.2.4'], ['sourceCommit', 'd'.repeat(40)], ['sourceTree', 'e'.repeat(40)]]) {
    assert.throws(() => bindCompletedSource(originalProducer, {
      ...source,
      [field]: value
    }));
  }
  assert.throws(() => routeFor(originalProducer, {
    tag: 'v1.2.4'
  }));
});
test('Mac job environment does not use the disallowed runner context', () => {
  const source = fs.readFileSync(new URL('../../../.github/workflows/verify-release-downloads.yml', import.meta.url), 'utf8');
  function assertJobEnvironment(text) {
    const section = text.slice(text.indexOf('  macos-public-packages:')).split('    steps:')[0];
    assert.ok(!section.includes('$' + '{{ runner.'));
  }
  assertJobEnvironment(source);
  assert.throws(() => assertJobEnvironment(source.replace('      MACOS_ARCH:', '      UNSAFE: $' + '{{ runner.temp }}\n      MACOS_ARCH:')));
  assert.ok(source.includes('PRODUCER_ATTEMPT: $' + '{{ github.event.workflow_run.run_attempt'));
});

function assertReadOnlyCompletionBoundary(text) {
  const value = YAML.parse(text);
  assert.match(text, /^on: # zizmor: ignore\[dangerous-triggers\]$/mu);
  assert.equal((text.match(/zizmor: ignore/gu) ?? []).length, 1);
  assert.deepEqual(Object.keys(value.on).sort(), ['workflow_run']);
  assert.deepEqual(value.on.workflow_run, {
    workflows: ['Desktop Release'], types: ['completed'], branches: ['main']
  });
  assert.deepEqual(value.permissions, {contents: 'read', actions: 'read'});
  assert.equal(value['cache-mode'], undefined, 'no cache-mode may broaden trigger defaults');
  assert.equal(value.jobs.resolve.if,
    "github.event.workflow_run.conclusion == 'success' && github.event.workflow_run.head_repository.full_name == github.repository");
  assert.doesNotMatch(text, /\$\{\{\s*secrets\./u);
  for (const [name, job] of Object.entries(value.jobs)) {
    assert.equal(job.permissions, undefined, 'no job may elevate token privileges');
    assert.equal(job['cache-mode'], undefined, 'no job may override cache denial');
    if (name !== 'resolve') assert.equal(job.needs, 'resolve');
    for (const step of job.steps) {
      assert.ok(!step.uses?.startsWith('actions/cache@'));
      if (step.uses?.startsWith('actions/setup-node@')) {
        assert.equal(step.with['package-manager-cache'], false, 'automatic npm cache is disabled');
        assert.equal(step.with.cache, undefined);
      }
      if (step.uses?.startsWith('actions/checkout@')) {
        assert.equal(step.with['persist-credentials'], false);
        const expected = step.with.path === '_source'
          ? '$' + '{{ needs.resolve.outputs.source_sha }}' : '$' + '{{ github.workflow_sha }}';
        assert.equal(step.with.ref, expected, 'no triggering branch or head checkout');
        assert.equal(step.with.repository, undefined);
      }
      if (step.uses?.endsWith('/.github/actions/download-artifact')) {
        assert.equal(step.with.name, 'public-release-selection');
        assert.equal(step.with['run-id'], undefined, 'only this run supplies selection data');
        assert.equal(step.with['github-token'], undefined);
      }
      if (step.uses?.startsWith('actions/upload-artifact@')) {
        for (const file of step.with.path.trim().split('\n')) {
          assert.ok(/\.(?:json|txt|log)$/u.test(file) ||
            file === '$' + '{{ runner.temp }}/browser-public-evidence/engine-checksum.partial');
        }
      }
    }
  }
  const select = value.jobs.resolve.steps.find(step => step.id === 'select');
  assert.equal(select.env.PRODUCER_RUN_ID, '$' + "{{ github.event.workflow_run.id || '' }}");
  assert.equal(select.env.PRODUCER_ATTEMPT, '$' + "{{ github.event.workflow_run.run_attempt || '' }}");
  assert.equal(select.run, 'node scripts/verify-public-release-downloads.mjs');
  assert.equal(select.env.RELEASE_TAG, undefined, 'producer receipt alone selects the tag');
}

test('the annotated completion trigger retains default-source read-only authenticated selection', () => {
  const source = fs.readFileSync(new URL('../../../.github/workflows/verify-release-downloads.yml', import.meta.url), 'utf8');
  assertReadOnlyCompletionBoundary(source);
  const mutations = [
    text => text.replace('  workflow_run:\n', '  workflow_dispatch: {}\n  workflow_run:\n'),
    ...['write', 'write-only', 'read', 'none'].map(mode => text =>
      text.replace('permissions:\n', 'cache-mode: ' + mode + '\npermissions:\n')),
    text => text.replace('  resolve:\n', '  resolve:\n    cache-mode: write\n'),
    text => text.replace('contents: read', 'contents: write'),
    text => text.replace('branches: [main]', 'branches: [untrusted]'),
    text => text.replace("conclusion == 'success'", "conclusion != 'cancelled'"),
    text => text.replace('head_repository.full_name == github.repository', 'true'),
    text => text.replace('ref: $' + '{{ github.workflow_sha }}',
      'ref: $' + '{{ github.event.workflow_run.head_sha }}'),
    text => text.replace('persist-credentials: false', 'persist-credentials: true'),
    text => text.replace('package-manager-cache: false', 'package-manager-cache: true'),
    text => text.replace('RELEASE_VERIFY_MODE: resolve', 'SECRET: $' + '{{ secrets.OTHER }}'),
    text => text.replaceAll('name: public-release-selection\n          path:',
      'name: untrusted-selection\n          path:'),
    text => text.replace('browser-public-evidence/evidence.json', 'browser-public-evidence/*.tgz')
  ];
  for (const mutate of mutations) assert.throws(() => assertReadOnlyCompletionBoundary(mutate(source)));
});

test('download resolution hydrates only its reviewed controller inputs while public verification stays exact', async () => {
  const {posix} = await import('node:path');
  const source = fs.readFileSync(new URL('../../../.github/workflows/verify-release-downloads.yml', import.meta.url), 'utf8');
  assertReadOnlyCompletionBoundary(source);
  const resolver = YAML.parse(source).jobs.resolve;
  assert.equal(resolver['timeout-minutes'], 6);
  const checkout = resolver.steps.find(step => step.uses === 'actions/checkout@v7');
  assert.equal(checkout['timeout-minutes'], 2);
  assert.deepEqual(checkout.with, {
    ref: '$' + '{{ github.workflow_sha }}', 'persist-credentials': false,
    'fetch-depth': 1, filter: 'blob:none', 'sparse-checkout-cone-mode': false,
    'sparse-checkout': '/scripts/\n/package.json\n',
  });
  assert.deepEqual(Object.fromEntries(Object.entries(checkout.env).map(([key, value]) => [key, String(value)])), {
    GIT_CONFIG_COUNT: '2', GIT_CONFIG_KEY_0: 'http.lowSpeedLimit', GIT_CONFIG_VALUE_0: '1024',
    GIT_CONFIG_KEY_1: 'http.lowSpeedTime', GIT_CONFIG_VALUE_1: '20',
  });
  const select = resolver.steps.find(step => step.id === 'select');
  assert.equal(select['timeout-minutes'], 3);
  assert.equal(select.run, 'node scripts/verify-public-release-downloads.mjs');
  const expected = [
    'scripts/github-artifact-by-identifier.mjs',
    'scripts/github-public-release-download.mjs',
    'scripts/lib/native-response-capture.mjs',
    'scripts/lib/native-session-projections.mjs',
    'scripts/lib/native-session-source.mjs',
    'scripts/qualified-data-artifact-profiles.mjs',
    'scripts/release-advertisement.mjs',
    'scripts/release-extension-github.mjs',
    'scripts/release-extension-policy.mjs',
    'scripts/release-extension-selection.mjs',
    'scripts/release-protocol.mjs',
    'scripts/verify-public-release-downloads.mjs',
  ];
  const queue = ['scripts/verify-public-release-downloads.mjs'], seen = new Set();
  let literalDynamicImportCovered = false;
  while (queue.length) {
    const file = queue.shift();
    if (seen.has(file)) continue;
    assert.ok(seen.size < 32, 'finite controller import closure');
    assert.ok(file.startsWith('scripts/'));
    seen.add(file);
    const text = fs.readFileSync(new URL('../../../' + file, import.meta.url), 'utf8');
    assert.doesNotMatch(text, /import\s*\((?!\s*['"])/u, 'nonliteral dynamic import requires explicit coverage');
    for (const match of text.matchAll(/(?:from\s*|import\s*\()(['"])([^'"]+)\1/g)) {
      const specifier = match[2];
      if (specifier.startsWith('.')) queue.push(posix.normalize(posix.join(posix.dirname(file), specifier)));
      else assert.ok(specifier.startsWith('node:'), 'no unhydrated third-party package');
    }
    if (file === 'scripts/verify-public-release-downloads.mjs') {
      assert.match(text, /import\('\.\/github-public-release-download\.mjs'\)/u);
      literalDynamicImportCovered = true;
    }
  }
  assert.deepEqual([...seen].sort(), expected);
  assert.equal(literalDynamicImportCovered, true);
  // Reviewed main 101992d84e7b12c843722992050585192dfc3996: acquisition changes cannot alter actual downstream verification.
  const downstream = source.slice(source.indexOf('\n  all-public-bytes:\n'));
  assert.notEqual(source.indexOf('\n  all-public-bytes:\n'), -1);
  // Normalize the reviewed runner pin, artifact transport, Mac shell and verification step label.
  // Every other downstream byte remains bound to the original verification contract.
  assert.doesNotMatch(downstream, /runs-on: ubuntu-latest/u);
  const adapter = '      - uses: ./.github/actions/download-artifact\n';
  const originalAdapter = '      - uses: actions/download-artifact@v8\n        env:\n          NODE_OPTIONS: --disable-warning=DEP0005\n';
  assert.equal(downstream.split(adapter).length - 1, 2);
  const shell = '    defaults:\n      run:\n        shell: bash\n';
  assert.equal(downstream.split(shell).length - 1, 1);
  assert.deepEqual(YAML.parse(source).jobs['macos-public-packages'].defaults, {run: {shell: 'bash'}});
  const stepLabel = 'Stream and verify every public asset';
  assert.equal(downstream.split(stepLabel).length - 1, 1);
  const original = downstream.replaceAll('runs-on: ubuntu-24.04', 'runs-on: ubuntu-latest')
    .replaceAll(adapter, originalAdapter).replace(shell, '')
    .replace(stepLabel, 'Stream, hash and delete every public asset');
  assert.equal(createHash('sha256').update(original).digest('hex'), '11977c67b61a050577b42afa0ea0f0f37bc321e03601bd66e1b5e03d3b8f4018');
});

// Main run 45295717630 exhausted the serial verifier's deadline after 26 of 74 assets.
test('full release verification overlaps transfers with a limit of three and retains only receipts', async t => {
  const supplied = fixture();
  const directory = fs.mkdtempSync(join(tmpdir(), 'formal-ai-public-release-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const payload = Buffer.from('verify the actual public bytes\n');
  const digest = 'sha256:' + createHash('sha256').update(payload).digest('hex');
  const checksums = Buffer.from(expected.map(name => digest.slice(7) + '  ' + name + '\n').join(''));
  const uniqueAssets = supplied.assets.map(item => item.name === 'SHA256SUMS.txt'
    ? { ...item, size: checksums.length, digest: 'sha256:' + createHash('sha256').update(checksums).digest('hex') }
    : { ...item, size: payload.length, digest });
  const selection = {
    repository, tag, assets: uniqueAssets, definition, releaseId: supplied.release.id
  };
  let inFlight = 0;
  let peak = 0;
  const visited = [];
  t.mock.method(globalThis, 'fetch', async url => {
    const name = new URL(url).pathname.split('/').at(-1);
    visited.push(name);
    inFlight++;
    peak = Math.max(peak, inFlight);
    await new Promise(resolve => setImmediate(resolve));
    inFlight--;
    return new Response(name === 'SHA256SUMS.txt' ? checksums : payload);
  });
  const outcome = await runVerification({ selection, outputDirectory: directory });
  assert.equal(peak, 3);
  const names = uniqueAssets.map(item => item.name);
  assert.deepEqual([...visited].sort(), [...names].sort());
  assert.deepEqual(outcome.downloads.map(item => item.name), names);
  assert.equal(outcome.coverage.byteVerifiedAssets, uniqueAssets.length);
  assert.equal(fs.existsSync(join(directory, 'SHA256SUMS.txt')), true);
  for (const item of uniqueAssets.filter(item => !item.name.endsWith('.txt') && !item.name.endsWith('.json'))) {
    assert.equal(fs.existsSync(join(directory, item.name)), false);
  }
});

test('a failed public transfer identifies its asset and cannot produce a complete result', async t => {
  const directory = fs.mkdtempSync(join(tmpdir(), 'formal-ai-public-release-failed-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const assets = Array.from({ length: 6 }, (_, index) => asset('artifact-' + index + '.tgz', index + 1));
  const selection = { repository, tag, assets, definition };
  const requests = [];
  t.mock.method(globalThis, 'fetch', async url => {
    requests.push(new URL(url).pathname.split('/').at(-1));
    if (requests.length === 1) return new Response(null, { status: 503 });
    await new Promise(resolve => setImmediate(resolve));
    return new Response(null, { status: 503 });
  });
  await assert.rejects(runVerification({ selection, outputDirectory: directory }), /public asset artifact-0\.tgz: .*complete bytes/u);
  assert.deepEqual(requests, assets.slice(0, 3).map(item => item.name));
  const progress = JSON.parse(fs.readFileSync(join(directory, 'download-observations.json'), 'utf8'));
  assert.equal(progress.verificationComplete, false);
  assert.deepEqual(progress.downloads, []);
  assert.equal(fs.existsSync(join(directory, 'download-evidence.json')), false);
});
