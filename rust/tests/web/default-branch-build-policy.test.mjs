// Replays the recorded release/PR topology without compiling or publishing products.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync, mkdtempSync, mkdirSync, writeFileSync, rmSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {Script} from 'node:vm';
import YAML from 'yaml';
import {admission, changedPaths, checkVersions, readBaseManifest} from '../../../scripts/ci-pull-request-admission.mjs';
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

function apiResponse(content,status=200) {
  const bytes=content instanceof Uint8Array?content:new TextEncoder().encode(content);
  return {status,redirected:false,body:{getReader(){
    let sent=false;
    return {async read(){if(sent)return {done:true};sent=true;return {done:false,value:bytes};},releaseLock(){}};
  }}};
}
function sourceFixture({base='main',layout='root',content='[package]\nversion = "1.2.3"\n'}={}) {
  const repository='link-assistant/formal-ai', prefix='https://api.github.com/repos/'+repository+'/';
  const commit='1'.repeat(40), root='2'.repeat(40), rust='3'.repeat(40);
  const bytes=Buffer.from(content), blob=createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex');
  const cargo={path:'Cargo.toml',mode:'100644',type:'blob',sha:blob,size:bytes.length,url:prefix+'git/blobs/'+blob};
  const routes={
    [prefix+'git/ref/heads/'+encodeURIComponent(base)]:{ref:'refs/heads/'+base,url:prefix+'git/refs/heads/'+base,
      object:{type:'commit',sha:commit,url:prefix+'git/commits/'+commit}},
    [prefix+'git/commits/'+commit]:{sha:commit,url:prefix+'git/commits/'+commit,
      tree:{sha:root,url:prefix+'git/trees/'+root}},
    [prefix+'git/trees/'+root]:{sha:root,url:prefix+'git/trees/'+root,truncated:false,
      tree:layout==='root'?[cargo]:[{path:'rust',mode:'040000',type:'tree',sha:rust,url:prefix+'git/trees/'+rust}]},
    [prefix+'git/trees/'+rust]:{sha:rust,url:prefix+'git/trees/'+rust,truncated:false,tree:[cargo]},
    [prefix+'git/blobs/'+blob]:{sha:blob,url:prefix+'git/blobs/'+blob,encoding:'base64',size:bytes.length,
      content:bytes.toString('base64')+'\n'}
  };
  return {repository,prefix,commit,root,rust,blob,routes};
}
function versionCli(context,{base='main',headVersion='1.2.3',layout='root',apiStatus=200,
  event='pull_request',headRef='fix/metadata-fetch'}={}) {
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-version-api-'));
  context.after(()=>rmSync(directory,{recursive:true,force:true}));
  if(layout==='rust')mkdirSync(join(directory,'rust'));
  writeFileSync(join(directory,layout==='rust'?'rust/Cargo.toml':'Cargo.toml'),
    '[package]\nversion = "'+headVersion+'"\n');
  const calls=join(directory,'calls.jsonl');writeFileSync(calls,'');
  const preload=join(directory,'api-preload.mjs');
  writeFileSync(preload,[
    "import fs from 'node:fs';",
    "import child from 'node:child_process';",
    "import {syncBuiltinESMExports} from 'node:module';",
    "child.execFileSync=()=>{throw Error('GitFallbackForbidden');};",
    "syncBuiltinESMExports();",
    "const routes=JSON.parse(process.env.MOCK_API_ROUTES);let firstSignal;",
    apiResponse.toString(),
    "globalThis.WebAssembly=undefined;",
    "globalThis.fetch=async(url,options)=>{",
    " firstSignal??=options.signal;",
    " fs.appendFileSync(process.env.MOCK_API_CALLS,JSON.stringify({url,method:options.method,redirect:options.redirect,",
    "  tokenBound:options.headers.Authorization==='Bearer '+process.env.GH_TOKEN,",
    "  sameSignal:options.signal===firstSignal})+'\\n');",
    " const status=Number(process.env.MOCK_API_STATUS);",
    " return apiResponse(JSON.stringify(routes[url]??{}),status);",
    "};",
    ""
  ].join('\n'));
  const fixture=sourceFixture({base,layout});
  const result=spawnSync(process.execPath,['--max-old-space-size=384','--import',preload,resolve('scripts/ci-pull-request-admission.mjs'),'version'],{
    cwd:directory,encoding:'utf8',timeout:5000,maxBuffer:128*1024,
    env:{...process.env,GITHUB_EVENT_NAME:event,GITHUB_HEAD_REF:headRef,GITHUB_BASE_REF:base,
      GITHUB_REPOSITORY:fixture.repository,GH_TOKEN:'fixture-token',
      MOCK_API_CALLS:calls,MOCK_API_ROUTES:JSON.stringify(fixture.routes),MOCK_API_STATUS:String(apiStatus)}
  });
  return {...result,fixture,calls:readFileSync(calls,'utf8').trim().split('\n').filter(Boolean).map(JSON.parse)};
}
for(const [base,layout] of [['main','root'],['maintenance/1.x','rust']]) {
  test('version CLI reads exact immutable '+layout+' manifest metadata without Git',context=>{
    const result=versionCli(context,{base,layout}), f=result.fixture;
    assert.equal(result.error,undefined);assert.equal(result.signal,null);assert.equal(result.status,0,result.stderr);
    assert.match(result.stdout,/Package version unchanged[.]/u);
    const suffixes=['git/ref/heads/'+encodeURIComponent(base),'git/commits/'+f.commit,'git/trees/'+f.root,
      ...(layout==='rust'?['git/trees/'+f.rust]:[]),'git/blobs/'+f.blob];
    assert.deepEqual(result.calls.map(call=>call.url),suffixes.map(suffix=>f.prefix+suffix));
    for(const call of result.calls) {
      assert.equal(call.method,'GET');assert.equal(call.redirect,'error');
      assert.equal(call.tokenBound,true);assert.equal(call.sameSignal,true);
    }
  });
}
test('immutable API metadata does not excuse a changed package version',context=>{
  const result=versionCli(context,{headVersion:'1.2.4'});
  assert.notEqual(result.status,0);
  assert.match(result.stderr,/Use a changelog fragment instead of changing the crate version/u);
  assert.doesNotMatch(result.stdout,/Package version unchanged/u);
});
test('GitHub denials and server failures remain terminal without retries or Git fallback',context=>{
  for(const apiStatus of [403,404,503]) {
    const result=versionCli(context,{apiStatus});
    assert.notEqual(result.status,0);assert.match(result.stderr,/denied or unavailable/u);
    assert.equal(result.calls.length,1);
    assert.doesNotMatch(result.stdout,/Package version unchanged/u);
  }
});
test('invalid base cannot start API reads and non-PR or release bypasses stay unchanged',context=>{
  const invalid=versionCli(context,{base:'--foreign'});
  assert.notEqual(invalid.status,0);assert.deepEqual(invalid.calls,[]);
  for(const options of [{event:'push'},{headRef:'automated-release/1.2.3'}]) {
    const bypass=versionCli(context,options);
    assert.equal(bypass.status,0,bypass.stderr);assert.deepEqual(bypass.calls,[]);
  }
});
async function observeSourceFixture(mutate=()=>{},options={}) {
  const fixture=sourceFixture(options);mutate(fixture);
  const calls=[];
  const fetcher=async(url,request)=>{
    calls.push({url,request});
    return apiResponse(JSON.stringify(fixture.routes[url]??{}));
  };
  const promise=readBaseManifest({repository:fixture.repository,base:options.base??'main',
    token:'fixture-token',fetcher,...options});
  return {promise,calls,fixture};
}
test('base source custody rejects wrong refs, commits, trees, paths, modes and blob bytes',async()=>{
  const mutations=[
    f=>{f.routes[f.prefix+'git/ref/heads/main'].ref='refs/heads/foreign';},
    f=>{f.routes[f.prefix+'git/ref/heads/main'].url='https://api.github.com/repos/foreign/repo/git/refs/heads/main';},
    f=>{f.routes[f.prefix+'git/commits/'+f.commit].sha='4'.repeat(40);},
    f=>{f.routes[f.prefix+'git/trees/'+f.root].sha='4'.repeat(40);},
    f=>{f.routes[f.prefix+'git/trees/'+f.root].truncated=true;},
    f=>{const tree=f.routes[f.prefix+'git/trees/'+f.root].tree;tree.push({...tree[0]});},
    f=>{f.routes[f.prefix+'git/trees/'+f.root].tree[0].mode='120000';},
    f=>{f.routes[f.prefix+'git/trees/'+f.root].tree[0].path='foreign/Cargo.toml';},
    f=>{f.routes[f.prefix+'git/blobs/'+f.blob].sha='4'.repeat(40);},
    f=>{f.routes[f.prefix+'git/blobs/'+f.blob].content='not base64!!';},
    f=>{
      const bytes=Uint8Array.from([255]);
      const sha=createHash('sha1').update('blob '+bytes.length+'\0').update(bytes).digest('hex');
      const entry=f.routes[f.prefix+'git/trees/'+f.root].tree[0];
      Object.assign(entry,{sha,size:bytes.length,url:f.prefix+'git/blobs/'+sha});
      f.routes[f.prefix+'git/blobs/'+sha]={sha,url:entry.url,encoding:'base64',size:bytes.length,
        content:Buffer.from(bytes).toString('base64')};
    },
    f=>{f.routes[f.prefix+'git/blobs/'+f.blob].size++;},
    f=>{const value=f.routes[f.prefix+'git/blobs/'+f.blob];value.content=Buffer.from('[package]\nversion = "1.2.4"\n').toString('base64');}
  ];
  for(const mutate of mutations) {
    const result=await observeSourceFixture(mutate);
    await assert.rejects(result.promise);
    assert.ok(result.calls.length<=4);
  }
});
test('rust fallback rejects a malformed root manifest instead of switching source layouts',async()=>{
  const result=await observeSourceFixture(f=>{
    const root=f.routes[f.prefix+'git/trees/'+f.root], rust=f.routes[f.prefix+'git/trees/'+f.rust];
    root.tree.push({path:'rust',mode:'040000',type:'tree',sha:f.rust,url:rust.url});
    root.tree[0].mode='120000';
  });
  await assert.rejects(result.promise,/regular base Cargo manifest blob/u);
  assert.equal(result.calls.length,3);
});
test('base source reads enforce one aggregate deadline across all awaited metadata',async()=>{
  const fixture=sourceFixture(),calls=[];let clock=0,signal;
  await assert.rejects(readBaseManifest({repository:fixture.repository,base:'main',token:'fixture-token',now:()=>clock,
    fetcher:async(url,request)=>{
      calls.push(url);signal??=request.signal;assert.equal(request.signal,signal);
      clock+=4000;return apiResponse(JSON.stringify(fixture.routes[url]));
    }}),/aggregate deadline/u);
  assert.equal(calls.length,4);assert.equal(signal.aborted,true);
});
test('base source reads fail closed on transport, redirects, oversized bodies and invalid UTF8',async()=>{
  const fixture=sourceFixture();
  for(const fetcher of [
    async()=>{throw Error('ETIMEDOUT');},
    async()=>({...apiResponse('{}'),redirected:true}),
    async()=>apiResponse('x'.repeat(4*1024*1024+1)),
    async()=>apiResponse('{'),
    async()=>apiResponse(Uint8Array.from([255]))
  ]) {
    let calls=0;
    await assert.rejects(readBaseManifest({repository:fixture.repository,base:'main',token:'fixture-token',
      fetcher:async(...args)=>{calls++;return fetcher(...args);}}));
    assert.equal(calls,1);
  }
  let calls=0;
  await assert.rejects(readBaseManifest({repository:fixture.repository,base:'main',token:'',
    fetcher:async()=>{calls++;throw Error('Unexpected');}}),/token/u);
  assert.equal(calls,0);
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

test('base source diagnostics retain public identity and redact unavailable response content',async()=>{
  const records=[];
  const result=await observeSourceFixture(()=>{},{report:record=>records.push(record)});
  assert.equal(await result.promise,'[package]\nversion = "1.2.3"\n');
  assert.equal(records.length,1);
  const row=records[0];
  assert.equal(row.outcome,'observed');assert.equal(row.baseCommit,result.fixture.commit);
  assert.equal(row.path,'Cargo.toml');assert.equal(row.blobSHA,result.fixture.blob);
  assert.equal(row.requests,4);assert.equal(row.deadlineMilliseconds,15000);
  assert.ok(Number.isFinite(row.elapsedMs)&&row.elapsedMs>=0);
  assert.doesNotMatch(JSON.stringify(row),/fixture-token|version =|Authorization/u);
  const failed=[];
  await assert.rejects(readBaseManifest({repository:result.fixture.repository,base:'main',token:'fixture-token',
    report:record=>failed.push(record),fetcher:async()=>apiResponse('SECRET-response-fragment')}),error=>{
      assert.doesNotMatch(error.message,/SECRET-response-fragment|fixture-token/u);return true;
    });
  assert.equal(failed.length,1);assert.equal(failed[0].outcome,'unavailable');
  assert.equal(failed[0].phase,'ref');assert.equal(failed[0].status,200);assert.equal(failed[0].requests,1);
  assert.equal(failed[0].reason,'invalid base source JSON');
});
