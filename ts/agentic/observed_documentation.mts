// One custody/effect state machine for documentation derived from one or more
// observed sources. Callers supply a declared task contract and a pure derivation;
// the derivation never confers authority or independent acceptance.
import { WorkItemFileProgress, fileIoReadStep, fileIoEffectStep, fileIoPendingStep } from './work_item_file_io.mjs';
import { Capability } from './capability.mjs';
import { FinalDisposition } from './plan.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { workItemRequirements } from './work_item_requirements.mjs';
import { refusal, workItemRecordFinal } from './work_item_report.mjs';
import { sourceDocumentationPolicy } from './source_documentation_policy.mjs';
import { documentationClaims, judgedClaims, sourceAnchoredFacts, preservedPreimage, addedLinesDiff, judgedRequirements } from './documentation_acceptance.mjs';
const maximumBytes = 262144;
function argumentsOf(attempt) {
  try {
    return JSON.parse(attempt.arguments);
  } catch {
    return null;
  }
}
function alias(value, names) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) return null;
  const values = names.filter(name => Object.hasOwn(value, name)).map(name => value[name]);
  return values.length && values.every(item => typeof item === 'string' && item === values[0]) ? values[0] : null;
}
const ownerOf = attempt => alias(argumentsOf(attempt), ['filePath', 'file_path', 'path']);
const complete = attempt => attempt?.succeeded === true && attempt.source_read?.error === null && attempt.source_read.complete === true &&
         typeof attempt.source_read.source === 'string' && new TextEncoder().encode(attempt.source_read.source).length <= maximumBytes ? attempt.source_read.source : null;
const ownedPath = path => typeof path === 'string' && /^[A-Za-z0-9_][A-Za-z0-9_./-]*$/u.test(path) && path.split('/').every(part => part !== '' && part !== '.' && part !== '..');
/**
 * The output was produced by the supplied documentation derivation, not
 * written autonomously; later reviewer work is not observed.
 */
const authorshipBoundary = observed => ({
  mechanism: 'supplied-documentation-derivation',
  targetOutputObserved: observed,
  autonomousCredit: 0,
  laterReviewerWork: 'not-observed'
});
export function planObservedDocumentation(task, messages, tools, contract, derive) {
  const sources = contract?.sources,
    target = contract?.target;
  const progress = WorkItemFileProgress.scan(messages),
    attempts = progress.attempts;
  // The work item is split into requirements before any evidence is read, so
  // a refusal names the requirement it concerns (issue #1560).
  const requirements = workItemRequirements(task).map(([text, span]) => ({
    text,
    span,
    kind: 'source-documentation',
    status: 'unverified'
  }));
  const state = {
    schema: contract?.schema ?? 'source-documentation-controller/v1',
    task,
    taskIdentity: sha256Hex(task),
    workItem: contract?.workItem ?? null,
    requirements,
    source: sources?.length === 1 ? sources[0] : null,
    sources,
    target,
    observations: [],
    candidateEdits: [],
    checks: [],
    effects: [],
    unresolved: [],
    remainingWork: ['independent-requirement-acceptance'],
    transportDiagnostics: progress.transportDiagnostics,
    authored: false,
    verified: false,
    complete: false
  };
  const finish = (reason, failure = false, refused = null) => workItemRecordFinal({
    ...state,
    reason,
    ...(refused === null ? {} : { refused })
  }, task, failure ? FinalDisposition.Failure : state.complete ? FinalDisposition.Finding : FinalDisposition.Gap, 'source-documentation-controller');
  // A refused read names the path and the requirement it concerns.
  const unplanned = (gap, path) => gap === 'ReadPolicyRefused' ? finish(gap, false, refusal(requirements, path)) : finish(gap);
  if (!Array.isArray(sources) || !sources.length || sources.length > 16 || !sources.every(ownedPath) || !ownedPath(target) || new Set([...sources, target]).size !== sources.length + 1 || typeof derive !== 'function') return finish('DocumentationContractInvalid', true);
  if (contract.authority !== 'owned') return finish(contract.authority ?? 'DocumentationActionUnowned');
  const pending = fileIoPendingStep(progress, tools);
  if (pending) return pending.step ?? finish(pending.gap);
  const read = path => {
    const planned = fileIoReadStep(messages, tools, path, task);
    return planned.step ?? unplanned(planned.gap, path);
  };
  if (attempts.some(attempt => attempt.capability === Capability.Read && ownerOf(attempt) === null)) return finish('AmbiguousReadPathAttempt', true);
  const effects = attempts.map((attempt, index) => ({
    attempt,
    index
  })).filter(({
    attempt
  }) => [Capability.Edit, Capability.Write].includes(attempt.capability));
  if (effects.some(({
    attempt
  }) => attempt.capability !== Capability.Edit || ownerOf(attempt) !== target)) return finish('UnrelatedOrAmbiguousMutationAttempt', true);
  if (effects.length > 1) return finish('MultipleEditAttempts', true);
  const effect = effects[0] ?? null;
  const reads = (path, before = attempts.length, after = -1) => attempts.map((attempt, index) => ({
    attempt,
    index
  })).filter(item => item.index < before && item.index > after && item.attempt.capability === Capability.Read && ownerOf(item.attempt) === path);
  for (const path of [...sources, target]) {
    const observed = reads(path).at(-1);
    if (!observed) return read(path);
    const content = complete(observed.attempt);
    if (content === null) return finish('IncompleteOrOversizedSourceObservation');
    state.observations.push({
      path,
      sourceIdentity: sha256Hex(content),
      complete: true
    });
  }
  const frozenSources = sources.map(path => {
    const observed = reads(path, effect?.index ?? attempts.length).find(item => complete(item.attempt) !== null);
    const source = complete(observed?.attempt);
    return {
      path,
      source,
      sourceIdentity: source === null ? null : sha256Hex(source)
    };
  });
  const guideRead = reads(target, effect?.index ?? attempts.length).at(-1),
    guide = complete(guideRead?.attempt);
  if (guide === null || frozenSources.some(row => row.source === null)) return finish('EditPreconditionObservationMissing', effect !== null);
  // The first complete observation is immutable task evidence. A changed later
  // observation does not silently retarget a candidate from another snapshot.
  for (const source of frozenSources) {
    const latest = complete(reads(source.path, effect?.index ?? attempts.length).at(-1)?.attempt);
    if (latest === null || sha256Hex(latest) !== source.sourceIdentity) return finish('SourceFreshnessMismatch', true);
  }
  let derived;
  try {
    derived = derive(frozenSources, guide);
  } catch {
    return finish('DocumentationDerivationUnavailable', effect !== null);
  }
  if (!derived || typeof derived !== 'object' || Array.isArray(derived) || derived.gap !== undefined &&
         (typeof derived.gap !== 'string' || derived.gap.length === 0 || derived.gap.length > 128) || derived.remainingWork !== undefined &&
         (!Array.isArray(derived.remainingWork) || derived.remainingWork.length > 64 || derived.remainingWork.some(value => typeof value !== 'string')) ||
         derived.facts !== undefined && (!Array.isArray(derived.facts) || derived.facts.length > 256)) return finish('DocumentationDerivationInvalid',
         true);
  if (derived.gap) return finish(derived.gap, effect !== null);
  if (typeof derived.content !== 'string' || new TextEncoder().encode(derived.content).length > maximumBytes) return finish('DocumentationCandidateBudgetExceeded', true);
  state.binding = derived.binding ?? {
    key: derived.key,
    value: derived.value,
    sourceIdentity: derived.sourceIdentity
  };
  if (derived.facts) state.facts = derived.facts;
  if (derived.remainingWork) state.remainingWork = [...new Set([...state.remainingWork, ...derived.remainingWork])];
  // Independent acceptance (issue #1483): the task's version claims are judged
  // against the observed declarations before any edit, and the artifact is
  // checked against the raw sources, never against the derivation's word.
  const policy = sourceDocumentationPolicy();
  const claims = state.facts ? judgedClaims(documentationClaims(task), state.facts) : [];
  const accept = (artifact, preservation) => {
    const anchored = state.facts ? sourceAnchoredFacts(state.facts, frozenSources, artifact) : null;
    if (anchored) state.checks.push(anchored);
    if (preservation) state.checks.push(preservation);
    state.requirements = judgedRequirements(requirements, {
      facts: state.facts,
      claims,
      target,
      factsPassed: anchored?.status === 'passed',
      preservation: preservation?.status ?? null,
      preserve: policy?.preserve ?? [],
      negations: policy?.negations ?? []
    });
    state.verified = anchored?.status === 'passed';
    state.complete = state.verified && state.requirements.every(row => row.status === 'verified') &&
      state.remainingWork.every(item => item === 'independent-requirement-acceptance');
    if (state.complete) state.remainingWork = [];
  };
  if (claims.length) {
    state.claims = claims;
    const contradicted = claims.filter(claim => claim.status === 'contradicted');
    state.checks.push({
      kind: 'task-claims',
      status: contradicted.length ? 'failed' : 'passed',
      confirmed: claims.filter(claim => claim.status === 'confirmed').length,
      contradicted: contradicted.length,
      unobserved: claims.filter(claim => claim.status === 'unobserved').length
    });
    if (contradicted.length) {
      // A contradicted claim is rejected: no edit documents it, and the
      // requirements naming it are contradicted rather than left open.
      state.unresolved = contradicted.map(claim => ({
        reference: claim.reference,
        reason: 'ContradictedIntegrationClaim',
        observed: claim.observed
      }));
      state.requirements = judgedRequirements(requirements, { facts: state.facts, claims, target, factsPassed: false, preservation: null, preserve: [], negations: [] });
      return finish('ContradictedIntegrationClaim', effect !== null);
    }
  }
  const preserved = preservedPreimage(guide, derived.content);
  if (effect) {
    const argumentsValue = argumentsOf(effect.attempt),
      old = alias(argumentsValue, ['oldString', 'old_string', 'old_str', 'old']),
      next = alias(argumentsValue, ['newString', 'new_string', 'new_str', 'new']);
    state.candidateEdits.push({
      path: target,
      preimageIdentity: sha256Hex(guide),
      postimageIdentity: sha256Hex(derived.content),
      diff: addedLinesDiff(preserved.added)
    });
    if (old !== guide || next !== derived.content || next === old) return finish('UnrelatedEditCandidate', true);
    if (!effect.attempt.succeeded) return finish('CandidateEditFailed', true);
    let lastFreshIndex = effect.index;
    for (const source of frozenSources) {
      const fresh = reads(source.path, attempts.length, effect.index).at(-1);
      if (!fresh) return read(source.path);
      const content = complete(fresh.attempt);
      if (content === null || sha256Hex(content) !== source.sourceIdentity) return finish('SourceFreshnessMismatch', true);
      lastFreshIndex = Math.max(lastFreshIndex, fresh.index);
    }
    const observed = reads(target, attempts.length, lastFreshIndex).at(-1);
    if (!observed) return read(target);
    const actual = complete(observed.attempt);
    if (actual === null || actual !== derived.content) return finish('ReadbackMismatch', true);
    state.effects.push({
      path: target,
      kind: 'edit',
      reportedSuccess: true,
      observedPostimageIdentity: sha256Hex(actual)
    });
    state.checks.push({
      kind: 'source-freshness-and-exact-postimage',
      passed: true
    });
    state.authored = true;
    state.authorshipBoundary = authorshipBoundary(true);
    accept(actual, preserved.check);
    return finish('UsefulCandidateReadbackVerified');
  }
  if (derived.content === guide) {
    state.checks.push({
      kind: 'source-bound-noop',
      passed: true
    });
    state.authorshipBoundary = authorshipBoundary(false);
    accept(guide, null);
    return finish('VerifiedNoOp');
  }
  const planned = fileIoEffectStep(messages, tools, target, guide, derived.content);
  return planned.step ?? finish(planned.gap);
}

// Shared custody validation; native owner_of and complete implement the same predicates.
export { ownerOf, complete };
