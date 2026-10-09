import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const workflow=readFileSync(new URL('../../../.github/workflows/release.yml',import.meta.url),'utf8');
const cargo=readFileSync(new URL('../../Cargo.toml',import.meta.url),'utf8');
function job(name){const start=workflow.indexOf('\n  '+name+':\n');assert.ok(start>=0,name);const rest=workflow.slice(start+1);const end=rest.slice(1).search(/\n  [a-z][a-z-]*:\n/u);return end<0?rest:rest.slice(0,end+1);}
test('doctests prepare the exact Cargo doc graph by listing before unchanged execution',()=>{
 const body=job('doc-tests');
 const prepare='cargo test --manifest-path rust/Cargo.toml --doc --verbose -- --list';
 const execute='cargo test --manifest-path rust/Cargo.toml --doc --verbose';
 const commands=body.split('\n').map(line=>line.trim());
 assert.equal(commands.filter(line=>line===prepare).length,1);
 assert.equal(commands.filter(line=>line===execute).length,1);
 assert.ok(commands.indexOf(prepare)<commands.indexOf(execute));
 assert.doesNotMatch(body,/cargo build|--release|--all-features|--no-default-features/u);
 assert.doesNotMatch(body.slice(body.indexOf('- name: Run doc tests')),/--list|--ignored|--skip/u);
 assert.match(cargo,/\[profile\.test\][\s\S]*?opt-level = 2/u);
 assert.match(cargo,/\[profile\.test\][\s\S]*?debug-assertions = true[\s\S]*?overflow-checks = true/u);
 assert.match(body,/setup-sccache/u);assert.match(body,/cache-cargo-registry/u);
});
test('documentation preparation has its own deadline while real doctests retain120seconds',()=>{
 const body=job('doc-tests');
 assert.match(body,/timeout-minutes: 30/u);
 assert.match(body,/Prepare default-feature documentation dependencies[\s\S]*?TEST_BUDGET_SECONDS: 1080/u);
 assert.match(body,/name: Run doc tests\n        env:\n          TEST_BUDGET_SECONDS: 120/u);
 assert.equal((body.match(/scripts\/run-with-budget-warning\.sh/gu)??[]).length,2);
 assert.ok(1080+120<30*60);assert.ok(1080/(30*60)<=0.7);
 assert.doesNotMatch(body,/continue-on-error|already-green/u);
});
test('documentation source is the actual PRhead freshly merged with the pinned base in a read-only parallel job',()=>{
 const body=job('doc-tests');
 assert.match(body,/needs: \[detect-changes, base\]/u);
 assert.doesNotMatch(body,/build-artifacts/u);
 assert.match(body,/permissions:\n      contents: read/u);
 assert.match(body,/if: github\.event_name == 'pull_request'[\s\S]*?ref: \$\{\{ github\.event\.pull_request\.head\.sha \}\}/u);
 assert.match(body,/if: github\.event_name != 'pull_request'[\s\S]*?ref: \$\{\{ github\.sha \}\}/u);
 assert.match(body,/BASE_COMMIT: \$\{\{ needs\.base\.outputs\.commit \}\}/u);
 assert.equal((body.match(/simulate-fresh-merge\.sh/gu)??[]).length,1);
 assert.ok(body.indexOf('simulate-fresh-merge.sh')<body.indexOf('cargo test'));
});
test('five full native shards retain all their tests without compiling documentation again',()=>{
 const body=job('test');
 assert.equal((body.match(/test-suite: full/gu)??[]).length,5);
 assert.match(body,/SHARD_RESERVED_SECONDS: 1=330\n/u);
 assert.doesNotMatch(body,/2=110|cargo test[^\n]*--doc/u);
 assert.match(body,/bash scripts\/run-prebuilt-tests\.sh/u);
 assert.match(body,/name: formal-ai-test-executables/u);
});
