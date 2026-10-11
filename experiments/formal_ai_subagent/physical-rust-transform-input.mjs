import { registeredPhysicalReadObservation } from '../../js/server/node-source-session-host.mjs';
import { prepareRustConstantRename } from '../../js/server/rust-source-transform.mjs';

/** A structural candidate retains every original obligation; no effect is authorized. */
export async function preparePhysicalRustRename(token, request, workspace, declaration) {
  const observed = registeredPhysicalReadObservation(token, request, workspace, declaration);
  const contract = observed.referenceRequest;
  if (!contract || contract.wholeRequestConsumed !== true || contract.originalRequest !== request) {
    throw Error('MissingCurrentReferenceRequest');
  }
  const rename = contract.clauses.find(clause => clause.kind === 'private-constant-rename');
  if (!rename || observed.operand !== rename.captures.sourcePath.text) {
    throw Error('DifferentPhysicalRustRenameOperand');
  }
  const candidate = await prepareRustConstantRename(
    observed.content, rename.captures.previous.text, rename.captures.replacement.text);
  if (candidate === null) throw Error('UnsupportedPhysicalRustRename');
  const current = registeredPhysicalReadObservation(token, request, workspace, declaration);
  if (current.sha256 !== observed.sha256 || current.referenceRequest !== contract
      || candidate.beforeSha256 !== observed.sha256) throw Error('PhysicalRustRenameSourceDrift');
  return Object.freeze({
    kind: 'OnlyPhysicalRustRenameCandidate',
    originalRequest: request,
    source: current,
    contract,
    candidate,
    pendingNeeds: contract.unresolvedNeeds,
    trackedGitEffect: 'Missing',
    writeAuthority: false,
    receiptAuthority: 'Missing',
    proofAuthority: 'Missing',
    workerLaunchAuthority: 'Missing',
    compilerValidity: 'Unknown',
    namespaceAuthority: 'Unknown',
    raceProof: false,
  });
}
