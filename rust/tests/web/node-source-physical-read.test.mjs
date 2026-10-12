import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {createNodeSourceSessionHost} from '../../../js/server/node-source-session-host.mjs';
import {test} from 'node:test';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
async function physicalReadControls() {
await installNodeHost(new WorkerHost());
const root=fs.realpathSync(new URL('../../..',import.meta.url));
const request =
  "In this selected-file scratch workspace, add exported selectedSummary(selection) to repair-summary.mjs. Return the compact canonical summary of the " +
  "candidate chosen by the selection, or null when no candidate is selected. Discover and compose existing exports from the observed " +
  "solver_formalization.mjs and translation_formalization.mjs source modules; reuse their semantics through imports rather than copying their " +
  "implementations. Read the source modules and destination before authoring. Produce a bounded repair plan identifying actual declaration spans/content " +
  "identities, arguments/results, import bindings/dependencies, the optional-value guard and effects. Run the supplied selected-summary.test.mjs " +
  "acceptance command, node --test selected-summary.test.mjs, and report the real result; do not modify source modules or tests, commit, or push. If safe " +
  "composition cannot be established, record the exact missing capability before any output write.";
const session=createNodeSourceSessionHost(p=>fs.readFileSync(root+'/'+p,'utf8'));
installHost({...host(),sourceSession:session,sourceOperation:session.sourceOperation});
const directory=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-private-read-')));
const canonical=directory+'/canonical';fs.mkdirSync(canonical);
const solver='export function selectedCandidate(selection) { return selection; }\n';
const translation='export function candidateCompactSummary(candidate) { return candidate; }\n';
fs.writeFileSync(directory+'/solver_formalization.mjs',solver);
fs.writeFileSync(directory+'/translation_formalization.mjs',translation);
fs.writeFileSync(canonical+'/solver_formalization.mjs',solver);
fs.writeFileSync(canonical+'/translation_formalization.mjs',translation);
fs.writeFileSync(directory+'/selected-summary.test.mjs',
  `import {selectedCandidate} from '${canonical}/solver_formalization.mjs';\n`
  +`import {candidateCompactSummary} from '${canonical}/translation_formalization.mjs';\n`);
let token,declaration;
const call={tool:'read',arguments:JSON.stringify({file_path:'solver_formalization.mjs'})};
function messages(id='owned-read'){return [{role:'user',content:request},
  {role:'assistant',tool_calls:[{id,function:{name:call.tool,arguments:call.arguments}}]}];}
try {
  assert.throws(()=>session.sourceOperation.physicalReadObservation({},request,directory,{}));
  await session.run({request,workspace:directory,tools:['read','write','bash']},async()=>{
    assert.equal(session.sourceOperation.hasContext(),true);
    const destinationCall = {tool:'read', arguments:JSON.stringify({file_path:'repair-summary.mjs'})};
    const destinationFrames = [{role:'user',content:request}, {role:'assistant',tool_calls:[{
      id:'destination',function:{name:'read',arguments:destinationCall.arguments}}]}];
    const diagnostic = {content:'actual diagnostic',source_read:{path:'repair-summary.mjs',success:false,error_code:'ENOENT'}};
    let fallbackCount = 0;
    const destinationResult = session.executeResult(destinationCall,destinationFrames,()=>{
      fallbackCount++; return diagnostic;
    });
    assert.equal(destinationResult,diagnostic);
    assert.equal(fallbackCount,1);
    assert.equal(Object.hasOwn(destinationResult,'owned_source_read'),false);
    const forged = session.executeResult(destinationCall,destinationFrames,()=>({owned_source_read:{}}));
    assert.equal(forged.is_error,true);
    assert.equal(forged.content,'UnexpectedDestinationReadAuthority');
    const failedDestination = session.executeResult(destinationCall,destinationFrames,()=>({is_error:true,content:'real driver failure'}));
    assert.equal(failedDestination.is_error,true);
    assert.equal(failedDestination.content,'real driver failure');
    const wrongDestination = structuredClone(destinationFrames);
    wrongDestination[0].content='cached prior request';
    assert.equal(session.executeResult(destinationCall,wrongDestination,()=>{
      throw Error('unauthorized fallback');
    }).is_error,true);
    const wrongDeclaration = structuredClone(destinationFrames);
    wrongDeclaration[1].tool_calls[0].function.name='bash';
    assert.equal(session.executeResult(destinationCall,wrongDeclaration,()=>{
      throw Error('undeclared fallback');
    }).is_error,true);
    const wrong=messages('wrong-request');wrong[0].content='cached prior request';
    assert.equal(session.executeResult(call,wrong,()=>{throw Error('fallback');}).is_error,true);
    const mismatch=messages('mismatch');mismatch[1].tool_calls[0].function.name='bash';
    assert.equal(session.executeResult(call,mismatch,()=>{throw Error('fallback');}).is_error,true);
    const receipt=session.executeResult(call,messages(),()=>{throw Error('fallback must not mint');});
    assert.equal(receipt.is_error,undefined);
    token=receipt.owned_source_read;
    declaration={id:'owned-read',name:'read',arguments:call.arguments};
    const observe=()=>session.sourceOperation.physicalReadObservation(token,request,directory,declaration);
    assert.equal(observe().content,solver);
    assert.equal(observe().kind,'OnlyPhysicalRead');
    const unrelated = fs.mkdtempSync(path.join(os.tmpdir(),'physical-read-unrelated-'));
    fs.mkdirSync(directory+'/unrelated-sibling');
    fs.writeFileSync(unrelated+'/noise.txt','unrelated exact bytes');
    assert.equal(observe().content,solver);
    assert.equal(observe().readRaceProof,false);
    fs.rmSync(unrelated,{recursive:true});
    fs.rmSync(directory+'/unrelated-sibling',{recursive:true});
    assert.equal(observe().content,solver);
    const relocated = directory+'-relocated';
    fs.renameSync(directory,relocated);
    fs.symlinkSync(relocated,directory);
    try { assert.throws(observe); }
    finally { fs.unlinkSync(directory);fs.renameSync(relocated,directory); }
    assert.equal(observe().content,solver);
    fs.renameSync(directory,relocated);
    fs.mkdirSync(directory);
    fs.writeFileSync(directory+'/solver_formalization.mjs',solver);
    try { assert.throws(observe); }
    finally { fs.rmSync(directory,{recursive:true});fs.renameSync(relocated,directory); }
    assert.equal(observe().content,solver);

    assert.equal(observe().callEffects,'Unknown');
    for(const forged of [{},JSON.parse(JSON.stringify(token)),receipt.source_read]) {
      assert.throws(()=>session.sourceOperation.physicalReadObservation(forged,request,directory,declaration));
    }
    assert.throws(()=>session.sourceOperation.physicalReadObservation(token,request+' changed',directory,declaration));
    assert.throws(()=>session.sourceOperation.physicalReadObservation(token,request,directory,{...declaration,name:'bash'}));
    fs.writeFileSync(directory+'/solver_formalization.mjs',solver+'// drift\n');
    assert.throws(observe);fs.writeFileSync(directory+'/solver_formalization.mjs',solver);
    const latest=session.executeResult(call,messages('latest'),()=>{throw Error('fallback');});
    assert.throws(observe);
    token=latest.owned_source_read;declaration={...declaration,id:'latest'};
    fs.unlinkSync(directory+'/solver_formalization.mjs');
    const failed=session.executeResult(call,messages('failed'),()=>{throw Error('fallback');});
    assert.equal(failed.is_error,true);fs.writeFileSync(directory+'/solver_formalization.mjs',solver);
    assert.throws(observe);
    fs.unlinkSync(directory+'/solver_formalization.mjs');fs.symlinkSync(canonical+'/solver_formalization.mjs',directory+'/solver_formalization.mjs');
    assert.equal(session.executeResult(call,messages('symlink'),()=>{throw Error('fallback');}).is_error,true);
  });
  assert.throws(()=>session.sourceOperation.physicalReadObservation(token,request,directory,declaration));
  await session.run({request:'Review an unknown cached example',workspace:directory,tools:['read','write','bash']},async()=>{
    assert.throws(()=>session.sourceOperation.physicalReadObservation(token,request,directory,declaration));
  });
  console.log('PHYSICAL_READ_LIVE_HOST_CONTROLS_PASS');
} finally {fs.rmSync(directory,{recursive:true,force:true});}

}
const childFlag = '--physical-read-observer-child';
if (process.argv[2] === childFlag) {
  assert.equal(process.argv.length, 3);
  assert.equal(process.env.NODE_TEST_CONTEXT, undefined);
  await physicalReadControls();
} else {
  test('isolated physical-read host controls preserve production provider guards', context => {
    assert.ok(process.env.NODE_TEST_CONTEXT === undefined || process.env.NODE_TEST_CONTEXT === 'child-v8');
    const file = fs.realpathSync(fileURLToPath(import.meta.url));
    const hash = bytes => createHash('sha256').update(bytes).digest('hex');
    const before = hash(fs.readFileSync(file));
    const env = {...process.env};
    delete env.NODE_TEST_CONTEXT;
    const argv = [file, childFlag];
    const result = spawnSync(process.execPath, argv, {
      env, encoding: 'utf8', timeout: 30000, maxBuffer: 1048576
    });
    context.diagnostic(JSON.stringify({kind: 'OnlyTestHarness', executable: process.execPath, argv,
      parentHarnessContext: process.env.NODE_TEST_CONTEXT ?? null,
      environmentDifference: {NODE_TEST_CONTEXT: 'removed only the Node test harness marker'},
      timeoutMilliseconds: 30000, maxBuffer: 1048576, exitCode: result.status,
      signal: result.signal, error: result.error?.code ?? null,
      stdout: result.stdout, stderr: result.stderr, sourceSHA256: before}));
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    assert.ok(result.stdout.includes('PHYSICAL_READ_LIVE_HOST_CONTROLS_PASS'));
    assert.equal(hash(fs.readFileSync(file)), before);
  });
}
