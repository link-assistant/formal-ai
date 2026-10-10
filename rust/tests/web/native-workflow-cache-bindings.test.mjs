import {readCheckedReleaseOperationView} from '../../../scripts/checked-release-operation-view.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync,execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {prepareReleaseBinary,validatePreparedSelection} from '../../../scripts/prepared-release-binary.mjs';
import {publishPreparedFullImage,mirrorPreparedFullImage,releaseImageCacheSettings} from '../../../scripts/release-image-factory.mjs';
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

const release=readCheckedReleaseOperationView().originalSource;
const settings=[...release.matchAll(/          BUILD_CACHE_SETTINGS: \|\n((?:            [^\n]*\n)+)/gu)].map(m=>m[1].split('\n').filter(Boolean).map(line=>line.slice(12)).join('\n'));
test('both actual GHCR workflow settings preserve the original cache contract',()=>{
 assert.equal(settings.length,2);for(const text of settings)assert.deepEqual(releaseImageCacheSettings({BUILD_CACHE_SETTINGS:text}),{'cache-from':'type=gha,scope=docker-image','cache-to':'type=gha,mode=max,scope=docker-image'});
 const observation=release.slice(release.indexOf('  native-response-observations:\n'),release.indexOf('  box-language-projects:\n'));
 assert.ok(observation.includes('group: ${{ github.workflow }}-${{ github.ref }}-native-response-observations'));
 assert.ok(observation.includes("cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}"));
 const workflow=readFileSync(new URL('../../../.github/workflows/native-response-observations.yml',import.meta.url),'utf8');
 const downloads=[...workflow.matchAll(/      - uses: actions\/download-artifact@v8\n([\s\S]*?)(?=      - |$)/gu)];assert.equal(downloads.length,2);
 for(const step of downloads)assert.ok(step[1].includes('        env:\n          NODE_OPTIONS: --disable-warning=DEP0005\n'));
 assert.equal(workflow.match(/NODE_OPTIONS:/gu).length,2);assert.ok(workflow.includes('    timeout-minutes: 30\n'));
});
for(const[index,text]of settings.entries())test('actual Node cacheenv '+index+' retains every full-image Docker CLI operand',async()=>{
 const f=fixture();try{const options=declaredGnuFactory(f);options.environment.BUILD_CACHE_SETTINGS=text;await publishPreparedFullImage(options);
 const builds=readFileSync(f.trace,'utf8').trim().split('\n').map(JSON.parse).filter(c=>c.command==='docker'&&c.args[0]==='buildx');assert.equal(builds.length,1);
 assert.deepEqual(builds[0].args,['buildx','build','--load','--file','Dockerfile','--build-arg','BINARY_SOURCE=prebuilt','--platform','linux/amd64','--tag','formal-ai:prepared-full-123','--label','org.opencontainers.image.revision='+f.git(['rev-parse','HEAD']),'--label','org.opencontainers.image.version=1.2.3','--label','io.link-assistant.formal-ai.executable.sha256='+sha(readFileSync(join(f.cwd,'rust/target/release/formal-ai'))),'--cache-from','type=gha,scope=docker-image','--cache-to','type=gha,mode=max,scope=docker-image','.']);
 }finally{f.close();}
});
for(const[label,text]of [
 ['empty',''],['missing export','cache-from: type=gha,scope=docker-image'],
 ['duplicate key','cache-from: type=gha,scope=docker-image\ncache-from: type=gha,scope=docker-image'],
 ['unknown key','cache-from: type=gha,scope=docker-image\nother: type=gha,mode=max,scope=docker-image'],
 ['different writer scope','cache-from: type=gha,scope=docker-image\ncache-to: type=gha,mode=max,scope=other'],
 ['different export mode','cache-from: type=gha,scope=docker-image\ncache-to: type=gha,mode=min,scope=docker-image'],
 ['registry substitution','cache-from: type=registry,ref=untrusted/image\ncache-to: type=gha,mode=max,scope=docker-image'],
 ['extra flag','cache-from: type=gha,scope=docker-image\ncache-to: type=gha,mode=max,scope=docker-image --push'],
 ['third writer','cache-from: type=gha,scope=docker-image\ncache-to: type=gha,mode=max,scope=docker-image\ncache-to: type=inline']
])test('actual invalid '+label+' cacheenv refuses before any Dockerprocess',async()=>{
 const f=fixture();try{const options=declaredGnuFactory(f);options.environment.BUILD_CACHE_SETTINGS=text;await assert.rejects(publishPreparedFullImage(options));
 assert.equal(readFileSync(f.trace,'utf8').trim().split('\n').map(JSON.parse).some(c=>c.command==='docker'),false);assert.equal(existsSync(join(f.options.directory,'full-image.json')),false);
 }finally{f.close();}
});
