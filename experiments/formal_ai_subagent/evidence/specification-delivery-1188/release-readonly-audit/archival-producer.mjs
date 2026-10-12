import {readFileSync,writeFileSync,mkdirSync,readdirSync,copyFileSync,statSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {join,relative} from 'node:path';
const scratch='/private/tmp/spec-recurrence-1188';
const target=join(process.cwd(),'experiments/formal_ai_subagent/evidence/specification-delivery-1188/release-readonly-audit');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const put=(path,bytes)=>{mkdirSync(join(path,'..'),{recursive:true});writeFileSync(path,bytes);};
const malformed=readFileSync(join(target,'README.md'));
put(join(target,'failed-authored-report/T2937-actual-malformed.md'),malformed);
for(const [input,output] of [['release-readonly-audit','network-failed'],['release-readonly-audit-retry','network-retry']]) {
 for(const name of readdirSync(join(scratch,input))) {
  const source=join(scratch,input,name);
  if(statSync(source).isFile())put(join(target,output,name),readFileSync(source));
 }
}
for(let id=2933;id<=2938;id++)for(const name of readdirSync(scratch).filter(name=>name.startsWith('T'+id+'-')||name==='T'+id+'.log'))put(join(target,'tasks',name),readFileSync(join(scratch,name)));
const tgz=join(target,'network-retry/actual-engine.tgz');
const wasm=execFileSync('tar',['-xOzf',tgz,'package/assets/formal_ai_worker.wasm']);
put(join(target,'network-retry/actual-packaged-worker.wasm'),wasm);
put(join(target,'audit-producer.mjs'),readFileSync(join(scratch,'release-readonly-audit.mjs')));
put(join(target,'archival-producer.mjs'),readFileSync(new URL(import.meta.url)));
const report=readFileSync(join(scratch,'release-readonly-findings.final.draft.md'));
put(join(target,'README.md'),report);
const metric=readFileSync(join(scratch,'metric-scope.draft.md'));
if(!readFileSync(join(target,'metric-scope.md')).equals(metric))throw new Error('metric correction bytes changed');
if(!readFileSync(join(target,'README.md')).equals(report))throw new Error('reviewed report bytes differ');
const packageObservation=JSON.parse(readFileSync(join(target,'network-retry/actual-package-observation.json')));
if(hash(readFileSync(join(target,'network-retry/formal-ai-engine-artifact.zip')))!==packageObservation.zipSha256)throw new Error('ZIP API digest mismatch');
if(hash(readFileSync(tgz))!==packageObservation.archiveSha256)throw new Error('TGZ digest mismatch');
if(wasm.length!==296906)throw new Error('actual WASM byte length mismatch');
const files=[];
function walk(dir){for(const name of readdirSync(dir).sort()){const path=join(dir,name);if(statSync(path).isDirectory())walk(path);else if(name!=='artifact-manifest.json'){const bytes=readFileSync(path);files.push({path:relative(target,path),bytes:bytes.length,sha256:hash(bytes)});}}}
walk(target);put(join(target,'artifact-manifest.json'),JSON.stringify({taskId:'T2939',originalLiteralFailure:'T2937',reviewedReportByteEqual:true,metricCorrectionByteEqual:true,wasmBytes:wasm.length,files},null,2)+'\n');
console.log(JSON.stringify({artifacts:files.length,reviewedReportByteEqual:true,metricCorrectionByteEqual:true,zipApiDigestEqual:true,tgzDigestEqual:true,actualWasmBytes:wasm.length,originalLiteralFailureRemainsOpen:true}));
