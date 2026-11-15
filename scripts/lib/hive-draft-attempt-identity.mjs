// Issue #1484: one attempt identity binds the selected Formal AI and Hive Mind sources to the
// authenticated binary, the exact image and what actually served the task. The contract mirrors
// link-assistant/hive-mind#3909 (schema hive-mind/formal-ai-draft-attempt-identity/v1): selection
// happens once, nothing falls back, and a missing or wrong artifact is a typed outcome with zero credit.
import {createHash} from 'node:crypto';

export const ATTEMPT_IDENTITY_SCHEMA = 'formal-ai/hive-draft-attempt-identity/v1';
export const ATTEMPT_RECEIPT_SCHEMA = 'formal-ai/hive-draft-attempt-receipt/v1';
export const HIVE_IDENTITY_SCHEMA = 'hive-mind/formal-ai-draft-attempt-identity/v1';
export const HIVE_RECORD_SCHEMA = 'hive-mind/formal-ai-draft-attempt-record/v1';
export const ATTEMPT_MODES = Object.freeze({
  liveCurrent: 'live-current',
  publishedRelease: 'published-release',
  historicalReplay: 'historical-replay',
});
export const ATTEMPT_OUTCOMES = Object.freeze({
  verified: 'verified',
  deferred: 'deferred-infrastructure',
  mismatch: 'identity-mismatch',
  servingUnverified: 'serving-identity-unverified',
});
/**
 * Session files written next to Hive Mind's own attempt-identity.json/attempt-record.json.
 *
 * Issue #1569: Hive Mind's typed outcome must not be named `<phase>-outcome.json`. The same directory
 * receives `scripts/capture-draft-command.mjs`'s own `attempt-outcome.json` capture record
 * (`draft-command-outcome/v1`), which it rewrites when the attempt ends. Under the old name every
 * attempt read that record as a Hive outcome and was reported as `deferred-infrastructure`,
 * "Hive Mind ended the attempt before solve", even when solve ran (run 43691120627).
 */
export const FILES = Object.freeze({
  identity: 'formal-ai-attempt-identity.json',
  outcome: 'formal-ai-attempt-outcome.json',
  receipt: 'formal-ai-attempt-receipt.json',
  hiveIdentity: 'attempt-identity.json',
  hiveRecord: 'attempt-record.json',
  hiveOutcome: 'attempt-hive-outcome.json',
});
/** The schema of the capture record `capture-draft-command.mjs` writes; never a Hive Mind outcome. */
export const CAPTURE_OUTCOME_SCHEMA = 'draft-command-outcome/v1';
const SHA1 = /^[0-9a-f]{40}$/u;
const SHA256 = /^[0-9a-f]{64}$/u;
const SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u;

export class AttemptIdentityError extends Error {
  constructor(outcome, message, evidence = {}) {
    super(message);
    this.name = 'AttemptIdentityError';
    this.outcome = outcome;
    this.evidence = evidence;
  }
}
const mismatch = (message, evidence) => new AttemptIdentityError(ATTEMPT_OUTCOMES.mismatch, message, evidence);
const deferred = (message, evidence) => new AttemptIdentityError(ATTEMPT_OUTCOMES.deferred, message, evidence);

/** The first release version in a `--version` answer such as `formal-ai 0.355.29`. */
export const versionOf = text => /\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?/u.exec(String(text ?? ''))?.[0] ?? null;

const canonicalJson = value => Array.isArray(value)
  ? '[' + value.map(canonicalJson).join(',') + ']'
  : value && typeof value === 'object'
    ? '{' + Object.keys(value).sort().map(key => JSON.stringify(key) + ':' + canonicalJson(value[key])).join(',') + '}'
    : JSON.stringify(value ?? null);

/**
 * The id covers the selection only: retries of the same run reuse it, while a different
 * Formal AI or Hive Mind source, image or run is a new attempt. The selection timestamp and
 * run attempt number are recorded but excluded so a retry cannot mint a new identity.
 */
const selectionOf = identity => ({
  schema: identity.schema, mode: identity.mode, repository: identity.repository, issue: identity.issue,
  formalAi: {repository: identity.formalAi.repository, sourceSha: identity.formalAi.sourceSha},
  hive: {repository: identity.hive.repository, sourceSha: identity.hive.sourceSha, image: identity.hive.image},
  runId: identity.selected.runId,
});
export const attemptIdentityId = identity => createHash('sha256').update(canonicalJson(selectionOf(identity))).digest('hex').slice(0, 32);

/**
 * Admit one live attempt. `selected` SHAs are what was resolved once from the default branches;
 * `checkedOut` SHAs are what the runner actually holds. Any difference is source drift.
 */
export function selectAttemptIdentity({mode = ATTEMPT_MODES.liveCurrent, repository, issue, formalAi, hive, selected}) {
  if (!Object.values(ATTEMPT_MODES).includes(mode)) throw mismatch('unknown attempt mode ' + JSON.stringify(mode));
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u.test(repository ?? '')) throw mismatch('invalid repository ' + JSON.stringify(repository));
  if (!Number.isSafeInteger(Number(issue)) || Number(issue) < 1) throw mismatch('invalid issue ' + JSON.stringify(issue));
  for (const [name, source] of [['Formal AI', formalAi], ['Hive Mind', hive]]) {
    if (!SHA1.test(source?.sourceSha ?? '')) throw mismatch(name + ' source must be a full commit SHA, received ' + JSON.stringify(source?.sourceSha));
    if (source.checkedOutSha !== undefined && source.checkedOutSha !== source.sourceSha)
      throw mismatch(name + ' checkout ' + source.checkedOutSha + ' drifted from the selected source ' + source.sourceSha, {selected: source.sourceSha, actual: source.checkedOutSha});
  }
  if (!SEMVER.test(hive.version ?? '')) throw mismatch('Hive Mind version must be a release version, received ' + JSON.stringify(hive.version));
  if (!hive.image || /:latest$/u.test(hive.image) || !hive.image.endsWith(':' + hive.version))
    throw mismatch('Hive Mind image ' + JSON.stringify(hive.image) + ' is not the exact image of version ' + hive.version + '; an attempt never runs :latest');
  if (!SHA256.test(formalAi.binarySha256 ?? '')) throw deferred('no authenticated Formal AI binary digest for ' + formalAi.sourceSha);
  if (!SEMVER.test(formalAi.version ?? '')) throw mismatch('the Formal AI binary reported no release version: ' + JSON.stringify(formalAi.version));
  const identity = {
    schema: ATTEMPT_IDENTITY_SCHEMA, id: null, mode,
    currentSource: mode === ATTEMPT_MODES.liveCurrent,
    repository, issue: Number(issue),
    formalAi: {repository: formalAi.repository ?? 'link-assistant/formal-ai', sourceSha: formalAi.sourceSha, version: formalAi.version,
      binarySource: formalAi.binarySource ?? 'build', binarySha256: formalAi.binarySha256, binaryBytes: formalAi.binaryBytes ?? null},
    hive: {repository: hive.repository ?? 'link-assistant/hive-mind', sourceSha: hive.sourceSha, version: hive.version, image: hive.image},
    selected: {at: selected?.at ?? null, runId: selected?.runId ?? null, runAttempt: selected?.runAttempt ?? null, runUrl: selected?.runUrl ?? null},
  };
  identity.id = attemptIdentityId(identity);
  return identity;
}

/** Re-read an identity handed between jobs; an edited selection no longer matches its id. */
export function parseAttemptIdentity(text) {
  let identity;
  try { identity = typeof text === 'string' ? JSON.parse(text) : text; } catch (error) { throw mismatch('the attempt identity is not JSON: ' + error.message); }
  if (identity?.schema !== ATTEMPT_IDENTITY_SCHEMA) throw mismatch('the attempt identity has schema ' + JSON.stringify(identity?.schema) + ', expected ' + ATTEMPT_IDENTITY_SCHEMA);
  const recomputed = attemptIdentityId(identity);
  if (identity.id !== recomputed) throw mismatch('the attempt identity ' + identity.id + ' was changed after admission (selection hashes to ' + recomputed + ')', {recorded: identity.id, recomputed});
  return identity;
}

/**
 * Before any task effect: the binary handed to Hive Mind must be the one admitted, and the
 * runner must still hold the selected sources and issue.
 */
export function verifyAttemptCustody({identity, binary, hiveCheckoutSha, issue}) {
  if (Number(issue) !== identity.issue) throw mismatch('the attempt identity is for issue #' + identity.issue + ', not #' + issue);
  if (hiveCheckoutSha !== identity.hive.sourceSha)
    throw mismatch('the Hive Mind checkout ' + hiveCheckoutSha + ' is not the selected source ' + identity.hive.sourceSha, {selected: identity.hive.sourceSha, actual: hiveCheckoutSha ?? null});
  if (!binary) throw deferred('the admitted Formal AI binary of ' + identity.formalAi.sourceSha + ' is unavailable; the attempt is deferred rather than run on another backend');
  if (binary.sha256 !== identity.formalAi.binarySha256)
    throw mismatch('the Formal AI binary hashes to ' + binary.sha256 + ', but the attempt admitted ' + identity.formalAi.binarySha256, {expected: identity.formalAi.binarySha256, actual: binary.sha256});
  if (versionOf(binary.version) !== identity.formalAi.version)
    throw mismatch('the Formal AI binary reports ' + JSON.stringify(binary.version) + ', but the attempt admitted ' + identity.formalAi.version);
  return {binarySha256: binary.sha256, version: identity.formalAi.version, hiveSourceSha: hiveCheckoutSha};
}

/**
 * After Hive Mind's admission and preflight: its identity and serving record must name the same
 * selected sources and the same executable. Without that record the attempt is not current-source
 * evidence. A typed Hive outcome (deferred/mismatch) is preserved, never upgraded.
 */
export function verifyHiveAttempt({identity, hiveIdentity = null, hiveRecord = null, hiveOutcome = null}) {
  const base = {schema: ATTEMPT_RECEIPT_SCHEMA, attemptId: identity.id, mode: identity.mode, formalAiSource: identity.formalAi.sourceSha,
    hiveSource: identity.hive.sourceSha, hiveImage: identity.hive.image, binarySha256: identity.formalAi.binarySha256, hiveAttemptId: hiveIdentity?.id ?? null};
  const result = (outcome, extra = {}) => {
    const receipt = {...base, outcome, servingVerified: outcome === ATTEMPT_OUTCOMES.verified, ...extra};
    return {...receipt, currentSourceCredit: currentSourceCredit(receipt)};
  };
  // A capture record is the wrapper's account of the process, not Hive Mind's verdict on the attempt.
  if (hiveOutcome?.schema === CAPTURE_OUTCOME_SCHEMA) hiveOutcome = null;
  if (hiveOutcome) return result([ATTEMPT_OUTCOMES.deferred, ATTEMPT_OUTCOMES.mismatch].includes(hiveOutcome.outcome) ? hiveOutcome.outcome : ATTEMPT_OUTCOMES.deferred, {reason: String(hiveOutcome.message ?? 'Hive Mind ended the attempt before solve')});
  if (!hiveIdentity || !hiveRecord)
    return result(ATTEMPT_OUTCOMES.servingUnverified, {reason: 'the shared Hive Mind checkout produced no ' + (hiveIdentity ? FILES.hiveRecord : FILES.hiveIdentity) + '; it predates the attempt identity contract (link-assistant/hive-mind#3909)'});
  const differences = [];
  const expect = (label, actual, expected) => { if (actual !== expected) differences.push({field: label, expected, actual: actual ?? null}); };
  expect('hive identity schema', hiveIdentity.schema, HIVE_IDENTITY_SCHEMA);
  expect('hive record schema', hiveRecord.schema, HIVE_RECORD_SCHEMA);
  expect('mode', hiveIdentity.mode, identity.mode);
  expect('repository', hiveIdentity.repository, identity.repository);
  expect('issue', hiveIdentity.issue, identity.issue);
  expect('formalAi.sourceSha', hiveIdentity.formalAi?.sourceSha, identity.formalAi.sourceSha);
  expect('formalAi.callerBinarySha256', hiveIdentity.formalAi?.callerBinarySha256, identity.formalAi.binarySha256);
  expect('hive.sourceSha', hiveIdentity.hive?.sourceSha, identity.hive.sourceSha);
  expect('hive.image', hiveIdentity.hive?.image, identity.hive.image);
  expect('record.attemptId', hiveRecord.attemptId, hiveIdentity.id);
  expect('record.binary.sha256', hiveRecord.binary?.sha256, identity.formalAi.binarySha256);
  expect('record.serving.servingExeSha256', hiveRecord.serving?.servingExeSha256, identity.formalAi.binarySha256);
  expect('record.serving.formalAiVersion', versionOf(hiveRecord.serving?.formalAiVersion), identity.formalAi.version);
  if (hiveRecord.serving?.hiveVersion) expect('record.serving.hiveVersion', hiveRecord.serving.hiveVersion, identity.hive.version);
  // Hive Mind binds the image by registry digest; a record without one names a tag, which can move.
  const imageDigest = String(hiveRecord.image?.digest ?? '').toLowerCase().replace(/^sha256:/u, '');
  if (!SHA256.test(imageDigest)) differences.push({field: 'record.image.digest', expected: 'sha256 registry digest', actual: hiveRecord.image?.digest ?? null});
  if (differences.length) return result(ATTEMPT_OUTCOMES.mismatch, {reason: 'Hive Mind ran a different selection than Formal AI admitted', differences});
  return result(ATTEMPT_OUTCOMES.verified, {imageReference: hiveRecord.image?.reference ?? null, imageDigest: 'sha256:' + imageDigest,
    imageRevision: hiveRecord.image?.revision ?? null, servingExeSha256: hiveRecord.serving.servingExeSha256,
    agentVersion: hiveRecord.serving?.agentVersion ?? null, servingCheckedAt: hiveRecord.serving?.probedAt ?? hiveRecord.verifiedAt ?? null});
}

/** Classify any draft receipt, including the #1364 historical replay ledger rows. */
export function attemptModeOf(receipt) {
  if (typeof receipt?.taskMode === 'string' && receipt.taskMode.startsWith('historical')) return ATTEMPT_MODES.historicalReplay;
  if (Object.values(ATTEMPT_MODES).includes(receipt?.mode)) return receipt.mode;
  throw Error('receipt has no attempt mode; it cannot be classified as current-source evidence');
}

/** Only a verified live-current attempt counts as current-source evidence; replays and releases never do. */
export function currentSourceCredit(receipt) {
  return attemptModeOf(receipt) === ATTEMPT_MODES.liveCurrent && receipt.outcome === ATTEMPT_OUTCOMES.verified && receipt.servingVerified === true;
}

/** Step-summary rows: the three evaluation kinds stay distinct on the dashboard. */
export function summarizeAttempt(identity, receipt) {
  const label = {[ATTEMPT_MODES.liveCurrent]: 'live current-source evaluation', [ATTEMPT_MODES.publishedRelease]: 'published-release qualification', [ATTEMPT_MODES.historicalReplay]: 'historical pinned replay'}[identity.mode];
  return ['### Formal AI draft attempt ' + identity.id, '', '| Field | Value |', '| --- | --- |',
    '| Kind | ' + label + ' (`' + identity.mode + '`) |',
    '| Formal AI | `' + identity.formalAi.sourceSha + '` (' + identity.formalAi.version + ') |',
    '| Binary | `sha256:' + identity.formalAi.binarySha256 + '` |',
    '| Hive Mind | `' + identity.hive.sourceSha + '` (' + identity.hive.version + ', `' + identity.hive.image + '`) |',
    '| Selected | ' + (identity.selected.at ?? 'unknown') + ' |',
    ...(receipt?.imageDigest ? ['| Served by | `' + receipt.servingExeSha256 + '` in `' + receipt.imageDigest + '` |'] : []),
    ...(receipt ? ['| Outcome | `' + receipt.outcome + '` |', '| Current-source credit | ' + (receipt.currentSourceCredit ? 'yes' : 'no') + ' |'] : []), ''].join('\n');
}
