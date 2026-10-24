// Canonical task identity controls credential scope; read-only preflight grants no delivery credit.
import assert from 'node:assert/strict';
import { requireHistoricalTask } from './hive-delivery-admission.mjs';
export function requireTaskCredentialConfiguration(environment = process.env) {
  const present = name => typeof environment[name] === 'string' && environment[name].trim().length > 0;
  const layer = present('AUTOMATION_APP_ID') && present('AUTOMATION_APP_PRIVATE_KEY')
    ? 'app' : present('AUTOMATION_TOKEN') ? 'token' : 'default';
  return {configured: layer !== 'default', layer, deliveryVerified: false};
}

/** Missing optional credentials select a run-owned fixture in the current repository. */
export function taskCredentialScope(canonical, {layer, repository}) {
  assert.ok(['app', 'token', 'default'].includes(layer), 'known credential layer required');
  if (layer !== 'default') return canonical;
  assert.match(repository ?? '', /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  const [owner, name] = repository.split('/');
  return {owner, name, repository};
}
export function taskCredentialTarget(manifestBytes, {
  inputHash,
  tool
}) {
  const manifest = JSON.parse(new TextDecoder('utf-8', {
    fatal: true
  }).decode(manifestBytes));
  const entry = manifest.clients?.[tool];
  assert.ok(entry, 'canonical client task missing');
  const match = /^https:\/\/github\.com\/([^/]+)\/([^/]+)\/issues\/[1-9]\d*$/u.exec(entry.issueUrl);
  assert.ok(match);
  const [, owner, name] = match;
  assert.match(owner, /^[A-Za-z0-9][A-Za-z0-9-]*$/u);
  assert.match(name, /^[A-Za-z0-9_.-]+$/u);
  const repository = owner + '/' + name;
  requireHistoricalTask({
    manifest,
    manifestBytes,
    inputHash,
    tool,
    evidence: {
      issueUrl: entry.issueUrl,
      pullRequestUrl: entry.pullRequestUrl
    },
    repository,
    branch: entry.branch
  });
  return {
    owner,
    name,
    repository
  };
}
export function requireTaskCredentialAccess(target, {
  repository,
  layer,
  installation = null,
  scopes = null,
  currentRepository = null
}) {
  assert.ok(['app', 'token', 'default'].includes(layer), 'known credential layer required');
  if (layer === 'default') assert.equal(target.repository, currentRepository, 'default token must stay in its own repository');
  assert.equal(repository.full_name, target.repository, 'credential target repository');
  assert.equal(repository.owner.login, target.owner, 'credential target owner');
  assert.equal(repository.name, target.name);
  assert.ok(Number.isSafeInteger(repository.id) && repository.id > 0);
  assert.equal(repository.permissions?.push, true, 'configured credential cannot push to the canonical task repository');
  assert.notEqual(repository.archived, true);
  assert.notEqual(repository.disabled, true);
  let workflowWrite = layer === 'default' ? 'staged-act' : 'unverified-token-scope';
  if (layer === 'app') {
    assert.equal(installation.total_count, 1, 'single canonical repository token required');
    assert.equal(installation.repositories.length, 1);
    assert.equal(installation.repositories[0].id, repository.id);
    assert.equal(installation.repositories[0].full_name, target.repository, 'app token scoped to wrong repository');
    workflowWrite = 'requested-app-workflows-write';
  } else if (layer === 'token' && scopes?.length) {
    assert.ok(scopes.includes('repo') || !repository.private && scopes.includes('public_repo'), 'classic credential lacks repository scope');
    assert.ok(scopes.includes('workflow'), 'classic credential explicitly lacks workflow update scope');
    workflowWrite = 'classic-workflow-scope';
  }
  return {
    repository: target.repository,
    layer,
    push: true,
    workflowWrite,
    deliveryVerified: false
  };
}
