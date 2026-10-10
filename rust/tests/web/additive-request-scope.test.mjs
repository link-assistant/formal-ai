import {test,before} from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {hasAdditivePosition} from '../../../js/agentic/planner/owned_goals.mjs';
import {ownsAdditiveScope} from '../../../js/agentic/workspace_change/additive_scope.mjs';
before(async()=>{await installNodeHost(new WorkerHost());});
const refusals=[
 'Do not write. Append the line "third" to notes.txt.',
 'Do not read. Append the line "third" to notes.txt.',
 'Append the line "third" to notes.txt. Deploy it.',
 'Append the line "third" to notes.txt. Do something unknown.',
 'Append the line "third" to notes.txt. Include it.',
 'Append the line "third" to notes.txt and include it.'
];
for(const request of refusals) test('additive effects preserve the entire request: '+request,async()=>{
 assert.equal(hasAdditivePosition(request),true);
 assert.equal(ownsAdditiveScope(request),false);
 const plan=await planChatStep([{role:'user',content:request}],['read','write','edit','bash']);
 assert.equal(plan.kind,'final');
});
for(const request of [
 'Append the line "third" to notes.txt.',
 'Prepend the line "first" to renamed.txt.',
 'Append the line "Deploy it. Do not read." to notes.txt.',
 'Append an empty line to notes.txt.'
]) test('bound additive payload stays data: '+request,()=>{
 assert.equal(hasAdditivePosition(request),true);
 assert.equal(ownsAdditiveScope(request),true);
});
test('position words in complete created content do not claim insertion',()=>{
 assert.equal(hasAdditivePosition('Create file note.txt containing "append prepend"'),false);
});
