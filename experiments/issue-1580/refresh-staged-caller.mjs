import {readCanonicalSource} from '../../scripts/canonical-release-source.mjs';
// External bootstrap, executed through Formal AI; zero autonomous-authorship credit.
// Refresh non-release caller changes without rewriting the 52 release operations.
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {buildStagedReleaseProjection} from '../../scripts/generate-staged-release.mjs';
import {workflowJobs} from '../../scripts/lib/ci-speed-workflows.mjs';

const prefix = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const read = file => readFileSync(file, 'utf8');
const original = readCanonicalSource(process.cwd()).text;
const packet = JSON.parse(read(prefix + 'stage-source-coverage.json'));
const before = buildStagedReleaseProjection(original, packet);
const callerPath = '.github/workflows/release.yml';
const oldCaller = before.outputs.get(prefix + 'candidate-release-caller.yml');
assert.equal(execFileSync('git', ['show', 'HEAD:' + callerPath], {encoding: 'utf8'}), oldCaller,
  'refresh starts from the checked committed projection');
const current = read(callerPath);
const jobs = value => new Map(workflowJobs(value).map(job => [job.id, job]));
const previousJobs = jobs(oldCaller), currentJobs = jobs(current), sourceJobs = jobs(original);
assert.deepEqual([...currentJobs.keys()], [...previousJobs.keys()], 'job insertion/removal requires a separate source migration');
const protectedJobs = new Set(['detect-changes', ...packet.callers.map(caller => caller.caller)]);
let canonical = original;
const changed = [];
for (const [id, job] of currentJobs) {
  if (job.body === previousJobs.get(id).body) continue;
  assert(!protectedJobs.has(id), 'release operations and detector transformations need an explicit migration: ' + id);
  const old = sourceJobs.get(id).body;
  assert.equal(canonical.split(old).length, 2, 'unique source job: ' + id);
  canonical = canonical.replace(old, job.body);
  changed.push(id);
}
const header = value => value.slice(0, value.indexOf('\njobs:\n'));
assert.equal(header(original), header(oldCaller), 'generator preserves the source header');
canonical = header(current) + canonical.slice(canonical.indexOf('\njobs:\n'));
packet.workflowSha256 = createHash('sha256').update(canonical).digest('hex');
const after = buildStagedReleaseProjection(canonical, packet);
assert.equal(after.outputs.get(prefix + 'candidate-release-caller.yml'), current, 'refreshed source must project exactly to the requested caller');
assert.equal(after.outputs.get('.github/workflows/release-staged.yml'), before.outputs.get('.github/workflows/release-staged.yml'),
  'non-release changes cannot alter any staged operation');
for (const [file, bytes] of after.outputs) writeFileSync(file, bytes);
console.log(JSON.stringify({changed, headerChanged: header(current) !== header(oldCaller), operations: after.operations, bindings: after.binding.length}));
