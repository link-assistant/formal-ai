import assert from 'node:assert/strict';
import fs from 'node:fs';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {decodeTyped} from '../../../../js/server/structured-log.mjs';

export const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));

export function temporaryHome(t){
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-structured-log-'));
  t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
  return directory;
}

// Preserve explicit Node import instrumentation when tests launch a CLI child.
// The normal repository test command has no preload; do not inherit NODE_OPTIONS.
function importFlags(){
  const flags=[];
  for(let i=0;i<process.execArgv.length;i++){
    if(process.execArgv[i]==='--import')flags.push('--import',process.execArgv[++i]);
    else if(process.execArgv[i].startsWith('--import='))flags.push(process.execArgv[i]);
  }
  return flags;
}

async function availablePort(){
  const server=net.createServer();
  await new Promise((resolve,reject)=>{
    server.once('error',reject);
    server.listen(0,'127.0.0.1',resolve);
  });
  const port=server.address().port;
  await new Promise(resolve=>server.close(resolve));
  return port;
}

export async function waitFor(read,{timeout=2000}={}){
  const until=Date.now()+timeout;
  while(Date.now()<until){
    const value=read();
    if(value)return value;
    await pause(20);
  }
  throw Error('fixture_observation_timeout');
}

export async function startCli(t,{args=[],env={}}={}){
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-structured-cli-'));
  const home=path.join(directory,'home');
  const privateEnv={PATH:'',HOME:home,FORMAL_AI_DREAMING:'false',FORMAL_AI_RECORD_CHAT:'0'};
  for(const [key,name]of Object.entries({
    XDG_CONFIG_HOME:'config',XDG_CACHE_HOME:'cache',XDG_DATA_HOME:'data',
    XDG_STATE_HOME:'state',TMPDIR:'tmp',
  })){
    privateEnv[key]=path.join(directory,name);
    fs.mkdirSync(privateEnv[key],{recursive:true});
  }
  fs.mkdirSync(home);
  privateEnv.FORMAL_AI_MEMORY_PATH=path.join(home,'memory.lino');
  Object.assign(privateEnv,env);
  const port=await availablePort();
  const entry=fileURLToPath(new URL('../../../../js/server/main.mjs',import.meta.url));
  const extra=typeof args==='function'?args(directory):args;
  const child=spawn(process.execPath,[...importFlags(),entry,'--port',String(port),...extra],{
    cwd:home,env:privateEnv,stdio:['ignore','pipe','pipe'],
  });
  let stdout='',stderr='',closed=false,code=null,signal=null,spawnError=null;
  const collect=(name,bytes)=>{
    if(name==='stdout')stdout+=bytes;else stderr+=bytes;
    if(stdout.length+stderr.length>4*1024*1024)child.kill('SIGKILL');
  };
  child.stdout.on('data',bytes=>collect('stdout',bytes));
  child.stderr.on('data',bytes=>collect('stderr',bytes));
  child.once('error',error=>{spawnError=error;});
  const done=new Promise(resolve=>child.once('close',(status,why)=>{
    code=status;signal=why;closed=true;resolve();
  }));
  const deadline=setTimeout(()=>child.kill('SIGKILL'),15_000);
  deadline.unref();
  async function stop(){
    if(!closed)child.kill('SIGTERM');
    await Promise.race([done,pause(1000)]);
    if(!closed)child.kill('SIGKILL');
    await Promise.race([done,pause(1000)]);
    clearTimeout(deadline);
    assert.equal(closed,true,'CLI child terminates within its bounded cleanup');
  }
  t.after(async()=>{
    try{await stop();}finally{if(closed)fs.rmSync(directory,{recursive:true,force:true});}
  });
  const run={
    directory,home,env:privateEnv,port,url:'http://127.0.0.1:'+port,pid:child.pid,stop,
    get stdout(){return stdout;},get stderr(){return stderr;},
    get code(){return code;},get signal(){return signal;},get closed(){return closed;},
    records(){
      return stderr.split('\n').slice(0,-1).filter(Boolean).map(decodeTyped);
    },
    legacy(){
      const directory=path.join(home,'dialog-logs');
      return fs.existsSync(directory)?fs.readdirSync(directory)
        .filter(file=>file.endsWith('.jsonl')||file.endsWith('.lino'))
        .map(file=>fs.readFileSync(path.join(directory,file),'utf8')).join(''):'';
    },
    async ready(){
      const until=Date.now()+8000;
      while(Date.now()<until){
        if(closed||spawnError)throw Error('fixture_cli_early_exit');
        try{
          const response=await fetch(run.url+'/api/hello',{signal:AbortSignal.timeout(300)});
          if(response.status===200){
            assert.equal(response.headers.get('content-type'),'application/json');
            assert.equal(await response.text(),'{\n  "message": "hello"\n}');
            return;
          }
        }catch(error){if(error.code==='ERR_ASSERTION')throw error;}
        await pause(20);
      }
      throw Error('fixture_cli_start_timeout');
    },
  };
  return run;
}

export async function debug(run,token,action='session'){
  const response=await fetch(run.url+'/v1/debug/'+action,{
    method:'POST',headers:{'content-type':'application/json'},
    body:JSON.stringify({token}),signal:AbortSignal.timeout(2000),
  });
  return {status:response.status,body:await response.json()};
}
