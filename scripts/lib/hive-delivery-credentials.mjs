// Canonical task identity controls credential scope; read-only preflight grants no delivery credit.
import assert from 'node:assert/strict';
import { requireHistoricalTask } from './hive-delivery-admission.mjs';
export function requireTaskCredentialConfiguration(environment = process.env) {
  const present = name => typeof environment[name] === 'string' && environment[name].trim().length > 0;
  assert.ok(present('AUTOMATION_TOKEN') || present('AUTOMATION_APP_ID') && present('AUTOMATION_APP_PRIVATE_KEY'),
    'External Hive tasks require AUTOMATION_TOKEN or AUTOMATION_APP_ID plus AUTOMATION_APP_PRIVATE_KEY; '
    + 'the default repository token cannot authorize the canonical task repositories. No task effect was started.');
  return {configured: true, deliveryVerified: false};
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
  scopes = null
}) {
  assert.ok(['app', 'token'].includes(layer), 'configured task credential required; default repository token is insufficient');
  assert.equal(repository.full_name, target.repository, 'credential target repository');
  assert.equal(repository.owner.login, target.owner, 'credential target owner');
  assert.equal(repository.name, target.name);
  assert.ok(Number.isSafeInteger(repository.id) && repository.id > 0);
  assert.equal(repository.permissions?.push, true, 'configured credential cannot push to the canonical task repository');
  assert.notEqual(repository.archived, true);
  assert.notEqual(repository.disabled, true);
  let workflowWrite = 'unverified-token-scope';
  if (layer === 'app') {
    assert.equal(installation.total_count, 1, 'single canonical repository token required');
    assert.equal(installation.repositories.length, 1);
    assert.equal(installation.repositories[0].id, repository.id);
    assert.equal(installation.repositories[0].full_name, target.repository, 'app token scoped to wrong repository');
    workflowWrite = 'requested-app-workflows-write';
  } else if (scopes?.length) {
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
