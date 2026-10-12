// Verify the actual archive/consumer profile receipt commands without compiling Rust.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,mkdtempSync,mkdirSync,rmSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
const workflow=readFileSync(process.env.FORMAL_AI_MACOS_WORKFLOW_DRAFT??'.github/workflows/macos-core-tests.yml','utf8');
const production=workflow.slice(workflow.indexOf('  build-archive:'),workflow.indexOf('  test-archive:'));
const consumption=workflow.slice(workflow.indexOf('  test-archive:'));
const profileCommands=[...workflow.matchAll(/printf 'profile test[\s\S]+?> "\$RUNNER_TEMP\/macos-core-tests\/(?:expected-)?profile\.lino"/gu)].map(m=>m[0]);
const settings={CARGO_PROFILE_TEST_OPT_LEVEL:'1',CARGO_PROFILE_TEST_DEBUG:'0',CARGO_PROFILE_TEST_DEBUG_ASSERTIONS:'true',CARGO_PROFILE_TEST_OVERFLOW_CHECKS:'true',CARGO_PROFILE_TEST_INCREMENTAL:'false',RUSTFLAGS:'-Dwarnings'};
test('platform compilation retains every integration target, source filter and runtime assertion',()=>{
 for(const [key,value] of Object.entries(settings))if(key!=='RUSTFLAGS')assert.ok(workflow.includes(`  ${key}: '${value}'`),key);
 assert.match(production,/archive --tests --all-features/u);
 assert.doesNotMatch(production,/archive[^\n]*(?:--lib|--bins|--release|--no-default-features)/u);
 assert.match(production,/TEST_BUDGET_SECONDS: 1800/u);assert.match(production,/TEST_BUDGET_SECONDS: 400/u);
 assert.match(consumption,/TEST_BUDGET_SECONDS: 600/u);
 assert.match(consumption,/--workspace-remap "\$GITHUB_WORKSPACE\/rust"/u);
 assert.match(consumption,/-E "\(\$PLATFORM\)/u);
 assert.equal(profileCommands.length,2);
 assert.ok(consumption.indexOf('cmp "$RUNNER_TEMP/macos-core-tests/profile.lino"')<consumption.indexOf('cargo nextest'));
 const cargo=readFileSync('rust/Cargo.toml','utf8').split('[profile.test]')[1].split('\n[')[0];assert.match(cargo,/opt-level = 2/u);
 const source=readFileSync('rust/tests/source/solver_handlers/installation_conversion.rs','utf8');assert.match(source,/source_tests\/solver_handlers\/installation_conversion\/tests.rs/u);
 const cases=readFileSync('rust/tests/source/source_tests/solver_handlers/installation_conversion/tests.rs','utf8');assert.equal((cases.match(/#\[test\]/gu)||[]).length,8);
});
test('actual profile stamp accepts identical contract and refuses each independent profile mutation',t=>{
 const root=mkdtempSync(join(tmpdir(),'macos-profile-receipt-'));t.after(()=>rmSync(root,{recursive:true,force:true}));mkdirSync(join(root,'macos-core-tests'));
 const run=(command,extra={})=>spawnSync('bash',['-e','-u','-o','pipefail','-c',command],{env:{...process.env,...settings,...extra,RUNNER_TEMP:root},encoding:'utf8'});
 assert.equal(run(profileCommands[0]).status,0);assert.equal(run(profileCommands[1]).status,0);
 const compare='cmp "$RUNNER_TEMP/macos-core-tests/profile.lino" "$RUNNER_TEMP/macos-core-tests/expected-profile.lino"';assert.equal(run(compare).status,0);
 for(const [key,value] of Object.entries(settings)){
  assert.equal(run(profileCommands[1],{[key]:value==='true'?'false':value+'-changed'}).status,0);
  assert.notEqual(run(compare).status,0,key);
 }
 const producer=readFileSync(join(root,'macos-core-tests/profile.lino'),'utf8');writeFileSync(join(root,'macos-core-tests/expected-profile.lino'),producer+'extra\n');assert.notEqual(run(compare).status,0);
});
