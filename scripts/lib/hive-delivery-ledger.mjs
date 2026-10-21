// Durable metadata refs must be retained. Actions artifact expiry never clears a consumed claim.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { boundedCommand } from '../../js/server/bounded-command.mjs';
import { admissionPolicy, requireQualifiedDelivery } from './hive-delivery-admission.mjs';
const claimKeys = ['schema',
 'qualification',
 'repository',
 'scope',
 'tool',
 'run',
 'attempt',
 'testedHead',
 'protocolCommit',
 'policyHash',
 'inputHash',
 'imageDigest',
 'actualTaskImageId',
 'binarySha256',
 'taskMode',
 'ordinal'];
const gitHash = (kind, bytes) => createHash('sha1').update(kind + ' ' + bytes.length + '\0').update(bytes).digest('hex');
const commit = value => assert.match(value ?? '', /^[a-f0-9]{40}$/u);
const sha256 = value => assert.match(value ?? '', /^[a-f0-9]{64}$/u);
function exactKeys(value, keys) {
  assert.deepEqual(Object.keys(value).sort(), [...keys].sort(), 'only declared public ledger metadata');
}
export function ledgerReference({
  repository,
  scope,
  tool,
  policyHash,
  inputHash
}) {
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(scope, /^(?:pr-[1-9]\d*|release-v\d+\.\d+\.\d+)$/u);
  assert.ok(['claude', 'agent', 'codex'].includes(tool));
  sha256(policyHash);
  sha256(inputHash);
  return 'refs/heads/formal-ai-hive-ledger/' + scope + '/' + policyHash + '/' + inputHash + '/' + tool;
}
export function publicLedgerClaim(value) {
  exactKeys(value, Object.hasOwn(value, 'ledger') ? [...claimKeys, 'ledger'] : claimKeys);
  const claim = Object.fromEntries(claimKeys.map(key => [key, value[key]]));
  assert.equal(claim.schema, 'formal-ai-hive-attempt-claim/v1');
  assert.equal(claim.qualification, admissionPolicy);
  ledgerReference(claim);
  for (const name of ['testedHead', 'protocolCommit']) commit(claim[name]);
  for (const name of ['policyHash', 'inputHash', 'binarySha256']) sha256(claim[name]);
  for (const name of ['imageDigest', 'actualTaskImageId']) assert.match(claim[name], /^sha256:[a-f0-9]{64}$/u);
  assert.ok(Number.isSafeInteger(claim.run) && claim.run > 0);
  assert.ok(Number.isSafeInteger(claim.attempt) && claim.attempt > 0 && claim.attempt <= 2);
  assert.equal(claim.taskMode, 'historical-1364');
  assert.ok([1, 2].includes(claim.ordinal));
  return claim;
}
function publicAcceptance(value, claim) {
  requireQualifiedDelivery(value, {
    inputHash: claim.inputHash,
    tool: claim.tool,
    source: claim.testedHead,
    imageDigest: claim.imageDigest
  });
  return {
    schema: value.schema,
    inputSha256: value.inputSha256,
    tool: value.tool,
    source: value.source,
    imageDigest: value.imageDigest,
    taskPRhead: value.taskPRhead,
    firstPass: {
      exitCode: value.firstPass.exitCode,
      clean: value.firstPass.clean,
      verifierPassed: value.firstPass.verifierPassed,
      workflowCiGreen: value.firstPass.workflowCiGreen,
      allObligations: value.firstPass.allObligations
    },
    secondPass: {
      exitCode: value.secondPass.exitCode,
      clean: value.secondPass.clean,
      noWrite: value.secondPass.noWrite,
      noCommit: value.secondPass.noCommit
    },
    complete: true
  };
}
export function publicLedgerResult(value, claim) {
  exactKeys(value, [...claimKeys, 'state', 'originalOraclePassed', 'acceptance', ...(Object.hasOwn(value, 'ledger') ? ['ledger'] : [])]);
  const projected = publicLedgerClaim({
    ...Object.fromEntries(claimKeys.map(key => [key, value[key]])),
    schema: 'formal-ai-hive-attempt-claim/v1'
  });
  assert.deepEqual(projected, claim);
  assert.equal(value.schema, 'formal-ai-hive-attempt-result/v1');
  assert.ok(['success', 'uncertain'].includes(value.state));
  assert.equal(value.originalOraclePassed, value.state === 'success');
  return {
    ...claim,
    schema: value.schema,
    state: value.state,
    originalOraclePassed: value.originalOraclePassed,
    acceptance: value.state === 'success' ? publicAcceptance(value.acceptance, claim) : null
  };
}
function bindingBytes(value) {
  const bytes = Buffer.from(JSON.stringify(value) + '\n');
  assert.ok(bytes.length > 0 && bytes.length <= 16384);
  return bytes;
}
function bindingTree(bytes) {
  const blob = gitHash('blob', bytes);
  const raw = Buffer.concat([Buffer.from('100644 binding.json\0'), Buffer.from(blob, 'hex')]);
  return {
    blob,
    tree: gitHash('tree', raw)
  };
}
async function readNode(reader, repository, sha) {
  commit(sha);
  const prefix = 'repos/' + repository + '/git/';
  const node = await reader.read(prefix + 'commits/' + sha);
  assert.equal(node.sha, sha);
  commit(node.tree.sha);
  assert.equal(node.message, 'Formal AI delivery ledger metadata v1');
  assert.ok(node.parents.length <= 1);
  const tree = await reader.read(prefix + 'trees/' + node.tree.sha);
  assert.equal(tree.sha, node.tree.sha);
  assert.equal(tree.truncated, false);
  assert.equal(tree.tree.length, 1);
  const member = tree.tree[0];
  assert.equal(member.path, 'binding.json');
  assert.equal(member.mode, '100644');
  assert.equal(member.type, 'blob');
  commit(member.sha);
  const blob = await reader.read(prefix + 'blobs/' + member.sha);
  assert.equal(blob.sha, member.sha);
  assert.equal(blob.encoding, 'base64');
  assert.ok(blob.size > 0 && blob.size <= 16384);
  const bytes = Buffer.from(blob.content, 'base64');
  assert.equal(bytes.length, blob.size);
  const expected = bindingTree(bytes);
  assert.equal(expected.blob, member.sha);
  assert.equal(expected.tree, node.tree.sha);
  const binding = JSON.parse(new TextDecoder('utf-8', {
    fatal: true
  }).decode(bytes));
  assert.equal(binding.schema, 'formal-ai-hive-ledger/v1');
  exactKeys(binding, binding.kind === 'claim' ? ['schema', 'kind', 'claim'] : ['schema', 'kind', 'claim', 'result']);
  assert.ok(['claim', 'result'].includes(binding.kind));
  const claim = publicLedgerClaim(binding.claim);
  assert.deepEqual(claim, binding.claim);
  if (binding.kind === 'result') assert.deepEqual(publicLedgerResult(binding.result, claim), binding.result);
  return {
    sha,
    node,
    binding
  };
}
export async function readDeliveryLedger({
  reader,
  repository,
  scope,
  tool,
  policyHash,
  inputHash
}) {
  const expected = {
      repository,
      scope,
      tool,
      policyHash,
      inputHash
    },
    ref = ledgerReference(expected);
  const matches = await reader.read('repos/' + repository + '/git/matching-refs/' + ref.slice(5));
  assert.ok(Array.isArray(matches) && matches.length <= 1, 'ambiguous durable claim namespace');
  if (matches.length === 0) return null;
  assert.equal(matches[0].ref, ref);
  assert.equal(matches[0].object.type, 'commit');
  const current = await readNode(reader, repository, matches[0].object.sha);
  for (const [key, value] of Object.entries(expected)) assert.equal(current.binding.claim[key], value);
  let claimNode = current;
  if (current.binding.kind === 'result') {
    assert.equal(current.node.parents.length, 1);
    claimNode = await readNode(reader, repository, current.node.parents[0].sha);
    assert.equal(claimNode.binding.kind, 'claim');
    assert.deepEqual(claimNode.binding.claim, current.binding.claim);
  }
  assert.equal(claimNode.node.parents.length, 0, 'claim must be an isolated metadata root');
  return {
    ref,
    head: current.sha,
    claimCommit: claimNode.sha,
    claim: claimNode.binding.claim,
    result: current.binding.kind === 'result' ? current.binding.result : null
  };
}
export function githubLedgerWriter({
  repository,
  environment = process.env,
  transport = boundedCommand
}) {
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  let actions = 0;
  const deadline = Date.now() + 60000;
  return async (method, endpoint, body) => {
    assert.ok(++actions <= 3, 'one bounded tree/commit/ref operation');
    assert.ok(method === 'POST' || method === 'PATCH');
    const prefix = 'repos/' + repository + '/git/';
    assert.ok(endpoint.startsWith(prefix));
    const relative = endpoint.slice(prefix.length);
    assert.ok(method === 'POST' ? ['trees', 'commits', 'refs'].includes(relative) : /^refs\/heads\/formal-ai-hive-ledger\/[A-Za-z0-9/.-]+$/u.test(relative));
    const input = Buffer.from(JSON.stringify(body));
    assert.ok(input.length <= 32768);
    const remaining = deadline - Date.now();
    assert.ok(remaining > 0);
    const result = await transport('gh', ['api', '--hostname', 'github.com', '--method', method, endpoint, '--input', '-'], {
      input,
      env: environment,
      timeout: Math.min(20000, remaining),
      stdoutLimitBytes: 1048576,
      stderrLimitBytes: 65536
    });
    assert.equal(result.status, 0, 'ledger write not acknowledged; stop without retry');
    assert.equal(result.signal, null);
    assert.ok(!result.error && !result.timed_out && result.cleanup_complete === true, 'ledger write outcome unknown; stop without retry');
    return JSON.parse(new TextDecoder('utf-8', {
      fatal: true
    }).decode(result.stdout));
  };
}
async function writeNode(writer, repository, binding, parents) {
  const bytes = bindingBytes(binding),
    expected = bindingTree(bytes),
    prefix = 'repos/' + repository + '/git/';
  const tree = await writer('POST', prefix + 'trees', {
    tree: [{
      path: 'binding.json',
      mode: '100644',
      type: 'blob',
      content: bytes.toString('utf8')
    }]
  });
  assert.equal(tree.sha, expected.tree);
  // Fixed storage attribution is synthetic; the binding's actual producer is authenticated through Actions APIs.
  const attribution = {
    name: 'Formal AI metadata storage',
    email: 'noreply@github.com',
    date: '2000-01-01T00:00:00Z'
  };
  const node = await writer('POST', prefix + 'commits', {
    message: 'Formal AI delivery ledger metadata v1',
    tree: expected.tree,
    parents,
    author: attribution,
    committer: attribution
  });
  commit(node.sha);
  assert.equal(node.tree.sha, expected.tree);
  assert.deepEqual(node.parents.map(value => value.sha), parents);
  return node.sha;
}
export async function createDeliveryClaim({
  reader,
  writer,
  claim
}) {
  const publicClaim = publicLedgerClaim(claim),
    repository = claim.repository,
    ref = ledgerReference(publicClaim);
  assert.equal(await readDeliveryLedger({
    ...publicClaim,
    reader
  }), null, 'durable claim already consumed');
  const sha = await writeNode(writer, repository, {
    schema: 'formal-ai-hive-ledger/v1',
    kind: 'claim',
    claim: publicClaim
  }, []);
  const created = await writer('POST', 'repos/' + repository + '/git/refs', {
    ref,
    sha
  });
  assert.equal(created.ref, ref);
  assert.equal(created.object.type, 'commit');
  assert.equal(created.object.sha, sha);
  return {
    ref,
    claimCommit: sha
  };
}
export async function publishDeliveryResult({
  reader,
  writer,
  claim,
  result
}) {
  const publicClaim = publicLedgerClaim(claim),
    ledger = await readDeliveryLedger({
      ...publicClaim,
      reader
    });
  assert.ok(ledger, 'durable claim missing');
  assert.deepEqual(claim.ledger, {
    ref: ledger.ref,
    claimCommit: ledger.claimCommit
  });
  assert.equal(ledger.head, ledger.claimCommit, 'only the exact unadvanced claim can receive a result');
  assert.deepEqual(ledger.claim, publicClaim);
  const publicResult = publicLedgerResult(result, publicClaim),
    sha = await writeNode(writer, claim.repository, {
      schema: 'formal-ai-hive-ledger/v1',
      kind: 'result',
      claim: publicClaim,
      result: publicResult
    }, [ledger.claimCommit]);
  const updated = await writer('PATCH', 'repos/' + claim.repository + '/git/refs/' + ledger.ref.slice(5), {
    sha,
    force: false
  });
  assert.equal(updated.ref, ledger.ref);
  assert.equal(updated.object.sha, sha);
  return {
    ref: ledger.ref,
    claimCommit: ledger.claimCommit,
    resultCommit: sha
  };
}
