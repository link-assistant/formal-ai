// Issue #1169 R1169-5: the daily job that moves every unblocked dependency to
// its latest release. A gate can only say "this drifted"; this workflow is
// what moves the manifests, so its shape is pinned here: the schedule, the
// tools it needs (rust-script is not on a stock runner), the live refresh,
// the offline --apply that tolerates only exit 1, the per-lockfile
// re-resolution (bun.lock by bun, package-lock.json by npm), all three gate
// stages, and the one pull request at the end.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const repository = fileURLToPath(new URL('../../../', import.meta.url));
const workflow = readFileSync(`${repository}.github/workflows/dependencies-latest.yml`, 'utf8');

function position(needle) {
  const at = workflow.indexOf(needle);
  assert.ok(at >= 0, `the workflow carries ${JSON.stringify(needle)}`);
  return at;
}

test('runs daily off the fleet stampede marks and on demand', () => {
  assert.match(workflow, /- cron: "23 5 \* \* \*"/);
  position('workflow_dispatch:');
});

test('installs rust-script before the first rust-script step', () => {
  const install = position('run: bash scripts/install-rust-script.sh');
  const firstUse = position('rust-script scripts/check-dependencies-latest.rs --refresh-snapshot');
  assert.ok(install < firstUse, 'rust-script is installed before it is used');
});

test('refreshes live, then applies offline and tolerates only the findings exit', () => {
  const refresh = position('rust-script scripts/check-dependencies-latest.rs --refresh-snapshot');
  const apply = position('rust-script scripts/check-dependencies-latest.rs --apply --offline || status=$?');
  assert.ok(refresh < apply);
  position('if [ "$status" -ne 0 ] && [ "$status" -ne 1 ]; then');
  assert.doesNotMatch(workflow, /\|\| true/, 'no failure is swallowed silently');
});

test('re-resolves every lockfile with the tool that owns it', () => {
  const apply = position('--apply --offline');
  const cargo = position('cargo update --workspace --manifest-path "$manifest"');
  const bun = position('bun install --lockfile-only');
  const npm = position('npm install --package-lock-only --ignore-scripts');
  assert.ok(apply < cargo && apply < bun && apply < npm);
  position("git ls-files -- '*bun.lock' '*package-lock.json'");
  assert.doesNotMatch(workflow, /--mode=save/);
});

test('re-runs all three gate stages after re-resolution before optional PR creation', () => {
  const resolved = position('npm install --package-lock-only');
  const stages = ['rust', 'wasm', 'web'].map((stage) =>
    position(`run: rust-script scripts/run-ci-gates.rs --stage ${stage}`),
  );
  assert.ok(resolved < stages[0] && stages[0] < stages[1] && stages[1] < stages[2]);
  const pullRequest = position('uses: peter-evans/create-pull-request@v8');
  assert.ok(stages[2] < pullRequest);
  position('branch: ${{ steps.publication.outputs.branch }}');
  position('delete-branch: false');
});

// Issue #1569: when the stable compiler moved to rustc 1.100.0 on 2026-11-13,
// the unchanged worker sources compiled to different bytes, this job failed its
// own wasm gate, and the pull request that should carry the rebuilt worker was
// never opened. The rebuilt worker and its input record are staged before the
// wasm gates, so they compare, and the pull request carries, the proposed commit.
test('stages the rebuilt worker and its input record before the wasm gates', () => {
  const rust = position('run: rust-script scripts/run-ci-gates.rs --stage rust');
  const write = position('node scripts/wasm-worker-inputs.mjs --write');
  const stage = position('git add -- js/formal_ai_worker.wasm data/meta/wasm-worker-inputs.lino');
  const check = position('node scripts/wasm-worker-inputs.mjs --check');
  const wasm = position('run: rust-script scripts/run-ci-gates.rs --stage wasm');
  const publication = position('run: node scripts/dependency-update-publication.mjs');
  assert.ok(rust < write && write < stage && stage < check && check < wasm && wasm < publication);
  assert.doesNotMatch(workflow, /run: sh js\/wasm-worker\/build\.sh/, 'one compiler invocation, the recorded one');
});
