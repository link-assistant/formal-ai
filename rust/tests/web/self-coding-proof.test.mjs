import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {verifySelfCodingSession,task,target,content} from '../../examples/self-coding/verify-session.mjs';
import {installNativeCaptureObservationHost} from '../../../scripts/lib/native-capture-observation.mjs';
import {completeOwnedSourceReadFrame} from '../../../js/agentic/tool_result.mjs';
import {ownedCreationReadbackCommand} from '../../../js/agentic/progress_command_readback.mjs';
await installNativeCaptureObservationHost();
const write={tool:'write_file',arguments:{path:target,content},result:'wrote '+Buffer.byteLength(content)+' byte(s) to '+target};
const read={tool:'read_file',arguments:{path:target},result:completeOwnedSourceReadFrame(content)};
const session=(steps=[write,read])=>({task,tools_advertised:['write_file','read_file','run_command'],steps,turns:steps.length+1,hit_turn_cap:false,final_answer:'Observed exact target'});
const bytes=value=>Buffer.from(JSON.stringify(value));
test('current target-only Write and fresh complete Read require no journal',()=>{
 assert.equal(verifySelfCodingSession(bytes(session()),Buffer.from(content)).task,task);
});
test('current framed Run proves the exact authored target',()=>{
 const command=ownedCreationReadbackCommand(target);
 const run={tool:'run_command',arguments:{command},result:JSON.stringify({schema:'command-execution-receipt/v1',command,exit_code:0,complete:true,timed_out:false,truncated:false,stdout:'('+content+')',stderr:''})};
 assert.equal(verifySelfCodingSession(bytes(session([write,run])),Buffer.from(content)).steps.length,2);
});
test('physical target comparison preserves trailing bytes',()=>{
 for(const physical of [content+'\n',content+' ',content.slice(1)])assert.throws(()=>verifySelfCodingSession(bytes(session()),Buffer.from(physical)));
});
test('current replay refuses absent failed foreign incomplete and stale observations',()=>{
 for(const steps of [[write],[read,write],[{...write,result:'Error: write failed'},read],
  [write,{...read,arguments:{path:'foreign.txt'}}],
  [write,{...read,result:content}],
  [write,{...read,source_read:{path:target,complete:false,format:'opaque'}}],
  [write,read,{...write,result:'Error: later mutation failed'}]])assert.throws(()=>verifySelfCodingSession(bytes(session(steps)),Buffer.from(content)));
});
test('historical raw cat cannot masquerade as current qualified observation',()=>{
 const run={tool:'run_command',arguments:{command:'cat self-coding-result.txt'},result:content};
 assert.throws(()=>verifySelfCodingSession(bytes(session([write,run])),Buffer.from(content)));
});
test('runner keeps the original task exact physical proof and optional journal',()=>{
 const source=readFileSync('rust/examples/self-coding/run.sh','utf8');
 assert.ok(source.includes("TASK='"+task+"'"));
 assert.ok(source.includes('cmp -s "$work/self-coding-result.txt" <(printf %s \'self-coding=passed\')'));
 assert.ok(source.includes('if [[ -f "$work/.formal-ai/general-change-plan.lino" ]]; then'));
 assert.ok(source.includes('verify-session.mjs" "$OUT/session.json" "$OUT/self-coding-result.txt"'));
});
