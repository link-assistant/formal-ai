import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {Script} from 'node:vm';
import YAML from 'yaml';
const workflow = YAML.parse(readFileSync('.github/workflows/desktop-release.yml', 'utf8'));
const capture = JSON.parse(readFileSync('rust/tests/fixtures/native-release-source-transport.json', 'utf8'));

test('captured failing native targets consume the verified same-run history instead of cold source checkouts', () => {
  const targets = workflow.jobs.native.strategy.matrix.include.map(row => row.target);
  for (const row of capture.observations) assert(targets.includes(row.target), row.target);
  for (const id of ['native','build','cli','vscode']) {
    const {steps} = workflow.jobs[id];
    assert.equal(steps.filter(step => step.uses?.includes('checkout@') && !step.with?.path).length, 0, id);
    assert.equal(steps.filter(step => step.with?.name === 'native-release-source-history').length, 1, id);
    assert(steps.some(step => step.run?.includes('native-release-source.mjs" import-offline native-source')), id);
    const sourceImport=steps.find(step=>step.run?.includes('native-release-source.mjs" import-offline native-source'));
    assert.equal(sourceImport['timeout-minutes'],7,id);
    const protocol = steps.find(step => step.uses?.includes('checkout@'));
    assert.equal(protocol.with.ref, '${{ github.workflow_sha }}');
    assert.equal(protocol.with['persist-credentials'], false);
  }
});

test('packaging console capture requires an attempted package operation and retains actual failures', () => {
  const steps = workflow.jobs.build.steps;
  const stage = steps.find(step => step.name === 'Stage redacted packaging console');
  const ids = steps.filter(step => step.name?.startsWith('Package desktop app')).map(step => step.id);
  assert.equal(ids.length, 3);
  const expression = stage.if.replace(/^\$\{\{\s*/u,'').replace(/\s*\}\}$/u,'');
  const selected = outcomes => new Script(expression).runInNewContext({always:()=>true,
    steps:Object.fromEntries(ids.map((id,index)=>[id,{outcome:outcomes[index]??'skipped'}]))}, {timeout:100});
  assert.equal(selected([]), false);
  assert.equal(selected(['','','']), false);
  for (let index=0;index<ids.length;index++) for (const outcome of ['success','failure','cancelled']) {
    const outcomes=ids.map(()=> 'skipped');outcomes[index]=outcome;
    assert.equal(selected(outcomes), true, ids[index]+'/'+outcome);
  }
});
