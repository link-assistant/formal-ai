import {test} from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {extractRequirements} from '../../../js/agentic/crate/requirement_extraction.mjs';
await installNodeHost(new WorkerHost());
test('bare acceptance headings retain declarative obligations and stop at peer headings',()=>{
  assert.deepEqual(extractRequirements('## Acceptance\nThe upload resumes after reconnect.\n### Offline mode\nThe queue survives restart.\n## Background\nThe service started last year.'),['The upload resumes after reconnect.','The queue survives restart.']);
});
test('acceptance words in ordinary prose do not open requirement sections',()=>{
  assert.deepEqual(extractRequirements('The team discussed acceptance yesterday.\nThe queue survives restart.'),[]);
});
