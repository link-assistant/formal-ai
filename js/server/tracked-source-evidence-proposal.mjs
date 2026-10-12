import { host as installedHost } from '../agentic/host.mjs';
import * as sourceHost from './node-source-session-host.mjs';
import { renderSeededChange } from '../agentic/code_task.mjs';
import { firstRoleMatch, words } from '../agentic/crate/seed_meanings.mjs';

function clause(contract, kind) {
  const matches = contract.clauses.filter(value => value.kind === kind);
  if (matches.length !== 1) throw Error('AmbiguousEvidenceClause');
  return matches[0].captures;
}
function minimumWords(text) {
  if (/^[0-9]+$/u.test(text)) return Number(text);
  const meaning = firstRoleMatch('cardinal_number_word', text);
  const values = meaning ? words(meaning).filter(word => /^[0-9]+$/u.test(word)) : [];
  if (values.length !== 1) throw Error('UnboundEvidenceWordCount');
  return Number(values[0]);
}

/** An opaque live effect receipt can propose bytes, never authorize their Write. */
export async function proposeTrackedSourceEvidence(token, request, workspace) {
  const current = installedHost(), operation = current.sourceOperation;
  if (!operation || typeof sourceHost.registeredTrackedSourceEffectObservation !== 'function') {
    throw Error('MissingTrackedEffectReceiptPlatform');
  }
  const observation = await sourceHost.registeredTrackedSourceEffectObservation(token, request, workspace);
  if (installedHost().sourceOperation !== operation
      || installedHost().sourceSession !== current.sourceSession) throw Error('ReplacedTrackedSourceHost');
  const contract = observation?.referenceRequest;
  if (observation?.kind !== 'OnlyTrackedSourceEffect'
      || observation.originalRequest !== request || observation.workspace !== workspace
      || observation.beforeSha256 === observation.afterSha256
      || observation.afterSha256 !== observation.readbackSha256
      || observation.completed !== false || observation.artifactWriteAuthority !== 'Missing'
      || observation.nativeValidity !== 'Unknown' || !contract
      || contract.originalRequest !== request || contract.wholeRequestConsumed !== true
      || contract.writeAuthority !== false) throw Error('UnownedEvidenceSourceEffect');
  const rename = clause(contract, 'private-constant-rename');
  const fields = clause(contract, 'exact-field-receipt');
  const target = clause(contract, 'proof-target');
  const first = clause(contract, 'proof-first-line');
  if (rename.sourcePath.text !== observation.operand
      || fields.requiredIdentifier.text !== rename.replacement.text) throw Error('DifferentEvidenceSource');
  const result = renderSeededChange('coding-identifier-renamed', request, observation.operand,
    [['{old}', rename.previous.text], ['{new}', rename.replacement.text]]);
  const count = minimumWords(fields.minimumWords.text);
  if (!Number.isSafeInteger(count) || count < 0 || typeof result !== 'string'
      || !result.includes(fields.requiredIdentifier.text)
      || result.trim().split(/\s+/u).length < count || /[\r\n]/u.test(result)) {
    throw Error('UnsupportedEvidenceResultContract');
  }
  const receipt = [fields.fieldOne.text, fields.fieldTwo.text, fields.fieldThree.text,
    fields.resultKey.text + result].join('\n') + '\n';
  const proof = first.firstLine.text + '\n' + result + '\n';
  const refreshed = await sourceHost.registeredTrackedSourceEffectObservation(token, request, workspace);
  if (installedHost().sourceOperation !== operation
      || installedHost().sourceSession !== current.sourceSession
      || refreshed.referenceRequest !== contract
      || refreshed.afterSha256 !== observation.afterSha256
      || refreshed.inverseSha256 !== observation.inverseSha256) throw Error('StaleEvidenceSourceEffect');
  return Object.freeze({kind: 'TrackedSourceEvidenceProposal',
    artifacts: Object.freeze([
      Object.freeze({path: fields.receiptPath.text, content: receipt}),
      Object.freeze({path: target.proofPath.text, content: proof})]),
    originalRequest: request, sourceAfterSha256: observation.afterSha256,
    unresolvedNeeds: contract.unresolvedNeeds, writeAuthority: false,
    nativeValidity: 'Unknown', completed: false});
}
