// Production workflow target and consumer invariants complement actual receipt corruption tests.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,mkdtempSync,mkdirSync,writeFileSync,existsSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {pathToFileURL} from 'node:url';
import YAML from 'yaml';
import {parseSpeedPolicy} from '../../../scripts/lib/ci-speed-policy.mjs';
import {NATIVE_TARGETS} from '../../../scripts/native-release-artifact.mjs';
import {boundedCommandSync} from '../../../js/server/bounded-command.mjs';
import {hydrateSourceHistory} from '../../../scripts/native-release-source.mjs';
const text=readFileSync(new URL('../../../.github/workflows/desktop-release.yml',import.meta.url),'utf8');
function job(name){const at=text.indexOf('\n  '+name+':\n');assert.ok(at>=0,name);const tail=text.slice(at+1),end=tail.slice(1).search(/\n  [a-z][a-z-]*:\n/u);return (end<0?tail:tail.slice(0,end+1)).split('\n').filter(line=>!/^\s*#/u.test(line)).join('\n');}
test('one source job seals source and compiler before all eight distinct native producers',()=>{
 const source=job('native-source'),native=job('native');
 assert.match(source,/github\.event\.pull_request\.head\.sha/u);assert.match(source,/BASE_COMMIT: \$\{\{ needs\.base\.outputs\.commit \}\}/u);
 assert.equal((source.match(/simulate-fresh-merge\.sh/gu)??[]).length,1);
 assert.match(source,/native-release-source\.mjs"? seal/u);assert.match(source,/compiler-commit: /u);
 const targets=[...native.matchAll(/target: ([a-z0-9_-]+), binext:/gu)].map(m=>m[1]);
 assert.deepEqual(targets.sort(),Object.keys(NATIVE_TARGETS).sort());assert.equal(new Set(targets).size,8);
 assert.match(native,/timeout-minutes: \$\{\{ matrix\.capmin \}\}/u);assert.match(native,/toolchain: \$\{\{ needs\.native-source\.outputs\.toolchain \}\}/u);
 assert.match(native,/--release --bin formal-ai --locked --target/u);
 assert.match(native,/native-release-artifact\.mjs"? write/u);assert.match(native,/native-release-artifact\.mjs"? verify/u);
});
test('the six desktop and five CLI consumers only package exact verified target artifacts',()=>{
 for(const name of ['build','cli']){
  const body=job(name);assert.doesNotMatch(body,/cargo build|Install Rust toolchain|simulate-fresh-merge\.sh/u);
  assert.match(body,/name: native-release-\$\{\{ matrix\.target \}\}/u);
  assert.match(body,/NATIVE_SELECTION_SHA256: \$\{\{ needs\.native-source\.outputs\.selection-sha256 \}\}/u);
  assert.match(body,/native-release-artifact\.mjs"? verify/u);
 }
 assert.equal([...job('build').matchAll(/label: "/gu)].length,6);
 assert.equal([...job('cli').matchAll(/label: cli-/gu)].length,5);
 assert.match(job('build'),/FORMAL_AI_DESKTOP_BINARY: \$\{\{ env\.FORMAL_AI_VERIFIED_NATIVE_BINARY \}\}/u);
 assert.match(job('cli'),/cp "\$FORMAL_AI_VERIFIED_NATIVE_BINARY"/u);
 assert.match(job('cli'),/"\$binary" --version/u);
 assert.match(job('cli'),/native-release-artifact\.mjs"? verify-extracted/u);
});
test('publication guards and full consolidated asset verification remain on release consumers',()=>{
 for(const name of ['build','cli'])assert.match(job(name),/if: github\.event_name != 'pull_request'/u);
 assert.match(job('finalize'),/needs: \[resolve, native-source, build, cli, vscode\]/u);
 assert.match(text,/FORMAL_AI_DESKTOP_REQUIRE_BINARY: "true"/u);
 assert.match(text,/subject-path: formal-ai-cli-/u);
 assert.match(text,/scripts\/native-release-\*\.mjs/u);
});

test("VS Code and final provenance use the same exact source selection",()=>{
 const extension=job("vscode");assert.ok(extension.includes('native-release-source.mjs" import'));assert.ok(!extension.includes("simulate-fresh-merge.sh"));
 assert.ok(job("finalize").includes("NATIVE_SOURCE_COMMIT: ${{ needs.native-source.outputs.commit }}"));
});

test('each packaging job captures the pinned workflow protocol before selecting binary source',()=>{
 for(const name of ['native-source','native','build','cli','vscode']){
  const body=job(name);
  const steps=YAML.parse(text).jobs[name].steps;
  const protocol=steps.findIndex(step=>step.uses?.startsWith('actions/checkout@')&&step.with?.ref==='${{ github.workflow_sha }}');
  assert.ok(protocol>=0,'workflow-pinned protocol checkout required');
  assert.equal(steps[protocol].with['persist-credentials'],false);
  assert.equal(steps[protocol].with['fetch-depth'],1);
  assert.match(body,/NATIVE_PROTOCOL_COMMIT: \$\{\{ github\.workflow_sha \}\}/u);
  assert.match(body,/cp scripts\/native-release-source\.mjs scripts\/native-release-artifact\.mjs/u);
  const capture=steps.findIndex(step=>step.name?.startsWith('Capture release protocol'));
  const checkout=steps.findIndex((step,index)=>index>capture&&step.with?.ref==='${{ github.sha }}');
  if(name!=='native-source'){
   const restore=steps.findIndex(step=>step.run?.includes('native-release-source.mjs" import-offline'));
   const artifact=steps.findIndex(step=>step.uses?.startsWith('actions/download-artifact@')&&step.with?.name==='native-release-source');
   const history=steps.findIndex(step=>step.uses?.startsWith('actions/download-artifact@')&&step.with?.name==='native-release-source-history');
   assert.ok(capture>protocol&&artifact>capture&&history>artifact&&restore>history,'pinned protocol and same-run source artifacts precede offline source restoration');
   assert.equal(checkout,-1,name+' source uses authenticated complete history, without another GitHub checkout');
  }else assert.ok(capture>protocol&&checkout>capture,'pinned checkout and capture must precede event-pinned source checkout');
  assert.ok(body.includes('FORMAL_AI_NATIVE_PROTOCOL_DIR'));
 }
});

test('long Windows checkout paths are enabled before any native, desktop or CLI checkout',()=>{
 for(const name of ['native','build','cli']){
  const body=job(name),setup=body.indexOf('Enable long Windows checkout paths'),checkout=body.indexOf('uses: actions/checkout@');
  assert.ok(setup>=0&&setup<checkout,name+' must configure Git before checkout');
  const step=body.slice(setup,checkout);
  assert.match(step,/if: runner\.os == 'Windows'/u);
  assert.match(step,/run: git config --system core\.longpaths true/u);
 }
});

test('protocol capture is isolated from full source and survives unavailable promisor blobs',context=>{
 const workflow=YAML.parse(text),folders=['scripts','desktop/scripts'];
 assert.equal(workflow.env.GIT_HTTP_LOW_SPEED_LIMIT,'1024');
 assert.equal(workflow.env.GIT_HTTP_LOW_SPEED_TIME,'20');
 for(const name of ['native','build','cli','vscode']){
  const steps=workflow.jobs[name].steps;
  const index=steps.findIndex(step=>step.uses?.startsWith('actions/checkout@')&&step.with?.ref==='${{ github.workflow_sha }}');
  const protocol=steps[index],source=steps.slice(index+1).find(step=>step.uses?.startsWith('actions/checkout@'));
  assert.equal(protocol.with.filter,'blob:none',name);
  assert.equal(protocol.with.path,'.formal-ai-protocol-source',name);
  assert.deepEqual(protocol.with['sparse-checkout'].split('\n'),folders,name);
  assert.notEqual(protocol.with['sparse-checkout-cone-mode'],false,name);
  assert.equal(source,undefined,name+' source restoration uses the complete same-run bundle');
  const restored=steps.find(step=>step.run?.includes('native-release-source.mjs" import-offline'));
  assert.ok(restored);
  const bindings={...workflow.jobs[name].env,...restored.env};
  assert.equal(bindings.NATIVE_SELECTED_HEAD,'${{ needs.native-source.outputs.selected-head }}');
  assert.equal(bindings.NATIVE_SELECTION_SHA256,'${{ needs.native-source.outputs.selection-sha256 }}');
  const history=steps.find(step=>step.with?.name==='native-release-source-history');
  assert.equal(history?.with.path,'native-source');
  assert.ok(steps.indexOf(history)<steps.indexOf(restored));
  assert.ok(job('native-source').includes('export-history native-source'));
  const capture=steps.find(step=>step.name?.startsWith('Capture release protocol'));
  assert.equal(capture['working-directory'],protocol.with.path,name);
 }
 const root=mkdtempSync(join(tmpdir(),'formal-ai-protocol-sparse-'));
 context.after(()=>rmSync(root,{recursive:true,force:true}));
 const seed=join(root,'seed'),remote=join(root,'remote.git'),workspace=join(root,'workspace');
 const protocol=join(workspace,'.formal-ai-protocol-source'),captured=join(root,'captured');
 const files=['scripts/protocol.mjs','desktop/scripts/prepare.mjs','rust/src/main.rs','data/seed/fixture.lino'];
 // Distinct blobs leave omitted source unavailable in the filtered clone.
 const contents=new Map(files.map(file=>[file,'fixture '+file+'\n']));
 for(const [file,content]of contents){mkdirSync(dirname(join(seed,file)),{recursive:true});writeFileSync(join(seed,file),content);}
 const git=(args,cwd=seed,status=0)=>{
  const result=boundedCommandSync('git',['-c','core.hooksPath=/dev/null','-c','commit.gpgsign=false',...args],{cwd,timeout:5000,maxBuffer:1048576});
  assert.equal(result.error,undefined,String(result.error));assert.equal(result.cleanup_complete,true);
  assert.equal(result.status,status,result.stderr.toString());
  return result.stdout.toString().trim();
 };
 git(['init','--quiet']);git(['add','--',...files]);
 git(['-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','--quiet','-m','fixture']);
 const primary=git(['branch','--show-current']);
 writeFileSync(join(seed,files[3]),'updated selected source\n');contents.set(files[3],'updated selected source\n');
 git(['add','--',files[3]]);git(['-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','--quiet','-m','selected ancestry']);
 git(['switch','--quiet','-c','unrelated']);
 writeFileSync(join(seed,'unrelated.txt'),'irrelevant branch payload\n');git(['add','--','unrelated.txt']);
 git(['-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','--quiet','-m','unrelated']);
 const unrelated=git(['rev-parse','HEAD']);git(['switch','--quiet',primary]);
 git(['clone','--bare','--quiet',seed,remote]);
 git(['config','uploadpack.allowFilter','true'],remote);
 const url=pathToFileURL(remote).href;
 git(['clone','--no-checkout','--depth','1','--filter=blob:none',url,protocol]);
 git(['sparse-checkout','set','--cone',...folders],protocol);
 git(['checkout','--force','HEAD'],protocol);
 assert.equal(git(['config','--get','remote.origin.promisor'],protocol),'true');
 assert.ok(git(['rev-list','--objects','--all','--missing=print'],protocol).split('\n').some(line=>line.startsWith('?')));
 for(const file of files.slice(2))assert.equal(existsSync(join(protocol,file)),false);
 mkdirSync(captured);
 for(const [index,file]of files.slice(0,2).entries())writeFileSync(join(captured,String(index)),readFileSync(join(protocol,file)));
 git(['remote','set-url','origin',join(root,'unavailable.git')],protocol);
 // Reusing the sparse repository reproduces the CI failure before checkout.
 git(['sparse-checkout','disable'],protocol,128);
 // An independent source bootstrap never reuses the filtered protocol repository.
 rmSync(workspace,{recursive:true,force:true});
 git(['clone','--quiet','--depth','1','--no-tags','--branch',primary,url,workspace]);
 assert.equal(git(['rev-parse','--is-shallow-repository'],workspace),'true');
 const hydrated=hydrateSourceHistory(workspace);assert.equal(hydrated.completedHistory,true);
 assert.equal(git(['rev-parse','--is-shallow-repository'],workspace),'false');
 git(['cat-file','-e',unrelated+'^{commit}'],workspace,128);
 git(['remote','set-url','origin',join(root,'unavailable.git')],workspace);
 git(['reset','--hard','HEAD'],workspace);
 assert.equal(git(['config','--get','remote.origin.promisor'],workspace,1),'');
 assert.equal(git(['config','--get','core.sparseCheckout'],workspace,1),'');
 for(const [file,content]of contents)assert.equal(readFileSync(join(workspace,file),'utf8'),content);
 for(const [index,file]of files.slice(0,2).entries())assert.equal(readFileSync(join(captured,String(index)),'utf8'),contents.get(file));
});

test('only Intel native carries explicit cold-build debt and a pinned process-group deadline',()=>{
 const native=YAML.parse(text).jobs.native,rows=native.strategy.matrix.include;
 assert.equal(rows.length,8);
 assert.deepEqual(Object.fromEntries(rows.map(row=>[row.target,row.capmin])),Object.fromEntries(Object.keys(NATIVE_TARGETS).map(target=>[target,target==='x86_64-apple-darwin'?45:30])));
 const capture=native.steps.find(step=>step.name==='Capture release protocol before selecting binary source').run;
 assert.ok(capture.includes('cp scripts/native-release-evidence.mjs scripts/build-selected-wasm.mjs scripts/run-with-budget-warning.sh "$protocol_dir/"'));
 assert.ok(capture.includes("const files=['build-selected-wasm.mjs'"));
 assert.ok(capture.includes("native-build-budget-protocol.json"));
 assert.ok(capture.includes("sha256:createHash('sha256').update(budgetBytes).digest('hex')"));
 const build=native.steps.find(step=>step.name==='Build the pinned default-feature native executable').run;
 assert.ok(build.includes('if [ "${{ matrix.target }}" = x86_64-apple-darwin ]; then'));
 assert.ok(build.includes('prefix=()'));assert.ok(build.includes('"${prefix[@]}" cargo build --manifest-path rust/Cargo.toml --target-dir target --release --bin formal-ai --locked --target ${{ matrix.target }}'));
 assert.ok(build.includes('TEST_BUDGET_ENFORCE=true TEST_BUDGET_GRACE_SECONDS=5 bash "$FORMAL_AI_NATIVE_PROTOCOL_DIR/run-with-budget-warning.sh" 2100'));
 const policy=parseSpeedPolicy(readFileSync(new URL('../../../data/meta/ci-speed.lino',import.meta.url),'utf8'));
 const exceptions=policy.exceptions.filter(entry=>entry.workflow==='.github/workflows/desktop-release.yml');
 assert.equal(exceptions.length,1);assert.equal(exceptions[0].job,'native');assert.equal(exceptions[0].timeoutMinutes,45);
 assert.ok(exceptions[0].reason.includes('decodedLogSHA5228190b36a1b72ccd6c7c91d1f9f08c3d7c191f8c08bcfa66cb9fe0ffc5012c'));assert.ok(exceptions[0].reason.includes('restore30'));assert.equal(policy.limitMinutes,30);
});


test('PR artifact consumers do not retry missing native outputs after a failed producer',()=>{
 const workflow=YAML.parse(text);
 for(const name of ['build','cli']){
  assert(workflow.jobs[name].needs.includes('native'));
  assert(workflow.jobs[name].if.includes("needs.native-source.result == 'success'"));
  assert(workflow.jobs[name].if.includes("(github.event_name != 'pull_request' || needs.native.result == 'success')"));
 }
 assert(workflow.jobs.vscode.if.includes("needs.native-source.result == 'success'"));
 assert(!workflow.jobs.vscode.needs.includes('native'),'VSIX remains an independent source consumer');
});
