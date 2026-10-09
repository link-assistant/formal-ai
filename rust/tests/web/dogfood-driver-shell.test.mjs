// Real shell transport regressions: dropped pipe bytes, output limits and failure receipts.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync, rmSync, statSync} from 'node:fs';
import {dirname} from 'node:path';
import {pathToFileURL} from 'node:url';
import {shellCapture} from '../../../experiments/js_dogfood/shell-capture.mjs';
import {execute} from '../../../experiments/js_dogfood/drive.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const {observedBytesMatch, reportedExitCode, stepOutcome, StepOutcome} = await import('../../../js/agentic/tool_result.mjs');
const quote = (text) => "'" + text.replaceAll("'", "'\\''") + "'";
const node = (source, replaceShell = false) => (replaceShell ? 'exec ' : '') + quote(process.execPath) + ' -e ' + quote(source);
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');
const cleanup = (record) => rmSync(dirname(record.stdout_capture.path), {recursive: true, force: true});

test('abrupt failing child preserves every diagnostic byte through the actual driver', () => {
  const expected = 'BEGIN_DIAGNOSTIC\n' + 'x'.repeat(131072) + '\nEND_DIAGNOSTIC\n';
  const result = execute(process.cwd(), {tool:'bash', arguments:JSON.stringify({command:node('process.stderr.write('+JSON.stringify(expected)+');process.exit(7);')})});
  assert.equal(result, 'Output: \nError: '+expected+'\nExit Code: 7');
  assert.equal(reportedExitCode(result),7);
  assert.equal(stepOutcome(result),StepOutcome.Failed);
});

test('successful Unicode, CRLF and trailing whitespace remain exact stdout evidence', () => {
  const expected = 'Привет 中文\r\nlast line  \n\n';
  const result = shellCapture(node('process.stdout.write('+JSON.stringify(expected)+');process.exit(0);'));
  assert.equal(result,'Output: '+expected+'\nExit Code: 0');
  assert.equal(observedBytesMatch(result,expected),true);
  assert.equal(observedBytesMatch(result,expected.trim()),false);
});

test('oversized stdout is retained without pipe-buffer limits or fabricated byte proof', () => {
  const expected = Buffer.from('BEGIN\n'+'z'.repeat(2*1024*1024)+'\nEND\n');
  const record = JSON.parse(shellCapture(node("process.stdout.write('BEGIN\\n'+'z'.repeat(2*1024*1024)+'\\nEND\\n');process.exit(0);"),{maxOutputBytes:4096}));
  try {
    const raw = readFileSync(record.stdout_capture.path);
    assert.deepEqual(raw,expected);assert.equal(record.stdout_capture.bytes,expected.length);
    assert.equal(record.stdout_capture.sha256,digest(expected));assert.equal(record.exit_code,0);
    assert.equal(record.command_output_complete,true);assert.equal(record.is_error,true);
    assert.ok(record.stdout_capture.preview.length<4300);assert.match(record.stdout_capture.preview,/END\n$/u);
    assert.equal(observedBytesMatch(JSON.stringify(record),record.stdout_capture.preview),false);
    assert.equal(statSync(record.stdout_capture.path).mode & 0o777,0o600);
  } finally { cleanup(record); }
});

test('oversized nonzero diagnostics retain failure and digest independently of preview', () => {
  const expected = Buffer.from('error '+ 'é'.repeat(10000));
  const record=JSON.parse(shellCapture(node('process.stderr.write('+JSON.stringify(expected.toString())+');process.exit(19);'),{maxOutputBytes:1024}));
  try {
    assert.equal(record.exit_code,19);assert.equal(record.stdout_capture.bytes,0);
    assert.deepEqual(readFileSync(record.stderr_capture.path),expected);
    assert.equal(record.stderr_capture.sha256,digest(expected));assert.equal(record.is_error,true);
  }finally{cleanup(record);}
});

test('timeout preserves owned-child diagnostics and never supplies a success exit code', () => {
  const record=JSON.parse(shellCapture(node("process.stderr.write('before timeout');setInterval(()=>{},1000);",true),{timeoutMs:350}));
  try {
    assert.equal(record.exit_code,null);assert.equal(record.timeout_ms,350);
    assert.equal(record.command_output_complete,false);assert.equal(record.signal,'SIGKILL');
    assert.match(readFileSync(record.stderr_capture.path,'utf8'),/before timeout/u);
    assert.equal(observedBytesMatch(JSON.stringify(record),''),false);
  }finally{cleanup(record);}
});

test('a terminated shell records its signal independently of timeout and exit code', () => {
  const result=shellCapture('kill -TERM $$');
  assert.match(result,/\nSignal: SIGTERM$/u);assert.doesNotMatch(result,/Exit Code|Timeout/u);
  assert.equal(observedBytesMatch(result,''),false);
});

test('invalid UTF-8 stays in raw capture and cannot become replacement-character evidence', () => {
  const record=JSON.parse(shellCapture(node('process.stdout.write(Buffer.from([255,0,13,10]));process.exit(0);')));
  try {
    assert.deepEqual(readFileSync(record.stdout_capture.path),Buffer.from([255,0,13,10]));
    assert.equal(record.is_error,true);assert.equal(record.exit_code,0);
    assert.equal(observedBytesMatch(JSON.stringify(record),record.stdout_capture.preview),false);
  }finally{cleanup(record);}
});

test('stderr on a successful command is observed and cannot be hidden in stdout proof', () => {
  const result=shellCapture(node("process.stdout.write('body');process.stderr.write('warning');"));
  assert.equal(result,'Output: body\nError: warning\nExit Code: 0');
  assert.equal(observedBytesMatch(result,'body'),false);
});

test('invalid transport limits refuse before launching side effects', () => {
  for(const options of [{timeoutMs:0},{timeoutMs:NaN},{maxOutputBytes:1},{maxOutputBytes:Infinity}]) {
    assert.throws(()=>shellCapture('exit 0',options),RangeError);
  }
  assert.throws(()=>shellCapture(null),TypeError);
});
