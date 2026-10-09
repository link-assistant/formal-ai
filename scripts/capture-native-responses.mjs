#!/usr/bin/env node
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,chmodSync,existsSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {buildIdentity,verifyReceipt} from './native-test-artifact.mjs';
import {augmentNativeTestIdentity} from './lib/native-response-identity.mjs';
import {executeBatches} from './lib/coverage-execution.mjs';
import {readTestDurations} from './lib/ci-speed-durations.mjs';
import {DEFAULT_SECONDS} from './lib/ci-speed-shards.mjs';
import {planNativeResponseCaptures} from './lib/native-response-scheduling.mjs';
import {collectNativeResponseRecords} from './lib/native-response-capture.mjs';
import {readNativeTestModuleGraph,bindOriginalSourceCase} from './lib/native-test-module-graph.mjs';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
/** Run unchanged, uniquely listed original programs. Observed answers never update expected test answers. */
export async function captureOriginalTests({cwd,directory,binary,outputDirectory,selections,expectedIdentity}) {
 assert.deepEqual(augmentNativeTestIdentity(cwd,expectedIdentity),expectedIdentity,'current physical source and flags differ from producer');
 const receipt=verifyReceipt(directory,expectedIdentity,binary);
 assert.ok(Array.isArray(selections)&&selections.length>0);
 assert.equal(new Set(selections).size,selections.length,'duplicate selected native source case');
 const graph=readNativeTestModuleGraph(cwd);
 const cases=selections.map(id=>{assert.match(id,/^(rust\/tests\/(unit|integration|source)\/[\w/.-]+\.rs)::([A-Za-z_][A-Za-z_0-9]*)$/u,'explicit original source case required');return {id,...bindOriginalSourceCase(graph,id)};});
 const executables=new Map(),listing=new Map();
 for(const target of new Set(cases.map(c=>c.target))){const path=resolve(directory,target);chmodSync(path,0o755);executables.set(target,path);
  const text=execFileSync(path,['--list','--format','terse'],{cwd,encoding:'utf8',maxBuffer:10e6});
  listing.set(target,text.split(/\r?\n/u).filter(line=>line.endsWith(': test')).map(line=>line.slice(0,-6)));
 }
 for(const c of cases){const names=listing.get(c.target).filter(name=>name===c.caller);assert.equal(names.length,1,'original source caller must be uniquely listed: '+c.id);c.callerListedByCheckedExecutable=true;}
 assert.equal(existsSync(outputDirectory),false,'fresh capture directory required');mkdirSync(outputDirectory,{recursive:true});
 writeFileSync(join(outputDirectory,'producer-receipt.json'),JSON.stringify(receipt,null,2)+'\n');
 const durationSource='data/meta/test-durations.lino',durationBytes=readFileSync(join(cwd,durationSource));
 const fallbackSeconds=Number(/^\s+default-seconds (\d+(?:\.\d+)?)$/mu.exec(durationBytes.toString('utf8'))?.[1]??DEFAULT_SECONDS);
 const runtimeListing=[...listing].flatMap(([target,names])=>names.map(name=>target+'\t'+name));
 const ordered=planNativeResponseCaptures(cases,runtimeListing,executables,readTestDurations(join(cwd,durationSource)),fallbackSeconds);
 const observations=[],launchOrder=[];
 for(let index=0;index<ordered.length;index++){
  const c=ordered[index],folder=join(outputDirectory,String(index).padStart(3,'0'));mkdirSync(folder);
  const stdout=[],stderr=[];
  const execution=await executeBatches([c.batch],executables,{
   cwd,threads:1,concurrency:1,nocapture:true,
   env:{...process.env,FORMAL_AI_NATIVE_RESPONSE_CAPTURE_DIR:folder},
   onStart:batch=>launchOrder.push({dispatchIndex:index,id:c.id,target:batch.target,
    caller:batch.names[0],weight:batch.weight}),
   onStdout:b=>stdout.push(b),onStderr:b=>stderr.push(b)});
  const stdoutBytes=Buffer.concat(stdout),stderrBytes=Buffer.concat(stderr);writeFileSync(join(folder,'stdout.bin'),stdoutBytes);writeFileSync(join(folder,'stderr.bin'),stderrBytes);
  const actual=execution.batches[0],file=join(folder,'native-'+actual.processId+'.jsonl');let captured=null,error=null;
  try{assert.doesNotMatch(stderrBytes.toString('utf8'),/^\[native-response-observation\] failed to retain actual response:/mu,'actual observer reported a persistence error');captured=collectNativeResponseRecords(readFileSync(file),{caller:c.caller,processId:actual.processId,execution:actual,identity:receipt.identity,expectedIdentity});}
  catch(failure){error=failure.message;}
  const record={...c,execution,captured,error,'stdout-sha256':sha(stdoutBytes),'stderr-sha256':sha(stderrBytes),'raw-file':existsSync(file)?'native-'+actual.processId+'.jsonl':null};
  writeFileSync(join(folder,'report.json'),JSON.stringify(record,null,2)+'\n');observations.push(record);
 }
 assert.deepEqual(augmentNativeTestIdentity(cwd,expectedIdentity),expectedIdentity,'physical source changed during native capture');
 verifyReceipt(directory,expectedIdentity,binary);
 const report={schema:'native-original-response-capture/v1',identity:expectedIdentity,originalAssertionsUnchanged:true,expectedAnswersUpdated:false,
  selected:cases.length,accepted:observations.filter(r=>r.captured!==null).length,observations,
  dispatch:{policy:'measured-longest-first',durationSource,durationSourceSha256:sha(durationBytes),fallbackSeconds,threads:1,concurrency:1,launchOrder},
  sourceRegistration:{targets:graph.targets,sourceFiles:graph.sources,unsupportedEdges:graph.unsupported},
  completeness:'Actual observed returns at the two instrumented boundaries; unbound threads are retained separately, never assigned to a test by inference.'};
 writeFileSync(join(outputDirectory,'report.json'),JSON.stringify(report,null,2)+'\n');return report;
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 const [directory,binary,outputDirectory,selectionFile,profile,features]=process.argv.slice(2);assert.ok(directory&&binary&&outputDirectory&&selectionFile&&profile&&features);
 const report=await captureOriginalTests({cwd:process.cwd(),directory:resolve(directory),binary:resolve(binary),outputDirectory:resolve(outputDirectory),
  selections:JSON.parse(readFileSync(selectionFile,'utf8')).cases,expectedIdentity:buildIdentity(process.cwd(),profile,features)});
 console.log(JSON.stringify({selected:report.selected,accepted:report.accepted,expectedAnswersUpdated:false}));if(report.accepted!==report.selected)process.exitCode=1;
}
