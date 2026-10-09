// Workflow-owned immutable release transfers. Native/compiler/source-authority commands run only in GitHub CI.
// Exported descriptor and tar operations support controlled fixtures without launching a native executable.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync, lstatSync, copyFileSync, chmodSync, openSync, closeSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const run = (command, args, options = {}) => execFileSync(command, args, {
  encoding: 'utf8',
  timeout: 120000,
  maxBuffer: 256 * 1024 * 1024,
  ...options
});
const git = args => run('git', args).trim();
const read = path => JSON.parse(readFileSync(path, 'utf8'));
const output = record => {
  for (const [key, value] of Object.entries(record)) {
    assert.ok(!String(value).includes('\n'));
    writeFileSync(process.env.GITHUB_OUTPUT, key + '=' + value + '\n', {
      flag: 'a'
    });
  }
};
const identity = () => ({
  repository: process.env.GITHUB_REPOSITORY,
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  protocol: process.env.GITHUB_SHA,
  source: process.env.EXPECT_SOURCE,
  tree: process.env.EXPECT_TREE,
  version: process.env.RELEASE_VERSION,
  compilerSha256: process.env.EXPECT_COMPILER
});
// Bind physical bytes to independently supplied workflow producer identity, never descriptor self-claims.
export function verifyDescriptor(record, expected, bytes) {
  assert.match(expected.repository, /^[\w.-]+\/[\w.-]+$/);
  assert.match(expected.run, /^[1-9][0-9]*$/);
  assert.match(expected.attempt, /^[1-9][0-9]*$/);
  for (const key of ['protocol', 'source', 'tree']) assert.match(expected[key], /^[a-f0-9]{40}$/);
  assert.match(expected.compilerSha256, /^[a-f0-9]{64}$/);
  assert.match(expected.bundleSha256, /^[a-f0-9]{64}$/);
  assert.match(expected.version, /^\d+\.\d+\.\d+$/);
  assert.match(expected.producerJob, /^(?:auto|manual)_(?:compile-release|verify-package|publish-verify-images)$/);
  assert.ok(['binary', 'package', 'images'].includes(expected.kind));
  assert.equal(record.schema, 'release-stage-transfer/v1');
  assert.equal(record.complete, true);
  assert.equal(record.status, 0);
  assert.equal(record.signal, null);
  for (const key of ['repository', 'run', 'attempt', 'protocol', 'source', 'tree', 'version', 'compilerSha256']) assert.equal(record.identity[key], expected[key], 'independent expected identity differs ' + key);
  assert.equal(record.kind, expected.kind);
  assert.equal(record.producerJob, expected.producerJob);
  assert.equal(sha(bytes), expected.bundleSha256, 'actual transfer bytes differ');
  assert.equal(record.sha256, expected.bundleSha256);
  assert.equal(record.bytes, bytes.length);
  assert.ok(bytes.length > 0);
  assert.match(record.sha256, /^[a-f0-9]{64}$/u);
  return record;
}
export function verifySourceDescriptor(record, expected, archive, independentArchive) {
  assert.equal(record.schema, 'release-stage-source/v1');
  assert.match(expected.producerJob, /^(?:auto|manual)_prepare-source$/);
  assert.equal(record.producerJob, expected.producerJob);
  for (const key of ['repository', 'run', 'attempt', 'protocol', 'source', 'tree', 'version', 'compilerSha256']) {
    assert.ok(expected[key]);
    assert.equal(record.identity[key], expected[key], 'source independent identity differs ' + key);
  }
  assert.match(expected.archiveSha256, /^[a-f0-9]{64}$/);
  assert.equal(sha(archive), expected.archiveSha256);
  assert.equal(record.archiveSha256, expected.archiveSha256);
  assert.equal(sha(independentArchive), expected.archiveSha256, 'independently checked-out source bytes differ');
  assert.equal(sha(record.compiler), expected.compilerSha256);
  return record;
}
function protocol() {
  assert.equal(process.env.GITHUB_WORKFLOW_REF,process.env.GITHUB_REPOSITORY+'/.github/workflows/release.yml@refs/heads/main','only original guarded release caller');
  assert.ok(['auto','manual'].includes(process.env.STAGED_RELEASE_MODE));
  assert.equal(process.env.GITHUB_EVENT_NAME,process.env.STAGED_RELEASE_MODE==='auto'?'push':'workflow_dispatch');
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'source/compiler/native authority commands are GitHub CI only');
  run('git', ['-C', '_protocol', 'diff', '--exit-code', 'HEAD']);
  assert.equal(run('git', ['-C', '_protocol', 'rev-parse', 'HEAD']).trim(), process.env.GITHUB_SHA, 'trusted protocol checkout differs');
  assert.equal(process.env.GITHUB_REF, 'refs/heads/main');
  assert.ok(['push', 'workflow_dispatch'].includes(process.env.GITHUB_EVENT_NAME));
  if (process.env.GITHUB_EVENT_NAME === 'workflow_dispatch') assert.equal(process.env.RELEASE_MODE, 'instant');
  assert.equal(process.env.RUSTFLAGS, '-Dwarnings');
  assert.ok(!process.env.CARGO_ENCODED_RUSTFLAGS);
}
function compiler() {
  const bytes = run('rustc', ['-vV']);
  assert.equal(sha(bytes), process.env.EXPECT_COMPILER, 'actual compiler differs from selected producer');
  return bytes;
}
// Recheck actual compiler, tracked source, independently fetched current main and exact release tag.
async function authority() {
  protocol();
  compiler();
  run('git', ['diff', '--exit-code', 'HEAD']);
  const {
    validatePreparedSelection
  } = await import('./prepared-release-binary.mjs');
  const selected = validatePreparedSelection(process.cwd(), process.env.RELEASE_VERSION, process.env);
  assert.equal(selected.head, process.env.EXPECT_SOURCE);
  assert.equal(selected.tree, process.env.EXPECT_TREE);
  return selected;
}
function tarTo(path, args) {
  mkdirSync(resolve(path, '..'), {
    recursive: true
  });
  const fd = openSync(path, 'w');
  try {
    const result = spawnSync('tar', args, {
      stdio: ['ignore', fd, 'pipe'],
      encoding: 'utf8',
      timeout: 120000
    });
    if (result.error) throw result.error;
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
  } finally {
    closeSync(fd);
  }
}
function inventory(paths) {
  const entries = [];
  function visit(path) {
    const info = lstatSync(path, {
      throwIfNoEntry: true
    });
    assert.ok(!info.isSymbolicLink());
    if (info.isDirectory()) {
      for (const name of readdirSync(path).sort()) visit(join(path, name));
    } else {
      assert.ok(info.isFile());
      assert.ok(!path.includes('..') && !path.startsWith('/'));
      entries.push({
        path,
        bytes: info.size,
        sha256: sha(readFileSync(path)),
        mode: info.mode & 0o777
      });
    }
  }
  for (const path of paths) visit(path);
  return entries;
}
export function seal(kind, paths) {
  const directory = '.release-transfer/' + kind;
  mkdirSync(directory, {
    recursive: true
  });
  const bundle = directory + '/bundle.tar';
  tarTo(bundle, ['-cf', '-', ...paths]);
  const bytes = readFileSync(bundle);
  const record = {
    schema: 'release-stage-transfer/v1',
    kind,
    identity: identity(),
    producerJob: process.env.GITHUB_JOB,
    complete: true,
    status: 0,
    signal: null,
    sha256: sha(bytes),
    bytes: bytes.length,
    inventory: inventory(paths)
  };
  writeFileSync(directory + '/descriptor.json', JSON.stringify(record, null, 2) + '\n');
  output({
    bundle_sha: record.sha256,
    descriptor_sha: sha(readFileSync(directory + '/descriptor.json'))
  });
  return record;
}
// Check typed paths, complete inventory and entry types before extraction; inner tar preserves mode.
export function importBundle(kind) {
  const directory = '.release-transfer/' + kind;
  const descriptorBytes = readFileSync(directory + '/descriptor.json');
  assert.equal(sha(descriptorBytes), process.env.EXPECT_DESCRIPTOR, 'descriptor physical bytes differ');
  const record = JSON.parse(descriptorBytes);
  const expected = {
    ...identity(),
    kind,
    producerJob: process.env.EXPECT_PRODUCER,
    bundleSha256: process.env.EXPECT_BUNDLE
  };
  const bytes = readFileSync(directory + '/bundle.tar');
  verifyDescriptor(record, expected, bytes);
  const names = run('tar', ['-tf', directory + '/bundle.tar']).split('\n').filter(Boolean);
  for (const name of names) {
    assert.ok(!name.startsWith('/') && !name.split('/').includes('..'), 'unsafe tar name');
    assert.ok(name.startsWith('prepared-release/') || name.startsWith('rust/target/package/') || name.startsWith('slim-image-release-receipt.json'), 'unexpected tar inventory');
  }
  const types = run('tar', ['-tvf', directory + '/bundle.tar']).split('\n').filter(Boolean);
  for (const line of types) assert.ok(['-', 'd'].includes(line[0]), 'reject symlink/hardlink/special tar entry');
  assert.equal(new Set(record.inventory.map(item => item.path)).size, record.inventory.length);
  assert.deepEqual(names.filter(name => !name.endsWith('/')).sort(), record.inventory.map(item => item.path).sort(), 'archive inventory differs from complete descriptor');
  const allowed = kind === 'package' ? ['rust/target/package/'] : kind === 'binary' ? ['prepared-release/'] : ['prepared-release/', 'slim-image-release-receipt.json'];
  for (const name of names) assert.ok(allowed.some(prefix => name.startsWith(prefix)), 'wrong typed transfer inventory');
  run('tar', ['-xf', directory + '/bundle.tar']);
  assert.deepEqual(inventory([...new Set(record.inventory.map(item => item.path))]), record.inventory);
  return record;
}
async function binary() {
  await authority();
  mkdirSync('rust/target/release', {
    recursive: true
  });
  copyFileSync('prepared-release/binary/formal-ai', 'rust/target/release/formal-ai');
  chmodSync('rust/target/release/formal-ai', statSync('prepared-release/binary/formal-ai').mode & 0o777);
  const {
    validatePreparedBinary
  } = await import('./release-image-factory.mjs');
  const verified = validatePreparedBinary({
    cwd: process.cwd(),
    directory: resolve('prepared-release'),
    version: process.env.RELEASE_VERSION
  });
  const receipt = read('prepared-release/binary/native-executable-receipt.json');
  assert.equal(receipt.producer.attempt, String(process.env.GITHUB_RUN_ATTEMPT));
  assert.equal(receipt.producer.job, process.env.EXPECT_BINARY_PRODUCER);
  assert.equal(sha(receipt.producer.compiler), process.env.EXPECT_COMPILER);
  return verified;
}
function crateContents(path) {
  const names = run('tar', ['-tzf', path]).split('\n').filter(Boolean).filter(name => !name.endsWith('/')).sort();
  assert.equal(new Set(names).size, names.length);
  return names.map(name => {
    assert.ok(!name.startsWith('/') && !name.split('/').includes('..'));
    return {
      name,
      sha256: sha(run('tar', ['-xOzf', path, name], {
        encoding: null
      }))
    };
  });
}
async function execute(mode, kind) {
  if (mode === 'baseline') {
    protocol();
    const bytes = run('rustc', ['-vV']);
    const version = /^version\s*=\s*"([^"]+)"/m.exec(readFileSync('rust/Cargo.toml', 'utf8'))?.[1];
    assert.ok(version);
    output({
      source_sha: git(['rev-parse', 'HEAD']),
      source_tree: git(['rev-parse', 'HEAD^{tree}']),
      version,
      compiler_sha: sha(bytes),
      rust_version: /^release: (.+)$/m.exec(bytes)[1]
    });
    return;
  }
  if (mode === 'capture-source') {
    protocol();
    const compilerBytes = run('rustc', ['-vV']);
    const {
      validatePreparedSelection
    } = await import('./prepared-release-binary.mjs');
    const selected = validatePreparedSelection(process.cwd(), process.env.RELEASE_VERSION, process.env);
    mkdirSync('.release-transfer/source', {
      recursive: true
    });
    const fd = openSync('.release-transfer/source/source.tar', 'w');
    try {
      const result = spawnSync('git', ['archive', '--format=tar', 'HEAD'], {
        stdio: ['ignore', fd, 'pipe'],
        encoding: 'utf8',
        timeout: 120000
      });
      assert.equal(result.status, 0, result.stderr);
      assert.equal(result.signal, null);
    } finally {
      closeSync(fd);
    }
    const sourceBytes = readFileSync('.release-transfer/source/source.tar');
    const record = {
      schema: 'release-stage-source/v1',
      producerJob: process.env.GITHUB_JOB,
      identity: {
        ...identity(),
        source: selected.head,
        tree: selected.tree,
        compilerSha256: sha(compilerBytes)
      },
      selection: selected,
      archiveSha256: sha(sourceBytes),
      compiler: compilerBytes
    };
    writeFileSync('.release-transfer/source/descriptor.json', JSON.stringify(record, null, 2) + '\n');
    output({
      source_sha: selected.head,
      source_tree: selected.tree,
      version: selected.version,
      compiler_sha: sha(compilerBytes),
      rust_version: /^release: (.+)$/m.exec(compilerBytes)[1],
      source_archive_sha: record.archiveSha256,
      source_descriptor_sha: sha(readFileSync('.release-transfer/source/descriptor.json'))
    });
    return;
  }
  if (mode === 'verify-source') {
    await authority();
    const bytes = readFileSync('.release-transfer/source/descriptor.json');
    assert.equal(sha(bytes), process.env.EXPECT_SOURCE_DESCRIPTOR);
    const record = JSON.parse(bytes);
    const archive = readFileSync('.release-transfer/source/source.tar');
    verifySourceDescriptor(record, {
      ...identity(),
      producerJob: process.env.EXPECT_SOURCE_PRODUCER,
      archiveSha256: process.env.EXPECT_SOURCE_ARCHIVE
    }, archive, run('git', ['archive', '--format=tar', 'HEAD'], {
      encoding: null
    }));
    return;
  }
  if (mode === 'capture' && kind === 'binary') {
    await binary();
    seal(kind, ['prepared-release']);
    return;
  }
  if (mode === 'capture' && kind === 'package') {
    await authority();
    const path = 'rust/target/package/formal-ai-' + process.env.RELEASE_VERSION + '.crate';
    const contents = crateContents(path);
    writeFileSync('rust/target/package/staged-package-contents.json', JSON.stringify(contents) + '\n');
    seal(kind, [path, 'rust/target/package/staged-package-contents.json']);
    return;
  }
  if (mode === 'import') {
    await authority();
    importBundle(kind);
    if (kind === 'binary' || kind === 'images') await binary();
    if (kind === 'package') assert.deepEqual(crateContents('rust/target/package/formal-ai-' + process.env.RELEASE_VERSION + '.crate'), read('rust/target/package/staged-package-contents.json'));
    if (kind === 'images') await imageReceipts();
    return;
  }
  if (mode === 'verify-registry') {
    await authority();
    const response = await fetch('https://static.crates.io/crates/formal-ai/formal-ai-' + process.env.RELEASE_VERSION + '.crate', {
      redirect: 'error',
      signal: AbortSignal.timeout(120000)
    });
    assert.equal(response.status, 200);
    const bytes = Buffer.from(await response.arrayBuffer());
    assert.ok(bytes.length > 0 && bytes.length < 64 * 1024 * 1024);
    writeFileSync('.release-transfer/registry.crate', bytes);
    assert.deepEqual(crateContents('.release-transfer/registry.crate'), read('rust/target/package/staged-package-contents.json'), 'actual registry source differs from real verified package');
    return;
  }
  if (mode === 'capture' && kind === 'images') {
    await binary();
    await imageReceipts();
    seal(kind, ['prepared-release', 'slim-image-release-receipt.json', 'slim-image-release-receipt.json.sha256']);
    return;
  }
  if (mode === 'guard-create') {
    await authority();
    await binary();
    await imageReceipts();
    return;
  }
  throw Error('unsupported mode/kind');
}
async function imageReceipts() {
  const build = read('prepared-release/prepared-build.json');
  const full = read('prepared-release/full-image.json');
  assert.equal(full.schema, 'prepared-release-image/v1');
  assert.equal(full.preparedBuildSha256, sha(readFileSync('prepared-release/prepared-build.json')));
  assert.equal(full.sourceSelectionSha256, build.sourceSelectionSha256);
  assert.equal(full.image, process.env.GHCR_IMAGE + '@' + process.env.EXPECT_FULL_DIGEST);
  assert.equal(full.expected.revision, process.env.EXPECT_SOURCE);
  assert.equal(full.expected.version, process.env.RELEASE_VERSION);
  const slim = read('slim-image-release-receipt.json');
  assert.equal(slim.source_commit, process.env.EXPECT_SOURCE);
  assert.equal(slim.package_version, process.env.RELEASE_VERSION);
  assert.equal(slim.executable.sha256, build.executable.sha256);
  assert.equal(slim.producer_run, String(process.env.GITHUB_RUN_ID));
  assert.equal(slim.producer_attempt, String(process.env.GITHUB_RUN_ATTEMPT));
  assert.match(slim.image, new RegExp('^' + process.env.GHCR_IMAGE.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + '@sha256:[a-f0-9]{64}$'));
  assert.equal(readFileSync('slim-image-release-receipt.json.sha256', 'utf8').trim().split(/\s+/)[0], sha(readFileSync('slim-image-release-receipt.json')));
  if (process.env.MIRROR_ENABLED === 'true') {
    const mirror = read('prepared-release/dockerhub-image.json');
    assert.equal(mirror.schema, 'prepared-release-image-mirror/v1');
    assert.equal(mirror.sourceImage, full.image);
    assert.match(mirror.destinationImage, /@sha256:[a-f0-9]{64}$/);
    assert.equal(mirror.expected.revision, process.env.EXPECT_SOURCE);
    assert.equal(mirror.expected.version, process.env.RELEASE_VERSION);
  }
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await execute(...process.argv.slice(2));
