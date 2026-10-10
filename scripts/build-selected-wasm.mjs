// Build only in CI; receipts bind the actual compiler dependency closure.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {existsSync,lstatSync,mkdirSync,readFileSync,readdirSync,realpathSync,writeFileSync} from 'node:fs';
import {dirname,join,relative,resolve,sep} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';

export const WASM_TARGET='wasm32-unknown-unknown';
const entry='js/wasm-worker/src/lib.rs';
const output='js/formal_ai_worker.wasm';
const receiptName='formal_ai_worker.receipt.json';
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const execute=(command,args,options)=>execFileSync(command,args,{...options,timeout:240000,maxBuffer:32*1024*1024});
const git=(root,args)=>execute('git',['-C',root,...args]).toString('utf8').trim();
const revision=value=>assert.match(value,/^[a-f0-9]{40}$/u);

/** Parse the compiler's first Make rule, preserving escaped path characters. */
export function dependencyPaths(text) {
  assert.doesNotMatch(text,/^# env-dep:/mu,'environment-dependent compilation needs an explicit receipt contract');
  const unfolded=text.replace(/\\\r?\n/gu,'');
  const line=unfolded.split(/\r?\n/u).find(value=>value.trim()&&!value.startsWith('#'));
  assert.ok(line,'compiler dependency rule missing');
  let separator=-1;
  for(let index=0;index<line.length;index++) {
    if(line[index]==='\\'){index++;continue;}
    if(line[index]===':'&&/\s/u.test(line[index+1]??'')){separator=index;break;}
  }
  assert.ok(separator>=0,'compiler dependency separator missing');
  const result=[];let value='';
  for(let index=separator+1;index<line.length;index++) {
    const character=line[index];
    if(character==='\\') {assert.ok(index+1<line.length,'incomplete dependency escape');value+=line[++index];}
    else if(/\s/u.test(character)) {if(value){result.push(value);value='';}}
    else value+=character;
  }
  if(value)result.push(value);
  assert.ok(result.length>0,'compiler emitted an empty dependency closure');
  return [...new Set(result)].sort();
}

function ordinaryPath(root,path) {
  const absolute=resolve(root,path),name=relative(root,absolute).split(sep).join('/');
  assert.ok(name&&!name.startsWith('../')&&!name.startsWith('/')&&!name.split('/').includes('.git'),'dependency escapes source');
  assert.equal(lstatSync(absolute).isSymbolicLink(),false,'symlink dependency');
  assert.equal(lstatSync(absolute).isFile(),true,'dependency must be an ordinary file');
  assert.equal(realpathSync(absolute),absolute,'symlink dependency');
  return {absolute,name};
}

/** Compare actual compiler inputs with immutable selected Git blobs. */
function sourceFile(root,commit,path) {
  const {absolute,name}=ordinaryPath(root,path);
  const tree=execute('git',['-C',root,'ls-tree','-z',commit,'--',name]).toString('utf8');
  assert.match(tree,/^100(?:644|755) blob [a-f0-9]{40}\t[^\0]+\0$/u,'untracked or non-file compiler input');
  assert.equal(tree.slice(tree.indexOf('\t')+1,-1),name);
  const original=execute('git',['-C',root,'show',commit+':'+name]),actual=readFileSync(absolute);
  assert.ok(actual.equals(original),'compiler input differs from selected source: '+name);
  return {path:name,gitBlob:tree.match(/^100(?:644|755) blob ([a-f0-9]{40})/u)[1],bytes:actual.length,sha256:digest(actual)};
}

function targetLibraryIdentity(directory) {
  directory=realpathSync(directory);
  const files=readdirSync(directory).sort().map(name=>{
    const path=join(directory,name);assert.equal(lstatSync(path).isFile(),true,'target library must be ordinary');
    const bytes=readFileSync(path);return {name,bytes:bytes.length,sha256:digest(bytes)};
  });
  assert.ok(files.some(file=>/^libcore-.*\.rlib$/u.test(file.name)),'target core library missing');
  return {files,sha256:digest(Buffer.from(JSON.stringify(files)))};
}

export function wasmCompilerArguments(dependencyFile) {
  return ['-D','warnings','--edition=2024','--target',WASM_TARGET,'--crate-type','cdylib',
    '-C','opt-level=z','-C','panic=abort','-C','lto=fat','-C','codegen-units=1',
    '-C','strip=symbols','-C','link-arg=-s','--emit=link,dep-info='+dependencyFile,entry,'-o',output];
}

/** The injected runner exists for boundary tests, never as native compilation evidence. */
export function buildSelectedWasm(root,{run=execute,environment=process.env}={}) {
  root=realpathSync(root);
  if(run===execute)assert.equal(environment.GITHUB_ACTIONS,'true','WASM compilation runs only in CI');
  const commit=git(root,['rev-parse','HEAD']),tree=git(root,['rev-parse','HEAD^{tree}']);revision(commit);revision(tree);
  execute('git',['-C',root,'diff','--quiet','HEAD','--','js/wasm-worker','rust/src','rust/embedded','data/seed']);
  const identity={repository:environment.GITHUB_REPOSITORY,run:String(environment.GITHUB_RUN_ID),
    attempt:String(environment.GITHUB_RUN_ATTEMPT),job:environment.GITHUB_JOB,event:environment.GITHUB_EVENT_NAME,
    workflowCommit:environment.GITHUB_WORKFLOW_SHA};
  assert.match(identity.repository,/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(identity.run,/^\d+$/u);assert.match(identity.attempt,/^[1-9]\d*$/u);revision(identity.workflowCommit);
  assert.ok(identity.job&&identity.event);
  const compiler=run('rustc',['-vV'],{cwd:root}).toString('utf8');
  const compilerCommit=/^commit-hash: ([a-f0-9]{40})$/mu.exec(compiler)?.[1];revision(compilerCommit);
  const libraryDirectory=run('rustc',['--print','target-libdir','--target',WASM_TARGET],{cwd:root}).toString('utf8').trim();
  const libraries=targetLibraryIdentity(libraryDirectory);
  const dependencyFile='js/formal_ai_worker.d',args=wasmCompilerArguments(dependencyFile);
  run('rustc',args,{cwd:root,stdio:'pipe'});
  assert.equal(run('rustc',['-vV'],{cwd:root}).toString('utf8'),compiler,'compiler changed during build');
  assert.deepEqual(targetLibraryIdentity(libraryDirectory),libraries,'target libraries changed during build');
  assert.equal(git(root,['rev-parse','HEAD']),commit,'selected source changed during build');
  const dependencies=dependencyPaths(readFileSync(join(root,dependencyFile),'utf8'));
  const inputs=[...new Set([...dependencies,entry,'js/wasm-worker/build.sh','rust/Cargo.toml'])]
    .map(path=>sourceFile(root,commit,path)).sort((a,b)=>a.path.localeCompare(b.path,'en'));
  const wasm=readFileSync(join(root,output));assert.ok(wasm.subarray(0,8).equals(Buffer.from([0,97,115,109,1,0,0,0])),'not a WASM binary');
  const version=/^version\s*=\s*"([^"]+)"/mu.exec(readFileSync(join(root,'rust/Cargo.toml'),'utf8'))?.[1];assert.ok(version);
  const record={schema:'selected-source-wasm/v1',sourceCommit:commit,sourceTree:tree,packageVersion:version,
    identity,target:WASM_TARGET,compiler,compilerCommit,targetLibraries:libraries,arguments:args,inputs,
    protocolSha256:digest(readFileSync(fileURLToPath(import.meta.url))),
    dependencyInfoSha256:digest(readFileSync(join(root,dependencyFile))),
    output:{path:'formal_ai_worker.wasm',bytes:wasm.length,sha256:digest(wasm)},
    nativeCompilationEvidence:run===execute};
  const bytes=Buffer.from(JSON.stringify(record,null,2)+'\n');writeFileSync(join(root,'js',receiptName),bytes);
  return {record,receiptSha256:digest(bytes)};
}

/** Bind any packaged copy to an independently recorded build receipt. */
export function verifySelectedWasm(root,directory,expectedDigest,{requireNative=true,bindings=null}={}) {
  root=realpathSync(root);directory=realpathSync(directory);assert.match(expectedDigest,/^[a-f0-9]{64}$/u);
  const bytes=readFileSync(join(directory,receiptName));assert.equal(digest(bytes),expectedDigest,'WASM receipt digest differs');
  const record=JSON.parse(bytes);assert.equal(record.schema,'selected-source-wasm/v1');
  if(requireNative)assert.equal(record.nativeCompilationEvidence,true,'a fixture compiler is not native proof');
  assert.equal(record.sourceCommit,git(root,['rev-parse','HEAD']));assert.equal(record.sourceTree,git(root,['rev-parse','HEAD^{tree}']));
  assert.equal(record.target,WASM_TARGET);revision(record.compilerCommit);
  assert.equal(record.protocolSha256,digest(readFileSync(fileURLToPath(import.meta.url))),'captured WASM protocol differs');
  if(bindings)for(const [key,value]of Object.entries(bindings))assert.equal(record.identity[key],value,'WASM producer identity differs: '+key);
  assert.equal(record.compilerCommit,/^commit-hash: ([a-f0-9]{40})$/mu.exec(record.compiler)?.[1]);
  assert.deepEqual(record.arguments,wasmCompilerArguments('js/formal_ai_worker.d'));
  assert.ok(Array.isArray(record.inputs)&&record.inputs.length>0);
  assert.equal(new Set(record.inputs.map(file=>file.path)).size,record.inputs.length);
  for(const input of record.inputs)assert.deepEqual(sourceFile(root,record.sourceCommit,input.path),input);
  assert.ok(record.inputs.some(file=>file.path===entry));assert.ok(record.inputs.some(file=>file.path==='rust/Cargo.toml'));
  const version=/^version\s*=\s*"([^"]+)"/mu.exec(readFileSync(join(root,'rust/Cargo.toml'),'utf8'))?.[1];assert.equal(record.packageVersion,version);
  assert.equal(record.output.path,'formal_ai_worker.wasm');
  const wasmPath=join(directory,record.output.path);assert.equal(lstatSync(wasmPath).isFile(),true);
  assert.equal(realpathSync(wasmPath),wasmPath,'symlink packaged WASM');
  const wasm=readFileSync(wasmPath);assert.equal(wasm.length,record.output.bytes);assert.equal(digest(wasm),record.output.sha256,'packaged WASM differs from fresh build');
  return record;
}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href) {
  const [mode,root=process.cwd(),directory=join(root,'js'),receipt]=process.argv.slice(2);
  const result=mode==='build'?buildSelectedWasm(root):mode==='verify'?verifySelectedWasm(root,directory,receipt,{bindings:process.env.GITHUB_ACTIONS==='true'?{repository:process.env.GITHUB_REPOSITORY,run:process.env.GITHUB_RUN_ID}:null}):assert.fail('build or verify required');
  if(mode==='build'&&process.env.GITHUB_OUTPUT)writeFileSync(process.env.GITHUB_OUTPUT,'receipt-sha256='+result.receiptSha256+'\nwasm-sha256='+result.record.output.sha256+'\n',{flag:'a'});
  console.log(JSON.stringify(result));
}
