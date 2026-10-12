import assert from 'node:assert/strict';
import test from 'node:test';
import {
  validateRecordedSigningProcesses,
  validateSigningStreams,
} from '../../../scripts/native-release-evidence.mjs';

// Typed data validates receipt contracts; these fixtures do not attest a signature.
function fixture() {
  const context = {
    source: { source_commit: 'a'.repeat(40), producer_run: '38005341210' },
    sourceSha256: 'b'.repeat(64),
    label: 'macos-x64',
    packageObservation: { name: 'sample.dmg', bytes: 7, sha256: 'c'.repeat(64) },
    mode: 'adhoc',
    observations: {},
  };
  const binding = {
    application: '/receipt-fixture/app.app',
    bundle_sha256: 'd'.repeat(64),
    package: { path: '/receipt-fixture/sample.dmg', bytes: 7, sha256: 'c'.repeat(64) },
    source_selection_sha256: context.sourceSha256,
    source_commit: context.source.source_commit,
    producer_run: context.source.producer_run,
    label: context.label,
  };
  const records = {};
  for (const phase of ['verify', 'display']) {
    const args = phase === 'verify'
      ? ['--verify', '--deep', '--strict', '--verbose=2', binding.application]
      : ['--display', '--verbose=2', binding.application];
    records[phase] = {
      schema: 'formal-ai-signing-process/v1', command: 'codesign', arguments: args,
      before: structuredClone(binding), after: structuredClone(binding),
      status: 0, signal: null, error: null, capture_error: null, complete: true,
      stdout: '\ufeff\u0000observation', stderr: '',
      raw_streams: {
        stdout_base64: Buffer.from('\ufeff\u0000observation').toString('base64'),
        stderr_base64: '',
      },
    };
    context.observations[phase] = { status: 0, stdout: records[phase].stdout, stderr: '' };
  }
  return { records, context };
}

test('canonical raw bytes preserve UTF-8 BOM and NUL in decoded receipt text', () => {
  const { records, context } = fixture();
  validateSigningStreams(records.verify);
  validateRecordedSigningProcesses(records, context);
});

test('invalid UTF-8 in either stream refuses a complete receipt', () => {
  for (const name of ['stdout', 'stderr']) {
    const { records } = fixture();
    records.verify.raw_streams[name + '_base64'] = Buffer.from([255, 0]).toString('base64');
    assert.throws(() => validateSigningStreams(records.verify));
  }
});

test('noncanonical base64 and byte/text disagreement refuse a receipt', () => {
  for (const encoded of ['@@', 'YQ', Buffer.from('different').toString('base64')]) {
    const { records } = fixture();
    records.verify.raw_streams.stdout_base64 = encoded;
    assert.throws(() => validateSigningStreams(records.verify));
  }
});

test('source, producer, package and bundle identity changes refuse durable receipts', () => {
  const mutations = [
    record => { record.before.source_commit = 'e'.repeat(40); },
    record => { record.before.source_selection_sha256 = 'e'.repeat(64); },
    record => { record.before.producer_run = 'different'; },
    record => { record.before.package.sha256 = 'e'.repeat(64); },
    record => { record.after.bundle_sha256 = 'e'.repeat(64); },
  ];
  for (const mutate of mutations) {
    const { records, context } = fixture();
    mutate(records.verify);
    assert.throws(() => validateRecordedSigningProcesses(records, context));
  }
});

test('real-process error fields and changed signing operands refuse durable receipts', () => {
  const mutations = [
    record => { record.status = 7; },
    record => { record.signal = 'SIGTERM'; },
    record => { record.capture_error = 'invalid-utf8-stream'; },
    record => { record.complete = false; },
    record => { record.arguments = ['--display']; },
  ];
  for (const mutate of mutations) {
    const { records, context } = fixture();
    mutate(records.verify);
    assert.throws(() => validateRecordedSigningProcesses(records, context));
  }
});
