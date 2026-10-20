import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {createLogger,decodeTyped,encodeTyped,makeRedactor,MAXIMUM_LOG_RECORD_BYTES} from '../../../js/server/structured-log.mjs';
import {boundedDiagnosticMetadata,DIAGNOSTIC_FIELD_BYTES,DIAGNOSTIC_METADATA_BYTES,DIAGNOSTIC_PREVIEW_BYTES} from '../../../js/logging/diagnostic-metadata.mjs';
import {projectServerLog} from '../../../experiments/agent_cli_e2e/server-log-evidence.mjs';
const digest=value=>createHash('sha256').update(value).digest('hex');
function observe(metadata,{environment={},level='debug',payload}={}) {
 const chunks=[],logger=createLogger({level,env:environment,stderrWrite:bytes=>(chunks.push(Buffer.from(bytes)),bytes.length)});
 const result=logger.log('debug','tool_call_bound',metadata,payload);logger.close();
 assert.equal(result.emitted,true);const raw=Buffer.concat(chunks).toString();
 assert.ok(Buffer.byteLength(raw)<=MAXIMUM_LOG_RECORD_BYTES,'actual percent-encoded record fits unchanged reader');
 return {raw,record:decodeTyped(raw.trimEnd())};
}
test('full CST diagnostic arguments are summarized without losing planner counters or changing the original data',()=>{
 const argumentsText='("syntax" "😀" "é" "%" "()")\n'.repeat(30000);
 const metadata={component:'planner',tool:'javascript_cst',state:'awaiting_client_receipt',repeat_limit:3,arguments:argumentsText};
 const original={...metadata};const {raw,record}=observe(metadata);
 assert.deepEqual(metadata,original);assert.equal(metadata.arguments,argumentsText);
 assert.equal(record.metadata.tool,metadata.tool);assert.equal(record.metadata.state,metadata.state);assert.equal(record.metadata.repeat_limit,3);
 const summary=record.metadata.arguments;
 assert.equal(summary.bytes,Buffer.byteLength(argumentsText));assert.equal(summary.sha256,digest(argumentsText));
 assert.equal(summary.encoding,'utf8');assert.equal(summary.preview_truncated,true);
 assert.ok(Buffer.byteLength(summary.preview)<=DIAGNOSTIC_PREVIEW_BYTES);assert.ok(!summary.preview.includes('\uFFFD'));
 assert.deepEqual(projectServerLog(raw).diagnostics,['tool_call_bound'],'original E2E reader accepts every physical diagnostic record');
});
test('ordinary metadata and own reserved keys remain lossless',()=>{
 const metadata=JSON.parse('{"__proto__":{"polluted":"data only"},"constructor":"distinct","prototype":"third","toString":"fourth","counter":3}');
 assert.deepEqual(observe(metadata).record.metadata,metadata);
 assert.equal(Object.prototype.polluted,undefined);
 const large=JSON.parse('{"__proto__":"own value","constructor":"constructor value","payload":""}');large.payload='é'.repeat(20000);
 const {record}=observe(large);assert.equal(Object.hasOwn(record.metadata,'__proto__'),true);assert.equal(record.metadata.__proto__,'own value');assert.equal(record.metadata.constructor,'constructor value');
 assert.equal(Object.prototype.polluted,undefined);
});
test('wide keys and arrays obey aggregate encoded budget while retaining small diagnostic values',()=>{
 for(const source of [Array.from({length:10000},(_,index)=>index),Object.fromEntries(Array.from({length:1500},(_,index)=>['😀'.repeat(50)+index,'é'.repeat(80)])),{['é'.repeat(40000)]:'huge own key'}]) {
  const metadata=Array.isArray(source)?{values:source,tool:'small',state:'pending',repeat_limit:3}:{...source,tool:'small',state:'pending',repeat_limit:3};
  const {record}=observe(metadata),bounded=record.metadata;
  assert.ok(Buffer.byteLength(encodeTyped(bounded))<=DIAGNOSTIC_METADATA_BYTES);
  const fields=bounded.fields??bounded;
  assert.equal(fields.tool,'small');assert.equal(fields.state,'pending');assert.equal(fields.repeat_limit,3);
  if(!Array.isArray(source)){assert.equal(bounded.diagnostic_metadata,'bounded-post-redaction/v1');assert.ok(bounded.omitted_fields>0);assert.equal(bounded.omitted.sha256,digest(encodeTyped(metadata)));}
 }
});
test('summary digest byte counts and Unicode preview are computed only after secret redaction',()=>{
 const secret='private-secret-value',argumentsText=('prefix '+secret+' 😀é %\n').repeat(3000);
 const metadata={tool:'inspect',arguments:argumentsText,password:secret,nested:{message:argumentsText}};
 const environment={GH_TOKEN:secret};const {raw,record}=observe(metadata,{environment});
 const redacted=makeRedactor(environment)(metadata),expected=redacted.arguments;
 assert.equal(record.metadata.password,'[REDACTED]');assert.equal(record.metadata.arguments.sha256,digest(expected));
 assert.notEqual(record.metadata.arguments.sha256,digest(argumentsText));assert.equal(record.metadata.arguments.bytes,Buffer.byteLength(expected));
 assert.equal(record.metadata.nested.sha256,digest(encodeTyped(redacted.nested)));
 assert.ok(!raw.includes(secret));assert.ok(!JSON.stringify(record).includes(secret));assert.ok(record.metadata.arguments.preview.includes('[REDACTED]'));
});
test('producer budgets mirror the native twin while diagnostic reader and codec policies stay unchanged',()=>{
 const native=fs.readFileSync(new URL('../../../rust/src/structured_log.rs',import.meta.url),'utf8');
 assert.equal(DIAGNOSTIC_FIELD_BYTES,4096);assert.equal(DIAGNOSTIC_METADATA_BYTES,64*1024);assert.equal(DIAGNOSTIC_PREVIEW_BYTES,256);
 assert.match(native,/const DIAGNOSTIC_FIELD_BYTES: usize = 4096;/u);assert.match(native,/const DIAGNOSTIC_METADATA_BYTES: usize = 64 \* 1024;/u);assert.match(native,/const DIAGNOSTIC_PREVIEW_BYTES: usize = 256;/u);
 assert.equal(MAXIMUM_LOG_RECORD_BYTES,1024*1024);assert.throws(()=>decodeTyped('('.repeat(MAXIMUM_LOG_RECORD_BYTES+1)),/record_size_limit/);
 const control=native.slice(native.indexOf('pub fn control('),native.indexOf('fn write_record('));assert.ok(!control.includes('bounded_diagnostic_metadata'));
 const ordinary={tool:'x',counter:2,nested:{flag:true}};assert.deepEqual(boundedDiagnosticMetadata(ordinary),ordinary);
});
