// Execute the adapter's actual shell/extraction steps; the remote transport alone is stubbed.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,mkdirSync,rmSync,existsSync,readdirSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';

const root=fileURLToPath(new URL('../../',import.meta.url));
const action=YAML.parse(readFileSync(join(root,'.github/actions/download-artifact/action.yml'),'utf8'));
const transports=action.runs.steps.filter(step=>step.uses?.startsWith('actions/download-artifact@'));
function replay(failures,{cancelAfterFailure=false,corrupt=false,resetFails=false,selection={name:'fresh-artifact'}}={}){
  const directory=mkdtempSync(join(tmpdir(),'artifact-retry-'));
  const temporary=join(directory,'temporary'),destination=join(directory,'download'),output=join(directory,'outputs');
  const commands=join(directory,'commands');mkdirSync(commands);mkdirSync(temporary);writeFileSync(output,'');
  writeFileSync(join(commands,'sleep'),'#!/bin/sh\nprintf "%s\n" "$1" >> "$BACKOFF_LOG"\n',{mode:0o755});
  const python=spawnSync('python3',['-c','import sys; print(sys.executable)'],{encoding:'utf8'});
  assert.equal(python.status,0,python.stderr);
  const inputs={name:'','artifact-ids':'',pattern:'','github-token':'producer-token',repository:'owner/repository','run-id':'123',path:destination,'merge-multiple':'false',...selection};
  const steps={python:{outcome:'success',outputs:{'python-path':python.stdout.trim()}}};
  const attempts=[],executed=[],errors=[];let cancelled=false,failed=false,raw;
  function value(expression){
    const input=expression.match(/^inputs[.]([a-z-]+)$/u);if(input)return inputs[input[1]];
    const out=expression.match(/^steps[.]([a-z_]+)[.]outputs[.]([a-z-]+)$/u);if(out)return steps[out[1]]?.outputs[out[2]]??'';
    if(expression==='github.workspace')return root;
    if(expression==='github.action_path')return join(root,'.github/actions/download-artifact');
    if(expression==="inputs.name == '' && inputs.artifact-ids == ''")return String(!inputs.name&&!inputs['artifact-ids']);
    throw new Error('Unexpected expression: '+expression);
  }
  const expand=text=>String(text).replace(/\$\{\{\s*(.*?)\s*\}\}/gu,(_,expression)=>value(expression));
  function enabled(predicate){
    if(!predicate)return !failed&&!cancelled;
    const expression=predicate.replace(/^\$\{\{\s*|\s*\}\}$/gu,'');
    return expression.split(' && ').every(part=>{
      if(part==='always()')return true;
      if(part==='!cancelled()')return !cancelled;
      const match=part.match(/^steps[.]([a-z_]+)[.]outcome (==|!=) '(success|failure)'$/u);
      assert(match,'Unexpected predicate: '+part);
      return (steps[match[1]]?.outcome===match[3])===(match[2]==='==');
    });
  }
  try{
    for(const step of action.runs.steps){
      if(step.id==='python')continue;
      if(!enabled(step.if)){if(step.id)steps[step.id]={outcome:'skipped',outputs:{}};continue;}
      executed.push(step.id??step.name);let status=0,stderr='';const outputs={};
      if(step.uses){
        assert(step.uses.startsWith('actions/download-artifact@'),'only transport is stubbed');
        const requested=Object.fromEntries(Object.entries(step.with).map(([key,item])=>[key,expand(item)]));
        attempts.push(requested);raw=requested.path;
        assert.deepEqual(readdirSync(raw),[],'each attempt starts without partial archives');
        if(attempts.length<=failures){
          writeFileSync(join(raw,'stale.zip'),'partial failed download');status=1;stderr='Artifact not found';
          if(cancelAfterFailure)cancelled=true;
        }else if(corrupt)writeFileSync(join(raw,'artifact.zip'),'invalid ZIP');
        else{
          const made=spawnSync('python3',['-c',"import sys,zipfile; z=zipfile.ZipFile(sys.argv[1],'w'); z.writestr('payload.bin',bytes(range(256))); z.close()",join(raw,'artifact.zip')],{encoding:'utf8'});
          assert.equal(made.status,0,made.stderr);
        }
      }else{
        const env={...process.env,RUNNER_TEMP:temporary,GITHUB_OUTPUT:output,BACKOFF_LOG:join(directory,'backoffs'),PATH:commands+':'+process.env.PATH};
        for(const [key,item] of Object.entries(step.env??{}))env[key]=expand(item);
        const ran=resetFails&&step.id==='retry_second'?{status:29,stderr:'reset refused'}:spawnSync('bash',['--noprofile','--norc','-e','-o','pipefail','-c',step.run],{cwd:step['working-directory']?expand(step['working-directory']):root,env,encoding:'utf8',maxBuffer:256*1024});
        assert.ifError(ran.error);status=ran.status;stderr=ran.stderr;
        for(const line of readFileSync(output,'utf8').trim().split('\n')){const split=line.indexOf('=');if(split>0)outputs[line.slice(0,split)]=line.slice(split+1);}
        writeFileSync(output,'');
      }
      if(step.id)steps[step.id]={outcome:status===0?'success':'failure',outputs};
      if(status!==0){errors.push(stderr);if(!step['continue-on-error'])failed=true;}
    }
    return {attempts,executed,failed,cancelled,errors,steps,rawRemoved:!raw||!existsSync(raw),payload:existsSync(join(destination,'payload.bin'))?readFileSync(join(destination,'payload.bin')):null,backoffs:existsSync(join(directory,'backoffs'))?readFileSync(join(directory,'backoffs'),'utf8').trim().split('\n'):[]};
  }finally{rmSync(directory,{recursive:true,force:true});}
}

test('each bounded attempt retains the pinned raw transport, strict digest and original selectors',()=>{
  assert.equal(transports.length,3);
  for(const transport of transports){
    assert.equal(transport.uses,'actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333');
    assert.deepEqual(transport.with,transports[0].with);
    assert.equal(transport.with['digest-mismatch'],'error');assert.equal(transport.with['skip-decompress'],true);
    assert.equal(transport.with['merge-multiple'],false);assert.equal(transport.env.NODE_OPTIONS,'${{ steps.observation_options.outputs.node-options }}');
    for(const name of ['name','artifact-ids','pattern','github-token','repository','run-id'])assert.equal(transport.with[name],'${{ inputs.'+name+' }}');
  }
  assert.equal(transports[0]['continue-on-error'],true);assert.equal(transports[1]['continue-on-error'],true);
  assert.notEqual(transports[2]['continue-on-error'],true);
});
for(const failures of [0,1,2])test('a download succeeding on attempt '+(failures+1)+' extracts only its complete bytes',()=>{
  const actual=replay(failures);assert.equal(actual.failed,false,actual.errors.join('\n'));
  assert.equal(actual.attempts.length,failures+1);assert.deepEqual(actual.payload,Buffer.from(Array.from({length:256},(_,i)=>i)));
  assert.equal(actual.rawRemoved,true);assert.equal(actual.steps.extract.outcome,'success');
  assert.deepEqual(actual.backoffs,['2','4'].slice(0,failures));
});
test('persistent transport failure exhausts exactly three attempts and never extracts partial archives',()=>{
  const actual=replay(10);assert.equal(actual.failed,true);assert.equal(actual.attempts.length,3);
  assert.equal(actual.steps.extract.outcome,'skipped');assert.equal(actual.payload,null);assert.equal(actual.rawRemoved,true);
});
test('cancellation prevents another download but still cleans its partial archive',()=>{
  const actual=replay(10,{cancelAfterFailure:true});assert.equal(actual.cancelled,true);
  assert.equal(actual.attempts.length,1);assert.equal(actual.steps.extract.outcome,'skipped');
  assert.equal(actual.payload,null);assert.equal(actual.rawRemoved,true);assert.deepEqual(actual.backoffs,[]);
});
test('a successful transport with a corrupt ZIP fails extraction without retrying or publishing a path',()=>{
  const actual=replay(0,{corrupt:true});assert.equal(actual.failed,true);assert.equal(actual.attempts.length,1);
  assert.equal(actual.steps.extract.outcome,'failure');assert.equal(actual.steps.extract.outputs['download-path'],undefined);
  assert.equal(actual.payload,null);assert.equal(actual.rawRemoved,true);
});
for(const selection of [{'artifact-ids':'42,43'},{pattern:'native-*'},{}])test('retry preserves '+JSON.stringify(selection)+' selection and producer authority',()=>{
  const actual=replay(1,{selection:{name:'',...selection}});assert.equal(actual.failed,false,actual.errors.join('\n'));
  assert.equal(actual.attempts.length,2);assert.deepEqual(actual.attempts[0],actual.attempts[1]);
  for(const [key,item] of Object.entries(selection))assert.equal(actual.attempts[1][key],item);
  assert.equal(actual.attempts[1]['github-token'],'producer-token');assert.equal(actual.attempts[1]['run-id'],'123');
  assert.equal(actual.attempts[1].repository,'owner/repository');
});

test('failed archive reset refuses another transport and never extracts the stale bytes',()=>{
  const actual=replay(10,{resetFails:true});assert.equal(actual.failed,true);
  assert.equal(actual.attempts.length,1);assert.equal(actual.steps.extract.outcome,'skipped');
  assert.equal(actual.payload,null);assert.equal(actual.rawRemoved,true);
});
