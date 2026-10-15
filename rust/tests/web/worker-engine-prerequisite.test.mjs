import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';
import {test} from 'node:test';
import {workerScriptPrerequisites} from '../../../js/server/worker-script-prerequisites.mjs';
import {createBrowserContext,loadBrowserScript} from '../../../js/server/worker-host.mjs';

const root=fileURLToPath(new URL('../../../',import.meta.url));
const provider='js/worker/formal_ai_worker_web_providers_and_wasm_calls.js';
const solver='js/worker/formal_ai_worker_solve.js';
const source=owner=>readFileSync(path.join(root,owner),'utf8');

// The provider below is JavaScript test capability plumbing, never compiled WASM.
// The genuine entry import hook checks the provider before solver evaluation.
// Initialization then runs with ordinary VM microtasks and bounded local seed transport.
function browserEntry({missingProvider=false,failedProvider=false}={}){
  const imports=[],messages=[],calls=[],buffer=new ArrayBuffer(262144);
  const bytes=new Uint8Array(buffer),encoder=new TextEncoder(),decoder=new TextDecoder();
  const exports={memory:{buffer},input_ptr:()=>0,input_capacity:()=>131072,output_ptr:()=>131072,
    engine_handler_precedence:length=>{
      const payload=decoder.decode(bytes.subarray(0,length));calls.push({operation:'registry',payload});
      const names=payload.split('\n').map(line=>/^  handler ([a-z_]+)$/u.exec(line)?.[1]).filter(Boolean);
      assert(names.length>0,'actual seeded handler rows reach the capability provider');
      const output=encoder.encode(names.join('\n'));bytes.set(output,131072);return output.length;
    },
    held_out_echo:length=>{
      const input=bytes.slice(0,length);calls.push({operation:'echo',payload:decoder.decode(input)});
      bytes.set(input,131072);return input.length;
    },
  };
  const browser=vm.createContext({console,URL,URLSearchParams,TextEncoder,TextDecoder,setTimeout,clearTimeout,
    location:{href:'https://example.test/js/worker/formal_ai_worker.js',search:''},
    postMessage:message=>messages.push(JSON.parse(JSON.stringify(message))),
    __testExports:exports,
    __readSeed:url=>{
      const relative=new URL(String(url),'https://example.test/js/').pathname.replace(/^\/js\//u,'');
      assert(relative.startsWith('seed/'),'test transport refuses non-seed text');
      return source('data/'+relative);
    },
  });
  browser.self=browser;
  vm.runInContext('let transportOpen=false; const transportWaiters=[]; function releaseTransport(){transportOpen=true; for(const release of transportWaiters.splice(0))release();} fetch=async url=>{if(!transportOpen)await new Promise(resolve=>transportWaiters.push(resolve));return {ok:true,status:200,text:async()=>__readSeed(url),arrayBuffer:async()=>String(url).includes("formal_ai_worker.wasm")?new ArrayBuffer(0):new TextEncoder().encode(__readSeed(url)).buffer};}; WebAssembly={instantiate:async()=>({instance:{exports:__testExports}})};',browser);
  browser.importScripts=(...urls)=>{
    for(const url of urls){
      const owner=new URL(String(url),'https://example.test/js/').pathname.replace(/^\/js\//u,'js/');
      imports.push(owner);
      if(owner===provider&&failedProvider)throw Error('TestProviderImportFailed');
      if(owner===solver)assert.equal(vm.runInContext('typeof wasmTextCall',browser),'function','actual solver import requires the completed call provider');
      vm.runInContext(source(owner),browser,{filename:owner});
      if(owner===solver)vm.runInContext('releaseTransport()',browser);
      if(owner==='js/worker-modules.js'&&missingProvider){
        browser.FORMAL_AI_WORKER_MODULES=Object.freeze(Array.from(browser.FORMAL_AI_WORKER_MODULES).filter(value=>'js/'+value!==provider));
      }
    }
  };
  const start=()=>vm.runInContext(source('js/worker/formal_ai_worker.js'),browser,{filename:'js/worker/formal_ai_worker.js'});
  return {browser,imports,messages,calls,start};
}

test('generated prerequisites bind solver startup to the actual WASM call provider',()=>{
  const record=workerScriptPrerequisites.find(record=>record.script===solver);
  assert(record,'solver startup has an explicit generated prerequisite');
  assert.equal(record.requires.length,1);assert.equal(record.requires[0].script,provider);
});

test('actual worker entry imports the provider before solver and completes the engine handshake',async()=>{
  const runtime=browserEntry();runtime.start();
  await vm.runInContext('init()',runtime.browser);
  const ready=runtime.messages.find(message=>message.kind==='ready');
  assert(ready,'actual worker initializer posts ready through the JavaScript capability provider');
  assert.equal(ready.mode,'wasm worker');assert.equal(ready.engineError,'');
  assert.equal(runtime.messages.some(message=>message.kind==='engine_unavailable'),false);
  assert(runtime.imports.indexOf(provider)<runtime.imports.indexOf(solver));
  const handshakes=runtime.calls.filter(call=>call.operation==='registry');
  assert.equal(handshakes.length,1);assert.equal(handshakes[0].payload,source('data/seed/handler-precedence.lino'));
  assert.equal(vm.runInContext('typeof wasmTextCall',runtime.browser),'function');
  runtime.browser.__heldOutPayload='héllo\n🙂 held out';
  assert.equal(vm.runInContext('wasmTextCall("held_out_echo",__heldOutPayload)',runtime.browser),'héllo\n🙂 held out');
  assert.equal(runtime.calls.filter(call=>call.operation==='echo').length,1);
});

test('missing provider never evaluates solver or silently chooses a fallback',()=>{
  const runtime=browserEntry({missingProvider:true});runtime.start();
  assert.equal(runtime.imports.includes(solver),false);
  assert.equal(vm.runInContext('typeof init',runtime.browser),'undefined');
  assert.equal(runtime.messages.some(message=>['ready','engine_unavailable'].includes(message.kind)),false);
  assert.equal(runtime.calls.length,0);
});

test('failed actual provider import does not admit the solver initializer',()=>{
  const runtime=browserEntry({failedProvider:true});
  assert.throws(()=>runtime.start(),/TestProviderImportFailed/u);
  assert.equal(runtime.imports.includes(solver),false);assert.equal(runtime.calls.length,0);
});

test('node host refuses solver startup despite caller supplied helper globals',()=>{
  const browser=createBrowserContext({wasmTextCall:()=> 'caller supplied'});
  assert.throws(()=>loadBrowserScript(browser,solver),/IncompleteWorkerScriptPrerequisite/u);
  assert.equal(vm.runInContext('typeof init',browser),'undefined');
  loadBrowserScript(browser,provider);
  assert.equal(vm.runInContext('typeof wasmTextCall',browser),'function');
});
