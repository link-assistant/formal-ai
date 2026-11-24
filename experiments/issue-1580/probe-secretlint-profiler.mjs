// Bounded real-engine probe: no credentials, 100 tiny inputs, no Rust build.
import assert from 'node:assert/strict';
import {performance} from 'node:perf_hooks';
import {createRequire} from 'node:module';
import {resolve} from 'node:path';
import {setImmediate} from 'node:timers/promises';
import {scannerModule,createScanner} from '../../scripts/check-secrets-files.mjs';
const baseline=process.argv.includes('--baseline');
const module=scannerModule('13.0.5');
const engine=baseline?await (await import(module)).createEngine({cwd:process.cwd(),formatter:'stylish',color:false,maskSecrets:true}):await createScanner('13.0.5');
for(let index=0;index<100;index++) {
  const result=await engine.executeOnContent({content:'ordinary evidence\n',filePath:resolve('profile-control.txt')});
  assert.equal(result.ok,true);
}
await setImmediate();
const {secretLintProfiler}=await import(createRequire(module).resolve('@secretlint/profiler'));
const marks=performance.getEntriesByType('mark').length;
console.log(JSON.stringify({inputs:100,baseline,profilerEnabled:secretLintProfiler.isEnabled,retainedMarks:marks}));
if(baseline)assert(marks>100);else assert.equal(marks,0);
