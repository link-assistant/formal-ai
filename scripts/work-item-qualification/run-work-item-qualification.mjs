#!/usr/bin/env node
// DRAFT hosted runner: source-correct actual Hive solve, frozen task tree, private independent grader.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {spawn} from 'node:child_process';
import {createCommandCapture} from './command-capture.mjs';
import {startBroker} from './transport-broker.mjs';
import {checkDocumentationCandidate} from './independent-oracle.mjs';
import {prepareFixture,taskNames,ownedTaskBody} from './fixtures.mjs';
import {actualTraceOracle} from './trace-oracle.mjs';
import {workspaceSnapshot} from './workspace-snapshot.mjs';
const root=process.env.GITHUB_WORKSPACE;
const shared=async file=>import(pathToFileURL(path.join(root,file)));
const {boundedCommand}=await shared('js/server/bounded-command.mjs');
const {githubReader}=await shared('scripts/admit-hive-original-delivery.mjs');
const {sourceBytes}=await shared('scripts/lib/hive-delivery-source.mjs');
const {createPublicEvidenceExport}=await shared('scripts/lib/issue-1364-public-evidence.mjs');
const {usefulDraftChanges}=await shared('scripts/hive-mind-draft-result.mjs');
const sha=value=>createHash('sha256').update(value).digest('hex');
assert.equal(process.env.GITHUB_ACTIONS,'true');
assert.equal(process.env.GITHUB_REF,'refs/heads/main','main-only runtime/effects');
assert.equal(process.env.GITHUB_RUN_ATTEMPT,'1','uncertain effects never automatically rerun');
const repository=process.env.GITHUB_REPOSITORY, tool=process.env.E2E_TOOL, task=process.env.WORK_ITEM_TASK;
assert.equal(repository,'link-assistant/formal-ai');assert(['agent','claude','codex'].includes(tool));
assert(taskNames.includes(task));
const selection=JSON.parse(fs.readFileSync(process.env.WORK_ITEM_SELECTION));
assert.equal(selection.admitted,true);assert.equal(selection.source,process.env.GITHUB_SHA);
assert.match(selection.hiveSource,/^[a-f0-9]{40}$/u);
// Admission and immutable resource owner must be uploaded by workflow BEFORE this command.
const owner=JSON.parse(fs.readFileSync(process.env.WORK_ITEM_OWNER));
assert.equal(owner.taskMode,'work-item-1483');assert.equal(owner.run,Number(process.env.GITHUB_RUN_ID));
assert.equal(owner.selectedSource,selection.source);assert.equal(owner.inputHash,selection.inputHash);
const owned=path.join(process.env.RUNNER_TEMP,'work-item-'+tool+'-'+task), raw=owned+'/raw';
fs.mkdirSync(raw,{recursive:true,mode:0o777});fs.chmodSync(owned,0o777);fs.chmodSync(raw,0o777);
const captureCommand=createCommandCapture(raw);
const observations=[], resources={schema:'formal-ai-work-item-owned-task/v1',owner,tool,task,
  repository,baseBranch:'e2e/work-item/'+owner.run+'/'+tool+'/'+task,baseCommit:null,
  issueNumber:null,pullRequestNumber:null,head:null,source:selection.source,phase:'preparing'};
const save=()=>fs.writeFileSync(owned+'/resource.json',JSON.stringify(resources,null,2)+'\n');save();
async function command(program,args,{cwd=root,environment=process.env,input,timeout=30000,requireSuccess=true}={}){
  const value=await boundedCommand(program,args,{cwd,env:environment,input,timeout,
    stdoutLimitBytes:16*1024*1024,stderrLimitBytes:1024*1024});
  captureCommand({program,args,status:value.status,signal:value.signal,timedOut:value.timed_out,
    cleanupComplete:value.cleanup_complete,elapsedNanoseconds:String(value.elapsedNs),
    error:value.error?{code:value.error.code??null,message:value.error.message}:null},value.stdout,value.stderr,input);
  observations.push({program,args,status:value.status,signal:value.signal,timedOut:value.timed_out,
    cleanupComplete:value.cleanup_complete,stdoutSha256:sha(value.stdout),stderrSha256:sha(value.stderr)});
  if(requireSuccess)assert(value.status===0&&!value.signal&&!value.error&&!value.timed_out&&value.cleanup_complete===true,
    'command failed: '+program+'; inspect retained status and logs');
  return value;
}
let writes=0;
async function api(endpoint,{method='GET',body}={}){
  assert(endpoint.startsWith('repos/'+repository+'/'));
  if(method!=='GET')assert(++writes<=16,'finite uncertain writes; never retry');
  const result=await command('gh',['api','--method',method,endpoint,...(body===undefined?[]:['--input','-'])],
    {input:body===undefined?undefined:Buffer.from(JSON.stringify(body))});
  return result.stdout.length?JSON.parse(result.stdout):null;
}
const reader=githubReader({repository});
const ownerJobs=await reader.read('repos/'+repository+'/actions/runs/'+owner.run+'/jobs?per_page=100&filter=latest');
assert.equal(ownerJobs.jobs.length,ownerJobs.total_count);assert(ownerJobs.total_count<=100);
const actualJobs=ownerJobs.jobs.filter(job=>job.name==='Actual Hive '+tool+' work-item '+task);
assert.equal(actualJobs.length,1);resources.job=actualJobs[0].id;save();
const fixture=await prepareFixture({task,source:selection.source,
  read:(commit,name)=>sourceBytes(reader,repository,commit,name)});
const {files,guide}=fixture;
// No .git history, existing answer, acceptance tests, expectation file or grader enters this tree.
const privateFacts={...fixture.privateFacts,guide,inputs:files.map(file=>({path:file.path,sha256:sha(file.content)})),
  unsupported:task==='unsupported',contradiction:task==='contradicted-documentation'};
fs.writeFileSync(owned+'/grader-inputs.json',JSON.stringify(privateFacts,null,2));
let broker, failure=null;
try{
  const tree=await api('repos/'+repository+'/git/trees',{method:'POST',body:{tree:files}});
  const commit=await api('repos/'+repository+'/git/commits',{method:'POST',body:{message:'Owned frozen work-item fixture',tree:tree.sha,parents:[]}});
  resources.baseCommit=commit.sha;save();
  await api('repos/'+repository+'/git/refs',{method:'POST',body:{ref:'refs/heads/'+resources.baseBranch,sha:commit.sha}});save();
  const body=ownedTaskBody(fixture,owner,tool,task);
  resources.phase='creating-issue';resources.contractBodySha256=sha(fixture.body);resources.issueBodySha256=sha(body);save();
  const issue=await api('repos/'+repository+'/issues',{method:'POST',body:{title:fixture.title,body,labels:['e2e-task','no-formal-ai-draft']}});
  resources.issueNumber=issue.number;resources.issueBodySha256=sha(body);save();
  const transport=owned+'/transport', rpc=transport+'/rpc', mirror=transport+'/mirror.git';
  fs.mkdirSync(rpc,{recursive:true,mode:0o777});fs.chmodSync(transport,0o777);fs.chmodSync(rpc,0o777);
  const bundles=transport+'/bundles';fs.mkdirSync(bundles,{mode:0o777});fs.chmodSync(bundles,0o777);
  await command('git',['init','--bare',mirror]);
  await command('git',['--git-dir',mirror,'-c','credential.helper=', '-c','credential.helper=!gh auth git-credential',
    'fetch','--no-tags','--depth=1','https://github.com/'+repository+'.git',
    'refs/heads/'+resources.baseBranch+':refs/heads/'+resources.baseBranch]);
  await command('git',['--git-dir',mirror,'symbolic-ref','HEAD','refs/heads/'+resources.baseBranch]);
  const privateDirectory=owned+'/private-broker';fs.mkdirSync(privateDirectory,{mode:0o700});
  const privateMirror=privateDirectory+'/git';
  await command('git',['clone','--bare',mirror,privateMirror]);
  // Grant only the controlled fixture mirror to box; private grader remains outside every mount.
  await command('chmod',['-R','a+rX',transport]);
  const policy={repository,issueNumber:issue.number,baseBranch:resources.baseBranch,baseCommit:commit.sha,mirror,privateDirectory,privateMirror};
  fs.writeFileSync(transport+'/policy.json',JSON.stringify(policy));
  const bin=transport+'/bin';fs.mkdirSync(bin,{mode:0o755});
  const client=fs.readFileSync(new URL('./transport-client.mjs',import.meta.url));
  fs.writeFileSync(bin+'/client.mjs',client,{mode:0o755});fs.symlinkSync('client.mjs',bin+'/gh');fs.symlinkSync('client.mjs',bin+'/git');
  const hive=path.join(root,'.hive-mind');
  assert.equal((await command('git',['-C',hive,'rev-parse','HEAD'])).stdout.toString().trim(),selection.hiveSource);
  const version=JSON.parse(fs.readFileSync(hive+'/package.json')).version;
  const {buildDockerArgv,buildRunContextEnv,buildGitIdentityEnv}=await import(pathToFileURL(hive+'/scripts/formal-ai-draft.lib.mjs'));
  const {buildE2eSolveArgv}=await import(pathToFileURL(hive+'/scripts/e2e-hello-world.lib.mjs'));
  const {resolveDraftImages}=await import(pathToFileURL(hive+'/scripts/formal-ai-draft-image.lib.mjs'));
  const image=resolveDraftImages({version}).image;
  await command('docker',['pull',image],{timeout:180000});
  const imageId=JSON.parse((await command('docker',['image','inspect',image])).stdout)[0].Id;
  assert.match(imageId,/^sha256:[a-f0-9]{64}$/u);
  const binary=path.join(root,'rust/target/release/formal-ai'), binarySha256=sha(fs.readFileSync(binary));
  const actualVersion=(await command('docker',['run','--rm','--network','none','-v',binary+':/usr/local/bin/formal-ai:ro',
    '--entrypoint','/usr/local/bin/formal-ai',imageId,'--version'])).stdout.toString().trim();
  const imagePath=(await command('docker',['run','--rm','--network','none','--entrypoint','/bin/sh',imageId,'-c','printf "%s" "$PATH"'])).stdout.toString();
  assert(!/[\r\n]/u.test(imagePath));
  const workspace=owned+'/solver-workspace';fs.mkdirSync(workspace,{mode:0o777});fs.chmodSync(workspace,0o777);
  const runtime={source:selection.source,hiveSource:selection.hiveSource,hiveVersion:version,image,imageId,binarySha256,
    actualVersion,transportSha256:sha(client),network:'none',model:'formal-ai',tool,authorshipCredit:0};
  fs.writeFileSync(raw+'/runtime.json',JSON.stringify(runtime,null,2));
  broker=startBroker({directory:rpc,policy,capture:captureCommand,observe:value=>observations.push(value)});
  resources.phase='solving';save();
  async function solve(url,pass){
    const passDirectory=path.join(raw,'pass-'+pass);fs.mkdirSync(passDirectory,{mode:0o777});fs.chmodSync(passDirectory,0o777);
    const argv=buildE2eSolveArgv({issueUrl:'https://github.com/'+repository+'/issues/'+issue.number,
      tool,model:'formal-ai',baseBranch:resources.baseBranch,logDir:'/home/box/logs'});
    argv[0]=url;
    argv.splice(argv.indexOf('--attach-logs'),1);
    argv.push('--working-directory','/home/box/work-item','--attribution','none','--no-tool-update');
    const environment={...process.env,GH_TOKEN:'hosted-transport-placeholder'};
    // Host secret values and LLM service keys are never forwarded. The binary runs its own local endpoint.
    const docker=buildDockerArgv({solveArgv:['/opt/current-hive/src/solve.mjs',...argv],solveCommand:'node',image:imageId,
      hostLogDir:passDirectory,forwardEnv:[],setEnv:{...buildGitIdentityEnv(),...buildRunContextEnv(process.env),
        GH_TOKEN:environment.GH_TOKEN,FORMAL_AI_INSTALL_GRANT:'workspace',
        GH_REPO:repository,
        FORMAL_AI_DIALOG_LOG_DIR:'/home/box/logs/formal-ai-dialog',
        HIVE_MIND_SOLVE_NO_CHANGES_EXIT_CODE:pass===2?'0':'5',
        PATH:'/work-item-transport/bin:'+imagePath},
      extraMounts:[transport+':/work-item-transport:ro',rpc+':/work-item-transport/rpc',bundles+':/work-item-transport/bundles',workspace+':/home/box/work-item',hive+':/opt/current-hive:ro',binary+':/usr/local/bin/formal-ai:ro']});
    docker.splice(1,0,'--network','none','--cap-drop','ALL','--security-opt','no-new-privileges');
    const imageIndex=docker.indexOf(imageId);
    docker.splice(imageIndex,0,'-v',mirror+':/work-item-transport/mirror.git:ro');
    const result=await command('docker',docker,{environment,timeout:540000,requireSuccess:false});
    fs.writeFileSync(passDirectory+'/stdout.log',result.stdout);fs.writeFileSync(passDirectory+'/stderr.log',result.stderr);
    const outcome={status:result.status,signal:result.signal,timedOut:result.timed_out,cleanupComplete:result.cleanup_complete,
      error:result.error?{code:result.error.code??null,message:result.error.message}:null,
      trace:null,traceError:null};
    resources.currentPass={index:pass,...outcome};save();
    try{outcome.trace=await actualTraceOracle({root,directory:passDirectory,noop:pass===2,unsupported:task==='unsupported'});}
    catch(error){outcome.traceError=error.message;}
    return outcome;
  }
  resources.firstPass=await solve(issue.html_url,1);save();
  const pulls=await api('repos/'+repository+'/pulls?state=all&base='+encodeURIComponent(resources.baseBranch)+'&per_page=100');
  assert(pulls.length<=1,'unique candidate required');
  if(pulls.length){resources.pullRequestNumber=pulls[0].number;resources.head=pulls[0].head.sha;resources.branch=pulls[0].head.ref;}
  if(resources.pullRequestNumber){
    const diff=await command('gh',['api','repos/'+repository+'/pulls/'+resources.pullRequestNumber,
      '-H','Accept: application/vnd.github.diff']);fs.writeFileSync(raw+'/initial-candidate.diff',diff.stdout);
  }
  resources.phase='candidate-captured';save();
  // Immutable candidate and first-pass result are captured before the host-only independent checker.
  if(task==='unsupported'){
    const changes=pulls.length?await api('repos/'+repository+'/pulls/'+pulls[0].number+'/files?per_page=100'):[];
    if(pulls.length)assert.equal(changes.length,pulls[0].changed_files);
    resources.independentCheck={complete:usefulDraftChanges(changes).length===0&&resources.firstPass.trace?.terminals.some(row=>
      row.disposition==='gap'&&row.verified===false&&row.unresolvedEvidence===true)===true,
      status:'unsupported-task-retained',usefulChanges:usefulDraftChanges(changes),
      completionCredit:false};
  }else{
    resources.independentCheck=await checkDocumentationCandidate({api,repository,resource:resources,
      originalFiles:files,guide,contradiction:task==='contradicted-documentation',kind:fixture.kind});save();
    if(resources.independentCheck.complete&&resources.firstPass.trace!==null){
      const before=resources.head;
      const beforeWorkspace=workspaceSnapshot(workspace);
      const writeObservation=owned+'/write-observation.json',ready=owned+'/write-observation.ready';
      const watcher=spawn('python3',[new URL('./observe-writes.py',import.meta.url).pathname,workspace,writeObservation,ready],
        {stdio:['ignore','ignore','pipe']});
      const stopped=new Promise(resolve=>{
        watcher.once('exit',(code,signal)=>resolve({code,signal,error:null}));
        watcher.once('error',error=>resolve({code:null,signal:null,error:error.code??'watcher-process-error'}));
      });
      async function waitForWatcher(milliseconds){
        let timer;
        try{return await Promise.race([stopped,new Promise(resolve=>{timer=setTimeout(()=>resolve(null),milliseconds);})]);}
        finally{clearTimeout(timer);}
      }
      try{
        const deadline=Date.now()+5000;
        while(!fs.existsSync(ready)){assert(Date.now()<deadline,'write observer failed to initialize');await new Promise(resolve=>setTimeout(resolve,20));}
        resources.secondPass=await solve('https://github.com/'+repository+'/pull/'+resources.pullRequestNumber,2);
      }finally{
        watcher.kill('SIGTERM');let outcome=await waitForWatcher(2000),usedKill=false;
        if(outcome===null){usedKill=true;watcher.kill('SIGKILL');outcome=await waitForWatcher(2000);}
        const complete=outcome!==null&&outcome.code===0&&outcome.error===null;
        resources.writeObserverCleanup={complete,processReaped:outcome!==null,usedKill,outcome};save();
        if(outcome===null){watcher.stderr?.destroy();watcher.unref();}
        assert(complete,'write observer cleanup incomplete after bounded TERM/KILL; no no-op credit');
      }
      resources.actualWrites=JSON.parse(fs.readFileSync(writeObservation));
      const after=await api('repos/'+repository+'/pulls/'+resources.pullRequestNumber);
      const afterWorkspace=workspaceSnapshot(workspace);
      resources.noopVerified=resources.secondPass.status===0&&resources.secondPass.cleanupComplete===true
        &&resources.secondPass.signal===null&&resources.secondPass.timedOut===false&&resources.secondPass.error===null
        &&after.head.sha===before&&JSON.stringify(beforeWorkspace)===JSON.stringify(afterWorkspace)
        &&resources.actualWrites.complete===true&&resources.actualWrites.events.length===0
        &&resources.secondPass.trace?.writes===0;
    }
  }
  resources.phase='verified';resources.complete=task==='unsupported'
    ? resources.independentCheck.complete
    : resources.firstPass.status===0&&resources.firstPass.cleanupComplete===true
      &&resources.firstPass.signal===null&&resources.firstPass.timedOut===false&&resources.firstPass.error===null
      &&resources.firstPass.trace!==null
      &&resources.independentCheck.complete&&resources.noopVerified===true;
  resources.taskDisposition=task==='unsupported'?'unresolved':resources.complete?'verified':'unresolved';
  save();
  assert(resources.complete,'qualification incomplete; retain exact unresolved obligations and failed status');
}catch(error){failure=error;resources.failure=error.message;resources.phase='failed';save();}
finally{
  broker?.stop();fs.writeFileSync(raw+'/observations.json',JSON.stringify(observations,null,2));
  const publication=createPublicEvidenceExport(owned+'/publish',process.env);
  publication.collect(raw,'attempt');publication.json('resource.json',resources);publication.finish();
  if(process.env.GITHUB_OUTPUT)fs.appendFileSync(process.env.GITHUB_OUTPUT,'evidence='+owned+'/publish\nresource='+owned+'/resource.json\n');
}
if(failure)throw failure;
