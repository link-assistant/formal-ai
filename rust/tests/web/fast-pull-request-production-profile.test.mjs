import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
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
  return {github: {event_name:event,ref}, vars:{FORMAL_AI_FULL_PR_CI:enabled?'run':'skip',FORMAL_AI_NON_LINUX_CI:'run'},
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
    assert.equal(Boolean(evaluate(oldPredicate, full)),true,row.file+'/'+row.id+' explicit restore');
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
    'inactive-desktop-publication','verify-public-release-downloads','dependency-update-publication'])
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
  for (const name of ['check-readable-code','check-file-size','check-ci-speed','assemble-requirements','check-requirement-status','mirror-package-data'])
    assert(commands.includes('node scripts/' + name + '.mjs'), name);
  assert.doesNotMatch(commands, /cargo |rust-script |build-selected-wasm/u);
  const changelog = release.jobs.changelog.steps.map(step => step.run ?? '').join('\n');
  assert.match(changelog, /node scripts\/check-changelog-fragment[.]mjs/u);
  assert.doesNotMatch(changelog, /rust-script /u);
});
