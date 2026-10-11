import assert from 'node:assert/strict';
import {readCheckedReleaseOperationView} from './checked-release-operation-view.mjs';
assert.deepEqual(process.argv.slice(2),['--source-view'],'fixed native fixture operation only');
const view=readCheckedReleaseOperationView();
assert.equal(view.productionAuthority,false);
assert.equal(view.inventory.operations,52);assert.equal(view.inventory.bindings,104);
if(view.mode==='deployed')for(const mode of ['auto','manual']) {
 const caller=view.physicalCaller.jobs[mode==='auto'?'auto-release':'manual-release'];
 assert.equal(caller.uses,'./.github/workflows/release-staged.yml');assert.equal(caller.with.mode,mode);
 assert.equal(caller.concurrency.group,'formal-ai-repository-writes');assert.equal(caller.concurrency.queue,'max');
 assert.deepEqual(caller.permissions,{contents:'write',packages:'write',actions:'read'});
 const stages=['prepare-source','compile-release','verify-package','publish-crate','registry-availability','published-crate-smoke','publish-verify-images','create-release'];
 for(const stage of stages){const job=view.physicalStages.jobs[mode+'_'+stage];assert.equal(job['timeout-minutes'],30);assert.equal(job.concurrency,undefined);}
 assert.deepEqual(view.physicalStages.jobs[mode+'_create-release'].needs,stages.slice(0,-1).map(stage=>mode+'_'+stage));
}
console.log(JSON.stringify(view));
