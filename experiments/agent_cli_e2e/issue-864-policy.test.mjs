import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {before,test} from 'node:test';
import {encodeTyped,MAXIMUM_LOG_RECORD_BYTES} from '../../js/server/structured-log.mjs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {appendInvitation} from '../../js/agentic/crate/failure_reporting.mjs';
import {TASK,COMMAND,semanticServerLog,assertAgentFailure,assertFailureEvidence} from './assert-issue-864.mjs';
before(async()=>{await installNodeHost(new WorkerHost());});
const final=`The command failed: Exit code 127\n/bin/sh: 1: ${COMMAND}: not found`;
const typed=message=>encodeTyped({schema:'formal-ai-diagnostic-v1',kind:'native_diagnostic',message});
const commandTrace=`[trace] agentic_outcome: planned ToolCalls([PlannedToolCall { tool: "bash", arguments: ${JSON.stringify(JSON.stringify({command:COMMAND}))} }])`;
const lines=['[trace] POST /api/openai/v1/chat/completions (44000 byte body)',commandTrace,'[trace] POST /api/openai/v1/chat/completions (45000 byte body)',`[trace] agentic_outcome: planned Final(${JSON.stringify(final)})`];
const evidence=()=>({task:TASK+'\n',final:final+'\n',serverRaw:lines.map(typed).join('\n')+'\n',agentStream:JSON.stringify({type:'message',role:'assistant',content:[{type:'text',text:final}]})+'\n'});
test('canonical encoded failure evidence retains the original command, final, and two POST rounds',()=>{assert.equal(assertFailureEvidence(evidence()).posts,2);});
for(const invitation of ['Would you like me to prepare an issue report with the diagnostic context? Reply `Report issue`.','Report issue','report ISSUE'])test('agent final rejects '+invitation,()=>{assert.throws(()=>assertAgentFailure(final+'\n'+invitation),/interactive/);});
for(const [name,mutate,pattern]of [
 ['exit status',e=>e.final=e.final.replace('127','0'),/127/],
 ['missing-command cause',e=>e.final=e.final.replace(': not found',': unavailable'),/missing-command/],
 ['original task',e=>e.task='Run false',/task/],
 ['real Agent final',e=>e.agentStream='',/assistant event/],
 ['extracted final custody',e=>e.agentStream=JSON.stringify({type:'message',role:'assistant',content:[{type:'text',text:final+' stale'}]}),/does not match/],
 ['planned command',e=>e.serverRaw=e.serverRaw.replace(typed(commandTrace),typed(commandTrace.replace(COMMAND,'false'))),/arguments/],
 ['planned final',e=>e.serverRaw=e.serverRaw.replace(typed(lines[3]),typed('not a final')),/planned final/],
 ['two POST rounds',e=>e.serverRaw=e.serverRaw.replace(typed(lines[0]),typed('large_native_diagnostic')),/round trip/],
 ['no auto-created issue',e=>e.serverRaw+='\n'+typed('gh issue create --title accidental'),/auto-creation/],
 ['malformed typed line',e=>e.serverRaw+='\n(formal-ai-log-v1 broken',/invalid typed/],
 ['noncanonical typed record',e=>e.serverRaw+='\n '+typed('extra whitespace'),/invalid typed/]
])test('negative evidence refuses missing '+name,()=>{const e=evidence();mutate(e);assert.throws(()=>assertFailureEvidence(e),pattern);});
test('typed decoder keeps its maintained default record-size limit',()=>{assert.equal(MAXIMUM_LOG_RECORD_BYTES,1024*1024);assert.throws(()=>semanticServerLog('(formal-ai-log-v1 '+ 'x'.repeat(MAXIMUM_LOG_RECORD_BYTES)+')'),/record_size_limit/);});
test('untyped or silently dropped records cannot stand in for canonical server diagnostics',()=>{assert.throws(()=>semanticServerLog(lines.join('\n')),/typed server records missing/);});
test('interactive chat retains the exact original opt-in invitation and Report issue oracle',()=>{
 assert.equal(appendInvitation('The command failed: Exit code 127','en'),'The command failed: Exit code 127\n\nI detected a failure while working on this request. Would you like me to prepare an issue report with the diagnostic context? Reply `Report issue`.');
});
test('real shell entry keeps actual Agent CLI, strict stderr policy, task and evidence assertion wired',()=>{
 const file=path.join(path.dirname(fileURLToPath(import.meta.url)),'run_issue_864.sh'),script=fs.readFileSync(file,'utf8');
 for(const needle of ['TASK="Run issue_864_command_that_does_not_exist"','"$AGENT" \\','--disable-stdin','filter-expected-agent-tool-failure.py','classify-agent-cli-stderr.sh','extract-final.mjs','assert-issue-864.mjs'])assert.ok(script.includes(needle),needle);
 assert.doesNotMatch(script,/failed command invited an opt-in report/);
});

test('elided requests count truthful native HTTP metadata without inline-header duplication',()=>{
 const e=evidence();const elided=encodeTyped({schema:'formal-ai-diagnostic-v1',message:'large_native_diagnostic',metadata:{http_method:'POST',http_path:'/api/openai/v1/chat/completions',body_bytes:47000}});
 e.serverRaw=e.serverRaw.replace(typed(lines[0]),elided);assert.equal(assertFailureEvidence(e).posts,2);
 const inline=encodeTyped({schema:'formal-ai-diagnostic-v1',message:lines[0],metadata:{http_method:'POST',http_path:'/api/openai/v1/chat/completions',body_bytes:47000}});
 e.serverRaw=e.serverRaw.replace(elided,inline);assert.equal(assertFailureEvidence(e).posts,2);
});
for(const metadata of [
 {http_method:'POST',http_path:'/api/openai/v1/chat/completions',body_bytes:-1},
 {http_method:'POST',http_path:'/api/openai/v1/chat/completions',body_bytes:'47000'},
 {http_method:'POST',http_path:'/api/openai/v1/chat/completions\nPOST /api/openai/v1/chat/completions',body_bytes:47000},
 {method:'POST',path:'/api/openai/v1/chat/completions',bytes:47000}
])test('invalid elision metadata cannot invent an HTTP round trip '+JSON.stringify(metadata),()=>{
 const e=evidence();e.serverRaw=e.serverRaw.replace(typed(lines[0]),encodeTyped({schema:'formal-ai-diagnostic-v1',message:'large_native_diagnostic',metadata}));
 assert.throws(()=>assertFailureEvidence(e),/round trip/);
});
test('request-body text cannot inflate HTTP request count',()=>{
 const e=evidence();e.serverRaw=e.serverRaw.replace(typed(lines[0]),typed('[trace] GET /health (50 byte body)\n'+lines[0]));
 assert.throws(()=>assertFailureEvidence(e),/round trip/);
});
test('complete native/dialog artifacts survive temporary workspace cleanup',()=>{
 const script=fs.readFileSync(path.join(path.dirname(fileURLToPath(import.meta.url)),'run_issue_864.sh'),'utf8');
 assert.ok(script.includes('FORMAL_AI_DIALOG_LOG_DIR="$OUT/dialog-logs"'));assert.ok(script.includes('FORMAL_AI_LOG_DIR="$OUT/native-logs"'));
 assert.ok(script.includes('node --test "$ROOT/experiments/agent_cli_e2e/issue-864-policy.test.mjs"'));
});
