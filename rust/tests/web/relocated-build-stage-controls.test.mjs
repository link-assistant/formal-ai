// Structural source controls and refusals; caps are declarations, never cold runtime proof.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {createRequire} from 'node:module';
const root=process.env.RELOCATED_CONTROL_REPO_ROOT??resolve(import.meta.dirname,'../../..');
const {parse}=createRequire(join(root,'package.json'))('yaml');
const desktop=parse(readFileSync(join(root,'.github/workflows/desktop-release.yml'),'utf8'));
const coverage=parse(readFileSync(join(root,'.github/workflows/coverage.yml'),'utf8'));
const nativeTargets=['x86_64-unknown-linux-gnu','aarch64-unknown-linux-gnu','x86_64-unknown-linux-musl','aarch64-unknown-linux-musl',
 'x86_64-apple-darwin','aarch64-apple-darwin','x86_64-pc-windows-msvc','aarch64-pc-windows-msvc'];
const packageLabels=['linux-x64','linux-arm64','macos-x64','macos-arm64','windows-x64','windows-arm64'];
const step=(job,name)=>{const matches=job.steps.filter(value=>value.name===name);assert.equal(matches.length,1,'unique operation '+name);return matches[0];};
const index=(job,name)=>job.steps.indexOf(step(job,name));
function order(job,names){const indices=names.map(name=>index(job,name));assert.ok(indices.slice(1).every((value,i)=>value>indices[i]),names.join(' -> '));}
function cap(job){const declared=job['timeout-minutes'];if(typeof declared==='number')assert.ok(declared>0&&declared<=30);else{
 assert.equal(declared,'${{ matrix.capmin }}');assert.ok(job.strategy.matrix.include.length>0);
 for(const row of job.strategy.matrix.include)assert.ok(Number.isInteger(row.capmin)&&row.capmin>0&&row.capmin<=30);
}}
function upload(job,name){const matches=job.steps.filter(value=>String(value.uses).startsWith('actions/upload-artifact@')&&value.with?.name===name);
 assert.equal(matches.length,1,'unique artifact upload '+name);assert.equal(matches[0].with['if-no-files-found'],'error');return matches[0];}
function download(job,name){const matches=job.steps.filter(value=>String(value.uses).startsWith('actions/download-artifact@')&&value.with?.name===name);
 assert.equal(matches.length,1,'unique artifact download '+name);return matches[0];}
function checkDesktop(workflow){
 const {native,build,cli}=workflow.jobs;for(const job of [native,build,cli])cap(job);
 assert.deepEqual(native.strategy.matrix.include.map(row=>row.target).sort(),[...nativeTargets].sort());
 assert.deepEqual(build.strategy.matrix.include.map(row=>row.label).sort(),[...packageLabels].sort());
 assert.deepEqual(build.strategy.matrix.include.filter(row=>row.label==='macos-x64'||row.label.startsWith('windows-')).map(row=>row.label).sort(),['macos-x64','windows-arm64','windows-x64']);
 assert.deepEqual(native.needs,['native-source']);
 const toolchains=native.steps.filter(value=>String(value.uses).startsWith('dtolnay/rust-toolchain@'));
 assert.equal(toolchains.length,1);assert.equal(toolchains[0].with.toolchain,'${{ needs.native-source.outputs.toolchain }}');
 assert.equal(toolchains[0].with.targets,'${{ matrix.target }}');
 const compile=step(native,'Build the pinned default-feature native executable');
 assert.equal(compile.run.trim(),'cargo build --manifest-path rust/Cargo.toml --target-dir target --release --bin formal-ai --locked --target ${{ matrix.target }}');
 assert.equal(native.steps.filter(value=>/\bcargo build /u.test(value.run??'')).length,1);
 order(native,['Import and verify exact source selection','Build the pinned default-feature native executable','Record exact native executable receipt','Verify executable bytes and actual startup']);
 assert.match(step(native,'Record exact native executable receipt').run,/native-release-artifact\.mjs" write native-bin native-source/u);
 assert.equal(step(native,'Verify executable bytes and actual startup').run.trim(),'node "$FORMAL_AI_NATIVE_PROTOCOL_DIR/native-release-artifact.mjs" verify native-bin native-source "${{ matrix.target }}"');
 const binary=upload(native,'native-release-${{ matrix.target }}');assert.equal(binary.with.path,'native-bin/');
 assert.ok(native.steps.indexOf(binary)>index(native,'Verify executable bytes and actual startup'));
 for(const job of [build,cli]){
  assert.ok(job.needs.includes('native-source')&&job.needs.includes('native'));
  assert.ok(job.steps.every(value=>!(/\bcargo build /u.test(value.run??''))));
  download(job,'native-release-source');const target=download(job,'native-release-${{ matrix.target }}');
  assert.equal(target.with.path,'native-bin');
  const imported=step(job,'Import and verify exact source selection');
  assert.equal(imported.env.NATIVE_SELECTED_HEAD,'${{ needs.native-source.outputs.selected-head }}');
  assert.equal(imported.env.NATIVE_SELECTION_SHA256,'${{ needs.native-source.outputs.selection-sha256 }}');
  assert.match(imported.run,/native-release-source\.mjs" import native-source/u);
  const verified=step(job,'Verify native receipt, bytes and startup before packaging');
  assert.equal(verified.env.NATIVE_SELECTION_SHA256,'${{ needs.native-source.outputs.selection-sha256 }}');
  assert.equal(verified.run.trim(),'node "$FORMAL_AI_NATIVE_PROTOCOL_DIR/native-release-artifact.mjs" verify native-bin native-source "${{ matrix.target }}"');
  assert.ok(job.steps.indexOf(target)>job.steps.indexOf(imported));assert.ok(job.steps.indexOf(verified)>job.steps.indexOf(target));
 }
 order(build,['Import and verify exact source selection','Download the same-run target executable','Verify native receipt, bytes and startup before packaging','Prepare desktop resources (web + bundled binary + version sync)']);
 const prepare=step(build,'Prepare desktop resources (web + bundled binary + version sync)');
 assert.equal(prepare.env.FORMAL_AI_DESKTOP_REQUIRE_BINARY,'true');assert.equal(prepare.env.FORMAL_AI_DESKTOP_BINARY,'${{ env.FORMAL_AI_VERIFIED_NATIVE_BINARY }}');
 assert.equal(prepare.run.trim(),'node scripts/prepare-resources.mjs');
 for(const name of ['Test command-stream production adapter','Package desktop app (Linux/Windows)','Package desktop app (macOS signed)',
  'Package desktop app (macOS ad-hoc)','Smoke test macOS release artifacts','Smoke test Linux/Windows release artifacts','Collect artifacts and checksums','Attest build provenance','Upload assets to release'])step(build,name);
 order(cli,['Verify native receipt, bytes and startup before packaging','Package the CLI archive','Smoke test CLI archive','Collect CLI checksum fragment','Attest build provenance','Upload CLI archive to release']);
}
function checkBrowserCoverage(workflow) {
 const producer=workflow.jobs['browser-coverage-shard'],collector=workflow.jobs['browser-coverage'];
 assert.equal(producer['timeout-minutes'],15);assert.equal(collector['timeout-minutes'],15);
 assert.equal(producer.strategy['fail-fast'],false);assert.deepEqual(producer.strategy.matrix.shard,[1,2,3,4,5,6]);
 assert.deepEqual(producer.needs,['detect-changes']);assert.deepEqual(collector.needs,['detect-changes','browser-coverage-shard']);
 for(const job of [producer,collector]) {
  assert.ok(job.if.includes('!cancelled()'));assert.ok(job.if.includes("github.event_name == 'workflow_dispatch'"));
  for(const output of ['any-code-changed','workflow-changed'])assert.ok(job.if.includes(`needs.detect-changes.outputs.${output} == 'true'`));
 }
 const installer=step(producer,'Setup the pinned web dependency installer');assert.equal(installer.uses,'oven-sh/setup-bun@v2');assert.equal(installer.with['bun-version-file'],'.bun-version');
 assert.equal(step(producer,'Install actual locked UI test dependencies').run,'bun install --frozen-lockfile --ignore-scripts');
 const measure=step(producer,'Measure the complete assigned browser test inventory');
 assert.equal(measure.run.trim(),'npm run coverage:web -- run ${{ matrix.shard }} coverage/browser-shard-${{ matrix.shard }}');
 const receipt=upload(producer,'browser-coverage-shard-${{ matrix.shard }}');assert.equal(receipt.if,'always()');assert.equal(receipt.with.path,'coverage/browser-shard-${{ matrix.shard }}/');
 order(producer,['Install actual locked UI test dependencies','Measure the complete assigned browser test inventory','Retain exact browser coverage and completion receipts']);
 const complete=step(collector,'Every browser shard measured its complete test slice');
 assert.equal(complete.run,'bash scripts/check-shard-results.sh');assert.equal(complete.env.JOBS,'browser-coverage-shard');assert.equal(complete.env.NEEDS_JSON,'${{ toJSON(needs) }}');
 const download=step(collector,'Download every browser coverage shard');assert.ok(String(download.uses).startsWith('actions/download-artifact@'));
 assert.equal(download.with.pattern,'browser-coverage-shard-*');assert.equal(download.with['merge-multiple'],false);assert.equal(download.with.path,'coverage/browser-shards');
 assert.equal(step(collector,'Collect the complete source-bound browser coverage').run,'npm run coverage:web -- collect coverage/browser-shards');
 assert.equal(step(collector,'Check browser coverage ratchet').run,'rust-script scripts/check-coverage-ratchet.rs --only browser');
 order(collector,['Every browser shard measured its complete test slice','Download every browser coverage shard','Collect the complete source-bound browser coverage','Check browser coverage ratchet']);
}
function checkCoverage(workflow){
 const originalNames=['detect-changes','coverage-build','coverage-shard','coverage','browser-coverage'];
 const browserProducer='browser-coverage-shard';
 assert.deepEqual(Object.keys(workflow.jobs).filter(name=>name!==browserProducer),originalNames);
 const names=[...originalNames.slice(0,-1),browserProducer,originalNames.at(-1)];assert.deepEqual(Object.keys(workflow.jobs),names);
 for(const name of names)cap(workflow.jobs[name]);checkBrowserCoverage(workflow);
 const producer=workflow.jobs['coverage-build'],shard=workflow.jobs['coverage-shard'],reducer=workflow.jobs.coverage;
 assert.equal(producer['timeout-minutes'],30);
 const compiled=step(producer,'Build the instrumented test executables');assert.equal(compiled.env.TEST_BUDGET_SECONDS,1260);
 assert.ok(compiled.env.TEST_BUDGET_SECONDS*100<=producer['timeout-minutes']*60*70);
 for(const operand of ['cargo llvm-cov show-env --sh','cargo llvm-cov show-env --export-prefix','scripts/run-with-budget-warning.sh',
  'cargo test --manifest-path rust/Cargo.toml --all-features --no-run','--message-format=json-render-diagnostics > coverage-build.json'])assert.ok(compiled.run.includes(operand),operand);
 const archiver=producer.steps.find(value=>value.run?.includes('tar -cf coverage-objects.tar'));assert.ok(archiver);
 for(const operand of ['coverage-build.json','coverage-tests.tsv','coverage-objects.txt','-perm -u+x','tar -cf coverage-objects.tar coverage-tests.tsv -T coverage-objects.txt'])assert.ok(archiver.run.includes(operand),operand);
 const archive=upload(producer,'coverage-objects');assert.equal(archive.with.path,'coverage-objects.tar');
 assert.ok(producer.steps.indexOf(compiled)<producer.steps.indexOf(archiver)&&producer.steps.indexOf(archiver)<producer.steps.indexOf(archive));
 assert.deepEqual(shard.needs,['coverage-build']);download(shard,'coverage-objects');
 assert.equal(step(shard,'Unpack the instrumented executables').run,'tar -xf coverage-objects/coverage-objects.tar');
 step(shard,'Run the instrumented coverage shard');
 const profileUploads=shard.steps.filter(value=>String(value.uses).startsWith('actions/upload-artifact@')&&value.with?.name?.startsWith('coverage-profile-'));
 assert.equal(profileUploads.length,1);assert.equal(profileUploads[0].with['if-no-files-found'],'error');
 assert.deepEqual(reducer.needs,['detect-changes','coverage-build','coverage-shard']);
 assert.equal(step(reducer,'Every coverage shard measured its tests').run,'bash scripts/check-shard-results.sh');
 download(reducer,'coverage-objects');
 const profiles=step(reducer,"Download every shard's profile");assert.equal(profiles.with.pattern,'coverage-profile-*');assert.equal(profiles.with['merge-multiple'],true);
 const report=step(reducer,'Report merged Rust coverage');
 for(const operand of ['[ -s "coverage-profiles/shard-$index-of-$total.profdata" ]','tar -xf coverage-objects/coverage-objects.tar','merge -sparse','cargo llvm-cov report --manifest-path rust/Cargo.toml --lcov --output-path lcov.info'])assert.ok(report.run.includes(operand),operand);
 assert.equal(step(reducer,'Check Rust coverage ratchet').run,'rust-script scripts/check-coverage-ratchet.rs --only rust');
 upload(reducer,'rust-lcov');
 for(const name of ['coverage-build','coverage','browser-coverage']){
  assert.ok(workflow.jobs[name].if.includes("github.event_name == 'workflow_dispatch'"));
  for(const output of name === 'browser-coverage' ? ['any-code-changed','workflow-changed'] : ['rs-changed','toml-changed']) assert.ok(workflow.jobs[name].if.includes(`needs.detect-changes.outputs.${output} == 'true'`));
 }
}
test('current native/package sources retain all relocated operations under declared thirty-minute caps',()=>checkDesktop(desktop));
test('current coverage sources retain the complete instrumented suite and reduction obligations under bounded stage clocks',()=>checkCoverage(coverage));
const desktopRefusals=[
 ['missing original heavy target',w=>{w.jobs.build.strategy.matrix.include=w.jobs.build.strategy.matrix.include.filter(row=>row.label!=='windows-arm64');}],
 ['unbounded native cap',w=>{w.jobs.native['timeout-minutes']=31;}],
 ['unbounded package cap',w=>{w.jobs.build.strategy.matrix.include[0].capmin=31;}],
 ['wrong compiler source',w=>{w.jobs.native.steps.find(s=>String(s.uses).startsWith('dtolnay/rust-toolchain@')).with.toolchain='unproved';}],
 ['compile default features dropped',w=>{step(w.jobs.native,'Build the pinned default-feature native executable').run+=' --no-default-features';}],
 ['missing native receipt',w=>{w.jobs.native.steps=w.jobs.native.steps.filter(s=>s.name!=='Record exact native executable receipt');}],
 ['native verification after upload',w=>{const s=step(w.jobs.native,'Verify executable bytes and actual startup');w.jobs.native.steps=w.jobs.native.steps.filter(v=>v!==s);w.jobs.native.steps.push(s);}],
 ['spoofed target download',w=>{download(w.jobs.build,'native-release-${{ matrix.target }}').with.name='unproved-target';}],
 ['source drift selection digest',w=>{step(w.jobs.build,'Import and verify exact source selection').env.NATIVE_SELECTION_SHA256='unproved';}],
 ['unqualified required binary',w=>{step(w.jobs.build,'Prepare desktop resources (web + bundled binary + version sync)').env.FORMAL_AI_DESKTOP_REQUIRE_BINARY='false';}],
 ['duplicate package compilation',w=>{w.jobs.build.steps.push({name:'Unproved build',run:'cargo build --release'});}],
 ['missing CLI smoke',w=>{w.jobs.cli.steps=w.jobs.cli.steps.filter(s=>s.name!=='Smoke test CLI archive');}]
];
for(const [name,mutate]of desktopRefusals)test('refuse changed relocated native/package source: '+name,()=>{const changed=structuredClone(desktop);mutate(changed);assert.throws(()=>checkDesktop(changed));});
const coverageRefusals=[
 ['unbounded producer cap',w=>{w.jobs['coverage-build']['timeout-minutes']=40;}],
 ['budget precedes job failure clock',w=>{step(w.jobs['coverage-build'],'Build the instrumented test executables').env.TEST_BUDGET_SECONDS=1261;}],
 ['native feature coverage weakened',w=>{step(w.jobs['coverage-build'],'Build the instrumented test executables').run=step(w.jobs['coverage-build'],'Build the instrumented test executables').run.replace('--all-features','');}],
 ['executable inventory missing',w=>{w.jobs['coverage-build'].steps=w.jobs['coverage-build'].steps.filter(s=>!s.run?.includes('tar -cf coverage-objects.tar'));}],
 ['wrong executable transfer',w=>{download(w.jobs['coverage-shard'],'coverage-objects').with.name='unproved';}],
 ['profile missing refusal removed',w=>{const s=step(w.jobs.coverage,'Report merged Rust coverage');s.run=s.run.replace('[ -s "coverage-profiles/shard-$index-of-$total.profdata" ]','true');}],
 ['coverage ratchet removed',w=>{w.jobs.coverage.steps=w.jobs.coverage.steps.filter(s=>s.name!=='Check Rust coverage ratchet');}],
 ['change gating weakened',w=>{w.jobs['coverage-build'].if='true';}]
];
for(const [name,mutate]of coverageRefusals)test('refuse changed instrumented coverage source: '+name,()=>{const changed=structuredClone(coverage);mutate(changed);assert.throws(()=>checkCoverage(changed));});

const browserCoverageRefusals=[
 ['missing original coverage job',w=>{delete w.jobs['coverage-shard'];}],
 ['foreign coverage job',w=>{w.jobs.unproved={};}],
 ['missing browser producer',w=>{delete w.jobs['browser-coverage-shard'];}],
 ['incomplete browser shard inventory',w=>{w.jobs['browser-coverage-shard'].strategy.matrix.shard.pop();}],
 ['duplicate browser shard',w=>{w.jobs['browser-coverage-shard'].strategy.matrix.shard[5]=5;}],
 ['browser fail-fast cancellation',w=>{w.jobs['browser-coverage-shard'].strategy['fail-fast']=true;}],
 ['missing locked producer bootstrap',w=>{step(w.jobs['browser-coverage-shard'],'Install actual locked UI test dependencies').run='bun install';}],
 ['missing raw receipt refusal',w=>{upload(w.jobs['browser-coverage-shard'],'browser-coverage-shard-${{ matrix.shard }}').with['if-no-files-found']='warn';}],
 ['missing required collector dependency',w=>{w.jobs['browser-coverage'].needs=['detect-changes'];}],
 ['collector completion guard skipped',w=>{step(w.jobs['browser-coverage'],'Every browser shard measured its complete test slice').run='true';}],
 ['collector receipt folders flattened',w=>{step(w.jobs['browser-coverage'],'Download every browser coverage shard').with['merge-multiple']=true;}],
 ['collector runs a new measurement',w=>{step(w.jobs['browser-coverage'],'Collect the complete source-bound browser coverage').run='npm run coverage:web';}],
 ['browser ratchet waived',w=>{step(w.jobs['browser-coverage'],'Check browser coverage ratchet').run='true';}],
 ['browser change gating waived',w=>{w.jobs['browser-coverage-shard'].if='true';}]
];
for(const [name,mutate]of browserCoverageRefusals)test('refuse incomplete source-bound browser coverage: '+name,()=>{const changed=structuredClone(coverage);mutate(changed);assert.throws(()=>checkCoverage(changed));});

function checkAttestations(workflow){
 const expected={build:'desktop/release/formal-ai-desktop-*\ndesktop/release/latest*.yml',cli:'formal-ai-cli-${{ matrix.target }}.${{ matrix.archive }}',vscode:'vscode/formal-ai-vscode-*.vsix',finalize:'release-evidence/formal-ai-*.json'};
 const all=Object.entries(workflow.jobs).flatMap(([job,value])=>(value.steps??[]).filter(step=>String(step.uses??'').includes('actions/attest')).map(step=>({job,step})));
 assert.equal(all.length,4);
 for(const [job,subject] of Object.entries(expected)){
  const matches=all.filter(item=>item.job===job);assert.equal(matches.length,1);
  const step=matches[0].step;assert.equal(step.uses,'actions/attest@v4');assert.equal(step.with['subject-path'].trim(),subject);assert.equal('subject-checksums' in step.with,false);
 }
}
test('all original artifact attestations and durable JSON evidence remain bound to their actual producers',()=>checkAttestations(desktop));
for(const job of ['build','cli','vscode','finalize']){
 test('refuse missing attestation producer '+job,()=>{const changed=structuredClone(desktop);changed.jobs[job].steps=changed.jobs[job].steps.filter(step=>step.uses!=='actions/attest@v4');assert.throws(()=>checkAttestations(changed));});
 test('refuse spoofed attestation subject '+job,()=>{const changed=structuredClone(desktop);changed.jobs[job].steps.find(step=>step.uses==='actions/attest@v4').with['subject-path']='unbound/*';assert.throws(()=>checkAttestations(changed));});
}
test('refuse legacy attestation action',()=>{const changed=structuredClone(desktop);changed.jobs.build.steps.find(step=>step.uses==='actions/attest@v4').uses='actions/attest-build-provenance@v2';assert.throws(()=>checkAttestations(changed));});
test('refuse cross-platform checksum text parsing',()=>{const changed=structuredClone(desktop);changed.jobs.build.steps.find(step=>step.uses==='actions/attest@v4').with['subject-checksums']='SHA256SUMS.txt';assert.throws(()=>checkAttestations(changed));});
