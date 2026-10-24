import { createDeliveryClaim, publishDeliveryResult, githubLedgerWriter } from './lib/hive-delivery-ledger.mjs';
import { taskCredentialTarget, taskCredentialScope, requireTaskCredentialAccess, requireTaskCredentialConfiguration } from './lib/hive-delivery-credentials.mjs';
// Credential preflight before claim; claim before effects. Publish only a qualified result after the unchanged independent oracle.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
import { boundedCommand } from '../js/server/bounded-command.mjs';
import { githubReader, recheckAdmission } from './admit-hive-original-delivery.mjs';
import { planClients, journalName, authenticateDurableProducer } from './lib/hive-delivery-journal.mjs';
import { admissionPolicy, requireQualifiedDelivery } from './lib/hive-delivery-admission.mjs';
import { requireExactProgramOutput } from './lib/hive-delivery-evidence.mjs';
export function readSmallBytes(file) {
  const descriptor = fs.openSync(file, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
  try {
    const before = fs.fstatSync(descriptor);
    assert.ok(before.isFile() && before.size > 0 && before.size <= 1048576);
    const bytes = Buffer.alloc(before.size + 1);
    let used = 0;
    for (let action = 0; action <= before.size; action++) {
      const count = fs.readSync(descriptor, bytes, used, bytes.length - used, null);
      if (count === 0) break;
      used += count;
      assert.ok(used <= before.size, 'JSON changed while reading');
    }
    const after = fs.fstatSync(descriptor);
    for (const key of ['dev', 'ino', 'size', 'mtimeMs', 'ctimeMs']) assert.equal(after[key], before[key]);
    assert.equal(used, before.size);
    return bytes.subarray(0, used);
  } finally {
    fs.closeSync(descriptor);
  }
}
export function readSmallJson(file) {
  return JSON.parse(new TextDecoder('utf-8', {
    fatal: true
  }).decode(readSmallBytes(file)));
}
function hostedContext(environment) {
  assert.equal(environment.GITHUB_ACTIONS, 'true');
  assert.ok(environment.GITHUB_WORKFLOW_REF?.startsWith(environment.GITHUB_REPOSITORY + '/.github/workflows/hive-original-delivery.yml@'));
  const run = Number(environment.GITHUB_RUN_ID),
    attempt = Number(environment.GITHUB_RUN_ATTEMPT);
  assert.ok(Number.isSafeInteger(run) && run > 0 && [1, 2].includes(attempt), 'at most two workflow attempts');
  const selection = readSmallJson(environment.HIVE_ADMISSION_SELECTION);
  assert.equal(selection.schema, admissionPolicy);
  assert.equal(selection.admitted, true);
  assert.equal(selection.producer.repository, environment.GITHUB_REPOSITORY);
  assert.equal(selection.producer.run, String(run));
  assert.equal(selection.producer.attempt, String(attempt));
  assert.equal(selection.selected.source, environment.HIVE_ADMISSION_SOURCE);
  const tool = environment.E2E_TOOL;
  assert.ok(['claude', 'agent', 'codex'].includes(tool));
  return {
    run,
    attempt,
    selection,
    tool
  };
}
function writeObservation(environment, name, value) {
  const directory = environment.HIVE_ATTEMPT_DIRECTORY;
  assert.ok(directory && path.isAbsolute(directory));
  const relative = path.relative(fs.realpathSync(environment.RUNNER_TEMP), directory);
  assert.ok(relative && !relative.startsWith('..') && !path.isAbsolute(relative));
  fs.mkdirSync(directory, {
    recursive: true,
    mode: 0o700
  });
  assert.equal(fs.realpathSync(directory), path.resolve(directory));
  fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + '\n', {
    flag: 'wx',
    mode: 0o600
  });
}
function outputs(environment, values) {
  if (environment.GITHUB_OUTPUT) fs.appendFileSync(environment.GITHUB_OUTPUT,
 Object.entries(values).map(([key,
 value]) => key + '=' + value).join('\n') + '\n');
}
export async function claimAttempt(environment = process.env, {
  transport = boundedCommand
} = {}) {
  const {
    run,
    attempt,
    selection,
    tool
  } = hostedContext(environment);
  const repository = environment.GITHUB_REPOSITORY;
  const reader = githubReader({
    repository,
    environment,
    transport
  });
  const event = readSmallJson(environment.GITHUB_EVENT_PATH);
  const fresh = await recheckAdmission({
    selection,
    repository,
    eventName: environment.GITHUB_EVENT_NAME,
    event,
    reader
  });
  assert.equal(fresh.admitted, true, 'all other CI must remain green');
  const decisions = await planClients({
    reader,
    selection: selection.selected,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash,
    currentRun: run
  });
  const decision = decisions[tool];
  if (decision.action !== 'execute') {
    outputs(environment, {
      execute: 'false',
      reason: decision.action,
      tested_head: decision.testedHead ?? ''
    });
    return decision;
  }
  const observed = await transport('git', ['rev-parse', 'HEAD'], {
    timeout: 10000,
    stdoutLimitBytes: 128,
    stderrLimitBytes: 65536
  });
  assert.equal(observed.status, 0);
  assert.equal(observed.signal, null);
  assert.ok(!observed.error && !observed.timed_out && observed.cleanup_complete === true);
  assert.equal(Buffer.from(observed.stdout).toString('utf8').trim(), selection.selected.source);
  const image = readSmallJson(path.join(environment.HIVE_IMAGE_BUNDLE_DIR, 'image-bundle.json'));
  assert.equal(image.identity.source, selection.selected.source);
  assert.equal(image.identity.workflowRun.run, String(run));
  assert.equal(image.identity.workflowRun.attempt, String(attempt));
  assert.equal(image.identity.binarySha256, image.identity.actualTaskImageBinarySha256);
  const runtime = selection.publishedRuntime;
  if (runtime) {
    assert.deepEqual(image.identity.publishedRuntime, runtime);
    assert.equal(image.identity.binarySha256, runtime.binarySha256);
  } else assert.equal(image.identity.publishedRuntime, undefined);
  const imageDigest = runtime ? runtime.image.split('@')[1] : image.imageId;
  assert.match(imageDigest, /^sha256:[a-f0-9]{64}$/u);
  const claim = {
    schema: 'formal-ai-hive-attempt-claim/v1',
    qualification: admissionPolicy,
    repository,
    scope: selection.selected.scope,
    tool,
    run,
    attempt,
    testedHead: selection.selected.source,
    protocolCommit: selection.producer.protocol,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash,
    imageDigest,
    actualTaskImageId: image.imageId,
    binarySha256: image.identity.binarySha256,
    taskMode: 'historical-1364',
    ordinal: decision.ordinal
  };
  claim.ledger = await createDeliveryClaim({
    reader,
    writer: githubLedgerWriter({
      repository,
      environment,
      transport
    }),
    claim
  });
  writeObservation(environment, 'claim.json', claim);
  outputs(environment, {
    execute: 'true',
    claim_name: journalName(claim.scope, tool, 'claim', claim.policyHash, attempt),
    result_name: journalName(claim.scope, tool, 'result', claim.policyHash, attempt),
    runtime_image: runtime?.image ?? image.imageId
  });
  return claim;
}
export function resultForAttempt(claim, {
  acceptance,
  independent,
  outcome
}) {
  const success = outcome === 'success';
  if (success) {
    assert.equal(independent.schema, 'formal-ai-original-hive-delivery-v1');
    assert.equal(independent.tool, claim.tool);
    assert.equal(independent.originalHarnessPassed, true);
    assert.equal(independent.ready, true);
    assert.equal(independent.binary.source, claim.testedHead);
    assert.equal(independent.binary.binarySha256, claim.binarySha256);
    assert.equal(independent.binary.actualTaskImageBinarySha256, claim.binarySha256);
    assert.equal(independent.binary.workflowRun.run, String(claim.run));
    assert.equal(independent.binary.workflowRun.attempt, String(claim.attempt));
    requireExactProgramOutput(independent.result, 'Hello, World!\n');
    requireQualifiedDelivery(acceptance, {
      inputHash: claim.inputHash,
      tool: claim.tool,
      source: claim.testedHead,
      imageDigest: claim.imageDigest
    });
    assert.equal(acceptance.taskPRhead, independent.head);
    if (acceptance.taskTransport) assert.deepEqual(acceptance.taskTransport, independent.taskTransport);
  }
  return {
    ...claim,
    schema: 'formal-ai-hive-attempt-result/v1',
    state: success ? 'success' : 'uncertain',
    originalOraclePassed: success,
    acceptance: success ? acceptance : null
  };
}
export function recordAttemptResult(environment = process.env) {
  const {
    run,
    attempt,
    selection,
    tool
  } = hostedContext(environment);
  const claim = readSmallJson(environment.HIVE_ATTEMPT_CLAIM);
  for (const [key, value] of Object.entries({
    run,
    attempt,
    tool,
    repository: environment.GITHUB_REPOSITORY,
    testedHead: selection.selected.source,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash
  })) assert.equal(claim[key], value);
  const outcome = environment.HIVE_VERIFICATION_OUTCOME;
  assert.ok(['success', 'failure', 'cancelled'].includes(outcome));
  const result = resultForAttempt(claim, {
    outcome,
    acceptance: outcome === 'success' ? readSmallJson(path.join(environment.E2E_LOG_DIR, 'issue-1364-delivery.json')) : null,
    independent: outcome === 'success' ? readSmallJson(path.join(environment.E2E_LOG_DIR, 'independent-delivery-receipt.json')) : null
  });
  writeObservation(environment, 'result.json', result);
  outputs(environment, {
    result_name: journalName(claim.scope, tool, 'result', claim.policyHash, attempt)
  });
  return result;
}
/** This owner record permits bounded terminal cleanup discovery, never delivery success credit. */
export function recordEffectOwner(environment = process.env) {
  const selection = readSmallJson(environment.HIVE_ADMISSION_SELECTION);
  assert.equal(selection.admitted, true);
  assert.equal(selection.producer.run, environment.GITHUB_RUN_ID);
  assert.equal(selection.producer.attempt, environment.GITHUB_RUN_ATTEMPT);
  const owner = {
    schema: 'formal-ai-hive-effect-owner/v1',
    repository: environment.GITHUB_REPOSITORY,
    run: Number(environment.GITHUB_RUN_ID),
    attempt: Number(environment.GITHUB_RUN_ATTEMPT),
    protocolCommit: selection.producer.protocol,
    selectedSource: selection.selected.source,
    scope: selection.selected.scope,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash,
    taskMode: 'historical-1364',
    tools: Object.entries(selection.clients).filter(([, value]) => value.action === 'execute').map(([tool]) => tool)
  };
  assert.ok(owner.tools.length > 0);
  writeObservation(environment, 'owner.json', owner);
  return owner;
}
/** The final gate authenticates exactly three qualified clients; reuse retains the actual prior tested head. */
export async function aggregateAcceptance(environment = process.env, {
  transport = boundedCommand
} = {}) {
  const selection = readSmallJson(environment.HIVE_ADMISSION_SELECTION);
  assert.equal(selection.admitted, true);
  assert.equal(selection.producer.run, environment.GITHUB_RUN_ID);
  assert.equal(selection.producer.attempt, environment.GITHUB_RUN_ATTEMPT);
  const reader = githubReader({
    repository: environment.GITHUB_REPOSITORY,
    environment,
    transport
  });
  const decisions = await planClients({
    reader,
    selection: selection.selected,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash,
    currentRun: Number(environment.GITHUB_RUN_ID)
  });
  for (const tool of ['claude', 'agent', 'codex']) assert.equal(decisions[tool].action, 'reuse', 'missing qualified independent ' + tool + ' delivery');
  const result = {
    schema: 'formal-ai-hive-three-client-acceptance/v1',
    selectedSource: selection.selected.source,
    clients: decisions,
    inputHash: selection.inputHash,
    policyHash: selection.policyHash,
    complete: true,
    authorshipCredit: 0
  };
  writeObservation(environment, 'acceptance.json', result);
  return result;
}
export function prepareCredentialTarget(environment = process.env) {
  const {
    selection,
    tool
  } = hostedContext(environment);
  const target = taskCredentialTarget(readSmallBytes(environment.HIVE_CANONICAL_INPUT_FILE), {
    inputHash: selection.inputHash,
    tool
  });
  outputs(environment, {
    target_owner: target.owner,
    target_name: target.name,
    target_repository: target.repository
  });
  return target;
}
export async function preflightTaskCredentials(environment = process.env, {
  transport = boundedCommand
} = {}) {
  const layer = environment.HIVE_CREDENTIAL_LAYER;
  const target = taskCredentialScope(prepareCredentialTarget(environment), {
    layer, repository: environment.GITHUB_REPOSITORY
  });
  const token = environment.GH_TOKEN;
  assert.ok(typeof token === 'string' && token.length > 0 && token.length <= 8192 && !/\s/u.test(token),
 'GitHub Actions must provide a task token');
  async function request(endpoint) {
    const result = await transport('gh', ['api', '--hostname', 'github.com', '--include', endpoint], {
      env: environment,
      timeout: 20000,
      stdoutLimitBytes: 1048576,
      stderrLimitBytes: 65536
    });
    assert.equal(result.status, 0, 'task credential read-only preflight failed');
    assert.equal(result.signal, null);
    assert.ok(!result.error && !result.timed_out && result.cleanup_complete === true);
    const text = new TextDecoder('utf-8', {
        fatal: true
      }).decode(result.stdout),
      separator = /\r?\n\r?\n/u.exec(text);
    assert.ok(separator, 'complete API headers required');
    const header = text.slice(0, separator.index);
    assert.match(header, /^HTTP\/\S+ 200(?: |\r?$)/mu);
    const value = JSON.parse(text.slice(separator.index + separator[0].length));
    const scopes = /^x-oauth-scopes:\s*([^\r\n]*)/imu.exec(header)?.[1].split(',').map(value => value.trim()).filter(Boolean) ?? null;
    return {
      value,
      scopes
    };
  }
  const observed = await request('repos/' + target.repository);
  const installation = layer === 'app' ? (await request('installation/repositories?per_page=100')).value : null;
  const proof = requireTaskCredentialAccess(target, {
    repository: observed.value,
    layer,
    installation,
    scopes: observed.scopes,
    currentRepository: environment.GITHUB_REPOSITORY
  });
  console.log('::add-mask::' + token);
  outputs(environment, {
    token,
    layer,
    'can-create-repositories': 'false'
  });
  return proof;
}
export async function persistAttemptResult(environment = process.env, {
  transport = boundedCommand
} = {}) {
  const {
      run,
      attempt,
      selection,
      tool
    } = hostedContext(environment),
    repository = environment.GITHUB_REPOSITORY;
  const claim = readSmallJson(environment.HIVE_ATTEMPT_CLAIM),
    result = readSmallJson(environment.HIVE_ATTEMPT_RESULT);
  for (const [key, value] of Object.entries({
    run,
    attempt,
    tool,
    repository,
    testedHead: selection.selected.source,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash
  })) assert.equal(claim[key], value);
  const reader = githubReader({
    repository,
    environment,
    transport
  });
  const proof = await authenticateDurableProducer({
    reader,
    repository,
    claim,
    result,
    requirePublisher: false
  });
  assert.equal(proof.state, result.state, 'actual independent verification must be complete');
  assert.equal(proof.producerConclusion, result.state === 'success' ? 'success' : 'failure', 'actual original oracle outcome');
  return publishDeliveryResult({
    reader,
    writer: githubLedgerWriter({
      repository,
      environment,
      transport
    }),
    claim,
    result
  });
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const mode = process.argv[2];
  assert.ok(['configuration', 'target', 'credentials', 'claim', 'result', 'owner', 'aggregate', 'publish-result'].includes(mode));
  const result = mode === 'configuration' ? requireTaskCredentialConfiguration()
    : mode === 'target' ? prepareCredentialTarget()
    : mode === 'credentials' ? await preflightTaskCredentials()
    : mode === 'claim' ? await claimAttempt()
    : mode === 'result' ? recordAttemptResult()
    : mode === 'owner' ? recordEffectOwner()
    : mode === 'publish-result' ? await persistAttemptResult()
    : await aggregateAcceptance();
  console.log(JSON.stringify({
    mode,
    action: result.action ?? result.state ?? 'observed',
    repository: result.repository ?? null,
    workflowWrite: result.workflowWrite ?? null,
    deliveryVerified: false,
    authorshipCredit: 0
  }));
}
