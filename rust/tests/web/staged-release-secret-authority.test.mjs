import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import { buildStagedReleaseProjection } from '../../../scripts/generate-staged-release.mjs';
import { compileContract } from '../../../scripts/staged-caller-authority.mjs';
const root = new URL('../../../', import.meta.url);
const evidence = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const YAML = createRequire(import.meta.url)('yaml');
const names = ['CARGO_REGISTRY_TOKEN', 'CARGO_TOKEN', 'DOCKERHUB_USERNAME', 'DOCKERHUB_TOKEN'];
function fixture() {
  const canonical = fs.readFileSync(new URL(evidence + 'original-release-workflow.yml', root), 'utf8');
  const packet = JSON.parse(fs.readFileSync(new URL(evidence + 'stage-source-coverage.json', root), 'utf8'));
  const projection = buildStagedReleaseProjection(canonical, packet);
  return {
    canonical,
    packet,
    projection,
    caller: YAML.parse(projection.outputs.get(evidence + 'candidate-release-caller.yml')),
    callee: YAML.parse(projection.outputs.get('.github/workflows/release-staged.yml'))
  };
}
function compile(value) {
  return compileContract({
    canonical: value.canonical,
    packet: value.packet,
    YAML,
    callerDraft: YAML.stringify(value.caller),
    staged: YAML.stringify(value.callee)
  });
}
test('four consumed optional secrets preserve all source operations and automatic token', () => {
  const value = fixture();
  const contract = compile(value);
  assert.equal(contract.routes.length, 52);
  assert.equal(contract.bindings, 104);
  const consumed = [...new Set([...JSON.stringify(value.callee).matchAll(/secrets\.([A-Z_]+)/g)].map(match => match[1]))].filter(name => name !== 'GITHUB_TOKEN').sort();
  assert.deepEqual(consumed, [...names].sort());
  assert.equal(value.callee.env.CARGO_REGISTRY_TOKEN, '$' + '{{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}');
  assert.equal(value.callee.on.workflow_call.secrets.GITHUB_TOKEN, undefined);
  for (const mode of ['auto', 'manual']) assert.equal(value.callee.jobs[mode + '_prepare-source'].permissions.actions, 'read');
});
for (const caller of ['auto-release', 'manual-release']) {
  for (const name of names) test(caller + ' missing consumed ' + name + ' refuses', () => {
    const value = fixture();
    delete value.caller.jobs[caller].secrets[name];
    assert.throws(() => compile(value));
  });
  test(caller + ' unrestricted inheritance refuses', () => {
    const value = fixture();
    value.caller.jobs[caller].secrets = 'inherit';
    assert.throws(() => compile(value));
  });
  test(caller + ' additional arbitrary authority refuses', () => {
    const value = fixture();
    value.caller.jobs[caller].secrets.UNRELATED = '$' + '{{ secrets.UNRELATED }}';
    assert.throws(() => compile(value));
  });
  test(caller + ' wrong credential binding refuses', () => {
    const value = fixture();
    value.caller.jobs[caller].secrets.CARGO_TOKEN = '$' + '{{ secrets.DOCKERHUB_TOKEN }}';
    assert.throws(() => compile(value));
  });
}
for (const name of names) {
  test('callee missing ' + name + ' declaration refuses', () => {
    const value = fixture();
    delete value.callee.on.workflow_call.secrets[name];
    assert.throws(() => compile(value));
  });
  test('callee changed ' + name + ' optional credential gate refuses', () => {
    const value = fixture();
    value.callee.on.workflow_call.secrets[name].required = true;
    assert.throws(() => compile(value));
  });
}
test('callee extra declared authority refuses', () => {
  const value = fixture();
  value.callee.on.workflow_call.secrets.UNRELATED = {
    required: false
  };
  assert.throws(() => compile(value));
});
