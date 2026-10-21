import test from 'node:test';
import assert from 'node:assert/strict';
import { applicableWorkflows,
 authenticatePullRequest,
 ciAdmission,
 clientAdmission,
 admissionPolicy,
 requireQualifiedDelivery,
 requireHistoricalTask } from '../../../scripts/lib/hive-delivery-admission.mjs';
const source = 'a'.repeat(40),
  other = 'b'.repeat(40),
  policyHash = 'c'.repeat(64),
  inputHash = 'd'.repeat(64);
const repository = 'owner/repo';
const request = () => ({
  repository,
  number: 7,
  requestedHead: source,
  pullRequest: {
    number: 7,
    state: 'open',
    merged: false,
    head: {
      sha: source,
      ref: 'feature',
      repo: {
        full_name: repository
      }
    },
    base: {
      sha: other,
      ref: 'main',
      repo: {
        full_name: repository
      }
    }
  }
});
const run = (path, id = 1) => ({
  id,
  run_attempt: 1,
  path,
  event: 'pull_request',
  head_sha: source,
  status: 'completed',
  conclusion: 'success'
});
const options = () => ({
  repository,
  scope: 'pr-7',
  tool: 'claude',
  source,
  policyHash,
  inputHash,
  currentRun: 9,
  history: [],
  complete: true
});
const attempt = () => ({
  repository,
  scope: 'pr-7',
  tool: 'claude',
  policyHash,
  inputHash,
  run: 8,
  attempt: 1,
  testedHead: other,
  authenticated: true,
  state: 'failure',
  producerConclusion: 'failure',
  cleanupComplete: true
});
test('authenticate exact current same-repository PR before checkout, reject stale, fork and self trigger', () => {
  assert.equal(authenticatePullRequest(request()).scope, 'pr-7');
  for (const change of [value => value.pullRequest.head.sha = other,
 value => value.pullRequest.head.repo.full_name = 'fork/repo',
 value => value.pullRequest.state = 'closed']) {
    const value = request();
    change(value);
    assert.throws(() => authenticatePullRequest(value));
  }
  const value = request();
  value.trigger = {
    repository: {
      full_name: repository
    },
    head_repository: {
      full_name: repository
    },
    head_sha: source,
    status: 'completed',
    path: '.github/workflows/hive-original-delivery.yml'
  };
  assert.throws(() => authenticatePullRequest(value));
});
test('missing applicable workflow cannot be mistaken for all green; unrelated domain status is not CI', () => {
  const expected = ['.github/workflows/main.yml', '.github/workflows/security.yml'];
  assert.throws(() => ciAdmission({
    source,
    expected,
    runs: [run(expected[0])],
    complete: true
  }));
  assert.equal(ciAdmission({
    source,
    expected,
    runs: expected.map((path, index) => run(path, index + 1)),
    complete: true
  }).length, 2);
});
test('pending, failed, wrong-source and partial CI snapshots refuse, successful newer retry supersedes failed old attempt', () => {
  const path = '.github/workflows/main.yml',
    value = {
      source,
      expected: [path],
      runs: [run(path)],
      complete: true
    };
  for (const patch of [{
    status: 'in_progress'
  }, {
    conclusion: 'failure'
  }, {
    head_sha: other
  }]) assert.throws(() => ciAdmission({
    ...value,
    runs: [{
      ...run(path),
      ...patch
    }]
  }));
  assert.throws(() => ciAdmission({
    ...value,
    complete: false
  }));
  assert.equal(ciAdmission({
    ...value,
    runs: [{
      ...run(path),
      conclusion: 'failure'
    }, run(path, 2)]
  }).length, 1);
});
test('source-derived applicability includes missing security workflow, honors ordered negative paths and refuses unknown syntax', () => {
  const workflows = [{
    path: '.github/workflows/main.yml',
    document: {
      on: {
        pull_request: null
      }
    }
  }, {
    path: '.github/workflows/security.yml',
    document: {
      on: {
        pull_request: {
          branches: ['main'],
          paths: ['rust/**', '!rust/docs/**']
        }
      }
    }
  }];
  assert.equal(applicableWorkflows(workflows, {
    event: 'pull_request',
    baseBranch: 'main',
    files: ['rust/src/main.rs']
  }).length, 2);
  assert.equal(applicableWorkflows(workflows, {
    event: 'pull_request',
    baseBranch: 'main',
    files: ['rust/docs/a.md']
  }).length, 1);
  workflows[1].document.on.pull_request.paths = ['rust/{src,tests}/**'];
  assert.throws(() => applicableWorkflows(workflows, {
    event: 'pull_request',
    baseBranch: 'main',
    files: ['rust/src/a.rs']
  }));
});
test('authenticated PR success is reused with actual tested head, never relabelled as current execution', () => {
  const success = {
    ...attempt(),
    state: 'success',
    producerConclusion: 'success',
    originalOraclePassed: true,
    qualification: admissionPolicy,
    imageDigest: 'sha256:' + 'e'.repeat(64),
    acceptance: {
      schema: 'formal-ai-issue-1364-delivery/v1',
      inputSha256: inputHash,
      tool: 'claude',
      source: other,
      imageDigest: 'sha256:' + 'e'.repeat(64),
      taskPRhead: 'f'.repeat(40),
      firstPass: {
        exitCode: 0,
        clean: true,
        verifierPassed: true,
        workflowCiGreen: true,
        allObligations: true
      },
      secondPass: {
        exitCode: 0,
        clean: true,
        noWrite: true,
        noCommit: true
      },
      complete: true
    }
  };
  assert.deepEqual(clientAdmission({
    ...options(),
    history: [success]
  }), {
    action: 'reuse',
    testedHead: other,
    run: 8,
    attempt: 1
  });
  assert.equal(clientAdmission({
    ...options(),
    history: [{
      ...success,
      qualification: 'legacy-three-green'
    }]
  }).action, 'refuse');
});
test('only failed and fully cleaned attempts retry, at most twice; partial cancelled or active attempts never repeat writes', () => {
  assert.equal(clientAdmission(options()).ordinal, 1);
  assert.equal(clientAdmission({
    ...options(),
    history: [attempt()]
  }).ordinal, 2);
  assert.equal(clientAdmission({
    ...options(),
    history: [attempt(), {
      ...attempt(),
      run: 7
    }]
  }).reason, 'finite-attempt-budget');
  for (const patch of [{
    state: 'claimed'
  }, {
    producerConclusion: 'cancelled'
  }, {
    cleanupComplete: false
  }]) assert.equal(clientAdmission({
    ...options(),
    history: [{
      ...attempt(),
      ...patch
    }]
  }).action, 'refuse');
});
test('duplicate triggers and duplicate or unauthenticated receipts do not produce a second effect grant', () => {
  assert.equal(clientAdmission({
    ...options(),
    history: [{
      ...attempt(),
      run: 9
    }]
  }).reason, 'already-claimed');
  assert.throws(() => clientAdmission({
    ...options(),
    history: [attempt(), attempt()]
  }));
  assert.throws(() => clientAdmission({
    ...options(),
    history: [{
      ...attempt(),
      authenticated: false
    }]
  }));
  assert.throws(() => clientAdmission({
    ...options(),
    complete: false
  }));
});
test('qualified success requires exact image/source/input, complete original obligations and a genuinely unchanged second pass', () => {
  const valid = {
    schema: 'formal-ai-issue-1364-delivery/v1',
    inputSha256: inputHash,
    tool: 'claude',
    source: other,
    imageDigest: 'sha256:' + 'e'.repeat(64),
    taskPRhead: 'f'.repeat(40),
    firstPass: {
      exitCode: 0,
      clean: true,
      verifierPassed: true,
      workflowCiGreen: true,
      allObligations: true
    },
    secondPass: {
      exitCode: 0,
      clean: true,
      noWrite: true,
      noCommit: true
    },
    complete: true
  };
  const expected = {
    inputHash,
    tool: 'claude',
    source: other,
    imageDigest: 'sha256:' + 'e'.repeat(64)
  };
  requireQualifiedDelivery(valid, expected);
  for (const mutate of [value => value.source = source,
 value => value.imageDigest = 'sha256:' + '0'.repeat(64),
 value => value.inputSha256 = policyHash,
 value => value.complete = 'true',
 value => value.firstPass.verifierPassed = false,
 value => value.firstPass.workflowCiGreen = false,
 value => value.firstPass.allObligations = false,
 value => value.firstPass.clean = false,
 value => value.secondPass.noWrite = false,
 value => value.secondPass.noCommit = false,
 value => value.secondPass.exitCode = 1]) {
    const changed = structuredClone(valid);
    mutate(changed);
    assert.throws(() => requireQualifiedDelivery(changed, expected));
  }
});
import { createHash } from 'node:crypto';
test('historical checker mode binds exact complete input bytes, public task, PR, repository and actual branch', () => {
  const body = 'Implement a real program.\n',
    bodySha256 = createHash('sha256').update(body).digest('hex');
  const entry = {
    issueUrl: 'https://github.com/owner/task/issues/1',
    pullRequestUrl: 'https://github.com/owner/task/pull/2',
    branch: 'issue-1-original',
    body,
    bodySha256
  };
  const manifest = {
    schema: 'formal-ai-issue-1364-client-replay-inputs/v1',
    clients: {
      claude: entry
    }
  };
  const manifestBytes = Buffer.from(JSON.stringify(manifest));
  const value = {
    manifest,
    manifestBytes,
    inputHash: createHash('sha256').update(manifestBytes).digest('hex'),
    tool: 'claude',
    evidence: {
      issueUrl: entry.issueUrl,
      pullRequestUrl: entry.pullRequestUrl
    },
    repository: 'owner/task',
    branch: entry.branch
  };
  assert.equal(requireHistoricalTask(value), entry);
  for (const change of [value => value.branch = 'new-branch',
 value => value.repository = 'owner/other',
 value => value.evidence.issueUrl = 'https://github.com/owner/task/issues/3',
 value => value.evidence.pullRequestUrl = 'https://github.com/owner/task/pull/4',
 value => value.manifest.clients.claude.body += 'changed']) {
    const candidate = structuredClone(value);
    change(candidate);
    assert.throws(() => requireHistoricalTask(candidate));
  }
});
import { observeAdmission } from '../../../scripts/admit-hive-original-delivery.mjs';
function hostedReader({
  pending = false,
  move = false
} = {}) {
  const pull = request().pullRequest;
  pull.changed_files = 1;
  const workflow = {
      on: {
        pull_request: null
      }
    },
    bytes = Buffer.from(JSON.stringify(workflow));
  let observations = 0;
  return {
    read: async endpoint => {
      if (endpoint.endsWith('/pulls/7')) {
        observations++;
        return structuredClone(move && observations === 3 ? {
          ...pull,
          head: {
            ...pull.head,
            sha: other
          }
        } : pull);
      }
      if (endpoint.includes('/contents/.github/workflows?')) return [{
        name: 'main.yml',
        type: 'file',
        path: '.github/workflows/main.yml'
      }];
      if (endpoint.includes('/contents/.github/workflows/main.yml?')) return {
        type: 'file',
        sha: createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex'),
        encoding: 'base64',
        content: bytes.toString('base64'),
        size: bytes.length
      };
      throw new Error('unexpected read ' + endpoint);
    },
    pages: async endpoint => {
      if (endpoint.includes('/pulls/7/files')) return [{
        filename: 'rust/src/main.rs'
      }];
      if (endpoint.includes('/actions/runs?')) return [{
        ...run('.github/workflows/main.yml'),
        repository: {
          full_name: repository
        },
        head_repository: {
          full_name: repository
        },
        status: pending ? 'in_progress' : 'completed'
      }];
      throw new Error('unexpected pages ' + endpoint);
    }
  };
}
test('hosted observer derives actual missing-run obligations and never admits partial CI or a moving head', async () => {
  const options = {
    repository,
    eventName: 'workflow_dispatch',
    event: {
      inputs: {
        'pull-request': '7'
      }
    }
  };
  assert.equal((await observeAdmission({
    ...options,
    reader: hostedReader()
  })).admitted, true);
  assert.equal((await observeAdmission({
    ...options,
    reader: hostedReader({
      pending: true
    })
  })).admitted, false);
  await assert.rejects(observeAdmission({
    ...options,
    reader: hostedReader({
      move: true
    })
  }));
});
import { authenticateJournalProducer } from '../../../scripts/lib/hive-delivery-journal.mjs';
import { requirePreparedVersionDelta } from '../../../scripts/lib/hive-delivery-release.mjs';
import { resultForAttempt } from '../../../scripts/hive-delivery-attempt.mjs';
test('journal producer binds protocol identity separately from selected product and refuses foreign or ambiguous custody', () => {
  const claim = {
    repository,
    qualification: admissionPolicy,
    run: 31,
    attempt: 1,
    protocolCommit: source,
    testedHead: other
  };
  const metadata = {
    workflow_run: {
      id: 31,
      head_sha: source
    }
  };
  const producer = {
    id: 31,
    run_attempt: 1,
    repository: {
      full_name: repository
    },
    head_repository: {
      full_name: repository
    },
    path: '.github/workflows/hive-original-delivery.yml',
    event: 'workflow_run',
    head_sha: source
  };
  authenticateJournalProducer(producer, claim, metadata, repository);
  for (const change of [value => value.head_sha = other,
 value => value.repository.full_name = 'foreign/repo',
 value => value.run_attempt = 2,
 value => value.path = '.github/workflows/other.yml']) {
    const changed = structuredClone(producer);
    change(changed);
    assert.throws(() => authenticateJournalProducer(changed, claim, metadata, repository));
  }
});
test('prepared release inherits checked parent only for exact version and declared metadata delta', () => {
  const policy = {
    witness: 'governed.v8',
    ledger: 'ledger.lino',
    mapping: 'map.tsv',
    archive: 'docs/changelog',
    surfaces: ['README.md']
  };
  const value = {
    merge: source,
    source: other,
    commit: {
      sha: other,
      parents: [{
        sha: source
      }]
    },
    files: [{
      filename: 'rust/Cargo.toml',
      status: 'modified'
    }],
    beforeCargo: '[package]\nversion = "0.1.0"\n',
    afterCargo: '[package]\nversion = "0.2.0"\n',
    beforeLock: '[[package]]\nname = "formal-ai"\nversion = "0.1.0"\n',
    afterLock: '[[package]]\nname = "formal-ai"\nversion = "0.2.0"\n',
    version: '0.2.0',
    policy
  };
  assert.equal(requirePreparedVersionDelta(value).ciSource, source);
  for (const change of [value => value.commit.parents.push({
    sha: other
  }), value => value.files.push({
    filename: 'rust/src/lib.rs',
    status: 'modified'
  }), value => value.afterLock += 'unrelated dependency\n', value => value.afterCargo += 'description = "new"\n', value => value.files.push({
    filename: 'changelog.d/unrelated.md',
    status: 'added'
  })]) {
    const changed = structuredClone(value);
    change(changed);
    assert.throws(() => requirePreparedVersionDelta(changed));
  }
});
test('an unsuccessful independent verification never manufactures a qualified receipt', () => {
  const claim = {
    testedHead: source,
    tool: 'claude'
  };
  assert.equal(resultForAttempt(claim, {
    outcome: 'failure'
  }).state, 'uncertain');
  assert.equal(resultForAttempt(claim, {
    outcome: 'failure'
  }).originalOraclePassed, false);
  assert.throws(() => resultForAttempt(claim, {
    outcome: 'success',
    independent: {},
    acceptance: {}
  }));
});
import { sourceBytes } from '../../../scripts/lib/hive-delivery-source.mjs';
import { observeEligibleAdmission } from '../../../scripts/admit-hive-original-delivery.mjs';
test('source custody uses the actual Git object header, independently known hello blob and rejects substituted metadata', async () => {
  const bytes = Buffer.from('hello\n'),
    sha = 'ce013625030ba8dba906f756967f9e9ca394464a';
  const document = {
    type: 'file',
    encoding: 'base64',
    size: bytes.length,
    content: bytes.toString('base64'),
    sha
  };
  assert.deepEqual(await sourceBytes({
    read: async () => document
  }, repository, source, 'README.md'), bytes);
  await assert.rejects(sourceBytes({
    read: async () => ({
      ...document,
      sha: '0'.repeat(40)
    })
  }, repository, source, 'README.md'));
});
test('unrelated or unsuccessful CI completion never selects an old published release', async () => {
  for (const [workflow, conclusion] of [['.github/workflows/release.yml', 'success'], ['.github/workflows/verify-release-downloads.yml', 'failure']]) {
    const calls = [];
    const reader = {
      read: async endpoint => {
        calls.push(endpoint);
        assert.equal(endpoint, 'repos/' + repository + '/actions/runs/12');
        return {
          id: 12,
          repository: {
            full_name: repository
          },
          head_repository: {
            full_name: repository
          },
          head_sha: source,
          path: workflow,
          status: 'completed',
          conclusion
        };
      },
      pages: async endpoint => {
        assert.equal(endpoint, 'repos/' + repository + '/commits/' + source + '/pulls');
        return [];
      }
    };
    const result = await observeEligibleAdmission({
      repository,
      eventName: 'workflow_run',
      event: {
        workflow_run: {
          id: 12
        }
      },
      reader
    });
    assert.equal(result.admitted, false);
    assert.ok(calls.every(value => !value.includes('/releases/') && !value.includes('/actions/workflows/')));
  }
});
test('released runtime keeps its immutable full reference while acceptance requires the bare independently bound digest', () => {
  const digest = 'sha256:' + 'e'.repeat(64);
  const runtime = {
    image: 'ghcr.io/link-assistant/formal-ai@' + digest,
    source
  };
  const receipt = {
    schema: 'formal-ai-issue-1364-delivery/v1',
    inputSha256: inputHash,
    tool: 'claude',
    source,
    imageDigest: digest,
    taskPRhead: other,
    firstPass: {
      exitCode: 0,
      clean: true,
      verifierPassed: true,
      workflowCiGreen: true,
      allObligations: true
    },
    secondPass: {
      exitCode: 0,
      clean: true,
      noWrite: true,
      noCommit: true
    },
    complete: true
  };
  const expected = {
    inputHash,
    tool: 'claude',
    source,
    imageDigest: digest
  };
  requireQualifiedDelivery(receipt, expected);
  assert.throws(() => requireQualifiedDelivery({
    ...receipt,
    imageDigest: runtime.image
  }, expected));
  assert.throws(() => requireQualifiedDelivery(receipt, {
    ...expected,
    imageDigest: 'sha256:' + 'f'.repeat(64)
  }));
  assert.equal(runtime.image, 'ghcr.io/link-assistant/formal-ai@' + digest);
});
import { policyIdentity, policyRoots } from '../../../scripts/lib/hive-delivery-source.mjs';
test('direct privacy export and collector source roots participate in reusable policy identity', async () => {
  const ownedPaths = ['scripts/lib/issue-1364-public-evidence.mjs', 'scripts/collect-test-replays.mjs'];
  for (const name of ownedPaths) assert.ok(policyRoots.includes(name));
  const body = 'actual public task\n';
  const task = {
    body,
    bodySha256: createHash('sha256').update(body).digest('hex'),
    issueUrl: 'https://github.com/owner/task/issues/1',
    pullRequestUrl: 'https://github.com/owner/task/pull/2',
    branch: 'issue-1-source'
  };
  const canonical = Buffer.from(JSON.stringify({
    schema: 'formal-ai-issue-1364-client-replay-inputs/v1',
    clients: {
      claude: task,
      agent: task,
      codex: task
    }
  }));
  function fixture(changed) {
    return {
      read: async endpoint => {
        const prefix = 'repos/' + repository + '/contents/';
        assert.ok(endpoint.startsWith(prefix));
        assert.ok(endpoint.endsWith('?ref=' + source));
        const name = endpoint.slice(prefix.length, -('?ref=' + source).length);
        const bytes = name === 'data/benchmarks/issue-1364-client-replay-inputs.json' ? canonical : Buffer.from(name === changed ? 'changed direct guard\n' : 'original owned guard\n');
        assert.ok(name === 'data/benchmarks/issue-1364-client-replay-inputs.json' || policyRoots.includes(name));
        return {
          type: 'file',
          encoding: 'base64',
          size: bytes.length,
          content: bytes.toString('base64'),
          sha: createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex')
        };
      }
    };
  }
  const original = await policyIdentity(fixture(null), repository, source);
  for (const name of ownedPaths) {
    const changed = await policyIdentity(fixture(name), repository, source);
    assert.equal(changed.inputHash, original.inputHash);
    assert.notEqual(changed.policyHash, original.policyHash);
    assert.notEqual(changed.roots.find(row => row.path === name).sha256, original.roots.find(row => row.path === name).sha256);
  }
});
import { taskCredentialTarget, requireTaskCredentialAccess } from '../../../scripts/lib/hive-delivery-credentials.mjs';
test('canonical task bytes determine external credential scope and wrong owner or input substitution refuses', () => {
  const body = 'original public task\n',
    task = {
      body,
      bodySha256: createHash('sha256').update(body).digest('hex'),
      issueUrl: 'https://github.com/owner/task/issues/1',
      pullRequestUrl: 'https://github.com/owner/task/pull/2',
      branch: 'issue-1-original'
    };
  const bytes = Buffer.from(JSON.stringify({
    schema: 'formal-ai-issue-1364-client-replay-inputs/v1',
    clients: {
      claude: task
    }
  }));
  const options = {
    tool: 'claude',
    inputHash: createHash('sha256').update(bytes).digest('hex')
  };
  assert.deepEqual(taskCredentialTarget(bytes, options), {
    owner: 'owner',
    name: 'task',
    repository: 'owner/task'
  });
  assert.throws(() => taskCredentialTarget(bytes, {
    ...options,
    inputHash: '0'.repeat(64)
  }));
  assert.throws(() => taskCredentialTarget(bytes, {
    ...options,
    tool: 'codex'
  }));
});
test('target permission preflight refuses wrong-scoped app and known insufficient classic tokens without inventing unknown scope proof', () => {
  const target = {
    owner: 'owner',
    name: 'task',
    repository: 'owner/task'
  };
  const repository = {
    id: 31,
    name: 'task',
    owner: {
      login: 'owner'
    },
    full_name: 'owner/task',
    permissions: {
      push: true
    },
    private: false
  };
  const installation = {
    total_count: 1,
    repositories: [{
      id: 31,
      full_name: 'owner/task'
    }]
  };
  assert.equal(requireTaskCredentialAccess(target, {
    repository,
    layer: 'app',
    installation
  }).workflowWrite, 'requested-app-workflows-write');
  assert.equal(requireTaskCredentialAccess(target, {
    repository,
    layer: 'token',
    scopes: ['repo', 'workflow']
  }).workflowWrite, 'classic-workflow-scope');
  const unknown = requireTaskCredentialAccess(target, {
    repository,
    layer: 'token'
  });
  assert.equal(unknown.workflowWrite, 'unverified-token-scope');
  assert.equal(unknown.deliveryVerified, false);
  for (const proof of [{
    repository: {
      ...repository,
      full_name: 'owner/current'
    },
    layer: 'token'
  }, {
    repository: {
      ...repository,
      permissions: {
        push: false
      }
    },
    layer: 'token'
  }, {
    repository,
    layer: 'token',
    scopes: ['repo']
  }, {
    repository,
    layer: 'app',
    installation: {
      total_count: 1,
      repositories: [{
        id: 32,
        full_name: 'other/current'
      }]
    }
  }, {
    repository,
    layer: 'default'
  }]) assert.throws(() => requireTaskCredentialAccess(target, proof));
});


import fs from 'node:fs';
import YAML from 'yaml';
// GitHub's context availability table excludes runner from job env; step env permits it.
// https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability
function jobEnvironmentContextProblems(workflow) {
  const problems = [];
  for (const [name, job] of Object.entries(workflow.jobs)) {
    for (const [key, value] of Object.entries(job.env ?? {})) {
      const expressions = String(value).match(/\$\{\{[\s\S]*?\}\}/gu) ?? [];
      if (expressions.some(expression => /\b(?:runner|steps|job|env)\s*[.[]/u.test(expression))) {
        problems.push(name + '.' + key);
      }
    }
  }
  return problems;
}
test('Hive workflow job environments use only contexts available before runner assignment', () => {
  for (const name of ['hive-original-delivery', 'hive-original-delivery-finalize']) {
    const file = new URL('../../../.github/workflows/' + name + '.yml', import.meta.url);
    assert.deepEqual(jobEnvironmentContextProblems(YAML.parse(fs.readFileSync(file, 'utf8'))), [], name);
  }
  const invalid = {jobs: {example: {env: {FILE: "${{ format('{0}/claim.json', runner.temp) }}"}}}};
  assert.deepEqual(jobEnvironmentContextProblems(invalid), ['example.FILE']);
});

test('hash-bound public task metadata admits without duplicating body bytes in knowledge data', () => {
  const body = 'Independent original task requirements.\n';
  const entry = {issueUrl:'https://github.com/public/task/issues/1',
    pullRequestUrl:'https://github.com/public/task/pull/2',branch:'issue-1-original',
    bodySha256:createHash('sha256').update(body).digest('hex')};
  const manifest = {schema:'formal-ai-issue-1364-client-replay-inputs/v1',clients:{codex:entry}};
  const manifestBytes = Buffer.from(JSON.stringify(manifest));
  const value = {manifest,manifestBytes,inputHash:createHash('sha256').update(manifestBytes).digest('hex'),
    tool:'codex',evidence:{issueUrl:entry.issueUrl,pullRequestUrl:entry.pullRequestUrl},
    repository:'public/task',branch:entry.branch};
  assert.equal(requireHistoricalTask(value),entry);
  for (const invalid of [null,'','NOT-A-DIGEST','A'.repeat(64)]) {
    const candidate = structuredClone(value);candidate.manifest.clients.codex.bodySha256 = invalid;
    candidate.manifestBytes = Buffer.from(JSON.stringify(candidate.manifest));
    candidate.inputHash = createHash('sha256').update(candidate.manifestBytes).digest('hex');
    assert.throws(() => requireHistoricalTask(candidate), {code:'ERR_ASSERTION'});
  }
});
test('published replay coordinates bind the original issue fixture without a second stored body', async () => {
  const {readFileSync} = await import('node:fs');
  const manifest = JSON.parse(readFileSync(new URL('../../../data/benchmarks/issue-1364-client-replay-inputs.json',import.meta.url),'utf8'));
  const issues = JSON.parse(readFileSync(new URL('../fixtures/issue-1364-hello-world-issues.json',import.meta.url),'utf8'));
  assert.equal(Object.keys(manifest.clients).length,3);
  for (const entry of Object.values(manifest.clients)) {
    assert.equal(Object.hasOwn(entry,'body'),false);
    const matches = issues.filter(issue => issue.html_url === entry.issueUrl);assert.equal(matches.length,1);
    assert.equal(createHash('sha256').update(matches[0].body).digest('hex'),entry.bodySha256);
  }
});


test('source-event relocation preserves exact readonly producer semantics without independent triggers', async () => {
  const fs = await import('node:fs');
  const read = name => fs.readFileSync(new URL('../../../' + name, import.meta.url), 'utf8');
  const called = '.github/workflows/hive-source-provenance.yml';
  const expectedCaller = {uses:'./' + called};
  const retained = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
  for (const name of ['.github/workflows/release.yml', retained + 'original-release-workflow.yml',
      retained + 'candidate-release-caller.yml']) {
    assert.deepEqual(YAML.parse(read(name)).jobs['hive-source-provenance'], expectedCaller);
  }
  const reusable = YAML.parse(read(called));
  assert.deepEqual(Object.keys(reusable.on), ['workflow_call']);
  assert.deepEqual(reusable.permissions, {contents:'read'});
  assert.deepEqual(reusable.concurrency, {
    group:'${{ github.workflow }}-${{ github.ref }}-hive-source-provenance',
    'cancel-in-progress':"${{ github.ref != 'refs/heads/main' }}"
  });
  assert.deepEqual(Object.keys(reusable.jobs), ['hive-source-provenance']);
  const expectedJob = {
  "name": "Hive source event provenance",
  "runs-on": "ubuntu-24.04",
  "timeout-minutes": 3,
  "permissions": {
    "contents": "read"
  },
  "steps": [
    {
      "uses": "actions/checkout@v7",
      "with": {
        "persist-credentials": false
      }
    },
    {
      "uses": "actions/setup-node@v7",
      "with": {
        "node-version": 24
      }
    },
    {
      "name": "Preserve minimal original source coordinates",
      "env": {
        "HIVE_SOURCE_EVENT_OUTPUT": "${{ runner.temp }}/hive-source-event/source-event.json"
      },
      "run": "node scripts/record-hive-source-event.mjs"
    },
    {
      "uses": "actions/upload-artifact@v7",
      "with": {
        "name": "hive-source-event-${{ github.run_attempt }}",
        "path": "${{ runner.temp }}/hive-source-event/source-event.json",
        "retention-days": 90,
        "if-no-files-found": "error"
      }
    }
  ]
};
  assert.deepEqual(reusable.jobs['hive-source-provenance'], expectedJob);
  assert.ok(policyRoots.includes(called), 'called producer source participates in policy identity');
  assert.equal(read(retained + 'original-release-workflow.yml').trimEnd().split('\n').length, 1500);
});
