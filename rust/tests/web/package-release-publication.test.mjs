// PR #1188: package releases follow automated GitHub releases and remain dry on PRs.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { resolvePackageRelease } from '../../../scripts/resolve-package-release.mjs';
import { publicationDecision } from '../../../scripts/publish-browser-engine.mjs';

const HEAD = 'a'.repeat(40);
const CHILD = 'b'.repeat(40);
const RELEASE = { tag_name: 'v1.2.3', draft: false, prerelease: false };
const ENVIRONMENT = {
  EVENT: 'workflow_run', REPOSITORY: 'owner/repository', RUN_BRANCH: 'main',
  RUN_REPOSITORY: 'owner/repository', RUN_HEAD: HEAD, RUN_CONCLUSION: 'success',
};
const readGithub = (commit, releases = [RELEASE]) => (endpoint) =>
  endpoint.includes('/commits/') ? commit : endpoint.includes('/tags/') ? RELEASE : releases;

test('the release child commit of a completed main run resolves for publication', () => {
  assert.deepEqual(resolvePackageRelease(ENVIRONMENT, readGithub({ sha: CHILD, parents: [{ sha: HEAD }] })),
    { tag: RELEASE.tag_name, publish: true, build: true });
});

test('an exact-head release resolves too, including when a later pipeline job failed', () => {
  assert.equal(resolvePackageRelease({ ...ENVIRONMENT, RUN_CONCLUSION: 'failure' },
    readGithub({ sha: HEAD, parents: [] })).publish, true);
});

test('a main run cannot publish an unrelated latest release', () => {
  assert.deepEqual(resolvePackageRelease(ENVIRONMENT, readGithub({ sha: CHILD, parents: [] })),
    { tag: '', publish: false, build: false });
});

test('untrusted runs and cancelled runs do not make any API request', () => {
  for (const extra of [{ RUN_BRANCH: 'feature' }, { RUN_REPOSITORY: 'fork/repository' },
    { RUN_CONCLUSION: 'cancelled' }, { RUN_CONCLUSION: 'skipped' }]) {
    assert.equal(resolvePackageRelease({ ...ENVIRONMENT, ...extra }, () => { throw new Error('unexpected API'); }).build, false);
  }
});

test('PRs and default dispatches build without publishing or querying releases', () => {
  for (const event of ['pull_request', 'workflow_dispatch']) {
    assert.deepEqual(resolvePackageRelease({ EVENT: event, REPOSITORY: 'owner/repository' },
      () => { throw new Error('unexpected API'); }), { tag: '', publish: false, build: true });
  }
});

test('manual publication requires main and a published stable tag', () => {
  const environment = { EVENT: 'workflow_dispatch', REPOSITORY: 'owner/repository', PUBLISH: 'true',
    REFERENCE: 'refs/heads/main', INPUT_TAG: RELEASE.tag_name };
  assert.equal(resolvePackageRelease(environment, readGithub({})).tag, RELEASE.tag_name);
  assert.throws(() => resolvePackageRelease({ ...environment, REFERENCE: 'refs/heads/feature' }, readGithub({})), /requires main/);
  assert.throws(() => resolvePackageRelease({ ...environment, INPUT_TAG: 'feature' }, readGithub({})), /Release tag/);
  assert.throws(() => resolvePackageRelease(environment, () => ({ ...RELEASE, draft: true })), /No published/);
});

test('drafts and prereleases never resolve; a completion requires the full commit', () => {
  assert.equal(resolvePackageRelease(ENVIRONMENT, readGithub({}, [{ ...RELEASE, prerelease: true }])).build, false);
  assert.throws(() => resolvePackageRelease({ ...ENVIRONMENT, RUN_HEAD: 'abc' }, readGithub({})), /full commit/);
});

test('a repeated npm publication verifies the immutable tarball bytes', () => {
  assert.deepEqual(publicationDecision('package@1.2.3', 'sha512-example', () => '"sha512-example"'), { publish: false });
  assert.throws(() => publicationDecision('package@1.2.3', 'sha512-new', () => '"sha512-old"'), /integrity differs/);
});

test('npm publishes only on a real missing-version response; transport/auth errors fail', () => {
  const failure = (code) => () => { throw Object.assign(new Error('registry error'), { stdout: JSON.stringify({ error: { code } }) }); };
  assert.deepEqual(publicationDecision('package@1.2.3', 'sha512-example', failure('E404')), { publish: true });
  for (const code of ['E401', 'E403', 'E429', 'ECONNRESET']) {
    assert.throws(() => publicationDecision('package@1.2.3', 'sha512-example', failure(code)), /registry error/);
  }
});

test('both package workflows admit automated releases, check PRs, and attach installable artifacts', () => {
  for (const name of ['publish-engine', 'publish-vscode']) {
    const workflow = readFileSync(new URL('../../../.github/workflows/' + name + '.yml', import.meta.url), 'utf8');
    assert.match(workflow, /workflow_run:\n\s+workflows: \["CI\/CD Pipeline"\]\n\s+types: \[completed\]\n\s+branches: \[main\]/);
    assert.match(workflow, /pull_request:/);
    assert.match(workflow, /head_repository.full_name == github.repository/);
    assert.match(workflow, /run: node scripts\/resolve-package-release.mjs/);
    assert.match(workflow, /git checkout --detach "refs\/tags\/\$PACKAGE_RELEASE_TAG"/);
    assert.match(workflow, /if: needs.resolve.outputs.publish == 'true'[\s\S]+run: gh release upload/);
    assert.doesNotMatch(workflow, /ref:.*workflow_run.head_sha/);
  }
});

test('each release path publishes and verifies slim before creating its GitHub release', () => {
  const workflow = readFileSync(new URL('../../../.github/workflows/release.yml', import.meta.url), 'utf8');
  for (const section of ['auto-release', 'manual-release']) {
    const body = workflow.slice(workflow.indexOf('  ' + section + ':'));
    const end = body.slice(1).search(/\n  [a-z][a-z-]*:/);
    const job = end < 0 ? body : body.slice(0, end + 1);
    assert.ok(job.indexOf('Publish and verify the slim GHCR sidecar') > 0);
    assert.ok(job.indexOf('Publish and verify the slim GHCR sidecar') < job.indexOf('Create GitHub Release'));
    assert.match(job, /uses: \.\/\.github\/actions\/publish-slim-image/);
  }
  assert.match(workflow, /docker-slim:[\s\S]+uses: \.\/\.github\/workflows\/docker-slim.yml/);
  assert.match(workflow, /needs: \[detect-changes, changelog, docker-build, docker-slim,/);
  const action = readFileSync(new URL('../../../.github/actions/publish-slim-image/action.yml', import.meta.url), 'utf8');
  assert.match(action, /file: Dockerfile.slim/);
  assert.match(action, /build-args: BINARY_SOURCE=prebuilt/);
  assert.match(action, /verify-formal-ai-slim/);
  assert.match(action, /2147483648/);
});
