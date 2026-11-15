import { createDeliveryClaim, publishDeliveryResult, githubLedgerWriter, readDeliveryLedger } from './lib/hive-delivery-ledger.mjs';
import { classifyRecheck, clientStates, outcomeIdentity, noEffectProof, deferredReceipt, refusedReceipt, plannedReceipt,
  claimedReceipt, fallbackReceipt, validateOutcome, outcomeName, wakeupName, wakeupRecord, nextContinuation,
  summarizeOutcomes } from './lib/hive-delivery-deferral.mjs';
import { taskCredentialTarget, taskCredentialScope, probeTaskPushAccess, requireTaskCredentialAccess, requireTaskCredentialConfiguration } from './lib/hive-delivery-credentials.mjs';
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
  for (const value of Object.values(values)) assert.ok(!/[\r\n]/u.test(String(value)), 'single-line step output');
  if (environment.GITHUB_OUTPUT) fs.appendFileSync(environment.GITHUB_OUTPUT,
 Object.entries(values).map(([key,
 value]) => key + '=' + value).join('\n') + '\n');
}
/** Bounded prerequisite rechecks before a leg is deferred; never an unbounded wait (#1485). */
export const recheckBudget = Object.freeze({checks: 2, delaySeconds: 10, maxChecks: 5, maxDelaySeconds: 60});
export function prerequisiteRecheckBudget(environment = {}) {
  const parse = (value, fallback, maximum, minimum) => {
    if (value === undefined || value === '') return fallback;
    assert.match(value, /^\d+$/u, 'bounded prerequisite recheck setting');
    const number = Number(value);
    assert.ok(number >= minimum && number <= maximum, 'bounded prerequisite recheck setting');
    return number;
  };
  return {
    checks: parse(environment.HIVE_PREREQUISITE_RECHECKS, recheckBudget.checks, recheckBudget.maxChecks, 1),
    delaySeconds: parse(environment.HIVE_PREREQUISITE_RECHECK_DELAY_SECONDS, recheckBudget.delaySeconds, recheckBudget.maxDelaySeconds, 0)
  };
}
const ledgerKey = identity => ({repository: identity.repository, scope: identity.scope, tool: identity.tool,
  policyHash: identity.policyHash, inputHash: identity.inputHash});
export const outcomeFile = tool => 'outcome-' + tool + '.json';
export const wakeupFile = tool => 'wakeup-' + tool + '.json';
/**
 * Claim one client leg, or leave a typed pre-effect outcome instead (#1485).
 * Pending prerequisites get a bounded number of fresh rechecks. If they are still
 * pending, the leg is deferred before effects: it writes a deferred receipt (and,
 * for a published candidate, a wakeup record keyed by the blocking run) and exits
 * successfully without claiming anything. A failed, cancelled or skipped
 * prerequisite, or a changed candidate, is a refusal. Only an eligible recheck
 * reaches the unchanged planner and durable claim.
 */
export async function claimAttempt(environment = process.env, {
  transport = boundedCommand,
  readerFactory = () => githubReader({repository: environment.GITHUB_REPOSITORY, environment, transport}),
  recheck = recheckAdmission,
  plan = planClients,
  readLedger = readDeliveryLedger,
  createClaim = createDeliveryClaim,
  ledgerWriter = () => githubLedgerWriter({repository: environment.GITHUB_REPOSITORY, environment, transport}),
  sleep = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds)),
  now = Date.now
} = {}) {
  const {
    run,
    attempt,
    selection,
    tool
  } = hostedContext(environment);
  const repository = environment.GITHUB_REPOSITORY;
  const identity = outcomeIdentity({repository, selection, tool, run, attempt});
  const event = readSmallJson(environment.GITHUB_EVENT_PATH);
  const budget = prerequisiteRecheckBudget(environment);
  let fresh, classification, checks = 0;
  for (;;) {
    // Each recheck uses a new bounded reader, so its request and deadline budget is per check.
    fresh = await recheck({
      selection,
      repository,
      eventName: environment.GITHUB_EVENT_NAME,
      event,
      reader: readerFactory()
    });
    checks++;
    classification = classifyRecheck(fresh);
    if (classification.state !== clientStates.deferred || checks >= budget.checks) break;
    await sleep(budget.delaySeconds * 1000);
  }
  const reader = readerFactory();
  const name = outcomeName(identity.scope, tool, identity.policyHash, attempt);
  if (classification.state !== clientStates.eligible) {
    const effects = noEffectProof(identity, await readLedger({reader, ...ledgerKey(identity)}));
    const observedAt = now();
    let outcome, wakeup = null;
    if (classification.state === clientStates.deferred) {
      const generation = identity.candidate.event === 'published-release' ? nextContinuation(selection) : null;
      if (identity.candidate.event === 'published-release' && generation === null) {
        outcome = refusedReceipt(identity, {...classification, state: clientStates.refused, reason: 'continuation-budget-exhausted'},
          {observedAt, effects});
      } else {
        outcome = deferredReceipt(identity, classification, {observedAt, effects, rechecks: checks, generation});
        const blocking = outcome.prerequisites.find(value => value.run && value.status !== 'completed');
        if (generation !== null && blocking) wakeup = wakeupRecord({repository, run, attempt, role: tool,
          candidate: identity.candidate, prerequisites: outcome.prerequisites, observedAt, generation});
      }
    } else outcome = refusedReceipt(identity, classification, {observedAt, effects});
    writeObservation(environment, outcomeFile(tool), outcome);
    if (wakeup) writeObservation(environment, wakeupFile(tool), wakeup);
    const blocking = wakeup?.prerequisites[0]?.run;
    outputs(environment, {
      execute: 'false',
      state: outcome.state,
      reason: outcome.reason,
      outcome_name: name,
      wakeup_name: blocking ? wakeupName(blocking, tool) : ''
    });
    summary(environment, outcome);
    return outcome;
  }
  const decisions = await plan({
    reader,
    selection: selection.selected,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash,
    currentRun: run
  });
  const decision = decisions[tool];
  if (decision.action !== 'execute') {
    // A duplicate delivery event or a prior success: the existing claim ledger decides, nothing re-executes.
    const outcome = plannedReceipt(identity, decision, {observedAt: now()});
    writeObservation(environment, outcomeFile(tool), outcome);
    outputs(environment, {
      execute: 'false',
      state: outcome.state,
      reason: decision.action === 'reuse' ? 'reuse' : decision.reason,
      outcome_name: name,
      wakeup_name: '',
      tested_head: decision.testedHead ?? ''
    });
    summary(environment, outcome);
    return outcome;
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
  claim.ledger = await createClaim({
    reader,
    writer: ledgerWriter(),
    claim
  });
  writeObservation(environment, 'claim.json', claim);
  const outcome = claimedReceipt(identity, claim, {observedAt: now()});
  writeObservation(environment, outcomeFile(tool), outcome);
  outputs(environment, {
    execute: 'true',
    state: outcome.state,
    outcome_name: name,
    wakeup_name: '',
    claim_name: journalName(claim.scope, tool, 'claim', claim.policyHash, attempt),
    result_name: journalName(claim.scope, tool, 'result', claim.policyHash, attempt),
    runtime_image: runtime?.image ?? image.imageId
  });
  return claim;
}
/**
 * Fallback when the claim step ended without a typed outcome. The durable ledger
 * decides: a claim held by this run means effects may have started
 * (interrupted, unknown after effects); otherwise the leg failed before any claim.
 */
export async function recordFallbackOutcome(environment = process.env, {
  transport = boundedCommand,
  readerFactory = () => githubReader({repository: environment.GITHUB_REPOSITORY, environment, transport}),
  readLedger = readDeliveryLedger,
  now = Date.now
} = {}) {
  const {run, attempt, selection, tool} = hostedContext(environment);
  const identity = outcomeIdentity({repository: environment.GITHUB_REPOSITORY, selection, tool, run, attempt});
  const name = outcomeName(identity.scope, tool, identity.policyHash, attempt);
  const file = path.join(environment.HIVE_ATTEMPT_DIRECTORY, outcomeFile(tool));
  let outcome;
  if (fs.existsSync(file)) outcome = validateOutcome(readSmallJson(file), identity);
  else {
    outcome = fallbackReceipt(identity, await readLedger({reader: readerFactory(), ...ledgerKey(identity)}), {observedAt: now()});
    writeObservation(environment, outcomeFile(tool), outcome);
  }
  outputs(environment, {state: outcome.state, outcome_name: name});
  summary(environment, outcome);
  return outcome;
}
/**
 * Downstream jobs read the typed outcome instead of downloading claim or result
 * artifacts that a deferred or refused leg never produced. Only a claimed leg is
 * verified and published. An interrupted leg (unknown after effects) fails here,
 * truthfully, because its custody needs cleanup rather than verification.
 */
export function readClientOutcome(environment = process.env) {
  const {run, attempt, selection, tool} = hostedContext(environment);
  const identity = outcomeIdentity({repository: environment.GITHUB_REPOSITORY, selection, tool, run, attempt});
  const outcome = validateOutcome(readSmallJson(environment.HIVE_CLIENT_OUTCOME), identity);
  summary(environment, outcome);
  assert.notEqual(outcome.state, clientStates.interrupted,
    'unknown after effects: the ' + tool + ' claim is held without custody; terminal cleanup decides, never a retry');
  outputs(environment, {state: outcome.state, verify: String(outcome.state === clientStates.claimed)});
  return outcome;
}
function summary(environment, outcome) {
  if (!environment.GITHUB_STEP_SUMMARY) return;
  const blocking = outcome.prerequisites.map(value => value.run
    ? value.path + ' run ' + value.run + ' (' + value.event + ', ' + value.status + (value.conclusion ? ', ' + value.conclusion : '') + ')'
    : value.path + ' (' + value.status + ')');
  fs.appendFileSync(environment.GITHUB_STEP_SUMMARY, [
    '### Hive ' + outcome.tool + ' client: ' + outcome.state,
    '',
    '- Reason: ' + (outcome.reason ?? 'none'),
    '- Candidate: ' + outcome.candidate.scope + ' at ' + outcome.testedHead,
    '- Effects claimed: ' + outcome.effects.claimed,
    ...(blocking.length ? ['- Prerequisites: ' + blocking.join('; ')] : []),
    ...(outcome.resume ? ['- Resume: ' + outcome.resume.via + ' for the same candidate'] : []),
    '- Execution credit: 0',
    ''
  ].join('\n') + '\n');
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
/** Read this run's typed client outcomes; a missing directory or file is reported, never invented. */
export function readClientOutcomes(environment, selection) {
  const directory = environment.HIVE_CLIENT_OUTCOMES;
  const run = Number(environment.GITHUB_RUN_ID), attempt = Number(environment.GITHUB_RUN_ATTEMPT);
  const outcomes = {};
  for (const tool of ['claude', 'agent', 'codex']) {
    const file = directory ? path.join(directory, outcomeFile(tool)) : null;
    if (!file || !fs.existsSync(file)) continue;
    outcomes[tool] = validateOutcome(readSmallJson(file),
      outcomeIdentity({repository: environment.GITHUB_REPOSITORY, selection, tool, run, attempt}));
  }
  return outcomes;
}
/**
 * The final gate authenticates exactly three qualified clients; reuse retains the actual prior tested head.
 * Credit only comes from the durable ledger planner. A run whose only missing clients
 * were deferred before effects is reported as deferred and incomplete, with zero
 * credit, instead of failing on missing claim artifacts (#1485). Any other gap fails.
 */
export async function aggregateAcceptance(environment = process.env, {
  transport = boundedCommand,
  reader = null,
  plan = planClients
} = {}) {
  const selection = readSmallJson(environment.HIVE_ADMISSION_SELECTION);
  assert.equal(selection.admitted, true);
  assert.equal(selection.producer.run, environment.GITHUB_RUN_ID);
  assert.equal(selection.producer.attempt, environment.GITHUB_RUN_ATTEMPT);
  const decisions = await plan({
    reader: reader ?? githubReader({
      repository: environment.GITHUB_REPOSITORY,
      environment,
      transport
    }),
    selection: selection.selected,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash,
    currentRun: Number(environment.GITHUB_RUN_ID)
  });
  const observed = summarizeOutcomes(decisions, readClientOutcomes(environment, selection));
  const result = {
    schema: 'formal-ai-hive-three-client-acceptance/v1',
    selectedSource: selection.selected.source,
    clients: decisions,
    states: Object.fromEntries(Object.entries(observed.clients).map(([tool, value]) => [tool, {
      state: value.state, ...(value.reason ? {reason: value.reason} : {}),
      ...(value.prerequisites ? {prerequisites: value.prerequisites} : {})}])),
    inputHash: selection.inputHash,
    policyHash: selection.policyHash,
    complete: observed.complete,
    deferred: observed.deferred,
    authorshipCredit: 0
  };
  writeObservation(environment, 'acceptance.json', result);
  if (environment.GITHUB_STEP_SUMMARY) fs.appendFileSync(environment.GITHUB_STEP_SUMMARY, [
    '### Hive three-client acceptance: ' + (observed.complete ? 'complete' : observed.deferred ? 'deferred before effects' : 'incomplete'),
    '',
    ...Object.entries(result.states).map(([tool, value]) => '- ' + tool + ': ' + value.state + (value.reason ? ' (' + value.reason + ')' : '')),
    ''
  ].join('\n') + '\n');
  if (observed.deferred) {
    console.log('::notice title=Hive clients deferred before effects::' + Object.entries(result.states)
      .filter(([, value]) => value.state === clientStates.deferred).map(([tool]) => tool).join(', ')
      + ' did not run and received no credit; the same candidate resumes after its prerequisites succeed');
    return result;
  }
  for (const tool of ['claude', 'agent', 'codex']) assert.equal(decisions[tool].action, 'reuse',
    'missing qualified independent ' + tool + ' delivery (' + result.states[tool].state + ')');
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
  const pushProbe = observed.value.permissions?.push === true ? null
    : await probeTaskPushAccess(target, {environment, transport});
  const installation = layer === 'app' ? (await request('installation/repositories?per_page=100')).value : null;
  const proof = requireTaskCredentialAccess(target, {
    repository: observed.value,
    layer,
    installation,
    scopes: observed.scopes,
    currentRepository: environment.GITHUB_REPOSITORY,
    pushProbe
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
  assert.ok(['configuration', 'target', 'credentials', 'claim', 'outcome', 'client-outcome', 'result', 'owner', 'aggregate', 'publish-result'].includes(mode));
  const result = mode === 'configuration' ? requireTaskCredentialConfiguration()
    : mode === 'target' ? prepareCredentialTarget()
    : mode === 'credentials' ? await preflightTaskCredentials()
    : mode === 'claim' ? await claimAttempt()
    : mode === 'outcome' ? await recordFallbackOutcome()
    : mode === 'client-outcome' ? readClientOutcome()
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
