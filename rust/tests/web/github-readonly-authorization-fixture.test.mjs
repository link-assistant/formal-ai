import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createRequire} from 'node:module';
import {requireObservationProfile} from '../../../scripts/github-readonly-authorization-fixture.mjs';
const require=createRequire(import.meta.url);const YAML=require('yaml');
const source=fs.readFileSync(new URL('../../../.github/workflows/github-readonly-authorization-fixture.yml',import.meta.url),'utf8');
const workflow=YAML.parse(source);
test('exact distinct read-only profiles and private groups',()=>{
 const first=requireObservationProfile(workflow,'actions-omitted','12','3');
 const second=requireObservationProfile(workflow,'actions-readable','12','3');
 assert.notEqual(first.group,second.group);
 assert.notEqual(first.group,requireObservationProfile(workflow,'actions-omitted','13','3').group);
});
test('missing permissions, unknown scope, colliding groups and malformed identities refuse',()=>{
 for(const mutate of [w=>delete w.jobs['actions-omitted'].permissions,w=>w.jobs['actions-omitted'].permissions.actions='write',
  w=>w.jobs['actions-omitted'].concurrency.group=w.jobs['actions-readable'].concurrency.group,
  w=>w.jobs['actions-omitted'].concurrency['cancel-in-progress']=true,w=>w.jobs['actions-omitted'].if='true',
  w=>w.jobs['actions-omitted'].steps.find(s=>s.run==='node scripts/github-readonly-authorization-fixture.mjs').env.GH_TOKEN='untrusted']) {
  const clone=structuredClone(workflow);mutate(clone);assert.throws(()=>requireObservationProfile(clone,'actions-omitted','12','3'));
 }
 assert.throws(()=>requireObservationProfile(workflow,'unknown','12','3'));
 assert.throws(()=>requireObservationProfile(workflow,'actions-omitted','0','3'));
});
