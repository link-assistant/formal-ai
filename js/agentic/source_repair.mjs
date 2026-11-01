// Manual generic engine bootstrap after a preserved owned-repair intake failure.
import { WorkItemFileProgress, fileIoReadStep, fileIoPendingStep, fileIoEffectStep } from './work_item_file_io.mjs';
import { Capability } from './capability.mjs';
import { readText, parseLino, host } from './host.mjs';
import { jsonText, resolvedFinalAnswer, FinalDisposition } from './plan.mjs';
import { workItemEvidencePaths } from './work_item_evidence.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { clausesWithSpans } from './crate/obligation_ledger.mjs';
import { sentences } from './shell_command_policy.mjs';
import { quotedSegmentSpans, quoteFault } from './crate/normal_markov.mjs';
import { conceptWords } from './source_documentation.mjs';
export function sourceRepairContract(task) {
  if (typeof task !== 'string' || new TextEncoder().encode(task).length > 262144 || quoteFault(task) !== null) return null;
  let policy;
  try {
    const parsed = parseLino(readText('data/meta/owned-source-repair.lino'));
    const root = parsed.name === 'owned-source-repair' ? parsed : parsed.children?.find(node => node.name === 'owned-source-repair');
    const values = key => (root?.children ?? []).filter(node => node.name === key).map(node => node.value ?? node.id);
    policy = {
      actions: values('action'),
      prefixes: values('prefix'),
      negations: values('negation'),
      keys: values('key-cue'),
      bindings: values('binding-cue'),
      dictionaryScopes: values('dictionary-scope-cue'),
      dictionaryRelations: values('dictionary-relation-cue')
    };
  } catch {
    return null;
  }
  let masked = task;
  for (const span of [...quotedSegmentSpans(task)].reverse()) masked = masked.slice(0, span.start) + masked.slice(span.start, span.end).replace(/[^\r\n]/g, ' ') + masked.slice(span.end);
  const paths = workItemEvidencePaths(task).filter(path => /\.(?:mjs|js)$/u.test(path));
  if (!paths.length) return null;
  let owned = false,
    negated = false,
    operation = null;
  const repairGoals = [];
  for (const clause of sentences(masked)) {
    const words = conceptWords(clause.text);
    for (const action of policy.actions) {
      const index = words.indexOf(action);
      if (index < 0) continue;
      const before = words.slice(0, index).join(' ');
      if (policy.negations.some(cue => (' ' + before + ' ').includes(' ' + conceptWords(cue).join(' ') + ' '))) negated = true;
      if (index === 0 || policy.prefixes.includes(before)) {
        owned = true;
        repairGoals.push(policy.dictionaryScopes.some(cue => words.includes(cue)) && words.some((word,
         keyIndex) => policy.keys.includes(word) && words.some((relation, relationIndex) => relationIndex > keyIndex &&
         policy.dictionaryRelations.includes(relation) && words.some((binding, bindingIndex) => bindingIndex > relationIndex &&
         policy.bindings.includes(binding)))));
      }
    }
  }
  if (!owned && !negated) return null;
  if (repairGoals.length > 0 && repairGoals.every(Boolean)) operation = 'own-key-storage';
  return {
    operation,
    owner: paths.length === 1 ? paths[0] : null,
    authority: negated ? 'NegatedSourceRepairAction' : paths.length === 1 ? 'owned' : 'AmbiguousSourceRepairOwner'
  };
}
function argumentPath(attempt) {
  let object;
  try {
    object = JSON.parse(attempt.arguments);
  } catch {
    return null;
  }
  if (!object || typeof object !== 'object' || Array.isArray(object)) return null;
  const values = ['filePath', 'file_path', 'path'].filter(key => Object.hasOwn(object, key)).map(key => object[key]);
  return values.length && values.every(value => typeof value === 'string' && value === values[0]) ? values[0] : null;
}
export function planOwnedSourceRepair(task, messages, tools, workItem = null) {
  const contract = sourceRepairContract(task);
  if (!contract) return null;
  const progress = WorkItemFileProgress.scan(messages),
    attempts = progress.attempts;
  const requirements = clausesWithSpans(task).map(([text, span]) => ({
    text,
    span,
    kind: 'source-repair',
    status: 'unverified'
  }));
  if (!requirements.length) requirements.push({
    text: task,
    span: [0, new TextEncoder().encode(task).length],
    kind: 'source-repair',
    status: 'unverified'
  });
  const state = {
    schema: 'owned-source-repair/v1',
    task,
    taskIdentity: sha256Hex(task),
    workItem,
    source: contract.owner,
    target: contract.owner,
    requirements,
    observations: [],
    candidateEdits: [],
    checks: [],
    effects: [],
    remainingWork: ['source-repair-operation', 'independent-requirement-acceptance'],
    authored: false,
    verified: false,
    complete: false
  };
  const finish = (reason, failure = false) => resolvedFinalAnswer(jsonText({
    ...state,
    reason
  }), failure ? FinalDisposition.Failure : FinalDisposition.Gap, 'owned-source-repair');
  if (contract.authority !== 'owned') return finish(contract.authority);
  const pending = fileIoPendingStep(progress, tools);
  if (pending) return pending.step ?? finish(pending.gap);
  state.transportDiagnostics = progress.transportDiagnostics;
  if (attempts.some(attempt => attempt.capability === Capability.Read && argumentPath(attempt) === null)) return finish('AmbiguousSourceRepairRead', true);
  const effects = attempts.map((attempt, index) => ({
    attempt,
    index
  })).filter(item => [Capability.Write, Capability.Edit].includes(item.attempt.capability));
  if (effects.length > 1) return finish('MultipleSourceRepairEffects', true);
  if (effects.some(item => item.attempt.capability !== Capability.Edit || argumentPath(item.attempt) !== contract.owner)) return finish('UnrelatedSourceRepairEffect', true);
  const effect = effects[0] ?? null;
  const reads = (before = attempts.length, after = -1) => attempts.map((attempt, index) => ({
    attempt,
    index
  })).filter(item => item.index < before && item.index > after && item.attempt.capability === Capability.Read && argumentPath(item.attempt) === contract.owner);
  const complete = attempt => attempt?.succeeded === true && attempt.source_read?.error === null && attempt.source_read.complete === true &&
         typeof attempt.source_read.source === 'string' && new TextEncoder().encode(attempt.source_read.source).length <= 262144 ? attempt.source_read.source : null;
  const prior = reads(effect?.index ?? attempts.length),
    first = prior.find(item => complete(item.attempt) !== null);
  if (!prior.length) {
    const planned = fileIoReadStep(messages, tools, contract.owner, task);
    return planned.step ?? finish(planned.gap);
  }
  const original = complete(first?.attempt),
    latest = complete(prior.at(-1)?.attempt);
  if (original === null || latest === null) return finish('SourceRepairCompleteObservationRequired');
  if (original !== latest) return finish('SourceRepairPreimageChanged', true);
  state.observations.push({
    path: contract.owner,
    sourceIdentity: sha256Hex(original),
    complete: true
  });
  if (contract.operation === null) return finish('SourceRepairOperatorUnavailable');
  let derived;
  try {
    const operation = host().observedSourceRepair;
    if (typeof operation !== 'function') return finish('SourceRepairOperatorUnavailable');
    derived = operation({
      operation: contract.operation,
      owner: contract.owner,
      source: original
    });
  } catch (error) {
    return finish(typeof error?.message === 'string' && /^[A-Za-z][A-Za-z0-9]{0,127}$/u.test(error.message) ? error.message : 'SourceRepairOperationFailed', effect !== null);
  }
  const candidate = derived?.candidate;
  if (derived?.schema !== 'observed-own-key-dictionary-repair/v1' || derived.owner !== contract.owner ||
         candidate?.path !== contract.owner || candidate.beforeSha256 !== sha256Hex(original) || typeof candidate.source !== 'string' ||
         candidate.afterSha256 !== sha256Hex(candidate.source) || candidate.source === original || new TextEncoder().encode(candidate.source).length > 262144 ||
         derived.verification?.completeConcreteResidual !== true || derived.verification?.independentGrammarReparse !== true ||
         derived.verification?.exactInverse !== true) return finish('SourceRepairCandidateInvalid', true);
  state.binding = derived.binding;
  state.sourceProof = derived.verification;
  state.knowledge = derived.knowledge;
  state.candidateEdits.push({
    path: contract.owner,
    preimageIdentity: sha256Hex(original),
    postimageIdentity: sha256Hex(candidate.source)
  });
  if (!effect) {
    const planned = fileIoEffectStep(messages, tools, contract.owner, original, candidate.source);
    return planned.step ?? finish(planned.gap);
  }
  let argumentsValue;
  try {
    argumentsValue = JSON.parse(effect.attempt.arguments);
  } catch {
    return finish('SourceRepairEffectArgumentsInvalid', true);
  }
  const alias = names => {
    const values = names.filter(name => Object.hasOwn(argumentsValue, name)).map(name => argumentsValue[name]);
    return values.length && values.every(value => typeof value === 'string' && value === values[0]) ? values[0] : null;
  };
  if (alias(['oldString', 'old_string', 'old_str', 'old']) !== original || alias(['newString', 'new_string', 'new_str', 'new']) !== candidate.source) return finish('SourceRepairEffectNotDerived', true);
  if (effect.attempt.succeeded !== true) return finish('SourceRepairEditFailed', true);
  const fresh = reads(attempts.length, effect.index).at(-1);
  if (!fresh) {
    const planned = fileIoReadStep(messages, tools, contract.owner, task);
    return planned.step ?? finish(planned.gap);
  }
  const actual = complete(fresh.attempt);
  if (actual !== candidate.source) return finish('SourceRepairReadbackMismatch', true);
  state.effects.push({
    path: contract.owner,
    kind: 'edit',
    observedPostimageIdentity: sha256Hex(actual)
  });
  state.checks.push({
    kind: 'derived-source-proof-and-exact-readback',
    passed: true
  });
  state.remainingWork = ['independent-requirement-acceptance'];
  return finish('UsefulSourceRepairReadbackVerified');
}
