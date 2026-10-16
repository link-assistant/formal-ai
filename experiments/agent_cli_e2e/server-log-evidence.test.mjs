import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {encodeTyped,MAXIMUM_LOG_RECORD_BYTES} from '../../js/server/structured-log.mjs';
import {encodeDialogRecord} from '../../js/server/dialog-codec.mjs';
import {projectServerLog,exchangeResponses} from './server-log-evidence.mjs';
const typed=(message,metadata={})=>encodeTyped({schema:'formal-ai-diagnostic-v1',message,metadata});
const header='[trace] POST /v1/chat/completions (50000 byte body)';
const metadata={http_method:'POST',http_path:'/v1/chat/completions',body_bytes:50000};
test('short and elided actual request records each contribute one header',()=>{
 const result=projectServerLog(typed(header,metadata)+'\n'+typed('large_native_diagnostic',metadata),{completionPath:'/v1/chat/completions'});assert.equal(result.posts,2);
});
test('the original endpoint minimums remain distinct across client surfaces',()=>{
 const raw=typed(header)+'\n'+typed('[trace] POST /api/openai/v1/chat/completions (40 byte body)');
 assert.equal(projectServerLog(raw).posts,1);assert.equal(projectServerLog(raw,{completionPath:'/v1/chat/completions'}).posts,1);
 assert.throws(()=>projectServerLog(raw,{completionPath:'/arbitrary'}),/unsupported/);
});
test('request bodies and plain text cannot fabricate additional POST evidence',()=>{
 const raw=typed(header+'\n'+header)+'\n'+header+'\n'+typed('[trace] GET /health (30 byte body)\n'+header);
 assert.equal(projectServerLog(raw,{completionPath:'/v1/chat/completions'}).posts,1);
});
for(const invalid of [{...metadata,body_bytes:-1},{...metadata,body_bytes:'50000'},{...metadata,http_path:'/v1/chat/completions\n'+header},{method:'POST',path:'/v1/chat/completions',bytes:50000}])test('invalid elision metadata cannot invent rounds '+JSON.stringify(invalid),()=>{assert.equal(projectServerLog(typed('large_native_diagnostic',invalid),{completionPath:'/v1/chat/completions'}).posts,0);});
test('decoded planner command text remains available to EXPECT_SERVER_TEXTS',()=>{
 const raw=typed('[trace] agentic_outcome: planned ToolCalls formal-ai learn algorithms')+'\n'+typed('[trace] formal-ai algorithm conformance');
 const result=projectServerLog(raw);assert.match(result.semantic,/formal-ai learn algorithms/);assert.match(result.semantic,/formal-ai algorithm conformance/);assert.equal(result.posts,0);
});
for(const raw of ['(formal-ai-log-v1 broken',' '+typed('padded'),'(formal-ai-log-v1 '+'x'.repeat(MAXIMUM_LOG_RECORD_BYTES)+')'])test('malformed, noncanonical or oversized diagnostics fail closed '+raw.slice(0,30),()=>{assert.throws(()=>projectServerLog(raw));});
test('an undecoded legacy log cannot stand in for current canonical diagnostics',()=>{assert.throws(()=>projectServerLog(header),/typed server records missing/);assert.equal(MAXIMUM_LOG_RECORD_BYTES,1024*1024);});
function withDirectory(run){const dir=fs.mkdtempSync(path.join(os.tmpdir(),'server-log-projection-'));try{return run(dir);}finally{fs.rmSync(dir,{recursive:true,force:true});}}
test('actual complete response bodies preserve elided sidecar evidence without counting rounds',()=>withDirectory(dir=>{
 const body='data: {"tool_calls":[{"arguments":"write /sidecar/leaked.txt"}]}\n\n';
 const raw=encodeDialogRecord({method:'POST',path:'/v1/chat/completions',response_body:body,request_body:'original request'})+'\n';
 const file=path.join(dir,'dialog.lino');fs.writeFileSync(file,raw);assert.equal(exchangeResponses(dir),body);assert.equal(fs.readFileSync(file,'utf8'),raw);
 assert.equal(projectServerLog(typed('large_native_diagnostic')).posts,0);
}));
for(const [name,raw]of [['malformed','(formal-ai-log-v1 broken\n'],['partial',encodeDialogRecord({method:'POST',path:'/v1/chat/completions',response_body:'data'})],['missing-body',encodeDialogRecord({method:'POST',path:'/v1/chat/completions'})+'\n']])test('incomplete persisted exchange evidence is refused '+name,()=>withDirectory(dir=>{fs.writeFileSync(path.join(dir,'dialog.lino'),raw);assert.throws(()=>exchangeResponses(dir));}));
test('no stored exchanges cannot pass an emitted-path negative oracle',()=>withDirectory(dir=>{assert.throws(()=>exchangeResponses(dir),/actual_dialog_exchange/);}));
test('all audited recipe consumers use shared projection while retaining actual tasks and raw artifacts',()=>{
 const here=path.dirname(fileURLToPath(import.meta.url));
 for(const name of ['run_agent_cli.sh','run_issue_687.sh','run_issue_715_opencode.sh','run_issue_758.sh','run_issue_771.sh','run_issue_822.sh','run_issue_1075.sh']){
  const script=fs.readFileSync(path.join(here,name),'utf8');assert.ok(script.includes('server-log-evidence.sh'),name);assert.ok(script.includes('project_server_log'),name);assert.ok(script.includes('retain_server_log_evidence'),name);
  assert.doesNotMatch(script,/grep[^\n]*['"]POST[^\n]*"\$(?:LOG|SERVER_LOG)"/,name);
 }
 const generic=fs.readFileSync(path.join(here,'run_agent_cli.sh'),'utf8');
 for(const needle of ['MIN_POSTS="${MIN_POSTS:-4}"','EXPECT_SERVER_TEXTS','"$SEMANTIC_LOG"','"$AGENT_SETUP_LOG" 2>&1','cp "$LOG" "$ARTIFACT_DIR/formal-ai.log"'])assert.ok(generic.includes(needle),needle);
 const sidecar=fs.readFileSync(path.join(here,'run_issue_1075.sh'),'utf8');assert.ok(sidecar.includes('-v "$EVIDENCE_DIR:/evidence"'));assert.ok(sidecar.includes('grep -Eo'));assert.ok(sidecar.includes('"$SEMANTIC_LOG"'));
});

test('missing completed exchanges cannot satisfy already observed HTTP POSTs',()=>withDirectory(dir=>{
 fs.writeFileSync(path.join(dir,'dialog.lino'),encodeDialogRecord({method:'POST',path:'/v1/chat/completions',response_body:'actual'})+'\n');
 assert.throws(()=>exchangeResponses(dir,{minimumPosts:2}),/missing_completed/);
}));
test('all-endpoint reporting counts actual POST headers once across both APIs',()=>{
 const raw=typed(header)+'\n'+typed('[trace] POST /v1/messages (3 byte body)');assert.equal(projectServerLog(raw,{completionPath:'*'}).posts,2);
});
