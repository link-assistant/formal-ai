// Real Node CLI, socket transport and credential handoff; private scrubbed homes.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import {createHash} from 'node:crypto';
import {test} from 'node:test';
import {encodeTyped} from '../../../js/server/structured-log.mjs';
import {startCli,debug,pause,waitFor} from './support/structured-log-fixture.mjs';

const secrets={FORMAL_AI_API_KEY:'api-private',GH_TOKEN:'gh-private',GITHUB_TOKEN:'github-private'};
const payload={message:'api-private\ngh-private github-private',nested:{password:'field-private'}};
const sanitized={message:'[REDACTED]\n[REDACTED] [REDACTED]',nested:{password:'[REDACTED]'}};

test('CLI defaults emit typed debug tee and preserve the HTTP/stdout protocol',{timeout:15000},async t=>{
  const run=await startCli(t,{args:['--agent-mode'],env:secrets});await run.ready();
  const response=await fetch(run.url+'/no-such-route',{method:'POST',
    headers:{'content-type':'application/json','x-goog-api-key':'goog-private',
      'anthropic-api-key':'anthropic-private',authorization:'Bearer auth-private'},
    body:JSON.stringify(payload),signal:AbortSignal.timeout(2000)});
  assert.equal(response.status,404);await response.text();await run.stop();
  assert.equal(run.code,0);assert.equal(run.stdout,'');
  const text=fs.readFileSync(path.join(run.home,'.formal-ai/logs/serve-'+run.pid+'.lino'),'utf8');
  assert.equal(text,run.stderr);
  for(const secret of [...Object.values(secrets),'field-private','goog-private','anthropic-private','auth-private']){
    assert.equal((text+run.legacy()).includes(secret),false);
  }
  const records=run.records();assert(records.some(x=>x.level==='debug'));
  assert(records.some(x=>x.kind==='server.start'&&x.metadata.agent_mode===true));
  assert(records.some(x=>x.kind==='request.terminal'&&x.metadata.reason==='response_complete'));
  assert(records.some(x=>x.kind==='server.terminal'&&x.metadata.reason==='signal'));
  assert.deepEqual(records.map(x=>x.sequence),records.map((_,i)=>i+1));
  assert(records.every(x=>x.timestamp_precision==='milliseconds'&&/\.\d{3}Z$/u.test(x.timestamp)));
  const observed=records.find(x=>x.kind==='request.received'&&x.metadata.path==='/no-such-route');
  const canonical=encodeTyped(sanitized);
  assert.equal(observed.payload.digest,createHash('sha256').update(canonical).digest('hex'));
  assert.equal(observed.payload.bytes,Buffer.byteLength(canonical));
  assert.equal(Object.hasOwn(observed.payload,'content'),false);
});

test('silent and environment levels reduce actual CLI diagnostics',{timeout:25000},async t=>{
  const silent=await startCli(t,{args:directory=>['--trace','--silent','--log-dir',path.join(directory,'explicit')]});
  await silent.ready();await silent.stop();assert.equal(silent.stderr,'');assert.equal(silent.stdout,'');
  assert.equal(fs.existsSync(path.join(silent.directory,'explicit')),false);
  assert.equal(fs.existsSync(path.join(silent.home,'dialog-logs')),false);
  const envSilent=await startCli(t,{args:['--trace'],env:{FORMAL_AI_SILENT:'yes'}});
  await envSilent.ready();await envSilent.stop();assert.equal(envSilent.stderr,'');
  assert.equal(fs.existsSync(path.join(envSilent.home,'.formal-ai/logs')),false);
  const warn=await startCli(t,{env:{FORMAL_AI_LOG_LEVEL:'warn'}});
  await warn.ready();await warn.stop();assert.equal(warn.stderr,'');assert.equal(warn.stdout,'');
  const invalid=await startCli(t,{env:{FORMAL_AI_LOG_LEVEL:'trace'}});
  await waitFor(()=>invalid.closed);await invalid.stop();assert.equal(invalid.code,2);
  assert.equal(invalid.records()[0].metadata.reason,'invalid_log_configuration');
});

test('trace redacts actual payloads and records aborted HTTP failures',{timeout:15000},async t=>{
  const run=await startCli(t,{args:['--trace'],env:secrets});await run.ready();
  const response=await fetch(run.url+'/no-such-route',{method:'POST',headers:{'content-type':'application/json'},
    body:JSON.stringify(payload),signal:AbortSignal.timeout(2000)});await response.text();
  const socket=net.createConnection({host:'127.0.0.1',port:run.port});
  socket.setTimeout(1000,()=>socket.destroy());
  await new Promise((resolve,reject)=>{socket.once('connect',resolve);socket.once('error',reject);});
  socket.write('POST /aborted-gh-private HTTP/1.1\r\nHost: localhost\r\nContent-Length: 2000\r\n\r\n{}');
  await pause(30);socket.destroy();
  await waitFor(()=>run.records().some(x=>x.kind==='request.failure'));
  await run.stop();assert.equal(run.stdout,'');
  const records=run.records(),received=records.find(x=>x.kind==='request.received'&&x.metadata.path==='/no-such-route');
  assert.deepEqual(received.payload.content,sanitized);
  const failure=records.find(x=>x.kind==='request.failure');
  assert.equal(failure.metadata.error.code,'ECONNRESET');assert.equal(failure.metadata.error.stack_omitted,true);
  assert(records.some(x=>x.kind==='request.terminal'&&x.metadata.request_id===failure.metadata.request_id&&
    ['client_disconnected','request_failed'].includes(x.metadata.reason)));
  for(const secret of Object.values(secrets))assert.equal(run.stderr.includes(secret),false);
});

test('regression: generated CLI credential handoff authenticates without logging tokens',{timeout:15000},async t=>{
  const run=await startCli(t,{args:['--debug-session']});await run.ready();
  const metadata=await waitFor(()=>run.records().find(x=>x.kind==='server.debug_session')?.metadata);
  assert.equal(metadata.generated,true);assert.equal(metadata.credential_source,'private_file');
  const file=metadata.credential_file,directory=path.dirname(file);
  assert(directory.startsWith(path.join(run.home,'.formal-ai/debug-session-')));
  const credential=JSON.parse(fs.readFileSync(file,'utf8'));
  assert.equal(fs.statSync(file).mode&0o777,0o600);assert.equal(fs.statSync(directory).mode&0o777,0o700);
  assert.equal(fs.statSync(file).uid,process.getuid());assert.match(credential.token,/^[a-f0-9]{48}$/u);
  assert.equal(credential.pid,run.pid);assert.equal(credential.url,run.url);
  assert.equal((await debug(run,'wrong')).status,401);
  assert.equal((await debug(run,credential.token)).status,200);
  assert.equal((await debug(run,credential.token,'pause')).body.stepping,true);
  assert.equal((await debug(run,credential.token,'release')).body.stepping,false);
  const response=await fetch(run.url+'/secret-'+credential.token,{method:'POST',
    headers:{'content-type':'application/json'},body:JSON.stringify({message:credential.token}),
    signal:AbortSignal.timeout(2000)});await response.text();await run.stop();
  assert.equal(run.code,0);assert.equal(run.stdout,'');
  assert.equal((run.stderr+run.legacy()).includes(credential.token),false);
  assert.equal(fs.existsSync(file),false);assert.equal(fs.existsSync(directory),false);
});

test('environment debug credentials keep the existing launcher path',{timeout:15000},async t=>{
  const token='private-launcher-debug-token',run=await startCli(t,{args:['--debug-session'],
    env:{FORMAL_AI_DEBUG_SESSION_TOKEN:token}});await run.ready();
  assert.equal((await debug(run,'wrong')).status,401);assert.equal((await debug(run,token)).status,200);
  await run.stop();assert.equal(run.code,0);assert.equal(run.stdout,'');
  const metadata=run.records().find(x=>x.kind==='server.debug_session').metadata;
  assert.equal(metadata.generated,false);assert.equal(metadata.credential_source,'environment');
  assert.equal(Object.hasOwn(metadata,'credential_file'),false);
  assert.equal(fs.readdirSync(path.join(run.home,'.formal-ai')).some(x=>x.startsWith('debug-session-')),false);
  assert.equal((run.stderr+run.legacy()).includes(token),false);
});
