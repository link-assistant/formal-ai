#!/usr/bin/env node
// Hosted acceptance uses the exact public issues and one persistent task checkout.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {boundedCommand} from '../js/server/bounded-command.mjs';
import {WorkerHost} from '../js/server/worker-host.mjs';
import {installNodeHost} from '../js/agentic/node-host.mjs';
import {loadConversationContext} from '../js/server/conversations.mjs';
import {requireHarnessDelivery, requireHistoricalReadyPullRequest, requireProgramMember} from './lib/hive-delivery-evidence.mjs';
import {redact} from './collect-ci-failures.mjs';
import {createPublicEvidenceExport} from './lib/issue-1364-public-evidence.mjs';
import {replaySchema, contentDigest, runtimeImageDigest, requireReplayTask, adaptReplayEntrypoint,
  taskSnapshot, requireNoopTask, requireActualTaskCi, requireTaskObligations,
  canonicalCapturedMessages, auditCapturedActions} from './lib/issue-1364-replay-contract.mjs';
const optionNames = ['--tool','--source','--image','--output'];
function argumentsOf(values) {
  assert.equal(values.length, 8);
  const result = {};
  for (let index=0; index<values.length; index+=2) {
    assert(optionNames.includes(values[index]) && !Object.hasOwn(result,values[index]));
    result[values[index]]=values[index+1];
  }
  return result;
}
export async function runIssue1364Replay(options, environment=process.env) {
  assert.equal(environment.GITHUB_ACTIONS,'true','real acceptance is hosted-only');
  const tool=options['--tool'], source=options['--source'], image=options['--image'];
  assert(['claude','codex','agent'].includes(tool)); assert.match(source,/^[a-f0-9]{40}$/u);
  const imageDigest=runtimeImageDigest(image);
  const root=process.cwd(), hive=path.join(root,'.hive-mind');
  const logRoot=environment.E2E_LOG_DIR, output=path.resolve(options['--output']);
  assert(logRoot && path.isAbsolute(logRoot) && path.dirname(output)===path.resolve(logRoot));
  assert.equal(fs.realpathSync(logRoot),path.resolve(logRoot));assert(!fs.existsSync(output));
  const manifestBytes=fs.readFileSync(path.join(root,'data/benchmarks/issue-1364-client-replay-inputs.json'));
  const inputSha256=contentDigest(manifestBytes), inputs=JSON.parse(manifestBytes);
  const task=inputs.clients[tool];assert(task);
  const repository=new URL(task.issueUrl).pathname.split('/').slice(1,3).join('/');
  const requestEnvironment={...environment};
  const observations=[];
  async function execute(file,args,{cwd=root,timeout=30000,env=requestEnvironment,captureDirectory=null}={}) {
    const result=await boundedCommand(file,args,{cwd,env,timeout,maxBuffer:16*1024*1024});
    if(captureDirectory){
      fs.writeFileSync(path.join(captureDirectory,'wrapper-stdout.log'),result.stdout,{flag:'wx'});
      fs.writeFileSync(path.join(captureDirectory,'wrapper-stderr.log'),result.stderr,{flag:'wx'});
    }
    observations.push({program:file,arguments:args,exitCode:result.status,signal:result.signal,
      timedOut:result.timed_out,cleanupComplete:result.cleanup_complete,
      stdoutSha256:contentDigest(result.stdout),stderrSha256:contentDigest(result.stderr)});
    assert.equal(result.error,undefined,String(result.error));assert.equal(result.cleanup_complete,true);
    assert.equal(result.signal,null);assert.equal(result.status,0,redact(result.stderr.toString()).slice(-4000));
    return result.stdout.toString();
  }
  const api=async endpoint=>JSON.parse(await execute('gh',['api','repos/'+repository+'/'+endpoint]));
  const pr=await api('pulls/2'), issue=await api('issues/1'); requireReplayTask(inputs,tool,issue,pr);
  const identity=JSON.parse(fs.readFileSync(path.join(logRoot,'source-binary-identity.json')));
  const bundle=JSON.parse(fs.readFileSync(path.join(environment.HIVE_IMAGE_BUNDLE_DIR,'image-bundle.json')));
  assert.equal(identity.source,source);assert.equal(identity.binarySha256,identity.actualTaskImageBinarySha256);
  assert.deepEqual(bundle.identity,identity);
  if(identity.publishedRuntime) {
    assert.equal(identity.publishedRuntime.image,image);assert.equal(identity.publishedRuntime.source,source);
    assert.equal(identity.publishedRuntime.binarySha256,identity.binarySha256);
  } else assert.equal(image,bundle.imageId,'candidate proof must use the actual task-layer content ID');
  const taskImage=JSON.parse(await execute('docker',['image','inspect',bundle.imageId]));
  assert.equal(taskImage[0].Id,bundle.imageId);
  const actualBinary=(await execute('docker',['run','--rm','--entrypoint','sha256sum',bundle.imageId,
    '/usr/local/bin/formal-ai'])).trim().split(/\s+/u)[0]; assert.equal(actualBinary,identity.binarySha256);
  const transport=JSON.parse(fs.readFileSync(path.join(hive,'.formal-ai-transport-adapter.json')));
  const original=fs.readFileSync(path.join(hive,'scripts/e2e-hello-world.mjs'),'utf8');
  assert.equal(contentDigest(original),transport.adaptedSha256,'reviewed pinned wrapper identity');
  const owned=fs.mkdtempSync(path.join(environment.RUNNER_TEMP || os.tmpdir(),'formal-ai-1364-owned-'));
  const workspace=path.join(owned,'workspace');fs.mkdirSync(workspace,{mode:0o777});
  fs.chmodSync(owned,0o777);fs.chmodSync(workspace,0o777);
  const publication=createPublicEvidenceExport(path.join(logRoot,'publish'),environment);
  publication.originalIdentity(fs.readFileSync(path.join(logRoot,'source-binary-identity.json')));
  const wrapper=path.join(hive,'scripts/e2e-issue-1364.mjs');
  fs.writeFileSync(wrapper,adaptReplayEntrypoint(original,task,workspace),{flag:'wx',mode:0o600});
  const receipt={schema:replaySchema,inputSha256,tool,source,imageDigest,taskPRhead:null,
    firstPass:null,secondPass:null,complete:false};
  const save=()=>fs.writeFileSync(output,JSON.stringify(receipt,null,2)+'\n');
  await installNodeHost(new WorkerHost());
  async function checkoutObservation() {
    const git=async args=>(await execute('git',['-c','safe.directory='+workspace,...args],{cwd:workspace}));
    const head=(await git(['rev-parse','HEAD'])).trim();
    const branch=(await git(['branch','--show-current'])).trim();assert.equal(branch,task.branch);
    const status=(await git(['status','--porcelain','--untracked-files=all'])).trim();assert.equal(status,'');
    const paths=(await git(['ls-files','-z'])).split('\0').filter(Boolean);
    const commitRecords=(await git(['reflog','show','--all','--format=%gs'])).split('\n')
      .filter(line=>/^commit(?:\s|:|\()/u.test(line)).length;
    return {head,branch,status,commitRecords,files:taskSnapshot(workspace,paths,task.issueUrl)};
  }
  async function traceObservation(directory) {
    const location=path.join(directory,'formal-ai-dialog');assert(fs.lstatSync(location).isDirectory());
    const entries=fs.readdirSync(location);assert(entries.length<=256);
    let bytes=0;const ids=new Set();
    for(const name of entries) {
      const stat=fs.lstatSync(path.join(location,name));assert(stat.isFile()&&!stat.isSymbolicLink());
      bytes+=stat.size;assert(bytes<=64*1024*1024&&stat.size<=32*1024*1024);
      const id=name.replace(/(?:\.conversation)?\.(?:lino|jsonl)$/u,'');assert(/^[A-Za-z0-9_-]+$/u.test(id));ids.add(id);
    }
    const groups=[], failures=new Map(), publicFailures=new Map();
    for(const id of ids) {
      const context=loadConversationContext(id,{...environment,FORMAL_AI_DIALOG_LOG_DIR:location});
      if(!context.server_logs.some(row=>row.method==='POST'&&row.request_body))continue;
      const messages=canonicalCapturedMessages(context);
      if(messages.some(row=>row.role==='assistant'&&row.tool_calls?.length)){
        const actual=auditCapturedActions(messages,failures);
        const published=auditCapturedActions(canonicalCapturedMessages(publication.safe(context)),publicFailures);
        assert.deepEqual(published,actual,'redaction changed the observed retry/write oracle');groups.push(actual);
      }
    }
    assert(groups.length>0,'complete source-owned tool traces required');
    return {groups:groups.length,calls:groups.reduce((sum,row)=>sum+row.calls,0),
      writes:groups.reduce((sum,row)=>sum+row.writes,0),maximumIdenticalFailures:Math.max(...groups.map(row=>row.maximumIdenticalFailures))};
  }
  const exportedPasses=new Set();
  function exportPass(index,directory){publication.collect(directory,'pass-'+index);exportedPasses.add(index);}
  async function onePass(index) {
    const directory=path.join(owned,'pass-'+index);fs.mkdirSync(directory,{mode:0o777});fs.chmodSync(directory,0o777);
    fs.writeFileSync(path.join(directory,'source-binary-identity.json'),JSON.stringify(identity,null,2)+'\n',{flag:'wx'});
    const env={...requestEnvironment,E2E_LOG_DIR:directory,E2E_IMAGE:bundle.imageId,E2E_TOOL:tool,E2E_KEEP:'true'};
    const stdout=await execute(process.execPath,[wrapper,'--tool',tool,'--model','formal-ai','--issue',task.issueUrl,'--keep'],
      {cwd:hive,env,timeout:index===1?540000:420000,captureDirectory:directory});
    fs.writeFileSync(path.join(directory,'runner-output-sanitized.log'),redact(stdout),{flag:'wx'});
    const originalResult=JSON.parse(fs.readFileSync(path.join(directory,'result.json')));
    requireHarnessDelivery(originalResult,tool);assert.equal(originalResult.issueUrl,task.issueUrl);
    assert.equal(originalResult.pullRequestUrl,task.pullRequestUrl);
    const current=await api('pulls/2');const head=requireHistoricalReadyPullRequest(current,repository,2,task.branch);
    const view=JSON.parse(await execute('gh',['pr','view',task.pullRequestUrl,'--json','statusCheckRollup']));
    requireActualTaskCi(view.statusCheckRollup);
    const observed=await checkoutObservation();assert.equal(observed.head,head,'all task changes must be pushed');
    const programPath=requireProgramMember(observed.files.map(file=>({filename:file.path})),{claude:'Kotlin',codex:'Rust',agent:'Scala'}[tool]);
    const tracked=observed.files.map(row=>row.path), workflows=tracked.filter(file=>/^\.github\/workflows\/[^/]+\.ya?ml$/u.test(file));
    assert.equal(workflows.length,1);
    const allObligations=requireTaskObligations({programPath,source:fs.readFileSync(path.join(workspace,programPath),'utf8'),
      workflow:fs.readFileSync(path.join(workspace,workflows[0]),'utf8'),verifier:fs.readFileSync(path.join(workspace,'tests/verify-output.sh'),'utf8')});
    const trace=await traceObservation(directory);
    assert.equal(requireHistoricalReadyPullRequest(await api('pulls/2'),repository,2,task.branch),head,'task PR changed during observation');
    return {directory,observed,head,trace,result:{exitCode:originalResult.solveExitCode,clean:true,
      verifierPassed:true,workflowCiGreen:true,allObligations}};
  }
  try {
    const first=await onePass(1);receipt.firstPass=first.result;receipt.taskPRhead=first.head;save();
    exportPass(1,first.directory);
    const second=await onePass(2);requireNoopTask(first.observed,second.observed);assert.equal(second.trace.writes,0,'second pass issued a source mutation tool');
    receipt.secondPass={exitCode:second.result.exitCode,clean:true,noWrite:true,noCommit:true};
    receipt.trace={first:first.trace,second:second.trace};
    exportPass(2,second.directory);
    const resultText=fs.readFileSync(path.join(second.directory,'result.json'),'utf8');
    publication.json('result.json',JSON.parse(resultText));
    publication.text('workflow.log',fs.readFileSync(path.join(second.directory,'workflow.log'),'utf8'));
    fs.writeFileSync(path.join(logRoot,'result.json'),JSON.stringify(publication.safe(JSON.parse(resultText)))+'\n');
    fs.writeFileSync(path.join(logRoot,'workflow.log'),String(publication.safe(fs.readFileSync(path.join(second.directory,'workflow.log'),'utf8'))));
    receipt.observationDigests=observations;receipt.complete=true;save();
    publication.json('issue-1364-delivery.json',receipt);publication.finish();
    console.log(JSON.stringify({schema:receipt.schema,tool,source,imageDigest:receipt.imageDigest,taskPRhead:receipt.taskPRhead,complete:true}));
    return receipt;
  } catch(error) {
    receipt.complete=false;receipt.retryDisposition='requires-effect-certainty-review';receipt.failure=redact(String(error.message));receipt.observationDigests=observations;
    receipt.missingPublicEvidence=[];
    for(const index of [1,2]){
      const directory=path.join(owned,'pass-'+index);
      if(!exportedPasses.has(index)&&fs.existsSync(directory)){
        try{exportPass(index,directory);}catch{receipt.missingPublicEvidence.push({pass:index,reason:'incomplete-or-unpublishable-text-capture'});}
      }
    }
    save();
    if(!fs.existsSync(path.join(logRoot,'publish/issue-1364-delivery.json')))publication.json('issue-1364-delivery.json',receipt);
    if(!fs.existsSync(path.join(logRoot,'publish/privacy-manifest.json')))publication.finish({omitted:receipt.missingPublicEvidence});
    throw error;
  } finally {
    try{
      if(process.getuid&&process.getgid)await execute('sudo',['chown','--recursive','--no-dereference',
        process.getuid()+':'+process.getgid(),'--',owned],{timeout:30000});
    }finally{fs.rmSync(owned,{recursive:true,force:true});fs.unlinkSync(wrapper);}
  }
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  await runIssue1364Replay(argumentsOf(process.argv.slice(2))).catch(error=>{console.error(redact(error.stack));process.exitCode=1;});
}
