import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {verifyContainerRuntimeTools} from '../../../scripts/verify-container-runtime-tools.mjs';
import {boundedCommand} from '../../../js/server/bounded-command.mjs';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const manifest=JSON.parse(fs.readFileSync(path.join(root,'packages/container-runtime-tools/package.json'),'utf8'));
test('importing the recovery module from JavaScript stdin has no installer effect',async()=>{
 const module=new URL('../../../scripts/install-client-registry-recovery.mjs',import.meta.url).href;
 const result=await boundedCommand(process.execPath,['--input-type=module','-'],{
  input:'import '+JSON.stringify(module)+'; console.log("imported");',timeout:5000,maxBuffer:65536,
 });
 assert.equal(result.error,undefined);assert.equal(result.status,0,result.stderr.toString());
 assert.equal(result.stdout.toString().trim(),'imported');assert.equal(result.stderr.length,0);
});
function fixture(t){
 const home=fs.mkdtempSync(path.join(os.tmpdir(),'formal-runtime-tools-'));
 t.after(()=>fs.rmSync(home,{recursive:true,force:true}));
 fs.writeFileSync(path.join(home,'package.json'),JSON.stringify(manifest));
 function packageAt(parent,name,version,dependencies={}){
  const directory=path.join(parent,'node_modules',name);fs.mkdirSync(directory,{recursive:true});
  const metadata={name,version,dependencies,exports:{'./package.json':'./package.json'}};
  fs.writeFileSync(path.join(directory,'package.json'),JSON.stringify(metadata));
  fs.writeFileSync(path.join(directory,'index.js'),'throw Error("metadata fixture must never execute");');
  return directory;
 }
 for(const [name,version]of Object.entries(manifest.dependencies))packageAt(home,name,version);
 const azure=packageAt(home,'@ai-sdk/azure','4.0.99',{'@ai-sdk/openai':'4.0.91','@ai-sdk/deepseek':'3.0.63'});
 const bedrock=packageAt(home,'@ai-sdk/amazon-bedrock','5.0.113',{'@ai-sdk/openai':'4.0.91'});
 packageAt(home,'@ai-sdk/openai','4.0.98');
 const openai=packageAt(azure,'@ai-sdk/openai','4.0.91');
 packageAt(azure,'@ai-sdk/deepseek','3.0.63');
 packageAt(bedrock,'@ai-sdk/openai','4.0.91');
 return {home,openai};
}
test('shared source manifest pins tools and only compatible provider ranges',()=>{
 assert.deepEqual(manifest.dependencies,{'start-command':'0.38.0','@link-assistant/agent':'0.26.15','agent-commander':'0.11.1'});
 assert.deepEqual(manifest.overrides,{'@ai-sdk/azure':'4.0.99','@ai-sdk/amazon-bedrock':'5.0.113'});
 assert.equal(manifest.private,true);
});
test('installed graph resolves each provider exact edge without forcing unrelated OpenAI',t=>{
 const {home}=fixture(t);const result=verifyContainerRuntimeTools(home);
 assert.equal(result.verified,true);assert.equal(result.fullTransitiveLock,false);
 assert.equal(result.versions['@ai-sdk/azure -> @ai-sdk/openai'],'4.0.91');
 assert.equal(result.versions['@ai-sdk/azure -> @ai-sdk/deepseek'],'3.0.63');
 assert.equal(result.versions['@ai-sdk/amazon-bedrock -> @ai-sdk/openai'],'4.0.91');
 assert.equal(JSON.parse(fs.readFileSync(path.join(home,'node_modules/@ai-sdk/openai/package.json'))).version,'4.0.98');
});
test('installed graph refuses wrong tool SDK edge and floating policy',t=>{
 const {home,openai}=fixture(t);
 const file=path.join(openai,'package.json'),before=fs.readFileSync(file,'utf8');
 fs.writeFileSync(file,before.replace('4.0.91','4.0.99'));
 assert.throws(()=>verifyContainerRuntimeTools(home),/compatibility edge mismatch/u);
 fs.writeFileSync(file,before);
 const policy=structuredClone(manifest);policy.dependencies['start-command']='^0.38.0';
 fs.writeFileSync(path.join(home,'package.json'),JSON.stringify(policy));
 assert.throws(()=>verifyContainerRuntimeTools(home),/Exact runtime tool pin/u);
});
test('both images share owned project policy and preserve all original runtime probes',()=>{
 for(const filename of ['Dockerfile','Dockerfile.slim']){
  const source=fs.readFileSync(path.join(root,filename),'utf8');
  assert(source.includes('COPY --chown=box:box packages/container-runtime-tools/package.json'));
  assert(source.includes('ENV PATH="/home/box/.formal-ai-runtime-tools/node_modules/.bin:${PATH}"'));
  assert(source.includes('RUN mkdir -p /home/box/.formal-ai-runtime-tools'));
  assert(source.includes('RUN /usr/local/lib/formal-ai-runtime-tools/scripts/install-container-runtime-tools.sh /home/box/.formal-ai-runtime-tools'));
  assert.doesNotMatch(source,/bun install -g/u);
  for(const probe of ['"$" --version','agent --version','start-agent --help >/dev/null'])assert(source.includes(probe));
 }
 const installer=fs.readFileSync(path.join(root,'scripts/install-container-runtime-tools.sh'),'utf8');
 assert.match(installer,/install-client-registry-recovery\.mjs/u);
 assert.doesNotMatch(installer,/--ignore-scripts|--trust|--force|\|\| true/u);
});
for(const filename of ['Dockerfile','Dockerfile.slim'])test(filename+' ships the executable recovery closure and preserves unrelated install failures',async t=>{
 const {home}=fixture(t),bin=path.join(home,'bin'),log=path.join(home,'calls');
 fs.mkdirSync(bin);
 const fake=path.join(bin,'bun');fs.writeFileSync(fake,'#!/bin/sh\nprintf "%s\\n" "$*" >> "$CALL_LOG"\nexit 37\n');fs.chmodSync(fake,0o700);
 // Stage the real COPY declarations, so a missing transitive source import fails
 // before our stand-in package manager can run. No image build or install.
 const stage=path.join(home,'stage'),prefix='/usr/local/lib/formal-ai-runtime-tools/';
 const docker=fs.readFileSync(path.join(root,filename),'utf8').replace(/\\\n\s*/gu,' ');
 for(const line of docker.split('\n')){
  if(!line.startsWith('COPY ')||!line.endsWith('/'))continue;
  const parts=line.split(/\s+/u),destination=parts.at(-1);
  if(!destination.startsWith(prefix))continue;
  const target=path.join(stage,destination.slice(prefix.length));fs.mkdirSync(target,{recursive:true});
  for(const source of parts.slice(1,-1).filter(value=>!value.startsWith('--')))
   fs.copyFileSync(path.join(root,source),path.join(target,path.basename(source)));
 }
 const result=await boundedCommand('bash',[path.join(stage,'scripts/install-container-runtime-tools.sh'),home],{
  cwd:home,env:{...process.env,PATH:bin+path.delimiter+process.env.PATH,CALL_LOG:log},timeout:5000,maxBuffer:65536,
 });
 assert.equal(result.error,undefined);assert.equal(result.status,37);assert.equal(result.cleanup_complete,true);
 assert.deepEqual(fs.readFileSync(log,'utf8').trim().split('\n'),['install --production --cwd '+home]);
});
