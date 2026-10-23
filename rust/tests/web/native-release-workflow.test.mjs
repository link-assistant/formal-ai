// Production workflow target and consumer invariants complement actual receipt corruption tests.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
import {parseSpeedPolicy} from '../../../scripts/lib/ci-speed-policy.mjs';
import {NATIVE_TARGETS} from '../../../scripts/native-release-artifact.mjs';
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

test('each packaging job captures the pinned workflow protocol before checking out binary source',()=>{
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
  assert.ok(capture>protocol&&checkout>capture,'pinned checkout and capture must precede event-pinned source checkout');
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
