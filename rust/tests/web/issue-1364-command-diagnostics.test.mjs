import test from 'node:test';
import assert from 'node:assert/strict';
import {requireSuccessfulReplayCommand} from '../../../scripts/lib/issue-1364-command-result.mjs';
const success={status:0,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0),stderr:Buffer.alloc(0)};
test('stdout-only Hive failure remains visible with exit status and redacted secrets',()=>{
 const secret='private-wrapper-token';
 assert.throws(()=>requireSuccessfulReplayCommand({...success,status:1,
  stdout:Buffer.from('Playwright MCP preflight failed '+secret)},{program:'/owned/node',environment:{GH_TOKEN:secret}}),error=>{
  assert.equal(error.code,'ISSUE1364_COMMAND_FAILED');assert.match(error.message,/Playwright MCP preflight failed/u);
  assert.match(error.message,/"exitCode":1/u);assert(!error.message.includes(secret));return true;
 });
});
test('uncertain command cleanup, timeout and signal are failures even with zero status',()=>{
 requireSuccessfulReplayCommand(success,{program:'node'});
 for(const fields of [{cleanup_complete:false},{timed_out:true},{signal:'SIGTERM'},{error:{code:'ETIMEDOUT'}}])
  assert.throws(()=>requireSuccessfulReplayCommand({...success,...fields},{program:'node'}),{code:'ISSUE1364_COMMAND_FAILED'});
});
test('diagnostic streams are bounded after whole-secret redaction',()=>{
 const secret='boundary-wrapper-token',text='prefix '+secret+'\n'+'z'.repeat(9000);
 assert.throws(()=>requireSuccessfulReplayCommand({...success,status:1,stdout:Buffer.from(text),stderr:Buffer.from(text)},
  {program:'node',environment:{GH_TOKEN:secret}}),error=>{assert(error.message.length<9000);assert(!error.message.includes(secret));return true;});
});
