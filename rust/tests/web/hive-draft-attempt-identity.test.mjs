// Issue #1484: fast controls for source drift, stale/missing artifacts and attempt-mode credit.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {ATTEMPT_MODES, ATTEMPT_OUTCOMES, CAPTURE_OUTCOME_SCHEMA, FILES, attemptModeOf, currentSourceCredit, parseAttemptIdentity, selectAttemptIdentity,
  summarizeAttempt, verifyAttemptCustody, verifyHiveAttempt} from '../../../scripts/lib/hive-draft-attempt-identity.mjs';
import {main} from '../../../scripts/hive-draft-attempt.mjs';
import {failureComment} from '../../../scripts/hive-mind-draft-result.mjs';
import {workflowJobs} from '../../../scripts/lib/ci-speed-workflows.mjs';

const root = path.resolve(import.meta.dirname, '../../..');

const formalSha = 'a'.repeat(40), hiveSha = 'b'.repeat(40), digest = 'c'.repeat(64), staleDigest = 'd'.repeat(64);
const selection = (overrides = {}) => ({
  repository: 'link-assistant/formal-ai', issue: 1484,
  formalAi: {sourceSha: formalSha, checkedOutSha: formalSha, version: '0.355.29', binarySha256: digest, binaryBytes: 10},
  hive: {sourceSha: hiveSha, checkedOutSha: hiveSha, version: '2.40.4', image: 'konard/hive-mind:2.40.4'},
  selected: {at: '2026-11-15T00:00:00Z', runId: '7', runAttempt: '1'},
  ...overrides,
});
const outcomeOf = fn => { try { fn(); } catch (error) { return error.outcome; } return 'no error'; };
const admitted = () => selectAttemptIdentity(selection());
const hiveIdentityFor = identity => ({schema: 'hive-mind/formal-ai-draft-attempt-identity/v1', id: 'h'.repeat(32), mode: identity.mode,
  repository: identity.repository, issue: identity.issue,
  formalAi: {sourceSha: identity.formalAi.sourceSha, binarySource: 'caller', callerBinarySha256: identity.formalAi.binarySha256},
  hive: {sourceSha: identity.hive.sourceSha, image: identity.hive.image}});
const hiveRecordFor = (identity, servingExe = identity.formalAi.binarySha256, servingVersion = identity.formalAi.version) => ({
  schema: 'hive-mind/formal-ai-draft-attempt-record/v1', attemptId: 'h'.repeat(32), binary: {sha256: identity.formalAi.binarySha256},
  image: {reference: 'konard/hive-mind@sha256:' + 'e'.repeat(64), digest: 'sha256:' + 'e'.repeat(64), revision: identity.hive.sourceSha},
  serving: {servingExeSha256: servingExe, formalAiVersion: servingVersion, hiveVersion: identity.hive.version, agentVersion: '0.9.1', probedAt: '2026-11-15T00:00:00.000Z'}});

test('one selection binds both sources, survives retries and changes with a new source selection', () => {
  const identity = admitted();
  assert.equal(identity.mode, ATTEMPT_MODES.liveCurrent);
  assert.equal(identity.currentSource, true);
  assert.match(identity.id, /^[0-9a-f]{32}$/u);
  const retry = selectAttemptIdentity(selection({selected: {at: '2026-11-15T01:00:00Z', runId: '7', runAttempt: '2'}}));
  assert.equal(retry.id, identity.id, 'a retry of the same run and sources keeps the attempt identity');
  const advancedFormalAi = selectAttemptIdentity(selection({formalAi: {...selection().formalAi, sourceSha: 'f'.repeat(40), checkedOutSha: 'f'.repeat(40)}}));
  const advancedHive = selectAttemptIdentity(selection({hive: {...selection().hive, sourceSha: 'e'.repeat(40), checkedOutSha: 'e'.repeat(40)}}));
  const newRun = selectAttemptIdentity(selection({selected: {runId: '8'}}));
  assert.equal(new Set([identity.id, advancedFormalAi.id, advancedHive.id, newRun.id]).size, 4);
  assert.deepEqual(parseAttemptIdentity(JSON.stringify(identity)), identity);
});

test('source drift, inexact images and missing binaries never become an attempt', () => {
  const drift = selection({formalAi: {...selection().formalAi, checkedOutSha: 'f'.repeat(40)}});
  assert.equal(outcomeOf(() => selectAttemptIdentity(drift)), ATTEMPT_OUTCOMES.mismatch);
  assert.equal(outcomeOf(() => selectAttemptIdentity(selection({hive: {...selection().hive, checkedOutSha: 'f'.repeat(40)}}))), ATTEMPT_OUTCOMES.mismatch);
  for (const image of ['konard/hive-mind:latest', 'konard/hive-mind:2.39.11', ''])
    assert.equal(outcomeOf(() => selectAttemptIdentity(selection({hive: {...selection().hive, image}}))), ATTEMPT_OUTCOMES.mismatch, image);
  assert.equal(outcomeOf(() => selectAttemptIdentity(selection({formalAi: {...selection().formalAi, binarySha256: undefined}}))), ATTEMPT_OUTCOMES.deferred);
  const edited = {...admitted(), formalAi: {...admitted().formalAi, sourceSha: 'f'.repeat(40)}};
  assert.equal(outcomeOf(() => parseAttemptIdentity(edited)), ATTEMPT_OUTCOMES.mismatch, 'an identity edited between jobs is refused');
});

test('custody rejects a stale binary of the same version before any task effect', () => {
  const identity = admitted();
  const binary = {sha256: digest, version: 'formal-ai 0.355.29'};
  assert.equal(verifyAttemptCustody({identity, binary, hiveCheckoutSha: hiveSha, issue: '1484'}).binarySha256, digest);
  assert.equal(outcomeOf(() => verifyAttemptCustody({identity, binary: null, hiveCheckoutSha: hiveSha, issue: 1484})), ATTEMPT_OUTCOMES.deferred);
  assert.equal(outcomeOf(() => verifyAttemptCustody({identity, binary: {...binary, sha256: staleDigest}, hiveCheckoutSha: hiveSha, issue: 1484})), ATTEMPT_OUTCOMES.mismatch,
    'version equality alone is insufficient');
  assert.equal(outcomeOf(() => verifyAttemptCustody({identity, binary: {...binary, version: 'formal-ai 0.355.15'}, hiveCheckoutSha: hiveSha, issue: 1484})), ATTEMPT_OUTCOMES.mismatch);
  assert.equal(outcomeOf(() => verifyAttemptCustody({identity, binary, hiveCheckoutSha: 'f'.repeat(40), issue: 1484})), ATTEMPT_OUTCOMES.mismatch);
  assert.equal(outcomeOf(() => verifyAttemptCustody({identity, binary, hiveCheckoutSha: hiveSha, issue: 1})), ATTEMPT_OUTCOMES.mismatch);
});

test('only Hive Mind serving the admitted executable earns current-source credit', () => {
  const identity = admitted();
  const verified = verifyHiveAttempt({identity, hiveIdentity: hiveIdentityFor(identity), hiveRecord: hiveRecordFor(identity)});
  assert.equal(verified.outcome, ATTEMPT_OUTCOMES.verified);
  assert.equal(verified.currentSourceCredit, true);
  assert.equal(verified.imageDigest, 'sha256:' + 'e'.repeat(64), 'the receipt names the image by digest, not by a movable tag');
  assert.equal(verified.agentVersion, '0.9.1');
  const record = hiveRecordFor(identity);
  const tagOnly = verifyHiveAttempt({identity, hiveIdentity: hiveIdentityFor(identity), hiveRecord: {...record, image: {reference: identity.hive.image}}});
  assert.deepEqual(tagOnly.differences.map(row => row.field), ['record.image.digest']);
  const otherHive = verifyHiveAttempt({identity, hiveIdentity: hiveIdentityFor(identity), hiveRecord: {...record, serving: {...record.serving, hiveVersion: '2.39.11'}}});
  assert.deepEqual(otherHive.differences.map(row => row.field), ['record.serving.hiveVersion']);
  const stale = verifyHiveAttempt({identity, hiveIdentity: hiveIdentityFor(identity), hiveRecord: hiveRecordFor(identity, staleDigest)});
  assert.equal(stale.outcome, ATTEMPT_OUTCOMES.mismatch);
  assert.deepEqual(stale.differences.map(row => row.field), ['record.serving.servingExeSha256']);
  const imageBaked = verifyHiveAttempt({identity, hiveIdentity: {...hiveIdentityFor(identity), formalAi: {sourceSha: null, binarySource: 'image-baked', callerBinarySha256: null}}, hiveRecord: hiveRecordFor(identity, staleDigest, '0.355.15')});
  assert.equal(imageBaked.outcome, ATTEMPT_OUTCOMES.mismatch);
  assert.equal(imageBaked.currentSourceCredit, false);
  const legacy = verifyHiveAttempt({identity});
  assert.equal(legacy.outcome, ATTEMPT_OUTCOMES.servingUnverified);
  assert.equal(legacy.currentSourceCredit, false);
  for (const outcome of [ATTEMPT_OUTCOMES.deferred, ATTEMPT_OUTCOMES.mismatch]) {
    const receipt = verifyHiveAttempt({identity, hiveIdentity: hiveIdentityFor(identity), hiveOutcome: {outcome, message: 'exact image konard/hive-mind:2.40.4 is not published'}});
    assert.equal(receipt.outcome, outcome);
    assert.equal(receipt.currentSourceCredit, false);
  }
});

test('historical replays and release qualifications are distinct and never current-source success', () => {
  const historicalLedgerClaim = {schema: 'formal-ai-hive-attempt-claim/v1', taskMode: 'historical-1364', state: 'success', outcome: 'verified', servingVerified: true};
  assert.equal(attemptModeOf(historicalLedgerClaim), ATTEMPT_MODES.historicalReplay);
  assert.equal(currentSourceCredit(historicalLedgerClaim), false);
  for (const mode of [ATTEMPT_MODES.historicalReplay, ATTEMPT_MODES.publishedRelease])
    assert.equal(currentSourceCredit({mode, outcome: ATTEMPT_OUTCOMES.verified, servingVerified: true}), false, mode);
  assert.throws(() => currentSourceCredit({outcome: ATTEMPT_OUTCOMES.verified, servingVerified: true}), /no attempt mode/);
  assert.match(summarizeAttempt(admitted()), /live current-source evaluation \(`live-current`\)/);
  assert.match(summarizeAttempt({...admitted(), mode: ATTEMPT_MODES.historicalReplay}), /historical pinned replay/);
  assert.match(failureComment({runUrl: 'https://example/run', outcome: 'deferred-infrastructure'}), /Attempt outcome: `deferred-infrastructure` \(zero capability credit\)/);
  assert.doesNotMatch(failureComment({runUrl: 'https://example/run', outcome: 'x`](https://evil)'}), /evil/);
});

test('CLI admits once, then refuses a swapped artifact with a typed outcome file', t => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hive-attempt-'));
  t.after(() => { fs.rmSync(dir, {recursive: true, force: true}); process.exitCode = 0; });
  const hiveDir = path.join(dir, 'hive');
  fs.mkdirSync(hiveDir);
  fs.writeFileSync(path.join(hiveDir, 'package.json'), JSON.stringify({version: '2.40.4'}));
  const outputs = path.join(dir, 'outputs');
  const env = {FORMAL_AI_DRAFT_LOG_DIR: dir, GITHUB_OUTPUT: outputs, GITHUB_REPOSITORY: 'link-assistant/formal-ai', GITHUB_RUN_ID: '7', GITHUB_RUN_ATTEMPT: '1',
    FORMAL_AI_DRAFT_ISSUE: '1484', FORMAL_AI_SELECTED_SHA: formalSha, FORMAL_AI_SELECTED_AT: '2026-11-15T00:00:00Z', FORMAL_AI_BINARY: 'built',
    HIVE_MIND_DIR: hiveDir, HIVE_MIND_SOURCE_SHA: hiveSha, HIVE_MIND_IMAGE: 'konard/hive-mind:2.40.4'};
  // Issue #1569: the CLI's job-log lines are collected, so the refusals below are asserted here instead of
  // surfacing as `##[error]` annotations on the passing js tier.
  const logged = [];
  const io = binaries => ({gitHead: dir => (dir === hiveDir ? hiveSha : formalSha), inspectBinary: name => binaries[name] ?? null,
    log: line => logged.push(line)});
  const identity = main(env, ['--select'], io({built: {sha256: digest, bytes: 10, version: 'formal-ai 0.355.29'}}));
  assert.equal(process.exitCode ?? 0, 0);
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(dir, FILES.identity), 'utf8')), identity);
  const line = fs.readFileSync(outputs, 'utf8').split('\n').find(row => row.startsWith('attempt_identity='));
  const draftEnv = {...env, FORMAL_AI_ATTEMPT_IDENTITY: line.slice('attempt_identity='.length), FORMAL_AI_DRAFT_BINARY: 'downloaded'};
  assert.equal(main(draftEnv, ['--verify-custody'], io({downloaded: {sha256: digest, version: 'formal-ai 0.355.29'}})).id, identity.id);
  assert.equal(process.exitCode ?? 0, 0);
  const swapped = main(draftEnv, ['--verify-custody'], io({downloaded: {sha256: staleDigest, version: 'formal-ai 0.355.29'}}));
  assert.equal(process.exitCode, 1);
  assert.equal(swapped.outcome, ATTEMPT_OUTCOMES.mismatch);
  assert.equal(JSON.parse(fs.readFileSync(path.join(dir, FILES.outcome), 'utf8')).capabilityCredit, 0);
  process.exitCode = 0;
  assert.equal(main(draftEnv, ['--verify-custody'], io({})).outcome, ATTEMPT_OUTCOMES.deferred);
  assert.equal(main({...draftEnv, FORMAL_AI_ATTEMPT_IDENTITY: ''}, ['--verify-custody'], io({})).outcome, ATTEMPT_OUTCOMES.deferred);
  process.exitCode = 0;
  const errors = logged.filter(entry => entry.startsWith('::error '));
  assert.equal(errors.length, 3, logged.join('\n'));
  assert.match(errors[0], /^::error title=Formal AI draft identity-mismatch::/u);
  assert.match(errors[2], /no attempt identity was admitted by the build job/u);
  fs.writeFileSync(path.join(dir, FILES.hiveIdentity), JSON.stringify(hiveIdentityFor(identity)));
  fs.writeFileSync(path.join(dir, FILES.hiveRecord), JSON.stringify(hiveRecordFor(identity)));
  assert.equal(main(draftEnv, ['--verify-hive'], io({})).id, identity.id);
  assert.equal(JSON.parse(fs.readFileSync(path.join(dir, FILES.receipt), 'utf8')).currentSourceCredit, true);
  assert.match(fs.readFileSync(outputs, 'utf8'), /current_source_credit=true/);
});

test('both issue-open jobs carry the identity and verify it around the shared Hive attempt', () => {
  const workflow = fs.readFileSync(new URL('../../../.github/workflows/hive-mind-draft.yml', import.meta.url), 'utf8');
  const [build, draft] = workflowJobs(workflow);
  assert.ok(build.body.indexOf('cargo build') < build.body.indexOf('hive-draft-attempt.mjs --select'));
  assert.match(build.body, /attempt_identity: \$\{\{ steps.identity.outputs.attempt_identity \}\}/);
  assert.match(build.body, /version='\+version/);
  const custody = draft.body.indexOf('--verify-custody'), attempt = draft.body.indexOf('id: attempt'), serving = draft.body.indexOf('--verify-hive');
  assert.ok(custody > 0 && custody < attempt && attempt < serving);
  assert.match(draft.body, /FORMAL_AI_DRAFT_SOURCE_SHA: \$\{\{ needs.build.outputs.source_sha \}\}/);
  assert.match(draft.body, /FORMAL_AI_DRAFT_MODE: live-current/);
  assert.match(draft.body, /FORMAL_AI_ATTEMPT_IDENTITY: \$\{\{ needs.build.outputs.attempt_identity \}\}/);
  assert.match(draft.body, /steps.serving.outputs.attempt_outcome != 'identity-mismatch'/);
  assert.match(draft.body, /Download the exact-main binary once[\s\S]*?continue-on-error: true/);
  assert.match(draft.body, /DRAFT_ATTEMPT_OUTCOME: \$\{\{ steps.custody.outputs.attempt_outcome/);
  assert.doesNotMatch(workflow, /hive-mind:latest|FORMAL_AI_DRAFT_ATTEMPT_IDENTITY/);
});

// Issue #1569: run 43691120627 ran solve for seven minutes, yet its receipt said `deferred-infrastructure`,
// "Hive Mind ended the attempt before solve". The only `attempt-outcome.json` in the session directory was
// the capture wrapper's own record of the attempt phase, read as if Hive Mind had written it.
test('the capture record of the attempt phase is never read as a Hive Mind outcome', t => {
  const captureSource = fs.readFileSync(path.join(root, 'scripts/capture-draft-command.mjs'), 'utf8');
  assert.ok(captureSource.includes("schema:'" + CAPTURE_OUTCOME_SCHEMA + "'"), 'the capture wrapper still writes ' + CAPTURE_OUTCOME_SCHEMA);
  assert.match(captureSource, /phase\+'-outcome\.json'/u);
  const phases = new Set();
  for (const directory of ['.github/workflows', '.github/actions'])
    for (const entry of fs.readdirSync(path.join(root, directory), {recursive: true}).filter(name => /\.ya?ml$/u.test(name)))
      for (const match of fs.readFileSync(path.join(root, directory, entry), 'utf8').matchAll(/run-with-session-evidence\.sh"? +\d+ +([a-z][a-z-]*)/gu)) phases.add(match[1]);
  assert.ok(phases.has('attempt') && phases.has('build'), [...phases].join(','));
  for (const phase of phases) assert.notEqual(FILES.hiveOutcome, phase + '-outcome.json', phase);

  const identity = admitted();
  // The record run 43691120627 left behind: solve finished with exit code 5 and no Hive contract files.
  const capture = {schema: CAPTURE_OUTCOME_SCHEMA, state: 'finished', budgetSeconds: 1200, exitCode: 5, commandExitCode: 5,
    deadlineExpired: false, error: null, captureExitCode: 0, completeConsoleCapture: true};
  const direct = verifyHiveAttempt({identity, hiveOutcome: capture});
  assert.equal(direct.outcome, ATTEMPT_OUTCOMES.servingUnverified);
  assert.match(direct.reason, /hive-mind#3909/u);

  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hive-attempt-capture-'));
  t.after(() => { fs.rmSync(dir, {recursive: true, force: true}); process.exitCode = 0; });
  fs.writeFileSync(path.join(dir, 'attempt-outcome.json'), JSON.stringify(capture));
  const outputs = path.join(dir, 'outputs');
  const logged = [];
  const receipt = main({FORMAL_AI_DRAFT_LOG_DIR: dir, GITHUB_OUTPUT: outputs, FORMAL_AI_ATTEMPT_IDENTITY: JSON.stringify(identity)},
    ['--verify-hive'], {log: line => logged.push(line)});
  assert.equal(receipt.id, identity.id);
  const written = JSON.parse(fs.readFileSync(path.join(dir, FILES.receipt), 'utf8'));
  assert.equal(written.outcome, ATTEMPT_OUTCOMES.servingUnverified);
  assert.doesNotMatch(written.reason, /before solve/u);
  assert.match(fs.readFileSync(outputs, 'utf8'), /attempt_outcome=serving-identity-unverified/u);
  assert.equal(logged.filter(line => line.startsWith('::error')).length, 0);

  fs.writeFileSync(path.join(dir, FILES.hiveOutcome), JSON.stringify({outcome: ATTEMPT_OUTCOMES.deferred, message: 'exact image is not published'}));
  main({FORMAL_AI_DRAFT_LOG_DIR: dir, GITHUB_OUTPUT: outputs, FORMAL_AI_ATTEMPT_IDENTITY: JSON.stringify(identity)}, ['--verify-hive'], {log: line => logged.push(line)});
  const deferred = JSON.parse(fs.readFileSync(path.join(dir, FILES.receipt), 'utf8'));
  assert.equal(deferred.outcome, ATTEMPT_OUTCOMES.deferred, 'a real Hive Mind outcome is still preserved');
  assert.equal(deferred.reason, 'exact image is not published');
});
