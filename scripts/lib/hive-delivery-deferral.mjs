// Typed pre-effect client outcomes (issue #1485).
// An unclaimed client leg whose prerequisites are still settling leaves a durable
// deferred-before-effects receipt. Downstream jobs read that receipt instead of
// looking for claim/result artifacts that were never produced. The receipt gives
// no execution or authorship credit. Resumption is a new admission of the same
// immutable candidate, so the existing claim ledger still decides which clients
// may run.
import assert from 'node:assert/strict';
import { ledgerReference } from './hive-delivery-ledger.mjs';

export const outcomeSchema = 'formal-ai-hive-client-outcome/v1';
export const wakeupSchema = 'formal-ai-hive-deferred-wakeup/v1';
/** The states a client leg can be in, before or after its effects start. */
export const clientStates = Object.freeze({
  eligible: 'eligible',
  deferred: 'deferred-before-effects',
  refused: 'refused',
  claimed: 'claimed',
  completed: 'completed',
  interrupted: 'interrupted'
});
/** Event-driven continuations of one candidate are finite; the ninth deferral refuses instead. */
export const maxContinuations = 8;
const terminal = new Set(['completed']);
const pendingStatuses = new Set(['queued', 'in_progress', 'requested', 'waiting', 'pending']);
const conclusions = new Set(['success', 'failure', 'neutral', 'cancelled', 'skipped', 'timed_out', 'action_required', 'stale', 'startup_failure']);
const tools = ['claude', 'agent', 'codex'];
const commit = value => assert.match(value ?? '', /^[a-f0-9]{40}$/u);
const positive = value => assert.ok(Number.isSafeInteger(value) && value > 0);
const sha256 = value => assert.match(value ?? '', /^[a-f0-9]{64}$/u);
const workflowPath = value => assert.match(value ?? '', /^\.github\/workflows\/[A-Za-z0-9_.-]+\.ya?ml$/u);

function prerequisite(value) {
  if (value.status === 'missing') {
    workflowPath(value.path);
    return {path: value.path, status: 'missing', conclusion: null};
  }
  positive(value.run);
  positive(value.attempt);
  workflowPath(value.path);
  assert.equal(typeof value.event, 'string');
  assert.ok(pendingStatuses.has(value.status) || terminal.has(value.status), 'prerequisite status metadata');
  if (value.status === 'completed') assert.ok(conclusions.has(value.conclusion), 'prerequisite conclusion metadata');
  else assert.equal(value.conclusion, null, 'nonterminal prerequisite conclusion');
  return {run: value.run, attempt: value.attempt, path: value.path, event: value.event,
    status: value.status, conclusion: value.conclusion};
}

/**
 * Map a fresh recheck result to the state a client leg is in before any effects.
 * Only a successful recheck is eligible. A pending or missing prerequisite defers
 * the leg. Any completed, non-successful prerequisite (failure, cancelled,
 * skipped and so on) refuses it, and is never treated as passing.
 */
export function classifyRecheck(fresh) {
  assert.equal(typeof fresh, 'object');
  if (fresh.admitted === true) return {state: clientStates.eligible, reason: null, prerequisites: []};
  assert.equal(fresh.admitted, false);
  assert.equal(typeof fresh.reason, 'string');
  if (fresh.reason === 'source-changed') {
    return {state: clientStates.refused, reason: 'source-changed', prerequisites: [], requiresNewAttemptIdentity: true};
  }
  if (fresh.blockedBy) {
    const blocking = prerequisite(fresh.blockedBy);
    if (blocking.status !== 'completed') return {state: clientStates.deferred, reason: fresh.reason, prerequisites: [blocking]};
    assert.notEqual(blocking.conclusion, 'success', 'a passing prerequisite cannot block admission');
    return {state: clientStates.refused, reason: 'prerequisite-failed', prerequisites: [blocking]};
  }
  const missing = /^missing applicable CI (\S+)$/u.exec(fresh.reason);
  if (missing) return {state: clientStates.deferred, reason: fresh.reason, prerequisites: [prerequisite({path: missing[1], status: 'missing'})]};
  return {state: clientStates.refused, reason: fresh.reason, prerequisites: []};
}

/** Identity fields that tie an outcome to one exact client leg of one attempt. */
export function outcomeIdentity({repository, selection, tool, run, attempt}) {
  assert.ok(tools.includes(tool));
  positive(run);
  assert.ok([1, 2].includes(attempt));
  commit(selection.selected.source);
  sha256(selection.policyHash);
  sha256(selection.inputHash);
  const candidate = {
    event: selection.selected.event,
    source: selection.selected.source,
    scope: selection.selected.scope,
    ...(selection.selected.number ? {pullRequest: selection.selected.number} : {}),
    ...(selection.publicDownloadVerification?.run ? {verifiedRun: selection.publicDownloadVerification.run} : {})
  };
  return {
    repository,
    scope: selection.selected.scope,
    tool,
    run,
    attempt,
    testedHead: selection.selected.source,
    protocolCommit: selection.producer.protocol,
    policyHash: selection.policyHash,
    inputHash: selection.inputHash,
    candidate
  };
}

/**
 * Proof that this leg claimed no task effects: the durable ledger for this exact
 * client either has no record, or has one that belongs to a different run.
 * A ledger record for this run means effects may have started. That is
 * ambiguous, so it throws instead of producing a deferral.
 */
export function noEffectProof(identity, durable) {
  const reference = ledgerReference(identity);
  if (durable) {
    assert.equal(durable.ref, reference);
    assert.notEqual(durable.claim.run, identity.run, 'ambiguous prior effects: this run already holds a durable claim');
  }
  return {
    claimed: false,
    ledgerReference: reference,
    ledgerHead: durable?.head ?? null,
    ledgerClaimRun: durable?.claim.run ?? null
  };
}

export function deferredReceipt(identity, classification, {observedAt, effects, rechecks, generation = null}) {
  assert.equal(classification.state, clientStates.deferred);
  assert.ok(classification.prerequisites.length > 0, 'a deferral names its prerequisites');
  assert.equal(effects.claimed, false);
  assert.ok(Number.isSafeInteger(rechecks) && rechecks > 0);
  const receipt = {
    schema: outcomeSchema,
    state: clientStates.deferred,
    ...identity,
    observedAt: new Date(observedAt).toISOString(),
    reason: classification.reason,
    prerequisites: classification.prerequisites.map(prerequisite),
    rechecks,
    effects,
    resume: identity.candidate.event === 'published-release'
      ? {via: 'workflow_run', wakeupRuns: classification.prerequisites.filter(value => value.run).map(value => value.run),
        sameCandidate: true, generation}
      : {via: 'workflow_dispatch', pullRequest: identity.candidate.pullRequest, sameCandidate: true},
    executionCredit: 0,
    authorshipCredit: 0
  };
  return validateOutcome(receipt, identity);
}

export function refusedReceipt(identity, classification, {observedAt, effects}) {
  assert.equal(classification.state, clientStates.refused);
  assert.equal(effects.claimed, false);
  return validateOutcome({
    schema: outcomeSchema,
    state: clientStates.refused,
    ...identity,
    observedAt: new Date(observedAt).toISOString(),
    reason: classification.reason,
    prerequisites: classification.prerequisites.map(prerequisite),
    requiresNewAttemptIdentity: classification.requiresNewAttemptIdentity === true,
    effects,
    executionCredit: 0,
    authorshipCredit: 0
  }, identity);
}

/** A non-executing planner decision (reuse a prior success, or a ledger refusal). */
export function plannedReceipt(identity, decision, {observedAt}) {
  assert.ok(['reuse', 'refuse'].includes(decision.action));
  const reused = decision.action === 'reuse';
  return validateOutcome({
    schema: outcomeSchema,
    state: reused ? clientStates.completed : clientStates.refused,
    ...identity,
    observedAt: new Date(observedAt).toISOString(),
    reason: reused ? 'reused-prior-success' : decision.reason,
    prerequisites: [],
    ...(reused ? {completedBy: {run: decision.run, attempt: decision.attempt, testedHead: decision.testedHead}} : {}),
    effects: {claimed: false, ledgerReference: ledgerReference(identity)},
    executionCredit: 0,
    authorshipCredit: 0
  }, identity);
}

export function claimedReceipt(identity, claim, {observedAt}) {
  for (const key of ['repository', 'scope', 'tool', 'run', 'attempt', 'testedHead', 'policyHash', 'inputHash']) {
    assert.equal(claim[key], identity[key], 'claimed outcome bound to claim ' + key);
  }
  return validateOutcome({
    schema: outcomeSchema,
    state: clientStates.claimed,
    ...identity,
    observedAt: new Date(observedAt).toISOString(),
    reason: null,
    prerequisites: [],
    effects: {claimed: true, ledgerReference: ledgerReference(identity), ledgerHead: claim.ledger?.claimCommit ?? null},
    executionCredit: 0,
    authorshipCredit: 0
  }, identity);
}

/** If the claim step ends without an outcome, the ledger decides between refused and interrupted. */
export function fallbackReceipt(identity, durable, {observedAt}) {
  const reference = ledgerReference(identity);
  if (durable) assert.equal(durable.ref, reference);
  const owned = durable?.claim.run === identity.run && durable?.claim.attempt === identity.attempt;
  return validateOutcome({
    schema: outcomeSchema,
    state: owned ? clientStates.interrupted : clientStates.refused,
    ...identity,
    observedAt: new Date(observedAt).toISOString(),
    reason: owned ? 'unknown-after-effects' : 'failed-before-claim',
    prerequisites: [],
    requiresNewAttemptIdentity: !owned,
    effects: owned ? {claimed: true, ledgerReference: reference, ledgerHead: durable.head}
      : {claimed: false, ledgerReference: reference, ledgerHead: durable?.head ?? null, ledgerClaimRun: durable?.claim.run ?? null},
    executionCredit: 0,
    authorshipCredit: 0
  }, identity);
}

/** Check an outcome receipt against the exact leg identity that consumes it. */
export function validateOutcome(value, identity) {
  assert.equal(value.schema, outcomeSchema);
  assert.ok(Object.values(clientStates).includes(value.state) && value.state !== clientStates.eligible,
    'eligible is transient and never persisted');
  for (const key of ['repository', 'scope', 'tool', 'run', 'attempt', 'testedHead', 'protocolCommit', 'policyHash', 'inputHash']) {
    assert.equal(value[key], identity[key], 'outcome bound to ' + key);
  }
  assert.deepEqual(value.candidate, identity.candidate, 'outcome bound to the same immutable candidate');
  assert.ok(!Number.isNaN(Date.parse(value.observedAt)), 'outcome timestamp');
  assert.equal(value.executionCredit, 0, 'a typed outcome never grants execution credit');
  assert.equal(value.authorshipCredit, 0);
  assert.ok(Array.isArray(value.prerequisites));
  value.prerequisites.forEach(prerequisite);
  assert.equal(value.effects.ledgerReference, ledgerReference(identity));
  if (value.state === clientStates.deferred) {
    assert.equal(value.effects.claimed, false, 'deferred legs never claimed effects');
    assert.ok(value.prerequisites.length > 0);
    assert.ok(value.prerequisites.every(item => item.status !== 'completed'),
      'a completed prerequisite cannot keep a leg deferred');
  }
  if (value.state === clientStates.refused || value.state === clientStates.completed) assert.equal(value.effects.claimed, false);
  if (value.state === clientStates.claimed || value.state === clientStates.interrupted) assert.equal(value.effects.claimed, true);
  return value;
}

export function outcomeName(scope, tool, policyHash, attempt) {
  assert.match(scope, /^(?:pr-[1-9]\d*|release-v\d+\.\d+\.\d+)$/u);
  assert.ok(tools.includes(tool));
  sha256(policyHash);
  assert.ok([1, 2].includes(attempt));
  return 'hive-outcome-' + scope + '-' + tool + '-' + policyHash + '-' + attempt;
}

/** Wakeup artifacts are found by the exact blocking run id when that run completes. */
export function wakeupName(blockingRun, role) {
  positive(blockingRun);
  assert.ok([...tools, 'admission'].includes(role));
  return 'hive-wakeup-' + blockingRun + '-' + role;
}

/** The continuation generation a new deferral of this selection would carry, or null when exhausted. */
export function nextContinuation(selection) {
  const previous = selection.resume?.generation ?? 0;
  assert.ok(Number.isSafeInteger(previous) && previous >= 0 && previous <= maxContinuations, 'continuation generation');
  return previous < maxContinuations ? previous + 1 : null;
}

export function wakeupRecord({repository, run, attempt, role, candidate, prerequisites, observedAt, generation}) {
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  positive(run);
  assert.ok(Number.isSafeInteger(generation) && generation >= 1 && generation <= maxContinuations, 'bounded continuation generation');
  assert.ok([1, 2].includes(attempt));
  assert.ok([...tools, 'admission'].includes(role));
  assert.equal(candidate.event, 'published-release', 'only published candidates resume from completion events');
  const pending = prerequisites.map(prerequisite).filter(item => item.run && item.status !== 'completed');
  assert.ok(pending.length > 0, 'a wakeup names a pending prerequisite run');
  return {
    schema: wakeupSchema,
    repository,
    run,
    attempt,
    role,
    candidate: {event: candidate.event, source: candidate.source, scope: candidate.scope, verifiedRun: candidate.verifiedRun},
    prerequisites: pending,
    generation,
    observedAt: new Date(observedAt).toISOString(),
    executionCredit: 0,
    authorshipCredit: 0
  };
}

export function validateWakeup(value, {repository, trigger}) {
  assert.equal(value.schema, wakeupSchema);
  assert.equal(value.repository, repository);
  positive(value.run);
  assert.ok([...tools, 'admission'].includes(value.role));
  assert.equal(value.candidate.event, 'published-release');
  commit(value.candidate.source);
  positive(value.candidate.verifiedRun);
  assert.match(value.candidate.scope ?? '', /^release-v\d+\.\d+\.\d+$/u);
  assert.ok(Number.isSafeInteger(value.generation) && value.generation >= 1 && value.generation <= maxContinuations,
    'bounded continuation generation');
  assert.equal(value.executionCredit, 0);
  assert.equal(value.authorshipCredit, 0);
  const named = value.prerequisites.map(prerequisite).filter(item => item.run === trigger.id);
  assert.equal(named.length, 1, 'wakeup must name the completed trigger run');
  return value;
}

/**
 * Summarize the per-client typed outcomes for the final gate. Credit only comes
 * from the ledger planner. A deferral is reported as incomplete; it is never
 * reported as accepted.
 */
export function summarizeOutcomes(decisions, outcomes) {
  const clients = {};
  for (const tool of tools) {
    const decision = decisions[tool];
    const outcome = outcomes[tool] ?? null;
    clients[tool] = {
      decision,
      // Completion credit only comes from the durable ledger planner, never from a receipt alone.
      state: decision.action === 'reuse' ? clientStates.completed
        : outcome?.state === clientStates.completed ? 'unconfirmed-completion'
        : outcome?.state ?? (decision.action === 'refuse' ? clientStates.refused : 'missing-outcome'),
      ...(outcome?.prerequisites?.length ? {prerequisites: outcome.prerequisites} : {}),
      ...(outcome?.reason ?? decision.reason ? {reason: outcome?.reason ?? decision.reason} : {})
    };
  }
  const states = Object.values(clients).map(value => value.state);
  const complete = states.every(state => state === clientStates.completed);
  const deferredOnly = !complete && states.every(state => state === clientStates.completed || state === clientStates.deferred);
  return {complete, deferred: deferredOnly, clients};
}
