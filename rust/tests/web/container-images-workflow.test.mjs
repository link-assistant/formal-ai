import {readCheckedReleaseOperationView} from '../../../scripts/checked-release-operation-view.mjs';
// Workflow contracts are static checks; runtime and cold-build acceptance require actual CI.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const workflow=readFileSync(new URL('../../../.github/workflows/container-images.yml',import.meta.url),'utf8');
function job(name) {
 const start=workflow.indexOf('\n  '+name+':\n');assert(start>=0,'actual workflow job missing '+name);
 const remainder=workflow.slice(start+1),next=/\n  [a-z][a-z0-9-]*:\n/u.exec(remainder);
 return next?remainder.slice(0,next.index):remainder;
}
test('release creation calls trusted container delivery without a completion trigger',()=>{
 const events=workflow.slice(0,workflow.indexOf('\npermissions:'));
 assert.match(events,/^  workflow_call:/mu);assert.match(events,/^  release:/mu);
 assert.doesNotMatch(events,/^  workflow_run:/mu);
 const release=readCheckedReleaseOperationView().originalSource;
 const publication=release.slice(release.indexOf('  native-container-images:'),release.indexOf('  pipeline-status:'));
 assert.match(publication,/needs: \[auto-release, manual-release\]/u);
 assert.match(publication,/needs\.auto-release\.result == 'success' && needs\.auto-release\.outputs\.container-tag != ''/u);
 assert.match(publication,/needs\.manual-release\.result == 'success' && needs\.manual-release\.outputs\.container-tag != ''/u);
 assert.match(publication,/uses: \.\/\.github\/workflows\/container-images\.yml/u);
 assert.ok(publication.includes('tag: ${{ needs.auto-release.outputs.container-tag || needs.manual-release.outputs.container-tag }}'));
 const resolver=job('resolve');assert.match(resolver,/resolvePackageRelease/u);
 assert.match(resolver,/publication cannot select an arbitrary custom source/u);
 assert.match(job('source'),/native-release-trust\.mjs/u);assert.match(job('source'),/env\.NATIVE_AUTHORIZED_RELEASE_COMMIT/u);
});
test('two same-run native targets feed every full and slim architecture without a static exclusion',()=>{
 const producer=job('native');assert.match(producer,/x86_64-unknown-linux-gnu/u);assert.match(producer,/aarch64-unknown-linux-gnu/u);
 assert.match(producer,/native-release-artifact\.mjs.* write/u);assert.match(producer,/native-release-artifact\.mjs.* verify/u);
 for(const name of ['validate-images','publish-images']) {
  const section=job(name);assert.equal((section.match(/arch: amd64/gu)??[]).length,2);assert.equal((section.match(/arch: arm64/gu)??[]).length,2);
  assert.equal((section.match(/variant: full/gu)??[]).length,2);assert.equal((section.match(/variant: slim/gu)??[]).length,2);
  assert.match(section,/container-native-/u);assert.match(section,/build-container-image\.sh/u);assert.doesNotMatch(section,/exclude:/u);
 }
});
test('PR validation has no registry write while publication retains independent source authority',()=>{
 const validation=job('validate-images'),publication=job('publish-images');
 assert.match(validation,/publish == 'false'/u);assert.match(validation,/packages: read/u);assert.doesNotMatch(validation,/docker\/login-action/u);
 assert.match(publication,/publish == 'true'/u);assert.match(publication,/github\.event_name != 'pull_request'/u);assert.match(publication,/github\.ref == 'refs\/heads\/main'/u);assert.match(publication,/packages: write/u);assert.match(publication,/docker\/login-action/u);
 for(const name of ['native','validate-images','publish-images','merge']) {
  const section=job(name);assert.match(section,/github\.workflow_sha/u);assert.match(section,/protocol differs/u);
  assert.match(section,/native-release-source\.mjs.* import/u);assert.match(section,/validateSelectedSourceAuthority/u);
 }
});
test('full source compilation has an independent path and every job keeps the required deadline',()=>{
 const independent=job('independent-full');assert.match(independent,/test ! -e rust\/target\/release\/formal-ai/u);
 assert.match(independent,/BINARY_SOURCE=compile/u);assert.match(independent,/verify-formal-ai-dind/u);
 assert.doesNotMatch(independent,/container-native-/u);assert.match(independent,/ubuntu-24\.04-arm/u);
 const caps=[...workflow.matchAll(/timeout-minutes: (\d+)/gu)].map(match=>Number(match[1]));
 assert(caps.length>=7);assert(caps.every(cap=>cap<=30));
 assert.match(job('merge'),/container-image-published-/u);assert.match(job('merge'),/container-image-receipt\.mjs.* merge/u);
 assert.doesNotMatch(workflow,/--amend/u);assert.doesNotMatch(workflow,/setup-qemu/u);
});

test('the release pipeline waits for native publication only after actual successful release creation',()=>{
 const release=readCheckedReleaseOperationView().originalSource;
 assert.equal((release.match(/id: create-release/gu)??[]).length,2);
 assert.equal((release.match(/container-tag:.*steps\.create-release\.outcome == 'success'/gu)??[]).length,2);
 assert.match(release,/native-container-images:\n[\s\S]*needs: \[auto-release, manual-release\]/u);
 assert.match(release,/uses: \.\/\.github\/workflows\/container-images\.yml/u);
 assert.match(release,/auto-release, manual-release, native-container-images,/u);
});
