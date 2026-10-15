import assert from 'node:assert/strict';
import test from 'node:test';
import childProcess from 'node:child_process';
import { syncBuiltinESMExports } from 'node:module';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {
  validateRecordedSigningProcesses,
  validateSigningStreams,
  signingBundleDigest,
  signingProcessBinding,
  observePackagedExecutable,
  captureSigningProcess,
  validateSigningProcess,
} from '../../../scripts/native-release-evidence.mjs';

// These fake observations exercise deadlines; they do not attest bundle bytes or signatures.
test('bundle observation uses fresh remaining job time beyond the former 30 second cap', () => {
  const calls = [];
  const value = signingBundleDigest('/fixture/app.app', {
    deadlineEpoch: 1000, now: () => 900000,
    runCommand: (...args) => { calls.push(args); return { status: 0, signal: null, stdout: '[]', stderr: '' }; },
  });
  assert.match(value, /^[a-f0-9]{64}$/u);
  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], 'python3');
  assert.equal(calls[0][2].timeout, 100000);
  assert.equal(calls[0][2].maxBuffer, 16 * 1024 * 1024);
});

test('expired, absent, fractional and overflowing deadlines refuse before executing a walker', () => {
  for (const deadlineEpoch of [undefined, NaN, 0, -1, 900, 899, 1000.5, Number.MAX_SAFE_INTEGER]) {
    let calls = 0;
    assert.throws(() => signingBundleDigest('/fixture/app.app', {
      deadlineEpoch, now: () => 900000,
      runCommand: () => { calls++; throw Error('must not execute'); },
    }));
    assert.equal(calls, 0);
  }
});

test('walker timeout, signal, failure, malformed output and deadline overrun remain fatal', () => {
  const timeout = Object.assign(Error('timeout fixture'), { code: 'ETIMEDOUT' });
  for (const result of [
    { error: timeout },
    { signal: 'SIGTERM', status: null, stdout: '[]' },
    { signal: null, status: 1, stderr: 'walker refused', stdout: '[]' },
    { signal: null, status: 0, stdout: 'not JSON' },
    { signal: null, status: 0, stdout: '{}' },
  ]) assert.throws(() => signingBundleDigest('/fixture/app.app', {
    deadlineEpoch: 1000, now: () => 900000, runCommand: () => result,
  }));
  let time = 900000;
  assert.throws(() => signingBundleDigest('/fixture/app.app', {
    deadlineEpoch: 1000, now: () => time,
    runCommand: () => { time = 1000000; return { status: 0, signal: null, stdout: '[]' }; },
  }), /expired or unsupported job deadline/);
});

test('binding preserves package bytes and forwards the shrinking deadline to the full walker', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-signing-deadline-'));
  try {
    const file = path.join(directory, 'sample.dmg'); fs.writeFileSync(file, 'small package fixture');
    let time = 905000; const timeouts = [];
    const options = { deadlineEpoch: 1000, now: () => time, runCommand: (...args) => {
      timeouts.push(args[2].timeout); return { status: 0, signal: null, stdout: '[]' };
    } };
    const context = { source: { source_commit: 'a'.repeat(40), producer_run: 'fixture' }, sourceSha256: 'b'.repeat(64), label: 'macos-x64' };
    const first = signingProcessBinding('/fixture/app.app', file, context, options);
    time = 940000;
    const second = signingProcessBinding('/fixture/app.app', file, context, options);
    assert.deepEqual(timeouts, [95000, 60000]);
    assert.deepEqual(first, second);
    assert.equal(first.package.bytes, Buffer.byteLength('small package fixture'));
    fs.writeFileSync(file, 'changed package fixture');
    assert.notEqual(signingProcessBinding('/fixture/app.app', file, context, options).package.sha256, first.package.sha256);
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});

test('standalone observation retains 30 seconds only for genuinely absent deadlines', () => {
  const original = process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
  try {
    delete process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
    let timeout;
    const runCommand = (command, args, options) => { timeout = options.timeout; return { status: 0, signal: null, stdout: '[]' }; };
    signingBundleDigest('/fixture/app.app', { runCommand });
    assert.equal(timeout, 30000);
    for (const value of ['', ' ', 'NaN', 'Infinity', '1e3', '0x1000', '-1', '1000.5', '0', '9007199254740991']) {
      process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = value;
      assert.throws(() => signingBundleDigest('/fixture/app.app', { runCommand, now: () => 900000 }));
    }
    delete process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
    for (const deadlineEpoch of [undefined, null, NaN, 0, -1, 1000.5, Number.MAX_SAFE_INTEGER]) {
      assert.throws(() => signingBundleDigest('/fixture/app.app', { deadlineEpoch, runCommand, now: () => 900000 }));
    }
    process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = '1000';
    signingBundleDigest('/fixture/app.app', { runCommand, now: () => 900000 });
    assert.equal(timeout, 100000);
  } finally {
    if (original === undefined) delete process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
    else process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = original;
  }
});

test('packaged executable startup keeps its own cap and refuses deadline overruns', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-startup-deadline-'));
  try {
    const file = path.join(directory, 'formal-ai'); fs.writeFileSync(file, 'small executable data fixture');
    for (const [time, expected] of [[900000, 30000], [990000, 10000]]) {
      let timeout;
      observePackagedExecutable(file, (command, args, options) => {
        timeout = options.timeout;
        return { status: 0, signal: null, stdout: 'formal-ai fixture', stderr: '' };
      }, { deadlineEpoch: 1000, now: () => time });
      assert.equal(timeout, expected);
    }
    let time = 990000;
    assert.throws(() => observePackagedExecutable(file, () => {
      time = 1000000;
      return { status: 0, signal: null, stdout: 'formal-ai fixture', stderr: '' };
    }, { deadlineEpoch: 1000, now: () => time }), /expired/);
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});

test('actual capture and readback exports share one shrinking deadline and preserve standalone behavior', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-signing-pipeline-'));
  const originalSpawn = childProcess.spawnSync;
  const originalDeadline = process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
  let time = 900000, mode = 'normal', calls = [];
  const now = () => time;
  try {
    const packageFile = path.join(directory, 'sample.dmg');
    fs.writeFileSync(packageFile, 'small package fixture');
    const application = path.join(directory, 'sample.app');
    const context = {
      source: { source_commit: 'a'.repeat(40), producer_run: 'fixture' },
      sourceSha256: 'b'.repeat(64), label: 'macos-x64',
    };
    // Rebind the built-in ESM export used by the actual observation module.
    // No Python, browser, signing tool, or native executable runs in this test.
    childProcess.spawnSync = (command, args, options) => {
      assert.equal(command, 'python3');
      calls.push({ command, timeout: options.timeout });
      time = mode === 'expire-before-command' ? 1000000 : time + 3000;
      // A later environment value must not restart the epoch passed at entry.
      process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = '2000';
      return { status: 0, signal: null, stdout: '[]', stderr: '' };
    };
    syncBuiltinESMExports();
    const runCommand = (command, args, options) => {
      assert.equal(command, 'codesign');
      calls.push({ command, timeout: options.timeout });
      time = mode === 'expire-after-command' ? 1000000 : time + 3000;
      return { status: 0, signal: null, stdout: Buffer.from('fixture observation'), stderr: Buffer.from('') };
    };
    const args = ['--verify'];
    process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = '1000';
    const record = captureSigningProcess(application, packageFile, context, 'codesign', args, {
      deadlineEpoch: 1000, now, runCommand,
    });
    assert.equal(record.complete, true);
    validateSigningProcess(record, application, packageFile, context, 'codesign', args, { deadlineEpoch: 1000, now });
    assert.deepEqual(calls, [
      { command: 'python3', timeout: 100000 },
      { command: 'codesign', timeout: 97000 },
      { command: 'python3', timeout: 94000 },
      { command: 'python3', timeout: 91000 },
    ]);
    for (const failure of ['expired-initial', 'expire-before-command', 'expire-after-command']) {
      mode = failure; time = failure === 'expired-initial' ? 1000000 : 900000; calls = [];
      assert.throws(() => captureSigningProcess(application, packageFile, context, 'codesign', args, {
        deadlineEpoch: 1000, now, runCommand,
      }), /expired/);
      if (failure === 'expired-initial') assert.equal(calls.length, 0);
      if (failure === 'expire-before-command') assert.ok(!calls.some(call => call.command === 'codesign'));
      if (failure === 'expire-after-command') assert.equal(calls.filter(call => call.command === 'python3').length, 1);
    }
    mode = 'normal'; time = 1000000; calls = [];
    assert.throws(() => validateSigningProcess(record, application, packageFile, context, 'codesign', args, {
      deadlineEpoch: 1000, now,
    }), /expired/);
    assert.equal(calls.length, 0);
    time = 900000; calls = []; delete process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
    validateSigningProcess(record, application, packageFile, context, 'codesign', args, { now });
    assert.deepEqual(calls, [{ command: 'python3', timeout: 30000 }]);
    delete process.env.FORMAL_AI_JOB_DEADLINE_EPOCH; calls = [];
    assert.throws(() => validateSigningProcess(record, application, packageFile, context, 'codesign', args, {
      deadlineEpoch: undefined, now,
    }));
    assert.equal(calls.length, 0);
  } finally {
    childProcess.spawnSync = originalSpawn;
    syncBuiltinESMExports();
    if (originalDeadline === undefined) delete process.env.FORMAL_AI_JOB_DEADLINE_EPOCH;
    else process.env.FORMAL_AI_JOB_DEADLINE_EPOCH = originalDeadline;
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

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
