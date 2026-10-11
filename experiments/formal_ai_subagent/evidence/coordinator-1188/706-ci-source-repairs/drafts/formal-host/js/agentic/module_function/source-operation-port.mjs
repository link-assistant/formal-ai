import { host } from "../host.mjs";
/** The installed host validates its own receipt; request data cannot implement this port. */
export function acceptedOperationSourceBytes(receipt, request, command, workspace) {
  const operation = host().sourceOperation;
  if (!operation || typeof operation.acceptedSources !== 'function') throw new Error('MissingSourceOperationHost');
  return operation.acceptedSources(receipt, request, command, workspace);
}

/** Resolution is a host identity operation; this function grants no effect authority. */
export function sourceOperationModuleURL(specifier, parent) {
  const operation = host().sourceOperation;
  if (!operation || typeof operation.moduleURL !== 'function') throw Error('MissingSourceModuleIdentityHost');
  return operation.moduleURL(specifier, parent);
}

/** Only a host receipt captured while the exact candidate existed can certify verification. */
export function acceptedOperationCandidateStatus(receipt, request, command, workspace, destination, identity) {
  const operation = host().sourceOperation;
  if (!operation || typeof operation.finalizedCandidateStatus !== 'function') throw Error('MissingCandidateOperationHost');
  return operation.finalizedCandidateStatus(receipt, request, command, workspace, destination, identity);
}
