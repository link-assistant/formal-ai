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
/** Probe receive-pack authentication without sending objects or updating a ref. */
export async function probeTaskPushAccess(target, {environment, transport}) {
  assert.match(target.repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(environment.GITHUB_RUN_ID ?? '', /^[1-9][0-9]*$/u);
  assert.ok(['claude', 'codex', 'agent'].includes(environment.E2E_TOOL));
  assert.ok(typeof environment.GH_TOKEN === 'string' && environment.GH_TOKEN.length > 0);
  const result = await transport('git', ['-c', 'credential.helper=',
    '-c', 'credential.helper=!gh auth git-credential', 'push', '--dry-run',
    '--porcelain', '--no-verify', '--no-follow-tags',
    'https://github.com/' + target.repository + '.git',
    'HEAD:refs/heads/formal-ai-preflight/' + environment.GITHUB_RUN_ID + '-' + environment.E2E_TOOL], {
    env: {...environment, GIT_TERMINAL_PROMPT: '0'}, timeout: 20000,
    stdoutLimitBytes: 65536, stderrLimitBytes: 65536
  });
  const detail = result.stderr?.toString().replaceAll(environment.GH_TOKEN, '***').slice(-1500) ?? '';
  assert.ok(result.status === 0 && result.signal === null && !result.error &&
    !result.timed_out && result.cleanup_complete === true,
    'read-only Git receive-pack preflight failed: ' + detail);
  return {kind: 'git-push-dry-run/v1', repository: target.repository, success: true};
}
export function requireTaskCredentialAccess(target, {
  repository,
  layer,
  installation = null,
  scopes = null,
  currentRepository = null,
  pushProbe = null
}) {
  assert.ok(['app', 'token', 'default'].includes(layer), 'known credential layer required');
  if (layer === 'default') assert.equal(target.repository, currentRepository, 'default token must stay in its own repository');
  assert.equal(repository.full_name, target.repository, 'credential target repository');
  assert.equal(repository.owner.login, target.owner, 'credential target owner');
  assert.equal(repository.name, target.name);
  assert.ok(Number.isSafeInteger(repository.id) && repository.id > 0);
  // Installation-token repository metadata can report the bot's role as false
  // even when the job has contents:write. Require an actual bounded dry-run in
  // that case; neither metadata nor this preflight grants delivery credit.
  const metadataPush = repository.permissions?.push === true;
  const probedPush = pushProbe?.kind === 'git-push-dry-run/v1' &&
    pushProbe.repository === target.repository && pushProbe.success === true;
  assert.ok(metadataPush || probedPush, 'credential lacks checked push access to the task repository');
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
    pushAuthority: metadataPush ? 'repository-metadata' : 'read-only-git-receive-pack',
    workflowWrite,
    deliveryVerified: false
  };
}
