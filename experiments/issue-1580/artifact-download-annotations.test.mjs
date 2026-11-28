// Execute real Node stdout/exit handling; no remote artifact service is claimed.
import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
const hook=fileURLToPath(new URL('../../.github/actions/download-artifact/retry-observations.cjs',import.meta.url));
const name='artifact-roundtrip-macos-15';
const reason='Unable to download artifact(s): Artifact not found for name: '+name;
const help='\n        Please ensure that your artifact is not expired and the artifact was uploaded using a compatible version of toolkit/upload-artifact.\n        For more information, visit the GitHub Artifacts FAQ: https://github.com/actions/toolkit/blob/main/packages/artifact/docs/faq.md';
const escape=value=>value.replaceAll('%','%25').replaceAll('\r','%0D').replaceAll('\n','%0A');
function run(text,{attempt='1',selected=name,status=1,preload=true,buffer=false}={}){
 const env={...process.env,INPUT_NAME:selected,ARTIFACT_RETRY_ATTEMPT:attempt};delete env.NODE_OPTIONS;
 const code='process.stdout.write('+(buffer?'Buffer.from(process.argv[1])':'process.argv[1]')+');process.exitCode=Number(process.argv[2]) || process.exitCode;';
 const result=spawnSync(process.execPath,[...(preload?['--require',hook]:[]),'-e',code,text,String(status)],{encoding:'utf8',env,maxBuffer:32768});
 assert.ifError(result.error);return result;
}
test('before repair the recovered first lookup leaves an error annotation',()=>{
 const line='::error::'+escape(reason+help)+'\n',before=run(line,{preload:false}),after=run(line);
 assert.equal(before.status,1);assert.equal(before.stdout,line);
 assert.equal(after.status,1);assert.ok(after.stdout.startsWith('::notice::'));assert.ok(after.stdout.includes(escape(reason+help)));
});
for(const attempt of ['1','2'])test('only retryable lookup '+attempt+' changes severity, retaining failure and complete diagnostics',()=>{
 for(const buffer of [false,true])for(const suffix of ['\n','\r\n']){
  const line='::error::'+escape(reason+help)+suffix,result=run(line,{attempt,buffer});
  assert.equal(result.status,1);assert.equal(result.stdout,line.replace('::error::','::notice::Artifact lookup failed; bounded retry remains: '));
 }
});
test('last, absent and unknown attempt retain the terminal missing-artifact error',()=>{
 const line='::error::'+escape(reason+help)+'\n';
 for(const attempt of ['3','','4']){const result=run(line,{attempt});assert.equal(result.status,1);assert.equal(result.stdout,line);}
});
test('unrelated names, digest failures and changed or unterminated messages remain errors',()=>{
 for(const line of ['::error::'+reason+'-foreign\n','::error::Digest validation failed for artifact(s): '+name+'\n','::error::'+reason+'%0Aunexpected extra diagnostic\n','::error::'+reason,'::warning::unrelated warning\n']){
  const result=run(line);assert.equal(result.status,1);assert.equal(result.stdout,line);
 }
});
test('empty and workflow-escaped artifact names cannot widen the match',()=>{
 const empty=run('::error::'+reason+'\n',{selected:''});assert.ok(empty.stdout.startsWith('::error::'));
 const selected='odd%name\nsecond',line='::error::Unable to download artifact(s): Artifact not found for name: '+escape(selected)+'\n';
 const result=run(line,{selected});assert.equal(result.status,1);assert.ok(result.stdout.startsWith('::notice::'));assert.ok(result.stdout.includes(escape(selected)));
});
test('partial ID selection is fatal on every attempt instead of publishing a subset',()=>{
 // The pinned action maps CSV tokens through parseInt before joining IDs.
 // Invalid tokens become NaN; large or negative integers also need rejection.
 for(const ids of ['42, 43','NaN','-1, 0','1e+21']){
  const line='::warning::Could not find the following artifact IDs: '+ids+'\n';
  const before=run(line,{status:0,preload:false});assert.equal(before.status,0);assert.equal(before.stdout,line);
  for(const attempt of ['1','2','3']){const result=run(line,{status:0,attempt});assert.equal(result.status,1);assert.ok(result.stdout.startsWith('::error::Incomplete requested artifact selection: '));assert.ok(result.stdout.includes(ids));}
 }
});
test('all three pinned action attempts load the guard with strict digest and distinct retry identity',()=>{
 const action=YAML.parse(readFileSync(new URL('../../.github/actions/download-artifact/action.yml',import.meta.url),'utf8'));
 const steps=action.runs.steps.filter(step=>step.uses?.startsWith('actions/download-artifact@'));assert.equal(steps.length,3);
 for(const [index,step] of steps.entries()){
  assert.equal(step.env.ARTIFACT_RETRY_ATTEMPT,String(index+1));
  assert.equal(step.env.NODE_OPTIONS,'${{ steps.observation_options.outputs.node-options }}');
  assert.equal(step.with['digest-mismatch'],'error');assert.equal(step.with['skip-decompress'],true);
 }
});
