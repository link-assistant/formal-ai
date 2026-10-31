import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {wallClockBudget,currentWorkflowMinutes} from '../../../scripts/lib/ci-wall-clock.mjs';
import {npmRegistryIdentity} from '../../../scripts/lib/npm-registry-identity.mjs';
const read=path=>readFileSync(path,'utf8');
test('the captured conservative PR bound uses the existing ceiling without a maintained count', () => {
  const record=read('data/meta/ci-wall-clock.lino');
  assert.doesNotMatch(record,/^  measured_minutes /mu);
  const result=wallClockBudget(record);
  assert.equal(result.ceiling,225);
  assert(result.minutes>0 && result.minutes<=result.ceiling);
  assert.throws(()=>wallClockBudget('  ceiling_minutes 0\n'));
  assert.throws(()=>wallClockBudget(''));
});
test('parallel jobs are not summed, dependencies are, and unknown bounds fail closed', () => {
  const workflow='jobs:\n  a:\n    timeout-minutes: 10\n  b:\n    timeout-minutes: 20\n';
  assert.equal(currentWorkflowMinutes('.github/workflows/fixture.yml',()=>workflow),20);
  assert.equal(currentWorkflowMinutes('.github/workflows/fixture.yml',()=>workflow+'    needs: a\n'),30);
  assert.throws(()=>currentWorkflowMinutes('.github/workflows/fixture.yml',()=> 'jobs:\n  a:\n    runs-on: ubuntu-latest\n'));
  assert.throws(()=>currentWorkflowMinutes('.github/workflows/fixture.yml',()=> 'jobs:\n  a:\n    uses: ./.github/workflows/fixture.yml\n'));
});
test('npm alias and local dependency identities match shared native fixtures', () => {
  const fixture=JSON.parse(read('data/benchmarks/npm-registry-identities.json'));
  for(const row of fixture.cases) assert.deepEqual(npmRegistryIdentity(row.name,row.declared),row.expected);
  const manifest=JSON.parse(read('packages/minimatch-compat/package.json'));
  const identity=npmRegistryIdentity('minimatch-modern',manifest.dependencies['minimatch-modern']);
  assert.equal(identity.name,'minimatch');
  const snapshot=read('data/meta/dependency-registry-snapshot.lino');
  assert(snapshot.includes('registry_entry npm/'+identity.name+'\n  latest "'+identity.requirement+'"'));
});
