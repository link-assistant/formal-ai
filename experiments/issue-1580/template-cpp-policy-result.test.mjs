// Exercise the pinned wrapper with finite failures of its policy-reading child.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

const fixture = new URL('./fixtures/cpp-template/', import.meta.url);
const source = readFileSync(new URL('scripts/check-pipeline-status.sh', fixture), 'utf8');
const identity = JSON.parse(readFileSync(new URL('source.json', fixture), 'utf8'));
assert.equal(createHash('sha256').update(source).digest('hex'), identity.files[0].sha256);
const oldReader = String.raw`    while IFS=$'\t' read -r job value; do
      cancellation_policy["$job"]="$value"
    done < <(
      WORKFLOW_FILE="$workflow_file" JOB_NAMES="$cancelled_jobs" \
        "$SCRIPT_DIR/read-job-cancel-in-progress.sh"
    )`;
const checkedReader = String.raw`    if policy_rows="$(
      WORKFLOW_FILE="$workflow_file" JOB_NAMES="$cancelled_jobs" \
        "$SCRIPT_DIR/read-job-cancel-in-progress.sh"
    )"; then
      while IFS=$'\t' read -r job value; do
        cancellation_policy["$job"]="$value"
      done <<< "$policy_rows"
    fi`;
assert.equal(source.split(oldReader).length, 2, 'exact pinned reader boundary required');
const uncheckedNeeds = 'needs = json.loads(os.environ["NEEDS_JSON"])';
const checkedNeeds = uncheckedNeeds + `
if not isinstance(needs, dict) or not needs:
    raise ValueError("Expected a nonempty workflow needs object")
if any(not isinstance(record, dict) or record.get("result") not in
       {"success", "skipped", "cancelled", "failure"} for record in needs.values()):
    raise ValueError("Expected a completed result for every required job")`;
assert.equal(source.split(uncheckedNeeds).length, 2);
const candidate = process.env.ISSUE_1580_DISABLE_PIPELINE_POLICY_REPAIR === '1'
  ? source : source.replace(oldReader, () => checkedReader).replace(uncheckedNeeds, () => checkedNeeds);

function replay(script, {output = 'build\ttrue\n', code = 0, superseded = true, result = 'cancelled', needs} = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-policy-status-'));
  try {
    writeFileSync(join(directory, 'check-pipeline-status.sh'), script);
    writeFileSync(join(directory, 'read-job-cancel-in-progress.sh'),
      '#!/usr/bin/env bash\nprintf %s ' + "'" + output + "'" + '\nexit ' + code + '\n', {mode: 0o755});
    const workflow = join(directory, 'workflow.yml');
    writeFileSync(workflow, 'jobs:\n  build:\n    concurrency:\n      cancel-in-progress: true\n');
    const observed = spawnSync('bash', ['check-pipeline-status.sh'], {
      cwd: directory, encoding: 'utf8', timeout: 3000,
      env: {...process.env, NEEDS_JSON: needs === undefined ? JSON.stringify({build: {result}}) : needs,
        WORKFLOW_FILE: workflow, RUN_SHA: 'a'.repeat(40),
        BRANCH_HEAD_SHA: (superseded ? 'b' : 'a').repeat(40), BRANCH_NAME: 'fixture',
        GH_TOKEN: '', GITHUB_RUN_ID: ''},
    });
    assert.equal(observed.signal, null, observed.stderr);
    return observed;
  } finally { rmSync(directory, {recursive: true, force: true}); }
}

test('the pinned C++ gate exposes false success after a partial failed policy read', {timeout: 5000}, () => {
  const observed = replay(source, {code: 29});
  assert.equal(observed.status, 0, observed.stdout + observed.stderr);
  assert.match(observed.stdout, /Pipeline passed[.]/u);
});

for (const [label, options, expected] of [
  ['verified superseded cancellation', {}, 0],
  ['partial failed policy read', {code: 29}, 1],
  ['empty failed policy read', {code: 29, output: ''}, 1],
  ['policy forbidding cancellation', {output: 'build\tfalse\n'}, 1],
  ['current-head cancellation', {superseded: false}, 1],
  ['failed required job', {result: 'failure'}, 1],
]) {
  test('the policy-reader candidate preserves ' + label, {timeout: 5000}, () => {
    const observed = replay(candidate, options);
    assert.equal(observed.status, expected, observed.stdout + observed.stderr);
    if (expected) assert.doesNotMatch(observed.stdout, /Pipeline passed[.]/u);
  });
}

test('the pinned C++ gate exposes false success for an empty needs object', {timeout: 5000}, () => {
  const observed = replay(source, {needs: '{}'});
  assert.equal(observed.status, 0, observed.stdout + observed.stderr);
  assert.match(observed.stdout, /Pipeline passed[.]/u);
});

for (const [label, needs, expected] of [
  ['empty needs', '{}', 1],
  ['null needs', 'null', 1],
  ['array needs', '[]', 1],
  ['missing result', '{"build":{}}', 1],
  ['unknown result', '{"build":{"result":"waiting"}}', 1],
  ['null result', '{"build":{"result":null}}', 1],
  ['malformed JSON', '{', 1],
  ['multiple JSON documents', '{} {}', 1],
  ['successful job', '{"build":{"result":"success"}}', 0],
  ['skipped job', '{"build":{"result":"skipped"}}', 0],
]) {
  test('the policy-reader candidate handles ' + label, {timeout: 5000}, () => {
    const observed = replay(candidate, {needs});
    assert.equal(observed.status, expected, observed.stdout + observed.stderr);
    if (expected) assert.doesNotMatch(observed.stdout, /Pipeline passed[.]/u);
  });
}
