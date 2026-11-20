#!/usr/bin/env node
// Check the entire workflow payload against the last published pre-migration tree.
// The supplied implementation and comparison earn no independent authorship credit.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import YAML from 'yaml';

const baseline = 'aca32b1aa27aee55f1d9443f54d6a1c08f77ccb0';
const hash = value => createHash('sha256').update(value).digest('hex');
const original = file => execFileSync('git', ['show', baseline + ':' + file], {encoding:'utf8', maxBuffer:2*1024*1024});
const files = execFileSync('git', ['ls-tree', '--name-only', baseline, '.github/workflows/'], {encoding:'utf8'}).trim().split('\n');
const rows = [], checkouts = [], sparse = [], sources = [];
let workflows = 0;
const isDownload = step => step.uses?.startsWith('actions/download-artifact@');
const addedName = 'Check out workflow-owned artifact extraction';
for (const file of files.filter(file => /\.ya?ml$/u.test(file))) {
  const oldBytes = original(file), newBytes = readFileSync(file, 'utf8');
  const old = YAML.parse(oldBytes), current = YAML.parse(newBytes), normalized = structuredClone(current);
  const count = Object.values(old.jobs ?? {}).flatMap(job => job.steps ?? []).filter(isDownload).length;
  if (!count) {assert.deepEqual(current, old, file + ' unrelated workflow changed'); continue;}
  workflows++;
  for (const [jobId, job] of Object.entries(old.jobs)) {
    if (!job.steps) continue;
    const live = normalized.jobs[jobId].steps, added = live.filter(step => step.name === addedName);
    assert(!job.steps.some(step => step.name === addedName));
    for (const step of added) {
      const root = ['agentic-cli-matrix.yml/learn','external-benchmarks.yml/shard-total'].includes(file.split('/').at(-1) + '/' + jobId);
      assert.deepEqual(step, {name:addedName, uses:'actions/checkout@v7', with:{
        ref:'${{ github.workflow_sha }}', ...(root ? {} : {path:'.formal-ai-artifact-protocol'}),
        'persist-credentials':false, 'fetch-depth':1, filter:'blob:none',
        'sparse-checkout':'/.github/actions/download-artifact/\n/scripts/extract-artifacts.py\n',
        'sparse-checkout-cone-mode':false,
      }}, file + '/' + jobId + ' extra checkout authority');
      checkouts.push({file, jobId, step});
    }
    normalized.jobs[jobId].steps = live.filter(step => step.name !== addedName);
    const after = normalized.jobs[jobId].steps;
    assert.equal(after.length, job.steps.length, file + '/' + jobId + ' retains every original step');
    for (const [index, before] of job.steps.entries()) {
      const step = after[index], label = file + '/' + jobId + '/' + index;
      if (isDownload(before)) {
        assert.match(step.uses, /^\.\/(?:[.\w-]+\/)*\.github\/actions\/download-artifact$/u, label);
        const expected = structuredClone(before);
        expected.uses = step.uses;
        if (expected.env?.NODE_OPTIONS) {
          assert.equal(expected.env.NODE_OPTIONS, '--disable-warning=DEP0005', label + ' narrow old suppression');
          delete expected.env.NODE_OPTIONS;
          if (!Object.keys(expected.env).length) delete expected.env;
        }
        if (expected.with?.['digest-mismatch']) {
          assert.equal(expected.with['digest-mismatch'], 'error', label);
          delete expected.with['digest-mismatch'];
        }
        assert.deepEqual(step, expected, label + ' keeps every selector, predicate and other field');
        rows.push({file, jobId, index, before, after:structuredClone(step)});
        after[index] = structuredClone(before);
      } else if (before.uses?.startsWith('actions/checkout@') && before.with?.path === '.formal-ai-protocol-source') {
        assert.equal(step.with['sparse-checkout'], before.with['sparse-checkout'] + '\n.github/actions/download-artifact', label);
        sparse.push({file,jobId,index,before:before.with['sparse-checkout'],after:step.with['sparse-checkout']});
        step.with['sparse-checkout'] = before.with['sparse-checkout'];
      }
    }
  }
  assert.deepEqual(normalized, old, file + ' preserves complete workflow semantics outside reviewed transport changes');
  sources.push({file,beforeSha256:hash(oldBytes),afterSha256:hash(newBytes)});
}
assert.equal(workflows,19);assert.equal(rows.length,97);assert.equal(checkouts.length,10);assert.equal(sparse.length,4);
console.log(JSON.stringify({schema:'artifact-consumer-migration-review/v1',baseline,workflows,transfers:rows.length,
  addedCheckouts:checkouts.length,expandedSparseCheckouts:sparse.length,sources,rows,checkouts,sparse,
  preserved:'All job and workflow fields, non-adapter steps, artifact selectors and predicates are compared.',
  hostedExecution:false,independentAuthorshipCredit:0},null,2));
