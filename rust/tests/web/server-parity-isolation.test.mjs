import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {pathToFileURL, fileURLToPath} from 'node:url';
import {normalize, compareResponses, createParityEnvironment, loadCorpus}
  from '../../../scripts/check-server-parity.mjs';

const roots={rust:path.join(os.tmpdir(),'formal-ai-parity-owned-rust','cache'),js:path.join(os.tmpdir(),'formal-ai-parity-owned-js','cache')};
const suffix='derivations/answer_0123456789abcdef/'+'a'.repeat(64)+'.lino';
const response=(value,type='application/json')=>({statusLine:'HTTP/1.1 200 OK',headers:{'content-type':type},body:JSON.stringify(value)});
const value=root=>({evidence_links:[path.join(root,...suffix.split('/'))],answer:'unchanged'});

test('only explicitly owned cache prefixes normalize, with the entire record identity retained',()=>{
 assert.deepEqual(normalize(value(roots.rust),'',roots.rust),normalize(value(roots.js),'',roots.js));
 assert.equal(normalize(value(roots.rust),'',roots.rust).evidence_links[0],'<runtime-cache>/'+suffix);
 assert.equal(compareResponses(response(value(roots.rust)),response(value(roots.js)),'GET',roots).kind,'same');
 assert.equal(compareResponses(response(value(roots.rust)),response(value(roots.js))).kind,'content');
 for(const change of [suffix.replace('a'.repeat(64),'b'.repeat(64)),suffix.replace('0123456789abcdef','1123456789abcdef'),suffix+'/extra']){
  const other={evidence_links:[path.join(roots.js,...change.split('/'))],answer:'unchanged'};
  assert.equal(compareResponses(response(value(roots.rust)),response(other),'GET',roots).kind,'content');
 }
 assert.equal(normalize(path.join(roots.rust,...suffix.split('/')),'answer',roots.rust),path.join(roots.rust,...suffix.split('/')));
 assert.equal(normalize('/foreign/'+suffix,'evidence_links',roots.rust),'/foreign/'+suffix);
 const wrong={...value(roots.js),answer:'different'};
 assert.equal(compareResponses(response(value(roots.rust)),response(wrong),'GET',roots).kind,'content');
});

test('health compares sink state and normalizes only the exact owned top-level log directory',()=>{
 const logRoots={rust:'/owned/rust/logs',js:'/owned/js/logs'};
 const health=directory=>({status:'ok',logs:{directory,file_sink:'active',stderr:true}});
 const left=response(health(logRoots.rust));
 const compare=body=>compareResponses(left,response(body),'GET',{},logRoots);
 assert.equal(compare(health(logRoots.js)).kind,'same');
 assert.equal(compareResponses(left,response(health(logRoots.js))).kind,'content');
 for(const directory of ['/foreign/logs',logRoots.js+'/extra',logRoots.js+'-other',logRoots.rust]){
  assert.equal(compare(health(directory)).kind,'content');
 }
 for(const logs of [{...health(logRoots.js).logs,file_sink:'failed'},{...health(logRoots.js).logs,stderr:false},{...health(logRoots.js).logs,failures:1}]){
  assert.notEqual(compare({status:'ok',logs}).kind,'same');
 }
 const payload={answer:logRoots.js,directory:logRoots.js,nested:{logs:{directory:logRoots.js}},list:[{logs:{directory:logRoots.js}}]};
 assert.deepEqual(normalize(payload,'',null,logRoots.js),payload);
});

test('SSE applies the same exact-root rule and retains protocol and payload differences',()=>{
 const stream=v=>({...response(v,'text/event-stream'),body:'event: response.completed\ndata: '+JSON.stringify(v)+'\n\n'});
 assert.equal(compareResponses(stream(value(roots.rust)),stream(value(roots.js)),'GET',roots).kind,'same');
 assert.equal(compareResponses(stream(value(roots.rust)),stream({...value(roots.js),answer:'different'}),'GET',roots).kind,'content');
 assert.equal(compareResponses(stream(value(roots.rust)),{...stream(value(roots.js)),statusLine:'HTTP/1.1 404 Not Found'},'GET',roots).kind,'protocol');
});

test('the runner isolates runtime state and enables its explicit conversation-log oracle',()=>{
 const env=createParityEnvironment('/private/parity-home','fixture-token',{FORMAL_AI_SILENT:'true',FORMAL_AI_CACHE_DIR:'/foreign',FORMAL_AI_AGENT_MODE:'1',PATH:'kept'});
 assert.equal(env.FORMAL_AI_CACHE_DIR,path.join('/private/parity-home','runtime-cache'));
 assert.equal(env.FORMAL_AI_DIALOG_LOG_DIR,path.join('/private/parity-home','dialogs'));
 assert.equal(env.FORMAL_AI_LOG_DIR,path.join('/private/parity-home','logs'));
 assert.equal(env.FORMAL_AI_LOG_LEVEL,'error');assert.equal(env.FORMAL_AI_SILENT,undefined);assert.equal(env.FORMAL_AI_AGENT_MODE,undefined);assert.equal(env.PATH,'kept');
});

test('the original recorded-conversation corpus remains usable with quiet diagnostics', {timeout:30000}, async()=>{
 const root=process.env.FORMAL_AI_REVIEW_ROOT??path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
 const {startServer}=await import(pathToFileURL(root+'/js/server/main.mjs'));
 const home=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-parity-isolation-'));
 const corpus=loadCorpus(fs.readFileSync(root+'/rust/tests/fixtures/server-parity/requests.lino','utf8'));
 const env=createParityEnvironment(home,corpus.token,{...process.env,PATH:'',FORMAL_AI_SILENT:'true',FORMAL_AI_LIVE_FETCH:'0'});
 let running;
 try{
  running=await startServer({env,host:'127.0.0.1',port:0});
  for(const identifier of ['chat_learning_trace','conversation_learn_recorded']){
   const request=corpus.requests.find(row=>row.id===identifier);assert(request);
   const headers={'authorization':'Bearer '+corpus.token,'content-type':'application/json'};
   for(const row of request.headers){const at=row.indexOf(':');headers[row.slice(0,at)]=row.slice(at+1).trim();}
   const actual=await fetch(running.url+request.path,{method:request.method,headers,...(request.body===null?{}:{body:request.body}),signal:AbortSignal.timeout(20000)});
   assert.equal(actual.status,200,identifier);await actual.text();
  }
  const conversationFile=path.join(home,'dialogs','server-parity-learning.conversation.lino');
  assert(fs.existsSync(conversationFile));
  assert.equal(fs.existsSync(path.join(home,'dialogs','server-parity-learning.conversation.jsonl')),false);
  const {decodeDialogRecord}=await import(pathToFileURL(root+'/js/server/dialog-codec.mjs'));
  const {toCompactJson}=await import(pathToFileURL(root+'/js/server/json.mjs'));
  const {loadConversationContext}=await import(pathToFileURL(root+'/js/server/conversations.mjs'));
  const lines=fs.readFileSync(conversationFile,'utf8');assert(lines.endsWith('\n'));
  const records=lines.slice(0,-1).split('\n').map(decodeDialogRecord);assert(records.length>0);
  const typed=loadConversationContext('server-parity-learning',env);
  assert(typed.messages.length>0);assert.equal(typed.metadata.messages_source,'conversation-record');
  const legacyDirectory=path.join(home,'legacy-dialogs');fs.mkdirSync(legacyDirectory);
  fs.writeFileSync(path.join(legacyDirectory,'server-parity-learning.conversation.jsonl'),records.map(toCompactJson).join('\n')+'\n');
  const legacy=loadConversationContext('server-parity-learning',{...env,FORMAL_AI_DIALOG_LOG_DIR:legacyDirectory});
  assert.deepEqual(legacy.messages,typed.messages);
  assert.equal(legacy.metadata.message_count,typed.metadata.message_count);
  assert.equal(legacy.metadata.messages_source,'conversation-record');
 }finally{if(running)await new Promise(resolve=>running.server.close(resolve));fs.rmSync(home,{recursive:true,force:true});}
});

test('the previous silent runner setup reproduces the recorded-conversation refusal', {timeout:30000}, async()=>{
 const root=process.env.FORMAL_AI_REVIEW_ROOT??path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
 const {startServer}=await import(pathToFileURL(root+'/js/server/main.mjs'));
 const home=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-parity-isolation-'));
 const corpus=loadCorpus(fs.readFileSync(root+'/rust/tests/fixtures/server-parity/requests.lino','utf8'));
 const env=createParityEnvironment(home,corpus.token,{...process.env,PATH:'',FORMAL_AI_SILENT:'true',FORMAL_AI_LIVE_FETCH:'0'});
 env.FORMAL_AI_SILENT='true';
 let running;
 try{
  running=await startServer({env,host:'127.0.0.1',port:0});
  for(const identifier of ['chat_learning_trace','conversation_learn_recorded']){
   const request=corpus.requests.find(row=>row.id===identifier);assert(request);
   const headers={'authorization':'Bearer '+corpus.token,'content-type':'application/json'};
   for(const row of request.headers){const at=row.indexOf(':');headers[row.slice(0,at)]=row.slice(at+1).trim();}
   const actual=await fetch(running.url+request.path,{method:request.method,headers,...(request.body===null?{}:{body:request.body}),signal:AbortSignal.timeout(20000)});
   assert.equal(actual.status,identifier==='conversation_learn_recorded'?404:200,identifier);await actual.text();
  }
  assert.equal(fs.existsSync(path.join(home,'dialogs','server-parity-learning.conversation.lino')),false);
  assert.equal(fs.existsSync(path.join(home,'dialogs','server-parity-learning.conversation.jsonl')),false);
 }finally{if(running)await new Promise(resolve=>running.server.close(resolve));fs.rmSync(home,{recursive:true,force:true});}
});
