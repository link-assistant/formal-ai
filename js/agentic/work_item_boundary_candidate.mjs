// Intended owner: js/agentic/work_item_boundary_candidate.mjs.
// This adapter binds a declared finite subset to current observed files. It
// does not own the surrounding task or discharge unsupported obligations.
import { readText, parseLino, host, hasHost } from './host.mjs';
import { sentences } from './shell_command_policy.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { WorkItemFileProgress as Progress, fileIoReadStep, fileIoEffectStep, fileIoPendingStep } from './work_item_file_io.mjs';
import { Capability } from './capability.mjs';
import { FinalDisposition } from './plan.mjs';
import { workItemRequirements } from './work_item_requirements.mjs';
import { refusal, workItemRecordFinal } from './work_item_report.mjs';
import { userReadPolicyBlocksPlan } from './file_read/ownership.mjs';
import { toolFor } from './capability_router.mjs';
import { planOne, jsonText } from './plan.mjs';
import { commandArgument } from './tool_result.mjs';
const words = value => value.toLowerCase().replace(/[`'"()[\],;:]/gu, ' ').trim().replace(/\s+/gu, ' ');
function grammar() {
  const parsed = parseLino(readText('data/meta/comparison-boundary-grammar.lino'));
  const root = parsed.name === 'comparison-boundary-grammar' ? parsed : parsed.children?.find(node => node.name === 'comparison-boundary-grammar');
  if (!root) throw Error('BoundaryGrammarUnavailable');
  return root.children;
}
const values = (rules, key) => rules.filter(node => node.name === key).map(node => node.value ?? node.id);
const mentions = (text, surfaces) => surfaces.some(value => (' ' + text + ' ').includes(' ' + words(value) + ' '));
const validOwner = path => typeof path === 'string' && /^[A-Za-z0-9_-][A-Za-z0-9_./-]*$/u.test(path) && path.split('/').every(part => part !== '' && part !== '.' && part !== '..');

/** Select one explicitly requested structural leaf. Remaining clauses stay in
 * the controller ledger; their text never becomes permission for this leaf. */
export function boundaryWorkItemContract(task, pathsForClause) {
  if (typeof task !== 'string' || new TextEncoder().encode(task).length > 32768 || typeof pathsForClause !== 'function') return null;
  let rules;
  try {
    rules = grammar();
  } catch {
    return null;
  }
  const clauses = sentences(task),
    owned = [];
  for (const clause of clauses) {
    const text = words(clause.text);
    const regression = mentions(text, values(rules, 'regression-action'));
    const repair = mentions(text, values(rules, 'repair-action'));
    if (!regression && !repair || mentions(text, values(rules, 'negated-property')) || mentions(text, values(rules, 'negated-action'))) continue;
    const upper = mentions(text, values(rules, 'upper-bound')),
      lower = mentions(text, values(rules, 'lower-bound'));
    const inclusive = mentions(text, values(rules, 'inclusive')),
      exclusive = mentions(text, values(rules, 'exclusive'));
    if (upper === lower || inclusive === exclusive) continue;
    const sourceSuffix = values(rules, 'source-suffix'),
      testSuffix = values(rules, 'test-suffix');
    const referenced = pathsForClause(clause.text).filter(validOwner);
    const tests = referenced.filter(path => testSuffix.some(suffix => path.endsWith(suffix)));
    const sources = referenced.filter(path => sourceSuffix.some(suffix => path.endsWith(suffix)) && !tests.includes(path));
    if (sources.length !== 1 || tests.length !== (regression ? 1 : 0)) continue;
    let declaration = null,
      ambiguous = false;
    // Identifier capture is owned by a seeded declaration marker, never by a
    // source comment or an issue-specific function name.
    const tokens = clause.text.split(/\s+/u).map(value => value.replace(/^[`'"(]+|[`'"),.;:]+$/gu, ''));
    for (let index = 0; index < tokens.length - 1; index++) {
      if (!values(rules, 'declaration-marker').some(marker => words(marker) === words(tokens[index]))) continue;
      const next = tokens[index + 1];
      if (!/^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(next)) {
        ambiguous = true;
        break;
      }
      if (declaration !== null && declaration !== next) {
        ambiguous = true;
        break;
      }
      declaration = next;
    }
    if (ambiguous) continue;
    const target = regression ? tests[0] : sources[0];
    const clauseStart = task.indexOf(clause.text, clause.span.start);
    if (clauseStart < clause.span.start || clauseStart + clause.text.length > clause.span.end) continue;
    const sourceSpan = [new TextEncoder().encode(task.slice(0, clauseStart)).length, new TextEncoder().encode(task.slice(0, clauseStart + clause.text.length)).length];
    owned.push({
      schema: 'work-item-comparison-contract/v1',
      mode: regression ? 'regression' : 'repair',
      owner: sources[0],
      target,
      declaration,
      bound: upper ? 'upper' : 'lower',
      inclusive,
      sourceClause: clause.text,
      sourceSpan,
      evidencePaths: [sources[0]],
      targetObservationRequired: true,
      allowMissing: regression ? [target] : [],
      independentAcceptanceRequired: true,
      scope: 'finite-number-interval-only'
    });
  }
  return owned.length === 1 ? owned[0] : null;
}
const complete = observation => observation && observation.error === null && observation.complete === true && observation.absent !== true && typeof observation.source === 'string';

/** The transcript itself retains the candidate preimage across planner calls.
 * A read AFTER our effect is readback, never a replacement baseline that could
 * erase first-attempt authorship or make an incomplete write appear a no-op. */
export function boundaryCandidateObservations(contract, messages) {
  const progress = Progress.scan(messages),
    attempts = progress.attempts;
  const firstEffect = attempts.findIndex(attempt => [Capability.Write, Capability.Edit].includes(attempt.capability) && ownedPath(argumentsOf(attempt), contract.target));
  const before = firstEffect < 0 ? attempts : attempts.slice(0, firstEffect),
    result = new Map();
  for (const path of [contract.owner, contract.target]) {
    const observation = [...before].reverse().find(attempt => attempt.capability === Capability.Read && ownedPath(argumentsOf(attempt), path))?.source_read ?? null;
    result.set(path, observation);
  }
  return result;
}

/** `operation` is an installed caller-owned capability. Do not deserialize one
 * from fetched data. Observations are current Progress.sourceReadFor receipts.
 * Returned candidate still requires guarded physical effect + fresh readback. */
export function deriveBoundaryWorkItemCandidate(contract, observations, operation) {
  if (!contract || typeof operation !== 'function') return {
    gap: 'BoundaryHostCapabilityUnavailable'
  };
  const get = path => observations instanceof Map ? observations.get(path) : observations[path];
  const source = get(contract.owner),
    target = get(contract.target);
  if (!complete(source)) return {
    gap: 'BoundaryCompleteSourceRequired',
    missingEvidence: [contract.owner]
  };
  if (new TextEncoder().encode(source.source).length > 131072) return {
    gap: 'BoundarySourceInputLimit'
  };
  if (!target) return {
    gap: 'BoundaryTargetObservationRequired',
    missingEvidence: [contract.target]
  };
  if (contract.mode === 'repair' && !complete(target)) return {
    gap: 'BoundaryCompleteTargetRequired'
  };
  if (contract.mode === 'regression' && !complete(target) && target.absent !== true) return {
    gap: 'BoundaryTargetAvailabilityUnresolved'
  };
  let result;
  try {
    result = operation({
      source: source.source,
      owner: contract.owner,
      declaration: contract.declaration,
      bound: contract.bound,
      inclusive: contract.inclusive,
      mode: contract.mode,
      ...(contract.mode === 'regression' ? {
        testOwner: contract.target
      } : {})
    });
  } catch (error) {
    return {
      gap: String(error?.message ?? 'BoundaryCandidateRefused')
    };
  }
  if (!result || result.schema !== 'observed-comparison-boundary-operation/v1' || result.candidate?.path !== contract.target || typeof result.candidate.source !== 'string' || result.candidate.afterSha256 !== sha256Hex(result.candidate.source)) return {
    gap: 'BoundaryCandidateIdentityMismatch'
  };
  if (contract.mode === 'repair' && result.candidate.beforeSha256 !== sha256Hex(source.source)) return {
    gap: 'BoundaryCandidatePreimageMismatch'
  };
  if (contract.mode === 'regression' && result.candidate.sourceOwnerSha256 !== sha256Hex(source.source)) return {
    gap: 'BoundaryCandidateSourceMismatch'
  };
  const before = complete(target) ? target.source : null;
  if (contract.mode === 'regression' && before !== null && before !== result.candidate.source) return {
    gap: 'BoundaryExistingRegressionPreserved'
  };
  return {
    candidate: {
      kind: 'structural-comparison-boundary',
      mode: contract.mode,
      path: contract.target,
      before,
      after: result.candidate.source,
      requiresAbsent: before === null,
      sourceOwner: contract.owner,
      sourceIdentity: sha256Hex(source.source),
      beforeIdentity: before === null ? null : sha256Hex(before),
      afterIdentity: result.candidate.afterSha256,
      noOp: before === result.candidate.source,
      sourceClause: contract.sourceClause,
      sourceSpan: contract.sourceSpan,
      preconditions: [{
        kind: 'complete-current-source',
        path: contract.owner,
        sha256: sha256Hex(source.source)
      }, before === null ? {
        kind: 'absent-target',
        path: contract.target
      } : {
        kind: 'exact-target-preimage',
        path: contract.target,
        sha256: sha256Hex(before)
      }],
      checks: [{
        kind: 'structural-contract',
        status: 'passed',
        detail: result.verification
      }, {
        kind: 'physical-readback',
        status: 'pending'
      }, {
        kind: 'independent-task-acceptance',
        status: 'pending'
      }],
      rule: result.rule ?? null,
      sourceModel: result.sourceModel,
      suppliedMechanism: true,
      autonomousAuthorshipCredit: 0,
      independentlyAccepted: false
    }
  };
}

/** A transport Write success alone never satisfies this check. A fresh exact
 * target observation must occur AFTER the bound effect (caller enforces order).
 * `noOp` earns idempotence credit but zero first-attempt implementation credit. */
export function verifyBoundaryCandidateReadback(candidate, observation) {
  if (!candidate || !complete(observation)) return {
    verified: false,
    reason: 'BoundaryCompleteReadbackRequired'
  };
  if (observation.source !== candidate.after || sha256Hex(observation.source) !== candidate.afterIdentity) return {
    verified: false,
    reason: 'BoundaryPhysicalReadbackMismatch'
  };
  return {
    verified: true,
    noOp: candidate.noOp,
    implementationChanged: !candidate.noOp,
    sourceIdentity: candidate.afterIdentity,
    independentTaskAcceptance: false
  };
}
function argumentsOf(attempt) {
  try {
    return JSON.parse(attempt.arguments);
  } catch {
    return null;
  }
}
function ownedPath(argumentsValue, path) {
  if (!argumentsValue) return false;
  const paths = ['path', 'filePath', 'file_path'].filter(key => Object.hasOwn(argumentsValue, key)).map(key => argumentsValue[key]);
  return paths.length > 0 && paths.every(value => value === path);
}

function boundaryAttemptGuard(progress,target) {
  const mutations=progress.attempts.filter(attempt=>
    [Capability.Write,Capability.Edit].includes(attempt.capability));
  const budget={maximum:1,observed:mutations.length};
  if(mutations.length>1)return {gap:'BoundaryEffectAttemptBudgetExceeded',budget};
  if(mutations.some(attempt=>!ownedPath(argumentsOf(attempt),target)))
    return {gap:'BoundaryUnrelatedOrAmbiguousMutationAttempt',budget};
  const ambiguousRead=progress.attempts.some(attempt=>{
    if(attempt.capability!==Capability.Read)return false;
    const argumentsValue=argumentsOf(attempt);
    if(!argumentsValue)return true;
    const paths=['path','filePath','file_path','absolute_path']
      .filter(key=>Object.hasOwn(argumentsValue,key)).map(key=>argumentsValue[key]);
    return paths.length===0||paths.some(path=>typeof path!=='string'||path!==paths[0]);
  });
  return ambiguousRead?{gap:'BoundaryAmbiguousReadPathAttempt',budget}:null;
}

/** Execute ONLY a persisted candidate. The enclosing controller retains task
 * identity, candidate source hashes and obligation statuses across these calls.
 * This function returns a pending tool step or an observed effect result; it
 * never labels the surrounding task independently accepted or complete. */
export function planBoundaryCandidateEffect(candidate, messages, tools, task = '') {
  const progress = Progress.scan(messages);
  const pending = fileIoPendingStep(progress, tools);
  if (pending) return pending;
  const guard=boundaryAttemptGuard(progress,candidate.path);if(guard)return guard;
  const effects = progress.attempts.map((attempt, index) => ({
    attempt,
    index,
    args: argumentsOf(attempt)
  })).filter(({
    attempt,
    args
  }) => [Capability.Write, Capability.Edit].includes(attempt.capability) && ownedPath(args, candidate.path));
  const effect = effects.at(-1) ?? null;
  if (effects.length > 1) return {
    gap: 'BoundaryEffectAttemptBudgetExceeded',
    budget: {
      maximum: 1,
      observed: effects.length
    }
  };
  if (effect) {
    const after = effect.args.content ?? effect.args.newString ?? effect.args.new_string;
    if (after !== candidate.after) return {
      gap: 'BoundaryUnrelatedTargetEffect'
    };
    if (!effect.attempt.succeeded) return {
      failure: 'BoundaryEffectFailed',
      detail: effect.attempt.detail
    };
    const reads = progress.attempts.slice(effect.index + 1).filter(attempt => attempt.capability === Capability.Read && ownedPath(argumentsOf(attempt), candidate.path));
    const observed = reads.at(-1)?.source_read ?? null;
    if (!observed) return fileIoReadStep(messages, tools, candidate.path, task);
    const checked = verifyBoundaryCandidateReadback(candidate, observed);
    return checked.verified ? {
      effect: {
        ...checked,
        path: candidate.path,
        authorshipBoundary: 'candidate-output-only',
        independentlyAccepted: false
      }
    } : {
      failure: checked.reason
    };
  }
  const target = progress.sourceReadFor(candidate.path);
  if (candidate.noOp) {
    const checked = verifyBoundaryCandidateReadback(candidate, target);
    return checked.verified ? {
      effect: {
        ...checked,
        path: candidate.path,
        authorshipBoundary: 'existing-artifact-no-authorship',
        independentlyAccepted: false
      }
    } : {
      gap: checked.reason
    };
  }
  if (candidate.requiresAbsent) {
    if (target?.absent !== true) return {
      gap: 'BoundaryTargetPreconditionChanged'
    };
    return fileIoEffectStep(messages, tools, candidate.path, null, candidate.after);
  }
  if (!complete(target) || target.source !== candidate.before || sha256Hex(target.source) !== candidate.beforeIdentity) return {
    gap: 'BoundaryTargetPreconditionChanged'
  };
  return fileIoEffectStep(messages, tools, candidate.path, candidate.before, candidate.after);
}

/** Markers a contract check prints; neither contains a character a transport escapes. */
const SAMPLE_MARKER = /formal-ai-contract-check: (passed|failed) ([0-9]+)\/([0-9]+)/u;
const REGRESSION_MARKER = /formal-ai-regression-test: (passed|failed)/u;
const IDENTIFIER = /^[A-Za-z_$][A-Za-z0-9_$]*$/u;

/**
 * The contract-derived check of a candidate (issue #1483). Its samples apply
 * the REQUESTED bound and inclusivity to the signature of the source observed
 * before any effect, never to the candidate, and run against the current
 * artifact through the Run tool: a repair that missed the bound fails it.
 * A regression candidate also runs its own test file. Null when no Run tool is
 * advertised or the samples cannot be derived; the check then stays pending.
 */
export function boundaryContractCheck(contract, observations, operation, tools) {
  const run = toolFor(tools, Capability.Run);
  if (run === null || !contract || typeof operation !== 'function') return null;
  const source = observations instanceof Map ? observations.get(contract.owner) : observations?.[contract.owner];
  if (!complete(source)) return null;
  let derived;
  try {
    derived = operation({
      source: source.source,
      owner: contract.owner,
      declaration: contract.declaration,
      bound: contract.bound,
      inclusive: contract.inclusive,
      mode: 'regression',
      testOwner: contract.mode === 'regression' ? contract.target : contract.owner.replace(/\.(m?js)$/u, '.formal-ai-check.$1')
    });
  } catch {
    return null;
  }
  const samples = derived?.samples, declaration = derived?.verification?.declaration;
  if (!Array.isArray(samples) || !samples.length || samples.length > 32 || typeof declaration !== 'string' || !IDENTIFIER.test(declaration)
    || samples.some(sample => !Array.isArray(sample?.args) || sample.args.some(value => !Number.isFinite(value)) || typeof sample.expected !== 'boolean')) return null;
  const rows = JSON.stringify(samples.map(sample => ({ args: sample.args, expected: sample.expected })));
  const script = `import {${declaration} as f} from "./${contract.owner}";const s=${rows};`
    + 'const bad=s.filter(x=>f(...x.args)!==x.expected);'
    + 'console.log("formal-ai-contract-check: "+(bad.length?"failed ":"passed ")+(s.length-bad.length)+"/"+s.length);'
    + 'process.exitCode=bad.length?1:0;';
  const sampleCommand = `node --input-type=module -e '${script}'`;
  const command = contract.mode === 'regression'
    ? `node --test '${contract.target}' >/dev/null 2>&1 && echo "formal-ai-regression-test: passed" || echo "formal-ai-regression-test: failed"; ${sampleCommand}`
    : sampleCommand;
  return { tool: run, command, samples: samples.length, declaration };
}

/** The outcome of an executed contract check: passed, failed, or unavailable when its marker is missing. */
export function boundaryCheckOutcome(check, attempt) {
  const text = String(attempt?.raw_receipt ?? attempt?.detail ?? '');
  const samples = SAMPLE_MARKER.exec(text);
  if (samples === null || Number(samples[3]) !== check.samples) return 'unavailable';
  if (check.command.startsWith('node --test ')) {
    const regression = REGRESSION_MARKER.exec(text);
    if (regression === null) return 'unavailable';
    if (regression[1] === 'failed') return 'failed';
  }
  return samples[1];
}

/** The latest Run attempt of exactly `command`. */
function checkAttemptFor(progress, command) {
  return [...progress.attempts].reverse().find(attempt => attempt.capability === Capability.Run && commandArgument(attempt.arguments) === command) ?? null;
}

/** Production route wrapper. Call for the actual fetched objective before the
 * generic failure/read fallback, so a requested NEW test's proven ENOENT can be
 * treated as target precondition rather than a work-item transport failure.
 * `pathsForClause` is the existing shared path parser; no circular import.
 * All whole-task outcomes remain Gap pending external independent acceptance. */
export function planWorkItemBoundaryStep(task, messages, tools, workItem, pathsForClause) {
  const contract = boundaryWorkItemContract(task, pathsForClause);
  if (!contract) return null;
  const progress = Progress.scan(messages),
    observations = [];
  let candidate = null,
    effect = null,
    contractCheck = null;
  // The work item is split into requirements before any evidence is read; a
  // requirement overlapping the bound clause awaits its independent check.
  const obligations = workItemRequirements(task).map(([text, span]) => ({
    text,
    span,
    status: span[0] < contract.sourceSpan[1] && span[1] > contract.sourceSpan[0] ? 'independent_check_pending' : 'unresolved'
  }));
  const finish = (reason, detail = null, refused = null) => {
    const authored = effect?.implementationChanged === true;
    // Only an executed, contract-derived check discharges the bound requirement.
    const checked = effect?.verified === true && contractCheck?.status === 'passed';
    if (checked) for (const obligation of obligations) if (obligation.status === 'independent_check_pending') obligation.status = 'verified';
    const allVerified = checked && obligations.length > 0 && obligations.every(obligation => obligation.status === 'verified');
    const candidateView = candidate ? {
      kind: candidate.kind,
      mode: candidate.mode,
      path: candidate.path,
      beforeIdentity: candidate.beforeIdentity,
      afterIdentity: candidate.afterIdentity,
      sourceIdentity: candidate.sourceIdentity,
      sourceOwner: candidate.sourceOwner,
      preconditions: candidate.preconditions,
      noOp: candidate.noOp,
      sourceRepresentation: {
        kind: 'complete-readable-cst',
        identity: sha256Hex(JSON.stringify(candidate.sourceModel))
      },
      checks: [...candidate.checks.map(check => check.kind === 'physical-readback' ? {
        ...check,
        status: effect?.verified === true ? 'passed' : 'pending'
      } : check.kind === 'independent-task-acceptance' && contractCheck !== null ? {
        ...check,
        status: contractCheck.status,
        detail: 'contract-derived-execution'
      } : check), ...(contractCheck === null ? [] : [contractCheck])]
    } : null;
    return workItemRecordFinal({
      schema: 'repository-work-item-controller/v1',
      transportDiagnostics: progress.transportDiagnostics,
      reason,
      workItem: workItem ?? null,
      taskIdentity: sha256Hex(task),
      requirements: obligations,
      supportedSubset: 'finite-number-interval-boundary-and-regression',
      boundContract: contract,
      observations,
      candidate: candidateView,
      checks: candidateView?.checks ?? [],
      effects: effect ? [effect] : [],
      authored,
      verified: checked,
      independentlyAccepted: checked,
      complete: allVerified,
      physicalReadbackVerified: effect?.verified === true,
      noOp: effect?.noOp === true,
      usefulChange: {
        observed: authored,
        independentlyAccepted: checked && authored,
        paths: authored ? [contract.target] : []
      },
      authorshipBoundary: {
        mechanism: 'supplied-generic-structural-operation',
        targetOutputObserved: authored,
        autonomousCredit: 0,
        laterReviewerWork: 'not-observed'
      },
      remainingWork: obligations.filter(obligation => obligation.status !== 'verified').map(obligation => ({
        text: obligation.text,
        span: obligation.span,
        reason: obligation.status === 'independent_check_pending' ? 'independent-requirement-acceptance-required' : 'unsupported-or-unbound-obligation'
      })),
      budget: {
        maximumEffects: 1,
        maximumEvidenceFiles: 2
      },
      detail,
      ...(refused === null ? {} : { refused })
    }, task, allVerified ? FinalDisposition.Finding : FinalDisposition.Gap, 'repository_work_item_boundary');
  };
  const refuse = path => finish('ReadPolicyRefused', null, refusal(obligations, path));
  const unplanned = (gap, path) => gap === 'ReadPolicyRefused' ? refuse(path) : finish(gap);
  const pending = fileIoPendingStep(progress, tools);
  if (pending) return pending.step ?? finish(pending.gap);
  const guard=boundaryAttemptGuard(progress,contract.target);
  if(guard)return finish(guard.gap,guard);
  for (const path of [...new Set([contract.owner, contract.target])]) {
    const observed = progress.sourceReadFor(path);
    if (!observed) {
      const planned = fileIoReadStep(messages, tools, path, task);
      return planned.step ?? unplanned(planned.gap, path);
    }
    const isComplete = complete(observed),
      allowedAbsent = contract.allowMissing.includes(path) && observed.absent === true;
    observations.push({
      path,
      complete: isComplete,
      absent: observed.absent === true,
      error: observed.error ?? null,
      status: observed.status,
      sourceIdentity: isComplete ? sha256Hex(observed.source) : null
    });
    if (!isComplete && !allowedAbsent) return finish('IncompleteSourceObservation');
  }
  const operation = hasHost() ? host().comparisonBoundaryOperation : null;
  const proposed = deriveBoundaryWorkItemCandidate(contract, boundaryCandidateObservations(contract, messages), operation);
  if (!proposed.candidate) return finish(proposed.gap ?? 'BoundaryCandidateUnavailable', proposed);
  candidate = proposed.candidate;
  const next = planBoundaryCandidateEffect(candidate, messages, tools, task);
  if (next.step) return userReadPolicyBlocksPlan(messages, task, next.step) ? refuse(candidate.path) : next.step;
  if (next.effect) {
    effect = next.effect;
    const awaiting = effect.noOp ? 'VerifiedNoOpAwaitingIndependentAcceptance' : 'UsefulCandidateAwaitingIndependentAcceptance';
    // The readback-verified artifact is then executed against samples derived from the contract.
    const check = boundaryContractCheck(contract, boundaryCandidateObservations(contract, messages), operation, tools);
    if (check === null) return finish(awaiting);
    const attempt = checkAttemptFor(progress, check.command);
    if (attempt === null) return planOne(check.tool, jsonText({ command: check.command }));
    const status = boundaryCheckOutcome(check, attempt);
    contractCheck = { kind: 'contract-derived-execution', command: check.command, samples: check.samples, declaration: check.declaration, status };
    if (status === 'passed') return finish(effect.noOp ? 'VerifiedNoOp' : 'UsefulChangeVerified');
    return finish(status === 'failed' ? 'IndependentCheckFailed' : 'IndependentCheckUnavailable', { check: contractCheck });
  }
  return finish(next.failure ?? next.gap ?? 'BoundaryExecutionUnresolved', next);
}
