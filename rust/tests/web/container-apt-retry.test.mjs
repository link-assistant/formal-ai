import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,writeFileSync,mkdirSync,copyFileSync,chmodSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

const docker=readFileSync('Dockerfile','utf8');
const helpers=['apt-install-with-retry.sh','run-with-deadline.sh'];
function run(failures,installStatus=0){
 const directory=mkdtempSync(join(tmpdir(),'container-apt-retry-'));
 try{
  const scripts=join(directory,'helpers'),bin=join(directory,'bin');
  mkdirSync(scripts);mkdirSync(bin);
  for(const name of helpers){copyFileSync(resolve('scripts',name),join(scripts,name));chmodSync(join(scripts,name),0o755);}
  const calls=join(directory,'calls.jsonl');
  const fake=join(bin,'apt-get');
  writeFileSync(fake,'#!'+process.execPath+'\n'+[
   "import fs from 'node:fs';",
   "const args=process.argv.slice(2);",
   "fs.appendFileSync(process.env.FAKE_CALLS,JSON.stringify({kind:'apt',args})+'\\n');",
   "const updates=fs.readFileSync(process.env.FAKE_CALLS,'utf8').trim().split('\\n').map(JSON.parse).filter(row=>row.kind==='apt'&&row.args.includes('update')).length;",
   "if(args.includes('update')&&updates<=Number(process.env.FAKE_FAILURES)){console.error('E: File has unexpected size. Mirror sync in progress?');process.exitCode=100;}",
   "if(args.includes('install'))process.exitCode=Number(process.env.FAKE_INSTALL_STATUS);",
  ].join('\n'),{mode:0o755});
  writeFileSync(join(bin,'rm'),'#!'+process.execPath+'\n'+[
   "import fs from 'node:fs';",
   "fs.appendFileSync(process.env.FAKE_CALLS,JSON.stringify({kind:'cleanup',args:process.argv.slice(2)})+'\\n');",
  ].join('\n'),{mode:0o755});
  const start=docker.indexOf('RUN FORMAL_AI_APT_PRIVILEGE=');
  assert(start>docker.indexOf('FROM konard/box-dind:2.10.2'));
  const end=docker.indexOf('\n\n',start);
  const command=docker.slice(start+4,end).replaceAll('/usr/local/lib/formal-ai-build',scripts);
  const result=spawnSync('bash',['-c',command],{cwd:directory,encoding:'utf8',timeout:15000,
   env:{...process.env,PATH:bin+':'+process.env.PATH,FORMAL_AI_APT_GET:fake,
    FORMAL_AI_APT_ATTEMPTS:'3',FORMAL_AI_APT_ATTEMPT_SECONDS:'3',FORMAL_AI_APT_RETRY_DELAY_SECONDS:'0',
    FAKE_CALLS:calls,FAKE_FAILURES:String(failures),FAKE_INSTALL_STATUS:String(installStatus)}});
  assert.equal(result.signal,null,result.stderr);
  const records=readFileSync(calls,'utf8').trim().split('\n').map(JSON.parse);
  return {status:result.status,records,stderr:result.stderr};
 }finally{rmSync(directory,{recursive:true,force:true});}
}

test('actual Docker apt command retries transient index failure and installs the same libraries',()=>{
 const result=run(2);
 assert.equal(result.status,0,result.stderr);
 const updates=result.records.filter(row=>row.kind==='apt'&&row.args.includes('update'));
 const installs=result.records.filter(row=>row.kind==='apt'&&row.args.includes('install'));
 assert.equal(updates.length,3);
 assert.deepEqual(installs.map(row=>row.args),[['-o','DPkg::Lock::Timeout=60','install','-y','-q',
  '--no-install-recommends','libstdc++6','libatomic1']]);
 assert.equal(result.records.at(-1).kind,'cleanup');
});

test('permanent index integrity failure remains failure and cannot install or clean lists',()=>{
 const result=run(99);
 assert.equal(result.status,100);
 assert.equal(result.records.length,3);
 assert(result.records.every(row=>row.kind==='apt'&&row.args.includes('update')));
 assert.match(result.stderr,/failed every attempt/);
});

test('permanent installation failure retains exit status and retries only within the existing bound',()=>{
 const result=run(0,42);
 assert.equal(result.status,42);
 assert.equal(result.records.filter(row=>row.args.includes('update')).length,3);
 assert.equal(result.records.filter(row=>row.args.includes('install')).length,3);
 assert(result.records.every(row=>row.kind==='apt'));
});

test('retry integration preserves repositories, package verification and helper ownership',()=>{
 assert.match(docker,/COPY --chmod=0755 scripts\/apt-install-with-retry\.sh scripts\/run-with-deadline\.sh scripts\/prepare-ubuntu-apt-mirror\.sh \/usr\/local\/lib\/formal-ai-build\//);
 assert.doesNotMatch(docker,/allow-unauthenticated|AllowInsecure|AllowUnauthenticated|Check-Valid-Until=false/);
 assert.doesNotMatch(docker,/rm[^\n]*(?:sources\.list|sources\.list\.d)/);
 const wrapper=readFileSync('scripts/apt-install-with-retry.sh','utf8');
 assert(wrapper.includes('attempts="'+ '\x24' + '{FORMAL_AI_APT_ATTEMPTS:-3}"'));
 assert(wrapper.includes('attempt_seconds="'+ '\x24' + '{FORMAL_AI_APT_ATTEMPT_SECONDS:-90}"'));
});
