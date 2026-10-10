import {readCheckedReleaseOperationView} from '../../../scripts/checked-release-operation-view.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync,execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {prepareReleaseBinary,validatePreparedSelection} from '../../../scripts/prepared-release-binary.mjs';
import {publishPreparedFullImage,mirrorPreparedFullImage} from '../../../scripts/release-image-factory.mjs';
import {sealSource} from '../../../scripts/native-release-source.mjs';
import {expectedIdentity} from '../../../scripts/native-release-artifact.mjs';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const compilerHost=process.platform==='darwin'?(process.arch==='arm64'?'aarch64-apple-darwin':'x86_64-apple-darwin'):process.platform==='win32'?'x86_64-pc-windows-msvc':process.arch==='arm64'?'aarch64-unknown-linux-gnu':'x86_64-unknown-linux-gnu';
function fixture(mode='push') {
 const directory=mkdtempSync(join(tmpdir(),'prepared-release-node-fixture-')),cwd=join(directory,'source'),remote=join(directory,'remote.git');
 mkdirSync(cwd);
 const metadata={workspace_members:['fixture-formal-ai'],packages:[{id:'fixture-formal-ai',name:'formal-ai',version:'1.2.3',targets:[{name:'formal-ai',kind:['bin']}],features:{default:['agentic-coding']}}]};
 for(const [path,body] of [['rust/src/main.rs','// Node process fixture; never compiled as Rust\n'],['rust/Cargo.toml','[package]\nname="formal-ai"\nversion="1.2.3"\n'],['rust/Cargo.lock','fixture-only-lock\n'],['.gitignore','rust/target/\n']]) {mkdirSync(join(cwd,path,'..'),{recursive:true});
 writeFileSync(join(cwd,path),body);
 }
 const git=args=>execFileSync('git',args,{cwd,encoding:'utf8'}).trim();
 git(['init','-q','-b','main']);
 git(['add','.']);
 git(['-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','-qm','real source fixture']);
 git(['tag','v1.2.3']);
 execFileSync('git',['init','--bare','-q',remote]);
 git(['remote','add','origin',remote]);
 git(['push','-q','origin','main','--tags']);
 const environment={...process.env,GITHUB_REF:'refs/heads/main',GITHUB_EVENT_NAME:mode,GITHUB_REPOSITORY:'fixture/formal-ai',GITHUB_RUN_ID:'123',GITHUB_RUN_ATTEMPT:'1',GITHUB_JOB:'fixture_producer',RUSTFLAGS:'-Dwarnings',RELEASE_MODE:'instant',DOCKERHUB_IMAGE:'fixture/mirror',ACTIONS_RUNTIME_TOKEN:'controlled-fixture-token',ACTIONS_RESULTS_URL:'https://cache.example.invalid/'};
 for(const key of Object.keys(environment))if(key.startsWith('CARGO_PROFILE_')||key.startsWith('CARGO_TARGET_')||['CARGO_BUILD_TARGET','CARGO_TARGET_DIR','CARGO_ENCODED_RUSTFLAGS','RUSTC','RUSTC_WORKSPACE_WRAPPER'].includes(key))delete environment[key];
 const control=join(directory,'control.json'),trace=join(directory,'trace.jsonl'),tool=join(directory,'node-tool.mjs');
 const compiler='Declared Node fixture compiler metadata only\nrelease: 1.99.0\ncommit-hash: '+'c'.repeat(40)+'\nhost: '+compilerHost+'\n';
 writeFileSync(control,JSON.stringify({cwd,metadata,compiler,trace,mode:'success'}));
 writeFileSync(tool,`import fs from 'node:fs';import path from 'node:path';const state=JSON.parse(fs.readFileSync(process.argv[2],'utf8')),command=process.argv[3],args=process.argv.slice(4);fs.appendFileSync(state.trace,JSON.stringify({command,args})+'\\n');const fail=(text,code)=>{console.error(text);process.exit(code);};
if(command==='cargo'){if(args[0]==='metadata')console.log(JSON.stringify(state.metadata));else {if(state.mode==='build-failed')fail('actual controlled process failure',17);const binary=path.join(state.cwd,'rust/target/release/formal-ai');fs.mkdirSync(path.dirname(binary),{recursive:true});fs.writeFileSync(binary,\"#!/usr/bin/env node\\nconsole.log('formal-ai 1.2.3');\\n\");fs.chmodSync(binary,0o755);if(process.platform==='win32')fs.copyFileSync(binary,binary+'.exe');if(state.mode==='source-mutated')fs.appendFileSync(path.join(state.cwd,'rust/src/main.rs'),'// changed during process\\n');} }
else if(command==='rustc')process.stdout.write(state.compiler);
else if(command==='docker'){const destination=args.some(value=>value.replace(/^docker\\.io\\//u,'').startsWith('fixture/mirror'));const image=destination?state.destination:state.image;const digest=destination?state.destinationDigest:state.digest;const reference=image+'@'+digest;
 if(state.mode==='docker-failed'&&args[0]==='buildx')fail('actual Node Docker stand-in failed',23);
 if(args[0]==='image'&&args[1]==='inspect')console.log(JSON.stringify([{Os:'linux',Architecture:'amd64',Id:destination&&state.mode==='mirror-config-changed'?'sha256:'+'f'.repeat(64):state.configDigest,RepoDigests:[reference],Config:{Env:['FORMAL_AI_IMAGE_VARIANT=dind'],Labels:{'org.opencontainers.image.revision':state.head,'org.opencontainers.image.version':'1.2.3','io.link-assistant.formal-ai.executable.sha256':state.binarySha}}}]));
 if(args[0]==='run'){if(args.includes('--version'))console.log(state.mode==='wrong-version'?'formal-ai 9.9.9':'formal-ai 1.2.3');else if(args.includes('sha256sum'))console.log((state.mode==='wrong-binary'?'e'.repeat(64):state.binarySha)+'  /usr/local/bin/formal-ai');else {if(state.mode==='runtime-failed')fail('actual runtime stand-in failed',24);console.log('Actual Node fixture DinD runtime observation');}}
}else fail('unknown controlled command',99);
`);
 const run=(command,args,options)=>command==='cargo'||command==='rustc'||command==='docker'?spawnSync(process.execPath,[tool,control,command,...args],{...options,stdio:'pipe'}):process.platform==='win32'&&command.startsWith(cwd)?spawnSync(process.execPath,[command,...args],options):spawnSync(command,args,options);
 const update=fields=>writeFileSync(control,JSON.stringify({...JSON.parse(readFileSync(control,'utf8')),...fields}));
 const options={cwd,version:'1.2.3',directory:join(directory,'prepared'),environment,run,repositoryUrl:remote};
 return {directory,cwd,remote,metadata,compiler,options,run,update,trace,git,close:()=>rmSync(directory,{recursive:true,force:true})};
}
function declaredGnuFactory(f) {
 // A real Node stand-in emits the bytes/status. GNU compiler/identity fields are declared fixture data, never actual native compilation.
 const built=f.run('cargo',['build','--manifest-path','rust/Cargo.toml','--release'],{cwd:f.cwd,env:f.options.environment,encoding:'utf8'});
 assert.equal(built.status,0);
 const selection=validatePreparedSelection(f.cwd,'1.2.3',f.options.environment,{repositoryUrl:f.remote});
 const compiler=f.compiler.replace(compilerHost,'x86_64-unknown-linux-gnu');
 mkdirSync(join(f.options.directory,'source'),{recursive:true});
 mkdirSync(join(f.options.directory,'binary'),{recursive:true});
 const source=sealSource(f.cwd,join(f.options.directory,'source'),{head:selection.head,base:selection.head,tag:selection.tag,run:'123',metadata:f.metadata,compiler});
 const identity=expectedIdentity(source.record,'x86_64-unknown-linux-gnu',source.sha256,'123'),binary=readFileSync(join(f.cwd,'rust/target/release/formal-ai'));
 const executable={name:'formal-ai',bytes:binary.length,sha256:sha(binary)};
 writeFileSync(join(f.options.directory,'binary/formal-ai'),binary);
 const nativeBytes=Buffer.from(JSON.stringify({version:1,identity,producer:{job:'declared_fixture',attempt:'1',compiler,environment:{RUSTFLAGS:'-Dwarnings'}},executable})+'\n');
 writeFileSync(join(f.options.directory,'binary/native-executable-receipt.json'),nativeBytes);
 const record={schema:'prepared-release-build/v1',authority:'guarded-main-prepared-release',selection,sourceSelectionSha256:source.sha256,nativeReceiptSha256:sha(nativeBytes),build:{command:'cargo',arguments:['build','--manifest-path','rust/Cargo.toml','--release'],status:built.status,signal:built.signal,complete:true},identity,executable};
 writeFileSync(join(f.options.directory,'prepared-build.json'),JSON.stringify(record)+'\n');
 const configDigest='sha256:'+sha(Buffer.from('actual declared fixture config'));
 const manifest={schemaVersion:2,mediaType:'application/vnd.docker.distribution.manifest.v2+json',config:{mediaType:'application/vnd.docker.container.image.v1+json',digest:configDigest,size:20},layers:[{mediaType:'application/vnd.docker.image.rootfs.diff.tar.gzip',digest:'sha256:'+sha(Buffer.from('layer fixture')),size:30}]};
 const bytes=Buffer.from(JSON.stringify(manifest)),destinationBytes=Buffer.from(JSON.stringify({...manifest,annotations:{fixture:'destination encoding differs'}}));
 const digest='sha256:'+sha(bytes),destinationDigest='sha256:'+sha(destinationBytes);
 f.update({head:selection.head,binarySha:executable.sha256,configDigest,image:'ghcr.io/fixture/formal-ai',destination:'fixture/mirror',digest,destinationDigest});
 const requests=[];
 const fetcher=async(url,options)=>{requests.push({url:String(url),authorization:options.headers?.Authorization??null});
 if(String(url).includes('/token?'))return Response.json({token:'anonymous-fixture-token'});
 return new Response(String(url).includes('registry-1.docker.io')?destinationBytes:bytes,{status:200,headers:{'Docker-Content-Digest':String(url).includes('registry-1.docker.io')?destinationDigest:digest}});
 };
 return {...f.options,image:'ghcr.io/fixture/formal-ai',fetcher,requests};
}
for(const event of ['push','workflow_dispatch'])test('actual Node producer binds the single existing build and source under '+event,()=>{
 const f=fixture(event);
 try{const record=prepareReleaseBinary(f.options);
 assert.equal(record.selection.head,f.git(['rev-parse','HEAD']));
 assert.equal(record.executable.sha256,sha(readFileSync(join(f.cwd,'rust/target/release/formal-ai'))));
 assert.deepEqual(record.build,{command:'cargo',arguments:['build','--manifest-path','rust/Cargo.toml','--release'],status:0,signal:null,complete:true});
 const calls=readFileSync(f.trace,'utf8').trim().split('\n').map(JSON.parse);
 assert.equal(calls.filter(call=>call.command==='cargo'&&call.args[0]==='build').length,1);
 assert.equal(record.authority,'guarded-main-prepared-release');
 }finally{f.close();
 }
});
for(const mode of ['build-failed','source-mutated'])test('actual controlled '+mode+' cannot produce a completed build receipt',()=>{
 const f=fixture();
 try{f.update({mode});
 assert.throws(()=>prepareReleaseBinary(f.options));
 assert.equal(existsSync(join(f.options.directory,'prepared-build.json')),false);
 }finally{f.close();
 }
});
test('untrusted event/branch, version, tag and build overrides refuse before controlled build',()=>{
 const f=fixture();
 try{for(const values of [
  {GITHUB_EVENT_NAME:'pull_request'},
  {GITHUB_REF:'refs/heads/other'},
  {GITHUB_EVENT_NAME:'workflow_dispatch',RELEASE_MODE:'checks'},
  {CARGO_BUILD_TARGET:'unobserved-target'},
  {CARGO_PROFILE_RELEASE_LTO:'thin'},
  {CARGO_ENCODED_RUSTFLAGS:'different'},
  {CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:'different'}])assert.throws(()=>prepareReleaseBinary({...f.options,environment:{...f.options.environment,...values}}));
 assert.throws(()=>prepareReleaseBinary({...f.options,version:'1.2.4'}));
 assert.equal(existsSync(f.trace),false);
 }finally{f.close();
 }
});
test('actual Node Docker stand-in builds prebuilt once, preserves immutable full observations and mirrors without compilation',async()=>{
 const f=fixture();
 try{const options=declaredGnuFactory(f);
 const full=await publishPreparedFullImage(options);
 const mirrored=await mirrorPreparedFullImage({...options,image:'fixture/mirror'});
 assert.equal(full.expected.binarySha256,sha(readFileSync(join(f.cwd,'rust/target/release/formal-ai'))));
 assert.equal(full.runtime.status,0);
 assert.equal(mirrored.source.configDigest,mirrored.destination.configDigest);
 assert.notEqual(mirrored.sourceImage.split('@')[1],mirrored.destinationImage.split('@')[1]);
 assert.ok(options.requests.some(request=>request.url.startsWith('https://ghcr.io/')));
 assert.ok(options.requests.some(request=>request.url.startsWith('https://registry-1.docker.io/')));
 const calls=readFileSync(f.trace,'utf8').trim().split('\n').map(JSON.parse);
 const docker=calls.filter(call=>call.command==='docker');
 const builds=docker.filter(call=>call.args[0]==='buildx'&&call.args[1]==='build');
 assert.equal(builds.length,1);
 for(const image of ['ghcr.io/fixture/formal-ai','fixture/mirror'])for(const suffix of ['1.2.3','latest'])assert.ok(docker.some(call=>call.args[0]==='push'&&call.args[1]===image+':'+suffix));
 const publicationOrder=[];
 for(const image of ['ghcr.io/fixture/formal-ai','fixture/mirror'])for(const suffix of ['1.2.3','latest']) {
  const tagged=docker.findIndex(call=>call.args[0]==='tag'&&call.args[2]===image+':'+suffix);
  const pushed=docker.findIndex(call=>call.args[0]==='push'&&call.args[1]===image+':'+suffix);
  assert.ok(tagged>=0&&pushed>tagged,'actual metadata suffix must be tagged before push: '+image+':'+suffix);
  publicationOrder.push(pushed);
 }
 assert.ok(publicationOrder[0]<publicationOrder[1]&&publicationOrder[1]<publicationOrder[2]&&publicationOrder[2]<publicationOrder[3],
  'actual full version/latest publication must precede mirror version/latest publication');
 assert.ok(builds[0].args.includes('BINARY_SOURCE=prebuilt'));
 assert.ok(builds[0].args.includes('type=gha,scope=docker-image'));
 assert.ok(builds[0].args.includes('type=gha,mode=max,scope=docker-image'));
 assert.equal(calls.filter(call=>call.command==='cargo'&&call.args[0]==='build').length,1);
 assert.equal(docker.filter(call=>call.args.includes('verify-formal-ai-dind')).length,3);
 }finally{f.close();
 }
});
for(const mode of ['docker-failed','wrong-version','wrong-binary','runtime-failed'])test('actual '+mode+' observation cannot certify an image publication',async()=>{
 const f=fixture();
 try{const options=declaredGnuFactory(f);
 f.update({mode});
 await assert.rejects(publishPreparedFullImage(options));
 assert.equal(existsSync(join(f.options.directory,'full-image.json')),false);
 }finally{f.close();
 }
});
test('altered producer bytes, source fields and status veto before any Docker process',async()=>{
 for(const kind of ['binary','source','status','identity']){const f=fixture();
 try{const options=declaredGnuFactory(f);
 if(kind==='binary')writeFileSync(join(f.cwd,'rust/target/release/formal-ai'),'altered');
 else {const path=join(f.options.directory,'prepared-build.json'),record=JSON.parse(readFileSync(path,'utf8'));
 if(kind==='source')record.selection.head='e'.repeat(40);
 if(kind==='status')record.build.status=1;
 if(kind==='identity')record.identity.rust_flags='different';
 writeFileSync(path,JSON.stringify(record));
 }await assert.rejects(publishPreparedFullImage(options));
 assert.equal(readFileSync(f.trace,'utf8').includes('"docker"'),false);
 }finally{f.close();
 }}
});
test('anonymous transport failure and altered manifest bytes refuse image receipts',async()=>{
 for(const kind of ['denied','bytes']){const f=fixture();
 try{const options=declaredGnuFactory(f);
 const original=options.fetcher;
 options.fetcher=async(url,request)=>kind==='denied'?new Response('',{status:401}):String(url).includes('/token?')?original(url,request):new Response('{}',{status:200});
 await assert.rejects(publishPreparedFullImage(options));
 assert.equal(existsSync(join(f.options.directory,'full-image.json')),false);
 }finally{f.close();
 }}
});
test('optional mirror cannot target another registry or disagree with configured opt-in target',async()=>{
 const f=fixture();
 try{const options=declaredGnuFactory(f);
 for(const image of ['evil.example/user/image','another/mirror'])await assert.rejects(mirrorPreparedFullImage({...options,image}));
 assert.equal(readFileSync(f.trace,'utf8').includes('"docker"'),false);
 }finally{f.close();
 }
});

test('Docker Hub canonical digest spelling accepts configured docker.io alias without another build',async()=>{
 const f=fixture();
 try{const options=declaredGnuFactory(f);
 await publishPreparedFullImage(options);
 options.environment.DOCKERHUB_IMAGE='docker.io/fixture/mirror';
 const mirrored=await mirrorPreparedFullImage({...options,image:'docker.io/fixture/mirror'});
 assert.match(mirrored.destinationImage,/^fixture\/mirror@sha256:/u);
 const calls=readFileSync(f.trace,'utf8').trim().split('\n').map(JSON.parse);
 assert.equal(calls.filter(call=>call.command==='docker'&&call.args[0]==='buildx').length,1);
 }finally{f.close();
 }
});
test('missing actual GHA cache environment refuses before any image process',async()=>{
 for(const key of ['ACTIONS_RUNTIME_TOKEN','ACTIONS_RESULTS_URL']){const f=fixture();
 try{const options=declaredGnuFactory(f);
 delete options.environment[key];
 await assert.rejects(publishPreparedFullImage(options));
 assert.equal(readFileSync(f.trace,'utf8').includes('"docker"'),false);
 }finally{f.close();
 }}
});
test('altered actual destination config refuses mirror receipt',async()=>{
 const f=fixture();
 try{const options=declaredGnuFactory(f);
 await publishPreparedFullImage(options);
 f.update({mode:'mirror-config-changed'});
 await assert.rejects(mirrorPreparedFullImage({...options,image:'fixture/mirror'}));
 assert.equal(existsSync(join(f.options.directory,'dockerhub-image.json')),false);
 }finally{f.close();
 }
});

test('both guarded publication jobs bind the same producer, full factory, optional mirror and original slim order',()=>{
 const workflow=readCheckedReleaseOperationView().originalSource;
 for(const [name,version] of [['auto-release','steps.current_version.outputs.version'],['manual-release','steps.version.outputs.new_version']]){
  const start=workflow.indexOf('  '+name+':\n'),rest=workflow.slice(start),end=rest.search(/\n  [a-z][a-z-]*:/u);
 const body=end<0?rest:rest.slice(0,end);
  assert.match(body,/runs-on: ubuntu-24\.04/u);
 assert.match(body,/timeout-minutes: 90/u);
  assert.equal(body.match(/run: node scripts\/prepared-release-binary\.mjs/g)?.length,1);
 assert.equal(body.match(/run: node scripts\/release-image-factory\.mjs publish/g)?.length,1);
 assert.equal(body.match(/run: node scripts\/release-image-factory\.mjs mirror/g)?.length,1);
  assert.ok(body.includes('RELEASE_VERSION: ${{ '+version+' }}'));
 assert.match(body,/if: steps\.dockerhub\.outputs\.enabled == 'true'[\s\S]*?run: node scripts\/release-image-factory\.mjs mirror/u);
  assert.match(body,/core\.setSecret\(value\)/u);
 assert.match(body,/core\.exportVariable\(key, value\)/u);
 assert.match(body,/ACTIONS_RESULTS_URL/u);
  assert.ok(body.indexOf('Publish and verify the slim GHCR sidecar')<body.indexOf('Create GitHub Release'));
 assert.match(body,/uses: \.\/\.github\/actions\/publish-slim-image/u);
  assert.match(body,/prepared-release-SHA256SUMS\.txt/u);
 assert.match(body,/packages: write/u);
 assert.doesNotMatch(body,/uses: docker\/build-push-action|uses: docker\/metadata-action/u);
 }
});

test('a mirror cannot substitute the source repository or independent source selection in its full-image receipt',async()=>{
 for(const field of ['image','sourceSelectionSha256','authority']){const f=fixture();
 try{const options=declaredGnuFactory(f);
 await publishPreparedFullImage(options);
 const path=join(f.options.directory,'full-image.json'),source=JSON.parse(readFileSync(path,'utf8'));
 source[field]=field==='image'?source.image.replace('ghcr.io/fixture/formal-ai','ghcr.io/other/repository'):field==='sourceSelectionSha256'?'f'.repeat(64):'unverified';
 writeFileSync(path,JSON.stringify(source));
 const before=readFileSync(f.trace,'utf8');
 await assert.rejects(mirrorPreparedFullImage({...options,image:'fixture/mirror'}));
 assert.equal(readFileSync(f.trace,'utf8'),before);
 assert.equal(existsSync(join(f.options.directory,'dockerhub-image.json')),false);
 }finally{f.close();
 }}
});
