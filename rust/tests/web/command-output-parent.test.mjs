import test, { before } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, chmodSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { composeGeneralChangePlan, GeneralPlanMode } from '../../../js/agentic/general_planner.mjs';
before(async () => installNodeHost(new WorkerHost()));
function run(target, command='printf learned-output', prepare=()=>{}) {
  const root=mkdtempSync(join(tmpdir(),'formal-ai-capture-'));
  try {
    const request=`Run '${command}' and write its exact stdout to ${target}`;
    const plan=composeGeneralChangePlan(request);
    assert.equal(plan.mode,GeneralPlanMode.CommandOutput);
    assert.equal(plan.target,target);
    prepare(root);
    const result=spawnSync('sh',['-c',plan.steps[1].command],{cwd:root,encoding:'utf8',timeout:2000});
    return {root,plan,result,bytes:existsSync(join(root,target))?readFileSync(join(root,target),'utf8'):null};
  } finally { rmSync(root,{recursive:true,force:true}); }
}
test('missing parent is created and the entire authorized stdout is captured',()=>{
  const actual=run('reports/learned.txt','printf one; printf two');
  assert.equal(actual.result.status,0);assert.equal(actual.bytes,'onetwo');
  assert.match(actual.plan.steps[1].command,/mkdir -p --/u);
  assert.equal(actual.plan.verification_command,"cat 'reports/learned.txt'");
});
test('flat target preserves exact command bytes without creating another parent',()=>{
  const actual=run('result.txt');assert.equal(actual.result.status,0);assert.equal(actual.bytes,'learned-output');
});
test('a parent file prevents execution and no target success is fabricated',()=>{
  const actual=run('reports/learned.txt','printf executed',root=>writeFileSync(join(root,'reports'),'file'));
  assert.notEqual(actual.result.status,0);assert.equal(actual.bytes,null);assert.equal(actual.result.stdout,'');
});
test('denied directory creation is an actual process failure',()=>{
  const actual=run('reports/nested/learned.txt','printf executed',root=>{mkdirSync(join(root,'reports'));chmodSync(join(root,'reports'),0o500);});
  assert.notEqual(actual.result.status,0);assert.equal(actual.bytes,null);assert.equal(actual.result.stdout,'');
});
test('opaque Unicode destination is quoted without altering its bytes',()=>{
  const actual=run('报告/结果.txt');
  assert.equal(actual.result.status,0);assert.equal(actual.bytes,'learned-output');assert.equal(actual.result.stdout,'');
  assert.equal(actual.plan.target,'报告/结果.txt');
});
test('injection-shaped destination is refused before a command is planned',()=>{
  assert.equal(composeGeneralChangePlan("Run 'printf learned-output' and write its exact stdout to reports;echo${IFS}injected/result.txt"),null);
});
test('the original failing request retains its whole goal and real destination',()=>{
  const original="Execute the auto-learning task. Run 'printf learned-output' and write its exact stdout to reports/learned.txt";
  const plan=composeGeneralChangePlan(original);assert.equal(plan.goal,original);assert.equal(plan.target,'reports/learned.txt');assert.equal(plan.content,'');
});
