// Production workflow target and consumer invariants complement actual receipt corruption tests.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
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
 assert.match(native,/timeout-minutes: 30/u);assert.match(native,/toolchain: \$\{\{ needs\.native-source\.outputs\.toolchain \}\}/u);
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
  assert.match(body,/ref: \$\{\{ github\.workflow_sha \}\}/u);
  assert.match(body,/NATIVE_PROTOCOL_COMMIT: \$\{\{ github\.workflow_sha \}\}/u);
  assert.match(body,/cp scripts\/native-release-source\.mjs scripts\/native-release-artifact\.mjs/u);
  const capture=body.indexOf('Capture release protocol');
  const checkout=body.indexOf('ref: ${{ github.sha }}',capture);
  assert.ok(capture>=0&&checkout>capture,'captured protocol must precede event-pinned source checkout');
  assert.ok(body.includes('FORMAL_AI_NATIVE_PROTOCOL_DIR'));
 }
});
