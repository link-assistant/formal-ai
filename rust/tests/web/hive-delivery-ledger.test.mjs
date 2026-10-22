import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createDeliveryClaim,
 publishDeliveryResult,
 readDeliveryLedger,
 publicLedgerClaim,
 githubLedgerWriter } from '../../../scripts/lib/hive-delivery-ledger.mjs';
import { observeClientHistory } from '../../../scripts/lib/hive-delivery-journal.mjs';
import { policyIdentity, policyRoots } from '../../../scripts/lib/hive-delivery-source.mjs';
import { clientAdmission, admissionPolicy } from '../../../scripts/lib/hive-delivery-admission.mjs';
const repository = 'owner/repo',
  source = 'a'.repeat(40),
  protocol = 'b'.repeat(40),
  digest = 'c'.repeat(64);
const gitHash = (kind, bytes) => createHash('sha1').update(kind + ' ' + bytes.length + '\0').update(bytes).digest('hex');
async function fixture() {
  const refs = new Map(),
    trees = new Map(),
    blobs = new Map(),
    commits = new Map(),
    writes = [];
  let proofAvailable = true,
    publisherConclusion = 'success',
    unknownRef = false,
    unknownPatch = false,
    beforePatch = null;
  const task = {
    body: 'public fixture\n',
    issueUrl: 'https://github.com/owner/task/issues/1',
    pullRequestUrl: 'https://github.com/owner/task/pull/2',
    branch: 'issue-1-original'
  };
  task.bodySha256 = createHash('sha256').update(task.body).digest('hex');
  const canonical = Buffer.from(JSON.stringify({
    schema: 'formal-ai-issue-1364-client-replay-inputs/v1',
    clients: {
      claude: task,
      agent: task,
      codex: task
    }
  }));
  const reader = {
    read: async endpoint => {
      const prefix = 'repos/' + repository + '/';
      assert.ok(endpoint.startsWith(prefix));
      const relative = endpoint.slice(prefix.length);
      if (relative.startsWith('git/matching-refs/')) {
        const ref = 'refs/' + relative.slice('git/matching-refs/'.length);
        return refs.has(ref) ? [{
          ref,
          object: {
            type: 'commit',
            sha: refs.get(ref)
          }
        }] : [];
      }
      for (const [kind, values] of [['commits', commits], ['trees', trees], ['blobs', blobs]]) if (relative.startsWith('git/' + kind + '/')) {
        const value = values.get(relative.slice(('git/' + kind + '/').length));
        assert.ok(value, 'missing fixture ' + kind);
        return structuredClone(value);
      }
      if (relative === 'git/ref/heads/main') return {
        object: {
          sha: protocol
        }
      };
      if (relative.startsWith('compare/')) return {
        status: 'identical'
      };
      if (relative.startsWith('actions/runs/')) {
        assert.ok(proofAvailable, 'producer proof unavailable');
        const match = /^actions\/runs\/(\d+)\/attempts\/(\d+)$/u.exec(relative);
        assert.ok(match);
        return {
          id: Number(match[1]),
          run_attempt: Number(match[2]),
          repository: {
            full_name: repository
          },
          head_repository: {
            full_name: repository
          },
          path: '.github/workflows/hive-original-delivery.yml',
          event: 'workflow_run',
          head_sha: protocol,
          status: 'completed',
          conclusion: 'success'
        };
      }
      if (relative.startsWith('contents/')) {
        const name = relative.slice('contents/'.length).split('?')[0];
        assert.ok(name === 'data/benchmarks/issue-1364-client-replay-inputs.json' || policyRoots.includes(name));
        const bytes = name === 'data/benchmarks/issue-1364-client-replay-inputs.json' ? canonical : Buffer.from('owned protocol ' + name + '\n');
        return {
          type: 'file',
          encoding: 'base64',
          size: bytes.length,
          content: bytes.toString('base64'),
          sha: gitHash('blob', bytes)
        };
      }
      throw new Error('unexpected fixture read ' + relative);
    },
    pages: async endpoint => {
      if (endpoint.endsWith('/jobs')) return [{
        name: 'Independent original Hive claude verification',
        status: 'completed',
        conclusion: 'success'
      }, {
        name: 'Persist durable Hive claude verification',
        status: 'completed',
        conclusion: publisherConclusion
      }];
      throw new Error('expired/deleted Actions artifacts must not be required');
    }
  };
  const writer = async (method, endpoint, body) => {
    writes.push({
      method,
      endpoint,
      body: structuredClone(body)
    });
    const relative = endpoint.slice(('repos/' + repository + '/git/').length);
    if (relative === 'trees') {
      assert.equal(body.tree.length, 1);
      const member = body.tree[0];
      assert.equal(member.path, 'binding.json');
      const bytes = Buffer.from(member.content),
        blob = gitHash('blob', bytes);
      const raw = Buffer.concat([Buffer.from('100644 binding.json\0'), Buffer.from(blob, 'hex')]),
        sha = gitHash('tree', raw);
      blobs.set(blob, {
        sha: blob,
        size: bytes.length,
        encoding: 'base64',
        content: bytes.toString('base64')
      });
      const value = {
        sha,
        truncated: false,
        tree: [{
          path: 'binding.json',
          mode: '100644',
          type: 'blob',
          sha: blob
        }]
      };
      trees.set(sha, value);
      return structuredClone(value);
    }
    if (relative === 'commits') {
      const sha = createHash('sha1').update(JSON.stringify(body)).digest('hex');
      const value = {
        sha,
        tree: {
          sha: body.tree
        },
        parents: body.parents.map(sha => ({
          sha
        })),
        message: body.message
      };
      commits.set(sha, value);
      return structuredClone(value);
    }
    if (relative === 'refs') {
      assert.equal(method, 'POST');
      assert.ok(!refs.has(body.ref), 'reference already exists');
      refs.set(body.ref, body.sha);
      if (unknownRef) throw new Error('write response unknown');
      return {
        ref: body.ref,
        object: {
          type: 'commit',
          sha: body.sha
        }
      };
    }
    if (relative.startsWith('refs/')) {
      assert.equal(method, 'PATCH');
      assert.equal(body.force, false);
      const ref = 'refs/' + relative.slice('refs/'.length);
      if (beforePatch) beforePatch(ref, refs);
      assert.equal(commits.get(body.sha).parents[0].sha, refs.get(ref), 'non-fast-forward rejected');
      refs.set(ref, body.sha);
      if (unknownPatch) throw new Error('update response unknown');
      return {
        ref,
        object: {
          type: 'commit',
          sha: body.sha
        }
      };
    }
    throw new Error('unexpected fixture write');
  };
  const identity = await policyIdentity(reader, repository, protocol);
  const claim = {
    schema: 'formal-ai-hive-attempt-claim/v1',
    qualification: admissionPolicy,
    repository,
    scope: 'pr-7',
    tool: 'claude',
    run: 101,
    attempt: 1,
    testedHead: source,
    protocolCommit: protocol,
    policyHash: identity.policyHash,
    inputHash: identity.inputHash,
    imageDigest: 'sha256:' + digest,
    actualTaskImageId: 'sha256:' + digest,
    binarySha256: digest,
    taskMode: 'historical-1364',
    ordinal: 1
  };
  const success = claim => ({
    ...claim,
    schema: 'formal-ai-hive-attempt-result/v1',
    state: 'success',
    originalOraclePassed: true,
    acceptance: {
      schema: 'formal-ai-issue-1364-delivery/v1',
      inputSha256: claim.inputHash,
      tool: claim.tool,
      source: claim.testedHead,
      imageDigest: claim.imageDigest,
      taskPRhead: 'd'.repeat(40),
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
  });
  return {
    reader,
    writer,
    claim,
    success,
    writes,
    refs,
    commits,
    trees,
    setProof: value => proofAvailable = value,
    setPublisher: value => publisherConclusion = value,
    setUnknownRef: () => unknownRef = true,
    setUnknownPatch: () => unknownPatch = true,
    setBeforePatch: value => beforePatch = value
  };
}
test('permanent qualified result survives expired/deleted Actions artifacts and retains the actual tested head', async () => {
  const value = await fixture();
  value.claim.ledger = await createDeliveryClaim(value);
  await publishDeliveryResult({
    ...value,
    result: value.success(value.claim)
  });
  const history = await observeClientHistory({
    ...value.claim,
    reader: value.reader
  });
  assert.equal(history[0].state, 'success');
  const decision = clientAdmission({
    ...value.claim,
    source: 'e'.repeat(40),
    currentRun: 303,
    history,
    complete: true
  });
  assert.equal(decision.action, 'reuse');
  assert.equal(decision.testedHead, source);
  const durable = await readDeliveryLedger({
    ...value.claim,
    reader: value.reader
  });
  assert.equal(durable.claimCommit, value.claim.ledger.claimCommit);
  assert.notEqual(durable.head, durable.claimCommit);
  assert.ok(value.writes.every(operation => !Object.hasOwn(operation.body, 'force') || operation.body.force === false));
});
test('expired producer proof quarantines an occupied permanent key instead of granting new effects', async () => {
  const value = await fixture();
  value.claim.ledger = await createDeliveryClaim(value);
  await publishDeliveryResult({
    ...value,
    result: value.success(value.claim)
  });
  value.setProof(false);
  await assert.rejects(observeClientHistory({
    ...value.claim,
    reader: value.reader
  }), /producer proof unavailable/u);
  const count = value.writes.length;
  await assert.rejects(createDeliveryClaim(value), /already consumed/u);
  assert.equal(value.writes.length, count);
});
test('a claim tombstone survives artifact deletion and cannot be reclassified as empty history', async () => {
  const value = await fixture();
  value.claim.ledger = await createDeliveryClaim(value);
  const history = await observeClientHistory({
    ...value.claim,
    reader: value.reader
  });
  assert.equal(history[0].state, 'claimed');
  assert.equal(clientAdmission({
    ...value.claim,
    source,
    currentRun: 303,
    history,
    complete: true
  }).action, 'refuse');
});
test('two racing writers acquire one permanent claim and never force or retry the losing create', async () => {
  const value = await fixture();
  const second = {
    ...value.claim,
    run: 102
  };
  const observed = await Promise.allSettled([createDeliveryClaim(value), createDeliveryClaim({
    ...value,
    claim: second
  })]);
  assert.equal(observed.filter(row => row.status === 'fulfilled').length, 1);
  assert.equal(observed.filter(row => row.status === 'rejected').length, 1);
  assert.equal(value.writes.filter(row => row.endpoint.endsWith('/git/refs')).length, 2);
  assert.equal(value.refs.size, 1);
});
test('unknown acknowledged claim response leaves a permanent tombstone and no automatic retry', async () => {
  const value = await fixture();
  value.setUnknownRef();
  await assert.rejects(createDeliveryClaim(value), /unknown/u);
  const count = value.writes.length;
  await assert.rejects(createDeliveryClaim(value), /already consumed/u);
  assert.equal(value.writes.length, count);
  const history = await observeClientHistory({
    ...value.claim,
    reader: value.reader
  });
  assert.equal(history[0].state, 'claimed');
});
test('result update rejects non-fast-forward races and unknown writes never repeat', async () => {
  const race = await fixture();
  race.claim.ledger = await createDeliveryClaim(race);
  race.setBeforePatch((ref, refs) => refs.set(ref, 'f'.repeat(40)));
  await assert.rejects(publishDeliveryResult({
    ...race,
    result: race.success(race.claim)
  }), /non-fast-forward/u);
  assert.equal(race.writes.filter(row => row.method === 'PATCH').length, 1);
  const unknown = await fixture();
  unknown.claim.ledger = await createDeliveryClaim(unknown);
  unknown.setUnknownPatch();
  await assert.rejects(publishDeliveryResult({
    ...unknown,
    result: unknown.success(unknown.claim)
  }), /unknown/u);
  unknown.setPublisher('failure');
  const history = await observeClientHistory({
    ...unknown.claim,
    reader: unknown.reader
  });
  assert.equal(history[0].state, 'claimed');
  const count = unknown.writes.length;
  await assert.rejects(publishDeliveryResult({
    ...unknown,
    result: unknown.success(unknown.claim)
  }), /unadvanced claim/u);
  assert.equal(unknown.writes.length, count);
});
test('source-free metadata rejects undeclared data and a timed-out transport is invoked only once', async () => {
  const value = await fixture();
  assert.throws(() => publicLedgerClaim({
    ...value.claim,
    privateCapture: 'never store this'
  }));
  assert.equal(value.writes.length, 0);
  let calls = 0;
  const writer = githubLedgerWriter({
    repository,
    transport: async () => {
      calls++;
      return {
        status: null,
        signal: 'SIGKILL',
        timed_out: true,
        cleanup_complete: false,
        stdout: Buffer.alloc(0)
      };
    }
  });
  await assert.rejects(writer('POST', 'repos/' + repository + '/git/refs', {
    ref: 'refs/heads/formal-ai-hive-ledger/example',
    sha: source
  }));
  assert.equal(calls, 1);
});
