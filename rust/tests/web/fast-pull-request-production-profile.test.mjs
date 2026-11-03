import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,existsSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {Script} from 'node:vm';
import YAML from 'yaml';
import {selectFastTests} from '../../../scripts/list-fast-pull-request-tests.mjs';
const read = file => readFileSync(file, 'utf8');
const manifest = JSON.parse(read('data/benchmarks/fast-pull-request-retained-jobs.json'));
const workflow = file => YAML.parse(read('.github/workflows/' + file));
const evaluate = (expression, context) => new Script(expression.trim()
  .replace(/^\$\{\{\s*/u, '').replace(/\s*\}\}$/u, '')
  .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu, (_, key) => '[' + JSON.stringify(key) + ']'))
  .runInNewContext({cancelled: () => false, always: () => true, ...context}, {timeout: 100});
function context(event, ref = 'refs/pull/1500/merge', enabled = false) {
  const outputs = Object.fromEntries(['any-code-changed','rs-changed','toml-changed','pipeline-changed','rust-changed','js-changed','ts-changed'].map(k => [k,'true']));
  return {github: {event_name:event,ref}, vars:{FORMAL_AI_FULL_PR_CI:enabled?'run':'skip',FORMAL_AI_NON_LINUX_CI:'run',FORMAL_AI_FULL_INTEGRATION:'run'},
    inputs:{}, needs:Object.fromEntries(['detect-changes','changes','build-artifacts','lint','test','macos-core-tests','js','ts','native-source','native','resolve']
      .map(k => [k,{result:'success',outputs:{...outputs,should_build:'true',build:'true'}}]))};
}
test('heavy original job bodies remain intact and default PR dispatches cannot allocate them', () => {
  for (const row of manifest.jobs) {
    const job = workflow(row.file).jobs[row.id];
    const payload = {steps:job.steps,uses:job.uses,with:job.with,strategy:job.strategy};
    assert.equal(createHash('sha256').update(JSON.stringify(payload)).digest('hex'), row.payloadSha256, row.file+'/'+row.id+' retained payload');
    assert.equal(Boolean(evaluate(job.if, context('pull_request'))),false,row.file+'/'+row.id);
    assert.equal(Boolean(evaluate(job.if, context('workflow_dispatch','refs/heads/fix/example'))),false,row.file+'/'+row.id+' bot dispatch');
    const full = context('pull_request',undefined,true);
    const oldPredicate=job.if.replace(/^\$\{\{\s*/u,'').replace(/\s*\}\}$/u,'');
    assert.equal(Boolean(evaluate(oldPredicate, full)),false,row.file+'/'+row.id+' override cannot restore PR builds');
  }
});
test('main keeps production builds and original selected integrations', () => {
  for (const row of manifest.jobs) {
    if (row.file === 'release.yml' && ['docker-build','docker-slim'].includes(row.id)) continue; // PR-only probes; main publishes real images separately.
    const job=workflow(row.file).jobs[row.id];
    assert.equal(Boolean(evaluate(job.if,context('push','refs/heads/main'))),true,row.file+'/'+row.id);
  }
  const release=workflow('release.yml');
  assert.equal(release.jobs['native-container-images'].uses,'./.github/workflows/container-images.yml');
  assert.match(release.jobs['auto-release'].if,/needs.build.result == 'success'/u);
  assert.match(read('.github/workflows/verify-release-downloads.yml'),/name: Verify Public Release Downloads/u);
});
test('fast inventory retains coding, questions, reports and captured CI failure regressions', () => {
  const selected=selectFastTests(JSON.parse(read('data/benchmarks/fast-pull-request-javascript-tests.json')));
  for(const name of ['ordinary-literal-addition','issue-1180-repository-qa','issue-1438-report-and-edit','generated-test-retirement','scheduled-published-admission-race',
    'inactive-desktop-publication','verify-public-release-downloads','dependency-update-publication','create-release-with-recovery',
    'github-json-observation','self-authored-publication-source'])
    assert(selected.includes('rust/tests/web/'+name+'.test.mjs'),name);
  for(const files of [[],['../outside.test.mjs'],[selected[0],selected[0]],['rust/tests/web/missing.test.mjs']])
    assert.throws(()=>selectFastTests({schema:'fast-pr-javascript-tests/v1',files}));
  assert.throws(()=>selectFastTests({schema:'unknown',files:selected}));
  const layer=workflow('layered-ci.yml').jobs.js;
  assert(evaluate(layer.if,context('pull_request')));
  const step=layer.steps.find(s=>s.name==='Run this shard of the JavaScript suites');
  assert.equal(evaluate(step.env.FAST_PR_PROFILE,context('pull_request')),true);
  assert.equal(evaluate(step.env.FAST_PR_PROFILE,context('push','refs/heads/main')),false);
  assert.match(step.run,/printf '%s\\n' rust\/tests\/web\/\*\.test\.mjs/u);
  assert.match(read('.github/workflows/pull-request-check-profile.yml'),/no execution or acceptance credit/u);
});

test('fast source lint is mandatory on PRs without invoking native compilation', () => {
  const release = workflow('release.yml'), job = release.jobs['fast-pr-lint'];
  assert.equal(evaluate(job.if, context('pull_request')), true);
  assert.equal(evaluate(job.if, context('push', 'refs/heads/main')), false);
  assert.match(job.concurrency.group, /-fast-pr-lint$/u);
  assert.equal(evaluate(job.concurrency['cancel-in-progress'], context('pull_request')), true);
  assert.equal(evaluate(job.concurrency['cancel-in-progress'], context('push', 'refs/heads/main')), false);
  assert(release.jobs['pipeline-status'].needs.includes('fast-pr-lint'));
  const commands = job.steps.map(step => step.run ?? '').join('\n');
  for (const name of ['check-readable-code','check-file-size','check-ci-speed','check-hardcoded-language','check-js-literals','assemble-requirements','check-requirement-status','mirror-package-data'])
    assert(commands.includes('node scripts/' + name + '.mjs'), name);
  assert.doesNotMatch(commands, /cargo |rust-script |build-selected-wasm/u);
  const changelog = release.jobs.changelog.steps.map(step => step.run ?? '').join('\n');
  assert.match(changelog, /node scripts\/check-changelog-fragment[.]mjs/u);
  assert.doesNotMatch(changelog, /rust-script /u);
});


test('explicit translation regeneration selects only the source emitter on every ref', () => {
  const layered = workflow('layered-ci.yml');
  const regeneration = layered.jobs['regenerate-translations'];
  assert(regeneration);
  assert.deepEqual(regeneration.permissions, {contents: 'read'});
  assert.equal(regeneration.needs, undefined);
  assert(regeneration['timeout-minutes'] <= 10);
  const commands = regeneration.steps.map(step => step.run ?? '').join('\n');
  assert.match(commands, /node --max-old-space-size=384 scripts\/regenerate-targeted-js-rust[.]mjs/u);
  assert.doesNotMatch(commands, /\bcargo\b|\brustc\b|\bdocker\b|desktop|vscode/u);
  for (const ref of ['refs/heads/main', 'refs/heads/fix/example']) {
    const dispatched = context('workflow_dispatch', ref);
    dispatched.inputs = {mode: 'regenerate-translations', 'diagnostic-native-run': '123', 'diagnostic-native-source': 'a'.repeat(40)};
    for (const [name, job] of Object.entries(layered.jobs))
      assert.equal(Boolean(evaluate(job.if, dispatched)), name === 'regenerate-translations', name + ' explicit source mode');
  }
  for (const event of ['pull_request', 'push'])
    assert.equal(Boolean(evaluate(regeneration.if, context(event, 'refs/heads/main'))), false, event);
});

test('measured TeachF integration stays on main while fast coding and QA controls remain selected', () => {
  const selected=selectFastTests(JSON.parse(read('data/benchmarks/fast-pull-request-javascript-tests.json')));
  const teach='rust/tests/web/pull-request-1188-teach-f.test.mjs';
  assert.equal(selected.includes(teach),false);
  assert.equal(existsSync(teach),true);
  const step=workflow('layered-ci.yml').jobs.js.steps
    .find(item=>item.name==='Run this shard of the JavaScript suites');
  assert.equal(evaluate(step.env.FAST_PR_PROFILE,context('push','refs/heads/main')),false);
  assert.match(step.run,/printf '%s\\n' rust\/tests\/web\/\*\.test\.mjs/u);
  for(const name of ['ordinary-literal-addition','issue-1180-repository-qa',
    'issue-1438-report-and-edit','file-command-contract','literal-create-shell-readback',
    'file-obligation-receipts','source-read-provenance','owned-edit-contract'])
    assert.equal(selected.includes('rust/tests/web/'+name+'.test.mjs'),true,name);
});


test('essential PR checkouts reserve their original test clocks and avoid unrelated archive hydration', () => {
  const release = workflow('release.yml'), layer = workflow('layered-ci.yml');
  const version = release.jobs['version-check'], javascript = layer.jobs.js;
  assert.equal(version['timeout-minutes'], 5);
  assert.equal(javascript['timeout-minutes'], 10);
  const check = step => {
    assert.equal(step.uses, 'actions/checkout@v7');
    assert.equal(step['timeout-minutes'], 2);
    assert.deepEqual(step.env, {GIT_HTTP_LOW_SPEED_LIMIT:'1024', GIT_HTTP_LOW_SPEED_TIME:'20'});
    assert.equal(step.with['persist-credentials'], false);
    assert.equal(step.with['fetch-depth'], 1);
    assert.equal(step.with.filter, 'blob:none');
    assert.equal(step.with['sparse-checkout-cone-mode'], false);
    assert.equal(step.with.ref, undefined, 'keep actual event source selection');
  };
  for (const job of [version, javascript]) {
    const checkout = job.steps[0];
    check(checkout);
    for (const mutate of [step=>step['timeout-minutes']=10,
      step=>step.with['fetch-depth']=0, step=>step.with['persist-credentials']=true,
      step=>step.env.GIT_HTTP_LOW_SPEED_TIME='300']) {
      const changed = structuredClone(checkout); mutate(changed);
      assert.throws(()=>check(changed));
    }
  }
  assert.equal(version.steps[0].with['sparse-checkout'],
    '/scripts/ci-pull-request-admission.mjs\n/scripts/lib/documentation-paths.mjs\n/Cargo.toml\n/rust/Cargo.toml\n');
  assert.equal(javascript.steps[0].with['sparse-checkout'],
    '/*\n!/experiments/formal_ai_subagent/evidence/coordinator-1188/\n');
  assert.equal(javascript.steps.find(step=>step.name==='Run this shard of the JavaScript suites').env.TEST_BUDGET_SECONDS, 360);
  assert.match(release.jobs['version-check'].steps.at(-1).run, /ci-pull-request-admission[.]mjs version/u);
});


test('base identity acquisition needs only its script and keeps one shared pin per workflow', () => {
  const job = workflow('pin-base-commit.yml').jobs.pin, checkout = job.steps[0];
  assert.equal(job['timeout-minutes'], 5);
  assert.equal(job.if, "github.event_name == 'pull_request'");
  assert.equal(checkout['timeout-minutes'], 2);
  assert.equal(checkout.with['fetch-depth'], 1);
  assert.equal(checkout.with.filter, 'blob:none');
  assert.equal(checkout.with['persist-credentials'], false);
  assert.equal(checkout.with['sparse-checkout'], '/scripts/pin-base-commit.sh');
  assert.equal(checkout.with['sparse-checkout-cone-mode'], false);
  assert.equal(checkout.with.ref, undefined);
  assert.deepEqual(job.env, {GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'});
  assert.deepEqual(checkout.env, job.env);
  const pin = job.steps.find(step=>step.id==='pin');
  assert.equal(pin.env.BASE_REF, '${{ github.base_ref }}');
  assert.equal(pin.run, 'bash scripts/pin-base-commit.sh');
  assert.equal(job.outputs.commit, '${{ steps.pin.outputs.commit }}');
  assert.match(read('scripts/pin-base-commit.sh'), /max_attempts=5/u);
});


test('TypeScript parity acquires its complete source inputs instead of the unrelated repository tree', () => {
  const job = workflow('layered-ci.yml').jobs.ts, checkout = job.steps[0];
  assert.equal(job['timeout-minutes'], 10);
  assert.equal(checkout['timeout-minutes'], 2);
  assert.equal(checkout.with['fetch-depth'], 1);
  assert.equal(checkout.with.filter, 'blob:none');
  assert.equal(checkout.with['persist-credentials'], false);
  assert.equal(checkout.with.ref, undefined);
  assert.equal(checkout.with['sparse-checkout-cone-mode'], false);
  assert.equal(checkout.with['sparse-checkout'], '/js/\n/ts/\n/scripts/translate-es.mjs\n/package.json\n/.gitignore\n');
  assert.deepEqual(checkout.env, {GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'});
  assert.deepEqual(job.steps.slice(1).map(step=>step.name), [
    'The ts tree covers exactly the js tree', 'The ts tree is what the JavaScript translator renders'
  ]);
  assert.match(job.steps[1].run, /cd js && find/u);
  assert.match(job.steps[1].run, /cd ts && find/u);
  assert.match(job.steps[2].run, /node scripts\/translate-es[.]mjs --check/u);
});
