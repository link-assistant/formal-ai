import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {spawnSync} from 'node:child_process';
import {runtimeFailureIdentity,checkedRuntimeFailureIdentity} from '../../../js/provenance/runtime-failure-identity.mjs';
const repository=fileURLToPath(new URL('../../../',import.meta.url));
test('runtime failure diagnostics retain only a code and known project module coordinates',()=>{
 const error=new Error('FreshReplayLiteralJournalAuthorityMismatch');
 error.stack='Error: private secret\n at '+pathToFileURL(path.join(repository,'js/provenance/fresh-recompute.mjs')).href+':123:7\n at file:///private/personal/secret.mjs:4:2';
 assert.deepEqual(runtimeFailureIdentity(error,repository),{name:'Error',code:'FreshReplayLiteralJournalAuthorityMismatch',
  module:'js/provenance/fresh-recompute.mjs',line:123,column:7});
 const privateError=new Error('private message /personal/secret');privateError.name='PrivateUserPayload';privateError.code='PRIVATE_SECRET';privateError.stack='Error at file:///private/personal/secret.mjs:4:2';
 assert.deepEqual(runtimeFailureIdentity(privateError,repository),{name:'Error',code:'UnclassifiedRuntimeFailure'});
 for(const changed of [{name:'Error',code:'Private path /secret'},
  {name:'Error',code:'RuntimeChildFailure',module:'../secret.mjs',line:1,column:1},
  {name:'Error',code:'RuntimeChildFailure',stack:'private'}])assert.equal(checkedRuntimeFailureIdentity(changed),null);
});
test('the actual fixed child entry retains a safe rejection identity and nonzero exit without public raw stderr',context=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'runtime-failure-fixture-'));
 context.after(()=>fileSystem.rmSync(directory,{recursive:true,force:true}));
 const resultFile=path.join(directory,'result.json'),configuration=path.join(directory,'config.json');
 fileSystem.writeFileSync(path.join(directory,'entry.cjs'),fileSystem.readFileSync(path.join(repository,'js/provenance/runtime-entry.js')));
 fileSystem.writeFileSync(path.join(directory,'main.mjs'),"throw Error('FreshReplayLiteralJournalAuthorityMismatch');");
 fileSystem.writeFileSync(configuration,JSON.stringify({repository,resultFile}));
 const child=spawnSync(process.execPath,['--max-old-space-size=384',path.join(directory,'entry.cjs')],{
  encoding:'utf8',timeout:5000,env:{...process.env,NODE_OPTIONS:'',FORMAL_AI_RUNTIME_CONFIG:configuration}});
 assert.equal(child.status,1);assert.equal(child.stdout,'');
 assert.match(child.stderr,/FreshReplayLiteralJournalAuthorityMismatch/u);
 const observed=JSON.parse(fileSystem.readFileSync(resultFile,'utf8'));
 assert.deepEqual(observed,{runtimeFailure:{name:'Error',code:'FreshReplayLiteralJournalAuthorityMismatch'}});
 assert.equal(JSON.stringify(observed).includes(directory),false);
});
