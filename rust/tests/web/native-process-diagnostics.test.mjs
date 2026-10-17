import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { failedNativeProcess, reportNativeProcessFailure } from '../../../scripts/lib/native-process-diagnostics.mjs';
const processResult = (change = {}) => ({
  status: 1,
  signal: null,
  error: null,
  stdout: Buffer.alloc(0),
  stderr: Buffer.alloc(0),
  ...change
});
function capture(stage, result, options = {}) {
  const chunks = [];
  const emitted = reportNativeProcessFailure(stage, result, {
    ...options,
    write: bytes => {
      assert.ok(bytes.length <= 65536);
      chunks.push(Buffer.from(bytes));
    }
  });
  return {
    emitted,
    bytes: Buffer.concat(chunks),
    text: Buffer.concat(chunks).toString('utf8')
  };
}
test('actual Cargo JSON shape emits rendered compiler diagnostic plus complete stderr and other rows', () => {
  const rendered = 'error[E0277]: observed trait bound\n',
    other = JSON.stringify({
      reason: 'build-finished',
      success: false
    }) + '\n';
  const stdout = Buffer.from(JSON.stringify({
      reason: 'compiler-message',
      message: {
        rendered
      }
    }) + '\n' + other),
    stderr = Buffer.from('error: could not compile\n'),
    before = Buffer.from(stdout);
  const observed = capture('build', processResult({
    stdout,
    stderr
  }), {
    cargoJson: true
  });
  assert.ok(observed.text.includes(rendered));
  assert.ok(observed.text.includes(other));
  assert.ok(observed.text.includes(stderr.toString()));
  assert.deepEqual(stdout, before);
  assert.equal(observed.emitted, true);
});
test('malformed and oversized JSON rows stream complete captured bytes in bounded chunks', () => {
  for (const stdout of [Buffer.from('not-json\npartial'), Buffer.alloc(2 * 1024 * 1024, 97)]) {
    const observed = capture('build', processResult({
      stdout
    }), {
      cargoJson: true
    });
    assert.ok(observed.bytes.includes(stdout));
  }
});
test('producer stdout/stderr remain exact including non-UTF8 and large payloads', () => {
  const stdout = Buffer.alloc(180000, 255),
    stderr = Buffer.from([0, 254, 10]);
  const observed = capture('producer', processResult({
    stdout,
    stderr
  }));
  assert.ok(observed.bytes.includes(stdout));
  assert.ok(observed.bytes.includes(stderr));
});
test('timeout, signal and capture overflow are reported without dumping environment or error messages', () => {
  for (const change of [{
    status: null,
    signal: 'SIGTERM',
    error: {
      code: 'ETIMEDOUT',
      message: 'PRIVATE_SENTINEL'
    }
  }, {
    status: null,
    error: {
      code: 'ENOBUFS',
      message: 'PRIVATE_SENTINEL'
    }
  }]) {
    const observed = capture('build', processResult(change));
    assert.ok(observed.text.includes(change.error.code));
    assert.ok(!observed.text.includes('PRIVATE_SENTINEL'));
  }
});
test('successful process is silent and failure metadata stays strict', () => {
  const success = processResult({
    status: 0
  });
  assert.equal(failedNativeProcess(success), false);
  assert.equal(capture('build', success).text, '');
  assert.equal(failedNativeProcess(processResult({
    status: null
  })), true);
  assert.throws(() => capture('foreign', processResult()), /stage/);
});
test('GitHub command parsing is disabled around untrusted diagnostic bytes then restored', () => {
  const payload = '::error title=untrusted::payload\n',
    observed = capture('producer', processResult({
      stdout: Buffer.from(payload)
    })).text;
  const match = /^::stop-commands::([0-9a-f-]+)\n/u.exec(observed);
  assert.ok(match);
  assert.ok(observed.includes(payload));
  assert.ok(observed.endsWith('::' + match[1] + '::\n'));
});
test('maintained producer persists failed receipts and reports before unchanged fatal checks; upload includes failures', () => {
  const source = fs.readFileSync(new URL('../../../scripts/generate-native-session-projections.mjs', import.meta.url), 'utf8'),
    workflow = fs.readFileSync(new URL('../../../.github/workflows/regenerate-native-session-projections.yml', import.meta.url), 'utf8');
  for (const [prefix, stage] of [['build', 'build'], ['run', 'producer']]) {
    assert.ok(source.indexOf("writeFileSync(join(output,'" + stage + ".stderr.bin')") < source.indexOf("reportNativeProcessFailure('" + stage + "'"));
    assert.ok(source.indexOf("reportNativeProcessFailure('" + stage + "'") < source.indexOf('assert.equal(' + prefix + '.receipt.error,null)'));
    assert.ok(source.includes(stage + '-failure-receipt.json'));
  }
  assert.ok(source.includes("assert.equal(process.env.GITHUB_ACTIONS,'true'"));
  assert.ok(source.includes('const deadline=Date.now()+1200000;'));
  assert.ok(source.includes('maxBuffer:64*1024*1024'));
  assert.match(workflow, /actions\/upload-artifact@v7\n        if: always\(\)/u);
});
import { spawnSync } from 'node:child_process';
test('default stderr transport drains complete captured diagnostics through a real Node child', () => {
  const helper = new URL('../../../scripts/lib/native-process-diagnostics.mjs', import.meta.url).href;
  const program = 'import {reportNativeProcessFailure} from ' + JSON.stringify(helper) + ';reportNativeProcessFailure("producer",{status:1,signal:null,error:null,stdout:Buffer.alloc(180000,120),stderr:Buffer.from("ACTUAL_NODE_STDERR_END")});';
  const child = spawnSync(process.execPath, ['--input-type=module', '-e', program], {
    encoding: 'buffer',
    timeout: 5000,
    maxBuffer: 1024 * 1024
  });
  assert.equal(child.status, 0);
  assert.equal(child.stdout.length, 0);
  assert.ok(child.stderr.includes(Buffer.alloc(180000, 120)));
  assert.ok(child.stderr.includes(Buffer.from('ACTUAL_NODE_STDERR_END')));
});
