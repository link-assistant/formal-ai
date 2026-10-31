// Replays the recorded release/PR topology without compiling or publishing products.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import {Script} from 'node:vm';
import YAML from 'yaml';
import {admission, changedPaths, checkVersions} from '../../../scripts/ci-pull-request-admission.mjs';
const read = path => readFileSync(path, 'utf8');
const workflow = name => YAML.parse(read('.github/workflows/' + name));
function evaluate(expression, event, ref, variables = {}) {
  const source = String(expression).trim().replace(/^\$\{\{\s*|\s*\}\}$/gu, '')
    .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu, (_, key) => '[' + JSON.stringify(key) + ']');
  const outputs = new Proxy({}, {get: () => 'true'});
  const needs = new Proxy({}, {get: () => ({result:'success', outputs})});
  return new Script(source).runInNewContext({github:{event_name:event,ref,event:{inputs:{release_mode:"checks"},workflow_run:{head_branch:'main'}}},
    vars:{FORMAL_AI_FULL_PR_CI:'run',FORMAL_AI_FULL_INTEGRATION:'run',...variables}, needs,
    inputs:{},cancelled:()=>false,always:()=>true,fromJSON:JSON.parse}, {timeout:100});
}
test('all former heavy PR overrides remain closed, including bot branch dispatch', () => {
  const manifest = JSON.parse(read('data/benchmarks/fast-pull-request-retained-jobs.json'));
  for (const row of manifest.jobs) {
    const job = workflow(row.file).jobs[row.id];
    for (const [event,ref] of [['pull_request','refs/pull/1500/merge'],['pull_request_target','refs/heads/main'],['workflow_dispatch','refs/heads/fix/example']])
      assert.equal(Boolean(evaluate(job.if,event,ref)),false,row.file+'/'+row.id+' '+event);
  }
});
test('actual compiler, image, application and deployment entries refuse PR execution', () => {
  for (const file of readdirSync('.github/workflows').filter(file=>file.endsWith('.yml'))) {
    for (const [id,job] of Object.entries(workflow(file).jobs)) {
      if (['changes','detect-changes','version-check'].includes(id)) continue;
      const heavy = job.steps?.some(step => /cargo\s+(?:build|test|check|package|install|doc|run|bench|nextest|llvm-cov|clippy)|rustc\s+(?!-vV|--version)|rust-script |docker (?:build|compose)|build-push-action|build:web|build-selected-wasm|build-rust-api-docs|wasm-pack|deploy-pages@|codeql-action[/]autobuild|[.]github[/]actions[/](?:formal-ai-binary|release-web)/u.test((step.run??'')+' '+(step.uses??'')));
      if (!heavy || file === 'release-staged.yml') continue; // Main-only prepare-source dominates every generated stage.
      assert(job.if,file+'/'+id+' needs explicit admission');
      assert.equal(Boolean(evaluate(job.if,'pull_request','refs/pull/1500/merge')),false,file+'/'+id);
      assert.equal(Boolean(evaluate(job.if,'workflow_dispatch','refs/heads/fix/example')),false,file+'/'+id+' dispatch');
    }
  }
});
test('recorded Docker cold rebuild is optional and cannot hold the production build behind another build', () => {
  const jobs = workflow('container-images.yml').jobs;
  assert.match(jobs['independent-full'].if,/FORMAL_AI_COLD_CONTAINER_REBUILD/u);
  assert.equal(Boolean(evaluate(jobs['independent-full'].if,'push','refs/heads/main')),false);
  assert.equal(Boolean(evaluate(jobs['independent-full'].if,'push','refs/heads/main',{FORMAL_AI_COLD_CONTAINER_REBUILD:'run'})),true);
  for (const id of ['validate-images','publish-images']) {
    assert(!jobs[id].needs.includes('independent-full'));
    assert(jobs[id].needs.includes('native'));
  }
  for (const mode of ['auto','manual']) {
    const stages=workflow('release-staged.yml').jobs;
    assert(stages[mode+'_verify-package'].needs.includes(mode+'_compile-release'));
    assert.match(stages[mode+'_verify-package'].steps.find(s=>s.name==='Verify packaged crate archive').run,/--no-verify/u);
    const publish=stages[mode+'_publish-crate'];
    assert(publish.needs.includes(mode+'_compile-release'));
    assert.match(publish.steps.find(s=>s.name==='Publish to Crates.io').run,/--verified-build true/u);
  }
});
test('PRs use one JS unit group; full main inventory retains its measured shard plan', () => {
  const job=workflow('layered-ci.yml').jobs.js;
  assert.deepEqual(Array.from(evaluate(job.strategy.matrix.shard,'pull_request','refs/pull/1500/merge')),[1]);
  assert.equal(evaluate(job.strategy.matrix.shard,'push','refs/heads/main').length,8);
  for (const file of ['release.yml','layered-ci.yml','coverage.yml']) {
    const jobs=workflow(file).jobs, job=jobs.changes??jobs['detect-changes'];
    assert(job.steps.some(s=>s.run?.includes('node scripts/ci-pull-request-admission.mjs detect')));
    for (const step of job.steps.filter(s=>s.run?.includes('install-rust-script') || s.uses==='dtolnay/rust-toolchain@stable'))
      assert.equal(Boolean(evaluate(step.if,'pull_request','refs/pull/1500/merge')),false);
  }
});
test('complete PR diff, deletions and source-to-doc renames cannot disappear behind a final docs commit', () => {
  const calls=[];
  const paths=changedPaths('pull_request',undefined,args=>{
    calls.push(args);
    return args[0]==='rev-list'?'merge base head\n':'rust/src/a.rs\0docs/a.md\0';
  });
  assert.deepEqual(calls[1],['diff','--no-renames','--name-only','-z','base','head']);
  assert.equal(admission(paths)['any-code-changed'],true);
  assert.equal(admission(['docs/guide.md','changelog.d/fix.md'])['any-code-changed'],false);
  assert.equal(admission(['data/runtime.md'])['any-code-changed'],true);
  assert.equal(admission(['rust/tests/web/new.test.mjs'])['js-changed'],true);
  assert.throws(()=>changedPaths('pull_request',undefined,()=> 'head parent'));
  checkVersions('[package]\nversion = "1.2.3"\n','[package]\nversion = "1.2.3"\n');
  assert.throws(()=>checkVersions('[package]\nversion = "1.2.4"\n','[package]\nversion = "1.2.3"\n'));
  assert.throws(()=>checkVersions('[dependencies]\nversion = "1.2.3"\n',''));
});
