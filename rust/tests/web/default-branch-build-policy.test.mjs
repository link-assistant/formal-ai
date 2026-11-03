// Replays the recorded release/PR topology without compiling or publishing products.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync, mkdtempSync, mkdirSync, writeFileSync, rmSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
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
      const buildCommands = [
        /cargo\s+(?:build|test|check|package|install|doc|run|bench|nextest|llvm-cov|clippy)/u,
        /rustc\s+(?!-vV|--version)|rust-script |docker (?:build|compose)|build-push-action/u,
        /build:web|build-selected-wasm|build-rust-api-docs|wasm-pack|deploy-pages@/u,
        /codeql-action[/]autobuild|[.]github[/]actions[/](?:formal-ai-binary|release-web)/u,
      ];
      const heavy = job.steps?.some(step => buildCommands.some(pattern => pattern.test((step.run??'')+' '+(step.uses??''))));
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
test('workflow lint checks only its source closure and retains the real ShellCheck canary', () => {
  const jobs=workflow('workflows.yml').jobs;
  for (const id of ['actionlint','zizmor']) {
    const checkout=jobs[id].steps.find(step=>step.uses==='actions/checkout@v7');
    assert.equal(checkout.with.filter,'blob:none');
    assert(checkout.with['sparse-checkout'].includes('/.github/'));
    assert(checkout.with['sparse-checkout'].includes('/rust/tests/fixtures/actionlint/'));
    assert.equal(checkout['timeout-minutes'],3);
  }
  assert(jobs.actionlint.steps.some(step=>step.run?.includes('shellcheck-canary.yml')));
});

function versionCli(context,{base='main',headVersion='1.2.3',layout='root',fetchFailure=false,
  event='pull_request',headRef='fix/metadata-fetch'}={}){
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-version-fetch-'));
  context.after(()=>rmSync(directory,{recursive:true,force:true}));
  if(layout==='rust')mkdirSync(join(directory,'rust'));
  writeFileSync(join(directory,layout==='rust'?'rust/Cargo.toml':'Cargo.toml'),
    '[package]\nversion = "'+headVersion+'"\n');
  const calls=join(directory,'calls.jsonl');writeFileSync(calls,'');
  const preload=join(directory,'git-preload.mjs');
  writeFileSync(preload,[
    "import fs from 'node:fs';",
    "import child from 'node:child_process';",
    "import {syncBuiltinESMExports} from 'node:module';",
    "child.execFileSync=(program,args,options)=>{",
    " if(program!=='git')throw Error('UnexpectedProgram');",
    " const row={args,timeout:options.timeout,maxBuffer:options.maxBuffer};",
    " fs.appendFileSync(process.env.MOCK_GIT_CALLS,JSON.stringify(row)+'\\n');",
    " if(args[0]==='fetch'){",
    "  if(process.env.MOCK_FETCH_FAILURE==='true')throw Object.assign(Error('git ETIMEDOUT'),{code:'ETIMEDOUT'});",
    "  return '';",
    " }",
    " if(args[0]==='show'){",
    "  const expected='origin/'+process.env.GITHUB_BASE_REF+':';",
    "  if(args[1]===expected+'Cargo.toml'&&process.env.MOCK_LAYOUT!=='rust')",
    "   return '[package]\\nversion = \"1.2.3\"\\n';",
    "  if(args[1]===expected+'rust/Cargo.toml')return '[package]\\nversion = \"1.2.3\"\\n';",
    "  throw Object.assign(Error('missing base manifest'),{status:128});",
    " }",
    " throw Error('UnexpectedGitArguments');",
    "};",
    "syncBuiltinESMExports();",
    ""
  ].join('\n'));
  const result=spawnSync(process.execPath,['--import',preload,resolve('scripts/ci-pull-request-admission.mjs'),'version'],{
    cwd:directory,encoding:'utf8',timeout:5000,maxBuffer:128*1024,
    env:{...process.env,GITHUB_EVENT_NAME:event,GITHUB_HEAD_REF:headRef,GITHUB_BASE_REF:base,
      MOCK_GIT_CALLS:calls,MOCK_LAYOUT:layout,MOCK_FETCH_FAILURE:String(fetchFailure)}
  });
  return {...result,calls:readFileSync(calls,'utf8').trim().split('\n').filter(Boolean).map(JSON.parse)};
}
for(const [base,layout] of [['main','root'],['maintenance/1.x','rust']]){
  test('version CLI fetches only the exact base metadata and retains '+layout+' manifest checks',context=>{
    const result=versionCli(context,{base,layout});
    assert.equal(result.error,undefined);assert.equal(result.signal,null);assert.equal(result.status,0,result.stderr);
    assert.match(result.stdout,/Package version unchanged[.]/u);
    assert.deepEqual(result.calls[0].args,['fetch','--filter=blob:none','--no-tags',
      'origin',base+':refs/remotes/origin/'+base,'--depth=1']);
    assert.deepEqual(result.calls.slice(1).map(row=>row.args),
      layout==='root'?[['show','origin/'+base+':Cargo.toml']]
        :[['show','origin/'+base+':Cargo.toml'],['show','origin/'+base+':rust/Cargo.toml']]);
    for(const row of result.calls){
      assert.equal(row.timeout,15000);assert.equal(row.maxBuffer,4*1024*1024);
    }
  });
}
test('metadata-only fetch does not excuse a changed package version',context=>{
  const result=versionCli(context,{headVersion:'1.2.4'});
  assert.notEqual(result.status,0);
  assert.match(result.stderr,/Use a changelog fragment instead of changing the crate version/u);
  assert.doesNotMatch(result.stdout,/Package version unchanged/u);
});
test('failed metadata fetch remains terminal without a manifest read or retry',context=>{
  const result=versionCli(context,{fetchFailure:true});
  assert.notEqual(result.status,0);assert.match(result.stderr,/ETIMEDOUT/u);
  assert.equal(result.calls.length,1);assert.equal(result.calls[0].args[0],'fetch');
  assert.doesNotMatch(result.stdout,/Package version unchanged/u);
});
test('invalid base cannot start Git and existing non-PR or release bypasses stay unchanged',context=>{
  const invalid=versionCli(context,{base:'--foreign'});
  assert.notEqual(invalid.status,0);assert.deepEqual(invalid.calls,[]);
  for(const options of [{event:'push'},{headRef:'automated-release/1.2.3'}]){
    const bypass=versionCli(context,options);
    assert.equal(bypass.status,0,bypass.stderr);assert.deepEqual(bypass.calls,[]);
  }
});

test('changelog acquisition keeps complete Git history and only hydrates its checker closure', () => {
  const job=workflow('release.yml').jobs.changelog;
  assert.equal(job['timeout-minutes'],10);
  assert.equal(job.if,"github.event_name == 'pull_request' && needs.detect-changes.outputs.any-code-changed == 'true'");
  assert.deepEqual(job.needs,['detect-changes']);
  const checkout=job.steps.find(step=>step.uses==='actions/checkout@v7');
  assert.equal(checkout['timeout-minutes'],2);
  assert.equal(checkout.with['persist-credentials'],false);
  assert.equal(checkout.with['fetch-depth'],0);
  assert.equal(checkout.with.filter,'blob:none');
  assert.equal(checkout.with['sparse-checkout-cone-mode'],false);
  assert.equal(checkout.with.ref,undefined);
  assert.deepEqual(checkout.env,{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'30'});
  assert.deepEqual(checkout.with['sparse-checkout'].trim().split(/\s+/u),[
    '/scripts/check-changelog-fragment.mjs','/Cargo.toml','/rust/Cargo.toml','/package.json'
  ]);
  assert.deepEqual(job.steps.filter(step=>step.run).map(step=>({run:step.run,env:step.env})),[
    {run:'node scripts/check-changelog-fragment.mjs',env:{GITHUB_BASE_REF:'${{ github.base_ref }}'}}
  ]);
  const checker=read('scripts/check-changelog-fragment.mjs');
  assert.deepEqual([...checker.matchAll(/^import .* from '([^']+)';$/gmu)].map(match=>match[1]),
    ['node:child_process','node:fs']);
  assert.match(checker,/import \{ existsSync \} from 'node:fs';/u);
  assert.deepEqual([...checker.matchAll(/existsSync\('([^']+)'\)/gu)].map(match=>match[1]),
    ['./Cargo.toml','./rust/Cargo.toml']);
  // A sparse worktree does not filter names from a full-history Git tree diff.
  assert(checker.includes("const output = exec('git', ['diff', '--name-only', `origin/${base}...HEAD`]);"));
  assert.equal([...checker.matchAll(/exec\('git'/gu)].length,1);
  assert.doesNotMatch(checker,/readFileSync|readdirSync|['"]fetch['"]/u);
});
