import { host as installedHost } from '../agentic/host.mjs';
import * as sourceHost from './node-source-session-host.mjs';
import { prepareRustConstantRename } from './rust-source-transform.mjs';

const preparedTransactions = new WeakMap();
const backendMethods = Object.freeze([
  'prepareTrackedSourceTransaction', 'persistTrackedSourceInverse',
  'writeTrackedSourceCandidate', 'readTrackedSourceCandidate',
  'trackedSourceTransactionDisposition', 'abortTrackedSourceTransaction'
]);

function physicalObservation(token, request, workspace, declaration) {
  if (typeof sourceHost.registeredPhysicalReadObservation !== 'function') {
    throw Error('MissingPhysicalReadPlatform');
  }
  return sourceHost.registeredPhysicalReadObservation(token, request, workspace, declaration);
}

function currentWitness(prepared) {
  const witness = preparedTransactions.get(prepared);
  if (!witness) throw Error('UnownedTrackedSourceTransaction');
  if (installedHost().sourceOperation !== witness.operation
      || installedHost().sourceSession !== witness.session) throw Error('ReplacedTrackedSourceHost');
  return witness;
}

async function disposition(witness, receipt, allowedStates) {
  const observed = await witness.operation.trackedSourceTransactionDisposition(witness.capability, receipt);
  if (installedHost().sourceOperation !== witness.operation
      || installedHost().sourceSession !== witness.session) throw Error('ReplacedTrackedSourceHost');
  if (!observed || !allowedStates.includes(observed.state)
      || observed.beforeSha256 !== witness.candidate.beforeSha256
      || observed.afterSha256 !== witness.candidate.afterSha256) {
    throw Error('UnownedTrackedSourceDisposition');
  }
  return Object.freeze({state: observed.state, beforeSha256: observed.beforeSha256,
    afterSha256: observed.afterSha256, nativeValidity: 'Unknown'});
}

/** The factory must own the current Read, staged request and opaque transaction capability. */
export async function prepareTrackedRustSourceTransaction(token, request, workspace, declaration) {
  const observed = physicalObservation(token, request, workspace, declaration);
  const contract = observed.referenceRequest;
  if (!contract || contract.wholeRequestConsumed !== true || contract.originalRequest !== request) {
    throw Error('MissingCompleteTrackedRequest');
  }
  const rename = contract.clauses.find(clause => clause.kind === 'private-constant-rename');
  if (!rename || rename.captures.sourcePath.text !== observed.operand) throw Error('DifferentTrackedSourceOperand');
  const candidate = await prepareRustConstantRename(observed.content,
    rename.captures.previous.text, rename.captures.replacement.text);
  if (!candidate || candidate.beforeSha256 !== observed.sha256) throw Error('UnsupportedTrackedSourceCandidate');
  const latest = physicalObservation(token, request, workspace, declaration);
  if (latest.sha256 !== observed.sha256 || latest.referenceRequest !== contract) throw Error('TrackedSourceReadDrift');
  const current = installedHost(), operation = current.sourceOperation;
  if (!operation || backendMethods.some(name => typeof operation[name] !== 'function')) {
    throw Error('MissingTrackedTransactionPlatform');
  }
  const capability = await operation.prepareTrackedSourceTransaction(token, request, workspace, declaration, candidate);
  if (installedHost().sourceOperation !== operation || installedHost().sourceSession !== current.sourceSession) {
    throw Error('ReplacedTrackedSourceHost');
  }
  physicalObservation(token, request, workspace, declaration);
  if (!capability || typeof capability !== 'object') throw Error('MissingOwnedTrackedCapability');
  const prepared = Object.freeze({});
  preparedTransactions.set(prepared, {operation, session: current.sourceSession, capability,
    token, request, workspace, declaration, candidate, contract, phase: 'prepared'});
  return prepared;
}

/** Durable inverse must precede the exact owned effect; no native completion is inferred. */
export async function executePreparedTrackedRustSource(prepared) {
  const witness = currentWitness(prepared);
  if (witness.phase !== 'prepared') throw Error('InvalidTrackedTransactionPhase');
  witness.phase = 'started';
  try {
    physicalObservation(witness.token, witness.request, witness.workspace, witness.declaration);
    let receipt = await witness.operation.persistTrackedSourceInverse(witness.capability);
    currentWitness(prepared);
    await disposition(witness, receipt, ['inverse-durable']);
    receipt = await witness.operation.writeTrackedSourceCandidate(witness.capability);
    currentWitness(prepared);
    await disposition(witness, receipt, ['written-unverified']);
    receipt = await witness.operation.readTrackedSourceCandidate(witness.capability);
    currentWitness(prepared);
    const observed = await disposition(witness, receipt, ['source-change-observed']);
    witness.readbackReceipt = receipt;
    witness.phase = 'observed-unverified';
    return Object.freeze({kind: 'ObservedTrackedSourceChange', physicalDisposition: observed,
      sourceChangeObserved: true, verified: false, nativeValidity: 'Unknown',
      unresolvedNeeds: witness.contract.unresolvedNeeds, receiptAuthority: 'Missing', proofAuthority: 'Missing'});
  } catch (error) {
    let recovery = Object.freeze({state: 'recovery-required', nativeValidity: 'Unknown'});
    try {
      currentWitness(prepared);
      const receipt = await witness.operation.abortTrackedSourceTransaction(witness.capability);
      recovery = await disposition(witness, receipt, ['rolled-back', 'refused-drift', 'released-without-effect']);
    } catch { witness.phase = 'recovery-required'; }
    if (witness.phase !== 'recovery-required') witness.phase = recovery.state;
    return Object.freeze({kind: 'IncompleteTrackedSourceTransaction', failure: error.message,
      physicalDisposition: recovery, sourceChangeObserved: false, verified: false,
      nativeValidity: 'Unknown', unresolvedNeeds: witness.contract.unresolvedNeeds,
      receiptAuthority: 'Missing', proofAuthority: 'Missing'});
  }
}

/** Mint only from the actual private readback in the current factory context. */
export async function issuePreparedTrackedSourceEffectReceipt(prepared) {
  const witness = currentWitness(prepared);
  if (witness.phase !== 'observed-unverified' || !witness.readbackReceipt) {
    throw Error('MissingObservedTrackedReadback');
  }
  const operation = witness.operation;
  if (typeof operation.issueTrackedSourceEffectReceipt !== 'function'
      || typeof operation.trackedSourceEffectObservation !== 'function') {
    throw Error('MissingTrackedEffectReceiptPlatform');
  }
  const token = await operation.issueTrackedSourceEffectReceipt(
    witness.capability, witness.readbackReceipt);
  currentWitness(prepared);
  const observed = await operation.trackedSourceEffectObservation(
    token, witness.request, witness.workspace);
  currentWitness(prepared);
  if (!observed || observed.kind !== 'OnlyTrackedSourceEffect'
      || observed.originalRequest !== witness.request
      || observed.workspace !== witness.workspace
      || observed.referenceRequest !== witness.contract
      || observed.beforeSha256 !== witness.candidate.beforeSha256
      || observed.afterSha256 !== witness.candidate.afterSha256
      || observed.readbackSha256 !== witness.candidate.afterSha256
      || observed.completed !== false || observed.artifactWriteAuthority !== 'Missing') {
    throw Error('DifferentTrackedEffectReceipt');
  }
  return token;
}
