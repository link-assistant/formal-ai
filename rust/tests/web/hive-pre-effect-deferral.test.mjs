// Issue #1485: a pending prerequisite defers unstarted Hive clients before any effects,
// downstream jobs consume the typed outcome, and the same candidate resumes on the
// blocking run's completion event without re-executing a claimed leg.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync, existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {deflateRawSync} from 'node:zlib';
import YAML from 'yaml';
import {claimAttempt, readClientOutcome, aggregateAcceptance, recordFallbackOutcome, prerequisiteRecheckBudget,
  outcomeFile, wakeupFile} from '../../../scripts/hive-delivery-attempt.mjs';
import {observeDeferredWakeups, admissionWakeup, wakeupMember} from '../../../scripts/admit-hive-original-delivery.mjs';
import {classifyRecheck, clientStates, outcomeIdentity, noEffectProof, deferredReceipt, validateOutcome,
  validateWakeup, wakeupName, wakeupRecord, nextContinuation, maxContinuations,
  summarizeOutcomes} from '../../../scripts/lib/hive-delivery-deferral.mjs';
import {admissionPolicy, clientAdmission} from '../../../scripts/lib/hive-delivery-admission.mjs';

const capture = JSON.parse(readFileSync(new URL('../fixtures/generated-e2e/scheduled-claim-race.json', import.meta.url), 'utf8'));
const repository = capture.repository, source = capture.source, ownerRun = capture.ownerRun;
const blockingRun = capture.diagnostic.blockedBy.run;
const verifiedRun = capture.releaseRuns.find(run => run.path === '.github/workflows/verify-release-downloads.yml').id;
const policyHash = '1'.repeat(64), inputHash = '2'.repeat(64), protocol = '3'.repeat(40), scope = 'release-v1.2.3';
const observedAt = Date.parse(capture.observedFailureAt);
const trustedMain = '4'.repeat(40);
// The captured diagnostic is the typed result recheckAdmission now returns for the pending scheduled run.
const pending = () => ({admitted: false, ...structuredClone(capture.diagnostic)});

function selectionFor({run = ownerRun, resume} = {}) {
  return {schema: admissionPolicy, admitted: true, policyHash, inputHash,
    selected: {repository, event: 'published-release', source, scope},
    publicDownloadVerification: {run: verifiedRun},
    producer: {repository, run: String(run), attempt: '1', protocol},
    ...(resume ? {resume} : {})};
}

function hosted(tool, {selection = selectionFor(), extra = {}} = {}) {
  const directory = mkdtempSync(path.join(tmpdir(), 'hive-deferral-'));
  writeFileSync(path.join(directory, 'selection.json'), JSON.stringify(selection));
  writeFileSync(path.join(directory, 'event.json'), JSON.stringify({workflow_run: {id: verifiedRun}}));
  return {directory, environment: {GITHUB_ACTIONS: 'true', GITHUB_REPOSITORY: repository,
    GITHUB_WORKFLOW_REF: repository + '/.github/workflows/hive-original-delivery.yml@refs/heads/main',
    GITHUB_RUN_ID: selection.producer.run, GITHUB_RUN_ATTEMPT: '1', GITHUB_EVENT_NAME: 'workflow_run',
    GITHUB_EVENT_PATH: path.join(directory, 'event.json'), HIVE_ADMISSION_SELECTION: path.join(directory, 'selection.json'),
    HIVE_ADMISSION_SOURCE: source, E2E_TOOL: tool, RUNNER_TEMP: directory,
    HIVE_ATTEMPT_DIRECTORY: path.join(directory, 'attempt'), GITHUB_OUTPUT: path.join(directory, 'output'),
    HIVE_PREREQUISITE_RECHECK_DELAY_SECONDS: '10', ...extra}};
}

const readOutputs = file => Object.fromEntries(readFileSync(file, 'utf8').trim().split('\n').map(line => {
  const at = line.indexOf('=');
  return [line.slice(0, at), line.slice(at + 1)];
}));
const forbidden = name => () => assert.fail(name + ' must not run for a deferred or refused leg');
const identityOf = (tool, selection = selectionFor()) => outcomeIdentity({repository, selection, tool, run: ownerRun, attempt: 1});

async function deferIncidentLeg(tool, options = {}) {
  const context = hosted(tool, options);
  const sleeps = [], rechecks = [];
  const outcome = await claimAttempt(context.environment, {
    readerFactory: () => ({}),
    recheck: async value => {
      rechecks.push(value);
      return pending();
    },
    readLedger: async () => null,
    plan: forbidden('planner'),
    createClaim: forbidden('durable claim'),
    sleep: async milliseconds => sleeps.push(milliseconds),
    now: () => observedAt
  });
  return {...context, outcome, sleeps, rechecks};
}

test('incident replay: the two unclaimed clients are deferred before effects and name run 40675356715', async () => {
  for (const tool of ['agent', 'codex']) {
    const {directory, environment, outcome, sleeps, rechecks} = await deferIncidentLeg(tool);
    try {
      assert.equal(outcome.state, clientStates.deferred);
      assert.equal(outcome.reason, 'other CI pending');
      assert.deepEqual(outcome.prerequisites.map(value => [value.run, value.status, value.conclusion]),
        [[blockingRun, 'in_progress', null]]);
      assert.equal(outcome.effects.claimed, false);
      assert.equal(outcome.effects.ledgerHead, null);
      assert.equal(outcome.executionCredit, 0);
      assert.equal(outcome.authorshipCredit, 0);
      assert.equal(outcome.observedAt, capture.observedFailureAt.replace('Z', '.000Z'));
      assert.deepEqual(outcome.candidate, {event: 'published-release', source, scope, verifiedRun});
      // Bounded rechecks: two fresh observations with one bounded pause, never an open-ended wait.
      assert.equal(outcome.rechecks, 2);
      assert.equal(rechecks.length, 2);
      assert.deepEqual(sleeps, [10000]);
      assert.deepEqual(outcome.resume, {via: 'workflow_run', wakeupRuns: [blockingRun], sameCandidate: true, generation: 1});
      const persisted = JSON.parse(readFileSync(path.join(environment.HIVE_ATTEMPT_DIRECTORY, outcomeFile(tool)), 'utf8'));
      assert.deepEqual(persisted, outcome);
      validateOutcome(persisted, identityOf(tool));
      const wakeup = JSON.parse(readFileSync(path.join(environment.HIVE_ATTEMPT_DIRECTORY, wakeupFile(tool)), 'utf8'));
      validateWakeup(wakeup, {repository, trigger: {id: blockingRun}});
      assert.equal(wakeup.run, ownerRun);
      assert.equal(wakeup.generation, 1);
      const outputs = readOutputs(environment.GITHUB_OUTPUT);
      assert.equal(outputs.execute, 'false');
      assert.equal(outputs.state, 'deferred-before-effects');
      assert.equal(outputs.wakeup_name, 'hive-wakeup-' + blockingRun + '-' + tool);
      assert.equal(outputs.outcome_name, 'hive-outcome-' + scope + '-' + tool + '-' + policyHash + '-1');
      assert.equal(outputs.claim_name, undefined, 'a deferred leg never names a claim artifact');
    } finally {
      rmSync(directory, {recursive: true, force: true});
    }
  }
});

test('downstream verification, ledger and acceptance report the deferral without missing-artifact cascades', async () => {
  const outcomes = mkdtempSync(path.join(tmpdir(), 'hive-outcomes-'));
  try {
    for (const tool of ['agent', 'codex']) {
      const {directory, environment, outcome} = await deferIncidentLeg(tool);
      writeFileSync(path.join(outcomes, outcomeFile(tool)), JSON.stringify(outcome));
      const consumer = path.join(directory, 'consumer-output');
      const consumed = readClientOutcome({...environment, GITHUB_OUTPUT: consumer,
        HIVE_CLIENT_OUTCOME: path.join(environment.HIVE_ATTEMPT_DIRECTORY, outcomeFile(tool))});
      assert.equal(consumed.state, clientStates.deferred);
      assert.deepEqual(readOutputs(consumer), {state: 'deferred-before-effects', verify: 'false'});
      rmSync(directory, {recursive: true, force: true});
    }
    const {directory, environment} = hosted('claude');
    try {
      const decisions = {claude: {action: 'reuse', testedHead: source, run: 1, attempt: 1},
        agent: {action: 'execute', testedHead: source, ordinal: 1}, codex: {action: 'execute', testedHead: source, ordinal: 1}};
      const notices = [];
      const result = await aggregateAcceptance({...environment, HIVE_CLIENT_OUTCOMES: outcomes},
        {reader: {}, plan: async () => decisions, log: line => notices.push(line)});
      assert.deepEqual(notices, ['::notice title=Hive clients deferred before effects::agent, codex did not run and received no credit;'
        + ' the same candidate resumes after its prerequisites succeed']);
      assert.equal(result.complete, false);
      assert.equal(result.deferred, true);
      assert.equal(result.authorshipCredit, 0);
      assert.equal(result.states.claude.state, 'completed');
      for (const tool of ['agent', 'codex']) {
        assert.equal(result.states[tool].state, 'deferred-before-effects');
        assert.equal(result.states[tool].prerequisites[0].run, blockingRun);
      }
      // A missing outcome is never invented as a deferral: the final gate still fails truthfully.
      rmSync(path.join(outcomes, outcomeFile('codex')));
      await assert.rejects(aggregateAcceptance({...environment, HIVE_ATTEMPT_DIRECTORY: path.join(directory, 'second'),
        HIVE_CLIENT_OUTCOMES: outcomes}, {reader: {}, plan: async () => decisions}), /missing qualified independent agent delivery/u);
      const mixed = JSON.parse(readFileSync(path.join(directory, 'second', 'acceptance.json'), 'utf8'));
      assert.equal(mixed.deferred, false);
      assert.equal(mixed.states.codex.state, 'missing-outcome');
    } finally {
      rmSync(directory, {recursive: true, force: true});
    }
  } finally {
    rmSync(outcomes, {recursive: true, force: true});
  }
});

test('an interrupted leg (claim held, outcome unknown) fails downstream instead of being verified or retried', async () => {
  const {directory, environment} = hosted('agent');
  try {
    const identity = identityOf('agent');
    const durable = {ref: noEffectProof(identity, null).ledgerReference, head: '5'.repeat(40),
      claim: {run: ownerRun, attempt: 1}};
    const outcome = await recordFallbackOutcome(environment, {readerFactory: () => ({}), readLedger: async () => durable,
      now: () => observedAt});
    assert.equal(outcome.state, clientStates.interrupted);
    assert.equal(outcome.reason, 'unknown-after-effects');
    assert.throws(() => readClientOutcome({...environment, GITHUB_OUTPUT: path.join(directory, 'consumer'),
      HIVE_CLIENT_OUTCOME: path.join(environment.HIVE_ATTEMPT_DIRECTORY, outcomeFile('agent'))}), /unknown after effects/u);
    // Without this run's claim, the same fallback is a pre-claim refusal that needs a new attempt identity.
    const other = hosted('codex');
    try {
      const refused = await recordFallbackOutcome(other.environment, {readerFactory: () => ({}), readLedger: async () => null,
        now: () => observedAt});
      assert.equal(refused.state, clientStates.refused);
      assert.equal(refused.reason, 'failed-before-claim');
      assert.equal(refused.requiresNewAttemptIdentity, true);
      assert.equal(refused.effects.claimed, false);
    } finally {
      rmSync(other.directory, {recursive: true, force: true});
    }
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});

function crc(buffer) {
  let value = ~0;
  for (const byte of buffer) {
    value ^= byte;
    for (let bit = 0; bit < 8; bit++) value = value >>> 1 ^ 0xedb88320 & -(value & 1);
  }
  return ~value >>> 0;
}

function zip(name, body) {
  const raw = Buffer.from(body), compressed = deflateRawSync(raw), label = Buffer.from(name);
  const local = Buffer.alloc(30), central = Buffer.alloc(46), end = Buffer.alloc(22);
  local.writeUInt32LE(0x04034b50);
  local.writeUInt16LE(20, 4);
  local.writeUInt16LE(8, 8);
  local.writeUInt32LE(crc(raw), 14);
  local.writeUInt32LE(compressed.length, 18);
  local.writeUInt32LE(raw.length, 22);
  local.writeUInt16LE(label.length, 26);
  central.writeUInt32LE(0x02014b50);
  central.writeUInt16LE(20, 4);
  central.writeUInt16LE(20, 6);
  central.writeUInt16LE(8, 10);
  central.writeUInt32LE(crc(raw), 16);
  central.writeUInt32LE(compressed.length, 20);
  central.writeUInt32LE(raw.length, 24);
  central.writeUInt16LE(label.length, 28);
  const offset = local.length + label.length + compressed.length;
  end.writeUInt32LE(0x06054b50);
  end.writeUInt16LE(1, 8);
  end.writeUInt16LE(1, 10);
  end.writeUInt32LE(central.length + label.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([local, label, compressed, central, label, end]);
}

const trigger = (conclusion = 'success') => ({id: blockingRun, run_attempt: 1, path: capture.backgroundRun.path,
  event: 'schedule', head_branch: 'main', status: 'completed', conclusion, head_sha: source,
  repository: {full_name: repository}, head_repository: {full_name: repository}});

function wakeupReader(roles, {latest = verifiedRun, wakeups = {}} = {}) {
  const prefix = 'repos/' + repository, calls = [], artifacts = new Map(), listings = {};
  roles.forEach((role, index) => {
    const record = wakeups[role] ?? wakeupRecord({repository, run: ownerRun, attempt: 1, role,
      candidate: {event: 'published-release', source, scope, verifiedRun}, prerequisites: [capture.diagnostic.blockedBy],
      observedAt, generation: 1});
    const archive = zip(wakeupMember(role), JSON.stringify(record));
    const metadata = {id: 900 + index, name: wakeupName(blockingRun, role), expired: false, size_in_bytes: archive.length,
      digest: 'sha256:' + createHash('sha256').update(archive).digest('hex'), workflow_run: {id: ownerRun, head_sha: protocol}};
    artifacts.set(prefix + '/actions/artifacts/' + metadata.id, {metadata, archive});
    listings[metadata.name] = [metadata];
  });
  return {calls,
    read: async endpoint => {
      calls.push(endpoint);
      if (endpoint === prefix + '/git/ref/heads/main') return {object: {sha: trustedMain}};
      if (endpoint === prefix + '/actions/runs/' + ownerRun) return {id: ownerRun, path: '.github/workflows/hive-original-delivery.yml',
        event: 'workflow_run', head_sha: protocol, repository: {full_name: repository}, head_repository: {full_name: repository}};
      if (endpoint === prefix + '/compare/' + protocol + '...' + trustedMain) return {status: 'ahead'};
      if (artifacts.has(endpoint)) return structuredClone(artifacts.get(endpoint).metadata);
      assert.equal(endpoint, prefix + '/actions/workflows/verify-release-downloads.yml/runs?per_page=1');
      return {workflow_runs: [{id: latest, status: 'completed', conclusion: 'success', path: '.github/workflows/verify-release-downloads.yml',
        repository: {full_name: repository}, head_repository: {full_name: repository}}]};
    },
    bytes: async endpoint => {
      calls.push(endpoint);
      return artifacts.get(endpoint.replace(/\/zip$/u, '')).archive;
    },
    pages: async endpoint => {
      calls.push(endpoint);
      const name = /actions\/artifacts\?name=(.+)$/u.exec(endpoint);
      if (name) return listings[decodeURIComponent(name[1])] ?? [];
      // The resumed admission re-enters the unchanged, authenticated published-release route for the same candidate.
      assert.equal(endpoint, prefix + '/actions/runs/' + verifiedRun + '/artifacts');
      throw new Error('same-candidate published admission reached');
    }};
}

test('resume: the blocking run completion re-admits only the same immutable candidate', async () => {
  const reader = wakeupReader(['agent', 'codex']);
  await assert.rejects(observeDeferredWakeups({repository, reader, trigger: trigger()}), /same-candidate published admission reached/u);
  assert.ok(reader.calls.includes('repos/' + repository + '/actions/artifacts?name=hive-wakeup-' + blockingRun + '-agent'));
  // Other completions on main or pull-request runs never look for continuations.
  for (const mutate of [value => value.head_branch = 'feature', value => value.status = 'in_progress',
    value => value.event = 'pull_request', value => value.path = '.github/workflows/hive-original-delivery.yml']) {
    const changed = trigger();
    mutate(changed);
    assert.equal(await observeDeferredWakeups({repository, reader: wakeupReader(['agent']), trigger: changed}), null);
  }
  assert.equal(await observeDeferredWakeups({repository, reader: wakeupReader([]), trigger: trigger()}), null);
});

test('after prerequisites pass, the unchanged ledger lets only unclaimed clients execute', () => {
  const record = (tool, values) => ({repository, scope, tool, policyHash, inputHash, attempt: 1, testedHead: source,
    authenticated: true, ...values});
  const decide = (tool, history, currentRun) => clientAdmission({repository, scope, tool, source, policyHash, inputHash,
    currentRun, history, complete: true});
  const resumedRun = ownerRun + 1000;
  // Claude claimed in the owner run; its custody is not settled, so it is never re-executed.
  assert.deepEqual(decide('claude', [record('claude', {run: ownerRun, state: 'claimed'})], resumedRun),
    {action: 'refuse', reason: 'uncertain-or-active-attempt'});
  for (const tool of ['agent', 'codex']) assert.deepEqual(decide(tool, [], resumedRun), {action: 'execute', testedHead: source, ordinal: 1});
  // A duplicate event in the same run reaches the same ledger and refuses instead of claiming twice.
  assert.deepEqual(decide('agent', [record('agent', {run: resumedRun, state: 'claimed'})], resumedRun),
    {action: 'refuse', reason: 'already-claimed'});
});

test('eligible recheck reaches the planner; a duplicate delivery event is refused without a claim', async () => {
  const {directory, environment} = hosted('agent');
  try {
    let planned = 0;
    const outcome = await claimAttempt(environment, {
      readerFactory: () => ({}),
      recheck: async () => ({admitted: true}),
      readLedger: forbidden('pre-effect ledger read'),
      plan: async () => {
        planned++;
        return {claude: {action: 'execute'}, agent: {action: 'refuse', reason: 'already-claimed'}, codex: {action: 'execute'}};
      },
      createClaim: forbidden('durable claim'),
      sleep: forbidden('recheck pause'),
      now: () => observedAt
    });
    assert.equal(planned, 1);
    assert.equal(outcome.state, clientStates.refused);
    assert.equal(outcome.reason, 'already-claimed');
    assert.equal(outcome.effects.claimed, false);
    assert.equal(readOutputs(environment.GITHUB_OUTPUT).execute, 'false');
    assert.equal(existsSync(path.join(environment.HIVE_ATTEMPT_DIRECTORY, 'claim.json')), false);
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});

test('an eligible unclaimed leg claims through the existing durable ledger and records a claimed outcome', async () => {
  const {directory, environment} = hosted('codex');
  try {
    const bundle = path.join(directory, 'image');
    mkdirSync(bundle);
    const binary = '6'.repeat(64), imageId = 'sha256:' + '7'.repeat(64);
    writeFileSync(path.join(bundle, 'image-bundle.json'), JSON.stringify({imageId, identity: {source,
      workflowRun: {run: String(ownerRun), attempt: '1'}, binarySha256: binary, actualTaskImageBinarySha256: binary}}));
    const claims = [];
    const claim = await claimAttempt({...environment, HIVE_IMAGE_BUNDLE_DIR: bundle}, {
      transport: async () => ({status: 0, signal: null, cleanup_complete: true, stdout: Buffer.from(source + '\n')}),
      readerFactory: () => ({}),
      recheck: async () => ({admitted: true}),
      plan: async () => ({claude: {action: 'refuse', reason: 'uncertain-or-active-attempt'},
        agent: {action: 'execute', testedHead: source, ordinal: 1}, codex: {action: 'execute', testedHead: source, ordinal: 1}}),
      createClaim: async value => {
        claims.push(value.claim);
        return {ref: noEffectProof(identityOf('codex'), null).ledgerReference, claimCommit: '8'.repeat(40)};
      },
      ledgerWriter: () => ({}),
      now: () => observedAt
    });
    assert.equal(claims.length, 1);
    assert.equal(claim.tool, 'codex');
    const outcome = JSON.parse(readFileSync(path.join(environment.HIVE_ATTEMPT_DIRECTORY, outcomeFile('codex')), 'utf8'));
    assert.equal(outcome.state, clientStates.claimed);
    assert.equal(outcome.effects.ledgerHead, '8'.repeat(40));
    assert.equal(outcome.executionCredit, 0);
    assert.equal(readOutputs(environment.GITHUB_OUTPUT).execute, 'true');
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});

test('failed, cancelled, skipped or timed-out prerequisites refuse instead of resuming', async () => {
  for (const conclusion of ['failure', 'cancelled', 'skipped', 'timed_out', 'startup_failure']) {
    const result = await observeDeferredWakeups({repository, reader: wakeupReader(['agent', 'codex']), trigger: trigger(conclusion)});
    assert.equal(result.admitted, false);
    assert.equal(result.reason, 'prerequisite-failed');
    assert.deepEqual(result.candidate, {event: 'published-release', source, scope, verifiedRun});
    assert.deepEqual(result.resume.roles, ['agent', 'codex']);
    const classified = classifyRecheck(result);
    assert.equal(classified.state, clientStates.refused);
    assert.equal(classified.reason, 'prerequisite-failed');
    assert.equal(classified.prerequisites[0].conclusion, conclusion);
    assert.equal(admissionWakeup(result, {repository, run: ownerRun + 1, attempt: 1, observedAt}), null);
  }
});

test('no pending, skipped or cancelled run ever counts as passing', () => {
  const blocked = (status, conclusion) => ({admitted: false, reason: 'other CI pending',
    blockedBy: {...capture.diagnostic.blockedBy, status, conclusion}});
  for (const status of ['queued', 'in_progress', 'requested', 'waiting', 'pending']) {
    assert.equal(classifyRecheck(blocked(status, null)).state, clientStates.deferred);
  }
  for (const conclusion of ['failure', 'cancelled', 'skipped', 'timed_out', 'neutral', 'action_required', 'stale']) {
    assert.equal(classifyRecheck(blocked('completed', conclusion)).state, clientStates.refused);
  }
  assert.throws(() => classifyRecheck(blocked('completed', 'success')), /a passing prerequisite cannot block admission/u);
  assert.throws(() => classifyRecheck(blocked('completed', null)), /prerequisite conclusion metadata/u);
  assert.throws(() => classifyRecheck(blocked('in_progress', 'success')), /nonterminal prerequisite conclusion/u);
  assert.equal(classifyRecheck({admitted: false, reason: 'missing applicable CI .github/workflows/ci.yml'}).state, clientStates.deferred);
  assert.equal(classifyRecheck({admitted: false, reason: 'not-current-open-pull-request'}).state, clientStates.refused);
  // A deferred receipt cannot be forged around a completed prerequisite.
  const identity = identityOf('agent');
  assert.throws(() => validateOutcome({...deferredReceipt(identity, classifyRecheck(pending()),
    {observedAt, effects: noEffectProof(identity, null), rechecks: 1, generation: 1}),
  prerequisites: [{...capture.diagnostic.blockedBy, status: 'completed', conclusion: 'cancelled'}]}, identity),
  /a completed prerequisite cannot keep a leg deferred/u);
  const deferred = {state: clientStates.deferred, prerequisites: [capture.diagnostic.blockedBy]};
  const summary = summarizeOutcomes({claude: {action: 'execute'}, agent: {action: 'execute'}, codex: {action: 'execute'}},
    {claude: deferred, agent: deferred, codex: {state: clientStates.completed}});
  assert.equal(summary.complete, false);
  assert.equal(summary.clients.codex.state, 'unconfirmed-completion', 'a receipt alone never grants completion');
});

test('a changed source or superseded producer refuses and requires a new attempt identity', async () => {
  const superseded = await observeDeferredWakeups({repository, reader: wakeupReader(['agent'], {latest: verifiedRun + 1}),
    trigger: trigger()});
  assert.equal(superseded.reason, 'source-changed');
  assert.equal(superseded.requiresNewAttemptIdentity, true);
  assert.equal(superseded.supersededBy, verifiedRun + 1);
  const classified = classifyRecheck(superseded);
  assert.equal(classified.state, clientStates.refused);
  assert.equal(classified.requiresNewAttemptIdentity, true);
  const {directory, environment} = hosted('agent');
  try {
    const outcome = await claimAttempt(environment, {readerFactory: () => ({}), readLedger: async () => null,
      recheck: async () => ({admitted: false, reason: 'source-changed', supersededBy: verifiedRun + 1}),
      plan: forbidden('planner'), createClaim: forbidden('durable claim'), sleep: forbidden('recheck pause'), now: () => observedAt});
    assert.equal(outcome.state, clientStates.refused);
    assert.equal(outcome.requiresNewAttemptIdentity, true);
    assert.equal(existsSync(path.join(environment.HIVE_ATTEMPT_DIRECTORY, wakeupFile('agent'))), false);
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
  // A wakeup for a different candidate cannot be replayed onto this trigger.
  const forged = wakeupRecord({repository, run: ownerRun, attempt: 1, role: 'agent',
    candidate: {event: 'published-release', source, scope, verifiedRun}, prerequisites: [capture.diagnostic.blockedBy], observedAt, generation: 1});
  assert.throws(() => validateWakeup(forged, {repository, trigger: {id: blockingRun + 1}}), /wakeup must name the completed trigger run/u);
  assert.throws(() => validateWakeup({...forged, executionCredit: 1}, {repository, trigger: {id: blockingRun}}));
  assert.throws(() => validateWakeup({...forged, repository: 'foreign/repo'}, {repository, trigger: {id: blockingRun}}));
});

test('ambiguous prior effects are never turned into a deferral', async () => {
  const identity = identityOf('agent');
  const reference = noEffectProof(identity, null).ledgerReference;
  assert.throws(() => noEffectProof(identity, {ref: reference, head: '5'.repeat(40), claim: {run: ownerRun, attempt: 1}}),
    /ambiguous prior effects/u);
  // A claim held by a different run is proof that this leg claimed nothing.
  assert.deepEqual(noEffectProof(identity, {ref: reference, head: '5'.repeat(40), claim: {run: 1, attempt: 1}}),
    {claimed: false, ledgerReference: reference, ledgerHead: '5'.repeat(40), ledgerClaimRun: 1});
  const {directory, environment} = hosted('agent');
  try {
    await assert.rejects(claimAttempt(environment, {readerFactory: () => ({}), recheck: async () => pending(),
      readLedger: async () => ({ref: reference, head: '5'.repeat(40), claim: {run: ownerRun, attempt: 1}}),
      plan: forbidden('planner'), createClaim: forbidden('durable claim'), sleep: async () => {}, now: () => observedAt}),
    /ambiguous prior effects/u);
    assert.equal(existsSync(path.join(environment.HIVE_ATTEMPT_DIRECTORY, outcomeFile('agent'))), false);
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});

test('continuations and rechecks are bounded', async () => {
  assert.equal(nextContinuation({}), 1);
  assert.equal(nextContinuation({resume: {generation: maxContinuations - 1}}), maxContinuations);
  assert.equal(nextContinuation({resume: {generation: maxContinuations}}), null);
  assert.throws(() => nextContinuation({resume: {generation: maxContinuations + 1}}));
  const exhausted = await deferIncidentLeg('agent', {selection: selectionFor({resume: {generation: maxContinuations}})});
  try {
    assert.equal(exhausted.outcome.state, clientStates.refused);
    assert.equal(exhausted.outcome.reason, 'continuation-budget-exhausted');
    assert.equal(existsSync(path.join(exhausted.environment.HIVE_ATTEMPT_DIRECTORY, wakeupFile('agent'))), false);
  } finally {
    rmSync(exhausted.directory, {recursive: true, force: true});
  }
  const resumed = await deferIncidentLeg('codex', {selection: selectionFor({resume: {generation: 3}})});
  try {
    assert.equal(resumed.outcome.resume.generation, 4);
  } finally {
    rmSync(resumed.directory, {recursive: true, force: true});
  }
  assert.deepEqual(prerequisiteRecheckBudget({}), {checks: 2, delaySeconds: 10});
  assert.deepEqual(prerequisiteRecheckBudget({HIVE_PREREQUISITE_RECHECKS: '5', HIVE_PREREQUISITE_RECHECK_DELAY_SECONDS: '60'}),
    {checks: 5, delaySeconds: 60});
  for (const extra of [{HIVE_PREREQUISITE_RECHECKS: '0'}, {HIVE_PREREQUISITE_RECHECKS: '6'},
    {HIVE_PREREQUISITE_RECHECK_DELAY_SECONDS: '61'}, {HIVE_PREREQUISITE_RECHECKS: '-1'}]) {
    assert.throws(() => prerequisiteRecheckBudget(extra), /bounded prerequisite recheck setting/u);
  }
});

test('workflow consumes typed outcomes and keeps the existing admission and verification gates', () => {
  const workflow = YAML.parse(readFileSync(new URL('../../../.github/workflows/hive-original-delivery.yml', import.meta.url), 'utf8'));
  const step = (job, id) => workflow.jobs[job].steps.find(value => value.id === id);
  for (const job of ['verification', 'ledger']) {
    assert.equal(workflow.jobs[job].if, "${{ !cancelled() && needs.image.result == 'success' && needs.admission.outputs.execute == 'true' }}");
    assert.match(step(job, 'outcome').run, /hive-delivery-attempt\.mjs client-outcome/u);
    for (const value of workflow.jobs[job].steps.slice(workflow.jobs[job].steps.indexOf(step(job, 'outcome')) + 1)) {
      assert.match(value.if ?? '', /steps\.outcome\.outputs\.verify == 'true'/u, (value.name ?? value.uses) + ' is gated on a claimed outcome');
    }
  }
  assert.match(step('delivery', 'fallback').if, /always\(\) && steps\.claim\.outputs\.outcome_name == ''/u);
  assert.ok(workflow.jobs.delivery.steps.some(value => /steps\.claim\.outputs\.wakeup_name != ''/u.test(value.if ?? '')));
  const download = workflow.jobs.acceptance.steps.find(value => value.with?.pattern);
  assert.match(download.with.pattern, /^hive-outcome-.*-\*-.*\$\{\{ github\.run_attempt \}\}$/u);
  assert.equal(download.with['merge-multiple'], true);
  // Admission keeps its opt-in; only main-branch completions are added as continuation triggers.
  assert.match(workflow.jobs.admission.if, /^\(vars\.FORMAL_AI_REAL_GITHUB_E2E == 'true' \|\|/u);
  assert.match(workflow.jobs.admission.if, /github\.event\.workflow_run\.event != 'pull_request_target'/u);
});
