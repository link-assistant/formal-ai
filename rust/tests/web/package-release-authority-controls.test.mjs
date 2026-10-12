import assert from 'node:assert/strict';
import test from 'node:test';
import { resolvePackageRelease, qualifiesCompletedRun } from '../../../scripts/resolve-package-release.mjs';
// Typed provider controls exercise source authority; they do not attest a publication.
test('authenticated run metadata preserves published-release policy and rejects foreign or incomplete authority', () => {
  const repository = 'owner/repository',
    headCommit = 'a'.repeat(40),
    releaseCommit = 'b'.repeat(40);
  const environment = {
    EVENT: 'workflow_run',
    REPOSITORY: repository,
    RUN_BRANCH: 'main',
    RUN_REPOSITORY: repository,
    RUN_HEAD: headCommit,
    RUN_CONCLUSION: 'success',
    RUN_ID: '42',
    RUN_ATTEMPT: '1',
    RUN_WORKFLOW_ID: '7'
  };
  const baseRun = {
    id: 42,
    run_attempt: 1,
    status: 'completed',
    conclusion: 'success',
    head_sha: headCommit,
    head_branch: 'main',
    repository: {
      full_name: repository
    },
    head_repository: {
      full_name: repository
    },
    workflow_id: 7,
    path: '.github/workflows/release.yml'
  };
  const baseWorkflow = {
    id: 7,
    path: '.github/workflows/release.yml'
  };
  const release = {
    tag_name: 'v1.2.3',
    draft: false,
    prerelease: false
  };
  function provider(run = baseRun, workflow = baseWorkflow, commit = {
    sha: releaseCommit,
    parents: [{
      sha: headCommit
    }]
  }) {
    const calls = [];
    const read = endpoint => {
      calls.push(endpoint);
      if (endpoint.endsWith('/actions/runs/42')) return run;
      if (endpoint.endsWith('/actions/workflows/release.yml')) return workflow;
      if (endpoint.includes('/commits/')) return commit;
      if (endpoint.includes('/releases/tags/')) return release;
      if (endpoint.includes('/releases?')) return [release];
      throw new Error('unknown endpoint ' + endpoint);
    };
    return {
      read,
      calls
    };
  }
  let controlCount = 0;
  for (const conclusion of ['success', 'failure', 'neutral', 'timed_out', 'action_required', 'stale']) {
    const responseProvider = provider({
      ...baseRun,
      conclusion
    });
    assert.deepEqual(resolvePackageRelease({
      ...environment,
      RUN_CONCLUSION: conclusion
    }, responseProvider.read), {
      tag: release.tag_name,
      publish: true,
      build: true
    });
    assert.deepEqual(responseProvider.calls.slice(0, 2), ['repos/' + repository + '/actions/runs/42', 'repos/' + repository + '/actions/workflows/release.yml']);
    controlCount++;
  }
  for (const mutation of [runMetadata => runMetadata.id = 43,
     runMetadata => runMetadata.run_attempt = 2,
     runMetadata => runMetadata.status = 'in_progress',
     runMetadata => runMetadata.conclusion = 'cancelled',
     runMetadata => runMetadata.conclusion = 'skipped',
     runMetadata => runMetadata.conclusion = 'invented',
     runMetadata => runMetadata.conclusion = 'failure',
     runMetadata => runMetadata.head_sha = 'c'.repeat(40),
     runMetadata => runMetadata.head_branch = 'feature',
     runMetadata => runMetadata.repository.full_name = 'foreign/repo',
     runMetadata => runMetadata.head_repository.full_name = 'fork/repo',
     runMetadata => delete runMetadata.head_repository,
     runMetadata => runMetadata.workflow_id = 8,
     runMetadata => runMetadata.path = '.github/workflows/foreign.yml']) {
    const run = structuredClone(baseRun);
    mutation(run);
    const responseProvider = provider(run);
    assert.deepEqual(resolvePackageRelease(environment, responseProvider.read), {
      tag: '',
      publish: false,
      build: false
    });
    assert.equal(responseProvider.calls.some(path => path.includes('/releases')), false);
    controlCount++;
  }
  for (const workflow of [{
    id: 8,
    path: baseWorkflow.path
  }, {
    id: 7,
    path: '.github/workflows/foreign.yml'
  }, {
    id: '7',
    path: baseWorkflow.path
  }]) {
    const responseProvider = provider(baseRun, workflow);
    assert.equal(resolvePackageRelease(environment, responseProvider.read).publish, false);
    controlCount++;
  }
  for (const extra of [{
    RUN_ID: ''
  }, {
    RUN_ID: '42/other'
  }, {
    RUN_ID: '9007199254740993'
  }, {
    RUN_ATTEMPT: ''
  }, {
    RUN_WORKFLOW_ID: ''
  }, {
    RUN_BRANCH: 'feature'
  }, {
    RUN_REPOSITORY: 'fork/repo'
  }, {
    RUN_CONCLUSION: 'cancelled'
  }, {
    RUN_CONCLUSION: 'skipped'
  }]) {
    assert.equal(resolvePackageRelease({
      ...environment,
      ...extra
    }, () => {
      throw new Error('no metadata request permitted');
    }).publish, false);
    controlCount++;
  }
  for (const actualConclusions of ['failure', 'success']) {
    const responseProvider = provider({
      ...baseRun,
      conclusion: actualConclusions
    });
    assert.equal(resolvePackageRelease({
      ...environment,
      RUN_CONCLUSION: actualConclusions === 'success' ? 'failure' : 'success'
    }, responseProvider.read).publish, false);
    controlCount++;
  }
  for (const failPath of ['/actions/runs/42', '/actions/workflows/release.yml']) {
    const responseProvider = provider();
    assert.throws(() => resolvePackageRelease(environment, path => {
      if (path.endsWith(failPath)) throw new Error('actual transport refused');
      return responseProvider.read(path);
    }));
    controlCount++;
  }
  const unrelatedRelease = provider(baseRun, baseWorkflow, {
    sha: releaseCommit,
    parents: [{
      sha: 'c'.repeat(40)
    }]
  });
  assert.equal(resolvePackageRelease(environment, unrelatedRelease.read).publish, false);
  controlCount++;
  const exactHead = provider(baseRun, baseWorkflow, {
    sha: headCommit,
    parents: []
  });
  assert.equal(resolvePackageRelease(environment, exactHead.read).publish, true);
  controlCount++;
  for (const event of ['pull_request', 'workflow_dispatch']) {
    assert.deepEqual(resolvePackageRelease({
      EVENT: event,
      REPOSITORY: repository
    }, () => {
      throw new Error('unexpected API');
    }), {
      tag: '',
      publish: false,
      build: true
    });
    controlCount++;
  }
  const manualPublication = provider();
  assert.equal(resolvePackageRelease({
    EVENT: 'workflow_dispatch',
    REPOSITORY: repository,
    PUBLISH: 'true',
    REFERENCE: 'refs/heads/main',
    INPUT_TAG: 'v1.2.3'
  }, manualPublication.read).publish, true);
  assert.equal(manualPublication.calls.some(path => path.includes('/actions/')), false);
  controlCount++;
  assert.throws(() => resolvePackageRelease({
    ...environment,
    RUN_HEAD: 'abc'
  }, provider().read), /full commit/);
  controlCount++;
  assert.equal(controlCount, 42);
});
