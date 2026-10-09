// R1188-U31..U34: byte accounting is a proxy, never a proof of usefulness or cost.
import { createHash } from 'node:crypto';
import { tokenize } from '../../scripts/lib/translation-blockers.mjs';

export const sourceDigest = source => createHash('sha256').update(source).digest('hex');
const byteLength = value => Buffer.byteLength(value, 'utf8');
const categories = new Set(['coding', 'self-coding', 'mathematics', 'other']);
const origins = new Set(['autonomous', 'reviewed-patch', 'generator']);
const roles = new Set(['production', 'test', 'generated', 'evidence', 'documentation']);

function texts(values, name) {
  if (!Array.isArray(values) || values.some(value => typeof value !== 'string')) {
    throw new TypeError(name + ' must contain the complete observed strings');
  }
  return values.reduce((sum, value) => sum + byteLength(value), 0);
}

// Ignore comments, formatting and literal payload length. This intentionally
// undercounts useful deletions/refactorings. It cannot identify unused code;
// independent acceptance and review remain necessary, including test quality.
function codeWeight(source) {
  return tokenize(source).filter(token => token.kind !== 'comment').reduce((sum, token) =>
    sum + byteLength(['string', 'template', 'regex', 'number'].includes(token.kind)
      ? '<' + token.kind + '>' : token.value), 0);
}

/** A verifier executes fresh, independent checks and binds their actual source bytes. */
export async function measureCodingRun(run, verify) {
  if (typeof run.runId !== 'string' || !run.runId) throw new TypeError('stable run identity is required');
  if (!categories.has(run.taskKind) || !origins.has(run.origin)) throw new TypeError('unknown task kind or origin');
  if (typeof run.task !== 'string' || run.task.length === 0) throw new TypeError('original task is required');
  if (typeof verify !== 'function') throw new TypeError('an independent verifier is required');
  const attempts = run.attemptInputs;
  if (!Array.isArray(attempts) || attempts[0] !== run.task) throw new TypeError('retain the original first attempt');
  const taskInputBytes = byteLength(run.task);
  const instructionBytes = texts(attempts, 'attemptInputs');
  const repositoryContextBytes = texts(run.repositoryContext, 'repositoryContext');
  const reviewedPatchBytes = texts(run.reviewedPatches, 'reviewedPatches');
  const toolReceiptBytes = texts(run.toolReceipts, 'toolReceipts');
  if (!Array.isArray(run.changes)) throw new TypeError('changes must be observed before/after files');
  const paths = new Set();
  let beforeCodeBytes = 0;
  let afterCodeBytes = 0;
  let excludedChangedBytes = 0;
  const bindings = [];
  for (const change of run.changes) {
    if (typeof change.path !== 'string' || !change.path || paths.has(change.path)) throw new TypeError('unique source paths required');
    paths.add(change.path);
    if (!roles.has(change.role) || typeof change.before !== 'string' || typeof change.after !== 'string') {
      throw new TypeError('explicit change role and exact source bytes required');
    }
    bindings.push({ path: change.path, before: sourceDigest(change.before), after: sourceDigest(change.after) });
    if (change.role === 'production' || change.role === 'test') {
      if (change.language !== 'javascript') throw new TypeError('code byte proxy currently supports JavaScript only');
      beforeCodeBytes += codeWeight(change.before);
      afterCodeBytes += codeWeight(change.after);
    } else if (change.before !== change.after) {
      excludedChangedBytes += byteLength(change.before) + byteLength(change.after);
    }
  }
  let proof;
  let verifierFailure = null;
  try { proof = await verify(run.changes); } catch (error) {
    verifierFailure = { name: error?.name ?? 'Error', message: String(error?.message ?? error) };
    proof = { passed: false, bindings: [], checks: [{ name: 'independent verifier threw', exitCode: null }] };
  }
  const sourceBound = Array.isArray(proof?.bindings) && proof.bindings.length === bindings.length
    && bindings.every(binding => proof.bindings.some(item => item.path === binding.path
      && item.before === binding.before && item.after === binding.after));
  const checksPassed = Array.isArray(proof?.checks) && proof.checks.length > 0
    && proof.checks.every(check => typeof check.name === 'string' && check.name.length > 0 && check.exitCode === 0);
  const accepted = proof?.passed === true && sourceBound && checksPassed;
  const eligible = run.taskKind === 'coding' || run.taskKind === 'self-coding';
  const rawNetCodeBytes = afterCodeBytes - beforeCodeBytes;
  const validatedNetCodeBytes = accepted && eligible ? Math.max(0, rawNetCodeBytes) : 0;
  const autonomous = run.origin === 'autonomous' && reviewedPatchBytes === 0;
  const autonomousNetCodeBytes = autonomous ? validatedNetCodeBytes : 0;
  const totalObservedInputBytes = instructionBytes + repositoryContextBytes + reviewedPatchBytes + toolReceiptBytes;
  return {
    runId: run.runId, taskKind: run.taskKind, origin: run.origin, accepted, sourceBound, checksPassed, verifierFailure,
    taskInputBytes, instructionBytes, repositoryContextBytes, reviewedPatchBytes, toolReceiptBytes,
    totalObservedInputBytes, attempts: attempts.length, beforeCodeBytes, afterCodeBytes,
    rawNetCodeBytes, removedCodeBytes: Math.max(0, -rawNetCodeBytes), excludedChangedBytes,
    validatedNetCodeBytes, autonomousNetCodeBytes,
    taskAmplification: eligible ? autonomousNetCodeBytes / taskInputBytes : null,
    observedInputAmplification: eligible ? autonomousNetCodeBytes / totalObservedInputBytes : null,
    qualityRequiresReview: true, monetarySavings: null, modelTokens: null,
    bindings, checks: proof?.checks ?? [],
  };
}

/** Include every attempted coding task, including failures and zero-growth repairs. */
export function summarizeCodingRuns(measurements, declaredRunIds) {
  if (!Array.isArray(declaredRunIds) || declaredRunIds.some(value => typeof value !== 'string' || !value)
    || new Set(declaredRunIds).size !== declaredRunIds.length) throw new TypeError('unique declared cohort identities required');
  const observedIds = measurements.map(item => item.runId);
  if (new Set(observedIds).size !== observedIds.length) throw new TypeError('duplicate run in cohort');
  if (observedIds.length !== declaredRunIds.length || observedIds.some(value => !declaredRunIds.includes(value))) {
    throw new TypeError('cohort must retain every declared run and no undeclared run');
  }
  const coding = measurements.filter(item => item.taskKind === 'coding' || item.taskKind === 'self-coding');
  const selfCoding = coding.filter(item => item.taskKind === 'self-coding');
  const summary = items => ({
    attempted: items.length,
    accepted: items.filter(item => item.accepted).length,
    amplified: items.filter(item => item.accepted && item.taskAmplification > 1).length,
    usuallyAmplifies: items.length > 0 && items.filter(item => item.accepted && item.taskAmplification > 1).length > items.length / 2,
  });
  return { coding: summary(coding), selfCoding: summary(selfCoding), monetarySavings: null };
}
