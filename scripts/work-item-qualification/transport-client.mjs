#!/usr/bin/env node
// DRAFT: reviewed transport only. Current Hive/clients and task text are not rewritten.
import fs from 'node:fs';
import {ownedPushRequest} from './push-policy.mjs';
import path from 'node:path';
import {randomUUID} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const root='/work-item-transport', rpc=root+'/rpc';
const policy=JSON.parse(fs.readFileSync(root+'/policy.json'));
const kind=path.basename(process.argv[1]), args=process.argv.slice(2);
function request(body){
  const id=randomUUID(), input=rpc+'/'+id+'.request.json', output=rpc+'/'+id+'.response.json';
  const temporary=input+'.tmp';
  fs.writeFileSync(temporary,JSON.stringify(body),{flag:'wx',mode:0o666});fs.renameSync(temporary,input);
  const wait=new Int32Array(new SharedArrayBuffer(4)), deadline=Date.now()+55000;
  while(!fs.existsSync(output)){
    if(Date.now()>=deadline)throw Error('bounded hosted transport RPC expired; do not retry mutation');
    Atomics.wait(wait,0,0,20);
  }
  const response=JSON.parse(fs.readFileSync(output));
  if(response.stdout)process.stdout.write(Buffer.from(response.stdout,'base64'));
  if(response.stderr)process.stderr.write(Buffer.from(response.stderr,'base64'));
  return response.status;
}
if(kind==='gh'){
  // Actual credential never enters the solver container; auth proof stays host-side.
  if(args[0]==='auth'&&args[1]==='token'){process.stdout.write('hosted-transport-placeholder\n');process.exit(0);}
  const transferred=[...args],bodyFile=transferred.indexOf('--body-file');
  if(bodyFile>=0){
    const file=transferred[bodyFile+1];if(file==='-')throw Error('stdin body-file is not supported by this bounded transport');
    const stat=fs.lstatSync(file);if(!stat.isFile()||stat.isSymbolicLink()||stat.size>65536)throw Error('bounded local body file required');
    transferred.splice(bodyFile,2,'--body',fs.readFileSync(file,'utf8'));
  }
  process.exit(request({kind:'gh',args:transferred,cwd:process.cwd()}));
}
if(kind!=='git')throw Error('unknown transport executable');
// Every fetch/push/clone sees only the snapshot mirror, which has no answer history.
let index=0;
while(index<args.length){
  if(['-C','-c','--git-dir','--work-tree','--namespace'].includes(args[index])){index+=2;continue;}
  if(args[index].startsWith('-')){index++;continue;}
  break;
}
const command=args[index];
if(command==='push'){
  const requested=ownedPushRequest(args.slice(index+1),policy);
  if(args.some(value=>(/^https:\/\//u.test(value)||/^git@/u.test(value))
    &&value.replace(/\.git$/u,'')!=='https://github.com/'+policy.repository
    &&value.replace(/\.git$/u,'')!=='git@github.com:'+policy.repository))throw Error('foreign push refused');
  // Never let host Git read candidate-controlled config/hooks. Transfer Git objects as a bundle only.
  const branch=spawnSync('/usr/bin/git',[...args.slice(0,index),'branch','--show-current'],{encoding:'utf8',timeout:5000});
  const name=branch.stdout?.trim();
  if(!new RegExp('^issue-'+policy.issueNumber+'-[A-Za-z0-9-]+$').test(name??''))throw Error('only owned issue branch may push');
  if(requested.branch!==null&&requested.branch!=='HEAD'&&requested.branch!==name)throw Error('PushBranchMismatch');
  const bundle='push-'+randomUUID()+'.bundle', file=root+'/bundles/'+bundle;
  const created=spawnSync('/usr/bin/git',[...args.slice(0,index),'bundle','create',file,'refs/heads/'+name],
    {encoding:null,timeout:30000,maxBuffer:1024*1024});
  if(created.status!==0){if(created.stderr)process.stderr.write(created.stderr);process.exit(created.status??1);}
  const status=request({kind:'push-bundle',bundle,branch:name});
  if(status===0&&(args.includes('-u')||args.includes('--set-upstream'))){
    for(const [key,value] of [['branch.'+name+'.remote','origin'],['branch.'+name+'.merge','refs/heads/'+name]])
      spawnSync('/usr/bin/git',[...args.slice(0,index),'config',key,value],{timeout:5000});
  }
  process.exit(status);
}
const network=['clone','fetch','pull','push','ls-remote','remote'].includes(command);
const configureRemote=command==='remote'&&['set-url','add'].includes(args[index+1]);
const rewritten=args.map(value=>/^https:\/\/github\.com\//u.test(value)||/^git@github\.com:/u.test(value)
  ? (value.replace(/\.git$/u,'')==='https://github.com/'+policy.repository||value.replace(/\.git$/u,'')==='git@github.com:'+policy.repository
    ? configureRemote?value:root+'/mirror.git' : (()=>{throw Error('transport denied foreign Git repository');})()) : value);
const result=spawnSync('/usr/bin/git',[
  ...(network?['-c','remote.origin.url='+root+'/mirror.git','-c','remote.origin.pushurl='+root+'/mirror.git']:[]),
  ...rewritten],{encoding:null,timeout:45000,maxBuffer:16*1024*1024});
if(result.stdout)process.stdout.write(result.stdout);if(result.stderr)process.stderr.write(result.stderr);
if(result.status!==0)process.exit(result.status??1);
if(command==='clone'){
  // Preserve the ordinary repository URL for Hive's origin parser; subsequent network operations are redirected above.
  const destination=rewritten.at(-1);
  const candidate=path.resolve(destination);
  if(fs.existsSync(candidate+'/.git'))spawnSync('/usr/bin/git',['-C',candidate,'remote','set-url','origin','https://github.com/'+policy.repository+'.git']);
}
process.exit(0);
