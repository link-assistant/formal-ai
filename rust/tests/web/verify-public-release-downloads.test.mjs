import assert from 'node:assert/strict';
import fs from 'node:fs';
import { test } from 'node:test';
import { REPO_ROOT } from '../../../js/server/lino.mjs';
import { releaseRoute, bindCompletedSource, validateInventory, checksumRows, assessCoverage, macosAssetNames, MAX_ASSET_BYTES, MAX_TOTAL_BYTES } from '../../../scripts/verify-public-release-downloads.mjs';
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
