import { host as installedHost } from "../host.mjs";
import { Progress } from "../progress.mjs";
import { Capability } from "../capability.mjs";
import { toolFor } from "../capability_router.mjs";
import { planOne, writeArguments } from "../plan.mjs";
import { sha256Hex } from "../crate/source_fetch.mjs";
import { deriveCompleteSourceRequest, bindCompleteSourceNeeds } from "./complete-source-preflight.mjs";
import { sourceNeedPreflight } from "./source-need-preflight.mjs";
import { deriveSeededSourcePlan } from "./seeded-source-plan.mjs";
import { createCandidateTransaction } from "./candidate-transaction.mjs";
const preparedWitnesses = new WeakMap();
/** The execution platform supplies the private operation and leased IO context. */
export function prepareSourceCandidate(operation, io, receipt, text, messages, workspace, toolNames) {
  if (!operation || installedHost().sourceOperation !== operation) throw Error('MissingSourceOperationHost');
  const writeTool = toolFor(toolNames, Capability.Write);
  if (writeTool === null) throw Error('MissingWriteTool');
  const frame = deriveCompleteSourceRequest(text, operation.requestSeed());
  const preflight = sourceNeedPreflight(text, messages);
  if (frame === null || preflight === null) throw Error('UnboundCompleteSourceRequest');
  if (!preflight.prerequisites.every(value => ['conditional-satisfied-read', 'conditional-satisfied-absence'].includes(value.status))) throw Error('UnsolvedImmutableReadPrerequisite');
  if (!preflight.prerequisites.some(value => value.role === 'destination' && value.status === 'conditional-satisfied-absence')) throw Error('UnprovedDestinationMerge');
  const request = frame.request,
    canonical = operation.acceptedSources(receipt, text, request.command, workspace);
  const descriptor = operation.operationDescriptor(receipt, text, request.command, workspace),
    progress = Progress.scan(messages);
  const observations = request.inputs.map(operand => {
    const read = progress.sourceReadFor(operand);
    if (!read || !read.complete || read.error !== null) throw Error('IncompleteSourceRead');
    const matching = canonical.filter(value => value.sha256 === sha256Hex(read.source) && value.content === read.source);
    if (matching.length !== 1) throw Error('AmbiguousCanonicalSourceCorrespondence');
    return {
      path: matching[0].path,
      content: read.source
    };
  });
  if (request.acceptance.length !== 1) throw Error('AmbiguousAcceptance');
  const acceptancePath = request.acceptance[0],
    acceptanceRead = progress.sourceReadFor(acceptancePath);
  if (!acceptanceRead || !acceptanceRead.complete || acceptanceRead.error !== null) throw Error('IncompleteAcceptanceRead');
  const composition = deriveSeededSourcePlan({
    ...request,
    text,
    identity: frame.requestIdentity
  }, observations, {
    path: acceptancePath,
    content: acceptanceRead.source
  }, descriptor, operation.compositionSeed());
  const conditionalProof = bindCompleteSourceNeeds(frame, composition, preflight, canonical);
  const revalidated = operation.acceptedSources(receipt, text, request.command, workspace);
  if (JSON.stringify(revalidated) !== JSON.stringify(canonical)) throw Error('CanonicalSourceDrift');
  const identity = sha256Hex(composition.source),
    transaction = createCandidateTransaction(io, request.destination, identity);
  const prepared = Object.freeze({
    plan: planOne(writeTool, writeArguments(request.destination, composition.source)),
    composition,
    conditionalProof
  });
  preparedWitnesses.set(prepared, {
    operation,
    transaction,
    text,
    command: request.command,
    workspace,
    destination: request.destination,
    identity
  });
  return prepared;
}
/** Only a privately prepared candidate can record its observed Write. */
export function authorizeSourceCandidateWrite(prepared) {
  const witness = preparedWitnesses.get(prepared);
  if (!witness) throw Error('UnownedPreparedCandidate');
  return witness.transaction.authorizeWrite();
}
export function recordPreparedSourceWrite(prepared) {
  const witness = preparedWitnesses.get(prepared);
  if (!witness) throw Error('UnownedPreparedCandidate');
  return witness.transaction.recordWrite();
}
/** Candidate-bound host status and independently owned process disposition both must agree. */
export function finishSourceCandidate(operation, prepared, receipt, processReceipt) {
  const witness = preparedWitnesses.get(prepared);
  if (!witness || witness.operation !== operation) throw Error('UnownedPreparedCandidate');
  let status = null,
    failure = null;
  try {
    status = operation.candidateStatus(receipt, witness.text, witness.command, witness.workspace, witness.destination, witness.identity);
  } catch (error) {
    failure = error.message;
  }
  let physicalDisposition;
  if (status === 0) {
    try {
      physicalDisposition = witness.transaction.finish(processReceipt);
    } catch (error) {
      failure = error.message;
      physicalDisposition = witness.transaction.abort();
    }
  } else physicalDisposition = witness.transaction.abort();
  return {
    authored: true,
    verified: status === 0 && physicalDisposition.state === 'committed',
    ownedStatus: status,
    verificationFailure: failure,
    physicalDisposition,
    universalEffectsProved: false
  };
}

/** A missing operation receipt never certifies a process; only owned unchanged bytes roll back. */
export function abortSourceCandidate(prepared) {
  const witness = preparedWitnesses.get(prepared);
  if (!witness) throw Error('UnownedPreparedCandidate');
  return {
    authored: true,
    verified: false,
    ownedStatus: null,
    verificationFailure: 'MissingOwnedVerificationReceipt',
    physicalDisposition: witness.transaction.abort(),
    universalEffectsProved: false
  };
}

/** Trusted executor callbacks retain the ordinary wire planner refusal boundary. */
export function executeSourceCandidate(operation, io, receipt, context, executeWrite, executeVerification) {
  const {
    request: text,
    messages,
    workspace,
    tools: toolNames
  } = context;
  if (typeof executeWrite !== 'function' || typeof executeVerification !== 'function') throw Error('MissingTrustedSourceExecutor');
  const prepared = prepareSourceCandidate(operation, io, receipt, text, messages, workspace, toolNames);
  const witness = preparedWitnesses.get(prepared);
  authorizeSourceCandidateWrite(prepared);
  try {
    executeWrite(prepared.plan);
  } catch (error) {
    try {
      recordPreparedSourceWrite(prepared);
    } catch {
      throw Error('UnownedWriteFailure');
    }
    return {
      ...abortSourceCandidate(prepared),
      executionFailure: error.message
    };
  }
  recordPreparedSourceWrite(prepared);
  let verification;
  try {
    verification = executeVerification(witness.command, witness.workspace);
  } catch (error) {
    return {
      ...abortSourceCandidate(prepared),
      executionFailure: error.message
    };
  }
  if (verification === null || typeof verification !== 'object') return abortSourceCandidate(prepared);
  return finishSourceCandidate(operation, prepared, verification.receipt, verification.processReceipt);
}
