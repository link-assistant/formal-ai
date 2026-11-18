// Install only the selected release compiler; retry transient transport failures once.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {pathToFileURL} from 'node:url';
import {setTimeout as sleep} from 'node:timers/promises';
import {boundedCommand} from '../js/server/bounded-command.mjs';
export function transientRustupFailure(text){
 return !/checksum|signature|not available|does not exist|HTTP[^\n]*4[0-9]{2}/iu.test(text)
  && /dns error|failed to lookup address|could not resolve|temporary failure in name resolution|connection reset|connection refused|connection timed out|HTTP[^\n]*50[234]/iu.test(text);
}
export async function installSelectedToolchain({toolchain,target,run=boundedCommand,delay=sleep,environment=process.env,report=console.log}){
 assert.match(toolchain??'',/^[0-9]+\.[0-9]+\.[0-9]+$/u);
 assert.match(target??'',/^[a-z0-9_]+(?:-[a-z0-9_]+){2,3}$/u);
 const deadline=Date.now()+280000,env={...environment,RUSTUP_TOOLCHAIN:toolchain,RUSTUP_PERMIT_COPY_RENAME:'1'};
 let calls=0,installAttempts=0;
 async function invoke(program,args,limit){
  assert(++calls<=4&&Date.now()<deadline,'selected toolchain command budget');
  const result=await run(program,args,{env,timeout:Math.min(limit,deadline-Date.now()),stdoutLimitBytes:65536,stderrLimitBytes:65536});
  const output=Buffer.from(result.stdout??'').toString(),error=Buffer.from(result.stderr??'').toString();
  report({program,args,call:calls,status:result.status,cleanupComplete:result.cleanup_complete===true,stdout:output,stderr:error});
  return {result,output,error};
 }
 for(let attempt=1;attempt<=2;attempt++){
  installAttempts=attempt;
  const observed=await invoke('rustup',['toolchain','install',toolchain,'--profile','minimal','--target',target,'--no-self-update'],120000);
  const {result}=observed;
  if(result.status===0&&!result.error&&!result.signal&&!result.timed_out&&result.cleanup_complete===true)break;
  const retry=attempt===1&&!result.error&&!result.signal&&!result.timed_out&&result.cleanup_complete===true&&transientRustupFailure(observed.error+'\n'+observed.output);
  report({installAttempt:attempt,maximumAttempts:2,retrying:retry});
  if(!retry)throw Error('Selected Rust toolchain installation failed; no further attempts');
  await delay(1000);
 }
 const selected=await invoke('rustup',['default',toolchain],15000);
 assert(selected.result.status===0&&!selected.result.error&&!selected.result.signal&&!selected.result.timed_out&&selected.result.cleanup_complete===true,'selected Rust default failed');
 const compiler=await invoke('rustc',['--version','--verbose'],15000);
 assert(compiler.result.status===0&&!compiler.result.error&&!compiler.result.signal&&!compiler.result.timed_out&&compiler.result.cleanup_complete===true,'selected Rust compiler identity failed');
 assert.equal(/^release: (.+)$/mu.exec(compiler.output)?.[1],toolchain,'selected Rust release mismatch');
 assert.match(compiler.output,/^commit-hash: [a-f0-9]{40}$/mu);
 return {toolchain,target,installAttempts,calls,verifiedCompiler:true,compiled:false};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 assert.equal(process.env.GITHUB_ACTIONS,'true','Rust toolchain setup runs in CI only');
 const result=await installSelectedToolchain({toolchain:process.argv[2],target:process.argv[3]});
 assert(process.env.GITHUB_ENV,'GitHub environment file required');
 fs.appendFileSync(process.env.GITHUB_ENV,'RUSTUP_TOOLCHAIN='+result.toolchain+'\nCARGO_INCREMENTAL=0\nCARGO_TERM_COLOR=always\n');
 console.log(JSON.stringify(result));
}
