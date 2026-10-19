// Transport-side reports are transcript evidence, not independent trust or licence authority.
import * as capturePolicy from './portable/coding_research_kernel.mjs';
import { plainText } from './content.mjs';
import { classifyTool } from './capability_router.mjs';
import { Capability } from './capability.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';

// Only a shallow JSON-shaped own-data record is inspected: getters and nested
// malformed values cannot execute or acquire authority through metadata parsing.
function reportRecord(value, allowed) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) return null;
  const prototype = Object.getPrototypeOf(value);
  if (prototype !== Object.prototype && prototype !== null) return null;
  const descriptors = Object.getOwnPropertyDescriptors(value), result = Object.create(null);
  for (const key of Reflect.ownKeys(descriptors)) {
    const descriptor = descriptors[key];
    if (typeof key !== 'string' || !allowed.includes(key) || !descriptor.enumerable
        || !Object.hasOwn(descriptor, 'value')) return null;
    result[key] = descriptor.value;
  }
  return result;
}

function reported(value, valid) {
  return valid ? { status: 'ProviderReported', value } : { status: 'Unknown' };
}

/** Mirrors source_fetch_observation in progress/source_fetch_observation.rs.
 * Content is never interpreted as a capture envelope. Only an explicit sidecar
 * on one exact, current-window tool receipt can carry provider-reported fields.
 */
export function sourceFetchObservation(messages, start, url, text) {
  const observation = {
    schema: 'formal-ai-source-observation-v1', url, sha256: sha256Hex(text),
    binding: { status: 'Unknown', callId: null, tool: null }, authority: 'Unknown',
    fetchedAt: reported(null, false), cached: reported(null, false),
    trust: reported(null, false), licence: reported(null, false),
  };
  const declarations = [];
  const receipts = [];
  for (let index = start; index < messages.length; index++) {
    const role = messages[index].role.toLowerCase();
    if (role === 'tool') receipts.push({ index, message: messages[index] });
    if (role !== 'assistant') continue;
    for (const call of messages[index].tool_calls ?? []) declarations.push({ call, index });
  }
  const matches = declarations.flatMap(({ call, index }) => {
    let args;
    try { args = JSON.parse(call.function?.arguments); } catch { args = null; }
    const results = receipts.filter(({ message }) => message.tool_call_id === call.id);
    const result = results[0]?.message;
    const declaration = capturePolicy.sourceCaptureDeclarationAccepted(
      typeof call.id === 'string' && call.id.length > 0,
      typeof call.function?.name === 'string' && classifyTool(call.function.name) === Capability.Fetch,
      args?.url === url, declarations.filter(item => item.call.id === call.id).length);
    const receipt = capturePolicy.sourceCaptureReceiptIdentityAccepted(results.length,
      results[0] !== undefined && results[0].index > index, result?.name === call.function?.name);
    const outcome = capturePolicy.sourceCaptureAllObserved(
      result !== undefined && !result.is_error && !result.isError,
      result !== undefined && plainText(result.content) === text, true);
    const eligible = capturePolicy.sourceCaptureAllObserved(declaration, receipt, outcome);
    return eligible ? [{ call, receipt: result }] : [];

  });
  if (!capturePolicy.sourceCaptureUniqueAccepted(matches.length)) return observation;
  const { call, receipt: message } = matches[0];
  observation.binding = { status: 'TranscriptBound', callId: call.id, tool: call.function.name };
  const keys = ['schema', 'callId', 'url', 'sha256', 'fetchedAt', 'cached', 'trust', 'licence'];
  const capture = reportRecord(message.source_capture, keys);
  const identity = capturePolicy.sourceCaptureAllObserved(
    capture?.schema === 'formal-ai-source-capture-v1', capture?.callId === call.id, capture?.url === url);
  if (!capturePolicy.sourceCaptureAllObserved(capture !== null, identity,
      capture?.sha256 === observation.sha256)) return observation;
  const timestamp = capture.fetchedAt;
  const timestampObserved = typeof timestamp === 'string';
  const timestampDecimal = timestampObserved && /^[0-9]+$/u.test(timestamp);
  const timestampLeadingZero = timestampObserved && timestamp.startsWith('0');
  const timestampLength = timestampObserved ? Array.from(timestamp).length : 0;
  const timestampSyntax = capturePolicy.sourceCaptureTimestampTextAccepted(
    timestampObserved, timestampDecimal, timestampLeadingZero, timestampLength);
  let timestampRepresentable = false;
  if (timestampSyntax) {
    try { const number = BigInt(timestamp); timestampRepresentable = BigInt.asUintN(64, number) === number; }
    catch { timestampRepresentable = false; }
  }
  observation.fetchedAt = reported(timestamp, capturePolicy.sourceCaptureAllObserved(
    timestampSyntax, timestampRepresentable, true));
  observation.cached = reported(capture.cached, typeof capture.cached === 'boolean');
  const trust = reportRecord(capture.trust, ['score', 'policy']);
  const scoreObserved = typeof trust?.score === 'number';
  const policyObserved = typeof trust?.policy === 'string';
  observation.trust = reported(trust && { score: trust.score === 0 ? 0 : trust.score, policy: trust.policy },
    capturePolicy.sourceCaptureTrustAccepted(trust !== null, trust ? Object.keys(trust).length : 0,
      scoreObserved, scoreObserved ? trust.score : 0, policyObserved,
      policyObserved ? Array.from(trust.policy).length : 0));
  const licence = reportRecord(capture.licence, ['name', 'url']);
  const nameObserved = typeof licence?.name === 'string', urlObserved = typeof licence?.url === 'string';
  observation.licence = reported(licence && { name: licence.name, url: licence.url },
    capturePolicy.sourceCaptureLicenceAccepted(licence !== null, licence ? Object.keys(licence).length : 0,
      nameObserved, nameObserved ? Array.from(licence.name).length : 0,
      urlObserved, urlObserved ? Array.from(licence.url).length : 0));
  return observation;
}
