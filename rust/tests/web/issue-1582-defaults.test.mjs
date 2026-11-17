// Regression evidence for #1582; supplied tests, zero autonomous authorship credit.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {test} from 'node:test';
import {createLogger,createServeLogger,decodeTyped} from '../../../js/server/structured-log.mjs';
import {parseLinks,writeLink} from '../../../js/logging/lino-codec.mjs';
import {spawnSync} from 'node:child_process';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {startCli} from './support/structured-log-fixture.mjs';

test('read-only retrieval is default-on and explicit restrictions win',async()=>{
  const {readNetworkAllowed}=await import('../../../js/server/read-access.mjs');
  assert.equal(readNetworkAllowed({}),true);
  for(const value of ['0','false','NO',' off '])assert.equal(readNetworkAllowed({FORMAL_AI_LIVE_FETCH:value}),false);
  for(const value of ['1','true','YES',' on '])assert.equal(readNetworkAllowed({FORMAL_AI_LIVE_FETCH:value}),true);
  assert.equal(readNetworkAllowed({FORMAL_AI_LIVE_FETCH:'invalid'}),false);
  assert.equal(readNetworkAllowed({FORMAL_AI_OFFLINE:'true',FORMAL_AI_LIVE_FETCH:'1'}),false);
  assert.equal(readNetworkAllowed({FORMAL_AI_NETWORK:'denied',FORMAL_AI_LIVE_FETCH:'1'}),false);
});

test('routine diagnostics go to the file without terminal writes',t=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-1582-'));
  t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
  const child=spawnSync(process.execPath,['--input-type=module','-e',
    `import {createServeLogger} from './js/server/structured-log.mjs'; const logger=createServeLogger({env:{HOME:${JSON.stringify(directory)}}}); logger.log('debug','wrapper.started'); logger.close();`],{encoding:'utf8'});
  assert.equal(child.status,0);assert.equal(child.stderr,'');assert.equal(child.stdout,'');
  const writes=[];
  const logger=createServeLogger({env:{HOME:directory},stderrWrite:bytes=>(writes.push(bytes),bytes.length)});
  logger.log('debug','wrapper.started',{component:'client_integrations'});
  logger.log('debug','wrapper.session',{session_file:'/private/session.jsonl'});
  logger.close();
  assert.equal(writes.length,0);
  const lines=fs.readFileSync(logger.file,'utf8').trimEnd().split('\n');
  assert.equal(lines.length,2);
  assert.deepEqual(lines.map(decodeTyped).map(row=>row.kind),['wrapper.started','wrapper.session']);
});

test('terminal and nested logging are opt-in, typed and redacted',t=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-1582-'));
  t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
  const chunks=[];
  const logger=createServeLogger({env:{HOME:directory,GH_TOKEN:'private-value',FORMAL_AI_LOG_STDERR:'1',FORMAL_AI_LOG_FORMAT:'nested'},stderrWrite:bytes=>(chunks.push(Buffer.from(bytes)),bytes.length)});
  logger.log('debug','wrapper.session',{secret:'private-value',details:{path:'session.jsonl'}});
  logger.close();
  const text=Buffer.concat(chunks).toString();
  assert.match(text,/\n  \(/u);
  assert.equal(text,fs.readFileSync(logger.file,'utf8'));
  assert.equal(text.includes('private-value'),false);
  assert.equal(decodeTyped(writeLink(parseLinks(text)[0])).metadata.secret,'[REDACTED]');
  assert.throws(()=>createLogger({format:'invalid'}),/invalid_log_format/u);
});

const questions=[
  {language:'en',prompt:'How do I enable internet access?'},
  {language:'ru',prompt:'Как включить доступ к интернету?'},
  {language:'hi',prompt:'इंटरनेट की अनुमति कैसे दें?'},
  {language:'zh',prompt:'如何启用互联网访问？'},
  {language:'es',prompt:'¿Cómo habilitar el acceso a internet?'},
];
for(const {language,prompt} of questions)test('internet permission help is actionable in '+language,{timeout:20000},async()=>{
  const host=new WorkerHost({WebAssembly:{instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}}});
  const result=await host.solve(prompt,[],{forcedResponseLanguage:language});
  assert.equal(result.intent,'capabilities');
  assert.match(result.content,/\/permissions/u);
  assert.match(result.content,/FORMAL_AI_LIVE_FETCH=1/u);
  assert.match(result.content,/FORMAL_AI_OFFLINE/u);
  assert.match(result.content,/WebFetch/u);
  assert.equal(result.evidence.some(row=>row.includes('permission:guidance')),true);
});

test('serving worker performs real HTTP reads and preserves explicit network denies',{timeout:20000},async t=>{
  const {createServer}=await import('node:http');
  const {createReadOnlyFetch}=await import('../../../js/server/read-access.mjs');
  const requests=[];
  const source=createServer((request,response)=>{requests.push(request.method);response.end('captured-source');});
  await new Promise(resolve=>source.listen(0,'127.0.0.1',resolve));
  t.after(()=>new Promise(resolve=>source.close(resolve)));
  const url='http://127.0.0.1:'+source.address().port+'/source';
  const networkFetch=createReadOnlyFetch({env:{}});
  const host=new WorkerHost({}, {networkFetch});
  await host.boot();
  const text=await host.run('fetch(__url).then(response=>response.text())',{__url:url});
  assert.equal(text,'captured-source');
  await assert.rejects(()=>networkFetch(url,{method:'POST'}),/read_network_method/u);
  await assert.rejects(()=>createReadOnlyFetch({env:{FORMAL_AI_NETWORK:'denied'}})(url),/read_network_denied/u);
  await assert.rejects(()=>createReadOnlyFetch({env:{FORMAL_AI_OFFLINE:'1'}})(url),/read_network_denied/u);
  assert.deepEqual(requests,['GET']);
});

test('served model metadata advertises the resource-derived context consistently',{timeout:15000},async t=>{
  const run=await startCli(t);
  await run.ready();
  const response=await fetch(run.url+'/v1/models',{signal:AbortSignal.timeout(2000)});
  assert.equal(response.status,200);
  const body=await response.json(),model=body.models.find(model=>model.id==='formal-ai');
  assert(model);
  assert(Number.isSafeInteger(model.context_window_tokens));
  assert(model.context_window_tokens>0);
  assert.equal(model.context_window_tokens,Math.floor(model.disk_free_bytes/model.avg_utf8_bytes_per_char));
  assert.equal(model.context_window,model.context_window_tokens);
  assert.equal(body.data.find(model=>model.id==='formal-ai').context_window_tokens,model.context_window_tokens);
  assert(Number.isSafeInteger(model.max_output_tokens));
  assert(model.max_output_tokens>0);
  await run.stop();
  assert.equal(run.stdout,'');assert.equal(run.stderr,'');
});
