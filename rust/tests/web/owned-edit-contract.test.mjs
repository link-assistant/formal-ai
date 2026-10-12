import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {join,dirname} from 'node:path';
import {tmpdir} from 'node:os';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {composeEditRequest} from '../../../js/agentic/write_request.mjs';
import {goalLedger} from '../../../js/agentic/planner/owned_goals.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
before(async()=>installNodeHost(new WorkerHost()));
const old = [
  "      case 'write': {",
  "        const target = within(dir, path);",
].join('\n');
const replacement = [
  "      case 'write': {",
  "        if (args.append_mode !== undefined) return toolFailure('append requires the explicit receipt adapter');",
  "        const target = within(dir, path);",
].join('\n');

test('complete Edit owns quoted code containing partial write candidates', async context => {
  const root=fs.mkdtempSync(join(tmpdir(),'formal-ai-owned-edit-'));
  context.after(()=>fs.rmSync(root,{recursive:true,force:true}));
  for(const [index,text] of [replacement,replacement.replace('requires','needs'),replacement.replace('requires','refuses')].entries()){
    const target=index===0?'experiments/js_dogfood/drive.mjs':`src/renamed-${index}.mjs`;
    fs.mkdirSync(dirname(join(root,target)),{recursive:true});
    fs.writeFileSync(join(root,target),old);
    const prompt=`In ${target} replace «${old}» with «${text}»`;
    assert.deepEqual(composeEditRequest(prompt),[target,old,text]);
    assert.equal(goalLedger(prompt),null);
    const result=await drive(planChatStepResolved,root,prompt,{tools:['read','edit'],steps:4});
    assert.equal(fs.readFileSync(join(root,target),'utf8'),text,JSON.stringify(result));
    assert.ok(result.transcript.some(call=>call.tool==='edit'));
  }
});

test('complete literal payload retains edit-looking prose without extra ownership',()=>{
  const payload='In source.mjs replace x with y; case write requires receipt';
  const prompt=`Write «${payload}» to note.txt`;
  assert.equal(goalLedger(prompt),null);
});

test('partial literal collision does not consume an independent unowned tail',()=>{
  const prompt=`In source.mjs replace «${old}» with «${replacement}» and deploy production`;
  const goals=goalLedger(prompt);
  assert.ok(goals?.some(goal=>goal.kind==='unsupported'));
});

test('source-edit metadata follows outer proof and quoted filename causes no write', async context => {
  const root = fs.mkdtempSync(join(tmpdir(), 'formal-ai-edit-metadata-'));
  context.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const target = 'source.mjs';
  const body = replacement + "\n// Write 'leak' to b.txt";
  const clause = `In ${target} replace «${old}» with «${body}»`;
  const prompt = clause + '\nWrite «marker» to proof.txt.';
  const goals = goalLedger(prompt);
  assert.equal(goals[0].kind, 'source_edit');
  assert.equal(goals[0].target, target);
  assert.equal(goals[0].expected, null);
  assert.deepEqual(goals[0].node.expectation, {
    kind: 'underivable', reason: 'source-edit-preimage-required',
  });
  assert.equal(goals[1].target, 'proof.txt');
  assert.equal(goals[1].expected, 'marker');
  const prefix = '// retained prefix\n';
  const suffix = '\n// retained suffix';
  fs.writeFileSync(join(root, target), prefix + old + suffix);
  fs.writeFileSync(join(root, 'b.txt'), 'protected older bytes');
  const result = await drive(planChatStepResolved, root, clause, {
    tools: ['read', 'edit'], steps: 4,
  });
  assert.equal(fs.readFileSync(join(root, target), 'utf8'), prefix + body + suffix, JSON.stringify(result));
  assert.equal(fs.existsSync(join(root, 'proof.txt')), false);
  assert.equal(fs.readFileSync(join(root, 'b.txt'), 'utf8'), 'protected older bytes');
  for (const call of result.transcript) {
    const args = JSON.parse(call.arguments);
    assert.notEqual(args.path ?? args.filePath ?? args.file_path, 'b.txt');
  }
});
