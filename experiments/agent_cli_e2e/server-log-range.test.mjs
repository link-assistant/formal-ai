import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {REPO_ROOT} from '../../js/server/lino.mjs';
import {encodeTyped} from '../../js/server/structured-log.mjs';
import {projectServerLog} from './server-log-evidence.mjs';
const directory=path.dirname(fileURLToPath(import.meta.url));
const root=REPO_ROOT;
const typed=message=>encodeTyped({schema:'formal-ai-diagnostic-v1',message});
const intent=String.raw`run_shell_command", arguments: "{\"command\":\"date\"}`;
function range(raw,first,last){
 const temp=fs.mkdtempSync(path.join(os.tmpdir(),'server-evidence-range-'));
 try{
  const input=path.join(temp,'input');
  fs.writeFileSync(input,raw);
  const result=spawnSync('bash',['-c',[
   'set -euo pipefail',
   'source "$1/server-log-evidence.sh"',
   'ROOT="$2"; RAW_SERVER_LOG="$3"; EVIDENCE_DIR="$4"',
   'project_server_log_range "$5" "$6" leg > "$4/posts"',
  ].join('\n'),'range',directory,root,input,temp,String(first),String(last)],
  {encoding:'utf8',timeout:10000});
  return {
   status:result.status,stderr:result.stderr,
   semantic:fs.existsSync(temp+'/leg.semantic.log')?fs.readFileSync(temp+'/leg.semantic.log','utf8'):'',
   snapshot:fs.readFileSync(temp+'/leg.raw.log','utf8'),
   original:fs.readFileSync(input,'utf8'),
   posts:fs.existsSync(temp+'/posts')?fs.readFileSync(temp+'/posts','utf8').trim():'',
  };
 }finally{fs.rmSync(temp,{recursive:true,force:true});}
}
test('typed date trace is decoded after raw turn slicing, never borrowed across turns',()=>{
 const first=typed('[trace] planned write\nmain.py');
 const second=typed('[trace] agentic_outcome: planned ToolCalls '+intent);
 const raw=first+'\n'+second+'\n';
 const task=range(raw,1,1);
 const question=range(raw,2,'$');
 assert.equal(task.status,0,task.stderr);
 assert.equal(question.status,0,question.stderr);
 assert.ok(!task.semantic.includes(intent));
 assert.ok(question.semantic.includes(intent));
 assert.equal(task.snapshot,first+'\n');
 assert.equal(question.snapshot,second+'\n');
 assert.equal(question.original,raw);
 assert.equal(question.posts,'0');
});
test('legacy plaintext alongside canonical records retains exact original oracle text',()=>{
 const legacy='[trace] agentic_outcome: planned ToolCalls '+intent;
 const raw=typed('server started')+'\n'+legacy+'\n';
 const result=range(raw,1,'$');
 assert.equal(result.status,0,result.stderr);
 assert.ok(result.semantic.includes(legacy));
 assert.equal(result.original,raw);
 assert.equal(result.posts,'0');
});
test('pure legacy cannot silently replace required current typed diagnostics',()=>{
 assert.throws(()=>projectServerLog('[trace] '+intent),/typed server records missing/);
});
test('malformed typed records fail projection even beside valid legacy evidence',()=>{
 const result=range(typed('started')+'\n[trace] '+intent+'\n(formal-ai-log-v1 broken\n',1,'$');
 assert.notEqual(result.status,0);
 assert.match(result.stderr,/invalid typed log/);
});
test('a prior-turn date trace cannot satisfy the later question oracle',()=>{
 const raw=typed(intent)+'\n'+typed('no date tool')+'\n';
 const question=range(raw,2,'$');
 assert.equal(question.status,0,question.stderr);
 assert.ok(!question.semantic.includes(intent));
});
test('raw body substrings cannot fabricate POST rounds during range projection',()=>{
 const header='[trace] POST /v1/chat/completions (40 byte body)';
 const result=range(typed(header+'\n'+header)+'\n'+header+'\n',1,'$');
 assert.equal(result.status,0,result.stderr);
 assert.equal(result.posts,'1');
});
test('issue907 retains original task, date question, exact intent and artifact checks',()=>{
 const script=fs.readFileSync(path.join(directory,'run_issue_907.sh'),'utf8');
 for(const literal of [
  'Write a hello world program in Python.', 'what is the date?',
  'grep -Fq "$EXPECT_TEXT" "$PROJECT_DIR/$EXPECT_FILE"',
  'LEG1_END="$(wc -l < "$SERVER_LOG")"',
  'project_server_log_range 1 "$LEG1_END" leg1',
  'project_server_log_range "$((LEG1_END + 1))"',
  '"$EVIDENCE_DIR/leg1.semantic.log"','"$EVIDENCE_DIR/leg2.semantic.log"',
  'cp "$SERVER_LOG" "$ARTIFACT_DIR/formal-ai.log"',
 ])assert.ok(script.includes(literal),literal);
 assert.equal(script.split(intent).length-1,2);
 assert.ok(script.includes('grep -Fq \'"command":"date"\' "$TASK_LOG"'));
});
