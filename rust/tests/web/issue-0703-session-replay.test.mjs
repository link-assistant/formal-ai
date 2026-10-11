// Issue #703 R703-7: a recorded external-agent session replays byte for byte,
// and an edit, a truncation, a reordering or a broken ancestry is rejected
// with the error that names it. JavaScript first: js/agentic/crate/
// orchestration_replay.mjs mirrors rust/src/orchestration/replay.rs, and
// rust/tests/integration/issue_703_replay_tamper.rs pins the Rust twin on the
// same committed sessions and the same tampering.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

import {
  ReplayError,
  canonicalSessionText,
  replayContinuation,
  replaySession,
  sessionSha256,
} from '../../../js/agentic/crate/orchestration_replay.mjs';

const root = new URL('../../../', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8');

// Every committed session file whose top level is a session (dispatch
// reports that embed sessions are read by their own tests).
const COMMITTED_SESSIONS = [
  'docs/case-studies/issue-703/comparison/sessions/000-codex.json',
  'docs/case-studies/issue-703/comparison/sessions/001-claude.json',
  'docs/case-studies/issue-703/controller-agent-run/controller-session.json',
  'docs/case-studies/issue-703/followup-authorship/controller-session.json',
  'docs/case-studies/issue-703/followup-authorship/corrected-session.json',
  'docs/case-studies/issue-703/followup-authorship/final-session.json',
  'docs/case-studies/issue-879/formal-ai-authorship-agent-session-3.json',
  'docs/case-studies/issue-879/formal-ai-authorship-requirements-session.json',
  'docs/case-studies/issue-879/formal-ai-authorship-verification-session.json',
  'docs/case-studies/issue-921/formal-ai-to-hive-mind/failure-session.json',
  'docs/case-studies/issue-921/formal-ai-to-hive-mind/orchestration-session.json',
  'docs/case-studies/issue-924/incremental-self-authorship/sessions/000-agent.json',
  'docs/case-studies/issue-924/incremental-self-authorship/sessions/001-agent.json',
  'docs/case-studies/issue-924/incremental-self-authorship/sessions/002-agent.json',
  'docs/case-studies/issue-924/incremental-self-authorship/sessions/003-agent.json',
  'docs/case-studies/issue-924/incremental-self-authorship/sessions/004-composed-verifier.json',
  'docs/case-studies/issue-933/self-hosting-authorship/sessions/000-agent.json',
  'docs/case-studies/issue-933/self-hosting-authorship/sessions/001-agent.json',
  'docs/case-studies/issue-933/self-hosting-authorship/sessions/002-agent.json',
  'docs/case-studies/issue-933/self-hosting-authorship/sessions/003-agent.json',
  'docs/case-studies/issue-933/self-hosting-authorship/sessions/004-agent.json',
  'docs/case-studies/issue-938/agent-cli-session.json',
];

const HIVE = 'docs/case-studies/issue-921/formal-ai-to-hive-mind/orchestration-session.json';
const CHAIN = 'docs/case-studies/issue-703/followup-authorship/';

/** The `Display` rendering of the error `replay` throws for `text`. */
const rejection = (replay) => {
  try {
    replay();
  } catch (error) {
    assert.ok(error instanceof ReplayError, String(error));
    return error.message;
  }
  assert.fail('the tampered session replayed');
};

/** Parse, edit, and re-render canonically, the way a careful tamperer would. */
const edited = (path, edit) => {
  const session = JSON.parse(read(path));
  edit(session);
  return canonicalSessionText(session);
};

test('R703-7: every committed session replays from its exact bytes', () => {
  for (const path of COMMITTED_SESSIONS) {
    const text = read(path);
    const session = replaySession(text);
    assert.equal(canonicalSessionText(session), text, path);
  }
  const hive = replaySession(read(HIVE));
  assert.deepEqual(hive.events.map((event) => event.kind), [
    'permission_granted',
    'adapter_selected',
    'process_started',
    'process_succeeded',
    'verification_started',
    'verification_passed',
    'workspace_effect',
  ]);
  assert.equal(hive.status, 'succeeded');
});

test('R703-7: edits, truncation and reordering are rejected by name', () => {
  const text = read(HIVE);
  assert.equal(rejection(() => replaySession(text.replace('"detail": "formal-ai-to-hive-mind.txt"', '"detail": "elsewhere.txt"'))), 'event_digest:6');
  assert.equal(rejection(() => replaySession(edited(HIVE, (session) => {
    [session.events[4], session.events[5]] = [session.events[5], session.events[4]];
  }))), 'event_sequence:4');
  assert.equal(rejection(() => replaySession(edited(HIVE, (session) => session.events.splice(4, 1)))), 'event_sequence:4');
  // Dropping the last event leaves a valid chain; the effect it stood for is
  // still listed in `changes`, so the truncation shows as a binding failure.
  assert.equal(rejection(() => replaySession(edited(HIVE, (session) => session.events.pop()))), 'event_binding:effects');
  assert.equal(rejection(() => replaySession(edited(HIVE, (session) => { session.status = 'failed'; }))), 'event_binding:status');
  assert.equal(rejection(() => replaySession(edited(HIVE, (session) => { session.changes[0].path = 'elsewhere.txt'; }))), 'event_binding:effects');
  assert.equal(rejection(() => replaySession(edited(`${CHAIN}corrected-session.json`, (session) => { delete session.continuation; }))), 'event_binding:continuation');
  assert.equal(rejection(() => replaySession(edited(HIVE, (session) => { session.schema = 'formal-ai-agent-session-v0'; }))), 'unsupported_session_schema');
  assert.equal(rejection(() => replaySession(JSON.stringify(JSON.parse(text)))), 'non_canonical_session');
  assert.equal(rejection(() => replaySession(`${text}\n`)), 'non_canonical_session');
  assert.equal(rejection(() => replaySession(edited(HIVE, (session) => { session.native_session = null; }))), 'non_canonical_session');
  const reordered = edited(HIVE, (session) => {
    const { schema, cli, ...rest } = session;
    Object.keys(session).forEach((key) => delete session[key]);
    Object.assign(session, { cli, schema, ...rest });
  });
  assert.equal(rejection(() => replaySession(reordered)), 'non_canonical_session');
  const truncated = rejection(() => replaySession(text.slice(0, text.length / 2)));
  assert.ok(truncated.startsWith('json:'), truncated);
});

test('R703-7: a correction replays only against the exact parent bytes it names', () => {
  const controller = read(`${CHAIN}controller-session.json`);
  const corrected = read(`${CHAIN}corrected-session.json`);
  const final = read(`${CHAIN}final-session.json`);
  assert.equal(replayContinuation(controller, corrected).continuation.parent_session_sha256, sessionSha256(replaySession(controller)));
  assert.equal(replayContinuation(corrected, final).continuation.parent_session_sha256, sessionSha256(replaySession(corrected)));
  assert.equal(rejection(() => replayContinuation(controller, final)), 'broken_ancestry');
  assert.equal(rejection(() => replayContinuation(read(HIVE), read(HIVE))), 'broken_ancestry');
  // Free text such as stdout is outside the event chain: an edited parent
  // still replays alone, and the child's parent digest is what exposes it.
  const editedParent = edited(`${CHAIN}controller-session.json`, (session) => { session.stdout += 'edited'; });
  assert.equal(replaySession(editedParent).stdout.endsWith('edited'), true);
  assert.equal(rejection(() => replayContinuation(editedParent, corrected)), 'broken_ancestry');
});
