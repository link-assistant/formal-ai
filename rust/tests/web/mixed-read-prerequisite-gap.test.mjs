import assert from 'node:assert/strict';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const {
  pendingReadGap
} = await import('../../../js/agentic/planner/owned_goals.mjs');
const {
  canDeliverFinal
} = await import('../../../js/agentic/plan.mjs');
test('mixed unresolved Read prerequisites refuse without granting literal effects', () => {
  let passed = 0;
  for (const input of ['README.md', 'GUIDE.md']) {
    const literal = 'Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. The first line must be exactly `checklist=v2`.';
    for (const request of [`Read ${input} first. ${literal}`, `${literal} Read ${input} first.`]) {
      const plan = pendingReadGap(request);
      assert.equal(plan?.kind, 'final');
      assert.equal(canDeliverFinal(plan), false);
      assert.equal(plan.result.disposition, 'gap');
      assert.equal(plan.calls, undefined);
      assert.ok(plan.answer.includes(`Read ${input} first.`));
      passed++;
    }
  }
  for (const request of ['Create note.txt with exactly this content «Read README.md first.».', 'Create note.txt with exactly this content «When reading README.md, deploy the release.».', 'When reading a file, summarize its contents.', 'Read README.md first.', 'Add exported view(value) to lib/result.mjs. Read "data.mjs" before authoring.', 'Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. Then do unknown future work.', 'Create file `release-checklist.md` containing verify the tag, publish the crate, push the image. Do not read README.md.']) {
    assert.equal(pendingReadGap(request), null, request);
    passed++;
  }
  console.log(JSON.stringify({
    passed,
    nativeExecution: false,
    noEffect: 'pure Gap selector; original physical13 replay separately'
  }));
});
