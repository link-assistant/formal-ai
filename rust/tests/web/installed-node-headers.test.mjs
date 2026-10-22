import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {boundedCommand} from '../../../js/server/bounded-command.mjs';
const root=fileURLToPath(new URL('../../../',import.meta.url));
function fixture(t){
 const home=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-node-headers-')));
 t.after(()=>fs.rmSync(home,{recursive:true,force:true}));
 const prefix=path.join(home,'installed-node'),bin=path.join(prefix,'bin');
 const include=path.join(prefix,'include','node');
 fs.mkdirSync(bin,{recursive:true});fs.mkdirSync(include,{recursive:true});
 for(const name of ['node.h','config.gypi'])fs.writeFileSync(path.join(include,name),'installed header fixture\n');
 const header=version=>version.split('.').map((part,index)=>'#define NODE_'+['MAJOR','MINOR','PATCH'][index]+'_VERSION '+part).join('\n')+'\n';
 fs.writeFileSync(path.join(include,'node_version.h'),header(process.versions.node));
 const quote=value=>"'"+value.replaceAll("'","'\\''")+"'";
 const bootstrap='process.execPath=process.env.FAKE_NODE_EXEC;process.argv=["node",...process.argv.slice(1)];require("node:vm").runInThisContext(require("node:fs").readFileSync(0,"utf8"));';
 fs.writeFileSync(path.join(bin,'node'),'#!/bin/sh\nexec '+quote(process.execPath)+' --eval '+quote(bootstrap)+' "$@"\n');
 fs.chmodSync(path.join(bin,'node'),0o700);
 const npm=`#!${process.execPath}
const fs=require('node:fs');const args=process.argv.slice(2);
if(args.includes('--version')){console.log('11.9.0');process.exit(0);}
fs.appendFileSync(process.env.CALL_LOG,JSON.stringify({args,nodedir:process.env.npm_package_config_node_gyp_nodedir,legacy:process.env.npm_config_nodedir,uppercase:process.env.NPM_PACKAGE_CONFIG_NODE_GYP_NODEDIR})+'\\n');
if(args.includes('config')){console.log(process.env.FAKE_CONFIG||'{}');process.exit(0);}
process.stderr.write(process.env.FAKE_DIAGNOSTIC||'');console.log('added 1 package');process.exit(Number(process.env.FAKE_STATUS||0));
`;
 fs.writeFileSync(path.join(bin,'npm'),npm);fs.chmodSync(path.join(bin,'npm'),0o700);
 return {home,prefix,include,bin,header,log:path.join(home,'calls')};
}
async function run(f,environment={}){
 const env={...process.env,PATH:f.bin+path.delimiter+process.env.PATH,CALL_LOG:f.log,FAKE_NODE_EXEC:path.join(f.bin,'node'),...environment};
 for(const key of Object.keys(env))if(/^npm_(?:config_|package_config_node_gyp_)(?:nodedir|target|runtime|dist_url|devdir)$/iu.test(key)&&!Object.hasOwn(environment,key))delete env[key];
 const result=await boundedCommand('/bin/bash',[path.join(root,'scripts/install-node-dependencies.sh'),f.home],{cwd:f.home,env,timeout:5000,maxBuffer:262144});
 assert.equal(result.error,undefined);assert.equal(result.cleanup_complete,true);
 const calls=fs.existsSync(f.log)?fs.readFileSync(f.log,'utf8').trim().split('\n').map(JSON.parse):[];
 return {result,calls,install:calls.find(call=>call.args.includes('install')||call.args.includes('ci'))};
}
test('ordinary installs retain original arguments without reading or selecting headers',async t=>{
 const f=fixture(t),{result,calls,install}=await run(f);
 assert.equal(result.status,0);assert.equal(calls.length,1);
 assert.deepEqual(install.args,['--prefix',f.home,'install','--no-audit','--no-fund']);
 assert.equal(install.nodedir,undefined);assert.equal(install.legacy,undefined);
});
test('explicit host install reuses only matching installed headers through preferred scoped option',async t=>{
 const f=fixture(t),{result,install}=await run(f,{INSTALL_NODE_DEPENDENCIES_USE_NODE_HEADERS:'1',INSTALL_NODE_DEPENDENCIES_MODE:'ci'});
 assert.equal(result.status,0);assert.equal(install.nodedir,f.prefix);assert.equal(install.legacy,undefined);
 assert.deepEqual(install.args,['--prefix',f.home,'ci','--no-audit','--no-fund']);
 assert.equal(process.env.npm_package_config_node_gyp_nodedir,undefined);
});
test('mismatched and incomplete installed headers refuse before dependency lifecycle starts',async t=>{
 for(const broken of ['version','missing']){
  const f=fixture(t);
  if(broken==='version')fs.writeFileSync(path.join(f.include,'node_version.h'),f.header('0.0.0'));
  else fs.unlinkSync(path.join(f.include,'config.gypi'));
  const {result,install}=await run(f,{INSTALL_NODE_DEPENDENCIES_USE_NODE_HEADERS:'1'});
  assert.notEqual(result.status,0);assert.equal(install,undefined);
 }
});
test('effective caller configuration and Electron targets retain their existing authority',async t=>{
 for(const config of [{nodedir:'/caller/headers'},{runtime:'electron',target:'42.0.0'},{'dist-url':'https://example.invalid/headers'}]){
  const f=fixture(t);fs.unlinkSync(path.join(f.include,'node.h'));
  const {result,install}=await run(f,{INSTALL_NODE_DEPENDENCIES_USE_NODE_HEADERS:'1',FAKE_CONFIG:JSON.stringify(config)});
  assert.equal(result.status,0);assert.equal(install.nodedir,undefined);
 }
 for(const key of ['npm_config_nodedir','npm_package_config_node_gyp_nodedir','NPM_PACKAGE_CONFIG_NODE_GYP_NODEDIR']){
  const f=fixture(t),{result,install}=await run(f,{INSTALL_NODE_DEPENDENCIES_USE_NODE_HEADERS:'1',[key]:'/caller/headers'});
  assert.equal(result.status,0);assert.equal(key==='npm_config_nodedir'?install.legacy:key==='npm_package_config_node_gyp_nodedir'?install.nodedir:install.uppercase,'/caller/headers');
 }
});
test('host header reuse preserves reviewed, unreviewed and install-script diagnostics without retry',async t=>{
 for(const [diagnostic,status,expected]of [
  ['npm warn deprecated glob@10.5.0: reviewed upstream warning\n',0,'::notice title='],
  ['npm warn deprecated unfamiliar-package@1.0.0: unreviewed\n',1,'Unexpected npm stderr'],
  ['npm warn allow-scripts   example-native@1.0.0 (install: node-gyp rebuild)\n',1,'Unreviewed npm install scripts'],
  ['npm error genuine failure\n',1,'npm error genuine failure'],
 ]){
  const f=fixture(t),{result,calls}=await run(f,{INSTALL_NODE_DEPENDENCIES_USE_NODE_HEADERS:'1',FAKE_DIAGNOSTIC:diagnostic,FAKE_STATUS:diagnostic.startsWith('npm error')?'17':'0'});
  assert.equal(result.status,status);assert((result.stdout.toString()+result.stderr.toString()).includes(expected));
  assert.equal(calls.filter(call=>call.args.includes('install')).length,1);
 }
});
test('desktop enables host-header selection only in the install step on macOS',()=>{
 const workflow=fs.readFileSync(path.join(root,'.github/workflows/desktop-release.yml'),'utf8');
 const start=workflow.indexOf('- name: Install desktop dependencies');
 const end=workflow.indexOf('- name: Prefetch electron-builder toolsets',start);
 const step=workflow.slice(start,end);
 assert(step.includes("INSTALL_NODE_DEPENDENCIES_USE_NODE_HEADERS: \"${{ runner.os == 'macOS' && '1' || '0' }}\""));
 assert.equal(workflow.split('INSTALL_NODE_DEPENDENCIES_USE_NODE_HEADERS').length,2);
 const installer=fs.readFileSync(path.join(root,'scripts/install-node-dependencies.sh'),'utf8');
 assert.doesNotMatch(installer,/--ignore-scripts|GITHUB_ENV/u);
 assert(installer.includes('npm_package_config_node_gyp_nodedir=$host_node_headers'));
});
