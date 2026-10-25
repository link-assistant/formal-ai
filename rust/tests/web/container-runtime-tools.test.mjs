import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {verifyContainerRuntimeTools} from '../../../scripts/verify-container-runtime-tools.mjs';
import {configureContainerClientTools,linkContainerMcp} from '../../../scripts/configure-container-client-tools.mjs';
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
 const home=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-runtime-tools-')));
 t.after(()=>fs.rmSync(home,{recursive:true,force:true}));
 fs.writeFileSync(path.join(home,'package.json'),JSON.stringify(manifest));
 function packageAt(parent,name,version,dependencies={},extra={}){
  const directory=path.join(parent,'node_modules',name);fs.mkdirSync(directory,{recursive:true});
  const metadata={name,version,dependencies,exports:{'./package.json':'./package.json'},...extra};
  fs.writeFileSync(path.join(directory,'package.json'),JSON.stringify(metadata));
  fs.writeFileSync(path.join(directory,'index.js'),'throw Error("metadata fixture must never execute");');
  return directory;
 }
 const commands={'@anthropic-ai/claude-code':['claude','bin/claude.exe'],
  '@openai/codex':['codex','bin/codex.js'],'@playwright/mcp':['playwright-mcp','cli.js']};
 for(const [name,version]of Object.entries(manifest.dependencies)){
  const command=commands[name],extra=command?{bin:{[command[0]]:command[1]}}:{};
  if(command?.[0]==='claude')extra.scripts={postinstall:'node install.cjs'};
  const directory=packageAt(home,name,version,{},extra);
  if(command&&command[0]!=='claude'){
   const bin=path.join(directory,command[1]);fs.mkdirSync(path.dirname(bin),{recursive:true});fs.writeFileSync(bin,'metadata-only fixture');
  }
  if(command?.[0]==='claude')fs.writeFileSync(path.join(directory,'install.cjs'),'throw Error("fixture installer must never execute");');
 }
 const azure=packageAt(home,'@ai-sdk/azure','4.0.99',{'@ai-sdk/openai':'4.0.91','@ai-sdk/deepseek':'3.0.63'});
 const bedrock=packageAt(home,'@ai-sdk/amazon-bedrock','5.0.113',{'@ai-sdk/openai':'4.0.91'});
 packageAt(home,'@ai-sdk/openai','4.0.98');
 const openai=packageAt(azure,'@ai-sdk/openai','4.0.91');
 packageAt(azure,'@ai-sdk/deepseek','3.0.63');
 packageAt(bedrock,'@ai-sdk/openai','4.0.91');
 return {home,openai};
}
test('shared source manifest pins tools and only compatible provider ranges',()=>{
 assert.deepEqual(manifest.dependencies,{'@anthropic-ai/claude-code':'2.1.252','@openai/codex':'0.147.0','@playwright/mcp':'0.0.83',
  'start-command':'0.38.0','@link-assistant/agent':'0.26.15','agent-commander':'0.11.1'});
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
  assert(source.includes('scripts/configure-container-client-tools.mjs'));
  assert(source.includes('--global-mcp /home/box/.formal-ai-runtime-tools'));
  assert.doesNotMatch(source,/bun install -g/u);
  for(const probe of ['"$" --version','agent --version','start-agent --help >/dev/null'])assert(source.includes(probe));
 }
 const installer=fs.readFileSync(path.join(root,'scripts/install-container-runtime-tools.sh'),'utf8');
 assert.match(installer,/install-client-registry-recovery\.mjs/u);
 assert.doesNotMatch(installer,/--ignore-scripts|--trust|--force|\|\| true/u);
});
function clientTransport(home,calls,{deny=null}={}){
 return async(command,args,options)=>{
  calls.push({command,args,options});let stdout='';
  if(args[0]?.endsWith('/install.cjs')){
   const directory=path.dirname(args[0]);assert.equal(options.cwd,directory);
   fs.mkdirSync(path.join(directory,'bin'));fs.writeFileSync(path.join(directory,'bin/claude.exe'),'metadata-only native fixture');
  }else if(args[0]==='--version')stdout=path.basename(command)==='claude'?'2.1.252 (Claude Code)':'codex-cli 0.147.0';
  else if(args[0]==='--help')stdout='--headless';
  else if(args[1]==='list')stdout=path.basename(command)==='claude'?'playwright: local - ✓ Connected':'playwright local enabled Unsupported';
  const rejected=deny?.(command,args);
  return {status:rejected?17:0,signal:null,cleanup_complete:true,stdout:Buffer.from(stdout),stderr:Buffer.from(rejected??'')};
 };
}
test('fresh image setup links only the pinned native client and registers real local MCP commands',async t=>{
 const {home}=fixture(t),calls=[];
 const result=await configureContainerClientTools(home,{transport:clientTransport(home,calls)});
 assert.equal(result.verified,true);assert.equal(result.browserNavigationTested,false);
 const installs=calls.filter(row=>row.args[0]?.endsWith('/install.cjs'));assert.equal(installs.length,1);
 const adds=calls.filter(row=>row.args[0]==='mcp'&&row.args[1]==='add');assert.equal(adds.length,2);
 for(const row of adds){assert(row.args.includes(path.join(home,'node_modules/.bin/playwright-mcp')));assert(row.args.includes('--headless'));}
 for(const row of calls){assert(row.options.timeout>0&&row.options.timeout<=60000);assert.equal(row.options.maxBuffer,262144);}
 const global=path.join(home,'global/node_modules');assert.equal(linkContainerMcp(home,global).linked,true);
 const target=path.join(global,'@playwright/mcp'),before=fs.lstatSync(target).mtimeMs;
 assert.equal(fs.realpathSync(target),fs.realpathSync(path.join(home,'node_modules/@playwright/mcp')));
 linkContainerMcp(home,global);assert.equal(fs.lstatSync(target).mtimeMs,before);
});
test('Bun may link directly to the official native executable while the package bin is its hard link',async t=>{
 const {home}=fixture(t),calls=[],base=clientTransport(home,calls);
 const platform=path.join(home,'node_modules/@anthropic-ai/claude-code-linux-arm64');
 fs.mkdirSync(platform);const native=path.join(platform,'claude');fs.writeFileSync(native,'metadata-only native fixture');
 const bin=path.join(home,'node_modules/.bin');fs.mkdirSync(bin);
 fs.symlinkSync(native,path.join(bin,'claude'));
 const transport=async(command,args,options)=>{
  if(!args[0]?.endsWith('/install.cjs'))return base(command,args,options);
  calls.push({command,args,options});const declared=path.join(path.dirname(args[0]),'bin/claude.exe');
  fs.mkdirSync(path.dirname(declared));fs.linkSync(native,declared);
  return {status:0,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0),stderr:Buffer.alloc(0)};
 };
 assert.equal((await configureContainerClientTools(home,{transport})).verified,true);
 assert.equal(fs.realpathSync(path.join(bin,'claude')),native,'preserve the official existing link');
});
test('a different existing client executable stays refused even if its current bytes match',async t=>{
 const {home}=fixture(t),calls=[],bin=path.join(home,'node_modules/.bin'),foreign=path.join(home,'foreign-claude');
 fs.mkdirSync(bin);fs.writeFileSync(foreign,'metadata-only native fixture');fs.symlinkSync(foreign,path.join(bin,'claude'));
 await assert.rejects(configureContainerClientTools(home,{transport:clientTransport(home,calls)}),/unexpected existing runtime link/u);
 assert.equal(calls.length,1,'refuse before executing the unrelated client');
 assert.equal(fs.realpathSync(path.join(bin,'claude')),foreign);
});
test('image client setup fails once and redacts credentials before any MCP registration',async t=>{
 const {home}=fixture(t),calls=[],secret='private-client-setup-token';
 await assert.rejects(configureContainerClientTools(home,{environment:{...process.env,GH_TOKEN:secret},
  transport:clientTransport(home,calls,{deny:(command,args)=>path.basename(command)==='codex'&&args[0]==='--version'?secret:null})}),
  error=>error.message.includes('Container client setup failed')&&!error.message.includes(secret));
 assert.equal(calls.filter(row=>path.basename(row.command)==='codex').length,1);
 assert.equal(calls.filter(row=>row.args[0]==='mcp').length,0);
});
test('an existing native placeholder is linked and an unhealthy MCP connection refuses the image',async t=>{
 const {home}=fixture(t),native=path.join(home,'node_modules/@anthropic-ai/claude-code/bin');
 fs.mkdirSync(native);fs.writeFileSync(path.join(native,'claude.exe'),'placeholder');
 const calls=[],base=clientTransport(home,calls);
 const transport=async(command,args,options)=>{
  if(args[0]?.endsWith('/install.cjs')){
   calls.push({command,args,options});fs.writeFileSync(path.join(native,'claude.exe'),'native fixture');
   return {status:0,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0),stderr:Buffer.alloc(0)};
  }
  const result=await base(command,args,options);
  if(args[1]==='list')result.stdout=Buffer.from('playwright: local - Failed to connect');
  return result;
 };
 await assert.rejects(configureContainerClientTools(home,{transport}),/Playwright MCP not healthy/u);
 assert.equal(calls.filter(row=>row.args[0]?.endsWith('/install.cjs')).length,1);
 assert.equal(calls.filter(row=>row.args[0]==='mcp'&&row.args[1]==='list').length,1);
});
test('unreviewed native install commands and existing unrelated MCP packages cannot be overwritten',async t=>{
 const {home}=fixture(t),file=path.join(home,'node_modules/@anthropic-ai/claude-code/package.json');
 const value=JSON.parse(fs.readFileSync(file));value.scripts.postinstall='sh arbitrary.sh';fs.writeFileSync(file,JSON.stringify(value));
 const calls=[];await assert.rejects(configureContainerClientTools(home,{transport:clientTransport(home,calls)}),/review Claude native linker/u);
 assert.equal(calls.length,0);
 const global=path.join(home,'global/node_modules');fs.mkdirSync(path.join(global,'@playwright/mcp'),{recursive:true});
 await assert.rejects(async()=>linkContainerMcp(home,global),/unexpected existing runtime link/u);
});
test('global MCP compatibility discovers the installed npm prefix without executing any client',async t=>{
 const {home}=fixture(t),bin=path.join(home,'probe-bin'),global=path.join(home,'discovered-prefix/node_modules');
 fs.mkdirSync(bin);const npm=path.join(bin,'npm');
 fs.writeFileSync(npm,'#!/bin/sh\ntest "$1" = root && test "$2" = --global || exit 19\nprintf "%s\\n" "$TEST_NPM_GLOBAL_ROOT"\n');fs.chmodSync(npm,0o700);
 const result=await boundedCommand(process.execPath,[path.join(root,'scripts/configure-container-client-tools.mjs'),'--global-mcp',home],{
  env:{...process.env,PATH:bin+path.delimiter+process.env.PATH,TEST_NPM_GLOBAL_ROOT:global},timeout:5000,maxBuffer:65536,
 });
 assert.equal(result.status,0,result.stderr.toString());assert.equal(JSON.parse(result.stdout).linked,true);
 assert.equal(fs.realpathSync(path.join(global,'@playwright/mcp')),fs.realpathSync(path.join(home,'node_modules/@playwright/mcp')));
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
