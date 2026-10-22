// Same-run handoffs retain custody and source identity; they grant no authorship credit.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
export function fileDigest(file) {
  const descriptor = fs.openSync(file, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
  const hash = createHash('sha256'),
    chunk = Buffer.alloc(65536);
  try {
    const before = fs.fstatSync(descriptor);
    assert.ok(before.isFile(), 'regular handoff member required');
    let count;
    while ((count = fs.readSync(descriptor, chunk, 0, chunk.length, null)) !== 0) hash.update(chunk.subarray(0, count));
    const after = fs.fstatSync(descriptor);
    assert.equal(after.size, before.size);
    assert.equal(after.mtimeMs, before.mtimeMs, 'handoff changed while hashing');
    return hash.digest('hex');
  } finally {
    fs.closeSync(descriptor);
  }
}
export function workflowRunIdentity(environment) {
  assert.equal(environment.GITHUB_ACTIONS, 'true', 'hosted handoffs only');
  for (const key of ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT']) assert.match(environment[key] ?? '', /^[1-9]\d*$/u);
  assert.match(environment.GITHUB_REPOSITORY ?? '', /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.ok(environment.GITHUB_WORKFLOW_REF?.startsWith(environment.GITHUB_REPOSITORY + '/.github/workflows/hive-original-delivery.yml@'),
 'original workflow identity required');
  assert.match(environment.GITHUB_SHA ?? '', /^[a-f0-9]{40}$/u);
  return {
    repository: environment.GITHUB_REPOSITORY,
    run: environment.GITHUB_RUN_ID,
    attempt: environment.GITHUB_RUN_ATTEMPT,
    workflow: environment.GITHUB_WORKFLOW_REF,
    eventCommit: environment.GITHUB_SHA
  };
}
export function sourceBinaryIdentity({
  root = process.cwd(),
  environment = process.env
} = {}) {
  const git = args => execFileSync('git', args, {
    cwd: root,
    encoding: 'utf8',
    timeout: 30000
  }).trim();
  const identity = {
    source: git(['rev-parse', 'HEAD']),
    sourceTree: git(['rev-parse', 'HEAD^{tree}']),
    binarySha256: fileDigest(path.join(root, 'rust/target/release/formal-ai')),
    workflowRun: workflowRunIdentity(environment)
  };
  if (environment.HIVE_ADMISSION_SOURCE) {
    assert.match(environment.HIVE_ADMISSION_SOURCE, /^[a-f0-9]{40}$/u);
    assert.equal(identity.source, environment.HIVE_ADMISSION_SOURCE, 'authenticated selected checkout');
  }
  if (environment.HIVE_PUBLISHED_RUNTIME_FILE) {
    const selection = JSON.parse(fs.readFileSync(environment.HIVE_PUBLISHED_RUNTIME_FILE));
    assert.equal(selection.admitted, true);
    assert.equal(selection.selected.source, identity.source);
    assert.equal(selection.producer.run, environment.GITHUB_RUN_ID);
    assert.equal(selection.producer.attempt, environment.GITHUB_RUN_ATTEMPT);
    const runtime = selection.publishedRuntime;
    assert.match(runtime.image, /^ghcr\.io\/link-assistant\/formal-ai@sha256:[a-f0-9]{64}$/u);
    assert.equal(runtime.source, identity.source);
    assert.equal(runtime.binarySha256, identity.binarySha256);
    identity.publishedRuntime = runtime;
  }
  return identity;
}
export function requireSourceBinaryIdentity(identity, expected, inputs) {
  for (const key of ['source', 'sourceTree', 'binarySha256', 'workflowRun']) assert.deepEqual(identity[key], expected[key], 'same-source/run handoff ' + key);
  assert.equal(identity.hiveCommit, inputs.hiveCommit);
  if (expected.publishedRuntime) {
    assert.deepEqual(identity.publishedRuntime, expected.publishedRuntime);
    assert.equal(identity.runtimeImage, expected.publishedRuntime.image);
  } else {
    assert.equal(identity.publishedRuntime, undefined);
    assert.equal(identity.runtimeImage, inputs.taskRuntimeImage);
  }
  assert.equal(identity.sameRunBinary, true);
  assert.equal(identity.actualTaskImageBinarySha256, expected.binarySha256);
  assert.equal(identity.authorshipCredit, 0);
  assert.deepEqual(identity.operationalEvidence.formalAiSource, {
    commit: expected.source,
    tree: expected.sourceTree
  });
}
export function requireImageBundle(manifest, {
  directory,
  expected,
  inputs
}) {
  assert.equal(manifest.schema, 'formal-ai-hive-image-bundle/v1');
  assert.match(manifest.imageId ?? '', /^sha256:[a-f0-9]{64}$/u);
  requireSourceBinaryIdentity(manifest.identity, expected, inputs);
  assert.equal(fileDigest(path.join(directory, 'candidate.tar.gz')), manifest.archiveSha256, 'same-run image archive digest');
  return manifest.identity;
}
export function evidenceInventory(directory) {
  assert.equal(fs.realpathSync(directory), path.resolve(directory), 'aliased handoff directory');
  const pending = [''],
    members = [];
  while (pending.length) {
    const parent = pending.pop();
    for (const entry of fs.readdirSync(path.join(directory, parent), {
      withFileTypes: true
    })) {
      // Match the original artifact contract: hidden paths are neither read nor published.
      if (entry.name.startsWith('.')) continue;
      const name = path.join(parent, entry.name);
      if (name === 'delivery-stage-manifest.json') continue;
      assert.equal(entry.isSymbolicLink(), false, 'evidence handoff symlink');
      if (entry.isDirectory()) {
        pending.push(name);
        continue;
      }
      assert.ok(entry.isFile(), 'regular evidence handoff required');
      const file = path.join(directory, name);
      members.push({
        path: name.split(path.sep).join('/'),
        bytes: fs.statSync(file).size,
        sha256: fileDigest(file)
      });
    }
  }
  assert.ok(members.length > 0, 'nonempty delivery evidence required');
  return members.sort((a, b) => a.path.localeCompare(b.path, 'en'));
}
export function sealDeliveryEvidence({
  directory,
  tool,
  expected,
  inputs
}) {
  assert.ok(Object.hasOwn(inputs.clients, tool), 'original client required');
  const identity = JSON.parse(fs.readFileSync(path.join(directory, 'source-binary-identity.json')));
  requireSourceBinaryIdentity(identity, expected, inputs);
  // Failed original harnesses are retained too; the independent oracle still decides success.
  const receipt = {
    schema: 'formal-ai-hive-delivery-stage/v1',
    tool,
    source: expected,
    members: evidenceInventory(directory),
    authorshipCredit: 0
  };
  fs.writeFileSync(path.join(directory, 'delivery-stage-manifest.json'), JSON.stringify(receipt, null, 2) + '\n', {
    flag: 'wx'
  });
  return receipt;
}
export function verifyDeliveryEvidence({
  directory,
  tool,
  expected,
  inputs
}) {
  const receipt = JSON.parse(fs.readFileSync(path.join(directory, 'delivery-stage-manifest.json')));
  assert.equal(receipt.schema, 'formal-ai-hive-delivery-stage/v1');
  assert.equal(receipt.tool, tool);
  assert.deepEqual(receipt.source, expected, 'same-run delivery source identity');
  assert.equal(receipt.authorshipCredit, 0);
  assert.deepEqual(evidenceInventory(directory), receipt.members, 'complete staged evidence byte membership');
  const identity = JSON.parse(fs.readFileSync(path.join(directory, 'source-binary-identity.json')));
  requireSourceBinaryIdentity(identity, expected, inputs);
  return receipt;
}
export function main() {
  const mode = process.argv[2];
  assert.ok(['--seal-evidence', '--verify-evidence'].includes(mode));
  const inputs = JSON.parse(fs.readFileSync('data/benchmarks/hive-original-delivery-inputs.json'));
  const options = {
    directory: process.env.HIVE_EVIDENCE_PUBLISH_DIRECTORY ?? process.env.E2E_LOG_DIR,
    tool: process.env.E2E_TOOL,
    expected: sourceBinaryIdentity(),
    inputs
  };
  const result = mode === '--seal-evidence' ? sealDeliveryEvidence(options) : verifyDeliveryEvidence(options);
  console.log(JSON.stringify({
    schema: result.schema,
    tool: result.tool,
    members: result.members.length,
    authorshipCredit: 0
  }));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
