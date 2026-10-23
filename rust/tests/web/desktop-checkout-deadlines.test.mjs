// Check the deployed workflow's network step boundaries without invoking GitHub.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
test('every desktop producer and consumer checkout has a short explicit deadline',()=>{
  const workflow=YAML.parse(readFileSync(new URL('../../../.github/workflows/desktop-release.yml',import.meta.url),'utf8'));
  const checked=[];
  for(const [name,job] of Object.entries(workflow.jobs))for(const step of job.steps??[]) {
    if(!step.uses?.startsWith('actions/checkout@'))continue;
    assert.ok(Number.isFinite(step['timeout-minutes'])&&step['timeout-minutes']>0&&step['timeout-minutes']<=5,name);
    checked.push(name);
  }
  for(const name of ['resolve','native-source','native','build','cli','vscode','finalize'])assert.ok(checked.includes(name),name);
});
