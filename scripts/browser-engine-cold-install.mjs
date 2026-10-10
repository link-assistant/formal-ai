// CI-only cold package smoke. Never publishes or compiles; uses a real browser Worker.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {createServer} from 'node:http';
import {mkdtempSync,readFileSync,writeFileSync,mkdirSync,readdirSync,lstatSync,realpathSync,rmSync} from 'node:fs';
import {join,resolve,relative,sep,extname,dirname} from 'node:path';
import {verifySelectedWasm} from './build-selected-wasm.mjs';
import {tmpdir as temporaryDirectory} from 'node:os';
import {pathToFileURL} from 'node:url';
const packageName='@link-assistant/formal-ai-engine';
const registry='https://registry.npmjs.org';
const sha256=bytes=>createHash('sha256').update(bytes).digest('hex');
export function specification(version){assert.match(version,/^\d+\.\d+\.\d+$/u);return packageName+'@'+version;}
export function integrityOf(archive){assert(lstatSync(archive).isFile());assert(lstatSync(archive).size<=64*1024*1024);return 'sha512-'+createHash('sha512').update(readFileSync(archive)).digest('base64');}
export function validateLock(lock,version,integrity){
  assert.equal(lock.lockfileVersion,3);
  const entry=lock.packages?.['node_modules/'+packageName];assert(entry);
  assert.equal(entry.version,version);assert.equal(entry.integrity,integrity);
  const url=new URL(entry.resolved);assert.equal(url.origin,new URL(registry).origin);
  assert.equal(url.pathname,'/@link-assistant/formal-ai-engine/-/formal-ai-engine-'+version+'.tgz');
  assert.equal(url.username,'');assert.equal(url.password,'');assert.equal(url.search,'');assert.equal(url.hash,'');return entry;
}
function filesWithin(directory){
  const files=[];let totalBytes=0;
  function visit(parent){for(const name of readdirSync(parent)){const file=join(parent,name),fileStatus=lstatSync(file);assert(!fileStatus.isSymbolicLink(),'package symlink');if(fileStatus.isDirectory())visit(file);else {assert(fileStatus.isFile());assert(fileStatus.size<=32*1024*1024);
      totalBytes+=fileStatus.size;assert(totalBytes<=128*1024*1024);files.push(file);assert(files.length<=10000);}}}
  visit(directory);return files;
}
export function inspectInstalled(directory,version,record){
  const manifest=JSON.parse(readFileSync(join(directory,'package.json'),'utf8'));
  assert.equal(manifest.name,packageName);assert.equal(manifest.version,version);
  assert.equal(manifest.exports['.'].import,'./src/index.js');assert.equal(manifest.type,'module');
  assert.equal(record.schema,'selected-source-wasm/v1');assert.equal(record.packageVersion,version);
  assert.equal(record.nativeCompilationEvidence,true);assert.match(record.sourceCommit,/^[a-f0-9]{40}$/u);
  const files=filesWithin(directory);const wasm=readFileSync(join(directory,'assets/formal_ai_worker.wasm'));
  assert.equal(wasm.length,record.output.bytes);assert.equal(sha256(wasm),record.output.sha256);
  assert(wasm.subarray(0,8).equals(Buffer.from([0,97,115,109,1,0,0,0])));
  for(const file of ['src/index.js','assets/memory.js','assets/worker/formal_ai_worker.js','assets/worker-modules.js','assets/seed-files.js'])assert(files.includes(join(directory,file)),file);
  const seeds=files.filter(file=>relative(directory,file).startsWith('assets'+sep+'seed'+sep)&&file.endsWith('.lino'));
  assert(seeds.length>0);
  return seeds.map(file=>({name:relative(join(directory,'assets'),file).split(sep).join('/'),sha256:sha256(readFileSync(file))}));
}
export function safeAsset(directory,pathname){
  let decoded;try{decoded=decodeURIComponent(pathname);}catch{return null;}
  if(!/^\/(src|assets)\//u.test(decoded)||decoded.includes('\\')||decoded.includes('\0'))return null;
  const base=realpathSync(directory);
  const candidate=resolve(base,'.'+decoded),prefix=base+sep;
  if(!candidate.startsWith(prefix))return null;
  try {const fileStatus=lstatSync(candidate);return fileStatus.isFile()&&!fileStatus.isSymbolicLink()&&realpathSync(candidate).startsWith(prefix)?candidate:null;}catch{return null;}
}
export async function realWorkerSmoke(directory,seeds,playwright,remaining){
  const server=createServer((request,response)=>{
    const pathname=new URL(request.url,'http://localhost').pathname;
    if(pathname==='/smoke.html'){response.setHeader('Content-Type','text/html');response.end('<!doctype html><title>Installed engine smoke</title>');return;}
    const file=safeAsset(directory,pathname);if(!file){response.writeHead(404);response.end();return;}
    const type=extname(file)==='.wasm'?'application/wasm':extname(file)==='.js'?'text/javascript':'text/plain';
    response.setHeader('Content-Type',type);response.end(readFileSync(file));
  });
  await new Promise(resolveReady=>server.listen(0,'127.0.0.1',resolveReady));
  let browser,timeout;
  try {
    browser=await playwright.chromium.launch({headless:true,timeout:Math.min(remaining(),30000)});
    const page=await browser.newPage();page.setDefaultTimeout(Math.min(remaining(),30000));
    const origin='http://127.0.0.1:'+server.address().port,failures=[];
    await page.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
    page.on('requestfailed',request=>failures.push(request.url()));
    page.on('response',response=>{if(response.status()>=400)failures.push(response.url()+':'+response.status());});
    // Instrument native Worker messages; the browser still creates and executes the genuine Worker.
    await page.addInitScript(()=>{
      const NativeWorker=globalThis.Worker;globalThis.smokeWorkerMessages=[];
      globalThis.Worker=class extends NativeWorker {
        constructor(...argumentsList){super(...argumentsList);this.addEventListener('message',({data})=>{
          if(['ready','message','seed_dump'].includes(data.kind))globalThis.smokeWorkerMessages.push({
            kind:data.kind,mode:data.mode,error:data.engineError,requestId:data.requestId,engine:data.engine});
        });}
      };
    });
    await page.goto(origin+'/smoke.html');
    const result=await Promise.race([page.evaluate(async expected=>{
      const {createEngine}=await import('/src/index.js');
      const engine=await createEngine({timeoutMs:20000});
      try {
        const ready=globalThis.smokeWorkerMessages.filter(item=>item.kind==='ready');
        if(ready.length!==1||ready[0].mode!=='wasm worker'||ready[0].error)throw Error('not a real available WASM worker');
        await engine.memory.clear();if((await engine.memory.list()).length!==0)throw Error('memory clear');
        await engine.memory.append({role:'user',content:'immutable package smoke event',sentAt:'2026-01-01T00:00:00.000Z'});
        const events=await engine.memory.list();if(events.length!==1||events[0].content!=='immutable package smoke event')throw Error('memory persistence');
        const answer=await engine.solve('2 + 2', {preferences:{demoMode:false}});
        if(answer.kind!=='message'||answer.engine!=='wasm'||answer.error||!answer.requestId||!answer.content)throw Error('solve protocol');
        const bundle=await engine.memory.exportBundle();const parsed=await engine.memory.importBundle(bundle);
        if(parsed.kind!=='bundle'||Object.keys(parsed.seedFiles).length!==expected.length)throw Error('seed dump inventory');
        for(const item of expected){const text=parsed.seedFiles[item.name];if(typeof text!=='string')throw Error('missing seed '+item.name);
          const hash=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text));
          const hex=Array.from(new Uint8Array(hash),value=>value.toString(16).padStart(2,'0')).join('');if(hex!==item.sha256)throw Error('seed bytes '+item.name);}
        const replies=globalThis.smokeWorkerMessages.filter(item=>item.kind!=='ready');
        if(replies.length!==2||new Set(replies.map(item=>item.requestId)).size!==2
          ||replies.some(item=>!/^engine-[1-9][0-9]*$/.test(item.requestId)))throw Error('request correlation');
        engine.dispose();let refused=false;try{await engine.solve('2 + 2');}catch{refused=true;}if(!refused)throw Error('disposed engine');
        return {ready:ready[0],engine:answer.engine,intent:answer.intent,requestId:answer.requestId,seedCount:expected.length,memoryRoundtrip:true,disposeRefused:true};
      }finally{engine.dispose();}
    },seeds),new Promise((_,reject)=>timeout=setTimeout(()=>reject(Error('total browser deadline')),remaining()))]);
    clearTimeout(timeout);assert.deepEqual(failures,[]);return result;
  } finally {clearTimeout(timeout);if(browser)await browser.close();await new Promise(done=>server.close(done));}
}
export async function coldPackageSmoke({root,archive,version,receipt,receiptDigest,publicationStatus}){
  assert.equal(process.env.CI,'true','CI-only real browser package smoke');
  assert(['published','already-matching','skipped-unconfigured','checks-only'].includes(publicationStatus));
  const started=Date.now(),deadline=started+480000;
  const remaining=()=>{const value=deadline-Date.now();assert(value>0,'total smoke deadline exhausted');return value;};
  assert.equal(process.env.GITHUB_ACTIONS,'true');assert.equal(process.env.GITHUB_JOB,'package');
  const bindings={repository:process.env.GITHUB_REPOSITORY,run:process.env.GITHUB_RUN_ID,
    attempt:process.env.GITHUB_RUN_ATTEMPT,job:process.env.GITHUB_JOB,event:process.env.GITHUB_EVENT_NAME,
    workflowCommit:process.env.GITHUB_WORKFLOW_SHA};
  const record=verifySelectedWasm(root,dirname(receipt),receiptDigest,{bindings});
  specification(version);const integrity=integrityOf(archive);
  // Independently verify both the fresh receipt and each installed copy against selected Git source and producer.
  const playwright=createRequire(join(root,'vscode/package.json'))('playwright');
  const temporary=mkdtempSync(join(temporaryDirectory(),'formal-ai-engine-cold-smoke-'));
  const npmConfiguration=join(temporary,'empty-user.npmrc'),globalConfiguration=join(temporary,'empty-global.npmrc');
  writeFileSync(npmConfiguration,'');writeFileSync(globalConfiguration,'');
  const evidence={publicationStatus,version,integrity,receiptDigest,sourceCommit:record.sourceCommit,registryInstalled:false,localTarball:null,registry:null};
  function runPackageManager(argumentsList,workingDirectory){const result=spawnSync('npm',argumentsList,{cwd:workingDirectory,encoding:'utf8',timeout:Math.min(remaining(),180000),maxBuffer:4*1024*1024,
    env:{...Object.fromEntries(Object.entries(process.env).filter(([name])=>!/^npm_config_/iu.test(name))),
      NODE_AUTH_TOKEN:'',NPM_TOKEN:'',npm_config_userconfig:npmConfiguration,npm_config_globalconfig:globalConfiguration,
      npm_config_cache:join(temporary,'cache'),npm_config_registry:registry,npm_config_update_notifier:'false'}});
    if(result.status!==0){let error;try{error=JSON.parse(result.stdout||result.stderr).error;}catch{}
      if(argumentsList[0]==='view'&&error?.code==='E404')return null;
      assert.fail(result.stderr||result.error?.message||'npm failed');}
    return result.stdout;}
  try {
    for(const mode of ['tarball',...(['published','already-matching'].includes(publicationStatus)?['registry']:[])]){
      const directory=join(temporary,mode);mkdirSync(directory);writeFileSync(join(directory,'package.json'),'{}\n');
      if(mode==='registry'){let metadata=null;
        for(let attempt=0;attempt<8;attempt++){metadata=runPackageManager(['view',specification(version),'dist.integrity','--json'],directory);
          if(metadata!==null)break;remaining();await new Promise(done=>setTimeout(done,2000));}
        assert.notEqual(metadata,null,'published exact version unavailable within bounded registry polling');
        assert.equal(JSON.parse(metadata),integrity);}
      runPackageManager(['install','--ignore-scripts','--save-exact','--no-audit','--no-fund',mode==='registry'?specification(version):resolve(archive)],directory);
      const lock=JSON.parse(readFileSync(join(directory,'package-lock.json'),'utf8'));
      const entry=lock.packages?.['node_modules/'+packageName];assert.equal(entry?.version,version);assert.equal(entry?.integrity,integrity);
      if(mode==='registry')validateLock(lock,version,integrity);
      const installed=join(directory,'node_modules',packageName);
      assert.deepEqual(verifySelectedWasm(root,join(installed,'assets'),receiptDigest,{bindings,receiptPath:receipt}),record);
      const seeds=inspectInstalled(installed,version,record);
      const result=await realWorkerSmoke(installed,seeds,playwright,remaining);
      if(mode==='registry'){evidence.registryInstalled=true;evidence.registry=result;}else evidence.localTarball=result;
    }
    evidence.elapsedMs=Date.now()-started;return evidence;
  } finally {rmSync(temporary,{recursive:true,force:true});}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
  const [root,archive,version,receipt,receiptDigest,publicationStatus]=process.argv.slice(2);
  coldPackageSmoke({root,archive,version,receipt,receiptDigest,publicationStatus}).then(result=>console.log(JSON.stringify(result))).catch(error=>{console.error(error);process.exitCode=1;});
}
