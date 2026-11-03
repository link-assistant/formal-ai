import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,existsSync,mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {Script} from 'node:vm';
import YAML from 'yaml';
import {selectFastTests} from '../../../scripts/list-fast-pull-request-tests.mjs';
import {fileLimit} from '../../../scripts/check-file-size.mjs';
import {ROOTS,familyOf} from '../../../scripts/check-readable-code.mjs';
const read = file => readFileSync(file, 'utf8');
const manifest = JSON.parse(read('data/benchmarks/fast-pull-request-retained-jobs.json'));
const workflow = file => YAML.parse(read('.github/workflows/' + file));
const evaluate = (expression, context) => new Script(expression.trim()
  .replace(/^\$\{\{\s*/u, '').replace(/\s*\}\}$/u, '')
  .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu, (_, key) => '[' + JSON.stringify(key) + ']'))
  .runInNewContext({cancelled: () => false, always: () => true, ...context}, {timeout: 100});
function context(event, ref = 'refs/pull/1500/merge', enabled = false) {
  const outputs = Object.fromEntries(['any-code-changed','rs-changed','toml-changed','pipeline-changed','rust-changed','js-changed','ts-changed'].map(k => [k,'true']));
  return {github: {event_name:event,ref}, vars:{FORMAL_AI_FULL_PR_CI:enabled?'run':'skip',FORMAL_AI_NON_LINUX_CI:'run',FORMAL_AI_FULL_INTEGRATION:'run',FORMAL_AI_FULL_LINK_CHECKS:'run',FORMAL_AI_FULL_CODING_LADDER:'run'},
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
  const check = (step, minutes) => {
    assert.equal(step.uses, 'actions/checkout@v7');
    assert.equal(step['timeout-minutes'], minutes);
    assert.deepEqual(step.env, {GIT_HTTP_LOW_SPEED_LIMIT:'1024', GIT_HTTP_LOW_SPEED_TIME:'20'});
    assert.equal(step.with['persist-credentials'], false);
    assert.equal(step.with['fetch-depth'], 1);
    assert.equal(step.with.filter, 'blob:none');
    assert.equal(step.with['sparse-checkout-cone-mode'], false);
    assert.equal(step.with.ref, undefined, 'keep actual event source selection');
  };
  for (const [job, minutes] of [[version,2],[javascript,3]]) {
    const checkout = job.steps[0];
    check(checkout,minutes);
    for (const mutate of [step=>step['timeout-minutes']=10,
      step=>step.with['fetch-depth']=0, step=>step.with['persist-credentials']=true,
      step=>step.env.GIT_HTTP_LOW_SPEED_TIME='300']) {
      const changed = structuredClone(checkout); mutate(changed);
      assert.throws(()=>check(changed,minutes));
    }
  }
  assert.equal(version.steps[0].with['sparse-checkout'],
    '/scripts/ci-pull-request-admission.mjs\n/scripts/lib/documentation-paths.mjs\n/Cargo.toml\n/rust/Cargo.toml\n');
  assert.equal(javascript.steps[0].with['sparse-checkout'],
    '/*\n!/experiments/formal_ai_subagent/evidence/coordinator-1188/\n!/dev/log/\n/dev/log/issues/702/pulls/818/agent-cli/\n');
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


test('JavaScript checkout keeps source and the real learning fixture while omitting historical log payloads', () => {
  const patterns=workflow('layered-ci.yml').jobs.js.steps[0].with['sparse-checkout'];
  const fixture='dev/log/issues/702/pulls/818/agent-cli/context-hierarchy-learning-report.lino';
  assert.match(read('rust/tests/web/issue-0702-nested-contexts.test.mjs'),new RegExp(fixture.replaceAll('.', '[.]'),'u'));
  assert.ok(read(fixture).length>0,'the unchanged real report remains an input');
  const retained=['package.json','bun.lock','js/agentic/planner.mjs','ts/agentic/planner.mts',
    'scripts/plan-test-shards.mjs','data/meta/javascript-test-durations.lino',
    'js/例 子.mjs','docs/line\nbreak.md',
    'rust/tests/web/issue-0702-nested-contexts.test.mjs','rust/src/lib.rs',
    '.github/actions/setup-source-model/action.yml','docs/case-studies/issue-717/e2e-npm-ci.log',fixture];
  const omitted=['dev/log/issues/781/pulls/803/sessions/old/solve.log',
    'experiments/formal_ai_subagent/evidence/coordinator-1188/raw/archive.jsonl.gz'];
  assert.deepEqual(checkoutSelection(patterns,[...retained,...omitted]),retained);
});


function checkoutSelection(patterns,files) {
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-checkout-rules-'));
  try {
    const file=join(directory,'patterns');writeFileSync(file,patterns);
    const selected=execFileSync('git',['sparse-checkout','check-rules','--no-cone','--rules-file',file,'-z'],{
      input:files.join('\0')+'\0',encoding:'utf8',timeout:5000,maxBuffer:16*1024*1024
    }).split('\0').filter(Boolean);
    return selected;
  } finally {rmSync(directory,{recursive:true,force:true});}
}


test('fast source checkout retains every measured tracked source while excluding only unmeasured archives', () => {
  const job=workflow('release.yml').jobs['fast-pr-lint'],step=job.steps[0];
  assert.equal(job['timeout-minutes'],5);assert.equal(step['timeout-minutes'],4);
  assert.equal(step.with.ref,undefined);assert.equal(step.with['persist-credentials'],false);
  assert.equal(step.with['fetch-depth'],1);assert.equal(step.with.filter,'blob:none');
  assert.equal(step.with['sparse-checkout-cone-mode'],false);
  const files=execFileSync('git',['ls-files','--cached','-z'],{
    encoding:'utf8',timeout:5000,maxBuffer:16*1024*1024
  }).split('\0').filter(Boolean);
  const retained=new Set(checkoutSelection(step.with['sparse-checkout'],files));
  const omitted=files.filter(file=>!retained.has(file));assert.ok(omitted.length>0);
  for(const file of omitted){
    if(file.startsWith('dev/log/'))continue;
    assert.match(file,/^experiments\/formal_ai_subagent\/evidence\/coordinator-1188\/.*[.]gz$/u);
    assert.equal(fileLimit(file),null,'do not omit measured content: '+file);
  }
  for(const file of files){
    if(file.startsWith('dev/log/'))continue;
    const readable=ROOTS.some(root=>file.startsWith(root+'/'))&&familyOf(file);
    if(readable||fileLimit(file)!==null)assert.ok(retained.has(file),'missing source input '+file);
  }
});


test('Cargo security acquisition retains the full crate and audit proofs without unrelated evidence', () => {
  const job=workflow('security.yml').jobs['cargo-audit'];
  const patterns='/.cargo/\n/rust/\n/scripts/check-rust-dependencies.sh\n/Cargo.toml\n/Cargo.lock\n/README.md\n/LICENSE\n';
  function check(value){
    assert.equal(value['timeout-minutes'],10);assert.equal(value.steps.length,4);
    const step=value.steps[0];assert.equal(step.uses,'actions/checkout@v7');assert.equal(step['timeout-minutes'],2);
    assert.deepEqual(step.env,{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'});
    assert.equal(step.with.ref,undefined);
    assert.deepEqual(step.with,{'persist-credentials':false,'fetch-depth':1,filter:'blob:none',
      'sparse-checkout':patterns,'sparse-checkout-cone-mode':false});
    assert.deepEqual(value.steps.slice(1),[
      {name:'Setup Rust',uses:'dtolnay/rust-toolchain@stable'},
      {uses:'taiki-e/install-action@v2',with:{tool:'cargo-audit@0.22.2'}},
      {name:'Audit committed Cargo.lock',run:'bash scripts/check-rust-dependencies.sh'}
    ]);
  }
  check(job);
  for(const mutate of [value=>value['timeout-minutes']=11,value=>value.steps[0]['timeout-minutes']=3,
    value=>value.steps[0].with.ref='refs/heads/foreign',value=>value.steps[0].with['persist-credentials']=true,
    value=>value.steps[0].with.filter='tree:0',value=>value.steps[0].with['sparse-checkout-cone-mode']=true,
    value=>value.steps[0].with['sparse-checkout']=patterns.replace('/rust/\n',''),
    value=>value.steps[0].env.GIT_HTTP_LOW_SPEED_TIME='300',value=>value.steps.pop()]){
    const changed=structuredClone(job);mutate(changed);assert.throws(()=>check(changed));
  }
  const tracked=execFileSync('git',['ls-files','--cached','-z','--','rust','.cargo'],{
    encoding:'utf8',timeout:5000,maxBuffer:4*1024*1024
  }).split('\0').filter(Boolean);
  assert.ok(tracked.length>0);assert.deepEqual(checkoutSelection(patterns,tracked),tracked);
  const retained=['rust/Cargo.toml','rust/Cargo.lock','rust/build.rs','rust/src/lib.rs','rust/src/main.rs',
    'rust/tests/unit/mod.rs','rust/tests/integration/mod.rs','rust/tests/source/mod.rs',
    '.cargo/audit.toml','scripts/check-rust-dependencies.sh','Cargo.toml','Cargo.lock','README.md','LICENSE'];
  const omitted=['dev/log/issues/1079/pulls/1080/upstream-reports/evidence.log',
    'experiments/formal_ai_subagent/evidence/coordinator-1188/trace.gz','docs/case-studies/history.log'];
  assert.deepEqual(checkoutSelection(patterns,[...retained,...omitted]),retained);
  for(const path of retained.filter(path=>!['Cargo.toml','Cargo.lock'].includes(path)))assert.ok(existsSync(path),path);
  const source=read('scripts/check-rust-dependencies.sh');
  for(const marker of ['config=".cargo/audit.toml"','lock="rust/Cargo.lock"','if [[ ! -f "$config" ]]',
    '[[ -n "$spec" && -n "$blocked_spec" ]]','[[ -z "$spec" && -z "$blocked_spec" ]]',
    'blocked-upstream','if [[ "$status" -ne 0 ]]','cargo audit --file "$lock" --deny warnings'])
    assert.ok(source.includes(marker),'audit proof guard lost: '+marker);
  assert.equal((source.match(/cargo tree --manifest-path rust\/Cargo[.]toml --locked --target all --all-features --edges all --invert/gu)??[]).length,2);
  assert.match(read('.cargo/audit.toml'),/^\[advisories\]/mu);
});


test('live link integration is explicit main-only while parser regressions stay mandatory and bounded', async () => {
  const document=workflow('links.yml'),live=document.jobs['link-checker'],parser=document.jobs['link-parser-checks'];
  assert.ok(manifest.jobs.some(row=>row.file==='links.yml'&&row.id==='link-checker'),'original live payload remains a retained job');
  const allowed=(event,ref='refs/heads/main',requested='',liveAudit=false)=>{
    const value=context(event,ref);value.vars.FORMAL_AI_FULL_LINK_CHECKS=requested;value.inputs['live-audit']=liveAudit;
    return Boolean(evaluate(live.if,value));
  };
  for(const event of ['push','schedule']){assert.equal(allowed(event),false);assert.equal(allowed(event,'refs/heads/main','run'),true);}
  assert.equal(allowed('workflow_dispatch'),false);
  assert.equal(allowed('workflow_dispatch','refs/heads/main','',true),true);
  assert.equal(allowed('workflow_dispatch','refs/heads/main','run'),true);
  const liveInput=document.on.workflow_dispatch.inputs['live-audit'];
  assert.equal(liveInput.type,'boolean');assert.equal(liveInput.default,false);assert.equal(liveInput.required??false,false);
  for(const event of ['pull_request','pull_request_target']){
    assert.equal(allowed(event,'refs/pull/1/merge','run',true),false);assert.equal(allowed(event,'refs/heads/main','run',true),false);
  }
  assert.equal(allowed('workflow_dispatch','refs/heads/fix/example','run',true),false);
  assert.equal(allowed('push','refs/heads/main','yes'),false);
  assert.equal(live['timeout-minutes'],10);
  assert.deepEqual(document.permissions,{contents:'read'});
  assert.equal(parser.if,undefined);assert.equal(parser.needs,undefined);assert.equal(parser['timeout-minutes'],3);
  assert.deepEqual(parser.permissions??document.permissions,{contents:'read'});assert.equal(parser.steps.length,2);
  const checkout=parser.steps[0],patterns='/scripts/\n/package.json\n/.lycheeignore\n';
  assert.equal(checkout.uses,'actions/checkout@v7');assert.equal(checkout['timeout-minutes'],2);
  assert.deepEqual(checkout.env,{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'});
  assert.equal(checkout.with.ref,undefined);
  assert.deepEqual(checkout.with,{'persist-credentials':false,'fetch-depth':1,filter:'blob:none',
    'sparse-checkout':patterns,'sparse-checkout-cone-mode':false});
  const originalParser={"name":"Test Web Archive report parser","run":"node --test \\\n  scripts/check-web-archive.test.mjs \\\n  scripts/recheck-broken-links.test.mjs \\\n  scripts/recheck-transient-http.test.mjs\n"};
  assert.deepEqual(parser.steps[1],originalParser);
  assert.deepEqual(live.steps.find(step=>step.name===originalParser.name),originalParser,'live opt-in retains the exact same parser command');
  const entries=[...parser.steps[1].run.matchAll(/scripts\/[A-Za-z0-9_.-]+[.]test[.]mjs/gu)].map(match=>match[0]);
  assert.deepEqual(entries,['scripts/check-web-archive.test.mjs','scripts/recheck-broken-links.test.mjs','scripts/recheck-transient-http.test.mjs']);
  // Finite reviewed module/literal-URL closure; runtime scratch files are independently created by parser tests.
  const {posix}=await import('node:path'),pending=[...entries],visited=new Set(),literalInputs=new Set();
  const covered=file=>file.startsWith('scripts/')||file==='package.json'||file==='.lycheeignore';
  while(pending.length){const file=pending.pop();if(visited.has(file))continue;assert.ok(covered(file),'parser module omitted: '+file);visited.add(file);
    const source=read(file);
    const staticImports=[...source.matchAll(/\bfrom\s+['"]([^'"]+)['"]/gu)].map(match=>match[1]);
    const dynamicImports=[...source.matchAll(/\bimport\s*\(\s*(['"])([^'"]+)\1\s*\)/gu)].map(match=>match[2]);
    assert.equal((source.match(/\bimport\s*\(/gu)??[]).length,dynamicImports.length,'review nonliteral dynamic source import before narrowing acquisition');
    for(const specifier of [...staticImports,...dynamicImports]){
      if(specifier.startsWith('node:')||['fs','url','path'].includes(specifier))continue;
      assert.ok(specifier.startsWith('.'),'unreviewed parser dependency: '+specifier);
      const next=posix.normalize(posix.join(posix.dirname(file),specifier));assert.ok(!next.startsWith('../'));pending.push(next);
    }
    for(const match of source.matchAll(/new URL\(\s*['"]([^'"]+)['"]\s*,\s*import[.]meta[.]url\s*\)/gu)){
      const target=posix.normalize(posix.join(posix.dirname(file),match[1]));assert.ok(covered(target),'parser literal input omitted: '+target);literalInputs.add(target);
    }
  }
  assert.deepEqual([...visited].sort(),['scripts/check-web-archive.mjs','scripts/check-web-archive.test.mjs',
    'scripts/recheck-broken-links.mjs','scripts/recheck-broken-links.test.mjs','scripts/recheck-transient-http.test.mjs']);
  assert.ok(literalInputs.has('.lycheeignore'));assert.ok(read('.lycheeignore').length>0);
  const retained=[...visited,'package.json','.lycheeignore'];
  assert.deepEqual(checkoutSelection(patterns,[...retained,'rust/src/lib.rs','docs/history.log',
    'experiments/formal_ai_subagent/evidence/coordinator-1188/trace.gz']),retained);
});

test('secrets scanning retains full history and every current file within separate finite phases', () => {
  const document=workflow('release.yml'),job=document.jobs['secrets-scan'];
  const verify=value=>{
    assert.equal(value.name,'Secrets Scan');assert.equal(value['runs-on'],'ubuntu-latest');
    assert.equal(value['timeout-minutes'],15);assert.deepEqual(value.needs,['detect-changes']);
    assert.equal(value.steps.length,3);assert.equal(value.permissions,undefined);
    const checkout=value.steps[0];
    assert.equal(checkout.uses,'actions/checkout@v7');assert.equal(checkout['timeout-minutes'],10);
    assert.equal(value['timeout-minutes']-checkout['timeout-minutes'],5);
    assert.deepEqual(checkout.env,{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'});
    assert.deepEqual(checkout.with,{'persist-credentials':false,'fetch-depth':0,filter:'blob:none'});
    assert.equal(checkout.with.ref,undefined);assert.equal(checkout.with['sparse-checkout'],undefined);
    assert.equal(checkout.with['sparse-checkout-cone-mode'],undefined);
    assert.deepEqual(value.steps.slice(1),[
      {name:'Setup Node',uses:'actions/setup-node@v7',with:{'node-version':22}},
      {name:'Check for secrets',env:{BASE_REF:"${{ github.base_ref || github.event.before || 'HEAD~1' }}"},run:'bash scripts/check-secrets.sh'}
    ]);
  };
  verify(job);
  assert.deepEqual(document.permissions,{"contents":"read"});
  assert.deepEqual(job.concurrency,{"group":"${{ github.workflow }}-${{ github.ref }}-secrets-scan","cancel-in-progress":"${{ github.ref != 'refs/heads/main' }}"});
  assert.equal(job.if.trim().replace(/\s+/gu,' '),"!cancelled() && ( github.event_name == 'workflow_dispatch' || needs.detect-changes.outputs.any-code-changed == 'true' || needs.detect-changes.outputs.docs-changed == 'true' || needs.detect-changes.outputs.pipeline-changed == 'true' )");
  for(const event of ['pull_request','push','workflow_dispatch']) {
    assert.equal(Boolean(evaluate(job.if,context(event))),true);
  }
  assert.equal(Boolean(evaluate(job.if,{...context('pull_request'),cancelled:()=>true})),false);
  const scanner=read('scripts/check-secrets.sh');
  for(const marker of ['git merge-base "$base" HEAD',
    'git diff --name-only --diff-filter=ACMRT -z "$merge_base" HEAD',
    'git ls-files -z | exclude','SECRETLINT_VERSION:-13.0.5',
    '-p "secretlint@${SECRETLINT_VERSION}"','-p "@secretlint/secretlint-rule-preset-recommend@${SECRETLINT_VERSION}"'])
    assert.ok(scanner.includes(marker),'full reviewed secret selection/guard lost: '+marker);
  for(const mutate of [
    value=>value.steps[0].with['fetch-depth']=1,
    value=>value.steps[0].with['sparse-checkout']='/*\n!/some-source/\n',
    value=>value.steps[0].with.ref='main',
    value=>value.steps[0]['timeout-minutes']=15,
    value=>value['timeout-minutes']=10,
    value=>value.steps[0].with['persist-credentials']=true,
    value=>value.steps[2].env.BASE_REF='HEAD~1'
  ]) {
    const changed=structuredClone(job);mutate(changed);assert.throws(()=>verify(changed));
  }
});


test('complete Coding Ladder is explicit main-only with mandatory default consistency and honest deferral', () => {
  const document=workflow('coding-ladder.yml'),ids=['ladder-binary','ladder-shard','coding-ladder'];
  assert.deepEqual(document.on.workflow_dispatch.inputs['full-ladder'],{description:'Run the complete 130-task authoring measurement on main',required:false,default:false,type:'boolean'});
  const selected=(id,event,ref='refs/heads/main',requested='',fullLadder=false)=>{
    const value=context(event,ref);value.vars.FORMAL_AI_FULL_CODING_LADDER=requested;value.inputs['full-ladder']=fullLadder;
    return Boolean(evaluate(document.jobs[id].if,value));
  };
  for(const id of ids){
    assert(manifest.jobs.some(row=>row.file==='coding-ladder.yml'&&row.id===id),'complete original payload is retained: '+id);
    for(const event of ['push','schedule']){assert.equal(selected(id,event),false);assert.equal(selected(id,event,'refs/heads/main','run'),true);}
    assert.equal(selected(id,'workflow_dispatch'),false,'generic checks dispatch does not select authoring');
    assert.equal(selected(id,'workflow_dispatch','refs/heads/main','',true),true);
    for(const event of ['pull_request','pull_request_target'])for(const ref of ['refs/heads/main','refs/pull/1/merge'])assert.equal(selected(id,event,ref,'run',true),false);
    for(const event of ['push','workflow_dispatch'])assert.equal(selected(id,event,'refs/heads/feature','run',true),false);
  }
  assert.deepEqual(document.jobs['ladder-shard'].strategy.matrix.shard,[1,2,3,4,5]);
  assert.equal(document.jobs['ladder-shard']['timeout-minutes'],30);
  const measurement=document.jobs['ladder-shard'].steps.find(step=>step.name==='Measure one shard of the 130 coding tasks');
  assert.equal(measurement.env.TEST_BUDGET_SECONDS,1200);
  const scope=document.jobs['ladder-scope'];assert.equal(scope['timeout-minutes'],1);assert.equal(scope.steps.length,1);
  assert.equal(scope.steps.some(step=>step.uses||/cargo|rust-script|run_coding_ladder[.]sh/u.test(step.run??'')),false);
  assert.match(scope.steps[0].run,/no authoring measurement, benchmark score or acceptance credit is claimed/u);
  const gate=read('data/meta/ci-gates/coding-ladder.lino');
  assert.match(gate,/stage rust/u);assert(gate.includes('rust-script --test scripts/check-coding-ladder.rs && rust-script scripts/check-coding-ladder.rs --base origin/main'));
  assert(read('.github/workflows/release.yml').includes('rust-script scripts/run-ci-gates.rs --stage rust'),'ordinary main lint retains the mandatory consistency gate');
  for(const [file,expected] of Object.entries({"experiments/issue_847_coding_ladder/run_coding_ladder.sh":"64f9fa51b7809b7ce6bde6ecb789e84324a5911fd2cf37130eae276e47feebd4","experiments/issue_847_coding_ladder/prompts.json":"d06dc4bd0d1eb17133f407be92a1b6f0ca2323a91e31047261240c734b2f3f0d","experiments/issue_847_coding_ladder/results.json":"450255f7d1d97083f365d66a92b3d5e9a96a10425de456ea0baf066ee0598230","data/meta/ladder-ratchet.lino":"58bc9c284d95115011c187c2f1bacba316a8fee026b4cfe0ef3f46c5981e6c4d","scripts/check-coding-ladder.rs":"42765a4a1ca3f8c2c69dbca408b4a255670f67f305a7b179009b8e52651f67f6","data/meta/ci-gates/coding-ladder.lino":"e76930c5c87a4ffaa9475118ae089bbae13850310f338af5c18b335a6feaa04d"}))assert.equal(createHash('sha256').update(readFileSync(file)).digest('hex'),expected,'retained original benchmark/ratchet '+file);
});
