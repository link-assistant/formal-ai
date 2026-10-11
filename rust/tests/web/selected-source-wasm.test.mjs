import assert from 'node:assert/strict';
import test from 'node:test';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {cpSync,mkdirSync,mkdtempSync,readFileSync,rmSync,symlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join} from 'node:path';
import {buildSelectedWasm,dependencyPaths,verifySelectedWasm,wasmCompilerArguments} from '../../../scripts/build-selected-wasm.mjs';

const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
function put(root,path,bytes){mkdirSync(dirname(join(root,path)),{recursive:true});writeFileSync(join(root,path),bytes);}
const git=(root,...args)=>execFileSync('git',['-C',root,...args],{encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
function fixture(t,change=()=>{}) {
  const root=mkdtempSync(join(tmpdir(),'selected WASM source '));t.after(()=>rmSync(root,{recursive:true,force:true}));
  git(root,'init','-q');git(root,'config','user.name','Fixture');git(root,'config','user.email','fixture@example.invalid');
  put(root,'js/wasm-worker/src/lib.rs','// fixture source, never compiled\n');
  put(root,'js/wasm-worker/build.sh','#!/bin/sh\n# selected original build contract\n');
  put(root,'rust/Cargo.toml','[package]\nname="fixture"\nversion="1.2.3"\n');
  put(root,'rust/src/input with space.rs','// actual tracked dependency fixture\n');
  git(root,'add','.');git(root,'commit','-qm','selected inputs');
  const libraries=join(root,'fixture-libraries');mkdirSync(libraries);writeFileSync(join(libraries,'libcore-fixture.rlib'),'fixture target library, never executed');
  let compilerCalls=0;
  const compiler='rustc fixture\ncommit-hash: '+'a'.repeat(40)+'\nhost: fixture\nrelease: fixture\n';
  const run=(command,args)=>{
    assert.equal(command,'rustc','only injected compiler observations');
    if(args[0]==='-vV'){compilerCalls++;return Buffer.from(compiler);}
    if(args[0]==='--print')return Buffer.from(libraries+'\n');
    assert.deepEqual(args,wasmCompilerArguments('js/formal_ai_worker.d'));
    put(root,'js/formal_ai_worker.d','js/formal_ai_worker.wasm: js/wasm-worker/src/lib.rs rust/src/input\\ with\\ space.rs\n');
    put(root,'js/formal_ai_worker.wasm',Buffer.from([0,97,115,109,1,0,0,0]));
    change({root,libraries,compilerCalls});return Buffer.alloc(0);
  };
  const environment={GITHUB_REPOSITORY:'fixture/source',GITHUB_RUN_ID:'123',GITHUB_RUN_ATTEMPT:'1',GITHUB_JOB:'build',GITHUB_EVENT_NAME:'pull_request',GITHUB_WORKFLOW_SHA:git(root,'rev-parse','HEAD')};
  return {root,libraries,run,environment};
}

test('compiler Make dependencies retain escaped spaces and folded lines',()=>{
  assert.deepEqual(dependencyPaths('out.wasm: source.rs input\\ with\\ spaces.rs \\\n nested.rs\nsource.rs:\n'),['input with spaces.rs','nested.rs','source.rs']);
  for(const text of ['', 'out.wasm: ', '# env-dep: SECRET=value\nout: source.rs\n', 'out.wasm: trailing\\'])assert.throws(()=>dependencyPaths(text));
});

test('an injected compiler can verify fixture boundaries but never supplies native evidence',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f);
  assert.equal(result.record.nativeCompilationEvidence,false);
  assert.deepEqual(result.record.inputs.map(input=>input.path),['js/wasm-worker/build.sh','js/wasm-worker/src/lib.rs','rust/Cargo.toml','rust/src/input with space.rs']);
  assert.throws(()=>verifySelectedWasm(f.root,join(f.root,'js'),result.receiptSha256),/fixture compiler/);
  assert.equal(verifySelectedWasm(f.root,join(f.root,'js'),result.receiptSha256,{requireNative:false}).packageVersion,'1.2.3');
});

test('production entry point refuses local compiler execution before invoking rustc',t=>{
  const f=fixture(t);assert.throws(()=>buildSelectedWasm(f.root,{environment:{GITHUB_ACTIONS:'false'}}),/only in CI/);
});

test('a packaged copy retains the independently pinned receipt and output bytes',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f),packaged=join(f.root,'package/assets');mkdirSync(packaged,{recursive:true});
  for(const name of ['formal_ai_worker.wasm','formal_ai_worker.receipt.json'])cpSync(join(f.root,'js',name),join(packaged,name));
  assert.equal(verifySelectedWasm(f.root,packaged,result.receiptSha256,{requireNative:false}).sourceCommit,result.record.sourceCommit);
  writeFileSync(join(packaged,'formal_ai_worker.wasm'),'stale committed WASM');
  assert.throws(()=>verifySelectedWasm(f.root,packaged,result.receiptSha256,{requireNative:false}));
});

test('receipt changes cannot be blessed by an unchanged independent digest',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f),path=join(f.root,'js/formal_ai_worker.receipt.json');
  const forged={...result.record,nativeCompilationEvidence:true};writeFileSync(path,JSON.stringify(forged));
  assert.throws(()=>verifySelectedWasm(f.root,join(f.root,'js'),result.receiptSha256),/digest differs/);
});

test('dirty and untracked compiler dependencies fail even after compiler success',t=>{
  const dirty=fixture(t,({root})=>put(root,'rust/src/input with space.rs','mutated during compile'));
  assert.throws(()=>buildSelectedWasm(dirty.root,dirty),/differs from selected source/);
  const untracked=fixture(t,({root})=>{put(root,'rust/src/new.rs','untracked');put(root,'js/formal_ai_worker.d','out: js/wasm-worker/src/lib.rs rust/src/new.rs\n');});
  assert.throws(()=>buildSelectedWasm(untracked.root,untracked),/untracked/);
});

test('target library mutation during compilation fails the original identity',t=>{
  const f=fixture(t,({libraries})=>writeFileSync(join(libraries,'libcore-fixture.rlib'),'changed'));assert.throws(()=>buildSelectedWasm(f.root,f),/libraries changed/);
});

test('changed selected source, omitted source, and symlink dependency fail',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f),directory=join(f.root,'js');
  put(f.root,'rust/src/input with space.rs','changed after build');assert.throws(()=>verifySelectedWasm(f.root,directory,result.receiptSha256,{requireNative:false}),/differs from selected/);
  git(f.root,'checkout','--','rust/src/input with space.rs');
  const path=join(directory,'formal_ai_worker.receipt.json'),forged={...result.record,inputs:result.record.inputs.filter(input=>input.path!=='js/wasm-worker/src/lib.rs')};
  const bytes=Buffer.from(JSON.stringify(forged));writeFileSync(path,bytes);assert.throws(()=>verifySelectedWasm(f.root,directory,digest(bytes),{requireNative:false}));
  writeFileSync(path,JSON.stringify(result.record,null,2)+'\n');
  const input=join(f.root,'rust/src/input with space.rs');rmSync(input);symlinkSync(join(f.root,'js/wasm-worker/src/lib.rs'),input);
  assert.throws(()=>verifySelectedWasm(f.root,directory,result.receiptSha256,{requireNative:false}),/symlink/);
});

test('foreign source and source revision changes cannot reuse a receipt',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f);
  git(f.root,'commit','--allow-empty','-qm','different selected revision');
  assert.throws(()=>verifySelectedWasm(f.root,join(f.root,'js'),result.receiptSha256,{requireNative:false}));
});

test('actual compiler identity must remain equal before and after its compile call',t=>{
  const fixtureValue=fixture(t);let versions=0;
  const run=(command,args,options)=>{
    const observed=fixtureValue.run(command,args,options);
    if(args[0]==='-vV'&&++versions===2) return Buffer.from(observed.toString().replace('a'.repeat(40),'b'.repeat(40)));
    return observed;
  };
  assert.throws(()=>buildSelectedWasm(fixtureValue.root,{...fixtureValue,run}),/compiler changed/);
});

// Fixture authority is explicit; these tests never compile or execute native code.
test('receipt retains every immutable Git blob and refuses a changed captured protocol',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f);
  for(const input of result.record.inputs)assert.equal(input.gitBlob,git(f.root,'rev-parse','HEAD:'+input.path));
  const path=join(f.root,'js/formal_ai_worker.receipt.json'),forged={...result.record,protocolSha256:'0'.repeat(64)};
  const bytes=Buffer.from(JSON.stringify(forged));writeFileSync(path,bytes);
  assert.throws(()=>verifySelectedWasm(f.root,join(f.root,'js'),digest(bytes),{requireNative:false}),/protocol differs/);
});

test('independent producer identity refuses foreign repository and run while retaining same-run transfers',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f),directory=join(f.root,'js');
  const options={requireNative:false,bindings:{repository:'fixture/source',run:'123'}};
  assert.equal(verifySelectedWasm(f.root,directory,result.receiptSha256,options).identity.run,'123');
  for(const bindings of [{repository:'foreign/source',run:'123'},{repository:'fixture/source',run:'124'}])
    assert.throws(()=>verifySelectedWasm(f.root,directory,result.receiptSha256,{requireNative:false,bindings}),/producer identity differs/);
});


test('private receipt binds packaged bytes without publishing producer identity',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f);
  const packaged=join(f.root,'package/assets');mkdirSync(packaged,{recursive:true});
  cpSync(join(f.root,'js/formal_ai_worker.wasm'),join(packaged,'formal_ai_worker.wasm'));
  const receiptPath=join(f.root,'js/formal_ai_worker.receipt.json');
  const options={requireNative:false,receiptPath,bindings:{run:'123'}};
  assert.deepEqual(verifySelectedWasm(f.root,packaged,result.receiptSha256,options),result.record);
  assert.throws(()=>verifySelectedWasm(f.root,packaged,result.receiptSha256,{requireNative:false}));
  assert.throws(()=>verifySelectedWasm(f.root,packaged,result.receiptSha256,{...options,bindings:{run:'456'}}));
  const linked=join(f.root,'linked-receipt.json');symlinkSync(receiptPath,linked);
  assert.throws(()=>verifySelectedWasm(f.root,packaged,result.receiptSha256,{...options,receiptPath:linked}),/symlink/);
  assert.throws(()=>verifySelectedWasm(f.root,packaged,result.receiptSha256,{...options,receiptPath:packaged}));
  writeFileSync(receiptPath,JSON.stringify({...result.record,identity:{...result.record.identity,run:'456'}}));
  assert.throws(()=>verifySelectedWasm(f.root,packaged,result.receiptSha256,options),/digest differs/);
});

test('actual npm archive stays identical across private run receipts while embedded receipts differ',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f),packageRoot=join(f.root,'npm-package');
  mkdirSync(join(packageRoot,'assets'),{recursive:true});
  put(packageRoot,'package.json',JSON.stringify({name:'formal-ai-inert-repeatability-fixture',version:'1.2.3',files:['assets']}));
  cpSync(join(f.root,'js/formal_ai_worker.wasm'),join(packageRoot,'assets/formal_ai_worker.wasm'));
  const receiptPath=join(f.root,'js/formal_ai_worker.receipt.json');
  const pack=()=>{
    const output=execFileSync('npm',['pack','--ignore-scripts','--json','--cache',join(f.root,'isolated-npm-cache')],{cwd:packageRoot,encoding:'utf8',stdio:['ignore','pipe','pipe']});
    const record=JSON.parse(output)[0];
    return {integrity:record.integrity,sha256:digest(readFileSync(join(packageRoot,record.filename))),files:record.files.map(file=>file.path)};
  };
  const first=pack();
  writeFileSync(receiptPath,JSON.stringify({...result.record,identity:{...result.record.identity,run:'456',attempt:'2'}}));
  const second=pack();assert.deepEqual(second,first);
  assert.equal(first.files.some(path=>path.includes('receipt')),false);
  cpSync(receiptPath,join(packageRoot,'assets/formal_ai_worker.receipt.json'));
  const embeddedSecond=pack();
  writeFileSync(join(packageRoot,'assets/formal_ai_worker.receipt.json'),JSON.stringify(result.record));
  const embeddedFirst=pack();assert.notEqual(embeddedFirst.integrity,embeddedSecond.integrity);
  assert.notEqual(embeddedFirst.sha256,embeddedSecond.sha256);
  console.log(JSON.stringify({scope:'inert npm pack only; no native or publication proof',private:first,embeddedFirst,embeddedSecond}));
});


test('actual distribution generator separates offline receipts from repeatable npm publication',t=>{
  const f=fixture(t),result=buildSelectedWasm(f.root,f),packageRoot=join(f.root,'packages/formal-ai-engine');
  for(const name of ['app.js','vendor.bundle.js','seed-files.js'])put(f.root,'js/'+name,'// Inert built asset fixture, never executed.');
  put(f.root,'js/app/index.html','<!doctype html><title>Inert packaging fixture</title>');
  put(f.root,'js/distribution/service-worker.js','// Inert service worker fixture.');
  put(f.root,'data/seed/fixture.lino','fixture\n');put(f.root,'LICENSE','Fixture only');
  put(packageRoot,'package.json',JSON.stringify({name:'formal-ai-inert-generator-fixture',version:'1.2.3',files:['assets','LICENSE']}));
  const script=join(f.root,'scripts/generate-web-distribution.py');
  mkdirSync(dirname(script),{recursive:true});cpSync(new URL('../../../scripts/generate-web-distribution.py',import.meta.url),script);
  const project=()=>{
    execFileSync('python3',[script],{cwd:f.root,stdio:['ignore','pipe','pipe']});
    const files=JSON.parse(execFileSync('npm',['pack','--ignore-scripts','--json','--cache',join(f.root,'isolated-generator-cache')],
      {cwd:packageRoot,encoding:'utf8',stdio:['ignore','pipe','pipe']}))[0];
    const archive=readFileSync(join(packageRoot,files.filename));
    return {integrity:files.integrity,sha256:digest(archive),files:files.files.map(file=>file.path)};
  };
  const first=project();
  assert.equal(first.files.some(path=>path.includes('receipt')),false);
  assert.match(readFileSync(join(f.root,'js/precache-manifest.js'),'utf8'),/formal_ai_worker\.receipt\.json/u);
  assert.deepEqual(JSON.parse(readFileSync(join(f.root,'js/formal_ai_worker.receipt.json'),'utf8')),result.record);
  put(packageRoot,'assets/stale-file.js','stale');put(packageRoot,'assets/formal_ai_worker.receipt.json','stale receipt');
  const changed={...result.record,identity:{...result.record.identity,run:'456',attempt:'2'}};
  put(f.root,'js/formal_ai_worker.receipt.json',JSON.stringify(changed));
  const second=project();assert.deepEqual(second,first);
  assert.deepEqual(JSON.parse(readFileSync(join(f.root,'js/formal_ai_worker.receipt.json'),'utf8')),changed);
  assert.equal(second.files.some(path=>path.includes('stale')||path.includes('receipt')),false);
  assert.match(readFileSync(join(f.root,'js/precache-manifest.js'),'utf8'),/formal_ai_worker\.receipt\.json/u);
  console.log(JSON.stringify({scope:'actual Python generator plus inert npm pack; no compiler/browser/publication proof',first,second}));
});

// Parse only the original static compiler command and its script-directory paths.
// Unknown shell effects or expansion grammars refuse the complete contract.
function originalCompilerArguments(source,scriptPath) {
  const statements=source.replace(/\\\r?\n/gu,' ').split(/\r?\n/u).map(line=>line.trim()).filter(Boolean);
  assert.equal(statements[0],'#!/usr/bin/env sh');
  assert.equal(statements[1],'set -eu');
  assert.equal(statements.length,3,'unknown shell operation');
  let command=statements[2],words=[];
  const prefix='"$(dirname "$0")/';
  while(command.trim()) {
    command=command.trimStart();
    if(command.startsWith(prefix)) {
      const end=command.indexOf('"',prefix.length);assert.ok(end>prefix.length);
      const suffix=command.slice(prefix.length,end);
      assert.match(suffix,/^[A-Za-z0-9_./-]+$/u);
      words.push(join(dirname(scriptPath),suffix).split(String.fromCharCode(92)).join('/'));
      command=command.slice(end+1);
    } else {
      const word=/^[A-Za-z0-9_./=,:+-]+(?=\s|$)/u.exec(command);
      assert.ok(word,'unknown shell word or effect');words.push(word[0]);command=command.slice(word[0].length);
    }
  }
  assert.equal(words.shift(),'rustc');return words;
}
function requireOriginalCompilerOperands(source,arguments_) {
  const expected=originalCompilerArguments(source,'js/wasm-worker/build.sh');
  const dependency='--emit=link,dep-info=js/formal_ai_worker.d';
  assert.equal(arguments_.filter(value=>value===dependency).length,1,'exact added compiler dependency receipt');
  assert.deepEqual(arguments_.filter(value=>value!==dependency),expected);
}
test('actual compiler tool boundary conserves all original shell operands and receipt authority',t=>{
  const source=readFileSync(new URL('../../../js/wasm-worker/build.sh',import.meta.url),'utf8');
  const f=fixture(t),run=f.run;let compilations=0;
  f.run=(command,arguments_)=>{
    if(arguments_.includes('--crate-type')) {
      assert.equal(command,'rustc');requireOriginalCompilerOperands(source,arguments_);compilations++;
    }
    return run(command,arguments_);
  };
  const result=buildSelectedWasm(f.root,f);assert.equal(compilations,1);
  assert.equal(result.record.nativeCompilationEvidence,false);
  assert.equal(result.record.arguments.includes('--emit=link,dep-info=js/formal_ai_worker.d'),true);
  assert.match(result.record.protocolSha256,/^[a-f0-9]{64}$/u);
  for(const input of result.record.inputs)assert.match(input.gitBlob,/^[a-f0-9]{40}$/u);
  assert.throws(()=>verifySelectedWasm(f.root,join(f.root,'js'),result.receiptSha256),/fixture compiler/);
});
test('every omitted or altered original compiler operand and unknown shell effect refuses',()=>{
  const source=readFileSync(new URL('../../../js/wasm-worker/build.sh',import.meta.url),'utf8');
  const original=originalCompilerArguments(source,'js/wasm-worker/build.sh');
  const arguments_=wasmCompilerArguments('js/formal_ai_worker.d');
  requireOriginalCompilerOperands(source,arguments_);
  for(let index=0;index<arguments_.length;index++) {
    assert.throws(()=>requireOriginalCompilerOperands(source,arguments_.filter((_,position)=>position!==index)));
    const changed=[...arguments_];changed[index]='unknown';assert.throws(()=>requireOriginalCompilerOperands(source,changed));
  }
  assert.ok(original.length>20);
  for(const mutation of [source+'echo unsafe'+String.fromCharCode(10),source.replace('rustc '+String.fromCharCode(92),'EVIL=x rustc '+String.fromCharCode(92)),source.replace('-D warnings','-D "$FLAGS"')]) {
    assert.throws(()=>originalCompilerArguments(mutation,'js/wasm-worker/build.sh'));
  }
});
test('lint runs exactly the selected source compiler before the WASM budget gate',async()=>{
  const {default:YAML}=await import('yaml');
  const workflow=YAML.parse(readFileSync(new URL('../../../.github/workflows/release.yml',import.meta.url),'utf8'));
  const validate=steps=>{
    const builds=steps.map((step,index)=>({step,index})).filter(({step})=>step.run==='node scripts/build-selected-wasm.mjs build');
    assert.equal(builds.length,1);const build=builds[0];assert.equal(build.step.if,'matrix.lane == 1');
    const wasmGate=steps.findIndex(step=>step.run==='rust-script scripts/run-ci-gates.rs --stage wasm');
    assert.ok(build.index<wasmGate);assert.equal(steps[wasmGate].if,'matrix.lane == 1');
    assert.equal(steps.some(step=>step.run==='sh js/wasm-worker/build.sh'),false,'no duplicate compiler invocation');
  };
  validate(workflow.jobs.lint.steps);
  const omitted=structuredClone(workflow.jobs.lint.steps);omitted.splice(omitted.findIndex(step=>step.run==='node scripts/build-selected-wasm.mjs build'),1);assert.throws(()=>validate(omitted));
  const duplicate=structuredClone(workflow.jobs.lint.steps);duplicate.push({...duplicate.find(step=>step.run==='node scripts/build-selected-wasm.mjs build')});assert.throws(()=>validate(duplicate));
  const reordered=structuredClone(workflow.jobs.lint.steps);const build=reordered.splice(reordered.findIndex(step=>step.run==='node scripts/build-selected-wasm.mjs build'),1)[0];reordered.push(build);assert.throws(()=>validate(reordered));
});
