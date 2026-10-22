import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { executeBoundedSourceCommand }
  from '../../../scripts/check-native-source-operations.mjs';

const modulePath = fileURLToPath(new URL('../../../js/server/execution-box.mjs', import.meta.url));
async function observation(t, child, input, blockedProbe = false) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'execution-deadline-'));
  const marker = path.join(directory, 'child-pid');
  const probeMarker = path.join(directory, 'probe-started');
  t.after(() => {
    try {
      const pid = Number(fs.readFileSync(marker, 'utf8'));
      try { process.kill(-pid, 'SIGKILL'); } catch { process.kill(pid, 'SIGKILL'); }
    } catch { /* The observed process may already have been reaped. */ }
    fs.rmSync(directory, { recursive: true, force: true });
  });
  if (blockedProbe) {
    fs.writeFileSync(path.join(directory, 'python3'),
      `#!${process.execPath}\nrequire('node:fs').writeFileSync(${JSON.stringify(probeMarker)},'started');setInterval(() => {}, 1000);\n`, { mode: 0o700 });
  }
  const childCode = `require('node:fs').writeFileSync(${JSON.stringify(marker)},String(process.pid));` + child;
  const driver = `import {ExecutionBox} from ${JSON.stringify(modulePath)};` +
    `const box=new ExecutionBox({kind:'host_sandbox'},'denied',25,${JSON.stringify(directory)});` +
    `const result=await box.observeInvocation({program:process.execPath,arguments:['-e',${JSON.stringify(childCode)}]},` +
    `${input ? 'Buffer.alloc(8*1024*1024)' : 'null'});` +
    `console.log(JSON.stringify({...result,elapsedNs:String(result.elapsedNs)}));`;
  const guarded = `(async()=>{${driver.replace(/^import [^;]+;/u, '')}})()` +
    `.catch(error=>{const [failure,...captured]=error.fields.detail.split('\\n');` +
    `console.log(JSON.stringify({failure,capturedBytes:Buffer.byteLength(captured.join('\\n'))}));});`;
  const source = `import {ExecutionBox} from ${JSON.stringify(modulePath)};` + guarded;
  const result = await executeBoundedSourceCommand(process.execPath,
    ['--input-type=module', '-e', source], {
      env: blockedProbe ? { ...process.env, PATH: directory + path.delimiter + process.env.PATH } : process.env,
    }, 2000);
  if (blockedProbe) assert.equal(fs.readFileSync(probeMarker, 'utf8'), 'started');
  return JSON.parse(result.output.trim());
}

test('execution deadline includes a child that never consumes oversized stdin', async t => {
  const result = await observation(t,
    "process.stdout.write('stdin-started\\n');setInterval(()=>{},1000)", true);
  assert.equal(result.timed_out, true);
  assert.equal(result.exit_code, null);
  assert.match(result.partial_output, /stdin-started/u);
  assert.equal(result.deadlineMs, 25);
});

test('execution deadline includes descendant-held pipes after the parent exits', async t => {
  const result = await observation(t,
    "require('node:child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)']," +
    "{stdio:['ignore',1,2]});process.stdout.write('held-output\\n');process.exit(0)", false);
  assert.equal(result.timed_out, true);
  assert.equal(result.exit_code, null);
  assert.match(result.partial_output, /held-output/u);
});


test('execution output overflow is an explicit bounded failure carrying partial output', async t => {
  const result = await observation(t,
    "process.stdout.write('x'.repeat(2*1024*1024));setInterval(()=>{},1000)", false);
  assert.equal(result.failure, 'execution_output_limit_exceeded');
  assert.equal(result.capturedBytes, 1024 * 1024);
});

test('execution startup calibration cannot hang before the invocation timer', async t => {
  const result = await observation(t, "setInterval(()=>{},1000)", false, true);
  assert.equal(result.timed_out, true);
  assert.equal(result.exit_code, null);
});
