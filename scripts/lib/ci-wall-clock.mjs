// Conservative timeout bounds, derived from current workflow definitions.
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';
const root=fileURLToPath(new URL('../../',import.meta.url));
export function currentWorkflowMinutes(selected, read = file => readFileSync(join(root, file), 'utf8')) {
  const workflows = new Map(), active = new Set();
  const nonPullRequest = condition => typeof condition === 'string'
    && !condition.includes('pull_request') && !condition.includes('.outputs.')
    && ['push', 'workflow_dispatch', 'release', 'schedule'].some(event =>
      [`github.event_name == '${event}'`, `github.event_name == "${event}"`]
        .some(marker => condition.includes(marker)));
  function measure(file, depth) {
    assert.ok(depth <= 6, 'reusable workflow depth is finite');
    assert.match(file, /^\.github\/workflows\/[A-Za-z0-9_.-]+\.ya?ml$/u);
    assert.ok(!active.has(file), 'reusable workflow DAG is acyclic');
    if (workflows.has(file)) return workflows.get(file);
    active.add(file);
    const source = read(file);
    assert.ok(typeof source === 'string' && Buffer.byteLength(source) <= 1048576);
    const document = YAML.parseDocument(source);
    assert.equal(document.errors.length, 0, 'selected workflow parses without ambiguity');
    const mapping = document.toJSON()?.jobs;
    assert.ok(mapping && typeof mapping === 'object' && !Array.isArray(mapping));
    const currentJobs = new Map(Object.entries(mapping));
    assert.ok(currentJobs.size > 0 && currentJobs.size <= 512);
    const visiting = new Set(), distances = new Map();
    const dependencies = new Map();
    for (const [id, job] of currentJobs) {
      assert.ok(job && typeof job === 'object' && !Array.isArray(job));
      const needs = job.needs === undefined ? [] : typeof job.needs === 'string' ? [job.needs] : job.needs;
      assert.ok(Array.isArray(needs) && needs.every(value => typeof value === 'string'));
      for (const dependency of needs) assert.ok(currentJobs.has(dependency), 'recorded DAG dependency exists: ' + dependency);
      dependencies.set(id, needs);
    }
    const dependencyActive = new Set(), dependencyDone = new Set();
    function validateDependencies(id) {
      assert.ok(!dependencyActive.has(id), 'recorded DAG is acyclic');
      if (dependencyDone.has(id)) return;
      dependencyActive.add(id);
      for (const dependency of dependencies.get(id)) validateDependencies(dependency);
      dependencyActive.delete(id); dependencyDone.add(id);
    }
    for (const id of currentJobs.keys()) validateDependencies(id);
    function currentLongest(id) {
      assert.ok(currentJobs.has(id), 'recorded DAG dependency exists: ' + id);
      assert.ok(!visiting.has(id), 'recorded DAG is acyclic');
      if (distances.has(id)) return distances.get(id);
      visiting.add(id);
      const job = currentJobs.get(id);
      let total = 0;
      if (!nonPullRequest(job.if)) {
        let weight;
        if (job.uses !== undefined) {
          assert.ok(typeof job.uses === 'string' && job.uses.startsWith('./'));
          const reusable = job.uses.slice(2);
          assert.match(reusable, /^\.github\/workflows\/[A-Za-z0-9_.-]+\.ya?ml$/u);
          weight = measure(reusable, depth + 1);
        } else {
          weight = job['timeout-minutes'];
          if (typeof weight === 'string') {
            const key = weight.match(/^\$\{\{\s*matrix\.([A-Za-z0-9_-]+)\s*\}\}$/u)?.[1];
            assert.ok(key, 'matrix timeout is a declared scalar key');
            const rows = job.strategy?.matrix?.include;
            assert.ok(Array.isArray(rows), 'matrix timeout has concrete include rows');
            const values = rows.map(row => row?.[key]).filter(value => Number.isSafeInteger(value) && value >= 0 && value <= 4294967295);
            assert.ok(values.length > 0, 'matrix timeout has an integer allowance');
            weight = Math.max(...values);
          }
          assert.ok(Number.isSafeInteger(weight) && weight > 0 && weight <= 4294967295, 'job timeout is finite and positive');
        }
        total = Math.max(0, ...dependencies.get(id).map(currentLongest)) + weight;
        assert.ok(Number.isSafeInteger(total) && total <= 4294967295);
      }
      visiting.delete(id); distances.set(id, total); return total;
    }
    const minutes = Math.max(...[...currentJobs.keys()].map(currentLongest));
    active.delete(file); workflows.set(file, minutes); return minutes;
  }
  return measure(selected, 0);
}

export function wallClockBudget(record = readFileSync(join(root,'data/meta/ci-wall-clock.lino'),'utf8')) {
  const ceiling=Number(record.match(/^  ceiling_minutes ([0-9]+)$/mu)?.[1]);
  assert(Number.isSafeInteger(ceiling) && ceiling>=0, 'finite ceiling required');
  const rows=[];
  for(const name of readdirSync(join(root,'.github/workflows')).filter(name=>name.endsWith('.yml')).sort()) {
    const path='.github/workflows/'+name, document=YAML.parse(readFileSync(join(root,path),'utf8'));
    const events=document.on;
    if (!(typeof events==='string'?events==='pull_request':Array.isArray(events)?events.includes('pull_request'):events && Object.hasOwn(events,'pull_request'))) continue;
    rows.push({workflow:path,minutes:currentWorkflowMinutes(path)});
  }
  assert(rows.length>0,'PR workflows required');
  rows.sort((a,b)=>a.minutes-b.minutes || a.workflow.localeCompare(b.workflow));
  const maximum=rows.at(-1);
  assert(maximum.minutes<=ceiling,'computed timeout bound exceeds unchanged ceiling');
  return {...maximum,ceiling};
}
