// Optional source-captured accounting; never provider authenticity or complete usage proof.
import { readFileSync, lstatSync, openSync, writeFileSync, fsyncSync, closeSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { sourceDigest, normalizeUsageReceipts } from './coding-amplification.mjs';
import { aggregateJournalUsage } from './journal-provider-usage.mjs';

export const usageReportProducer = () => {
  const path = fileURLToPath(import.meta.url);
  return { path, sha256: sourceDigest(readFileSync(path)) };
};

/** Consume only the raw capture field physically retained by the maintained runner. */
export function writeJournalUsageReport(journalBinding, indexPath) {
  if (!journalBinding || typeof journalBinding.path !== 'string'
    || !/^[a-f0-9]{64}$/u.test(journalBinding.sha256)) {
    throw new TypeError('exact journal source binding required');
  }
  const information = lstatSync(journalBinding.path);
  if (!information.isFile() || information.size > 8388608) {
    throw new TypeError('bounded regular journal source required');
  }
  const bytes = readFileSync(journalBinding.path);
  const text = bytes.toString('utf8');
  if (sourceDigest(bytes) !== journalBinding.sha256 || !Buffer.from(text).equals(bytes)
    || !text.endsWith('\n')) {
    throw new TypeError('journal source drift or incomplete UTF8 capture');
  }
  const lines = text.slice(0, -1).split('\n');
  if (!lines.length || lines.length > 10000) throw new TypeError('journal record bound');
  const records = lines.map(line => JSON.parse(line));
  const first = records[0];
  if (first?.kind !== 'admitted' || !Array.isArray(first.manifest?.cases)) {
    throw new TypeError('source-owned admission required');
  }
  const owners = new Map(first.manifest.cases.map(item => [item.runId, item]));
  const receipts = [];
  const normalizedReceipts = [];
  const dispositions = [];
  const failures = [];
  const receiptIds = new Set();
  const requests = new Set();
  for (const record of records.filter(value => value.kind === 'finished')) {
    const usage = record.result?.usage;
    if (usage?.status !== 'CapturedUnnormalized') continue;
    const base = { runId: record.runId, attemptId: record.attemptId, sequence: record.sequence,
      rawSHA256: usage.rawSHA256 };
    let raw;
    try {
      raw = usage.raw;
      if (sourceDigest(JSON.stringify(raw)) !== usage.rawSHA256) {
        throw new TypeError('raw driver capture identity mismatch');
      }
      if (raw?.schemaVersion !== 1 || raw.kind !== 'normalized-provider-usage') {
        dispositions.push({ ...base, status: 'Unknown', reason: 'unsupported raw provider schema' });
        continue;
      }
      const owner = owners.get(record.runId);
      if (!owner || raw.runId !== record.runId || raw.attemptId !== record.attemptId) {
        throw new TypeError('raw capture belongs to another case or retry');
      }
      const normalized = normalizeUsageReceipts({ runId: record.runId,
        attemptIds: [record.attemptId], attemptInputs: [owner.task], usageReceipts: [raw] });
      const receipt = normalized.usageReceipts[0];
      const request = JSON.stringify([receipt.provider, receipt.requestId]);
      if (receiptIds.has(receipt.receiptId) || requests.has(request)) {
        throw new TypeError('duplicate source receipt or provider request');
      }
      receiptIds.add(receipt.receiptId);
      requests.add(request);
      receipts.push(raw);
      normalizedReceipts.push(receipt);
      dispositions.push({ ...base, status: 'SourceValidated', receiptId: receipt.receiptId });
    } catch (error) {
      const failure = { ...base, status: 'Rejected', name: error?.name ?? 'Error',
        reason: String(error?.message ?? error) };
      failures.push(failure);
      dispositions.push(failure);
    }
  }
  const groups = new Map();
  for (const receipt of normalizedReceipts) {
    const key = JSON.stringify([receipt.runId, receipt.provider, receipt.model]);
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(receipt);
  }
  for (const group of groups.values()) {
    for (const counter of ['inputTokens', 'outputTokens', 'cachedInputTokens', 'reasoningTokens']) {
      if (group.some(receipt => receipt.counters[counter] === null)) continue;
      const total = group.reduce((sum, receipt) => sum + receipt.counters[counter], 0);
      if (!Number.isSafeInteger(total)) {
        failures.push({ status: 'Rejected', phase: 'provider-model-total',
          runId: group[0].runId, provider: group[0].provider, model: group[0].model, counter,
          reason: 'captured total exceeds safe integer' });
      }
    }
  }
  // An invalid supported capture prevents partial counters being mistaken for valid ingestion.
  let retained = failures.length ? [] : receipts;
  const index = { schemaVersion: 1, kind: 'captured-provider-receipt-index',
    cohortId: first.cohortId, journalSHA256: journalBinding.sha256,
    accountingProducer: usageReportProducer(), receipts: retained,
    captureDispositions: dispositions, normalizationFailures: failures };
  let indexBytes = Buffer.from(JSON.stringify(index) + '\n');
  if (indexBytes.length > 1048576) {
    failures.push({ status: 'Rejected', phase: 'receipt-index-bound',
      reason: 'optional receipt index exceeds retained byte bound' });
    retained = [];
    index.receipts = retained;
    // Full captures remain in the source-bound journal; full dispositions return in the report.
    // An empty bounded index grants no captured counters or coverage.
    delete index.captureDispositions;
    delete index.normalizationFailures;
    index.accountingStatus = 'Rejected';
    index.normalizationFailureCount = failures.length;
    indexBytes = Buffer.from(JSON.stringify(index) + '\n');
  }
  if (indexBytes.length > 1048576) throw new TypeError('bounded empty receipt index required');
  if (sourceDigest(readFileSync(journalBinding.path)) !== journalBinding.sha256) {
    throw new TypeError('journal changed before receipt index retention');
  }
  const fd = openSync(indexPath, 'wx', 0o600);
  try { writeFileSync(fd, indexBytes); fsyncSync(fd); } finally { closeSync(fd); }
  const indexBinding = { path: indexPath, sha256: sourceDigest(indexBytes) };
  const report = aggregateJournalUsage(journalBinding, indexBinding);
  return { ...report, accountingProducer: index.accountingProducer,
    captureDispositions: dispositions, normalizationFailures: failures,
    capturedReceiptIngestion: failures.length ? 'Rejected' : retained.length ? 'SourceCaptured' : 'Unknown',
    ingestionScope: 'existing journaled driver raw field; normalized source captures only',
    rawCapturesObserved: dispositions.length, providerAuthenticityRequiresReview: true,
    fullModelUsageStatus: 'Unknown', observedInputCompleteness: 'Unknown',
    actualCost: null, monetarySavings: null, semanticUsefulness: 'Unknown' };
}
