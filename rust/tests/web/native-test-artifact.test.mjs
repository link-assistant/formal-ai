// Verified native artifacts reject incompatible inputs before running the original testcase.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,existsSync,rmSync,readdirSync} from 'node:fs';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {tmpdir} from 'node:os';
import {writeReceipt,verifyReceipt,runVerifiedTest} from '../../../scripts/native-test-artifact.mjs';
import YAML from 'yaml';
const identity={'source-commit':'a'.repeat(40),'source-tree':'b'.repeat(40),'cargo-lock-sha256':'c'.repeat(64),platform:'linux',architecture:'x64','rust-flags':'-Dwarnings',compiler:'actual producer compiler',profile:'release',features:'all'};
function fixture(){
 const root=mkdtempSync(join(tmpdir(),'native-test-artifact-')),directory=join(root,'tests'),binary=join(root,'formal-ai');mkdirSync(directory);
 const script='#!/bin/sh\nif [ "$1" = --list ]; then printf "suite::wanted: test\\n"; exit 0; fi\n[ "$1" = --exact ] && [ "$2" = suite::wanted ] || exit 9\nprintf "actual testcase ran\\n" > ran.txt\n';
 writeFileSync(join(directory,'unit'),script);for(const target of ['integration','source'])writeFileSync(join(directory,target),target+' executable bytes');writeFileSync(binary,'actual binary bytes');
 writeReceipt(directory,identity,binary);return {root,directory,binary,dispose:()=>rmSync(root,{recursive:true,force:true})};
}
test('same build identity and bytes execute the exact registered testcase',()=>{
 const f=fixture();try{assert.equal(runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root),0);assert.equal(readFileSync(join(f.root,'ran.txt'),'utf8'),'actual testcase ran\n');}finally{f.dispose();}
});
test('an explicit integration target executes only its own verified member',()=>{
 const f=fixture();try{
  writeFileSync(join(f.directory,'integration'),readFileSync(join(f.directory,'unit')));
  writeReceipt(f.directory,identity,f.binary);
  rmSync(join(f.directory,'unit'));
  assert.equal(runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root,'integration'),0);
  assert.equal(readFileSync(join(f.root,'ran.txt'),'utf8'),'actual testcase ran\n');
 }finally{f.dispose();}
});
test('unknown or tampered explicit targets refuse before executing bytes',()=>{
 const f=fixture();try{
  for(const target of ['unknown','../unit'])assert.throws(()=>runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root,target));
  writeFileSync(join(f.directory,'integration'),'tampered');
  assert.throws(()=>runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root,'integration'));
  assert.equal(existsSync(join(f.root,'ran.txt')),false);
 }finally{f.dispose();}
});
test('critical latency checks precede artifact distribution within the bounded shared producer',()=>{
 const release=YAML.parse(readFileSync(new URL('../../../.github/workflows/release.yml',import.meta.url),'utf8'));
 const job=release.jobs['build-artifacts'],steps=job.steps;
 const early=steps.find(step=>step.name==='Bind native test artifacts to their actual build inputs');
 assert.equal(job['timeout-minutes'],25);
 assert(steps.indexOf(early)<steps.findIndex(step=>step.name==='Upload formal-ai binaries'));
 assert.equal(early.run,'FORMAL_AI_DEADLINE_GRACE_SECONDS=5 scripts/run-with-deadline.sh 300 node scripts/native-test-artifact.mjs write-and-check dist/tests release all rust/tests/fixtures/released-client-requests/smoke-tests.json');
 const cases=JSON.parse(readFileSync(new URL('../fixtures/released-client-requests/smoke-tests.json',import.meta.url),'utf8'));
 assert.deepEqual(cases.map(entry=>[entry.target,entry.name]),[
  ['unit','released_client_request_latency::absent_predictions_do_not_resolve_the_captured_coding_prompt'],
  ['unit','released_client_request_latency::released_client_requests_plan_without_consuming_the_delivery_budget'],
  ['integration','released_client_request_latency::released_agent_request_is_bounded_through_native_http'],
 ]);
});
test('the selected unit transfer preserves the full original receipt while omitting unused executables',()=>{
 const f=fixture();try{
  for(const target of ['integration','source'])rmSync(join(f.directory,target));
  assert.throws(()=>verifyReceipt(f.directory,identity,f.binary),'full consumers still require all artifacts');
  assert.equal(runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root),0);
  assert.equal(readFileSync(join(f.root,'ran.txt'),'utf8'),'actual testcase ran\n');
 }finally{f.dispose();}
});
test('selected transfer refuses an empty selection, unknown target or malformed unused checksum',()=>{
 const f=fixture();try{
  for(const targets of [[],['unknown'],['unit','unit']]){
   assert.throws(()=>verifyReceipt(f.directory,identity,f.binary,{targets}));
  }
  const file=join(f.directory,'native-build-receipt.json'),receipt=JSON.parse(readFileSync(file,'utf8'));
  receipt.files.integration='invalid';writeFileSync(file,JSON.stringify(receipt));
  assert.throws(()=>runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root));
  assert.equal(existsSync(join(f.root,'ran.txt')),false);
 }finally{f.dispose();}
});
test('foreign source tree and build variants refuse before executing artifact bytes',()=>{
 const f=fixture();try{
  for(const [field,value] of [['source-commit','d'.repeat(40)],['source-tree','d'.repeat(40)],['cargo-lock-sha256','d'.repeat(64)],['platform','darwin'],['architecture','arm64'],['compiler','another compiler'],['rust-flags','--cfg incompatible'],['profile','debug'],['features','default']]){
   assert.throws(()=>runVerifiedTest(f.directory,{...identity,[field]:value},f.binary,'suite::wanted',f.root));assert.equal(existsSync(join(f.root,'ran.txt')),false);
  }
 }finally{f.dispose();}
});
test('tampered unit or runtime binary bytes are rejected before execution',()=>{
 for(const target of ['unit','binary']){const f=fixture();try{
  writeFileSync(target==='unit'?join(f.directory,'unit'):f.binary,'tampered');assert.throws(()=>runVerifiedTest(f.directory,identity,f.binary,'suite::wanted',f.root));assert.equal(existsSync(join(f.root,'ran.txt')),false);
 }finally{f.dispose();}}
});
test('missing testcase and noncanonical receipt cannot silently run a different test',()=>{
 const f=fixture();try{
  assert.throws(()=>runVerifiedTest(f.directory,identity,f.binary,'suite::absent',f.root));assert.equal(existsSync(join(f.root,'ran.txt')),false);
  const path=join(f.directory,'native-build-receipt.json'),receipt=JSON.parse(readFileSync(path,'utf8'));receipt.files['../outside']='0'.repeat(64);writeFileSync(path,JSON.stringify(receipt));assert.throws(()=>verifyReceipt(f.directory,identity,f.binary));
 }finally{f.dispose();}
});
test('self-coding transfer carries only its executed member and the original source-bound receipt',()=>{
 const release=YAML.parse(readFileSync(new URL('../../../.github/workflows/release.yml',import.meta.url),'utf8'));
 const steps=release.jobs['build-artifacts'].steps;
 const selected=steps.find(step=>step.with?.name==='formal-ai-self-coding-test-executable');
 assert(selected);
 assert.deepEqual(selected.with.path.trim().split('\n'),['dist/tests/unit','dist/tests/native-build-receipt.json']);
 assert(steps.indexOf(selected)>steps.findIndex(step=>step.name==='Bind native test artifacts to their actual build inputs'));
 assert.equal(steps.find(step=>step.with?.name==='formal-ai-test-executables').with.path,'dist/tests');
 const consumer=YAML.parse(readFileSync(new URL('../../../.github/workflows/agent-cli-e2e.yml',import.meta.url),'utf8'));
 const download=consumer.jobs['agent-cli-e2e'].steps.find(step=>step.name==='Download the current native self-coding test executable');
 assert.equal(download.with.name,selected.with.name);assert.equal(download.with.path,'dist/self-coding-tests');
 assert.equal(download['timeout-minutes'],3);
 assert(!download.if.includes('inputs.full-replay'),'ordinary pull requests must exercise the selected transfer');
 const replay=consumer.jobs['agent-cli-e2e'].steps.find(step=>step.name?.includes('Replay Hive Mind self-coding'));
 assert(!replay.if.includes('inputs.full-replay'));
 assert(replay.run.includes('native-test-artifact.mjs run dist/self-coding-tests release all agentic_coding::self_coding::self_coding_session_replays'));
});

test('primary server parity keeps its full source and every case within separate acquisition headroom',()=>{
 const release=YAML.parse(readFileSync(new URL('../../../.github/workflows/release.yml',import.meta.url),'utf8'));
 const job=release.jobs['server-parity'];
 assert.equal(job['timeout-minutes'],30);assert.deepEqual(job.needs,['build-artifacts']);
 assert.equal(job.steps.length,4);
 assert.deepEqual(job.steps[0],{uses:'actions/checkout@v7','timeout-minutes':18,
  env:{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'},with:{'persist-credentials':false}});
 assert.equal(job['timeout-minutes']-job.steps[0]['timeout-minutes'],12);
 assert.deepEqual(job.steps.slice(1),[{"name":"Setup Node","uses":"actions/setup-node@v7","with":{"node-version":22}},{"uses":"./.github/actions/download-formal-ai-binary"},{"name":"Both servers answer the parity corpus alike (R1013-R1015)","run":"node scripts/check-server-parity.mjs --rust-binary rust/target/release/formal-ai --list"}]);
 assert.equal(job.if,"${{ (github.event_name != 'pull_request' && github.event_name != 'pull_request_target' && (github.ref == 'refs/heads/main' || github.event_name == 'release' || (github.event_name == 'workflow_run' && github.event.workflow_run.head_branch == 'main'))) && (!cancelled() && needs.build-artifacts.result == 'success') }}");
 assert.deepEqual(job.concurrency,{"group":"${{ github.workflow }}-${{ github.ref }}-server-parity${{ (github.event_name != 'pull_request' || github.event.action != 'synchronize') && format('-{0}', github.run_id) || '' }}","cancel-in-progress":"${{ github.event_name == 'pull_request' && github.event.action == 'synchronize' }}"});
 const cli=readFileSync(new URL('../../../scripts/check-server-parity.mjs',import.meta.url),'utf8');
 assert.match(cli,/for\s*\(\s*const request of corpus[.]requests\s*\)/u);
 assert.match(cli,/loadRatchet/u);assert.match(cli,/coverageProblems\(\s*corpus\s*\)/u);
 assert.equal(job.steps.at(-1).run,'node scripts/check-server-parity.mjs --rust-binary rust/target/release/formal-ai --list');
});

// Exercise extraction with actual ZIP bytes, including cached regular destinations.
test('artifact adapter streams real archives and refuses unsafe or corrupt members', {timeout:30000},()=>{
 const result=spawnSync('python3',[fileURLToPath(new URL('../../../experiments/issue-1580/artifact-extraction-fixtures.py',import.meta.url))],{encoding:'utf8',maxBuffer:256*1024});
 assert.equal(result.status,0,result.stdout+result.stderr);
 assert.match(result.stderr,/Ran 8 tests/u);
 assert.match(result.stderr,/\nOK\n/u);
});
test('shared native and renderer downloads replace the deprecated extractor without suppressing warnings',()=>{
 for(const name of ['download-formal-ai-binary','release-web']){
  const action=YAML.parse(readFileSync(new URL('../../../.github/actions/'+name+'/action.yml',import.meta.url),'utf8'));
  const download=action.runs.steps.find(step=>step.with?.name===(name==='release-web'?'native-release-web':'formal-ai-release-binary')&&step.uses?.includes('download-artifact'));
  assert.equal(download.uses,'./.github/actions/download-artifact');
  assert.equal(download.env?.NODE_OPTIONS,undefined);
  assert.equal(download.with.path,name==='release-web'?'${{ runner.temp }}/native-release-web':'rust/target/release');
  if(name==='release-web')assert.equal(download.if,"inputs.mode == 'import'");
 }
 const adapter=YAML.parse(readFileSync(new URL('../../../.github/actions/download-artifact/action.yml',import.meta.url),'utf8'));
 const transport=adapter.runs.steps.find(step=>step.uses?.startsWith('actions/download-artifact@'));
 assert.equal(transport.uses,'actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333');
 assert.equal(transport.with['skip-decompress'],true);
 assert.equal(transport.with['digest-mismatch'],'error');
 assert.equal(transport.with['merge-multiple'],false);
 assert.equal(transport.env.NODE_OPTIONS,'--throw-deprecation');
 const extraction=adapter.runs.steps.find(step=>step.id==='extract');
 assert.equal(extraction['working-directory'],'${{ github.workspace }}');
 assert.equal(extraction.env.ALLOW_EMPTY_SELECTION,"${{ inputs.name == '' && inputs.artifact-ids == '' }}");
 assert.ok(extraction.run.includes('--allow-empty-selection "$ALLOW_EMPTY_SELECTION"'));
 assert.match(extraction.run,/extract-artifacts[.]py/u);
 for(const input of ['name','artifact-ids','pattern','github-token','repository','run-id'])assert.equal(transport.with[input],'${{ inputs.'+input+' }}');
 assert.equal(adapter.outputs['download-path'].value,'${{ steps.extract.outputs.download-path }}');
});

test('hosted artifact round trip uses the real raw adapter on all supported desktop hosts',()=>{
 const workflow=YAML.parse(readFileSync(new URL('../../../.github/workflows/workflows.yml',import.meta.url),'utf8'));
 for(const event of ['push','pull_request'])for(const path of ['scripts/extract-artifacts.py','experiments/issue-1580/artifact-transport-roundtrip.py'])assert(workflow.on[event].paths.includes(path));
 const job=workflow.jobs['artifact-roundtrip'];
 assert(job);assert.equal(job['timeout-minutes'],10);
 assert.deepEqual(job.strategy.matrix.os,['ubuntu-24.04','macos-15','windows-2025']);
 const steps=job.steps,prepare=steps.findIndex(step=>step.run?.includes(' prepare ')),verify=steps.findIndex(step=>step.run?.includes(' verify '));
 const upload=steps.findIndex(step=>step.uses==='actions/upload-artifact@v7');
 const download=steps.findIndex(step=>step.uses==='./.github/actions/download-artifact');
 assert(prepare>=0 && upload>prepare && download>upload && verify>download);
 assert.equal(steps[upload].with.name,steps[download].with.name);
 assert.equal(steps[upload].with['if-no-files-found'],'error');
 assert.equal(steps[upload].with.path,'${{ runner.temp }}/artifact-roundtrip/source');
 assert.equal(steps[download].with.path,'${{ runner.temp }}/artifact-roundtrip/download');
 assert.equal(steps[0].with['persist-credentials'],false);
 for(const path of ['/.github/actions/download-artifact/','/scripts/extract-artifacts.py','/experiments/issue-1580/artifact-transport-roundtrip.py'])assert(steps[0].with['sparse-checkout'].includes(path));
});

// Every direct workflow consumer must bring its adapter from an available checkout.
test('all workflow artifact consumers use the raw adapter with its complete local source',()=>{
 const root=new URL('../../../',import.meta.url),directory=new URL('.github/workflows/',root);
 let consumers=0;
 for(const file of readdirSync(directory).filter(name=>/\.ya?ml$/u.test(name))){
  const workflow=YAML.parse(readFileSync(new URL(file,directory),'utf8'));
  for(const [id,job] of Object.entries(workflow.jobs??{}))for(const [index,step] of (job.steps??[]).entries()){
   assert(!step.uses?.startsWith('actions/download-artifact@'),file+' '+id+' still uses the deprecated extractor');
   if(!step.uses?.endsWith('/.github/actions/download-artifact') && step.uses!=='./.github/actions/download-artifact')continue;
   consumers+=1;
   assert.equal(step.env?.NODE_OPTIONS,undefined,file+' '+id+' must not suppress diagnostics');
   assert.equal(step.with?.['digest-mismatch'],undefined,'strict digests are enforced by the shared adapter');
   const prefix=step.uses.slice(2,-'.github/actions/download-artifact'.length).replace(/\/$/u,'');
   const checkout=job.steps.slice(0,index).findLast(value=>value.uses?.startsWith('actions/checkout@')&&(value.with?.path??'')===prefix&&!value.with?.repository);
   assert(checkout,file+' '+id+' adapter checkout must precede consumption');
   assert.equal(checkout.with?.['persist-credentials'],false);
   const sparse=checkout.with?.['sparse-checkout'];
   if(sparse){
    const paths=sparse.split('\n').map(value=>value.replace(/^\//u,'').replace(/\/$/u,''));
    assert(paths.includes('.github/actions/download-artifact'),file+' '+id+' sparse adapter missing');
    assert(paths.includes('scripts')||paths.includes('scripts/extract-artifacts.py'),file+' '+id+' sparse extractor missing');
   }
  }
 }
 assert.equal(consumers,98,'retain all 97 production transfers plus hosted qualification');
});
test('artifact setup preserves the caller Python environment and uses the selected interpreter explicitly',()=>{
 const adapter=YAML.parse(readFileSync(new URL('../../../.github/actions/download-artifact/action.yml',import.meta.url),'utf8'));
 const setup=adapter.runs.steps.find(step=>step.uses?.startsWith('actions/setup-python@'));
 assert.equal(setup.id,'python');assert.equal(setup.with['update-environment'],false);
 const extract=adapter.runs.steps.find(step=>step.id==='extract');
 assert.equal(extract.env.ARTIFACT_PYTHON,'${{ steps.python.outputs.python-path }}');
 assert.match(extract.run,/^"\$ARTIFACT_PYTHON" /u);
});

test('raw artifact retries recover lookup failures without extracting partial or corrupt archives', {timeout:30000},()=>{
 const env={...process.env,NODE_OPTIONS:'--max-old-space-size=128'};delete env.NODE_TEST_CONTEXT;
 const result=spawnSync(process.execPath,['--test','--test-reporter=tap','--test-concurrency=1',fileURLToPath(new URL('../../../experiments/issue-1580/artifact-download-retry.test.mjs',import.meta.url))],{encoding:'utf8',maxBuffer:256*1024,env});
 assert.equal(result.status,0,result.stdout+result.stderr);
 assert.match(result.stdout,/(?:tests 11|# tests 11)/u);
});
