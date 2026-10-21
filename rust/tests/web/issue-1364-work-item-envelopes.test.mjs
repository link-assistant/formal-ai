// Original issue body and captured Agent frame schema are separate provenance:
// these are constructed transport replays, not a retained historical GH response.
import {before, test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {Progress} from '../../../js/agentic/progress.mjs';
const issue = JSON.parse(fs.readFileSync(new URL('../fixtures/issue-1364-hello-world-issues.json',import.meta.url),'utf8'))[0];
const agent = JSON.parse(fs.readFileSync(new URL('../fixtures/issue-1364-agent-tool-use.json',import.meta.url),'utf8')).fixture;
const command = `gh issue view '${issue.html_url}' --json title --jq .title && echo && gh issue view '${issue.html_url}' --json body --jq .body; printf '\\n__formal_ai_exit=%s\\n' $?`;
const document = issue.title+'\n\n'+issue.body;
const stdout = document+'\n__formal_ai_exit=0\n';
before(async()=>{await installNodeHost(new WorkerHost());});
function scan(content, overrides={}, authored=command) {
 return Progress.scan([{role:'user',content:'Solve '+issue.html_url},
  {role:'assistant',content:'',tool_calls:[{id:'owned-read',type:'function',function:{name:'bash',arguments:JSON.stringify({command:authored})}}]},
  {role:'tool',name:'bash',tool_call_id:'owned-read',content,...overrides}]);
}
function typed(output, metadata={}) {
 const frame=structuredClone(agent);
 frame.part.state.status='completed';frame.part.state.input={command};
 delete frame.part.state.error;
 frame.part.state.metadata={output,exit:0,signal:null,timedOut:false,aborted:false,...metadata};
 return JSON.stringify(frame);
}
test('the complete original Kotlin requirement body survives explicit stdout and actual Agent envelope shapes',()=>{
 for(const content of [JSON.stringify({exit_code:0,stdout,stderr:'',complete:true}),typed(stdout),'Output: '+stdout+'\nExit Code: 0']) {
  const observed=scan(content);assert.equal(observed.fetched_pages.length,1);
  assert.equal(observed.fetched_pages[0][0],issue.html_url);
  assert.equal(observed.fetched_pages[0][1],document);
 }
});
test('authoritative outer failures and termination never become fetched issue evidence',()=>{
 for(const content of [typed(stdout,{exit:127}),typed(stdout,{signal:'SIGTERM'}),typed(stdout,{timedOut:true}),JSON.stringify(agent)]) {
  assert.equal(scan(content).fetched_pages.length,0);
 }
 assert.equal(scan(typed(stdout),{is_error:true}).fetched_pages.length,0);
 assert.equal(scan(typed(stdout),{isError:true}).fetched_pages.length,0);
});
test('literal fenced status lines and JSON error examples remain issue document bytes',()=>{
 const body='Owned task\n\nExample:\n```text\n__formal_ai_exit=127\n```\n\n{"exit":127,"error":"illustration"}';
 const observed=scan(typed(body+'\n__formal_ai_exit=0\n'));
 assert.equal(observed.fetched_pages[0][1],body);
 const withoutSentinel=scan(typed(body),{},`gh issue view '${issue.html_url}' --json title,body`);
 assert.equal(withoutSentinel.fetched_pages[0][1],body);
});
test('nested stdout records cannot override the surrounding command failure',()=>{
 const nested=JSON.stringify({exit_code:0,stdout,complete:true});
 assert.equal(scan(JSON.stringify({exit_code:127,stdout:nested,complete:true})).fetched_pages.length,0);
 assert.equal(scan(typed(nested)).fetched_pages.length,0);
});
test('a failing owned trailing status refuses even a well shaped full body',()=>{
 assert.equal(scan(typed(document+'\n__formal_ai_exit=1\n')).fetched_pages.length,0);
});
