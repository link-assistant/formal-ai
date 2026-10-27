import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import vm from 'node:vm';
import {spawnSync} from 'node:child_process';
import {adaptDraftDiagnostics,prepareDraftDiagnostics,HIVE_DRAFT_PIN} from '../../../scripts/lib/hive-draft-diagnostics.mjs';
import {prepareDraftEvidence} from '../../../scripts/prepare-hive-draft-evidence.mjs';
import {collectDraftJobLogs} from '../../../scripts/collect-hive-draft-logs.mjs';
import {captureDraftCommand} from '../../../scripts/capture-draft-command.mjs';
import {transferOperationalEvidenceOwner} from '../../../scripts/lib/hive-operational-evidence.mjs';
import {usefulDraftChanges} from '../../../scripts/hive-mind-draft-result.mjs';
const root=path.resolve(import.meta.dirname,'../../..');
function workspace(t){
 const directory=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-draft-logs-')));
 t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));return directory;
}
const publicRunner=fs.readFileSync(path.join(root,'rust/tests/fixtures/hive-draft-runner.mjs.txt'),'utf8');
test('pinned shared runner retains full payloads on the existing mounted directory without changing solve',t=>{
 const directory=workspace(t),hive=path.join(directory,'hive'),logs=path.join(directory,'logs');
 fs.mkdirSync(path.join(hive,'scripts'),{recursive:true});fs.writeFileSync(path.join(hive,'scripts/formal-ai-draft.mjs'),publicRunner);
 const report=prepareDraftDiagnostics({hive,logDirectory:logs}),adapted=fs.readFileSync(path.join(hive,'scripts/formal-ai-draft.mjs'),'utf8');
 assert.equal(report.upstreamCommit,HIVE_DRAFT_PIN);assert.equal(report.taskChanged,false);
 const start=adapted.indexOf('buildDockerArgv({'),end=adapted.indexOf('\n});',start);
 const args=vm.runInNewContext(adapted.slice(start,end+3),{
  buildDockerArgv:args=>args,buildSolveArgv:args=>args,issueUrlFor:()=> 'https://github.com/owner/repo/issues/1',
  issue:{},buildGitIdentityEnv:()=>({}),buildRunContextEnv:()=>({HIVE_MIND_EPHEMERAL_CONTAINER:'1'}),
  env:{},containerLogDir:'/home/box/logs',hostLogDir:logs,image:'pinned-image',extraMounts:[]});
 for(const key of ['FORMAL_AI_LOG_DIR','FORMAL_AI_DIALOG_LOG_DIR','FORMAL_AI_NATIVE_RESPONSE_CAPTURE_DIR'])
  assert(args.setEnv[key].startsWith('/home/box/logs/formal-ai-'));
 assert.equal(args.setEnv.FORMAL_AI_LOG_LEVEL,'trace');
 assert.equal(args.solveArgv.logDir,'/home/box/logs');assert.equal(args.image,'pinned-image');
 assert.throws(()=>adaptDraftDiagnostics(publicRunner+'\n// changed'),/unreviewed/);
});
test('complete console capture preserves middle output, stderr and the failing command status',t=>{
 const directory=workspace(t),payload='α😀'.repeat(20000)+'\nMIDDLE-ERROR\n'+'z'.repeat(100000)+'\n';
 const result=spawnSync('bash',['scripts/run-with-session-evidence.sh','10','attempt',process.execPath,'-e',
  'process.stdout.write("α😀".repeat(20000)+"\\nMIDDLE-ERROR\\n"+"z".repeat(100000)+"\\n");process.stderr.write("stderr failure\\n");process.exitCode=7;'],
  {cwd:root,env:{...process.env,FORMAL_AI_DRAFT_LOG_DIR:directory},encoding:'utf8',timeout:15000,maxBuffer:1048576});
 assert.equal(result.status,7,result.stderr);assert(!result.error);
 const capture=fs.readFileSync(path.join(directory,'attempt.log'),'utf8');
 const chunks=capture.trimEnd().split('\n').map(line=>JSON.parse(line));
 assert(Buffer.from(chunks.filter(row=>row.stream==='stdout').map(row=>row.content).join('')).equals(Buffer.from(payload)));
 assert.equal(chunks.filter(row=>row.stream==='stderr').map(row=>row.content).join(''),'stderr failure\n');
 assert(fs.readFileSync(path.join(directory,'attempt.stdout.log')).equals(Buffer.from(payload)));
 const receipt=JSON.parse(fs.readFileSync(path.join(directory,'attempt-outcome.json'),'utf8'));
 assert.equal(receipt.exitCode,7);assert.equal(receipt.captureExitCode,0);assert.equal(receipt.completeConsoleCapture,true);
});
test('deadline expiration leaves the partial session and explicit exit 124 receipt',t=>{
 const directory=workspace(t);
 const result=spawnSync('bash',['scripts/run-with-session-evidence.sh','0','attempt',process.execPath,'-e',
  'process.stdout.write("before deadline\\n");setInterval(()=>{},1000);'],
  {cwd:root,env:{...process.env,FORMAL_AI_DRAFT_LOG_DIR:directory,FORMAL_AI_DEADLINE_GRACE_SECONDS:'0'},
   encoding:'utf8',timeout:5000,maxBuffer:65536});
 assert.equal(result.status,124,result.stderr);assert(!result.error);
 const receipt=JSON.parse(fs.readFileSync(path.join(directory,'attempt-outcome.json'),'utf8'));
 assert.equal(receipt.deadlineExpired,true);assert.equal(receipt.state,'finished');
});
test('artifact staging preserves complete UTF-8 payloads while removing credentials and personal data',t=>{
 const directory=workspace(t),source=path.join(directory,'session'),destination=path.join(directory,'artifact');
 fs.mkdirSync(path.join(source,'formal-ai-native'),{recursive:true});fs.mkdirSync(path.join(source,'formal-ai-dialog'));
 const content='α😀'.repeat(20000)+'\nEXACT-MIDDLE\n'+'x'.repeat(200000)+'\n';
 fs.writeFileSync(path.join(source,'formal-ai-native/native.jsonl'),JSON.stringify({content})+'\n');
 fs.writeFileSync(path.join(source,'formal-ai-dialog/dialog.lino'),'actual dialog\n');
 fs.writeFileSync(path.join(source,'attempt.log'),'failure opaque-secret-value alice@example.invalid /Users/alice/private\n');
 fs.writeFileSync(path.join(source,'attempt-outcome.json'),JSON.stringify({state:'finished',captureExitCode:0,completeConsoleCapture:true}));
 fs.writeFileSync(path.join(source,'draft-diagnostics-adapter.json'),'{}\n');
 fs.writeFileSync(path.join(source,'capture-manifest.json'),'{"priorBuild":true}\n');
 const receipt=prepareDraftEvidence({source,destination,env:{GH_TOKEN:'opaque-secret-value',DRAFT_ATTEMPT_OUTCOME:'failure'}});
 assert.equal(receipt.complete,true);assert.equal(receipt.fullModelEvidence,true);
 assert.equal(JSON.parse(fs.readFileSync(path.join(destination,'formal-ai-native/native.jsonl'),'utf8')).content,content);
 assert.equal(fs.readFileSync(path.join(destination,'build-capture-manifest.json'),'utf8'),'{"priorBuild":true}\n');
 const safe=fs.readFileSync(path.join(destination,'attempt.log'),'utf8');
 assert(!safe.includes('opaque-secret-value'));assert(!safe.includes('alice'));assert.match(safe,/failure/);
 assert.equal(receipt.taskAcceptance,false);
});
test('links, byte budgets, missing files and interrupted capture are explicit incomplete evidence',t=>{
 const directory=workspace(t),source=path.join(directory,'session');fs.mkdirSync(source);
 fs.writeFileSync(path.join(directory,'private'),'MUST NOT COPY');fs.symlinkSync(path.join(directory,'private'),path.join(source,'linked.log'));
 fs.writeFileSync(path.join(source,'big.log'),'x'.repeat(1000));
 fs.writeFileSync(path.join(source,'attempt-outcome.json'),'{"state":"running"}\n');
 const destination=path.join(directory,'artifact'),receipt=prepareDraftEvidence({source,destination,maxFileBytes:100,
  env:{DRAFT_ATTEMPT_OUTCOME:'failure'}});
 assert.equal(receipt.complete,false);assert.equal(receipt.fullModelEvidence,false);
 assert(receipt.unavailable.some(row=>row.reason==='non_regular_or_linked_file'));
 assert(receipt.unavailable.some(row=>row.reason==='explicit_byte_budget_exhausted'));
 assert(receipt.missingRequired.includes('attempt.log'));assert.deepEqual(receipt.incompleteProcesses,['attempt-outcome.json']);
 assert(!fs.existsSync(path.join(destination,'linked.log')));assert(!fs.existsSync(path.join(destination,'big.log')));
});
function apiFixture({wrongHead=false,truncated=false}={}){
 const head='a'.repeat(40),calls=[],run={id:11,run_attempt:2,head_sha:wrongHead?'b'.repeat(40):head,
  repository:{full_name:'owner/repo'},path:'.github/workflows/hive-mind-draft.yml',event:'issues',status:'completed',conclusion:'cancelled'};
 const jobs=[{id:22,run_id:11,name:'draft',status:'completed',conclusion:'cancelled',steps:[
  {name:'Attempt',number:1,status:'completed',conclusion:'cancelled'}]},
  {id:23,run_id:11,name:'unstarted',status:'completed',conclusion:'skipped',steps:[]}];
 return {head,calls,transport:async(args,options)=>{
  calls.push({args,options});assert.equal(args[0],'api');assert(!args.includes('--method'));
  let text;if(args[1].endsWith('/attempts/2'))text=JSON.stringify(run);
  else if(args[1].includes('/jobs?'))text=JSON.stringify({total_count:jobs.length,jobs});
  else {assert(args[1].endsWith('/jobs/22/logs'));text='start\n'+'x'.repeat(80000)+'\nMIDDLE ROOT CAUSE\n'+'z'.repeat(80000)+'\nend\n';}
  return {ok:true,text,streamComplete:true,truncated:options.captureRaw&&truncated,bytesReceived:Buffer.byteLength(text)};
 }};
}
test('post-run collector retains complete cancelled-attempt logs with exact source and job identities',async t=>{
 const directory=workspace(t),fixture=apiFixture();
 const receipt=await collectDraftJobLogs({repository:'owner/repo',runId:11,attempt:2,head:fixture.head,
  output:path.join(directory,'artifact'),transport:fixture.transport});
 assert.equal(receipt.complete,true);assert.equal(receipt.modelRerun,false);assert.equal(receipt.jobs.length,2);
 assert.equal(receipt.jobs[1].log,'not-executed');assert.equal(fixture.calls.length,3);
 const log=fs.readFileSync(path.join(directory,'artifact/job-22.log'),'utf8');
 assert(log.startsWith('start\n'));assert(log.endsWith('end\n'));assert(log.includes('MIDDLE ROOT CAUSE'));
 assert(!log.includes('omitted'));assert.equal(receipt.jobs[0].log.bytes,Buffer.byteLength(log));
});
test('source mismatch refuses job fetching; truncated logs remain available with no completeness credit',async t=>{
 const directory=workspace(t);
 const bad=apiFixture({wrongHead:true}),a=await collectDraftJobLogs({repository:'owner/repo',runId:11,attempt:2,head:bad.head,
  output:path.join(directory,'bad'),transport:bad.transport});
 assert.equal(a.complete,false);assert.equal(bad.calls.length,1);assert.equal(a.jobs.length,0);
 const partial=apiFixture({truncated:true}),b=await collectDraftJobLogs({repository:'owner/repo',runId:11,attempt:2,head:partial.head,
  output:path.join(directory,'partial'),transport:partial.transport});
 assert.equal(b.complete,false);assert.equal(b.jobs[0].log.complete,false);
 assert.equal(b.unavailable[0].reason,'partial_job_log');assert(fs.existsSync(path.join(directory,'partial/job-22.log')));
});
test('full-log recovery uses only trusted default source and read-only credentials',()=>{
 const workflow=fs.readFileSync(path.join(root,'.github/workflows/hive-mind-draft-logs.yml'),'utf8');
 assert.match(workflow,/types: \[completed\]/);assert.match(workflow,/contents: read/);assert.match(workflow,/actions: read/);
 assert.match(workflow,/ref: \$\{\{ github.event.repository.default_branch \}\}/);
 assert.doesNotMatch(workflow,/head_sha \}\}|actions\/download-artifact|(?:contents|actions|issues|pull-requests): write/);
 assert.match(workflow,/if: always\(\)/);
});
test('actual historical plan-only attempts remain non-solving replay observations',()=>{
 const observations=JSON.parse(fs.readFileSync(path.join(root,'data/benchmarks/self-coding-ci-observations.json'),'utf8'));
 assert.equal(observations.cases.length,3);
 for(const row of observations.cases){
  assert.equal(row.observedTerminal,'planned_not_executed');assert.equal(row.implemented,false);
  assert.match(row.completePublicLog.sha256,/^[a-f0-9]{64}$/);assert(row.completePublicLog.bytes>400000);
  assert.deepEqual(usefulDraftChanges(row.files),[]);
 }
});

test('a command returning 124 is not mislabeled as an expired deadline',t=>{
 const directory=workspace(t);
 const result=spawnSync('bash',['scripts/run-with-session-evidence.sh','10','attempt',process.execPath,'-e','process.exitCode=124;'],
  {cwd:root,env:{...process.env,FORMAL_AI_DRAFT_LOG_DIR:directory},encoding:'utf8',timeout:15000,maxBuffer:65536});
 assert.equal(result.status,124,result.stderr);
 const receipt=JSON.parse(fs.readFileSync(path.join(directory,'attempt-outcome.json'),'utf8'));
 assert.equal(receipt.deadlineExpired,false);assert.equal(receipt.commandExitCode,124);
});

test('mounted private diagnostics retain their mode and reject foreign subtrees and links before owner transfer',t=>{
 const directory=workspace(t),category='formal-ai-diagnostics',subtree=path.join(directory,category);
 fs.mkdirSync(subtree,{mode:0o700});fs.writeFileSync(path.join(subtree,'native.lino'),'private CI log\n',{mode:0o600});
 const identity=fs.statSync(subtree);
 const options={logDirectory:directory,userIdentity:identity.uid,groupIdentity:identity.gid,category};
 assert.throws(()=>transferOperationalEvidenceOwner({...options,category:'../private'}),/owned evidence category/);
 fs.symlinkSync(path.join(subtree,'native.lino'),path.join(subtree,'link'));
 assert.throws(()=>transferOperationalEvidenceOwner(options),/symlink/);
 fs.unlinkSync(path.join(subtree,'link'));
 assert.equal(transferOperationalEvidenceOwner(options).transferred,2);
 assert.equal(fs.statSync(path.join(subtree,'native.lino')).mode&0o777,0o600);
});

test('a retried consumer reuses the producer artifact IDs and keeps prior attempt evidence immutable',()=>{
 const workflow=fs.readFileSync(path.join(root,'.github/workflows/hive-mind-draft.yml'),'utf8');
 assert.match(workflow,/binary_artifact_id: \$\{\{ steps.binary-artifact.outputs.artifact-id \}\}/);
 assert.match(workflow,/artifact-ids: \$\{\{ needs.build.outputs.binary_artifact_id \}\}/);
 assert.match(workflow,/artifact-ids: \$\{\{ needs.build.outputs.evidence_artifact_id \}\}/);
 assert.match(workflow,/name: hive-mind-main-evidence-\$\{\{ github.run_id \}\}-\$\{\{ github.run_attempt \}\}/);
 assert.doesNotMatch(workflow,/overwrite: true/);
});

test('streaming byte budget stops the process and retains an explicitly incomplete outcome',async t=>{
 const directory=workspace(t);
 const receipt=await captureDraftCommand({seconds:5,phase:'attempt',command:process.execPath,
  args:['-e','process.stdout.write("x".repeat(10000));setInterval(()=>{},1000);'],directory,maxBytes:128,mirror:false});
 assert.equal(receipt.completeConsoleCapture,false);assert.equal(receipt.error,'ELOGSIZE');
 assert.equal(receipt.cleanupComplete,true);assert(receipt.bytes<=128);
});

test('structured capture redaction preserves JSON/Lino syntax and exact large integer coordinates',async t=>{
 const {encodeDialogRecord,decodeDialogRecord}=await import('../../../js/server/dialog-codec.mjs');
 const {parseJson}=await import('../../../js/server/json.mjs');
 const directory=workspace(t),source=path.join(directory,'session');fs.mkdirSync(source);
 fs.writeFileSync(path.join(source,'native.jsonl'),'{"job":122318653442,"large":901234567891234567890,"password":"must-hide","text":"alice@example.invalid"}\n');
 fs.writeFileSync(path.join(source,'dialog.lino'),encodeDialogRecord({job:122318653442,password:'must-hide',text:'alice@example.invalid'})+'\n');
 const destination=path.join(directory,'artifact'),receipt=prepareDraftEvidence({source,destination,env:{}});
 assert.equal(receipt.complete,true);
 const value=parseJson(fs.readFileSync(path.join(destination,'native.jsonl'),'utf8'),{unboundedIntegers:true});
 assert.equal(value.job,122318653442);assert.equal(value.large,901234567891234567890n);
 assert.equal(value.password,'[REDACTED]');assert.equal(value.text,'[email]');
 const dialog=decodeDialogRecord(fs.readFileSync(path.join(destination,'dialog.lino'),'utf8').trimEnd());
 assert.equal(dialog.job,122318653442);assert.equal(dialog.password,'[REDACTED]');assert.equal(dialog.text,'[email]');
});

test('an interrupted JSONL tail remains available and never earns complete payload credit',t=>{
 const directory=workspace(t),source=path.join(directory,'session');fs.mkdirSync(source);
 fs.writeFileSync(path.join(source,'native.jsonl'),'{"message":"full earlier tool receipt"}\n{"cut":');
 const destination=path.join(directory,'artifact'),receipt=prepareDraftEvidence({source,destination,env:{}});
 assert.equal(receipt.complete,false);assert.equal(receipt.fullModelEvidence,false);
 const retained=fs.readFileSync(path.join(destination,'native.jsonl'),'utf8');
 assert(retained.includes('full earlier tool receipt'));assert(retained.endsWith('{"cut":'));
 assert.equal(receipt.unavailable[0].reason,'malformed_structured_capture');
});
test('the final pipeline status fetches its exact tiny script without another repository clone',async()=>{
 const {createRequire}=await import('node:module'),YAML=createRequire(import.meta.url)('yaml');
 const workflow=YAML.parse(fs.readFileSync(path.join(root,'.github/workflows/release.yml'),'utf8'));
 const steps=workflow.jobs['pipeline-status'].steps;
 assert(!steps.some(step=>step.uses?.startsWith('actions/checkout@')));
 assert.equal(steps.at(-1).run,'bash scripts/check-pipeline-status.sh');
 let queried,written;
 const fsStub={mkdirSync(){},writeFileSync(file,bytes){written={file,bytes};}};
 const context={repo:{owner:'owner',repo:'repo'},sha:'a'.repeat(40)};
 const execute=new vm.Script('(async()=>{'+steps[0].with.script+'})()');
 await execute.runInNewContext({context,Buffer,require:()=>fsStub,github:{rest:{repos:{getContent:async query=>{
  queried=query;return {data:{type:'file',encoding:'base64',size:11,content:Buffer.from('status body').toString('base64')}};
 }}}}});
 assert.equal(queried.ref,context.sha);assert.equal(queried.request.timeout,20000);
 assert.equal(written.file,'scripts/check-pipeline-status.sh');assert.equal(written.bytes.toString(),'status body');
});


test('post-run collection retains logs larger than the observed former eight MiB ceiling',async t=>{
 const directory=workspace(t),fixture=apiFixture();
 const text='BEGIN FULL LOG\n'+'x'.repeat(9*1024*1024)+'\nLAST ROOT CAUSE\n';
 const transport=async(args,options)=>{
  if(!args[1].endsWith('/logs'))return fixture.transport(args,options);
  const bytes=Buffer.from(text),retained=bytes.subarray(0,options.maxDownloadBytes);
  return {ok:true,text:retained.toString(),bytesReceived:retained.length,
   streamComplete:retained.length===bytes.length,downloadTruncated:retained.length<bytes.length};
 };
 const receipt=await collectDraftJobLogs({repository:'owner/repo',runId:11,attempt:2,head:fixture.head,
  output:path.join(directory,'artifact'),transport});
 assert.equal(receipt.complete,true);assert.equal(receipt.jobs[0].log.complete,true);
 assert.equal(fs.readFileSync(path.join(directory,'artifact/job-22.log'),'utf8'),text);
});
