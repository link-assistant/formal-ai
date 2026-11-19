// Replays the recorded release/PR topology without compiling or publishing products.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync, mkdtempSync, mkdirSync, writeFileSync, rmSync, existsSync} from 'node:fs';
import {dirname,join,relative,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {Script} from 'node:vm';
import {builtinModules} from 'node:module';
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
for (const [label, needs, expectedStatus] of [
  ['successful and skipped dependencies', {lint: {result: 'success'}, test: {result: 'skipped'}}, 0],
  ['a failed dependency', {lint: {result: 'failure'}}, 1],
  ['a cancelled dependency', {'native-type-check': {result: 'cancelled'}}, 1],
]) {
  test('pipeline status reports ' + label + ' on every branch', {timeout: 15000}, () => {
    for (const isMain of ['true', 'false']) {
      const result = spawnSync('bash', ['scripts/check-pipeline-status.sh'], {encoding: 'utf8', timeout: 5000,
        env: {...process.env, NEEDS_JSON: JSON.stringify(needs), IS_MAIN: isMain}});
      assert.equal(result.signal, null, result.stderr);
      assert.equal(result.status, expectedStatus, `IS_MAIN=${isMain}: ${result.stdout}${result.stderr}`);
      if (expectedStatus) assert.doesNotMatch(result.stdout, /All required jobs succeeded/u);
    }
  });
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
      if (file === 'native-type-check.yml') continue; // The one focused PR check (#1191), pinned below.
      assert(job.if,file+'/'+id+' needs explicit admission');
      assert.equal(Boolean(evaluate(job.if,'pull_request','refs/pull/1500/merge')),false,file+'/'+id);
      assert.equal(Boolean(evaluate(job.if,'workflow_dispatch','refs/heads/fix/example')),false,file+'/'+id+' dispatch');
    }
  }
});
// Issue #1191: the only PR job that runs the Rust compiler is main's clippy
// profile on the library and binaries, plus the isolated regression targets.
// Full suites, product builds and uploads remain main-only.
test('the focused PR native check limits execution to lint and isolated regressions', () => {
  const job = workflow('release.yml').jobs['native-type-check'];
  assert.equal(job.uses, './.github/workflows/native-type-check.yml');
  assert.deepEqual(job.permissions, {contents: 'read'});
  const called = workflow('native-type-check.yml');
  assert.deepEqual(Object.keys(called.on), ['workflow_call'], 'only release.yml selects it');
  assert.equal(called.env.RUSTFLAGS, '-Dwarnings', 'the same warnings-as-errors flags as main');
  // superseded_read_only_work_releases_its_runners (step_budgets_within_job_clocks.rs) reads the
  // called file's own top-level concurrency, before `jobs:`, not a job-level block.
  const head = read('.github/workflows/native-type-check.yml').split('\njobs:\n')[0];
  assert.match(head, /\nconcurrency:\n  group: \$\{\{ github\.workflow \}\}-\$\{\{ github\.ref \}\}-native-type-check\n  cancel-in-progress: \$\{\{ github\.ref != 'refs\/heads\/main' \}\}\n/u);
  const [check, ...others] = Object.values(called.jobs);
  assert.equal(others.length, 0);
  const runs = check.steps.map(step => step.run).filter(Boolean);
  assert.deepEqual(runs.filter(run => !run.startsWith('echo ')), ['bash scripts/ci-clippy-profile.sh --library-and-binaries',
    'cargo test --manifest-path rust/Cargo.toml --locked --no-fail-fast --test calendar_currency --test openai_forced_functions --test meta_executor_parity --test synthesis_contracts --test cnl_output --test cnl_parity --test unknown_reasoning --test data_files --test scheduling_contracts --test operational_contracts --test reasoning_contracts -- --test-threads=1',
    'regression_status=0\n' +
    'cargo test --manifest-path rust/Cargo.toml --locked --lib solver_unknown_reasoning::tests:: -- --test-threads=1 || regression_status=$?\n' +
    'cargo test --manifest-path rust/Cargo.toml --locked --test any_language a_language_without_localized_openers_reports_a_gap_not_english -- --exact --test-threads=1 || regression_status=$?\n' +
    'exit "$regression_status"\n',
    'node --test --test-concurrency=1 rust/tests/web/issue-1590-forced-functions.test.mjs',
    'node --test --test-concurrency=1 rust/tests/web/issue-1586-structured-output.test.mjs',
    'node --test --test-concurrency=1 rust/tests/web/issue-1396-native.test.mjs']);
  assert.equal(called.on.workflow_call.inputs['compile-inputs-changed'].default, true);
  for (const step of check.steps.filter(step => (step.uses && step.uses !== 'actions/setup-node@v7') || ['Type-check and lint the library and binaries', 'Verify isolated native behavior regressions'].includes(step.name)))
    assert.equal(step.if, '${{ inputs.compile-inputs-changed }}');
  assert.equal(check.steps[0].if, '${{ !inputs.compile-inputs-changed }}');
  assert.equal(workflow('release.yml').on.pull_request['paths-ignore'], undefined, 'required contexts must report for docs-only PRs');
  assert.deepEqual(check.steps.find(step => step.name === 'Verify isolated native behavior regressions').env,
    {CARGO_BUILD_JOBS: '2', CARGO_PROFILE_TEST_OPT_LEVEL: '0', FORMAL_AI_OFFLINE: '1'});
  assert.equal(called.env.FORMAL_AI_OFFLINE, undefined, 'native CLI transport replays keep the default network policy');
  assert.equal(check.env?.FORMAL_AI_OFFLINE, undefined, 'offline mode belongs to the deterministic behavior step');
  for (const command of runs.filter(command => command.startsWith('node --test '))) {
    assert.deepEqual(check.steps.find(step => step.run === command).env,
      {FORMAL_AI_TEST_NATIVE_CLI: '${{ github.workspace }}/rust/target/debug/formal-ai'});
  }
  assert.deepEqual(check.steps.find(step => step.uses === 'dtolnay/rust-toolchain@stable').with, {components: 'clippy'});
  assert(!check.steps.some(step => /upload-artifact|attest|cache@|cache\/save/u.test(step.uses ?? '')));
  assert.deepEqual(check.steps.find(step => step.uses === './.github/actions/cache-cargo-registry').with, {'restore-only': 'true'});
  assert.deepEqual(check.permissions, {contents: 'read'});
  assert(check['timeout-minutes'] <= 20);
  assert.equal(check.outputs ?? called.on.workflow_call?.outputs, undefined, 'a lint check publishes no evidence');
  const selected = (event, ref, result, value) => Boolean(new Script(String(job.if).trim().replace(/^\$\{\{\s*|\s*\}\}$/gu, '')
    .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu, (_, key) => '[' + JSON.stringify(key) + ']'))
    .runInNewContext({github: {event_name: event, ref}, cancelled: () => false,
      needs: {'detect-changes': {result, outputs: value === undefined ? {} : {'native-type-check': value}}}}, {timeout: 100}));
  assert.equal(selected('pull_request', 'refs/pull/1/merge', 'success', 'true'), true);
  assert.equal(selected('pull_request', 'refs/pull/1/merge', 'success', 'false'), true, 'docs-only and JS-only PRs still report the required context');
  const compile = (result, value) => new Script(job.with['compile-inputs-changed'].slice(3, -2).replaceAll('needs.detect-changes', "needs['detect-changes']").replaceAll('outputs.native-type-check', "outputs['native-type-check']")).runInNewContext({needs: {'detect-changes': {result, outputs: {'native-type-check': value}}}});
  assert.equal(compile('success', 'false'), false);
  assert.equal(compile('success', 'true'), true);
  assert.equal(compile('failure', 'false'), true);
  assert.equal(compile('success', undefined), true);
  assert.equal(selected('pull_request', 'refs/pull/1/merge', 'failure', 'false'), true, 'an unknown classification is conservative');
  assert.equal(selected('pull_request', 'refs/pull/1/merge', 'success', undefined), true);
  assert.equal(selected('workflow_dispatch', 'refs/heads/fix/example', 'skipped', undefined), true);
  assert.equal(selected('push', 'refs/heads/main', 'success', 'true'), false, 'main keeps its full lint and builds instead');
  assert(workflow('release.yml').jobs['pipeline-status'].needs.includes('native-type-check'));
  const native = paths => admission(paths)['native-type-check'];
  for (const path of ['rust/src/proof_engine/types.rs', 'rust/build.rs', 'rust/Cargo.lock', 'rust/embedded/data/seed/a.lino',
    'data/seed/api-cache/x.lino', '.cargo/config.toml', 'rust-toolchain.toml', '.github/workflows/release.yml', '.github/workflows/native-type-check.yml',
    '.github/actions/setup-sccache/action.yml', 'scripts/ci-clippy-profile.sh', 'scripts/ci-pull-request-admission.mjs',
    'scripts/lib/documentation-paths.mjs', 'rust/tests/web/issue-1590-forced-functions.test.mjs',
    'rust/tests/web/issue-1586-structured-output.test.mjs',
    'rust/tests/web/support/structured-log-fixture.mjs', 'js/server/structured-log.mjs', 'clippy.toml', 'new-top-level/unknown.txt'])
    assert.equal(native([path]), true, path);
  for (const path of ['docs/a.md', 'changelog.d/x.md', 'js/app.js', 'rust/tests/web/a.test.mjs',
    'rust/tests/unit/proof_request.rs', 'rust/tests/fixtures/unrelated.json', 'data/parity/unrelated.json', 'scripts/check-tests-as-docs.mjs', 'data/meta/x.txt',
    '.github/workflows/links.yml', 'package.json', 'bun.lock'])
    assert.equal(native([path]), false, path);
  assert.equal(native([]), true, 'an empty observation is unknown, so the check runs');
});
// A test-only change must execute the same isolated target that consumes it.
test('isolated native regressions select execution for their sources and embedded fixtures', () => {
  const step = Object.values(workflow('native-type-check.yml').jobs)[0].steps
    .find(step => step.name === 'Verify isolated native behavior regressions');
  const manifest = read('rust/Cargo.toml').split('[[test]]').slice(1);
  const targets = [...step.run.matchAll(/--test (\S+)/gu)].map(match => match[1]);
  assert(targets.length > 0);
  for (const target of targets) {
    const block = manifest.find(block => new RegExp('^name = "' + target + '"$', 'mu').test(block));
    assert(block, 'explicit Cargo target ' + target);
    const source = 'rust/' + /^path = "([^"]+)"$/mu.exec(block)[1];
    assert.equal(admission([source])['native-type-check'], true, source);
    for (const match of read(source).matchAll(/include_str!\(\s*"([^"]+)"/gu)) {
      const fixture = relative(process.cwd(), resolve(dirname(source), match[1]));
      assert(readFileSync(fixture).length > 0, fixture);
      assert.equal(admission([fixture])['native-type-check'], true, fixture);
    }
  }
});
test('native CNL parity selects execution when a discovered language fixture changes', () => {
  const source = read('rust/tests/unit/cnl_parity.rs');
  assert.match(source, /join\("\.\.\/data\/parity\/issue-1584-cnl"\)/u);
  const fixtures = readdirSync('data/parity/issue-1584-cnl').filter(name => name.endsWith('.json'));
  assert(fixtures.length > 0);
  for (const name of fixtures) assert.equal(admission(['data/parity/issue-1584-cnl/' + name])['native-type-check'], true, name);
  assert.equal(admission(['data/parity/issue-1584-cnl/new-language.json'])['native-type-check'], true);
  assert.equal(admission(['data/parity/unrelated.json'])['native-type-check'], false);
});
test('native unsupported-language regressions execute the captured-source units and unchanged sibling contract', () => {
  const step = Object.values(workflow('native-type-check.yml').jobs)[0].steps
    .find(step => step.name === 'Verify native unresolved-language source binding');
  assert.equal(step.if, "${{ !cancelled() && inputs.compile-inputs-changed && steps.native-lint.outcome == 'success' }}");
  assert.deepEqual(step.env, {CARGO_BUILD_JOBS: '2', CARGO_PROFILE_TEST_OPT_LEVEL: '0'});
  assert.deepEqual(step.run.trim().split('\n'), [
    'regression_status=0',
    'cargo test --manifest-path rust/Cargo.toml --locked --lib solver_unknown_reasoning::tests:: -- --test-threads=1 || regression_status=$?',
    'cargo test --manifest-path rust/Cargo.toml --locked --test any_language a_language_without_localized_openers_reports_a_gap_not_english -- --exact --test-threads=1 || regression_status=$?',
    'exit "$regression_status"',
  ]);
  const target = read('rust/Cargo.toml').split('[[test]]').find(block => /^name = "any_language"$/mu.test(block));
  assert.match(target, /^path = "tests\/unit\/issue_706_any_language.rs"$/mu);
  const source = 'rust/tests/unit/solver_unknown_reasoning_captured_sources.rs';
  const fixture = relative(process.cwd(), resolve(dirname(source), /include_str!\(\s*"([^"]+)"/u.exec(read(source))[1]));
  for (const path of [source, 'rust/src/solver_unknown_reasoning.rs', 'rust/tests/unit/issue_706_any_language.rs', fixture]) {
    assert(readFileSync(path).length > 0, path);
    assert.equal(admission([path])['native-type-check'], true, path);
  }
  assert.equal(admission(['rust/tests/fixtures/issue-1138-b1/unrelated.json'])['native-type-check'], false);
});
// The production-only failure this job closes: PR #1497 was green, then main
// failed with E0308 in rust/src. Its own changed paths must select the check.
test('replays main run 43710854562: the E0308 pull request selects the native type check', () => {
  const replay = JSON.parse(readFileSync(new URL('../fixtures/issue-1191/native-type-check-e0308.json', import.meta.url), 'utf8'));
  const {observedFailure: failure} = replay;
  assert.equal(replay.schema, 'main-only-regression-replay/v1');
  assert.equal(failure.greenPullRequest.checks.failure, 0, 'every PR check passed before the fix');
  assert.equal(admission(replay.pullRequestPaths)['native-type-check'], true);
  assert.equal(failure.compilerOutput[0], 'error[E0308]: mismatched types');
  assert(failure.compilerOutput.includes('   --> src/solver_handlers/user_intent.rs:141:9'));
  assert(failure.compilerOutput.at(-1).startsWith('error: could not compile `formal-ai` (lib)'), 'the library target is what --lib checks');
  assert.equal(admission([replay.fixedBy.file])['native-type-check'], true);
  // Clippy stopped at E0308; once fixed, the same PR's lines failed clippy::perf.
  const lint = replay.followUpFailure;
  assert.equal(lint.gate, 'run_clippy');
  assert.equal(lint.clippyOutput.filter(line => line.startsWith('error: using `contains()`')).length, 2);
  assert(lint.clippyOutput.includes('    = note: `-D clippy::manual-contains` implied by `-D clippy::perf`'));
  for (const file of ['rust/src/formalization/proposition_structure.rs', 'rust/src/proof_engine/claim_binding.rs']) {
    assert(replay.pullRequestPaths.includes(file), file);
    assert.doesNotMatch(readFileSync(new URL('../../../' + file, import.meta.url), 'utf8'), /\.iter\(\)\s*\.any\(\|(joiner|cue)\| \*(joiner|cue) == /u);
  }
});
// The PR mode must lint exactly as main's run_clippy gate does, minus the test
// and example targets: same deny list, same deferred style categories.
test('the PR clippy mode runs main\'s lint flags on the library and binaries only', () => {
  const bin = mkdtempSync(join(tmpdir(), 'fake-cargo-'));
  try {
    writeFileSync(join(bin, 'cargo'), '#!/bin/sh\necho "cargo $*"\n', {mode: 0o755});
    const profile = mode => spawnSync('bash', ['scripts/ci-clippy-profile.sh', ...mode], {
      cwd: resolve(import.meta.dirname, '../../..'), encoding: 'utf8',
      env: {...process.env, PATH: bin + ':' + process.env.PATH, FORMAL_AI_STRICT_STYLE_LINTS: ''}});
    const main = profile([]), pull = profile(['--library-and-binaries']);
    assert.equal(main.status, 0, main.stderr); assert.equal(pull.status, 0, pull.stderr);
    const commands = run => run.stdout.split('\n').filter(line => line.startsWith('cargo '));
    const [mainClippy, mainExamples] = commands(main);
    assert.match(mainExamples, /^cargo check .*--examples/u);
    assert.deepEqual(commands(pull), [mainClippy.replace(' --lib --bins --tests ', ' --locked --lib --bins ')]);
    assert.match(mainClippy, / -- -D warnings -A clippy::pedantic .* -D clippy::perf$/u);
  } finally { rmSync(bin, {recursive: true, force: true}); }
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
// Issue #1569: one PR group outgrew its 360s budget (run 43693412224), so the
// fast inventory splits over two groups, and the step divides by the same count.
test('PRs split the fast JS inventory over two groups; full main inventory retains its measured shard plan', () => {
  const job=workflow('layered-ci.yml').jobs.js;
  assert.deepEqual(Array.from(evaluate(job.strategy.matrix.shard,'pull_request','refs/pull/1500/merge')),[1,2]);
  const shards=job.steps.find(s=>s.name==='Run this shard of the JavaScript suites').env.SHARDS;
  assert.equal(evaluate(shards,'pull_request','refs/pull/1500/merge'),2);
  assert.equal(evaluate(shards,'push','refs/heads/main'),8);
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

// Issue #1191: the fast lint job has no dependency install step, so every
// script it runs, and everything those scripts import, may load only Node
// built-ins and repository files. PR #1573's first push failed here when
// scripts/check-runner-labels.mjs (which imports `yaml`) was added to it.
test('the fast PR lint job runs only scripts that need no installed package', () => {
  const job = workflow('release.yml').jobs['fast-pr-lint'];
  assert.ok(!job.steps.some(step => /\b(bun|npm|pnpm|yarn) (install|ci)\b/u.test(step.run ?? '')), 'the fast lint job installs no packages');
  const entries = job.steps.flatMap(step => [...(step.run ?? '').matchAll(/\bnode (scripts\/[\w./-]+\.mjs)/gu)].map(match => match[1]));
  assert.ok(entries.length >= 5, 'the fast lint job runs its node scripts');
  // Only static imports load with the script; a lazy `import()` runs only on the path that needs it.
  const builtins = new Set(builtinModules);
  const seen = new Set(), bare = [];
  const visit = path => {
    if (seen.has(path)) return;
    seen.add(path);
    for (const match of read(path).matchAll(/^\s*(?:import|export)\b[^'"`;()=]*?\bfrom\s*['"]([^'"]+)['"]|^\s*import\s*['"]([^'"]+)['"]/gmu)) {
      const specifier = match[1] ?? match[2];
      if (specifier.startsWith('node:') || builtins.has(specifier)) continue;
      if (specifier.startsWith('.')) visit(join(path, '..', specifier).replaceAll('\\', '/'));
      else bare.push(`${path} imports ${specifier}`);
    }
  };
  entries.forEach(visit);
  assert.deepEqual(bare, [], 'scripts reachable from the fast lint job import installed packages');
});


test('native data-file regressions select every consumed source and data format', () => {
  for (const path of ['rust/tests/unit/data_files.rs', 'rust/tests/unit/data_files/technical_scalar.rs',
    'data/seed/meanings-calendar.lino', 'data/meta/technical-scalar-contracts.lino',
    'data/cache/wikidata/entity/Q146786.lino', 'data/cache/wikidata/entity/Q146786.json',
    'data/cache/wiktionary/en/example.json', 'data/parity/fixture.lino', 'data/seed/api-cache/fixture.bin'])
    assert.equal(admission([path])['native-type-check'], true, path);
  for (const path of ['data/benchmarks/fixture.json', 'data/meta/fixture.txt',
    'data/cache/fixture.txt', 'rust/tests/unit/unrelated.rs'])
    assert.equal(admission([path])['native-type-check'], false, path);
});


test('native scheduling regressions select every consumed source', () => {
  for (const path of ['rust/tests/unit/scheduling_contracts.rs',
    'rust/tests/unit/issue_1138_place_timezone.rs', 'rust/tests/unit/solver/issue_869_meeting_scheduling.rs',
    'rust/tests/unit/specification/spoken_hour_scheduling.rs',
    'rust/tests/unit/capability-routing/issue_1138_self_use_intent_routing.rs',
    'rust/tests/fixtures/issue-1580-calendar-clock.json', 'data/benchmarks/self-use-intent-routing.lino'])
    assert.equal(admission([path])['native-type-check'], true, path);
  for (const path of ['rust/tests/unit/unrelated.rs', 'rust/tests/fixtures/unrelated.json'])
    assert.equal(admission([path])['native-type-check'], false, path);
});


test('localized native replay selects both its test source and consumed fixture', () => {
  const paths = ['rust/tests/web/issue-1396-native.test.mjs',
    'rust/tests/fixtures/issue-1396/wiki-how-ru.json'];
  assert.deepEqual(paths.map(path => admission([path])['native-type-check']), [true, true]);
  for (const path of ['rust/tests/web/unrelated.test.mjs', 'rust/tests/fixtures/unrelated.json'])
    assert.equal(admission([path])['native-type-check'], false, path);
});


test('independent native replays continue after a regression fails and preserve failure status', () => {
  const job = workflow('native-type-check.yml').jobs.clippy;
  const later = job.steps.filter(step => step.name === 'Verify native unresolved-language source binding'
    || step.uses === 'actions/setup-node@v7' || step.name?.startsWith('Replay '));
  assert.equal(later.length, 5);
  const selected = (step, {earlierFailure = true, compile = true, lint = 'success', cancelled = false} = {}) => {
    const expression = step.if.slice(3, -2).trim()
      .replaceAll('inputs.compile-inputs-changed', "inputs['compile-inputs-changed']")
      .replaceAll('steps.native-lint', "steps['native-lint']");
    // Actions supplies success() unless an explicit status function is present.
    if (earlierFailure && !/\b(?:always|cancelled|failure|success)\s*\(/u.test(expression)) return false;
    return Boolean(new Script(expression).runInNewContext({
      inputs: {'compile-inputs-changed': compile}, steps: {'native-lint': {outcome: lint}},
      cancelled: () => cancelled,
    }, {timeout: 100}));
  };
  for (const step of later) {
    assert.equal(selected(step), true, step.name ?? step.uses);
    assert.equal(selected(step, {earlierFailure: false}), true);
    assert.equal(selected(step, {cancelled: true}), false);
    assert.equal(selected(step, {compile: false}), false);
    for (const lint of ['failure', 'skipped', 'cancelled'])
      assert.equal(selected(step, {lint}), false);
  }
  assert.equal(job.steps.find(step => step.name === 'Type-check and lint the library and binaries').id, 'native-lint');
  const isolated = job.steps.find(step => step.name === 'Verify isolated native behavior regressions');
  assert.match(isolated.run, /--no-fail-fast/u);
  assert.equal(isolated['continue-on-error'], undefined);
  for (const step of later) assert.equal(step['continue-on-error'], undefined);
  const binding = later.find(step => step.name === 'Verify native unresolved-language source binding');
  // Run the actual shell body against a finite stub: either failure must still
  // run its sibling and leave the step failed; two successes must succeed.
  const directory = mkdtempSync(join(tmpdir(), 'native-replay-failures-'));
  try {
    const stub = join(directory, 'cargo');
    writeFileSync(stub, '#!/bin/sh\nprintf "%s\\n" "$*" >> "$REPLAY_CALLS"\ncase "$*" in *" --lib "*) exit "$REPLAY_LIBRARY_STATUS";; *) exit "$REPLAY_SPANISH_STATUS";; esac\n', {mode: 0o755});
    for (const [library, spanish, expected] of [[1,0,1],[0,1,1],[0,0,0]]) {
      const calls = join(directory, `calls-${library}-${spanish}.txt`);
      const result = spawnSync('bash', ['-e', '-c', binding.run], {encoding:'utf8', timeout:5000,
        env:{...process.env, PATH:directory+':'+process.env.PATH, REPLAY_CALLS:calls,
          REPLAY_LIBRARY_STATUS:String(library), REPLAY_SPANISH_STATUS:String(spanish)}});
      assert.equal(result.status, expected, result.stderr);
      const commands = read(calls).trim().split('\n');
      assert.equal(commands.length, 2);
      assert.match(commands[0], /--lib solver_unknown_reasoning::tests::/u);
      assert.match(commands[1], /--test any_language/u);
    }
  } finally { rmSync(directory, {recursive:true, force:true}); }
});

test('the unchanged shell and download contracts execute in the focused native lane', () => {
  const step = Object.values(workflow('native-type-check.yml').jobs)[0].steps.find(step => step.name === 'Verify isolated native behavior regressions');
  assert.match(step.run, /--test operational_contracts(?: |$)/u);
  const manifest = read('rust/Cargo.toml');
  assert.match(manifest, /name = "operational_contracts"\npath = "tests\/unit\/operational_contracts[.]rs"/u);
  const source = read('rust/tests/unit/operational_contracts.rs');
  assert.match(source, /#\[path = "docs_requirements\/agent_cli_shell[.]rs"\]/u);
  assert.match(source, /#\[path = "ci-cd\/network_download_retry[.]rs"\]/u);
  for (const path of ['rust/tests/unit/operational_contracts.rs', 'rust/tests/unit/docs_requirements/agent_cli_shell.rs',
    'rust/tests/unit/ci-cd/network_download_retry.rs', 'README.md', 'docs/desktop/server-api.md',
    'scripts/install.sh', 'experiments/agentic_cli_matrix/install_client.sh', 'experiments/a/b.sh', 'new-download-script.sh'])
    assert.equal(admission([path])['native-type-check'], true, path);
  for (const path of ['docs/unrelated.md', 'experiments/unrelated.mjs'])
    assert.equal(admission([path])['native-type-check'], false, path);
});

test('native sparse checkout retains nested shell inputs without experiment payloads', {timeout:15000}, () => {
  const checkout = Object.values(workflow('native-type-check.yml').jobs)[0].steps.find(step => step.uses === 'actions/checkout@v7');
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-native-shell-inputs-'));
  const paths = ['README.md', 'scripts/install.sh', 'experiments/agentic_cli_matrix/install_client.sh',
    'experiments/issue-1571/osv-cargo-lock.sh', 'experiments/new/nested/download.sh', 'experiments/direct.sh',
    'experiments/new/nested/payload.json', 'experiments/unrelated.mjs', 'dev/log/payload.json'];
  const git = (args, input) => {
    const result = spawnSync('git', args, {cwd:directory, input, encoding:'utf8', timeout:5000, maxBuffer:65536});
    assert.equal(result.status, 0, result.stderr || String(result.error));
  };
  try {
    for (const path of paths) {
      mkdirSync(dirname(join(directory, path)), {recursive:true});
      writeFileSync(join(directory, path), 'finite checkout fixture\n');
    }
    git(['init', '-q']);
    git(['add', '.']);
    git(['-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', '-c', 'commit.gpgsign=false',
      'commit', '-qm', 'Finite source inputs']);
    assert.equal(checkout.with['sparse-checkout-cone-mode'], false);
    git(['sparse-checkout', 'set', '--no-cone', '--stdin'], checkout.with['sparse-checkout']);
    for (const path of paths.slice(0, 6)) assert(existsSync(join(directory, path)), path);
    for (const path of paths.slice(6)) assert(!existsSync(join(directory, path)), path);
  } finally { rmSync(directory, {recursive:true, force:true}); }
});


test('the complete reasoning-loop and interpretation contracts execute in the focused native lane', {timeout:15000}, () => {
  for (const path of ['rust/tests/unit/reasoning_contracts.rs',
    'rust/tests/unit/specification/reasoning_loop.rs', 'rust/tests/unit/specification/formalization.rs'])
    assert.equal(admission([path])['native-type-check'], true, path);
  for (const path of ['rust/tests/unit/specification/unrelated.rs', 'rust/tests/fixtures/unrelated.json'])
    assert.equal(admission([path])['native-type-check'], false, path);
  const step = Object.values(workflow('native-type-check.yml').jobs)[0].steps.find(step => step.name === 'Verify isolated native behavior regressions');
  assert.match(step.run, /--test reasoning_contracts(?: |$)/u);
  assert.match(step.run, /--no-fail-fast/u);
  assert.match(read('rust/Cargo.toml'), /name = "reasoning_contracts"\npath = "tests\/unit\/reasoning_contracts[.]rs"/u);
  const source = read('rust/tests/unit/reasoning_contracts.rs');
  for (const path of ['specification/reasoning_loop.rs', 'specification/formalization.rs'])
    assert(source.includes('#[path = "' + path + '"]'), path);
});


test('all checked-out shell downloads retain the native retry contract', {timeout:15000}, () => {
  const source = read('rust/tests/unit/ci-cd/network_download_retry.rs');
  const table = name => source.split('const ' + name + ':')[1].split('];')[0];
  const required = new Map([...table('DOWNLOADS_THAT_MUST_RETRY').matchAll(/\("([^"\n]+)", (\d+)\)/gu)]
    .map(([, path, count]) => [path, Number(count)]));
  const ownRetry = new Set([...table('FETCHES_WITH_THEIR_OWN_RETRY_LOOP').matchAll(/"((?:scripts|experiments)\/[^"\n]+[.]sh)"/gu)]
    .map(([, path]) => path));
  assert(required.size >= 4);
  assert(ownRetry.size >= 6);
  const invocations = path => read(path).replaceAll('\\\n', ' ').split('\n').map(line => line.trim())
    .filter(line => !line.startsWith('#') && !line.startsWith('echo') && line.includes('curl -'));
  for (const [path, count] of required) {
    const commands = invocations(path);
    assert.equal(commands.length, count, path + ' download count');
    for (const command of commands) {
      assert(command.includes('--retry ') && command.includes('--retry-all-errors'), path + ': ' + command);
      assert(command.includes('--retry-delay '), path + ': ' + command);
    }
  }
  const listing = spawnSync('git', ['ls-files', '-z', '--', '*.sh', ':!:dev/log/**'],
    {encoding:'utf8', timeout:5000, maxBuffer:2097152});
  assert.equal(listing.status, 0, listing.stderr || String(listing.error));
  let swept = 0;
  for (const path of listing.stdout.split('\0').filter(path => path && existsSync(path))) {
    for (const command of invocations(path)) {
      if (![' -o ', '| bash', '| sh', '| tar'].some(part => command.includes(part))
          || command.includes('127.0.0.1') || command.includes('localhost')) continue;
      swept++;
      assert(required.has(path) || ownRetry.has(path), path + ': unregistered remote download: ' + command);
    }
  }
  assert(swept >= 10, 'the sweep must retain the native minimum');
});


test('coreference and debug-session modules remain executable native inputs', {timeout:15000}, () => {
  for (const path of ['rust/tests/unit/specification/pronoun_topic_follow_up.rs',
    'rust/tests/unit/specification/debug_session.rs', 'rust/tests/unit/specification/event_source_origin.rs'])
    assert.equal(admission([path])['native-type-check'], true, path);
  for (const path of ['rust/tests/unit/specification/unrelated.rs', 'rust/tests/fixtures/unrelated.json'])
    assert.equal(admission([path])['native-type-check'], false, path);
  const step = Object.values(workflow('native-type-check.yml').jobs)[0].steps.find(step => step.name === 'Verify isolated native behavior regressions');
  assert.match(step.run, /--test reasoning_contracts(?: |$)/u);
  assert.match(step.run, /--no-fail-fast/u);
  const source = read('rust/tests/unit/reasoning_contracts.rs');
  for (const path of ['specification/pronoun_topic_follow_up.rs', 'specification/debug_session.rs',
    'specification/reasoning_loop.rs', 'specification/formalization.rs', 'specification/event_source_origin.rs'])
    assert(source.includes('#[path = "' + path + '"]'), path);
});


test('web search and fetched-page modules remain executable native inputs', {timeout:15000}, () => {
  for (const path of ['rust/tests/unit/web_requests.rs', 'rust/tests/unit/web_requests/extended.rs',
    'rust/tests/unit/page_formalization_route.rs', 'rust/tests/unit/seed/issue_1172_live_fact_answer.rs',
    'data/benchmarks/web-formalization/en.lino'])
    assert.equal(admission([path])['native-type-check'], true, path);
  assert.equal(admission(['rust/tests/unit/unrelated-source.rs'])['native-type-check'], false);
  const source = read('rust/tests/unit/reasoning_contracts.rs');
  // Preserve Rust's module directory for web_requests/extended.rs.
  assert.match(source, /^mod web_requests;/mu);
  assert(source.includes('#[path = "page_formalization_route.rs"]'));
  assert(source.includes('#[path = "seed/issue_1172_live_fact_answer.rs"]'));
  assert.match(read('rust/tests/unit/web_requests.rs'), /mod extended;/u);
  assert.match(read('rust/tests/unit/page_formalization_route.rs'), /include_str!\("\.\.\/\.\.\/\.\.\/data\/benchmarks\/web-formalization\/en\.lino"\)/u);
});


test('benchmark derivations retain their complete native input closure', {timeout:15000}, () => {
  for (const path of ['rust/tests/unit/specification/benchmarks.rs',
    'data/benchmarks/industry-suite.lino', 'data/benchmarks/LICENSES.md',
    'docs/case-studies/issue-244/raw-data/online-research.md'])
    assert.equal(admission([path])['native-type-check'], true, path);
  for (const path of ['rust/tests/unit/specification/unrelated.rs',
    'docs/case-studies/unrelated.md', 'data/benchmarks/unrelated.md'])
    assert.equal(admission([path])['native-type-check'], false, path);
  assert(read('rust/tests/unit/reasoning_contracts.rs')
    .includes('#[path = "specification/benchmarks.rs"]'));
  const step = Object.values(workflow('native-type-check.yml').jobs)[0].steps
    .find(step => step.name === 'Verify isolated native behavior regressions');
  assert.match(step.run, /--test reasoning_contracts(?: |$)/u);
  assert.match(step.run, /--no-fail-fast/u);
  assert.equal(step.env.FORMAL_AI_OFFLINE, '1');
});


test('external lookup fixture contracts execute in the focused native lane', {timeout:15000}, () => {
  const path = 'rust/tests/unit/issue_1173_fallback_executes_search.rs';
  assert.equal(admission([path])['native-type-check'], true, path);
  assert(read('rust/tests/unit/reasoning_contracts.rs')
    .includes('#[path = "issue_1173_fallback_executes_search.rs"]'));
  assert(read('rust/tests/unit/reasoning_contracts.rs').includes('mod external_search_fallback;'));
});
