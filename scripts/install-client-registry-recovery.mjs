// One identical retry requires an official tarball 404 plus fresh availability evidence.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {performance} from 'node:perf_hooks';
import {boundedCommand} from '../js/server/bounded-command.mjs';
import {makeRedactor} from '../js/server/structured-log.mjs';
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const maximumOutput=4*1024*1024;
export function failedRegistryTarball(stdout,stderr){
  const text=Buffer.concat([Buffer.from(stdout),Buffer.from('\n'),Buffer.from(stderr)]).toString()
    .replace(/\x1b\[[0-9;]*m/gu,'');
  const lines=text.split(/\r?\n/u).map(line=>line.trim());
  const errors=lines.filter(line=>/^(?:error|fatal|panic|exception)\b/iu.test(line));
  if(errors.length!==1)return null;
  const found=/^error: GET (https:\/\/registry\.npmjs\.org\/\S+) - 404$/u.exec(errors[0]);
  if(!found)return null;
  let url;try{url=new URL(found[1]);}catch{return null;}
  if(url.origin!=='https://registry.npmjs.org'||url.username||url.password||url.search||url.hash)return null;
  if(found[1]!==url.href)return null;
  const match=/^\/(?:@([a-z0-9][a-z0-9._-]*)\/)?([a-z0-9][a-z0-9._-]*)\/-\/([^/]+)\.tgz$/u.exec(url.pathname);
  if(!match)return null;
  const [,scope,base,archive]=match;
  if(!archive.startsWith(base+'-'))return null;
  const version=archive.slice(base.length+1);
  if(!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?$/u.test(version))return null;
  return {name:scope?'@'+scope+'/'+base:base,version,tarball:url.href};
}
export function requireRegistryMetadata(candidate,metadata){
  assert.equal(metadata.name,candidate.name);assert.equal(metadata.version,candidate.version);
  assert.equal(metadata.dist?.tarball,candidate.tarball);
  assert.match(metadata.dist?.integrity??'',/^sha512-[A-Za-z0-9+/]{86}==$/u);
  const value=metadata.dist.integrity.slice(7),bytes=Buffer.from(value,'base64');
  assert.equal(bytes.length,64);assert.equal(bytes.toString('base64'),value);
}
export async function recoverClientInstall(file,args,options={},injected={}){
  assert.equal(file,'bun');assert.equal(args.length,4);
  assert.deepEqual(args.slice(0,3),['add','-g','--ignore-scripts']);
  assert.match(args[3],/^(?:@[a-z0-9._-]+\/)?[a-z0-9._-]+@\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u);
  const seconds=options.seconds??600;
  assert(Number.isSafeInteger(seconds)&&seconds>0&&seconds<=600);
  const env=options.env??process.env,cwd=options.cwd??process.cwd(),redact=makeRedactor(env);
  const ops={command:boundedCommand,fetch,now:()=>performance.now(),
    output:(stream,bytes)=>process[stream].write(bytes),metadata:value=>console.error(JSON.stringify(value)),...injected};
  const deadline=ops.now()+seconds*1000;
  const remaining=limit=>{const left=deadline-ops.now();assert(left>0,'ClientInstallDeadline');return Math.max(1,Math.min(limit,left));};
  const metadata=(phase,extra={})=>ops.metadata({schema:'client-registry-recovery/v1',phase,...extra});
  async function attempt(index){
    const started=ops.now();metadata('started',{attempt:index});
    const result=await ops.command(file,args,{cwd,env,timeout:remaining(600000),
      stdoutLimitBytes:maximumOutput,stderrLimitBytes:maximumOutput});
    const stdout=Buffer.from(result.stdout??''),stderr=Buffer.from(result.stderr??'');
    // Full bounded output is replayed after completion, not advertised as live streaming.
    ops.output('stdout',String(redact(stdout.toString())));ops.output('stderr',String(redact(stderr.toString())));
    metadata('finished',{attempt:index,status:result.status,signal:result.signal,timedOut:result.timed_out===true,
      cleanupComplete:result.cleanup_complete===true,elapsedMs:ops.now()-started,stdoutBytes:stdout.length,stderrBytes:stderr.length,
      stdoutSha256:digest(stdout),stderrSha256:digest(stderr)});
    return result;
  }
  const clean=result=>result.error===undefined&&result.signal===null&&result.timed_out!==true&&result.cleanup_complete===true;
  const first=await attempt(1);
  if(!clean(first)||first.status!==1)return first;
  const candidate=failedRegistryTarball(first.stdout,first.stderr);if(candidate===null)return first;
  let probe='metadata',observedStatus=null;
  try{
    const url='https://registry.npmjs.org/'+encodeURIComponent(candidate.name)+'/'+encodeURIComponent(candidate.version);
    metadata('metadata-started',{name:candidate.name,version:candidate.version});
    const response=await ops.fetch(url,{redirect:'error',signal:AbortSignal.timeout(remaining(15000)),headers:{Accept:'application/json'}});
    let bytes;
    try{
      observedStatus=response.status;assert.equal(response.status,200);if(response.url)assert.equal(response.url,url);
      const chunks=[];let size=0;
      for await(const chunk of response.body){size+=chunk.length;assert(size<=1048576,'RegistryMetadataLimit');chunks.push(chunk);}
      bytes=Buffer.concat(chunks);const record=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(bytes));
      requireRegistryMetadata(candidate,record);
    }finally{await response.body?.cancel?.();}
    metadata('metadata-qualified',{name:candidate.name,version:candidate.version,bytes:bytes.length,sha256:digest(bytes)});
    probe='head';observedStatus=null;
    const head=await ops.fetch(candidate.tarball,{method:'HEAD',redirect:'error',signal:AbortSignal.timeout(remaining(15000))});
    try{
      observedStatus=head.status;assert.equal(head.status,200);if(head.url)assert.equal(head.url,candidate.tarball);
    }finally{await head.body?.cancel?.();}
    metadata('head-qualified',{name:candidate.name,version:candidate.version,status:head.status});
  }catch(error){
    const code=typeof error.code==='string'&&/^[A-Z0-9_]{1,64}$/u.test(error.code)?error.code:null;
    metadata('recovery-refused',{probe,status:observedStatus,errorCode:code});return first;
  }
  // Bun still authenticates downloaded package integrity. Metadata/HEAD only permit this one retry.
  return attempt(2);
}
export async function main(argv=process.argv.slice(2)){
  const [file,...args]=argv;const result=await recoverClientInstall(file,args);
  process.exitCode=result.error||result.signal||result.timed_out||result.cleanup_complete!==true?1:result.status??1;
  return result;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href)main().catch(error=>{
  console.error(error.code??'ClientInstallRecoveryRefused');process.exitCode=1;
});
