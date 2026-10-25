// Source and policy identity are read from immutable Git objects through the bounded authenticated adapter.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { admissionPolicy, requireHistoricalTask } from './hive-delivery-admission.mjs';
const text = bytes => new TextDecoder('utf-8', {
  fatal: true
}).decode(bytes);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export const policyRoots = ['.github/workflows/hive-original-delivery.yml',
 '.github/actions/automation-token/action.yml',
 '.github/workflows/hive-source-provenance.yml',
 'scripts/admit-hive-original-delivery.mjs',
 'scripts/hive-delivery-attempt.mjs',
 'scripts/run-issue-1364-client-replay.mjs',
 'scripts/verify-released-server-lifecycle.mjs',
 'js/server/memory-store.mjs',
 'rust/tests/fixtures/memory/schema-1.lino',
 'rust/tests/fixtures/memory/schema-2.lino',
 'scripts/lib/issue-1364-replay-contract.mjs',
 'scripts/lib/issue-1364-public-evidence.mjs',
 'scripts/lib/issue-1364-command-result.mjs',
 'scripts/collect-test-replays.mjs',
 'scripts/collect-ci-failures.mjs',
 'scripts/lib/hive-delivery-admission.mjs',
 'scripts/lib/hive-delivery-credentials.mjs',
 'scripts/lib/hive-isolated-task.mjs',
 'scripts/lib/hive-delivery-source.mjs',
 'scripts/lib/hive-delivery-journal.mjs',
 'scripts/lib/hive-delivery-ledger.mjs',
 'scripts/lib/hive-delivery-release.mjs',
 'scripts/lib/hive-delivery-trees.mjs',
 'scripts/build-hive-original-delivery-image.mjs',
 'scripts/prepare-hive-original-delivery.mjs',
 'scripts/check-hive-original-delivery.mjs',
 'scripts/hive-original-delivery-oracle.py',
 'scripts/lib/hive-delivery-evidence.mjs',
 'scripts/lib/hive-delivery-bundle.mjs',
 'scripts/lib/hive-operational-evidence.mjs',
 'scripts/github-artifact-by-identifier.mjs',
 'scripts/qualified-data-artifact-profiles.mjs',
 'scripts/refresh-release-specification-witness.mjs',
 'scripts/verify-anonymous-image-manifest.mjs',
 'js/server/bounded-command.mjs',
 'data/benchmarks/hive-original-delivery-inputs.json',
 'bun.lock'];
export async function sourceBytes(reader, repository, commit, name) {
  assert.match(commit, /^[a-f0-9]{40}$/u);
  assert.match(name, /^[A-Za-z0-9_./-]+$/u);
  assert.ok(name.split('/').every(value => value && value !== '.' && value !== '..'));
  let value = await reader.read('repos/' + repository + '/contents/' + name + '?ref=' + commit);
  assert.equal(value.type, 'file');
  assert.ok(Number.isSafeInteger(value.size) && value.size > 0 && value.size <= 2 * 1024 * 1024, 'bounded source member');
  if (value.encoding !== 'base64') value = await reader.read('repos/' + repository + '/git/blobs/' + value.sha);
  assert.equal(value.encoding, 'base64');
  const bytes = Buffer.from(value.content, 'base64');
  assert.equal(bytes.length, value.size);
  assert.equal(createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex'), value.sha, 'source Git blob');
  return bytes;
}
/** Four immutable members per wave; output order remains the caller's source order. */
export async function orderedSourceMembers(reader, repository, commit, names) {
  assert.ok(Array.isArray(names) && names.length <= 128, 'bounded source member inventory');
  const members = [];
  for (let index = 0; index < names.length; index += 4) {
    const wave = names.slice(index, index + 4);
    members.push(...await Promise.all(wave.map(name => sourceBytes(reader, repository, commit, name))));
  }
  return members;
}
export async function readWorkflowPolicy(reader, repository, source) {
  const entries = await reader.read('repos/' + repository + '/contents/.github/workflows?ref=' + source);
  assert.ok(Array.isArray(entries) && entries.length <= 128);
  const {
    default: YAML
  } = await import('yaml');
  const selected = entries.filter(entry => /\.ya?ml$/u.test(entry.name));
  const members = await orderedSourceMembers(reader, repository, source, selected.map(entry => entry.path));
  return selected.map((entry, index) => ({
    path: entry.path,
    document: YAML.parse(text(members[index]))
  }));
}
const identityCache = new WeakMap();
/** Read only a literal path array from immutable source; historical code never executes. */
export function declaredPolicyRoots(bytes, name, required) {
  assert.ok(['policyRoots', 'cleanupRoots'].includes(name));
  assert.ok(bytes.length > 0 && bytes.length <= 131072);
  const pattern = new RegExp('^(?:export )?const ' + name + ' = (\\[[\\s\\S]*?\\]);', 'gmu');
  const declarations = [...text(bytes).matchAll(pattern)];
  assert.equal(declarations.length, 1, 'one source-declared policy root list required');
  const parts = declarations[0][1].slice(1, -1).split(',').map(value => value.trim());
  if (parts.at(-1) === '') parts.pop();
  assert.ok(parts.length > 0 && parts.length <= 128);
  const roots = parts.map(value => {
    const match = /^(['"])([A-Za-z0-9_./-]+)\1$/u.exec(value);
    assert.ok(match, 'policy roots must be literal relative paths');
    assert.ok(match[2].split('/').every(part => part && part !== '.' && part !== '..'));
    return match[2];
  });
  assert.equal(new Set(roots).size, roots.length, 'unique declared policy roots');
  assert.ok(roots.includes(required), 'policy must bind its own source');
  return Object.freeze(roots);
}
/** Policy root hashes identify this admission/oracle contract; they do not relabel product source as executed. */
export async function policyIdentity(reader, repository, protocolCommit) {
  let cached = identityCache.get(reader);
  if (!cached) {
    cached = new Map();
    identityCache.set(reader, cached);
  }
  const cacheKey = repository + '@' + protocolCommit;
  if (cached.has(cacheKey)) return cached.get(cacheKey);
  assert.ok(cached.size < 128);
  const pending = computePolicyIdentity(reader, repository, protocolCommit);
  cached.set(cacheKey, pending);
  return pending;
}
async function computePolicyIdentity(reader, repository, protocolCommit) {
  const policyPath = 'scripts/lib/hive-delivery-source.mjs';
  const declared = declaredPolicyRoots(await sourceBytes(reader, repository, protocolCommit, policyPath), 'policyRoots', policyPath);
  const inputBytes = await sourceBytes(reader, repository, protocolCommit, 'data/benchmarks/issue-1364-client-replay-inputs.json');
  const manifest = JSON.parse(text(inputBytes)),
    inputHash = hash(inputBytes);
  assert.deepEqual(Object.keys(manifest.clients).sort(), ['agent', 'claude', 'codex']);
  for (const tool of ['claude', 'agent', 'codex']) {
    const entry = manifest.clients[tool],
      repository = /^https:\/\/github\.com\/([^/]+\/[^/]+)\/issues\/[1-9]\d*$/u.exec(entry.issueUrl)?.[1];
    requireHistoricalTask({
      manifest,
      manifestBytes: inputBytes,
      inputHash,
      tool,
      evidence: {
        issueUrl: entry.issueUrl,
        pullRequestUrl: entry.pullRequestUrl
      },
      repository,
      branch: entry.branch
    });
  }
  const members = await orderedSourceMembers(reader, repository, protocolCommit, declared);
  const roots = declared.map((name, index) => ({
    path: name,
    sha256: hash(members[index])
  }));
  const identity = Object.freeze({
    inputHash,
    policyHash: hash(JSON.stringify({
      schema: admissionPolicy,
      inputHash,
      roots
    })),
    roots: Object.freeze(roots.map(Object.freeze)),
    protocolCommit
  });
  return identity;
}
