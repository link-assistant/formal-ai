import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { sourceDigest } from '../../../experiments/formal_ai_subagent/coding-amplification.mjs';
import { writeJournalUsageReport } from '../../../experiments/formal_ai_subagent/cohort-usage-report.mjs';

// Synthetic source-bound accounting controls, never vendor usage or autonomous performance samples.
function fixture() {
  const dir = mkdtempSync(join(tmpdir(), 'captured-usage-report-'));
  const item = (runId, taskKind, category) => ({ runId, taskKind, category, task: 'Original ' + runId,
    taskSHA256: sourceDigest('Original ' + runId), expectedRelation: category === 'repair' ? 'unrestricted' : 'output-larger' });
  const manifest = { schemaVersion: 1, cohortId: 'synthetic-ingestion-only',
    cases: [item('coding', 'coding', 'feature-implementation'), item('self', 'self-coding', 'repair')] };
  const records = [{ kind: 'admitted', sequence: 0, previousDigest: null, cohortId: manifest.cohortId,
    manifestSHA256: sourceDigest(JSON.stringify(manifest)), manifest }];
  const append = (kind, runId, attemptId, result) => records.push({ kind, runId, attemptId,
    sequence: records.length, previousDigest: sourceDigest(JSON.stringify(records.at(-1))),
    manifestSHA256: records[0].manifestSHA256, taskSHA256: sourceDigest('Original ' + runId),
    ...(result === undefined ? {} : { result }) });
  const capture = (runId, attemptId, receiptId, inputTokens = 7, outputTokens = 3) => {
    const path = join(dir, receiptId + '.json');
    const producer = join(dir, 'synthetic-producer.mjs');
    writeFileSync(producer, '// retained synthetic producer, no vendor authenticity\n');
    const record = { schemaVersion: 1, kind: 'normalized-provider-usage', runId, attemptId,
      receiptId, requestId: receiptId, provider: 'synthetic-provider', model: 'synthetic-model',
      usage: { inputTokens, outputTokens } };
    writeFileSync(path, JSON.stringify(record));
    return { ...record, capture: { path, sha256: sourceDigest(readFileSync(path)) },
      producer: { path: producer, sha256: sourceDigest(readFileSync(producer)) } };
  };
  const result = (accepted, raw) => ({ accepted, usage: raw === undefined ? { status: 'Unknown' }
    : { status: 'CapturedUnnormalized', raw, rawSHA256: sourceDigest(JSON.stringify(raw)) } });
  const attempt = (runId, attemptId, accepted, raw) => {
    append('started', runId, attemptId);
    append('finished', runId, attemptId, result(accepted, raw));
  };
  const run = () => {
    const path = join(dir, 'journal.jsonl');
    writeFileSync(path, records.map(JSON.stringify).join('\n') + '\n');
    return writeJournalUsageReport({ path, sha256: sourceDigest(readFileSync(path)) }, join(dir, 'receipts-index.json'));
  };
  return { dir, records, append, capture, attempt, run, cleanup: () => rmSync(dir, { recursive: true, force: true }) };
}
const control = (name, body) => test(name, () => { const f = fixture(); try { body(f); } finally { f.cleanup(); } });

control('absent captures retain failed, retried, unfinished cases and declared categories with Unknown', f => {
  f.attempt('coding', 'first', false);
  f.attempt('coding', 'retry', true);
  f.append('started', 'self', 'first');
  const r = f.run();
  assert.equal(r.cases.length, 2);
  assert.equal(r.cases[0].attempts.length, 2);
  assert.equal(r.cases[0].attempts[0].accepted, false);
  assert.equal(r.cases[1].attempts[0].accepted, null);
  assert.equal(r.cases[1].expectedRelation, 'unrestricted');
  assert.equal(r.capturedReceiptIngestion, 'Unknown');
  assert.equal(r.fullModelUsageStatus, 'Unknown');
  assert.equal(r.actualCost, null);
});
control('the existing journal raw driver field automatically enters source-bound receipt accounting', f => {
  f.attempt('coding', 'first', false, f.capture('coding', 'first', 'first'));
  f.attempt('coding', 'retry', true, f.capture('coding', 'retry', 'retry', 11, 5));
  const r = f.run();
  assert.equal(r.capturedReceiptIngestion, 'SourceCaptured');
  assert.equal(r.cases[0].models[0].counters.inputTokens, 18);
  assert.equal(r.cases[0].attempts[0].accepted, false);
  assert.equal(r.cases[0].attempts[1].receiptIds[0], 'retry');
  assert.equal(r.providerAuthenticityRequiresReview, true);
  assert.equal(r.observedInputCompleteness, 'Unknown');
  assert.equal(r.semanticUsefulness, 'Unknown');
});
control('opaque provider payloads and token estimates are retained as Unknown without guessed normalization', f => {
  f.attempt('coding', 'first', true, { usage: { input_tokens: 9000, output_tokens: 8000 }, estimate: true });
  const r = f.run();
  assert.equal(r.rawCapturesObserved, 1);
  assert.equal(r.captureDispositions[0].status, 'Unknown');
  assert.equal(r.cases[0].models.length, 0);
});
control('supported capture identity mismatch rejects accounting without rewriting behavioral disposition', f => {
  f.attempt('coding', 'first', true, f.capture('self', 'first', 'wrong-owner'));
  const r = f.run();
  assert.equal(r.capturedReceiptIngestion, 'Rejected');
  assert.equal(r.cases[0].attempts[0].accepted, true);
  assert.equal(r.cases[0].attempts[0].usageStatus, 'Unknown');
  assert.equal(r.normalizationFailures.length, 1);
});
control('receipt producer and capture source drift refuse counters while preserving raw source identity', f => {
  const raw = f.capture('coding', 'first', 'drift');
  writeFileSync(raw.capture.path, '{}');
  f.attempt('coding', 'first', false, raw);
  const r = f.run();
  assert.equal(r.capturedReceiptIngestion, 'Rejected');
  assert.equal(r.cases[0].models.length, 0);
  assert.match(r.normalizationFailures[0].reason, /drift/);
});
control('duplicate provider requests across retries reject all partial aggregate counters', f => {
  const first = f.capture('coding', 'first', 'a');
  const second = f.capture('coding', 'retry', 'b');
  const record = JSON.parse(readFileSync(second.capture.path, 'utf8'));
  record.requestId = first.requestId;
  writeFileSync(second.capture.path, JSON.stringify(record));
  second.requestId = first.requestId;
  second.capture.sha256 = sourceDigest(readFileSync(second.capture.path));
  f.attempt('coding', 'first', false, first);
  f.attempt('coding', 'retry', true, second);
  const r = f.run();
  assert.equal(r.capturedReceiptIngestion, 'Rejected');
  assert.equal(r.cases[0].models.length, 0);
  assert.match(r.normalizationFailures[0].reason, /duplicate/);
});
control('missing counters and omitted retry captures stay Unknown with no estimated token fill', f => {
  f.attempt('coding', 'first', false, f.capture('coding', 'first', 'partial', null, null));
  f.attempt('coding', 'retry', true);
  const r = f.run();
  assert.equal(r.cases[0].models[0].counters.inputTokens, null);
  assert.equal(r.cases[0].attempts[0].usageStatus, 'Unknown');
  assert.equal(r.cases[0].attempts[1].usageStatus, 'Unknown');
  assert.equal(r.fullModelUsageStatus, 'Unknown');
});
control('receipt index is immutable exclusive output rather than a mutable overwrite', f => {
  f.attempt('coding', 'first', true);
  f.run();
  assert.throws(f.run, /EEXIST/);
});
control('raw capture hashing and journal chain validation do not trust arbitrary captured flags', f => {
  f.attempt('coding', 'first', true, f.capture('coding', 'first', 'a'));
  f.records.at(-1).result.usage.rawSHA256 = '0'.repeat(64);
  const r = f.run();
  assert.equal(r.capturedReceiptIngestion, 'Rejected');
  assert.equal(r.cases[0].models.length, 0);
});
control('unsupported fake assertion counters never become model usage or independent acceptance', f => {
  f.attempt('coding', 'first', false, { successfulAssertions: 1000, complete: true, trusted: true });
  const r = f.run();
  assert.equal(r.capturedReceiptIngestion, 'Unknown');
  assert.equal(r.cases[0].attempts[0].accepted, false);
  assert.equal(r.cases[0].models.length, 0);
});

control('optional captured aggregate overflow becomes Unknown without suppressing case outcomes', f => {
  f.attempt('coding', 'first', false, f.capture('coding', 'first', 'maximum', Number.MAX_SAFE_INTEGER, 0));
  f.attempt('coding', 'retry', true, f.capture('coding', 'retry', 'one', 1, 0));
  const r = f.run();
  assert.equal(r.capturedReceiptIngestion, 'Rejected');
  assert.equal(r.cases[0].attempts.length, 2);
  assert.equal(r.cases[0].attempts[0].accepted, false);
  assert.equal(r.cases[0].attempts[1].accepted, true);
  assert.equal(r.cases[0].models.length, 0);
  assert.equal(r.fullModelUsageStatus, 'Unknown');
  assert.ok(r.normalizationFailures.some(failure => failure.phase === 'provider-model-total'));
});

control('bounded optional receipt index rejects counters without suppressing failed, retried or unfinished outcomes', f => {
  const large = (attemptId) => {
    const raw = f.capture('coding', attemptId, attemptId);
    const original = JSON.parse(readFileSync(raw.capture.path));
    original.diagnostic = 'x'.repeat(550000);
    writeFileSync(raw.capture.path, JSON.stringify(original));
    raw.diagnostic = original.diagnostic;
    raw.capture.sha256 = sourceDigest(readFileSync(raw.capture.path));
    return raw;
  };
  f.attempt('coding', 'first', false, large('first'));
  f.attempt('coding', 'retry', true, large('retry'));
  f.append('started', 'self', 'unfinished');
  const r = f.run();
  assert.equal(r.capturedReceiptIngestion, 'Rejected');
  assert.equal(r.cases[0].attempts[0].accepted, false);
  assert.equal(r.cases[0].attempts[1].accepted, true);
  assert.equal(r.cases[1].attempts[0].accepted, null);
  assert.equal(r.cases[0].models.length, 0);
  assert.equal(r.captureDispositions.length, 2);
  assert.ok(r.normalizationFailures.some(failure => failure.phase === 'receipt-index-bound'));
  assert.equal(r.fullModelUsageStatus, 'Unknown');
  assert.equal(r.actualCost, null);
  assert.equal(r.semanticUsefulness, 'Unknown');
  const indexBytes = readFileSync(join(f.dir, 'receipts-index.json'));
  assert.ok(indexBytes.length <= 1048576);
  assert.deepEqual(JSON.parse(indexBytes).receipts, []);
  assert.ok(readFileSync(join(f.dir, 'journal.jsonl')).length > 1048576);
});
