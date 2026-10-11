import { createHash } from 'node:crypto';
import { host as installedHost } from '../agentic/host.mjs';
import * as sourceHost from './node-source-session-host.mjs';
import { proposeTrackedSourceEvidence } from './tracked-source-evidence-proposal.mjs';

const hash = content => createHash('sha256').update(content, 'utf8').digest('hex');


function physicalNeedEvidence(effect, proposal, observed) {
  const contract = effect.referenceRequest;
  if (effect.kind !== 'OnlyTrackedSourceEffect' || contract.wholeRequestConsumed !== true
      || contract.originalRequest !== proposal.originalRequest
      || effect.afterSha256 !== proposal.sourceAfterSha256
      || effect.readbackSha256 !== effect.afterSha256
      || effect.beforeSha256 === effect.afterSha256) throw Error('DifferentNeedEffect');
  const capture = (kind, name) => {
    const clauses = contract.clauses.filter(value => value.kind === kind);
    if (clauses.length !== 1 || typeof clauses[0].captures[name]?.text !== 'string') {
      throw Error('UnboundNeedCapture');
    }
    return clauses[0].captures[name].text;
  };
  const receiptPath = capture('exact-field-receipt', 'receiptPath');
  const proofPath = capture('proof-target', 'proofPath');
  const receipt = proposal.artifacts.find(value => value.path === receiptPath);
  const proof = proposal.artifacts.find(value => value.path === proofPath);
  const matches = artifact => artifact && observed.some(value =>
    value.path === artifact.path && value.sha256 === hash(artifact.content));
  if (!matches(receipt) || !matches(proof)) throw Error('MissingObservedNeedArtifact');
  const lines = receipt.content.trimEnd().split('\n');
  const expected = ['fieldOne', 'fieldTwo', 'fieldThree'].map(name =>
    capture('exact-field-receipt', name));
  if (lines.length !== 4 || expected.some((line, index) => lines[index] !== line)
      || !lines[3].startsWith(capture('exact-field-receipt', 'resultKey'))
      || !lines[3].includes(capture('exact-field-receipt', 'requiredIdentifier'))
      || proof.content.split('\n')[0] !== capture('proof-first-line', 'firstLine')) {
    throw Error('DifferentObservedNeedFields');
  }
  const physical = new Set(['exact-field-receipt', 'proof-target', 'proof-first-line',
    'tracked-source-condition', 'separate-git-evidence']);
  return Object.freeze(contract.clauses.map(clause => Object.freeze({
    kind: clause.kind, span: clause.span,
    observation: physical.has(clause.kind) ? 'PhysicalEvidenceObserved' : 'Unresolved',
    obligationStatus: 'Unresolved', nativeValidity: 'Unknown', verified: false,
    sourceAfterSha256: effect.afterSha256,
    artifacts: physical.has(clause.kind) ? Object.freeze(observed.slice()) : Object.freeze([])
  })));
}

/** Proposal bytes never grant permission; only the genuine live platform creates them. */
export async function executeTrackedSourceEvidence(token, request, workspace) {
  const current = installedHost(), operation = current.sourceOperation;
  const proposal = await proposeTrackedSourceEvidence(token, request, workspace);
  const validateHost = () => {
    if (installedHost().sourceOperation !== operation
        || installedHost().sourceSession !== current.sourceSession) throw Error('ReplacedTrackedSourceHost');
  };
  validateHost();
  if (typeof operation.createTrackedSourceEvidenceArtifact !== 'function'
      || typeof operation.trackedSourceEvidenceArtifactObservation !== 'function') {
    throw Error('MissingTrackedArtifactPlatform');
  }
  const receipts = [], observed = [];
  const observe = async (receipt, artifact) => {
    const value = await operation.trackedSourceEvidenceArtifactObservation(receipt, request, workspace);
    validateHost();
    if (!value || value.kind !== 'OnlyObservedTrackedArtifact'
        || value.path !== artifact.path || value.sha256 !== hash(artifact.content)
        || value.nativeValidity !== 'Unknown' || value.completed !== false) {
      throw Error('DifferentTrackedArtifactObservation');
    }
    return Object.freeze({path: value.path, sha256: value.sha256});
  };
  try {
    for (const artifact of proposal.artifacts) {
      await sourceHost.registeredTrackedSourceEffectObservation(token, request, workspace);
      validateHost();
      const receipt = await operation.createTrackedSourceEvidenceArtifact(
        token, request, workspace, artifact.path, artifact.content);
      validateHost();
      const value = await observe(receipt, artifact);
      receipts.push(receipt);
      observed.push(value);
    }
    for (let index = 0; index < receipts.length; index += 1) {
      await observe(receipts[index], proposal.artifacts[index]);
    }
    const effect = await sourceHost.registeredTrackedSourceEffectObservation(token, request, workspace);
    validateHost();
    const needObservations = physicalNeedEvidence(effect, proposal, observed);
    return Object.freeze({kind: 'ObservedTrackedSourceEvidenceArtifacts',
      artifacts: Object.freeze(observed), needObservations, sourceAfterSha256: proposal.sourceAfterSha256,
      unresolvedNeeds: proposal.unresolvedNeeds, nativeValidity: 'Unknown',
      completed: false, verified: false, writeAuthority: false});
  } catch (error) {
    return Object.freeze({kind: 'IncompleteTrackedSourceEvidenceArtifacts',
      failure: error.message, priorArtifactObservations: Object.freeze(observed),
      effectOutcome: 'Unknown', recoveryAuthority: 'Missing',
      unresolvedNeeds: proposal.unresolvedNeeds, nativeValidity: 'Unknown',
      completed: false, verified: false, writeAuthority: false});
  }
}
