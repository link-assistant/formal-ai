// Turn a captured Hive draft session into a small deterministic replay
// fixture (issue #1486).
//
// A draft attempt costs a Rust release build, a Docker image and a Hive solve.
// The planner decision that ended it, however, is a pure function of the
// OpenAI-compatible request the Hive client sent: the messages and the tool
// names. The draft evidence artifact keeps that request verbatim in
// `formal-ai-dialog/dialog_*.lino`, so the failing decision can be replayed
// offline in seconds through the JavaScript planner without Docker, a release
// or Rust compilation.
import { createHash } from 'node:crypto';
import { decodeDialogTyped, decodeTyped } from '../../js/logging/typed-codec.mjs';

export const DRAFT_REPLAY_SCHEMA = 'draft-request-replay/v1';

/** Decode every record of one dialog log; a line the codec rejects is an error. */
export function dialogRecords(text) {
  return String(text).split('\n').filter(line => line.startsWith('(formal-ai-log-v1 ')).map(line => {
    try {
      return decodeDialogTyped(line);
    } catch {
      return decodeTyped(line);
    }
  });
}

/** The assistant text a streamed (SSE) or plain chat completion carried. */
export function responseText(body) {
  const text = String(body ?? '');
  if (!text.trimStart().startsWith('data:')) {
    try {
      return JSON.parse(text).choices?.[0]?.message?.content ?? '';
    } catch {
      return '';
    }
  }
  let content = '';
  for (const line of text.split('\n')) {
    const payload = line.startsWith('data:') ? line.slice(5).trim() : '';
    if (!payload || payload === '[DONE]') continue;
    try {
      for (const choice of JSON.parse(payload).choices ?? []) content += choice.delta?.content ?? '';
    } catch {
      // A malformed chunk carries no content.
    }
  }
  return content;
}

/** The work-item `reason` a final answer reported, if it reported one. */
export function capturedReason(content) {
  const text = String(content ?? '');
  try {
    const value = JSON.parse(text);
    if (value && typeof value.reason === 'string') return value.reason;
  } catch {
    // A localized report is not JSON; fall through to the literal marker.
  }
  const match = /\b(ReadPolicyRefused|MissingReadTool|UnresolvedReferences|IncompleteSourceObservation|EvidenceCollected)\b/u.exec(text);
  return match ? match[1] : null;
}

function toolNames(request) {
  return (request.tools ?? []).map(tool => tool?.function?.name ?? tool?.name).filter(name => typeof name === 'string');
}

/**
 * The replay fixture for the final request of a captured dialog: the request
 * whose response ended the attempt. Every field the replay needs is copied from
 * the capture; nothing is reconstructed.
 */
export function extractDraftReplay(text, { issue, run = null, sourceCommit = null, family = null, mapping = null } = {}) {
  const records = dialogRecords(text).filter(record => typeof record.request_body === 'string');
  if (!records.length) throw Error('DialogHasNoRequest');
  const last = records.at(-1);
  const request = JSON.parse(last.request_body);
  if (!Array.isArray(request.messages) || !request.messages.length) throw Error('RequestHasNoMessages');
  const content = responseText(last.response_body);
  const reason = capturedReason(content);
  const resolvedFamily = family ?? familyForReason(mapping, reason);
  if (!resolvedFamily) throw Error('UnmappedFailureFamily:' + reason);
  return {
    schema: DRAFT_REPLAY_SCHEMA,
    issue,
    run,
    sourceCommit,
    family: resolvedFamily,
    captured: {
      dialogId: last.dialog_id ?? null,
      requestSha256: createHash('sha256').update(last.request_body).digest('hex'),
      reason,
      kind: reason === null ? 'tool_calls' : 'final'
    },
    tools: toolNames(request),
    messages: request.messages
  };
}

/** The failure family a captured reason belongs to, from the focused-test map. */
export function familyForReason(mapping, reason) {
  if (!mapping || !reason) return null;
  return mapping.families.find(entry => (entry.reasons ?? []).includes(reason))?.id ?? null;
}

/** Fail closed on a fixture that is not a complete, self-consistent replay. */
export function requireDraftReplay(fixture) {
  if (fixture?.schema !== DRAFT_REPLAY_SCHEMA) throw Error('DraftReplaySchema');
  if (!/^\d+$/u.test(String(fixture.issue))) throw Error('DraftReplayIssue');
  if (typeof fixture.family !== 'string' || !fixture.family) throw Error('DraftReplayFamily');
  if (!Array.isArray(fixture.messages) || !fixture.messages.length) throw Error('DraftReplayMessages');
  if (!Array.isArray(fixture.tools) || !fixture.tools.every(tool => typeof tool === 'string')) throw Error('DraftReplayTools');
  if (!/^[0-9a-f]{64}$/u.test(fixture.captured?.requestSha256 ?? '')) throw Error('DraftReplayRequestDigest');
  return fixture;
}
