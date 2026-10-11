import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,rmSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
const workflow=readFileSync('.github/workflows/container-images.yml','utf8');
const prefix="        node --input-type=module - <<'NODE'\n";
const start=workflow.indexOf(prefix);assert.ok(start>=0);
const end=workflow.indexOf('        NODE',start+prefix.length);
const resolver=workflow.slice(start+prefix.length,end).split('\n').map(line=>line.slice(8)).join('\n');
function execute(environment,{published=false}={}){
 const root=mkdtempSync(join(tmpdir(),'formal-ai-container-checks-'));
 const output=join(root,'outputs'),shim=join(root,'gh');
 writeFileSync(shim,'#!/usr/bin/env node\nif(process.env.FIXTURE_PUBLISHED!=="true")throw Error("checks unexpectedly queried GitHub");console.log(JSON.stringify({tag_name:"v1.2.3",draft:false,prerelease:false}));\n',{mode:0o755});
 try{
  const result=spawnSync(process.execPath,['--input-type=module','-'],{
   input:resolver,encoding:'utf8',timeout:15000,
   env:{...process.env,PATH:root+':'+process.env.PATH,GITHUB_OUTPUT:output,
    FIXTURE_PUBLISHED:String(published),EVENT:'workflow_dispatch',CHECKS_MODE:'checks',
    RELEASE_MODE:'checks',PUBLISH:'false',REPOSITORY:'fixture/repository',REFERENCE:'refs/heads/feature',
    CUSTOM_REF:'',INPUT_TAG:'',...environment}});
  return {...result,outputs:result.status===0?readFileSync(output,'utf8'):''};
 }finally{rmSync(root,{recursive:true,force:true});}
}
test('bot checks dispatch inputs are declared and build without publication',()=>{
 const triggers=workflow.slice(0,workflow.indexOf('\npermissions:'));
 assert.match(triggers,/branches-ignore: \['e2e\/\*\*'\]/u);
 assert.match(triggers,/options: \[checks\]/u);assert.match(triggers,/pull-request:/u);
 const result=execute({});assert.equal(result.status,0,result.stderr);
 assert.match(result.outputs,/build=true\npublish=false\ntag=\n/u);
});
test('checks dispatch refuses registry publication even on main',()=>{
 const result=execute({PUBLISH:'true',REFERENCE:'refs/heads/main',INPUT_TAG:'v1.2.3'});
 assert.notEqual(result.status,0);assert.match(result.stderr,/checks dispatch cannot request registry publication/u);
});
test('direct publication remains explicit and requires main plus a published stable tag',()=>{
 const environment={RELEASE_MODE:'publication',PUBLISH:'true',INPUT_TAG:'v1.2.3'};
 const foreign=execute(environment,{published:true});assert.notEqual(foreign.status,0);
 assert.match(foreign.stderr,/requires main/u);
 const valid=execute({...environment,REFERENCE:'refs/heads/main'},{published:true});
 assert.equal(valid.status,0,valid.stderr);assert.match(valid.outputs,/build=true\npublish=true\ntag=v1\.2\.3\n/u);
 const noPush=execute({RELEASE_MODE:'publication'});assert.notEqual(noPush.status,0);
 assert.match(noPush.stderr,/explicit publication also requires push/u);
});
test('automatic and reusable publication retain their original stable resolver path',()=>{
 const result=execute({EVENT:'workflow_call',CHECKS_MODE:'',RELEASE_MODE:'',PUBLISH:'true',REFERENCE:'refs/heads/main',INPUT_TAG:'v1.2.3'},{published:true});
 assert.equal(result.status,0,result.stderr);assert.match(result.outputs,/publish=true/u);
 const parent=execute({CHECKS_MODE:'',RELEASE_MODE:'',PUBLISH:'true',REFERENCE:'refs/heads/main',INPUT_TAG:'v1.2.3'},{published:true});
 assert.equal(parent.status,0,parent.stderr);assert.match(parent.outputs,/publish=true/u);
});
