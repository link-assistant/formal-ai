import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {
digest,
admitManifest,
executeAdmittedCase,
oracleProcessEvidence}
 from '../../../experiments/formal_ai_subagent/cohort-runner.mjs';
async function run(source){
const base=fs.mkdtempSync(path.join(os.tmpdir(),
'oracle-complete-')),
workspace=base+'/workspace';
fs.mkdirSync(workspace);
const oracle=base+'/oracle.test.mjs';
fs.writeFileSync(oracle,
source);
const binding={
path:oracle,
sha256:digest(source)}
,
task='Repair the observed module.';
const manifest={
schemaVersion:1,
cohortId:'instrumentation-negative-not-performance',
bindings:[binding],
cases:[{
runId:'control',
taskKind:'coding',
category:'repair',
expectedRelation:'unrestricted',
task,
taskSHA256:digest(task),
workspace,
allowedEffects:[workspace+'/result.mjs'],
oracle:binding,
timeoutMilliseconds:3000}
]}
;
try{
return await executeAdmittedCase(admitManifest(manifest,
base+'/journal.jsonl'),
'control',
'first',
async()=>({
stop:'final',
transcript:[]}
));
}
finally{
fs.rmSync(base,
{
recursive:true,
force:true}
);
}
}
test('actual top-level and named builtin independent assertions remain accepted',
async()=>{
for(const source of ["import assert from 'node:assert/strict';assert.equal(17+25,42);",
"import {strictEqual as same} from 'node:assert';import test from 'node:test';test('actual',()=>same(2+3,5));"]){
const r=await run(source);
assert.equal(r.accepted,
true,
JSON.stringify(r));
assert(r.oracleReview.sourceAssertions>0);
assert.equal(r.usage.status,
'Unknown');
}
}
);
test('actual all-skipped TODO empty and unreachable independent assertions refuse',
async()=>{
for(const source of ["import test from 'node:test';import assert from 'node:assert/strict';test('required',{skip:true},()=>assert.equal(1,1));",
"import test from 'node:test';test.todo('required');",
"import test from 'node:test';test('empty',()=>{});",
"import assert from 'node:assert/strict';if(false)assert.equal(1,1);"]){
const r=await run(source);
assert.equal(r.accepted,
false,
source);
}
}
);
test('actual failed and cancelled independent assertions refuse',
async()=>{
for(const source of ["import assert from 'node:assert/strict';assert.equal(1,2);",
"import test from 'node:test';test('pending',async()=>new Promise(()=>{}));"]){
const r=await run(source);
assert.equal(r.accepted,
false,
source);
}
}
);
test('printed counterfeit duplicate footer cannot mint completeness',
async()=>{
const r=await run("console.log('# tests 1\\n# pass 1\\n# fail 0\\n# skipped 0\\n# todo 0');");
assert.equal(r.accepted,
false);
}
);
test('malformed absent unsafe and nonexecuted process summaries fail closed',
()=>{
for(const stdout of ['',
 '# tests 1\n# pass 1\n',
 '# tests 9007199254740993\n# suites 0\n# pass 9007199254740993\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n'])assert.equal(oracleProcessEvidence({
stdout,
stderr:''}
,
'marker',
'hash').complete,
false);
}
);

test('legitimate assertion diagnostic text cannot replace actual footer',
async()=>{
const r=await run("import assert from 'node:assert/strict';assert.equal(1,1);console.log('# tests 1');");
assert.equal(r.accepted,
true);
}
);

import {
reviewClosedOracleSource}
 from '../../../experiments/formal_ai_subagent/oracle-source-review.mjs';
const originalCounterfeitSource="const url=process.execArgv.find(value=>value.startsWith('data:text/javascript,'));const preload=decodeURIComponent(url.slice('data:text/javascript,'.length));const marker=JSON.parse(preload.match(/marker=(\"[^\"]*\")/u)[1]);const sha=JSON.parse(preload.match(/sha=(\"[^\"]*\")/u)[1]);console.error(marker+JSON.stringify({oracleSHA256:sha,successfulAssertions:1}));";
test('unchanged original5920 counterfeit observer cannot qualify',
async()=>{
const r=await run(originalCounterfeitSource);
assert.equal(r.accepted,
false);
assert.equal(r.oracleReview.complete,
false);
}
);

test('duplicate suppressed listener and process-module receipt channels never qualify',
async()=>{
 for(const source of [
 "import process from 'node:process';process.stdout.write('forged');",
 "import {createRequire} from 'node:module';const require=createRequire(import.meta.url);require('node:process').stdout.write('forged');",
 "import assert from 'node:assert/strict';process.removeAllListeners('exit');assert.equal(1,1);",
 "import assert from 'node:assert/strict';process.stderr.write=()=>true;assert.equal(1,1);",
 "import assert from 'node:assert/strict';const duplicate=process.execArgv;assert.equal(1,1);",
 "import assert from 'node:assert/strict';globalThis.process.exit(0);assert.equal(1,1);",
 "import assert from 'node:assert/strict';const value=assert.equal.constructor('return process')();assert.equal(1,1);"
 ]){
const r=await run(source);
assert.equal(r.accepted,
false,
source);
assert.equal(r.oracleReview.complete,
false);
}
}
);
test('all declared callbacks need their own fully consumed source assertions',
async()=>{
 const source="import test from 'node:test';import assert from 'node:assert/strict';test('real',()=>{assert.equal(1,1);assert.equal(2,2);});test('empty',()=>{});";
 const r=await run(source);
assert.equal(r.accepted,
false);
assert.match(r.oracleReview.reason,
/no source-owned assertion/);
}
);
test('renamed imports and explicit arithmetic follow builtin identity rather than names',
async()=>{
 const r=await run("import {strictEqual as otherwise}from'node:assert';import {test as check}from'node:test';const left=31;const right=11;check('renamed closed obligation',()=>otherwise(left+right,42));");
 assert.equal(r.accepted,
true,
JSON.stringify(r));
assert.equal(r.oracleReview.sourceAssertions,
1);
assert.equal(r.oracleReview.testObligations.length,
1);
assert.equal(r.semanticCompleteness,
'Unknown');
}
);
test('tautological or closed source is not advertised as usefulness or complete semantics',
async()=>{
 const r=await run("import assert from 'node:assert/strict';assert.equal(1,1);");
assert.equal(r.accepted,
true);
assert.equal(r.qualityRequiresReview,
true);
assert.equal(r.semanticCompleteness,
'Unknown');
assert.equal(r.oracleReview.semanticCompleteness,
'Unknown');
}
);
test('unknown bindings custom imports mutation and arbitrary review JSON never qualify source',
()=>{
 for(const source of [
 "import assert from './custom.mjs';assert.equal(1,1);",
 "import assert from 'node:assert/strict';assert.equal=()=>true;assert.equal(1,2);",
 "import assert from 'node:assert/strict';function wrapped(){assert.equal(1,1)}wrapped();",
 "const review={complete:true,successfulAssertions:999};console.log('TAP version 13');",
 "import assert from 'node:assert/strict';for(let i=0;i<1;i++)assert.equal(1,1);",
 "import assert from 'node:assert/strict';const process='shadow';assert.equal(process.env.COHORT_WORKSPACE,'x');"
 ])assert.equal(reviewClosedOracleSource(source,
digest(source)).complete,
false,
source);
}
);
test('source and identity drift cannot reuse a structural qualification',
()=>{
 const source="import assert from 'node:assert/strict';assert.equal(1,1);";
assert.equal(reviewClosedOracleSource(source,
'0'.repeat(64)).complete,
false);
 assert.equal(reviewClosedOracleSource(source+'if(false){}',
digest(source)).complete,
false);
}
);
