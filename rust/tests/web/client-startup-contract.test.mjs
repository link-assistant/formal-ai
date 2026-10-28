import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {promptAfterFor,promptInteractionsFor,startupInteractionsFor}
  from '../../../experiments/agent_cli_e2e/issue_819_tui/client-contract.mjs';
const capture=JSON.parse(readFileSync(new URL('../../../experiments/agent_cli_e2e/issue_819_tui/fixtures/codex-startup.json',import.meta.url),'utf8'));

test('actual captured footer during loading cannot authorize prompt submission',()=>{
  const ready=promptAfterFor('codex');
  assert.equal(ready.test(capture.loadingFrame),false);
  assert.equal(ready.test(capture.readyFrame),true);
  assert.equal(ready.test('formal-ai default'),false);
  assert.equal(ready.test('model: formal-ai-wrong'),false);
});
test('actual captured draft receives exactly one Enter after the paste-suppression interval',()=>{
  const plan=promptInteractionsFor('codex',capture.prompt);
  assert.equal(plan.length,2);
  assert.equal(plan[0].text,capture.prompt);
  assert.equal(plan[0].key,undefined);
  assert.equal(plan.filter(row=>row.key==='ENTER').length,1);
  assert.equal(plan[1].after.test(capture.draftFrame),true);
  assert.equal(plan[1].after.test(capture.loadingFrame),false);
  assert(plan[1].idleMilliseconds>capture.pasteEnterSuppressionMilliseconds);
});
test('held-out punctuation and terminal formatting remain owned literal prompt text',()=>{
  const prompt='Write x+y [ok]?';
  const plan=promptInteractionsFor('codex',prompt);
  assert.equal(plan[0].text,prompt);
  assert.equal(plan[1].after.test('Write\u001b[12;8Hx+y\u001b[12;14H[ok]?'),true);
  assert.equal(plan[1].after.test('Write xxxy ok'),false);
  assert.deepEqual(startupInteractionsFor('claude'),[{after:'Enter y/n:',text:'y',key:'ENTER'}]);
  assert.equal(promptAfterFor('claude'),'$');
});
